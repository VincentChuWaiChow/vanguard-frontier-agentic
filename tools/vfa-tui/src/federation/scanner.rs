//! Workspace scanner — multi-strategy installed-asset detection.
//!
//! # Design overview
//!
//! [`WorkspaceScanner`] walks each workspace's harness directories and applies
//! three detection strategies per file (Req 7.2):
//!
//! (a) **Filename** — the file's basename matches a canonical asset `path`
//!     basename AND the harness-directory layout is valid for that harness.
//! (b) **MetadataComment** — the first lines contain a `# VFA-EXPORT: {json}`
//!     comment with a parseable asset `id` (Req 7.7).
//! (c) **ContentSignature** — a conservative line-overlap ratio between the
//!     file's first 50 lines and the canonical template's first 50 lines
//!     exceeds a threshold (Req 7.2 / 7.3).
//!
//! An asset is "confirmed installed" when **at least two** distinct strategies
//! agree (Req 7.2).  A SHA-256 hash is computed for every matched file (Req 7.3).
//!
//! Scanning is parallelised via tokio tasks bounded by a [`tokio::sync::Semaphore`]
//! at the configured concurrency limit (default 8, Req 23.1).

#![deny(warnings)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::Semaphore;
use tracing::warn;

use crate::models::workspace::ResolvedWorkspace;
use crate::workspace::harness_layout::{
    detect_harness_dirs, validate_harness_layout, HarnessDir, LayoutMatch,
};

// ---------------------------------------------------------------------------
// DetectionMethod
// ---------------------------------------------------------------------------

/// Which strategy triggered a match for an installed file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionMethod {
    /// File basename matches a canonical asset path basename, and the harness
    /// layout validates (Req 7.2 strategy a).
    Filename,
    /// File contains a `# VFA-EXPORT: {json}` metadata comment (Req 7.7).
    MetadataComment,
    /// First-50-line content signature overlaps with a canonical template
    /// above the similarity threshold (Req 7.2 strategy c).
    ContentSignature,
}

// ---------------------------------------------------------------------------
// ExportMeta — parsed VFA-EXPORT header
// ---------------------------------------------------------------------------

/// Parsed content of a `# VFA-EXPORT: {json}` metadata line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportMeta {
    /// Asset identifier (required).
    pub id: String,
    /// Installed version, if present.
    pub version: Option<String>,
    /// ISO timestamp written by the export CLI, if present.
    pub installed_at: Option<String>,
}

// ---------------------------------------------------------------------------
// InstalledAsset
// ---------------------------------------------------------------------------

/// A single asset detected as installed in a workspace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstalledAsset {
    /// Absolute path to the installed file inside the workspace.
    pub workspace_path: PathBuf,
    /// Canonical asset identifier (e.g. `"agents/aws/cdk-agent"`).
    pub asset_id: String,
    /// Version extracted via VFA-EXPORT metadata or frontmatter, if any.
    pub installed_version: Option<String>,
    /// SHA-256 hex digest of the file bytes.
    pub content_hash: String,
    /// Which strategies fired for this file.
    pub detection_methods: Vec<DetectionMethod>,
    /// `true` when ≥2 distinct strategies agreed (Req 7.2).
    pub confirmed: bool,
    /// Harness directory name (e.g. `".claude"`).
    pub harness: String,
}

// ---------------------------------------------------------------------------
// Confirmation rule (pure helper — property-testable)
// ---------------------------------------------------------------------------

/// Returns `true` when `methods` contains at least two **distinct** detection
/// signals, satisfying the "≥2 confirming strategies" rule (Req 7.2).
///
/// Duplicates are collapsed before counting so that accidentally applying the
/// same strategy twice does not inflate the count.
pub fn is_confirmed(methods: &[DetectionMethod]) -> bool {
    // Collect unique strategies.
    let unique: std::collections::HashSet<_> = methods.iter().collect();
    unique.len() >= 2
}

// ---------------------------------------------------------------------------
// CatalogIndex — lightweight precomputed lookup structure
// ---------------------------------------------------------------------------

