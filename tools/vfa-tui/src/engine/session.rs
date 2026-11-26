//! Explicit checkout identity and session coordination, not execution authority.
//!
//! Native identity is currently implemented on Unix only. Other platforms fail
//! explicitly instead of substituting path strings, sizes or timestamps.

use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::snapshot::CatalogSnapshot;
use crate::persistence::operations::OperationStatus;

/// Both causes use identical snapshot loading; watcher ownership is explicit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshTrigger {
    Manual,
    ExternalChange,
}

pub const MAX_SESSION_OPERATIONS: usize = 32;

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("select an explicit absolute checkout path")]
    AbsolutePathRequired,
    #[error("native checkout identity is unavailable on this platform")]
    IdentityUnavailable,
    #[error("checkout unavailable at {path}: {reason}")]
    SourceUnavailable { path: PathBuf, reason: &'static str },
    #[error("selected checkout no longer names the retained native directory")]
    SourceChanged,
    #[error("nonterminal operations prevent source replacement")]
    Busy,
    #[error("select a checkout before registering an operation")]
    NoSource,
    #[error("unknown operation in this session")]
    UnknownOperation,
    #[error("invalid operation state transition")]
    InvalidTransition,
    #[error("only terminal operations can be released")]
    OperationNotTerminal,
    #[error("session operation retention budget is full")]
    OperationLimit,
    #[error("source epoch space is exhausted")]
    EpochExhausted,
    #[error("operation identity collision; registration refused")]
    IdConflict,
}

/// Session-scoped source epoch. Future presentation/trust/preview adapters must
/// reject tags that are no longer current; this is not an execution permit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextTag {
    session_id: Uuid,
    epoch: u64,
    source: SourceIdentity,
}

impl ContextTag {
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    pub fn source(&self) -> &SourceIdentity {
        &self.source
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OperationId(Uuid);

#[derive(Debug)]
pub struct SessionOperation {
    context: ContextTag,
    status: OperationStatus,
}

impl SessionOperation {
    pub fn context(&self) -> &ContextTag {
        &self.context
    }
    pub fn status(&self) -> OperationStatus {
        self.status
    }
}

/// Serialized engine model. Registering/changing state does not launch work,
/// authorize effects, prove cleanup or commit evidence. The future controller
/// must supply those observations. Dropping an observer changes no owned state.
pub struct SourceSession {
    id: Uuid,
    epoch: u64,
    snapshot: Option<Arc<CatalogSnapshot>>,
    operations: BTreeMap<OperationId, SessionOperation>,
}

impl Default for SourceSession {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            epoch: 0,
            snapshot: None,
            operations: BTreeMap::new(),
        }
    }
}

impl SourceSession {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a candidate completely before replacing the current owned context.
    /// Selecting an alias of the same identity is a no-op, not manual refresh.
    pub fn select(&mut self, selected: &Path) -> Result<Arc<CatalogSnapshot>, SessionError> {
        if self.operations.values().any(|op| !is_terminal(op.status)) {
            return Err(SessionError::Busy);
        }
        let source = SourceIdentity::open(selected)?;
        if let Some(current) = &self.snapshot {
            if current.context().source == source {
                return Ok(Arc::clone(current));
            }
        }
        let epoch = self
            .epoch
            .checked_add(1)
            .ok_or(SessionError::EpochExhausted)?;
        let context = ContextTag {
            session_id: self.id,
            epoch,
            source,
        };
        let candidate = Arc::new(CatalogSnapshot::load(context)?);
        self.snapshot = Some(Arc::clone(&candidate));
        self.epoch = epoch;
        Ok(candidate)
    }

    /// Explicitly re-observe the selected checkout. A source-level failure is
    /// returned after publishing retained stale data; collection failures are
    /// recorded independently in the returned snapshot. Never invokes helpers.
    pub fn refresh(
        &mut self,
        _trigger: RefreshTrigger,
    ) -> Result<Arc<CatalogSnapshot>, SessionError> {
        let previous = self.snapshot.as_ref().ok_or(SessionError::NoSource)?;
        match CatalogSnapshot::refresh(previous) {
            Ok(snapshot) => {
                let refreshed = Arc::new(snapshot);
                self.snapshot = Some(Arc::clone(&refreshed));
                Ok(refreshed)
            }
            Err(error) => {
                // Even source-level failure invalidates pending observations;
                // callers receive the error AND can inspect honest retained data.
                self.snapshot = Some(Arc::new(CatalogSnapshot::source_failed(previous, &error)));
                Err(error)
            }
        }
    }

    /// Apply only notifications captured for this exact session/source epoch.
    /// Old generations must not re-observe or invalidate the selected source.
    pub fn refresh_external(
        &mut self,
        context: &ContextTag,
    ) -> Result<Option<Arc<CatalogSnapshot>>, SessionError> {
        if !self.is_current(context) {
            return Ok(None);
        }
        self.refresh(RefreshTrigger::ExternalChange).map(Some)
    }

