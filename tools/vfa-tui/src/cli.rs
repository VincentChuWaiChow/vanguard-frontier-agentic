//! CLI argument parsing — clap 4 derive interface (Tasks 9.9 / 9.10).
//!
//! # Exit codes (documented per Req 18.5)
//!
//! ```text
//! 0 — Success, no violations
//! 1 — Compliance failures detected (violations, content drift, stale assets
//!     exceeding threshold, gate failures)
//! 2 — Operational error (invalid config, missing registry, missing catalog
//!     directory, inaccessible resources); also used for usage errors
//!     (unrecognized flags, invalid values)
//! 3 — Partial catalog failure (catalog directory exists but individual files
//!     are corrupted or unreadable)
//! ```
//!
//! Multiple conditions → highest severity wins (3 > 2 > 1 > 0).

#![deny(warnings)]

use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use crate::models::report::{OutputFormat, ReportType};
use crate::ui::theme::ThemePreference;

// ---------------------------------------------------------------------------
// Cli
// ---------------------------------------------------------------------------

/// Platform operator console for agentic asset governance.
///
/// # Exit codes
///
/// ```text
/// 0  Success — no violations
/// 1  Compliance failures (violations, content drift, stale threshold, gate failures)
/// 2  Operational error (invalid config, missing registry, bad flags)
/// 3  Partial catalog failure (catalog dir exists but files corrupted)
/// ```
#[derive(Debug, Parser)]
#[command(
    name = "vfa-tui",
    version,
    about = "Platform operator console for agentic asset governance",
    long_about = concat!(
        "Platform operator console for agentic asset governance.\n\n",
        "EXIT CODES\n",
        "  0  Success — no violations\n",
        "  1  Compliance failures (violations, content drift, stale threshold, gate failures)\n",
        "  2  Operational error (invalid config, missing registry, invalid flags)\n",
        "  3  Partial catalog failure (catalog directory exists but files are corrupted)\n\n",
        "Multiple failure conditions simultaneously → highest severity code wins (3 > 2 > 1 > 0)."
    )
)]
pub struct Cli {
    // -----------------------------------------------------------------------
    // Pre-existing flags (preserved)
    // -----------------------------------------------------------------------
    /// Path to the workspace root (auto-detected if omitted).
    #[arg(long)]
    pub workspace: Option<PathBuf>,

    /// Path to the log file for audit output.
    #[arg(long)]
    pub log_file: Option<PathBuf>,

    /// Logging verbosity level.
    #[arg(long, value_enum, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,

    /// Disable colored output (also honoured via NO_COLOR env var).
    #[arg(long)]
    pub no_color: bool,

    /// Color theme mode: `auto` (detect terminal background), `dark`, or `light`.
    #[arg(long, value_enum, default_value_t = ThemePreference::Auto)]
    pub theme: ThemePreference,

    // -----------------------------------------------------------------------
    // New flags (Req 26.1–26.8)
    // -----------------------------------------------------------------------
    /// Path to the workspace registry TOML file.
    /// Bug #3: Tilde (~) expanded after clap processing via expand_home_paths().
    #[arg(long, default_value = "~/.config/vfa/workspaces.toml")]
    pub registry: String,

    /// Path to the policy rules TOML file.
    /// Bug #3: Tilde (~) expanded after clap processing via expand_home_paths().
    #[arg(long, default_value = "~/.config/vfa/policies.toml")]
    pub policies: String,

    /// Path to the SQLite index file.
    /// Bug #3: Tilde (~) expanded after clap processing via expand_home_paths().
    #[arg(long, default_value = "~/.local/share/vfa/index.db")]
    pub index_path: String,

    /// Internal marker: whether home expansion has been applied to default paths.
    #[arg(skip)]
    pub _home_expanded: bool,

    /// Produce a headless report instead of launching the TUI.
    ///
    /// Supported types: coverage, violations, drift, stale, gates, integrity,
    /// versions, dependencies, lifecycle, summary, all.
    #[arg(long, value_delimiter = ',')]
    pub report: Option<Vec<ReportTypeCli>>,

    /// Output format for headless mode.
    ///
    /// Supported formats: json (default), markdown, table.
    #[arg(long, default_value = "json")]
    pub format: OutputFormatCli,

    /// Restrict headless operations to workspaces matching this glob pattern.
    #[arg(long)]
    pub workspace_filter: Option<String>,

    /// Force a complete index rebuild from a fresh filesystem scan.
    #[arg(long)]
    pub rebuild_index: bool,

    /// Suppress progress output; emit only the final structured report.
    #[arg(long)]
    pub quiet: bool,