/// Precomputed index derived from a [`crate::catalog::store::CatalogStore`]
/// snapshot so the scanner can work with owned data inside spawned tasks.
#[derive(Debug, Clone)]
pub struct CatalogIndex {
    /// Map from file basename (e.g. `"cdk-agent.md"`) → canonical asset ID.
    pub basename_to_id: HashMap<String, String>,
    /// Map from canonical asset ID → first-50-lines of canonical content
    /// (when the canonical file is readable on disk).
    pub id_to_template: HashMap<String, String>,
}

impl CatalogIndex {
    /// Build a [`CatalogIndex`] from parallel slices of (asset_path, asset_id,
    /// optional_canonical_content).
    ///
    /// `asset_path` is the catalog `path` field (e.g. `"agents/aws/cdk-agent"`);
    /// the basename used for filename matching is its last path component with
    /// any supported extension appended for each harness.
    pub fn new(entries: impl IntoIterator<Item = (String, String, Option<String>)>) -> Self {
        let mut basename_to_id = HashMap::new();
        let mut id_to_template = HashMap::new();

        for (path, id, maybe_content) in entries {
            // Extract basename of the canonical path (without extension).
            let stem = PathBuf::from(&path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&path)
                .to_string();

            // Register with several extensions so all harness types can match.
            // `.agent.md` is the copilot export convention
            // (`.github/agents/<id>.agent.md`); without it the basename never
            // matched and copilot exports raised only one detection signal.
            for ext in &[".md", ".agent.md", ".json", ".toml", ".yaml", ".yml"] {
                let key = format!("{stem}{ext}");
                basename_to_id.entry(key).or_insert_with(|| id.clone());
            }
            // Also register the bare stem (no extension).
            basename_to_id
                .entry(stem.clone())
                .or_insert_with(|| id.clone());

            if let Some(content) = maybe_content {
                id_to_template.insert(id, first_n_lines(&content, 50).to_string());
            }
        }

        Self {
            basename_to_id,
            id_to_template,
        }
    }
}

// ---------------------------------------------------------------------------
// WorkspaceScanner
// ---------------------------------------------------------------------------

/// Scans downstream workspaces for installed catalog assets.
///
/// Construct with [`WorkspaceScanner::new`].  The scanner is `Clone + Send`
/// and safe to share across tokio tasks via `Arc`.
#[derive(Debug, Clone)]
pub struct WorkspaceScanner {
    /// Maximum number of workspaces scanned concurrently.
    pub concurrency: usize,
}

impl WorkspaceScanner {
    /// Create a scanner with the given concurrency limit.
    ///
    /// `concurrency` controls how many workspace scans run simultaneously via
    /// tokio tasks.  The default recommended value is `8` (Req 23.1).
    pub fn new(concurrency: usize) -> Self {
        Self {
            concurrency: concurrency.max(1),
        }
    }

    // -----------------------------------------------------------------------
    // parse_export_metadata
    // -----------------------------------------------------------------------

    /// Parse the `# VFA-EXPORT: {json}` metadata comment format (Req 7.7).
    ///
    /// Searches `content` line-by-line (up to 20 lines) for a line whose
    /// trimmed form starts with `# VFA-EXPORT:` (case-sensitive).  The JSON
    /// payload after the colon is parsed with `serde_json`.
    ///
    /// JSON documents carry no comment syntax and therefore no marker: see the
    /// note in `scripts/export-marketplace-agents.mjs`.
    ///
    /// Returns `None` for any malformed or missing line without panicking.
    pub fn parse_export_metadata(content: &str) -> Option<ExportMeta> {
        const PREFIX: &str = "VFA-EXPORT:";
        for line in content.lines().take(20) {
            let trimmed = line.trim();
            // Accept `# VFA-EXPORT:` and `// VFA-EXPORT:` variants.
            let after_hash = if let Some(rest) = trimmed.strip_prefix('#') {
                rest.trim_start()
            } else if let Some(rest) = trimmed.strip_prefix("//") {
                rest.trim_start()
            } else {
                continue;
            };
            if let Some(json_payload) = after_hash.strip_prefix(PREFIX) {
                let json_payload = json_payload.trim();
                return serde_json::from_str::<ExportMeta>(json_payload).ok();
            }
        }

        None
    }

