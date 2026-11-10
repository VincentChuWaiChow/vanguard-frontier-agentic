mod audit_logging;
mod catalog_loading;
#[cfg(feature = "terminal")]
mod headless_persistence;
#[cfg(feature = "terminal")]
mod headless_reports;
mod policy_evaluation;
mod search;
mod sqlite_persistence;
mod subprocess;
#[cfg(feature = "terminal")]
mod tui_primary_render;
#[cfg(feature = "terminal")]
mod tui_reload;
#[cfg(feature = "terminal")]
mod tui_tabs;
mod workspace_scanning;
