//! Snapshot-bound findings, not execution authority or a filesystem transaction.
//!
//! Graph semantics are the existing DependencyGraph algorithms. File hashes use
//! the same SHA-256 algorithm as federation::integrity, through bounded reads
//! rather than that legacy verifier (whose path/error behavior is unchanged).
//! Captured catalog hashes do not prove freshness. Actual file observations are
//! sequential, not simultaneous; in-place concurrent writes can change content.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use super::session::ContextTag;
use super::snapshot::{CatalogSnapshot, CollectionKind, CollectionStatus};
use crate::catalog::store::EdgeKind;
use crate::federation::dep_graph::DependencyGraph;

/// Completeness of observations, NOT an assertion that every finding passed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingsQuality {
    Current,
    Partial,
    Unavailable,
}

/// Original parser input observation, retained with its snapshot context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindingInput {
    pub collection: CollectionKind,
    pub path: PathBuf,
    pub sha256: Option<String>,
    pub status: CollectionStatus,
    /// Internal diagnostics; presentation/export adapters must sanitize them.
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetSource {
    pub id: String,
    pub collection: CollectionKind,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdCollision {
    pub id: String,
    pub collections: Vec<CollectionKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvedReference {
    pub from: String,
    pub to: String,
    pub expected_collection: CollectionKind,
}

pub struct DependencyFindings {
    pub context: ContextTag,
    pub quality: FindingsQuality,
    pub inputs: Vec<FindingInput>,
    pub assets: Vec<AssetSource>,
    /// No untyped graph is built when cross-collection identities collide.
    pub graph: Option<DependencyGraph>,
    /// Sorted graph node IDs, including unknown referenced endpoints. Empty
    /// when collisions make the untyped graph unavailable.
    pub node_ids: Vec<String>,
    /// Existing DFS cycle observations, not a claim to enumerate every simple cycle.
    pub cycles: Vec<Vec<String>>,
    pub collisions: Vec<IdCollision>,
    pub unresolved: Vec<UnresolvedReference>,
}

fn input(snapshot: &CatalogSnapshot, collection: CollectionKind, name: &str) -> FindingInput {
    let state = snapshot.collection(collection);
    let path = snapshot
        .context()
        .source()
        .root()
        .join("catalog")
        .join(name);
    FindingInput {
        collection,
        sha256: snapshot
            .catalog()
            .content_hash_for(&path)
            .map(str::to_owned),
        path,
        status: state.status,
        issues: state.issues.clone(),
    }
}

fn complete(input: &FindingInput) -> bool {
    matches!(
        input.status,
        CollectionStatus::Current | CollectionStatus::Empty
    ) && input.sha256.is_some()
        && input.issues.is_empty()
}

/// Inspect only immutable captured catalog data; never re-read dependency files.
pub fn dependency_findings(snapshot: &CatalogSnapshot) -> DependencyFindings {
    let store = snapshot.catalog();
    let inputs = vec![
        input(snapshot, CollectionKind::Agents, "agents.json"),
        input(snapshot, CollectionKind::Roles, "install-roles.json"),
        input(
            snapshot,
            CollectionKind::McpReferences,
            "mcp-references.json",
        ),
        input(snapshot, CollectionKind::Rules, "rules.json"),
        input(snapshot, CollectionKind::Skills, "skills.json"),
    ];
    let mut assets = Vec::new();
    for (kind, rows) in [
        (
            CollectionKind::Agents,
            store
                .agents
                .iter()
                .map(|a| (&a.id, &a.path))
                .collect::<Vec<_>>(),
        ),
        (
            CollectionKind::Skills,
            store.skills.iter().map(|a| (&a.id, &a.path)).collect(),
        ),
        (
            CollectionKind::McpReferences,
            store.mcp_refs.iter().map(|a| (&a.id, &a.path)).collect(),
        ),
        (
            CollectionKind::Rules,
            store.rules.iter().map(|a| (&a.id, &a.path)).collect(),
        ),
    ] {
        assets.extend(rows.into_iter().map(|(id, path)| AssetSource {
            id: id.clone(),
            collection: kind,
            path: path.clone(),
        }));
    }
    assets.extend(store.roles.keys().map(|id| AssetSource {
        id: id.clone(),
        collection: CollectionKind::Roles,
        path: "catalog/install-roles.json".into(),
    }));
    assets.sort_by(|a, b| (&a.id, a.collection, &a.path).cmp(&(&b.id, b.collection, &b.path)));
    let mut ids: BTreeMap<&str, BTreeSet<CollectionKind>> = BTreeMap::new();
    for asset in &assets {
        ids.entry(&asset.id).or_default().insert(asset.collection);
    }
    let collisions = ids
        .iter()
        .filter(|(_, kinds)| kinds.len() > 1)
        .map(|(id, kinds)| IdCollision {
            id: (*id).into(),
            collections: kinds.iter().copied().collect(),
        })
        .collect::<Vec<_>>();
    let mut unresolved = Vec::new();
    for (from, to, kind) in store.dependency_edges() {
        let expected_collection = match kind {
            EdgeKind::AgentSkill => CollectionKind::Skills,
            EdgeKind::RoleAgent => CollectionKind::Agents,
            EdgeKind::AgentMcp => CollectionKind::McpReferences,
            EdgeKind::AgentRule => CollectionKind::Rules,
        };
        if !ids
            .get(to.as_str())
            .is_some_and(|kinds| kinds.contains(&expected_collection))
        {
            unresolved.push(UnresolvedReference {
                from,
                to,
                expected_collection,
            });
        }
    }
    unresolved.sort_by(|a, b| {
        (&a.from, &a.to, a.expected_collection).cmp(&(&b.from, &b.to, b.expected_collection))
    });
    unresolved.dedup();
    let graph = collisions.is_empty().then(|| {
        let mut graph = DependencyGraph::build(store);
        // Existing build sorts endpoints. Break equal-endpoint ties by edge kind
        // so HashMap role enumeration cannot affect the returned edge ordering.
        graph.edges.sort_by(|a, b| {
            (&a.from, &a.to, format!("{:?}", a.edge_type)).cmp(&(
                &b.from,
                &b.to,
                format!("{:?}", b.edge_type),
            ))
        });
        graph
    });
    let cycles = graph
        .as_ref()
        .map(DependencyGraph::find_cycles)
        .unwrap_or_default();
    let mut node_ids = graph
        .as_ref()
        .map(|g| g.nodes.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    node_ids.sort();
    let quality = if inputs
        .iter()
        .all(|i| matches!(i.status, CollectionStatus::Unavailable))
    {
        FindingsQuality::Unavailable
    } else if inputs.iter().all(complete) && collisions.is_empty() && unresolved.is_empty() {
        FindingsQuality::Current
    } else {
        FindingsQuality::Partial
    };
    DependencyFindings {
        context: snapshot.context().clone(),
        quality,
        inputs,
        assets,
        graph,
        node_ids,
        cycles,
        collisions,
        unresolved,
    }
}

/// Hard per-file byte ceiling, even if a caller supplies a larger limit.
pub const MAX_INTEGRITY_FILE_BYTES: u64 = 16 * 1024 * 1024;
pub const FILE_OBSERVATION_LIMIT: &str = "Sequential owned-file observations, not an atomic multi-file snapshot; concurrent in-place writes, hard links and mount changes are not excluded. Tree and manifest aggregate digests are not verified.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Pass,
    Fail,
    Missing,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFinding {
    pub path: String,
    pub expected_hash: String,
    pub actual_hash: Option<String>,
    pub expected_bytes: u64,
    pub actual_bytes: Option<u64>,
    pub status: FileStatus,
    pub issue: Option<String>,
}

#[derive(Debug)]
pub struct IntegrityFindings {
    pub context: ContextTag,
    pub quality: FindingsQuality,
    pub inputs: Vec<FindingInput>,
    pub files: Vec<FileFinding>,
    pub issues: Vec<String>,
    pub observation_limit: &'static str,
}

fn digest_valid(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Verify manifest-listed regular files, with no writes, execution or network.
/// Unsupported safe filesystem backends fail closed rather than use fs::read.
pub fn integrity_findings(snapshot: &CatalogSnapshot, max_file_bytes: u64) -> IntegrityFindings {
    let manifest_input = input(snapshot, CollectionKind::Integrity, "asset-integrity.json");
    let mut report = IntegrityFindings {
        context: snapshot.context().clone(),
        quality: FindingsQuality::Unavailable,
        inputs: vec![manifest_input],
        files: Vec::new(),
        issues: Vec::new(),
        observation_limit: FILE_OBSERVATION_LIMIT,
    };
    let Some(manifest) = snapshot.catalog().integrity.as_ref() else {
        report
            .issues
            .push("Integrity manifest is unavailable".into());
        return report;
    };
    if !complete(&report.inputs[0]) {
        report
            .issues
            .push("Integrity manifest input is partial, stale or unavailable".into());
        return report;
    }
    let mut entries = manifest
        .root_files
        .iter()
        .chain(manifest.trees.iter().flat_map(|t| &t.files))
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    let listed_roots = manifest
        .root_files
        .iter()
        .map(|f| f.path.as_str())
        .collect::<BTreeSet<_>>();
    let scope_roots = manifest
        .scope
        .root_files
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let listed_trees = manifest
        .trees
        .iter()
        .map(|t| t.tree.as_str())
        .collect::<BTreeSet<_>>();
    let scope_trees = manifest
        .scope
        .trees
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if manifest.manifest_version != 1
        || manifest.algorithm != "sha256"
        || !digest_valid(&manifest.aggregate_sha256)
        || manifest
            .trees
            .iter()
            .any(|t| !digest_valid(&t.aggregate_sha256))
        || entries.iter().any(|f| !digest_valid(&f.sha256))
        || entries.windows(2).any(|w| w[0].path == w[1].path)
        || listed_roots != scope_roots
        || listed_trees != scope_trees
        || listed_roots.len() != manifest.root_files.len()
        || scope_roots.len() != manifest.scope.root_files.len()
        || listed_trees.len() != manifest.trees.len()
        || scope_trees.len() != manifest.scope.trees.len()
    {
        report.issues.push("Integrity manifest is invalid, incomplete or unsupported (version 1 / sha256 required)".into());
        return report;
    }
    #[cfg(unix)]
    let reader = match SafeRoot::open(snapshot.context()) {
        Ok(reader) => reader,
        Err(issue) => {
            report.issues.push(issue);
            return report;
        }
    };
    #[cfg(not(unix))]
    {
        let _ = max_file_bytes;
        report.issues.push(
            "Safe descriptor-relative integrity verification is unsupported on this platform"
                .into(),
        );
        return report;
    }
    #[cfg(unix)]
    {
        for entry in entries {
            let mut finding = FileFinding {
                path: entry.path.clone(),
                expected_hash: entry.sha256.clone(),
                actual_hash: None,
                expected_bytes: entry.bytes,
                actual_bytes: None,
                status: FileStatus::Unavailable,
                issue: None,
            };
            match reader.hash(&entry.path, max_file_bytes.min(MAX_INTEGRITY_FILE_BYTES)) {
                Ok((hash, bytes)) => {
                    finding.status = if hash == entry.sha256 && bytes == entry.bytes {
                        FileStatus::Pass
                    } else {
                        FileStatus::Fail
                    };
                    finding.actual_hash = Some(hash);
                    finding.actual_bytes = Some(bytes);
                }
                Err(FileReadError::Missing) => {
                    finding.status = FileStatus::Missing;
                    finding.issue = Some("Manifest-listed file is missing".into());
                }
                Err(FileReadError::Unavailable(issue)) => finding.issue = Some(issue),
            }
            report.files.push(finding);
        }
        if let Err(issue) = reader.verify_named_identity() {
            // A changed root invalidates the entire pass, not only later files.
            for finding in &mut report.files {
                finding.status = FileStatus::Unavailable;
                finding.actual_hash = None;
                finding.actual_bytes = None;
                finding.issue = Some("Source directory identity changed during observation".into());
            }
            report.issues.push(issue);
            return report;
        }
        report.quality = if report
            .files
            .iter()
            .any(|f| f.status == FileStatus::Unavailable)
        {
            FindingsQuality::Partial
        } else {
            FindingsQuality::Current
        };
        report
    }
}

#[cfg(unix)]
#[derive(Debug)]
enum FileReadError {
    Missing,
    Unavailable(String),
}

#[cfg(unix)]
impl From<std::io::Error> for FileReadError {
    fn from(error: std::io::Error) -> Self {
        if error.kind() == std::io::ErrorKind::NotFound {
            Self::Missing
        } else {
            Self::Unavailable(format!("File access unavailable: {:?}", error.kind()))
        }
    }
}

#[cfg(unix)]
struct SafeRoot<'a> {
    file: std::fs::File,
    context: &'a ContextTag,
}

#[cfg(unix)]
impl<'a> SafeRoot<'a> {
    fn open(context: &'a ContextTag) -> Result<Self, String> {
        use std::os::unix::fs::OpenOptionsExt;
        let file = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY | libc::O_NONBLOCK | libc::O_CLOEXEC)
            .open(context.source().root())
            .map_err(|_| "Source directory cannot be safely opened".to_string())?;
        let root = Self { file, context };
        root.verify_named_identity()?;
        Ok(root)
    }
    fn verify_named_identity(&self) -> Result<(), String> {
        use super::session::NativeFileId;
        use std::os::unix::fs::MetadataExt;
        let same = |m: &std::fs::Metadata| {
            m.is_dir()
                && !m.file_type().is_symlink()
                && NativeFileId::Unix {
                    device: m.dev(),
                    inode: m.ino(),
                } == self.context.source().native_id()
        };
        let held = self
            .file
            .metadata()
            .map_err(|_| "Cannot inspect held source identity".to_string())?;
        let named = std::fs::symlink_metadata(self.context.source().root())
            .map_err(|_| "Selected source directory is unavailable".to_string())?;
        if same(&held) && same(&named) {
            Ok(())
        } else {
            Err("Selected source directory no longer has captured native identity".into())
        }
    }
    fn hash(&self, path: &str, limit: u64) -> Result<(String, u64), FileReadError> {
        use sha2::{Digest, Sha256};
        use std::io::Read;
        use std::os::fd::{AsRawFd, FromRawFd};
        let unavailable = |message: &str| FileReadError::Unavailable(message.into());
        // Portable lexical policy: never silently normalize traversal, Windows
        // drives/backslashes or empty components into a different manifest path.
        if path.is_empty()
            || path.contains('\\')
            || path.contains(':')
            || path
                .split('/')
                .any(|p| p.is_empty() || p == "." || p == "..")
        {
            return Err(unavailable(
                "Manifest path is not an unambiguous relative path",
            ));
        }
        self.verify_named_identity()
            .map_err(FileReadError::Unavailable)?;
        let components = path.split('/').collect::<Vec<_>>();
        let mut directory = None::<std::fs::File>;
        for (index, component) in components.iter().enumerate() {
            let name = std::ffi::CString::new(*component)
                .map_err(|_| unavailable("Manifest path contains NUL"))?;
            let parent = directory.as_ref().unwrap_or(&self.file);
            let final_component = index + 1 == components.len();
            let flags = libc::O_RDONLY
                | libc::O_NOFOLLOW
                | libc::O_NONBLOCK
                | libc::O_CLOEXEC
                | if final_component {
                    0
                } else {
                    libc::O_DIRECTORY
                };
            // SAFETY: parent owns a live descriptor through the call; name owns
            // a NUL-terminated single component, no slash/parent traversal. Flags
            // never create/write, so no variadic mode argument is required.
            let fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags) };
            if fd < 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            // SAFETY: successful openat returned a fresh owned fd; it has no
            // other Rust owner. File closes it exactly once on all return paths.
            let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
            let before = file.metadata()?;
            if final_component {
                if !before.is_file() {
                    return Err(unavailable("Manifest entry is not a regular file"));
                }
                if before.len() > limit {
                    return Err(unavailable("File exceeds integrity read byte limit"));
                }
                let mut hasher = Sha256::new();
                let mut total = 0u64;
                let mut buffer = [0u8; 8192];
                loop {
                    let capacity =
                        (limit.saturating_sub(total) + 1).min(buffer.len() as u64) as usize;
                    let count = file.read(&mut buffer[..capacity])?;
                    if count == 0 {
                        break;
                    }
                    total += count as u64;
                    if total > limit {
                        return Err(unavailable("File exceeds integrity read byte limit"));
                    }
                    hasher.update(&buffer[..count]);
                }
                let after = file.metadata()?;
                if before.len() != after.len()
                    || before.modified().ok() != after.modified().ok()
                    || total != after.len()
                {
                    return Err(unavailable("File changed during integrity observation"));
                }
                self.verify_named_identity()
                    .map_err(FileReadError::Unavailable)?;
                return Ok((format!("{:x}", hasher.finalize()), total));
            }
            if !before.is_dir() {
                return Err(unavailable("Manifest parent is not a directory"));
            }
            directory = Some(file);
        }
        Err(unavailable("Empty manifest path"))
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::FileReadError;

    #[test]
    fn inaccessible_is_not_missing() {
        assert!(matches!(
            FileReadError::from(std::io::Error::from(std::io::ErrorKind::PermissionDenied)),
            FileReadError::Unavailable(_)
        ));
        assert!(matches!(
            FileReadError::from(std::io::Error::from(std::io::ErrorKind::NotFound)),
            FileReadError::Missing
        ));
    }
}