    // -----------------------------------------------------------------------
    // match_content_signature
    // -----------------------------------------------------------------------

    /// Conservative content-similarity heuristic for the first 50 lines.
    ///
    /// **Heuristic:** normalise both sides (trim whitespace, lowercase), split
    /// into a set of non-empty lines, and compute the Jaccard-like overlap:
    ///
    /// ```text
    /// overlap = |file_lines ∩ template_lines| / |template_lines|
    /// ```
    ///
    /// Returns `true` when `overlap >= 0.40` AND both sides have at least 3
    /// non-empty lines.  This threshold is deliberately conservative to avoid
    /// false positives on short boilerplate.  Callers should treat this as a
    /// supporting signal only; confirmation requires a second agreeing strategy.
    pub fn match_content_signature(first_50_lines: &str, canonical_first_50: &str) -> bool {
        let file_set: std::collections::HashSet<String> = first_50_lines
            .lines()
            .map(|l| l.trim().to_lowercase())
            .filter(|l| !l.is_empty())
            .collect();

        let template_set: std::collections::HashSet<String> = canonical_first_50
            .lines()
            .map(|l| l.trim().to_lowercase())
            .filter(|l| !l.is_empty())
            .collect();

        if file_set.len() < 3 || template_set.len() < 3 {
            return false;
        }

        let overlap = file_set.intersection(&template_set).count();
        let ratio = overlap as f64 / template_set.len() as f64;
        ratio >= 0.40
    }

    // -----------------------------------------------------------------------
    // scan_harness_dir
    // -----------------------------------------------------------------------

    /// Scan a single harness directory and return detected assets.
    ///
    /// Per-harness file-extension filters (Req 7.8):
    ///
    /// | Harness   | Extensions scanned             |
    /// |-----------|-------------------------------|
    /// | Claude    | `*.md`                         |
    /// | Cursor    | `*.md`, `*.json`               |
    /// | Kiro      | `*.md`, `*.json`               |
    /// | Codex     | `plugin.json`, `*.toml`        |
    /// | Opencode  | `*.toml`, `*.yaml`, `*.yml`    |
    /// | Copilot   | `*.md`                         |
    /// | Gemini    | `*.md`                         |
    ///
    /// Validates the layout before walking (Req 7.6); returns empty vec with a
    /// `warn!` if the layout does not match any known pattern.
    pub fn scan_harness_dir(
        &self,
        _workspace_path: &Path,
        harness: &HarnessDir,
        dir: &Path,
        index: &CatalogIndex,
    ) -> Vec<InstalledAsset> {
        // Validate layout first (Req 7.6).
        if validate_harness_layout(harness, dir) == LayoutMatch::NoMatch {
            warn!(
                harness = harness.dir_name(),
                path = %dir.display(),
                "harness directory does not match expected layout — skipping"
            );
            return vec![];
        }

        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(err) => {
                warn!(path = %dir.display(), error = %err, "could not read harness directory");
                return vec![];
            }
        };

        let mut results = Vec::new();