    /// Permit a headless report to run with no policy rules loaded.
    ///
    /// Without this flag an empty policy set is an operational error: a run
    /// that evaluated nothing must not report 100% compliance.  Exploratory
    /// runs opt in explicitly so that enforcement stays fail-closed.
    #[arg(long)]
    pub allow_empty_policy: bool,

    /// Parse all configuration files, report errors, and exit without running.
    #[arg(long)]
    pub validate_config: bool,

    /// Export the audit log.  Format: `<format> <output-path>` (e.g. `json /tmp/audit.json`).
    #[arg(long, num_args = 2, value_names = ["FORMAT", "PATH"])]
    pub export_audit: Option<Vec<String>>,

    /// Start the embedded web server (stretch goal, Req 33).
    #[arg(long)]
    pub web: bool,

    /// Web server bind address (only used when --web is active).
    #[arg(long, default_value = "127.0.0.1:8080")]
    pub web_bind: String,
}

impl Cli {
    /// Expand home directory (~) in default paths.
    /// This should be called after parsing to resolve ~ to the user's home.
    pub fn expand_home_paths(&mut self) {
        self.registry = expand_tilde(&self.registry);
        self.policies = expand_tilde(&self.policies);
        self.index_path = expand_tilde(&self.index_path);
    }

    /// Returns `true` when color output should be disabled.
    ///
    /// Color is disabled when either `--no-color` was passed **or** the
    /// `NO_COLOR` environment variable is set to any value (per no-color.org,
    /// Req 29.1 / 29.3).  This only affects formatting; it never alters report
    /// data (Req 27.4).
    pub fn is_no_color(&self) -> bool {
        self.no_color || std::env::var_os("NO_COLOR").is_some()
    }

    /// Convert the CLI `--report` list into `models::ReportType` values.
    pub fn report_types(&self) -> Vec<ReportType> {
        match &self.report {
            None => vec![],
            Some(types) => types.iter().map(|t| t.to_model()).collect(),
        }
    }

    /// Convert the CLI `--format` into `models::OutputFormat`.
    pub fn output_format(&self) -> OutputFormat {
        self.format.to_model()
    }
}

// ---------------------------------------------------------------------------
// ReportTypeCli — clap ValueEnum wrapper for ReportType
// ---------------------------------------------------------------------------

/// Headless report type (CLI parse target; maps to [`ReportType`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ReportTypeCli {
    Coverage,
    Violations,
    Drift,
    Stale,
    Gates,
    Integrity,
    Versions,
    Dependencies,
    Lifecycle,
    Summary,
    All,
}

impl ReportTypeCli {
    /// Convert to the canonical model enum.
    pub fn to_model(self) -> ReportType {
        match self {
            Self::Coverage => ReportType::Coverage,
            Self::Violations => ReportType::Violations,
            Self::Drift => ReportType::Drift,
            Self::Stale => ReportType::Stale,
            Self::Gates => ReportType::Gates,
            Self::Integrity => ReportType::Integrity,
            Self::Versions => ReportType::Versions,
            Self::Dependencies => ReportType::Dependencies,
            Self::Lifecycle => ReportType::Lifecycle,
            Self::Summary => ReportType::Summary,
            Self::All => ReportType::All,
        }
    }
}

// ---------------------------------------------------------------------------
// OutputFormatCli — clap ValueEnum wrapper for OutputFormat
// ---------------------------------------------------------------------------

/// Output format selector (CLI parse target; maps to [`OutputFormat`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum OutputFormatCli {
    #[default]
    Json,
    Markdown,
    Table,
}

impl OutputFormatCli {
    /// Convert to the canonical model enum.
    pub fn to_model(self) -> OutputFormat {
        match self {
            Self::Json => OutputFormat::Json,
            Self::Markdown => OutputFormat::Markdown,
            Self::Table => OutputFormat::Table,
        }
    }
}

// ---------------------------------------------------------------------------
// LogLevel
// ---------------------------------------------------------------------------

/// Supported log levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

// ---------------------------------------------------------------------------
// Home directory expansion helper
// ---------------------------------------------------------------------------

