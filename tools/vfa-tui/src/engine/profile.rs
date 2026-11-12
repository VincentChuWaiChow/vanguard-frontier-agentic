//! Desktop-only profile identity and cooperative, session-long writer ownership.
//!
//! This module never opens the legacy index, preferences, or operation journal.
//! Future writers must retain the owner (for example, in an `Arc<ProfileOwner>`)
//! before opening those files. OS locks exclude cooperating application writers;
//! they do not defend against a malicious process running as the same user.

use std::fs::{self, File, Metadata, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
#[cfg(windows)]
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("a private native data root is unavailable: {0}")]
    UnavailableDataRoot(&'static str),
    #[error("profile/data root must be absolute: {0:?}")]
    RelativeRoot(PathBuf),
    #[error("invalid private profile {path:?}: {reason}")]
    InvalidRoot { path: PathBuf, reason: &'static str },
    #[error("unsafe profile lock {path:?}: {reason}")]
    UnsafeLock { path: PathBuf, reason: &'static str },
    #[error("profile is already owned by another session: {root:?}")]
    Busy { root: PathBuf },
    #[error("cannot {operation} at {path:?}: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

/// Canonical native identity for one Desktop profile, not evidence of ownership.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfilePaths {
    root: PathBuf,
    journal: PathBuf,
    preferences: PathBuf,
    lock_file: PathBuf,
}

impl ProfilePaths {
    /// Establish only the selected private directory and canonicalize its identity.
    ///
    /// Linux deliberately requires an absolute `XDG_DATA_HOME` when no override
    /// is supplied. macOS/Windows use their native per-user data directories.
    /// No missing/invalid root is replaced with HOME, a temporary directory, or
    /// an in-memory profile. Existing directory permissions are never changed.
    /// On Windows, private ACL inheritance from the selected data root remains
    /// a platform admission requirement; this is not a general ACL auditor.
    pub fn resolve(explicit_root: Option<&Path>) -> Result<Self, ProfileError> {
        let selected = match explicit_root {
            Some(path) => path.to_path_buf(),
            None => native_data_root()?
                .join("vanguard-frontier-agentic")
                .join("desktop"),
        };
        if !selected.is_absolute() {
            return Err(ProfileError::RelativeRoot(selected));
        }
        // Remove trailing separators/dot components without interpreting `..`
        // ahead of the filesystem (which could change symlink semantics).
        let selected: PathBuf = selected.components().collect();
        ensure_directory(&selected)?;
        validate_root(&selected)?;
        let root = fs::canonicalize(&selected)
            .map_err(|error| path_io("canonicalize profile", &selected, error))?;
        validate_root(&root)?;
        Ok(Self {
            journal: root.join("operations.sqlite"),
            preferences: root.join("preferences.json"),
            lock_file: root.join(".owner.lock"),
            root,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn journal(&self) -> &Path {
        &self.journal
    }

    pub fn preferences(&self) -> &Path {
        &self.preferences
    }

    pub fn lock_file(&self) -> &Path {
        &self.lock_file
    }
}

/// Non-clonable owner: dropping the retained file releases the OS lock.
///
/// No PID/liveness inference, lock-file truncation, unlink, or replacement is
/// involved. `File` makes this guard `Send + Sync`; sharing the guard with `Arc`
/// retains the *same* ownership rather than opening another lock handle.
#[derive(Debug)]
pub struct ProfileOwner {
    paths: ProfilePaths,
    _lock: File,
}

impl ProfileOwner {
    pub fn acquire(paths: ProfilePaths) -> Result<Self, ProfileError> {
        validate_root(paths.root())?;
        match fs::symlink_metadata(paths.lock_file()) {
            Ok(metadata) => validate_lock(paths.lock_file(), &metadata)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(path_io("inspect profile lock", paths.lock_file(), error)),
        }
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        options
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
        #[cfg(windows)]
        // Open reparse points themselves for rejection, and forbid deletion
        // sharing while the handle is held. Read/write sharing permits a second
        // session to reach the actual OS lock and receive the named Busy error.
        options.custom_flags(0x0020_0000).share_mode(0x0000_0003);
        let lock = options
            .open(paths.lock_file())
            .map_err(|error| path_io("open profile lock", paths.lock_file(), error))?;
        validate_open_lock(paths.lock_file(), &lock)?;
        if let Err(error) = fs2::FileExt::try_lock_exclusive(&lock) {
            let contended = fs2::lock_contended_error().raw_os_error();
            if error.kind() == io::ErrorKind::WouldBlock
                || (contended.is_some() && contended == error.raw_os_error())
            {
                return Err(ProfileError::Busy {
                    root: paths.root.clone(),
                });
            }
            return Err(path_io(
                "lock profile exclusively",
                paths.lock_file(),
                error,
            ));
        }
        validate_open_lock(paths.lock_file(), &lock)?;
        Ok(Self { paths, _lock: lock })
    }

    pub fn paths(&self) -> &ProfilePaths {
        &self.paths
    }
}

#[cfg(target_os = "linux")]
fn native_data_root() -> Result<PathBuf, ProfileError> {
    let root = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).ok_or(
        ProfileError::UnavailableDataRoot("XDG_DATA_HOME is missing"),
    )?;
    if !root.is_absolute() {
        return Err(ProfileError::RelativeRoot(root));
    }
    Ok(root)
}

#[cfg(any(target_os = "macos", windows))]
fn native_data_root() -> Result<PathBuf, ProfileError> {
    dirs_next::data_local_dir().ok_or(ProfileError::UnavailableDataRoot(
        "native per-user data directory is unavailable",
    ))
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
fn native_data_root() -> Result<PathBuf, ProfileError> {
    Err(ProfileError::UnavailableDataRoot(
        "native profile discovery is unsupported on this platform",
    ))
}

fn ensure_directory(path: &Path) -> Result<(), ProfileError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => Ok(()),
        Ok(_) => Err(ProfileError::InvalidRoot {
            path: path.to_path_buf(),
            reason: "expected a directory, not a symlink or non-directory",
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let parent = path.parent().ok_or_else(|| ProfileError::InvalidRoot {
                path: path.to_path_buf(),
                reason: "missing parent directory",
            })?;
            ensure_directory(parent)?;
            #[cfg(unix)]
            let created = fs::DirBuilder::new().mode(0o700).create(path);
            #[cfg(not(unix))]
            let created = fs::create_dir(path);
            match created {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => validate_root(path),
                Err(error) => Err(path_io("create private directory", path, error)),
            }
        }
        Err(error) => Err(path_io("inspect profile directory", path, error)),
    }
}

fn validate_root(path: &Path) -> Result<(), ProfileError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| path_io("inspect private profile", path, error))?;
    let reason = if !metadata.is_dir() || metadata.file_type().is_symlink() {
        Some("expected a real directory")
    } else if metadata.permissions().readonly() {
        Some("directory is read-only")
    } else {
        #[cfg(unix)]
        {
            // SAFETY: geteuid has no preconditions and does not mutate state.
            if metadata.uid() != unsafe { libc::geteuid() } {
                Some("directory is not owned by the effective user")
            } else if metadata.permissions().mode() & 0o777 != 0o700 {
                Some("directory must have private owner rwx permissions (0700)")
            } else {
                None
            }
        }
        #[cfg(not(unix))]
        {
            None
        }
    };
    match reason {
        Some(reason) => Err(ProfileError::InvalidRoot {
            path: path.to_path_buf(),
            reason,
        }),
        None => Ok(()),
    }
}

fn validate_lock(path: &Path, metadata: &Metadata) -> Result<(), ProfileError> {
    let mut reason = None;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        reason = Some("expected a regular non-symlink file");
    } else if metadata.permissions().readonly() {
        reason = Some("lock file is read-only");
    }
    #[cfg(unix)]
    {
        // SAFETY: geteuid has no preconditions and does not mutate state.
        if metadata.uid() != unsafe { libc::geteuid() }
            || metadata.permissions().mode() & 0o777 != 0o600
            || metadata.nlink() != 1
        {
            reason = Some("lock must be user-owned, private (0600), and single-link");
        }
    }
    #[cfg(windows)]
    if metadata.file_attributes() & 0x400 != 0 {
        reason = Some("lock cannot be a reparse point");
    }
    match reason {
        Some(reason) => Err(ProfileError::UnsafeLock {
            path: path.to_path_buf(),
            reason,
        }),
        None => Ok(()),
    }
}

fn validate_open_lock(path: &Path, file: &File) -> Result<(), ProfileError> {
    let opened = file
        .metadata()
        .map_err(|error| path_io("inspect open lock", path, error))?;
    let named =
        fs::symlink_metadata(path).map_err(|error| path_io("inspect named lock", path, error))?;
    validate_lock(path, &opened)?;
    validate_lock(path, &named)?;
    #[cfg(unix)]
    if opened.dev() != named.dev() || opened.ino() != named.ino() {
        return Err(ProfileError::UnsafeLock {
            path: path.to_path_buf(),
            reason: "lock path no longer names the open file",
        });
    }
    Ok(())
}

fn path_io(operation: &'static str, path: &Path, source: io::Error) -> ProfileError {
    ProfileError::Io {
        operation,
        path: path.to_path_buf(),
        source,
    }
}