        for entry in entries.flatten() {
            let file_path = entry.path();
            if !file_path.is_file() {
                continue;
            }

            // Apply harness-specific file-extension filter (Req 7.8).
            if !self.file_matches_harness(&file_path, harness) {
                continue;
            }

            // Read file bytes; skip on I/O error.
            let bytes = match std::fs::read(&file_path) {
                Ok(b) => b,
                Err(err) => {
                    warn!(path = %file_path.display(), error = %err, "could not read file");
                    continue;
                }
            };

            // Compute SHA-256 (Req 7.3).
            let content_hash = sha256_hex(&bytes);

            let content = String::from_utf8_lossy(&bytes);
            let first_50 = first_n_lines(&content, 50);

            let basename = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            let mut methods: Vec<DetectionMethod> = Vec::new();
            let mut asset_id: Option<String> = None;
            let mut installed_version: Option<String> = None;

            // --- Strategy (a): Filename + layout ---
            if let Some(id) = index.basename_to_id.get(&basename) {
                methods.push(DetectionMethod::Filename);
                asset_id = Some(id.clone());
            }

            // --- Strategy (b): VFA-EXPORT metadata comment (Req 7.7) ---
            if let Some(meta) = Self::parse_export_metadata(&content) {
                methods.push(DetectionMethod::MetadataComment);
                // Metadata comment is authoritative for ID if filename didn't fire.
                if asset_id.is_none() {
                    asset_id = Some(meta.id.clone());
                }
                if meta.version.is_some() {
                    installed_version = meta.version;
                }
                // Confirm consistency: if both fired, prefer the metadata ID.
                else {
                    asset_id = Some(meta.id);
                }
            }

            // --- Strategy (c): Content signature ---
            if let Some(ref id) = asset_id.clone() {
                if let Some(template) = index.id_to_template.get(id) {
                    if Self::match_content_signature(first_50, template) {
                        methods.push(DetectionMethod::ContentSignature);
                    }
                }
            } else {
                // No id yet — try content signature against all templates.
                for (tid, template) in &index.id_to_template {
                    if Self::match_content_signature(first_50, template) {
                        methods.push(DetectionMethod::ContentSignature);
                        if asset_id.is_none() {
                            asset_id = Some(tid.clone());
                        }
                        break;
                    }
                }
            }

            // Only emit an asset if at least one strategy fired and we know the ID.
            if let Some(id) = asset_id {
                // Dedup methods before confirmation check.
                methods.sort_by_key(|m| format!("{m:?}"));
                methods.dedup();
                let confirmed = is_confirmed(&methods);

                results.push(InstalledAsset {
                    workspace_path: file_path,
                    asset_id: id,
                    installed_version,
                    content_hash,
                    detection_methods: methods,
                    confirmed,
                    harness: harness.dir_name().to_string(),
                });
            }
        }

        results
    }

    // -----------------------------------------------------------------------
    // scan_workspace
    // -----------------------------------------------------------------------

    /// Scan a single resolved workspace, returning all detected assets sorted
    /// by `asset_id` (Req 27.2).
    ///
    /// Harness directories that do not match any known layout are warned and
    /// skipped (Req 7.6).
    ///
    /// Bug #2: Recursively checks for `agents/` and `skills/` subdirectories within
    /// each harness root (e.g., `.claude/agents/`, `.cursor/agents/`, etc.).
    pub fn scan_workspace(
        &self,
        ws: &ResolvedWorkspace,
        index: &CatalogIndex,
    ) -> Vec<InstalledAsset> {
        let harness_dirs = detect_harness_dirs(&ws.canonical_path);

        let ws_root = &ws.canonical_path;
        let mut all: Vec<InstalledAsset> = harness_dirs
            .iter()
            .flat_map(|(harness, dir)| self.scan_harness_dir(ws_root, harness, dir, index))
            .collect();

        // Bug #2: Recursively check for agents/ and skills/ subdirectories within each harness root.
        // Example: .claude/agents/, .cursor/agents/, etc.
        for (harness, dir) in &harness_dirs {
            for subdir_name in &["agents", "skills"] {
                let subdir = dir.join(subdir_name);
                if subdir.is_dir() {
                    let sub_assets = self.scan_harness_dir(ws_root, harness, &subdir, index);
                    all.extend(sub_assets);
                }
            }
        }

        all.sort_by(|a, b| a.asset_id.cmp(&b.asset_id));
        all
    }

    // -----------------------------------------------------------------------
    // scan_all
    // -----------------------------------------------------------------------

    /// Scan all workspaces in parallel (up to `self.concurrency` simultaneous
    /// tasks) and return a map from workspace canonical path → installed assets.
    ///
    /// Uses a [`Semaphore`] to bound concurrency (Req 23.1).  Each workspace
    /// scan runs in its own `tokio::spawn` task so I/O does not block the
    /// executor.
    ///
    /// Offline/unavailable workspaces are skipped silently here; callers should
    /// pre-filter or mark them based on [`ResolvedWorkspace::status`].
    pub async fn scan_all(
        &self,
        workspaces: &[ResolvedWorkspace],
        index: &CatalogIndex,
    ) -> HashMap<PathBuf, Vec<InstalledAsset>> {
        let sem = Arc::new(Semaphore::new(self.concurrency));
        let scanner = Arc::new(self.clone());
        let index = Arc::new(index.clone());

        let mut handles = Vec::with_capacity(workspaces.len());

        for ws in workspaces {
            let sem = Arc::clone(&sem);
            let scanner = Arc::clone(&scanner);
            let index = Arc::clone(&index);
            let ws_owned = ws.clone();

            let handle = tokio::spawn(async move {
                // Acquire permit — released when `_permit` drops.
                let _permit = sem.acquire().await.expect("semaphore closed");
                let assets = scanner.scan_workspace(&ws_owned, &index);
                (ws_owned.canonical_path.clone(), assets)
            });

            handles.push(handle);
        }

        let mut result = HashMap::new();
        for handle in handles {
            match handle.await {
                Ok((path, assets)) => {
                    result.insert(path, assets);
                }
                Err(err) => {
                    warn!(error = %err, "workspace scan task panicked");
                }
            }
        }

        result
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Returns `true` if the file at `path` matches the harness-specific
    /// extension filter (Req 7.8).
    fn file_matches_harness(&self, path: &Path, harness: &HarnessDir) -> bool {
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => return false,
        };
        // These filters must match what `vfa-export-agents` actually writes
        // (PLATFORM_CONFIG in scripts/export-marketplace-agents.mjs). Three of
        // them did not, so a correctly exported agent was filtered out before
        // any detection strategy ran: codex writes `.codex/agents/<id>.toml`
        // and never a plugin.json, kiro-cli writes `.kiro/agents/<id>.json`
        // beside kiro-ide's `.md`, and cursor writes `.cursor/agents/<id>.md`
        // rather than JSON.
        match harness {
            HarnessDir::Claude => name.ends_with(".md"),
            HarnessDir::Cursor => name.ends_with(".md") || name.ends_with(".json"),
            HarnessDir::Kiro => name.ends_with(".md") || name.ends_with(".json"),
            HarnessDir::Codex => name == "plugin.json" || name.ends_with(".toml"),
            HarnessDir::Opencode => {
                name.ends_with(".toml") || name.ends_with(".yaml") || name.ends_with(".yml")
            }
            HarnessDir::Copilot => name.ends_with(".md"),
            HarnessDir::Gemini => name.ends_with(".md"),
        }
    }
}

