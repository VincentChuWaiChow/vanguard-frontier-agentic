use std::path::{Path, PathBuf};

/// Identifies a supported harness directory type.
///
/// Each variant maps 1:1 to a well-known directory that a downstream workspace
/// may contain. The names are stable identifiers — use [`HarnessDir::dir_name`]
/// to obtain the on-disk directory name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HarnessDir {
    /// `.claude/` — Claude Code CLI agent definitions (markdown files).
    Claude,
    /// `.cursor/` — Cursor IDE agent configuration (JSON configs).
    Cursor,
    /// `.kiro/` — Kiro IDE steering files with agent references.
    Kiro,
    /// `.codex/` — Codex plugin.json-based agent entries.
    Codex,
    /// `.opencode/` — OpenCode TOML/YAML agent definitions.
    Opencode,
}

impl HarnessDir {
    /// Returns the directory name (without a leading `.`) as it appears on disk,
    /// e.g. `".claude"` for [`HarnessDir::Claude`].
    pub fn dir_name(&self) -> &'static str {
        match self {
            HarnessDir::Claude => ".claude",
            HarnessDir::Cursor => ".cursor",
            HarnessDir::Kiro => ".kiro",
            HarnessDir::Codex => ".codex",
            HarnessDir::Opencode => ".opencode",
        }
    }

    /// Iterate over all five known harness variants in declaration order.
    pub fn all() -> [HarnessDir; 5] {
        [
            HarnessDir::Claude,
            HarnessDir::Cursor,
            HarnessDir::Kiro,
            HarnessDir::Codex,
            HarnessDir::Opencode,
        ]
    }
}

/// Scans `workspace_root` for the five known harness directories and returns
/// every one that is present as a `(HarnessDir, PathBuf)` pair.
///
/// Only directories that actually exist on the filesystem are returned; the
/// order matches the declaration order in [`HarnessDir::all`].
pub fn detect_harness_dirs(workspace_root: &Path) -> Vec<(HarnessDir, PathBuf)> {
    HarnessDir::all()
        .into_iter()
        .filter_map(|h| {
            let path = workspace_root.join(h.dir_name());
            if path.is_dir() {
                Some((h, path))
            } else {
                None
            }
        })
        .collect()
}

/// Outcome of a harness-layout validation check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutMatch {
    /// The directory contains at least one file that matches the expected
    /// harness layout pattern.
    Matches,
    /// The directory exists but contains no files that match the expected
    /// layout, or it is empty.
    NoMatch,
}

/// Validates whether `path` matches the expected on-disk layout for `dir`.
///
/// Each harness has a characteristic layout heuristic:
///
/// | Harness   | Recognised layout signal                                           |
/// |-----------|-------------------------------------------------------------------|
/// | Claude    | At least one `.md` file (agent ID encoded in filename).           |
/// | Cursor    | At least one `.json` or `.md` file.                               |
/// | Kiro      | At least one `.md`, `.txt` or `.json` file.                       |
/// | Codex     | A `plugin.json` file, or at least one `.toml` agent file.         |
/// | Opencode  | At least one `.toml` or `.yaml`/`.yml` file (agent definitions).  |
///
/// Returns [`LayoutMatch::NoMatch`] for any I/O errors so callers can treat
/// an unreadable directory as "skip with warning".
pub fn validate_harness_layout(dir: &HarnessDir, path: &Path) -> LayoutMatch {
    match dir {
        HarnessDir::Claude => has_file_with_extensions(path, &[".md"]),
        // cursor exports write `.cursor/agents/<id>.md`, not JSON, so a
        // `.json`-only layout rejected a correct export.
        HarnessDir::Cursor => has_file_with_extensions(path, &[".json", ".md"]),
        // `.kiro/agents` receives both harnesses: kiro-ide writes `.md` and
        // kiro-cli writes `.json`. Accepting only `.md`/`.txt` meant a
        // kiro-cli-only export failed layout validation and its agents were
        // reported as not installed.
        HarnessDir::Kiro => has_file_with_extensions(path, &[".md", ".txt", ".json"]),
        // `vfa-export-agents --platform codex` writes `.codex/agents/<id>.toml`
        // and never a plugin.json, so requiring plugin.json alone meant a
        // correctly exported Codex workspace failed layout validation and its
        // agents were reported as not installed. Accept either signal.
        HarnessDir::Codex => match has_exact_file(path, "plugin.json") {
            LayoutMatch::Matches => LayoutMatch::Matches,
            LayoutMatch::NoMatch => has_file_with_extensions(path, &[".toml"]),
        },
        HarnessDir::Opencode => has_file_with_extensions(path, &[".toml", ".yaml", ".yml"]),
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Returns `LayoutMatch::Matches` if `dir` contains at least one regular file
/// whose name ends with any of the given `extensions` (case-sensitive).
/// Searches only the top level of the directory (non-recursive).
fn has_file_with_extensions(dir: &Path, extensions: &[&str]) -> LayoutMatch {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return LayoutMatch::NoMatch,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if extensions.iter().any(|ext| name.ends_with(ext)) {
                return LayoutMatch::Matches;
            }
        }
    }

    LayoutMatch::NoMatch
}