    pub fn snapshot(&self) -> Option<Arc<CatalogSnapshot>> {
        self.snapshot.as_ref().map(Arc::clone)
    }
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    pub fn is_current(&self, context: &ContextTag) -> bool {
        self.snapshot
            .as_ref()
            .is_some_and(|s| s.context() == context)
    }

    pub fn register_operation(&mut self) -> Result<OperationId, SessionError> {
        let snapshot = self.snapshot.as_ref().ok_or(SessionError::NoSource)?;
        if self.operations.len() >= MAX_SESSION_OPERATIONS {
            return Err(SessionError::OperationLimit);
        }
        snapshot.context().source.verify_current()?;
        let id = OperationId(Uuid::new_v4());
        if self.operations.contains_key(&id) {
            return Err(SessionError::IdConflict);
        }
        self.operations.insert(
            id,
            SessionOperation {
                context: snapshot.context().clone(),
                status: OperationStatus::Queued,
            },
        );
        Ok(id)
    }

    pub fn operation(&self, id: OperationId) -> Option<&SessionOperation> {
        self.operations.get(&id)
    }

    pub fn transition_operation(
        &mut self,
        id: OperationId,
        next: OperationStatus,
    ) -> Result<(), SessionError> {
        let operation = self
            .operations
            .get_mut(&id)
            .ok_or(SessionError::UnknownOperation)?;
        let current = operation.status;
        let valid = current == next
            || match current {
                OperationStatus::Queued => matches!(
                    next,
                    OperationStatus::Running
                        | OperationStatus::Failed
                        | OperationStatus::Cancelled
                        | OperationStatus::TimedOut
                        | OperationStatus::Interrupted
                ),
                OperationStatus::Running => {
                    next == OperationStatus::Cancelling || is_terminal(next)
                }
                OperationStatus::Cancelling => is_terminal(next),
                _ => false,
            };
        if !valid {
            return Err(SessionError::InvalidTransition);
        }
        operation.status = next;
        Ok(())
    }

    pub fn release_operation(&mut self, id: OperationId) -> Result<(), SessionError> {
        let operation = self
            .operations
            .get(&id)
            .ok_or(SessionError::UnknownOperation)?;
        if !is_terminal(operation.status) {
            return Err(SessionError::OperationNotTerminal);
        }
        self.operations.remove(&id);
        Ok(())
    }
}

/// An explicitly owned notification adapter around the shared source session.
/// No refresh task mutates shared state: the adapter polls `next_change` on its
/// own event loop. A source replacement drains/stops the old notification owner.
#[derive(Default)]
pub struct WatchedSourceSession {
    session: SourceSession,
    watcher: Option<crate::catalog::watcher::WatcherHandle>,
    events: Option<tokio::sync::mpsc::Receiver<crate::catalog::watcher::WatcherEvent>>,
    watch_context: Option<ContextTag>,
}

