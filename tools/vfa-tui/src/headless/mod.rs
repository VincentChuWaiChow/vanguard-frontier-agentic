//! Headless module — structured output generation and formatters.

pub mod formats;
pub mod reporter;

// Re-export the most commonly used public items.
pub use crate::models::report::{
    ReportInputStatus, ReportProducer, ReportProvenance, ReportQuality, ReportRequest,
    ReportResult, ReportWorkspaceProvenance,
};
pub use formats::{
    format_json, format_markdown, format_table, sort_by_id, with_status, ReportData, STATUS_DRIFT,
    STATUS_FAIL, STATUS_MISSING, STATUS_PASS, STATUS_STALE, STATUS_WARN,
};
pub use reporter::{
    all_report_types, compute_exit_code, FindingSeverity, HeadlessReporter, ReportService,
};
