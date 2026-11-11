//! Headless reporter data models — output format, report types, and structured output.

use serde::Serialize;
use std::path::PathBuf;

/// Caller identity; the shared engine never substitutes its package version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReportProducer {
    pub name: String,
    pub version: String,
}

/// Owned, adapter-neutral inputs. Paths are selected by the caller, not CWD.
#[derive(Debug, Clone)]
pub struct ReportRequest {
    pub catalog_root: PathBuf,
    pub registry_path: PathBuf,
    pub policy_path: PathBuf,
    /// Recorded for provenance; generation never opens or writes this index.
    pub index_path: PathBuf,
    pub report_types: Vec<ReportType>,
    pub format: OutputFormat,
    pub workspace_filter: Option<String>,
    pub producer: ReportProducer,
}

/// Source observation quality, independent of the legacy payload and exit code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReportInputStatus {
    Current,
    NotConfigured,
    Unavailable,
    Partial,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportQuality {
    pub catalog: ReportInputStatus,
    pub registry: ReportInputStatus,
    pub policy: ReportInputStatus,
    /// Registry resolution quality, not completeness of scanner file reads.
    /// A valid registry with zero selected workspaces is current, not scan proof.
    pub workspaces: ReportInputStatus,
    pub expected_workspaces: usize,
    pub available_workspaces: usize,
}

/// Registry resolution observation, not an assertion that every file was readable.
#[derive(Debug, Clone, Serialize)]
pub struct ReportWorkspaceProvenance {
    pub path: PathBuf,
    pub name: String,
    pub status: ReportInputStatus,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportProvenance {
    /// Selected paths are preserved as supplied, not claimed to be canonical.
    pub catalog_root: PathBuf,
    pub registry_path: PathBuf,
    pub policy_path: PathBuf,
    pub index_path: PathBuf,
    pub observed_at: String,
    pub workspaces: Vec<ReportWorkspaceProvenance>,
}

/// Owned report plus explicit evidence quality. Metadata is not injected into
/// `payload`: doing so would change the existing executable JSON contract.
#[derive(Debug)]
pub struct ReportResult {
    pub payload: serde_json::Value,
    pub exit_code: u8,
    pub producer: ReportProducer,
    pub quality: ReportQuality,
    pub provenance: ReportProvenance,
    /// Ordered legacy progress/error messages; generation itself never prints.
    pub diagnostics: Vec<String>,
    // Only the terminal adapter consumes this write plan. Core generation has
    // the identical public result but does not need to retain adapter cache data.
    #[cfg(feature = "terminal")]
    pub(crate) persistence: crate::headless::reporter::ReportPersistence,
}

/// Which kind of report to produce in headless mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportType {
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

/// Output format requested by the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Markdown,
    Table,
}

/// Top-level envelope for a single headless report section.
///
/// `data` is flattened into the JSON output so that consumers receive a single
/// object with `report_type`, `timestamp`, `console_version`, `exit_code`, and
/// all report-specific keys at the top level.
#[derive(Debug, Serialize)]
pub struct HeadlessOutput {
    /// String representation of the [`ReportType`] that produced this output.
    pub report_type: String,
    /// ISO 8601 timestamp of when the report was generated.
    pub timestamp: String,
    /// Version of the `vfa-tui` binary that produced this output.
    pub console_version: String,
    /// Proposed exit code for this report section.
    pub exit_code: i32,
    /// Arbitrary report payload — flattened into the enclosing JSON object.
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_type_equality() {
        assert_eq!(ReportType::Coverage, ReportType::Coverage);
        assert_ne!(ReportType::Coverage, ReportType::Violations);
        assert_eq!(ReportType::All, ReportType::All);
    }

    #[test]
    fn output_format_equality() {
        assert_eq!(OutputFormat::Json, OutputFormat::Json);
        assert_ne!(OutputFormat::Json, OutputFormat::Markdown);
        assert_eq!(OutputFormat::Table, OutputFormat::Table);
    }

    #[test]
    fn headless_output_serializes_correctly() {
        let output = HeadlessOutput {
            report_type: "coverage".to_string(),
            timestamp: "2025-01-01T00:00:00.000Z".to_string(),
            console_version: "1.0.0".to_string(),
            exit_code: 0,
            data: serde_json::json!({"aggregate_score": 95.0}),
        };
        let json = serde_json::to_string(&output).expect("serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("parse");

        assert_eq!(parsed["report_type"], "coverage");
        assert_eq!(parsed["timestamp"], "2025-01-01T00:00:00.000Z");
        assert_eq!(parsed["console_version"], "1.0.0");
        assert_eq!(parsed["exit_code"], 0);
        // flattened key from `data`:
        assert_eq!(parsed["aggregate_score"], 95.0);
    }

    #[test]
    fn headless_output_flatten_merges_data() {
        // Confirm that `#[serde(flatten)]` on `data` merges keys at the top level.
        let output = HeadlessOutput {
            report_type: "gates".to_string(),
            timestamp: "2025-06-01T00:00:00.000Z".to_string(),
            console_version: "2.0.0".to_string(),
            exit_code: 1,
            data: serde_json::json!({"gates_passed": 3, "gates_failed": 1}),
        };
        let json = serde_json::to_string(&output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["exit_code"], 1);
        assert_eq!(parsed["gates_passed"], 3);
        assert_eq!(parsed["gates_failed"], 1);
        // ensure no nested "data" key leaks through
        assert!(parsed.get("data").is_none());
    }

    #[test]
    fn report_type_all_variants_are_distinct() {
        let variants = [
            ReportType::Coverage,
            ReportType::Violations,
            ReportType::Drift,
            ReportType::Stale,
            ReportType::Gates,
            ReportType::Integrity,
            ReportType::Versions,
            ReportType::Dependencies,
            ReportType::Lifecycle,
            ReportType::Summary,
            ReportType::All,
        ];
        // Each variant is equal only to itself.
        for (i, a) in variants.iter().enumerate() {
            for (j, b) in variants.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }
}