/// Returns `LayoutMatch::Matches` if `dir` contains a regular file with
/// exactly the given `filename` at the top level.
fn has_exact_file(dir: &Path, filename: &str) -> LayoutMatch {
    let candidate = dir.join(filename);
    if candidate.is_file() {
        LayoutMatch::Matches
    } else {
        LayoutMatch::NoMatch
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // -----------------------------------------------------------------------
    // detect_harness_dirs — presence detection
    // -----------------------------------------------------------------------

    #[test]
    fn detect_none_when_no_harness_dirs() {
        let tmp = TempDir::new().unwrap();
        let result = detect_harness_dirs(tmp.path());
        assert!(
            result.is_empty(),
            "expected no harness dirs, got {result:?}"
        );
    }

    #[test]
    fn detect_all_five_harness_dirs() {
        let tmp = TempDir::new().unwrap();
        for h in HarnessDir::all() {
            fs::create_dir(tmp.path().join(h.dir_name())).unwrap();
        }
        let result = detect_harness_dirs(tmp.path());
        assert_eq!(
            result.len(),
            5,
            "expected 5 harness dirs, got {}",
            result.len()
        );
        let found: Vec<&HarnessDir> = result.iter().map(|(h, _)| h).collect();
        assert!(found.contains(&&HarnessDir::Claude));
        assert!(found.contains(&&HarnessDir::Cursor));
        assert!(found.contains(&&HarnessDir::Kiro));
        assert!(found.contains(&&HarnessDir::Codex));
        assert!(found.contains(&&HarnessDir::Opencode));
    }

    #[test]
    fn detect_subset_of_harness_dirs() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".claude")).unwrap();
        fs::create_dir(tmp.path().join(".kiro")).unwrap();
        let result = detect_harness_dirs(tmp.path());
        assert_eq!(
            result.len(),
            2,
            "expected 2 harness dirs, got {}",
            result.len()
        );
        let found: Vec<&HarnessDir> = result.iter().map(|(h, _)| h).collect();
        assert!(found.contains(&&HarnessDir::Claude));
        assert!(found.contains(&&HarnessDir::Kiro));
        assert!(!found.contains(&&HarnessDir::Cursor));
    }

    #[test]
    fn detect_returns_absolute_paths() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".cursor")).unwrap();
        let result = detect_harness_dirs(tmp.path());
        assert_eq!(result.len(), 1);
        let (_, path) = &result[0];
        assert!(path.is_absolute(), "expected absolute path, got {path:?}");
        assert_eq!(path, &tmp.path().join(".cursor"));
    }

    #[test]
    fn detect_ignores_files_named_like_harness_dirs() {
        // A file named `.claude` (not a directory) should NOT be detected.
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".claude"), "not a dir").unwrap();
        let result = detect_harness_dirs(tmp.path());
        assert!(
            result.is_empty(),
            "files named like harness dirs should be ignored: {result:?}"
        );
    }

    // -----------------------------------------------------------------------
    // validate_harness_layout — Claude (.md files)
    // -----------------------------------------------------------------------

    #[test]
    fn claude_layout_matches_md_file() {
        let tmp = TempDir::new().unwrap();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir(&claude_dir).unwrap();
        fs::write(claude_dir.join("security-agent-v1.md"), "# agent content").unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Claude, &claude_dir),
            LayoutMatch::Matches
        );
    }

    #[test]
    fn claude_layout_no_match_wrong_extension() {
        let tmp = TempDir::new().unwrap();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir(&claude_dir).unwrap();
        fs::write(claude_dir.join("config.json"), "{}").unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Claude, &claude_dir),
            LayoutMatch::NoMatch
        );
    }

    #[test]
    fn claude_layout_no_match_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir(&claude_dir).unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Claude, &claude_dir),
            LayoutMatch::NoMatch
        );
    }

    // -----------------------------------------------------------------------
    // validate_harness_layout — Cursor (.json files)
    // -----------------------------------------------------------------------

    #[test]
    fn cursor_layout_matches_json_file() {
        let tmp = TempDir::new().unwrap();
        let cursor_dir = tmp.path().join(".cursor");
        fs::create_dir(&cursor_dir).unwrap();
        fs::write(cursor_dir.join("agents.json"), r#"{"agents": []}"#).unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Cursor, &cursor_dir),
            LayoutMatch::Matches
        );
    }

    /// Contract change: this previously asserted that a `.md` file must NOT
    /// satisfy the Cursor layout, which contradicted the exporter — cursor
    /// exports are `.cursor/agents/<id>.md`, so the rule rejected every correct
    /// export. Markdown now satisfies it.
    ///
    /// The original intent, that a stray document should not be mistaken for an
    /// installed asset, is preserved elsewhere and more strongly: layout is only
    /// a cheap pre-filter for walking a directory, and confirmation still
    /// requires two independent detection signals, which a readme has neither of.
    #[test]
    fn cursor_layout_matches_markdown_because_exports_are_markdown() {
        let tmp = TempDir::new().unwrap();
        let cursor_dir = tmp.path().join(".cursor");
        fs::create_dir(&cursor_dir).unwrap();
        fs::write(cursor_dir.join("readme.md"), "docs").unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Cursor, &cursor_dir),
            LayoutMatch::Matches
        );
    }

    #[test]
    fn cursor_layout_no_match_unrelated_extension() {
        let tmp = TempDir::new().unwrap();
        let cursor_dir = tmp.path().join(".cursor");
        fs::create_dir(&cursor_dir).unwrap();
        fs::write(cursor_dir.join("notes.txt"), "notes").unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Cursor, &cursor_dir),
            LayoutMatch::NoMatch
        );
    }

    // -----------------------------------------------------------------------
    // validate_harness_layout — Kiro (.md or .txt steering files)
    // -----------------------------------------------------------------------

    #[test]
    fn kiro_layout_matches_md_file() {
        let tmp = TempDir::new().unwrap();
        let kiro_dir = tmp.path().join(".kiro");
        fs::create_dir(&kiro_dir).unwrap();
        fs::write(kiro_dir.join("steering.md"), "## steering").unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Kiro, &kiro_dir),
            LayoutMatch::Matches
        );
    }

    #[test]
    fn kiro_layout_matches_txt_file() {
        let tmp = TempDir::new().unwrap();
        let kiro_dir = tmp.path().join(".kiro");
        fs::create_dir(&kiro_dir).unwrap();
        fs::write(kiro_dir.join("agents.txt"), "agent-ref-1\nagent-ref-2").unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Kiro, &kiro_dir),
            LayoutMatch::Matches
        );
    }

    /// Contract change: this previously asserted that a `.json` file must NOT
    /// satisfy the Kiro layout. `.kiro/agents` receives both harnesses — kiro-ide
    /// writes `.md` and kiro-cli writes `.json` — so the rule rejected every
    /// correct kiro-cli export. JSON now satisfies it; confirmation still needs
    /// two independent detection signals, which a bare config file lacks.
    #[test]
    fn kiro_layout_matches_json_because_kiro_cli_exports_json() {
        let tmp = TempDir::new().unwrap();
        let kiro_dir = tmp.path().join(".kiro");
        fs::create_dir(&kiro_dir).unwrap();
        fs::write(kiro_dir.join("config.json"), "{}").unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Kiro, &kiro_dir),
            LayoutMatch::Matches
        );
    }

    #[test]
    fn kiro_layout_no_match_unrelated_extension() {
        let tmp = TempDir::new().unwrap();
        let kiro_dir = tmp.path().join(".kiro");
        fs::create_dir(&kiro_dir).unwrap();
        fs::write(kiro_dir.join("notes.rst"), "notes").unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Kiro, &kiro_dir),
            LayoutMatch::NoMatch
        );
    }

    // -----------------------------------------------------------------------
    // validate_harness_layout — Codex (plugin.json)
    // -----------------------------------------------------------------------

    #[test]
    fn codex_layout_matches_plugin_json() {
        let tmp = TempDir::new().unwrap();
        let codex_dir = tmp.path().join(".codex");
        fs::create_dir(&codex_dir).unwrap();
        fs::write(codex_dir.join("plugin.json"), r#"{"name": "my-plugin"}"#).unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Codex, &codex_dir),
            LayoutMatch::Matches
        );
    }

    #[test]
    fn codex_layout_no_match_wrong_filename() {
        let tmp = TempDir::new().unwrap();
        let codex_dir = tmp.path().join(".codex");
        fs::create_dir(&codex_dir).unwrap();
        // "plugins.json" ≠ "plugin.json"
        fs::write(codex_dir.join("plugins.json"), r#"{"name": "my-plugin"}"#).unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Codex, &codex_dir),
            LayoutMatch::NoMatch
        );
    }

    /// Regression: `vfa-export-agents --platform codex` writes
    /// `.codex/agents/<id>.toml` and no plugin.json. Requiring plugin.json
    /// alone made a correctly exported Codex workspace fail layout validation,
    /// so its agents were reported as not installed.
    #[test]
    fn codex_layout_matches_exported_toml_agents() {
        let tmp = TempDir::new().unwrap();
        let codex_agents = tmp.path().join(".codex").join("agents");
        fs::create_dir_all(&codex_agents).unwrap();
        fs::write(
            codex_agents.join("aws-iam-least-privilege-review-agent.toml"),
            "name = \"aws-iam-least-privilege-review-agent\"\n",
        )
        .unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Codex, &codex_agents),
            LayoutMatch::Matches
        );
    }

    /// Regression: `vfa-export-agents --platform kiro-cli` writes
    /// `.kiro/agents/<id>.json`, which the `.md`/`.txt` layout rejected.
    #[test]
    fn kiro_layout_matches_exported_json_agents() {
        let tmp = TempDir::new().unwrap();
        let kiro_agents = tmp.path().join(".kiro").join("agents");
        fs::create_dir_all(&kiro_agents).unwrap();
        fs::write(
            kiro_agents.join("aws-iam-least-privilege-review-agent.json"),
            r#"{"x-vfa-export":{"id":"aws-iam-least-privilege-review-agent"},"name":"IAM"}"#,
        )
        .unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Kiro, &kiro_agents),
            LayoutMatch::Matches
        );
    }

    /// Regression: `vfa-export-agents --platform cursor` writes
    /// `.cursor/agents/<id>.md`, which the `.json`-only layout rejected.
    #[test]
    fn cursor_layout_matches_exported_md_agents() {
        let tmp = TempDir::new().unwrap();
        let cursor_agents = tmp.path().join(".cursor").join("agents");
        fs::create_dir_all(&cursor_agents).unwrap();
        fs::write(
            cursor_agents.join("aws-iam-least-privilege-review-agent.md"),
            "---\nname: IAM\n---\nbody\n",
        )
        .unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Cursor, &cursor_agents),
            LayoutMatch::Matches
        );
    }

    #[test]
    fn codex_layout_no_match_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let codex_dir = tmp.path().join(".codex");
        fs::create_dir(&codex_dir).unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Codex, &codex_dir),
            LayoutMatch::NoMatch
        );
    }

    // -----------------------------------------------------------------------
    // validate_harness_layout — Opencode (.toml / .yaml / .yml)
    // -----------------------------------------------------------------------

    #[test]
    fn opencode_layout_matches_toml_file() {
        let tmp = TempDir::new().unwrap();
        let oc_dir = tmp.path().join(".opencode");
        fs::create_dir(&oc_dir).unwrap();
        fs::write(oc_dir.join("agent.toml"), "[agent]\nname = \"foo\"").unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Opencode, &oc_dir),
            LayoutMatch::Matches
        );
    }

    #[test]
    fn opencode_layout_matches_yaml_file() {
        let tmp = TempDir::new().unwrap();
        let oc_dir = tmp.path().join(".opencode");
        fs::create_dir(&oc_dir).unwrap();
        fs::write(oc_dir.join("agent.yaml"), "name: foo").unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Opencode, &oc_dir),
            LayoutMatch::Matches
        );
    }

    #[test]
    fn opencode_layout_matches_yml_file() {
        let tmp = TempDir::new().unwrap();
        let oc_dir = tmp.path().join(".opencode");
        fs::create_dir(&oc_dir).unwrap();
        fs::write(oc_dir.join("agent.yml"), "name: foo").unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Opencode, &oc_dir),
            LayoutMatch::Matches
        );
    }

    #[test]
    fn opencode_layout_no_match_json_only() {
        let tmp = TempDir::new().unwrap();
        let oc_dir = tmp.path().join(".opencode");
        fs::create_dir(&oc_dir).unwrap();
        fs::write(oc_dir.join("config.json"), "{}").unwrap();
        assert_eq!(
            validate_harness_layout(&HarnessDir::Opencode, &oc_dir),
            LayoutMatch::NoMatch
        );
    }

    // -----------------------------------------------------------------------
    // validate_harness_layout — nonexistent / unreadable dir
    // -----------------------------------------------------------------------

    #[test]
    fn validate_nonexistent_dir_returns_no_match() {
        let tmp = TempDir::new().unwrap();
        let ghost = tmp.path().join(".ghost");
        // ghost does not exist at all
        assert_eq!(
            validate_harness_layout(&HarnessDir::Claude, &ghost),
            LayoutMatch::NoMatch
        );
    }

    // -----------------------------------------------------------------------
    // HarnessDir helpers
    // -----------------------------------------------------------------------

    #[test]
    fn harness_dir_names_are_correct() {
        assert_eq!(HarnessDir::Claude.dir_name(), ".claude");
        assert_eq!(HarnessDir::Cursor.dir_name(), ".cursor");
        assert_eq!(HarnessDir::Kiro.dir_name(), ".kiro");
        assert_eq!(HarnessDir::Codex.dir_name(), ".codex");
        assert_eq!(HarnessDir::Opencode.dir_name(), ".opencode");
    }

    #[test]
    fn harness_dir_all_returns_five_variants() {
        assert_eq!(HarnessDir::all().len(), 5);
    }
}