#[derive(Debug, thiserror::Error)]
pub enum WatchedSessionError {
    #[error(transparent)]
    Session(#[from] SessionError),
    #[error(transparent)]
    Watcher(#[from] crate::error::TuiError),
    #[error("watcher cleanup failed: {0}")]
    Shutdown(String),
}

impl WatchedSourceSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn session(&self) -> &SourceSession {
        &self.session
    }

    pub async fn select(
        &mut self,
        selected: &Path,
    ) -> Result<Arc<CatalogSnapshot>, WatchedSessionError> {
        // The existing selector performs all identity, busy and snapshot checks.
        // No events can be processed concurrently through this exclusive borrow.
        let snapshot = self.session.select(selected)?;
        if self.watch_context.as_ref() == Some(snapshot.context()) && self.watcher.is_some() {
            return Ok(snapshot);
        }
        self.shutdown().await?;
        let (watcher, events) =
            crate::catalog::watcher::spawn_watcher(snapshot.context().source().root(), None, &[])?;
        self.watcher = Some(watcher);
        self.events = Some(events);
        self.watch_context = Some(snapshot.context().clone());
        // Close the select/load-to-watch startup gap via the same refresh path.
        self.session
            .refresh(RefreshTrigger::ExternalChange)
            .map_err(Into::into)
    }

    pub fn refresh(&mut self) -> Result<Arc<CatalogSnapshot>, SessionError> {
        self.session.refresh(RefreshTrigger::Manual)
    }

    pub async fn next_change(&mut self) -> Result<Option<Arc<CatalogSnapshot>>, SessionError> {
        let Some(events) = self.events.as_mut() else {
            return Ok(None);
        };
        if events.recv().await.is_none() {
            return Ok(None);
        }
        let Some(context) = self.watch_context.as_ref() else {
            return Ok(None);
        };
        self.session.refresh_external(context)
    }

    /// Cancellation-safe: do not remove the owned handle until its join returns.
    pub async fn shutdown(&mut self) -> Result<(), WatchedSessionError> {
        if let Some(watcher) = self.watcher.as_mut() {
            watcher
                .shutdown_async()
                .await
                .map_err(WatchedSessionError::Shutdown)?;
        }
        self.watcher = None;
        self.events = None;
        self.watch_context = None;
        Ok(())
    }
}

fn is_terminal(status: OperationStatus) -> bool {
    !matches!(
        status,
        OperationStatus::Queued | OperationStatus::Running | OperationStatus::Cancelling
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeFileId {
    Unix { device: u64, inode: u64 },
}

/// Canonical native identity, retaining a directory handle through its lifetime.
#[derive(Debug, Clone)]
pub struct SourceIdentity {
    root: PathBuf,
    native: NativeFileId,
    directory: Arc<File>,
}

impl PartialEq for SourceIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root && self.native == other.native
    }
}

impl Eq for SourceIdentity {}

impl SourceIdentity {
    /// Validate only the selected root. Never search its parents or assume CWD.
    /// Package identity and a real catalog directory are root markers; a missing
    /// individual collection is reported by snapshot loading, not hidden here.
    pub fn open(selected: &Path) -> Result<Self, SessionError> {
        if !cfg!(unix) {
            return Err(SessionError::IdentityUnavailable);
        }
        if !selected.is_absolute() {
            return Err(SessionError::AbsolutePathRequired);
        }
        let root = fs::canonicalize(selected)
            .map_err(|_| unavailable(selected, "cannot resolve selected directory"))?;
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            // Reject non-directories before open can block on a FIFO. The
            // canonical final component must still be the real directory.
            options.custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        let directory = options
            .open(&root)
            .map_err(|_| unavailable(&root, "cannot open selected directory"))?;
        let metadata = directory
            .metadata()
            .map_err(|_| unavailable(&root, "cannot inspect selected directory"))?;
        if !metadata.is_dir() {
            return Err(unavailable(&root, "selected source is not a directory"));
        }
        let identity = Self {
            root,
            native: native_id(&metadata)?,
            directory: Arc::new(directory),
        };
        identity.verify_current()?;
        Ok(identity)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn native_id(&self) -> NativeFileId {
        self.native
    }

    /// Revalidate observed native identity/markers. This is not an atomic
    /// filesystem transaction or a defense against malicious same-user races.
    pub fn verify_current(&self) -> Result<(), SessionError> {
        let opened = self
            .directory
            .metadata()
            .map_err(|_| SessionError::SourceChanged)?;
        let named = fs::symlink_metadata(&self.root).map_err(|_| SessionError::SourceChanged)?;
        if !named.is_dir()
            || named.file_type().is_symlink()
            || native_id(&opened)? != self.native
            || native_id(&named)? != self.native
        {
            return Err(SessionError::SourceChanged);
        }
        #[derive(Deserialize)]
        struct PackageMarker {
            name: String,
        }
        let package = self.root.join("package.json");
        let bytes = read_regular_file(&package, 1024 * 1024)?;
        let marker: PackageMarker = serde_json::from_slice(&bytes)
            .map_err(|_| unavailable(&package, "invalid package identity"))?;
        if marker.name != "@raishin/vanguard-frontier-agentic" {
            return Err(unavailable(&package, "unexpected package identity"));
        }
        let catalog = self.root.join("catalog");
        let metadata = fs::symlink_metadata(&catalog)
            .map_err(|_| unavailable(&catalog, "catalog directory is missing"))?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(unavailable(&catalog, "catalog must be a real directory"));
        }
        // Detect an observed root replacement during marker reads as well.
        let named = fs::symlink_metadata(&self.root).map_err(|_| SessionError::SourceChanged)?;
        if named.file_type().is_symlink() || native_id(&named)? != self.native {
            return Err(SessionError::SourceChanged);
        }
        Ok(())
    }
}

pub(crate) fn read_regular_file(path: &Path, max: usize) -> Result<Vec<u8>, SessionError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| unavailable(path, "file is missing or unreadable"))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(unavailable(path, "expected a regular non-symlink file"));
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // A substituted FIFO must not block before its handle can be inspected.
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options
        .open(path)
        .map_err(|_| unavailable(path, "cannot open file"))?;
    let metadata = file
        .metadata()
        .map_err(|_| unavailable(path, "cannot inspect file"))?;
    if !metadata.is_file() || metadata.len() > max as u64 {
        return Err(unavailable(path, "file type or size limit is invalid"));
    }
    let mut bytes = Vec::new();
    file.take(max as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| unavailable(path, "cannot read file"))?;
    if bytes.len() > max {
        return Err(unavailable(path, "file exceeds size limit"));
    }
    Ok(bytes)
}

fn unavailable(path: &Path, reason: &'static str) -> SessionError {
    SessionError::SourceUnavailable {
        path: path.to_path_buf(),
        reason,
    }
}

#[cfg(unix)]
fn native_id(metadata: &fs::Metadata) -> Result<NativeFileId, SessionError> {
    use std::os::unix::fs::MetadataExt;
    Ok(NativeFileId::Unix {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

#[cfg(not(unix))]
fn native_id(_metadata: &fs::Metadata) -> Result<NativeFileId, SessionError> {
    Err(SessionError::IdentityUnavailable)
}