/// Expand `~` to the user's home directory.
/// If the path starts with `~/`, replace the `~` with the home directory.
/// Otherwise, return the path unchanged.
fn expand_tilde(path: &str) -> String {
    if !path.starts_with('~') {
        return path.to_string();
    }

    match std::env::var("HOME") {
        Ok(home) => {
            if path == "~" {
                home
            } else if path.starts_with("~/") {
                format!("{}{}", home, &path[1..])
            } else {
                path.to_string()
            }
        }
        Err(_) => path.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tests (Task 9.10)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    // -----------------------------------------------------------------------
    // Helper
    // -----------------------------------------------------------------------

    /// Parse a `Cli` from a string slice, returning the parsed struct on
    /// success or the error text on failure.
    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(args)
    }

    fn ok(args: &[&str]) -> Cli {
        parse(args).expect("expected successful parse")
    }

    // -----------------------------------------------------------------------
    // Basic flags
    // -----------------------------------------------------------------------

    #[test]
    fn default_parse_succeeds() {
        let cli = ok(&["vfa-tui"]);
        assert!(!cli.no_color);
        assert!(!cli.quiet);
        assert!(!cli.rebuild_index);
        assert!(!cli.web);
        assert!(cli.report.is_none());
        assert_eq!(cli.format, OutputFormatCli::Json);
    }

    #[test]
    fn no_color_flag() {
        let cli = ok(&["vfa-tui", "--no-color"]);
        assert!(cli.no_color);
    }

    #[test]
    fn quiet_flag() {
        let cli = ok(&["vfa-tui", "--quiet"]);
        assert!(cli.quiet);
    }

    #[test]
    fn rebuild_index_flag() {
        let cli = ok(&["vfa-tui", "--rebuild-index"]);
        assert!(cli.rebuild_index);
    }

    #[test]
    fn validate_config_flag() {
        let cli = ok(&["vfa-tui", "--validate-config"]);
        assert!(cli.validate_config);
    }

    #[test]
    fn web_flag() {
        let cli = ok(&["vfa-tui", "--web"]);
        assert!(cli.web);
    }

    #[test]
    fn web_bind_custom() {
        let cli = ok(&["vfa-tui", "--web", "--web-bind", "0.0.0.0:9090"]);
        assert_eq!(cli.web_bind, "0.0.0.0:9090");
    }

    // -----------------------------------------------------------------------
    // --registry / --policies / --index-path
    // -----------------------------------------------------------------------

    #[test]
    fn registry_path() {
        let cli = ok(&["vfa-tui", "--registry", "/custom/workspaces.toml"]);
        assert_eq!(cli.registry, "/custom/workspaces.toml");
    }

    #[test]
    fn policies_path() {
        let cli = ok(&["vfa-tui", "--policies", "/custom/policies.toml"]);
        assert_eq!(cli.policies, "/custom/policies.toml");
    }

    #[test]
    fn index_path() {
        let cli = ok(&["vfa-tui", "--index-path", "/tmp/test.db"]);
        assert_eq!(cli.index_path, "/tmp/test.db");
    }

    // -----------------------------------------------------------------------
    // --report
    // -----------------------------------------------------------------------

    #[test]
    fn report_coverage() {
        let cli = ok(&["vfa-tui", "--report", "coverage"]);
        let types = cli.report_types();
        assert_eq!(types, vec![ReportType::Coverage]);
    }

    #[test]
    fn report_all_types_individually() {
        let type_pairs = [
            ("coverage", ReportType::Coverage),
            ("violations", ReportType::Violations),
            ("drift", ReportType::Drift),
            ("stale", ReportType::Stale),
            ("gates", ReportType::Gates),
            ("integrity", ReportType::Integrity),
            ("versions", ReportType::Versions),
            ("dependencies", ReportType::Dependencies),
            ("lifecycle", ReportType::Lifecycle),
            ("summary", ReportType::Summary),
            ("all", ReportType::All),
        ];
        for (name, expected) in &type_pairs {
            let cli = ok(&["vfa-tui", "--report", name]);
            let types = cli.report_types();
            assert_eq!(types, vec![expected.clone()], "failed for '{name}'");
        }
    }

    #[test]
    fn report_multiple_comma_separated() {
        let cli = ok(&["vfa-tui", "--report", "coverage,violations"]);
        let types = cli.report_types();
        assert_eq!(types, vec![ReportType::Coverage, ReportType::Violations]);
    }

    #[test]
    fn report_invalid_type_exits_2() {
        let err = parse(&["vfa-tui", "--report", "notavalidtype"]).unwrap_err();
        // clap exits with UsageError for invalid enum values
        assert_eq!(
            err.exit_code(),
            2,
            "invalid report type should give exit code 2"
        );
    }

    // -----------------------------------------------------------------------
    // --format
    // -----------------------------------------------------------------------

    #[test]
    fn format_json_default() {
        let cli = ok(&["vfa-tui"]);
        assert_eq!(cli.output_format(), OutputFormat::Json);
    }

    #[test]
    fn format_markdown() {
        let cli = ok(&["vfa-tui", "--format", "markdown"]);
        assert_eq!(cli.output_format(), OutputFormat::Markdown);
    }

    #[test]
    fn format_table() {
        let cli = ok(&["vfa-tui", "--format", "table"]);
        assert_eq!(cli.output_format(), OutputFormat::Table);
    }

    #[test]
    fn format_invalid_exits_2() {
        let err = parse(&["vfa-tui", "--format", "xml"]).unwrap_err();
        assert_eq!(err.exit_code(), 2, "invalid format should give exit code 2");
    }

    // -----------------------------------------------------------------------
    // --workspace-filter
    // -----------------------------------------------------------------------

    #[test]
    fn workspace_filter_set() {
        let cli = ok(&["vfa-tui", "--workspace-filter", "prod-*"]);
        assert_eq!(cli.workspace_filter.as_deref(), Some("prod-*"));
    }

    // -----------------------------------------------------------------------
    // --export-audit
    // -----------------------------------------------------------------------

    #[test]
    fn export_audit_json() {
        let cli = ok(&["vfa-tui", "--export-audit", "json", "/tmp/audit.json"]);
        let parts = cli.export_audit.as_ref().unwrap();
        assert_eq!(parts[0], "json");
        assert_eq!(parts[1], "/tmp/audit.json");
    }

    // -----------------------------------------------------------------------
    // NO_COLOR env var (Req 29.3)
    // -----------------------------------------------------------------------

    #[test]
    fn no_color_env_var_detected() {
        // is_no_color() should return true when NO_COLOR is set.
        // We cannot set the env var in a test safely (races), so test the
        // flag-based path and verify the logic via a separate helper.
        let cli = ok(&["vfa-tui", "--no-color"]);
        assert!(
            cli.is_no_color(),
            "--no-color flag should trigger is_no_color"
        );

        let cli2 = ok(&["vfa-tui"]);
        // Without the env var, should be false (env var may or may not be set in
        // the test environment — we can't control that, so we only test the flag path).
        // This assertion is only safe when NO_COLOR is absent.
        if std::env::var_os("NO_COLOR").is_none() {
            assert!(
                !cli2.is_no_color(),
                "without flag or env var, is_no_color should be false"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Unknown flag → exit 2
    // -----------------------------------------------------------------------

    #[test]
    fn unknown_flag_exits_2() {
        let err = parse(&["vfa-tui", "--not-a-real-flag"]).unwrap_err();
        assert_eq!(err.exit_code(), 2, "unknown flag should give exit code 2");
    }

    // -----------------------------------------------------------------------
    // Combined valid flag combinations
    // -----------------------------------------------------------------------

    #[test]
    fn combined_headless_flags() {
        let cli = ok(&[
            "vfa-tui",
            "--report",
            "coverage",
            "--format",
            "markdown",
            "--quiet",
            "--registry",
            "/tmp/ws.toml",
            "--policies",
            "/tmp/pol.toml",
            "--workspace-filter",
            "team-*",
        ]);
        assert_eq!(cli.report_types(), vec![ReportType::Coverage]);
        assert_eq!(cli.output_format(), OutputFormat::Markdown);
        assert!(cli.quiet);
        assert_eq!(cli.registry, "/tmp/ws.toml");
        assert_eq!(cli.policies, "/tmp/pol.toml");
        assert_eq!(cli.workspace_filter.as_deref(), Some("team-*"));
    }

    #[test]
    fn log_level_debug() {
        let cli = ok(&["vfa-tui", "--log-level", "debug"]);
        assert_eq!(cli.log_level, LogLevel::Debug);
    }

    // -----------------------------------------------------------------------
    // --theme (Req 35.3)
    // -----------------------------------------------------------------------

    #[test]
    fn theme_flag_defaults_to_auto() {
        assert_eq!(ok(&["vfa-tui"]).theme, ThemePreference::Auto);
    }

    #[test]
    fn theme_flag_parses_all_variants() {
        assert_eq!(
            ok(&["vfa-tui", "--theme", "auto"]).theme,
            ThemePreference::Auto
        );
        assert_eq!(
            ok(&["vfa-tui", "--theme", "dark"]).theme,
            ThemePreference::Dark
        );
        assert_eq!(
            ok(&["vfa-tui", "--theme", "light"]).theme,
            ThemePreference::Light
        );
    }

    #[test]
    fn theme_flag_invalid_exits_2() {
        let err = parse(&["vfa-tui", "--theme", "sepia"]).unwrap_err();
        assert_eq!(err.exit_code(), 2, "invalid theme should give exit code 2");
    }

    #[test]
    fn theme_and_no_color_are_independent() {
        // A user can pass both; --no-color wins at render time, but the theme
        // preference still parses to its own field.
        let cli = ok(&["vfa-tui", "--no-color", "--theme", "light"]);
        assert!(cli.no_color);
        assert_eq!(cli.theme, ThemePreference::Light);
    }
}