// ---------------------------------------------------------------------------
// Private free functions
// ---------------------------------------------------------------------------

/// Compute the SHA-256 hex digest of a byte slice.
fn sha256_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    result.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// Return the first `n` lines of `s` as a `&str` slice (no allocation when
/// `s` already has fewer than `n` lines).
fn first_n_lines(s: &str, n: usize) -> &str {
    let mut end = 0;
    let mut count = 0;
    for (i, c) in s.char_indices() {
        if c == '\n' {
            count += 1;
            if count >= n {
                end = i;
                break;
            }
        }
        end = i + c.len_utf8();
    }
    &s[..end]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Unit tests — parse_export_metadata
    // -----------------------------------------------------------------------

    #[test]
    fn parse_export_metadata_basic() {
        let line = r#"# VFA-EXPORT: {"id":"agents/aws/cdk-agent","version":"1.2.0","installed_at":"2024-01-01"}"#;
        let meta = WorkspaceScanner::parse_export_metadata(line).expect("should parse");
        assert_eq!(meta.id, "agents/aws/cdk-agent");
        assert_eq!(meta.version.as_deref(), Some("1.2.0"));
        assert_eq!(meta.installed_at.as_deref(), Some("2024-01-01"));
    }

    #[test]
    fn parse_export_metadata_no_version() {
        let line = r#"# VFA-EXPORT: {"id":"skills/openai/summarize"}"#;
        let meta = WorkspaceScanner::parse_export_metadata(line).expect("should parse");
        assert_eq!(meta.id, "skills/openai/summarize");
        assert!(meta.version.is_none());
    }

    #[test]
    fn parse_export_metadata_not_in_first_20_lines() {
        let content = "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\n\
                       line11\nline12\nline13\nline14\nline15\nline16\nline17\nline18\nline19\nline20\n\
                       # VFA-EXPORT: {\"id\":\"too-late\"}";
        // Should NOT find the VFA-EXPORT because it's on line 21.
        assert!(WorkspaceScanner::parse_export_metadata(content).is_none());
    }

    #[test]
    fn parse_export_metadata_malformed_json_returns_none() {
        let line = "# VFA-EXPORT: {not valid json}";
        assert!(WorkspaceScanner::parse_export_metadata(line).is_none());
    }

    #[test]
    fn parse_export_metadata_empty_input_returns_none() {
        assert!(WorkspaceScanner::parse_export_metadata("").is_none());
    }

    #[test]
    fn parse_export_metadata_with_leading_whitespace() {
        let line = r#"  # VFA-EXPORT: {"id":"agents/gcp/vertex-agent"}"#;
        let meta = WorkspaceScanner::parse_export_metadata(line).expect("should parse");
        assert_eq!(meta.id, "agents/gcp/vertex-agent");
    }

    // -----------------------------------------------------------------------
    // Unit tests — match_content_signature
    // -----------------------------------------------------------------------

    #[test]
    fn match_content_signature_identical_returns_true() {
        let content =
            "# Header\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10";
        assert!(WorkspaceScanner::match_content_signature(content, content));
    }

    #[test]
    fn match_content_signature_completely_different_returns_false() {
        let a = "apple\nbanana\ncherry\ndate\nelderberry";
        let b = "foo\nbar\nbaz\nqux\nquux";
        assert!(!WorkspaceScanner::match_content_signature(a, b));
    }

    #[test]
    fn match_content_signature_too_short_returns_false() {
        let short = "line1\nline2";
        assert!(!WorkspaceScanner::match_content_signature(short, short));
    }

    // -----------------------------------------------------------------------
    // Unit tests — is_confirmed
    // -----------------------------------------------------------------------

    #[test]
    fn is_confirmed_zero_methods() {
        assert!(!is_confirmed(&[]));
    }

    #[test]
    fn is_confirmed_one_method() {
        assert!(!is_confirmed(&[DetectionMethod::Filename]));
    }

    #[test]
    fn is_confirmed_two_distinct_methods() {
        assert!(is_confirmed(&[
            DetectionMethod::Filename,
            DetectionMethod::MetadataComment,
        ]));
    }

    #[test]
    fn is_confirmed_two_identical_methods_not_confirmed() {
        assert!(!is_confirmed(&[
            DetectionMethod::Filename,
            DetectionMethod::Filename,
        ]));
    }

    #[test]
    fn is_confirmed_three_methods() {
        assert!(is_confirmed(&[
            DetectionMethod::Filename,
            DetectionMethod::MetadataComment,
            DetectionMethod::ContentSignature,
        ]));
    }

    // -----------------------------------------------------------------------
    // Integration tests — scan_harness_dir with tempfile
    // -----------------------------------------------------------------------

    #[test]
    fn scan_harness_dir_confirmed_with_filename_and_metadata() {
        use std::fs;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir(&claude_dir).unwrap();

        // File whose basename matches a canonical asset AND has VFA-EXPORT.
        let content = "# VFA-EXPORT: {\"id\":\"agents/aws/cdk-agent\",\"version\":\"1.0.0\"}\n\
                       # CDK Agent\nsome content here\n";
        fs::write(claude_dir.join("cdk-agent.md"), content).unwrap();

        // Build index where basename "cdk-agent.md" → "agents/aws/cdk-agent".
        let index = CatalogIndex::new(vec![(
            "agents/aws/cdk-agent".to_string(),
            "agents/aws/cdk-agent".to_string(),
            None,
        )]);

        let scanner = WorkspaceScanner::new(1);
        let assets = scanner.scan_harness_dir(tmp.path(), &HarnessDir::Claude, &claude_dir, &index);

        assert_eq!(assets.len(), 1);
        let asset = &assets[0];
        assert_eq!(asset.asset_id, "agents/aws/cdk-agent");
        assert!(
            asset.confirmed,
            "should be confirmed with filename+metadata"
        );
        assert!(asset.detection_methods.contains(&DetectionMethod::Filename));
        assert!(asset
            .detection_methods
            .contains(&DetectionMethod::MetadataComment));
    }

    #[test]
    fn scan_harness_dir_filename_only_not_confirmed() {
        use std::fs;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir(&claude_dir).unwrap();

        // File whose basename matches but has NO VFA-EXPORT and no template.
        let content = "# Some Agent\nThis is just some text without metadata.\n";
        fs::write(claude_dir.join("cdk-agent.md"), content).unwrap();

        let index = CatalogIndex::new(vec![(
            "agents/aws/cdk-agent".to_string(),
            "agents/aws/cdk-agent".to_string(),
            None, // no template → content signature can't fire
        )]);

        let scanner = WorkspaceScanner::new(1);
        let assets = scanner.scan_harness_dir(tmp.path(), &HarnessDir::Claude, &claude_dir, &index);

        assert_eq!(assets.len(), 1);
        let asset = &assets[0];
        assert_eq!(asset.asset_id, "agents/aws/cdk-agent");
        assert!(
            !asset.confirmed,
            "only filename match — should not be confirmed"
        );
        assert_eq!(asset.detection_methods, vec![DetectionMethod::Filename]);
    }

    // -----------------------------------------------------------------------
    // Async integration test — scan_all over two temp workspaces
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn scan_all_aggregates_two_workspaces() {
        use crate::models::workspace::{ResolvedWorkspace, WorkspaceStatus};
        use std::fs;
        use tempfile::TempDir;

        let tmp1 = TempDir::new().unwrap();
        let tmp2 = TempDir::new().unwrap();

        // Workspace 1 — has a .claude dir with a confirmed asset.
        let claude1 = tmp1.path().join(".claude");
        fs::create_dir(&claude1).unwrap();
        fs::write(
            claude1.join("cdk-agent.md"),
            "# VFA-EXPORT: {\"id\":\"agents/aws/cdk-agent\",\"version\":\"2.0.0\"}\n\
             # CDK Agent\ncontent here\n",
        )
        .unwrap();

        // Workspace 2 — has a .claude dir with a different confirmed asset.
        let claude2 = tmp2.path().join(".claude");
        fs::create_dir(&claude2).unwrap();
        fs::write(
            claude2.join("vertex-agent.md"),
            "# VFA-EXPORT: {\"id\":\"agents/gcp/vertex-agent\"}\n\
             # Vertex Agent\ncontent here\n",
        )
        .unwrap();

        let index = CatalogIndex::new(vec![
            (
                "agents/aws/cdk-agent".to_string(),
                "agents/aws/cdk-agent".to_string(),
                None,
            ),
            (
                "agents/gcp/vertex-agent".to_string(),
                "agents/gcp/vertex-agent".to_string(),
                None,
            ),
        ]);

        let ws1 = ResolvedWorkspace {
            canonical_path: tmp1.path().to_path_buf(),
            name: "ws1".to_string(),
            team: None,
            tags: vec![],
            status: WorkspaceStatus::Available,
        };
        let ws2 = ResolvedWorkspace {
            canonical_path: tmp2.path().to_path_buf(),
            name: "ws2".to_string(),
            team: None,
            tags: vec![],
            status: WorkspaceStatus::Available,
        };

        let scanner = WorkspaceScanner::new(4);
        let results = scanner.scan_all(&[ws1, ws2], &index).await;

        assert_eq!(results.len(), 2, "should have results for both workspaces");

        let ws1_assets = results.get(tmp1.path()).expect("ws1 should be in results");
        assert_eq!(ws1_assets.len(), 1);
        assert_eq!(ws1_assets[0].asset_id, "agents/aws/cdk-agent");

        let ws2_assets = results.get(tmp2.path()).expect("ws2 should be in results");
        assert_eq!(ws2_assets.len(), 1);
        assert_eq!(ws2_assets[0].asset_id, "agents/gcp/vertex-agent");
    }

    // -----------------------------------------------------------------------
    // Property tests — Task 5.6
    // -----------------------------------------------------------------------

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        // -------------------------------------------------------------------
        // Property 18 — Multi-strategy detection confirmation (Req 7.2)
        //
        // For any combination of 0–3 detection signals, `is_confirmed` should
        // return true iff at least 2 DISTINCT signals are present.
        // -------------------------------------------------------------------

        prop_compose! {
            /// Generate a Vec<DetectionMethod> with 0..=3 entries, possibly
            /// containing duplicates (since real scanning could fire the same
            /// strategy twice through different code paths).
            fn arb_methods()(
                include_filename in any::<bool>(),
                include_metadata in any::<bool>(),
                include_content in any::<bool>(),
                dup_filename in any::<bool>(),
            ) -> Vec<DetectionMethod> {
                let mut methods = Vec::new();
                if include_filename { methods.push(DetectionMethod::Filename); }
                if include_metadata { methods.push(DetectionMethod::MetadataComment); }
                if include_content  { methods.push(DetectionMethod::ContentSignature); }
                // Optionally duplicate Filename to test dedup.
                if dup_filename && include_filename {
                    methods.push(DetectionMethod::Filename);
                }
                methods
            }
        }

        proptest! {
            #![proptest_config(proptest::test_runner::Config {
                cases: 256,
                ..Default::default()
            })]

            /// Property 18: confirmed == (number of distinct signals >= 2).
            #[test]
            fn prop18_confirmation_rule(methods in arb_methods()) {
                let distinct: std::collections::HashSet<_> = methods.iter().collect();
                let expected = distinct.len() >= 2;
                prop_assert_eq!(
                    is_confirmed(&methods),
                    expected,
                    "is_confirmed({:?}) should be {} (distinct={})",
                    methods, expected, distinct.len()
                );
            }
        }

        // -------------------------------------------------------------------
        // Property 19 — VFA-EXPORT metadata parsing (Req 7.7)
        //
        // For any valid {id, version, installed_at} serialised into a
        // `# VFA-EXPORT: {json}` line, parse_export_metadata recovers the id
        // and version.  Arbitrary non-matching lines return None without panic.
        // -------------------------------------------------------------------

        prop_compose! {
            /// Generate a valid asset ID: non-empty, no control chars.
            fn arb_asset_id()(
                s in "[a-z][a-z0-9_-]{1,20}(/[a-z][a-z0-9_-]{1,20}){0,3}"
            ) -> String {
                s
            }
        }

        prop_compose! {
            /// Generate an optional semver-like version string.
            fn arb_version()(
                major in 0u8..10,
                minor in 0u8..20,
                patch in 0u8..30,
                include in any::<bool>(),
            ) -> Option<String> {
                if include {
                    Some(format!("{major}.{minor}.{patch}"))
                } else {
                    None
                }
            }
        }

        proptest! {
            #![proptest_config(proptest::test_runner::Config {
                cases: 256,
                ..Default::default()
            })]

            /// Property 19a: round-trip — serialised VFA-EXPORT line parses back.
            #[test]
            fn prop19a_vfa_export_round_trip(
                id in arb_asset_id(),
                version in arb_version(),
                installed_at in prop::option::of("[0-9]{4}-[0-9]{2}-[0-9]{2}"),
            ) {
                let meta_orig = ExportMeta {
                    id: id.clone(),
                    version: version.clone(),
                    installed_at: installed_at.clone(),
                };
                let json = serde_json::to_string(&meta_orig).unwrap();
                let line = format!("# VFA-EXPORT: {json}");

                let parsed = WorkspaceScanner::parse_export_metadata(&line);
                prop_assert!(parsed.is_some(), "should parse valid VFA-EXPORT line");
                let parsed = parsed.unwrap();
                prop_assert_eq!(&parsed.id, &id);
                prop_assert_eq!(&parsed.version, &version);
            }

            /// Property 19b: arbitrary non-matching text never panics and
            /// returns None when there is no VFA-EXPORT line.
            #[test]
            fn prop19b_non_matching_returns_none_no_panic(
                // Generate arbitrary text that explicitly does NOT contain
                // the "VFA-EXPORT:" substring.
                s in "[^V]{0,200}"
            ) {
                // Must not contain "VFA-EXPORT:" (filter at strategy level).
                prop_assume!(!s.contains("VFA-EXPORT:"));
                // Must not panic.
                let result = WorkspaceScanner::parse_export_metadata(&s);
                prop_assert!(
                    result.is_none(),
                    "non-VFA-EXPORT input should return None, got {:?}", result
                );
            }
        }
    }
}
