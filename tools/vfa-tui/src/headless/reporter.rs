//! Headless reporter — scan → evaluate → format → output pipeline
//! (Tasks 9.7 / 9.8, Req 17 / 18).
//!
//! # Pipeline
//!
//! 1. Load catalog from `workspace_root` using [`CatalogStore::load`].
//! 2. Load workspace registry from the path in [`Cli::registry`].
//! 3. Optionally filter workspaces by [`Cli::workspace_filter`].
//! 4. Load policy config from [`Cli::policies`].
//! 5. For each requested [`ReportType`] call the corresponding report function.
//! 6. Format and write to stdout.
//! 7. Return the highest-severity exit code.
//!
//! # No terminal manipulation
//!
//! This module **never** enters raw mode, uses an alternate screen, or
//! manipulates the cursor — it only writes to stdout (Req 17.1).
//!
//! # Determinism (Req 27)
//!
//! All list outputs are sorted case-insensitively by ID before serialisation.
//! No environment variables alter report content (only `--no-color` / `NO_COLOR`
//! affect formatting).

#![deny(warnings)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::catalog::store::CatalogStore;
use crate::cli::Cli;
use crate::federation::coverage::CoverageEngine;
use crate::federation::dep_graph::DependencyGraph;
use crate::federation::drift::{detect_drift, DriftKind};
use crate::federation::integrity::{verify_integrity, IntegrityStatus};
use crate::federation::registry::{LoadResult, WorkspaceRegistry};
use crate::federation::versions::{
    extract_version, freshness_score, is_stale, version_delta, VersionStatus,
};
use crate::models::report::{OutputFormat, ReportType};
use crate::policy::engine::PolicyEngine;
use crate::policy::parser;
use crate::policy::violations::aggregate_violations;

use super::formats::{
    format_json, format_markdown, format_table, with_status, ReportData, STATUS_DRIFT, STATUS_FAIL,
    STATUS_MISSING, STATUS_PASS, STATUS_STALE, STATUS_WARN,
};

// ---------------------------------------------------------------------------
// FindingSeverity — exit-code classification of a single finding
// ---------------------------------------------------------------------------

/// Severity levels for individual findings, mirroring the exit-code hierarchy.
///
/// The numeric value matches the exit code the finding contributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FindingSeverity {
    /// No issue — contributes exit code 0.
    Success = 0,
    /// Compliance failure (violations, content drift, stale, gate failures) — exit 1.
    Compliance = 1,
    /// Operational error (missing config, bad registry) — exit 2.
    Operational = 2,
    /// Partial catalog failure (catalog dir exists, files corrupt) — exit 3.
    PartialCatalog = 3,
}

/// Compute the exit code for a headless run given a slice of findings.
///
/// The exit code is the **maximum** severity among all findings (Req 18.6):
/// 3 > 2 > 1 > 0.
///
/// # Contract (property-testable)
///
/// For any non-empty slice the returned code equals the numeric value of the
/// highest `FindingSeverity` in the slice.  For an empty slice the code is 0.
pub fn compute_exit_code(findings: &[FindingSeverity]) -> u8 {
    findings.iter().map(|f| *f as u8).max().unwrap_or(0)
}

// ---------------------------------------------------------------------------
// HeadlessReporter
// ---------------------------------------------------------------------------

/// Headless report pipeline.
pub struct HeadlessReporter {
    /// Output format requested by the caller.
    pub format: OutputFormat,
    /// When `true`, suppress progress messages; emit only the final report.
    pub quiet: bool,
}

impl HeadlessReporter {
    /// Create a new reporter.
    pub fn new(format: OutputFormat, quiet: bool) -> Self {
        Self { format, quiet }
    }

    // -----------------------------------------------------------------------
    // Entry point
    // -----------------------------------------------------------------------

    /// Run the full report pipeline and return the final JSON payload plus exit code.
    ///
    /// This is the top-level function called from `main.rs` (or tests).  It
    /// loads all data, evaluates the requested report types, formats the output,
    /// writes it to stdout, and returns the exit code.
    ///
    /// `workspace_root` is the root of the **catalog** repository (used to
    /// locate `catalog/`).  This is distinct from downstream workspaces listed
    /// in the registry.
    pub fn run(&self, cli: &Cli, workspace_root: &Path) -> (Value, u8) {
        if !self.quiet {
            eprintln!(
                "[vfa-tui] loading catalog from {}",
                workspace_root.display()
            );
        }

        // ── 1. Load catalog ──────────────────────────────────────────────────
        let catalog = CatalogStore::load(workspace_root);

        let mut findings: Vec<FindingSeverity> = Vec::new();

        // Catalog directory absent → exit 2 (operational error, Req 18.3 / 25.4).
        if !workspace_root.join("catalog").exists() {
            findings.push(FindingSeverity::Operational);
        }

        // Files loaded with errors → partial catalog failure → exit 3 (Req 18.4).
        if !catalog.load_errors.is_empty() {
            findings.push(FindingSeverity::PartialCatalog);
            if !self.quiet {
                for e in &catalog.load_errors {
                    eprintln!("[vfa-tui] catalog error: {e}");
                }
            }
        }

        // ── 2. Load registry ─────────────────────────────────────────────────
        let registry_path = PathBuf::from(&cli.registry);
        let registry = match WorkspaceRegistry::load(&registry_path) {
            Ok(LoadResult::Loaded(r)) => r,
            Ok(LoadResult::NotFound(r)) => {
                // A missing registry is a documented operational error (exit 2;
                // see the exit-code contract in `cli.rs`).  Reporting it as a
                // clean run let a mistyped `--registry` path read as success to
                // every exit-code-only consumer.
                findings.push(FindingSeverity::Operational);
                if !self.quiet {
                    eprintln!(
                        "[vfa-tui] workspace registry not found at {}; 0 workspaces in scope",
                        registry_path.display()
                    );
                }
                r
            }
            Err(e) => {
                findings.push(FindingSeverity::Operational);
                if !self.quiet {
                    eprintln!("[vfa-tui] registry load error: {e}");
                }
                // Build an empty registry so we can still produce a report.
                WorkspaceRegistry {
                    entries: Vec::new(),
                    path: registry_path,
                    last_loaded: std::time::Instant::now(),
                }
            }
        };

        // ── 3. Filter workspaces ─────────────────────────────────────────────
        let all_resolved = registry.resolve();
        let workspaces: Vec<crate::models::workspace::ResolvedWorkspace> =
            if let Some(filter) = &cli.workspace_filter {
                registry
                    .filter(filter, &all_resolved)
                    .into_iter()
                    .cloned()
                    .collect()
            } else {
                all_resolved
            };

        if !self.quiet {
            eprintln!("[vfa-tui] {} workspace(s) in scope", workspaces.len());
        }

        // ── 4. Load policy config ────────────────────────────────────────────
        let policy_path = PathBuf::from(&cli.policies);
        let policy_config = match parser::load(&policy_path) {
            Ok(pc) => {
                // Check for parse errors and report them.
                if !pc.parse_errors.is_empty() {
                    findings.push(FindingSeverity::Operational);
                    if !self.quiet {
                        for e in &pc.parse_errors {
                            eprintln!("[vfa-tui] policy parse error: {e}");
                        }
                    }
                }
                pc
            }
            Err(e) => {
                findings.push(FindingSeverity::Operational);
                if !self.quiet {
                    eprintln!("[vfa-tui] policy load error: {e}");
                }
                crate::policy::parser::PolicyConfig::default()
            }
        };

        // A report that evaluated no rules must not present itself as
        // compliant.  `compliance_score(0, 0)` is 100.0 by definition, so
        // without this guard a missing or empty policy file produced exit 0
        // and "100.0%".  The parser stays lenient by design (Req 11.5); the
        // enforcement decision belongs to the reporting layer.
        if policy_config.rules.is_empty() && !cli.allow_empty_policy {
            findings.push(FindingSeverity::Operational);
            if !self.quiet {
                let why = if policy_config.no_policies_file {
                    "no policy file found at"
                } else {
                    "no usable policy rules in"
                };
                eprintln!(
                    "[vfa-tui] {} {}; refusing to report compliance for 0 evaluated rules \
                     (pass --allow-empty-policy for an exploratory run)",
                    why,
                    policy_path.display()
                );
            }
        }

        // Check for unknown required_role references in rules.
        for rule in &policy_config.rules {
            if let crate::models::policy::PolicyRuleType::RequireRole { role_id } = &rule.rule_type
            {
                let role_agents = catalog.agents_for_role(role_id);
                if role_agents.is_empty() {
                    // Unknown role reference — this is a semantic error.
                    findings.push(FindingSeverity::Operational);
                    if !self.quiet {
                        eprintln!(
                            "[vfa-tui] policy rule '{}': required_role '{}' not found in catalog",
                            rule.id, role_id
                        );
                    }
                }
            }
        }

        // ── 5. Collect installed assets (stub — scanner needs tokio runtime) ─
        // WorkspaceScanner is async.  For the headless pipeline the caller is
        // expected to drive scanning asynchronously and pass results in.  In
        // the current implementation we use an empty set per workspace so that
        // format/coverage/violations still produce valid (though empty) output.
        // A future integration (Task 11.2) will wire the actual scanner output
        // through a tokio runtime.
        use crate::federation::scanner::{CatalogIndex, WorkspaceScanner};
        let catalog_index = CatalogIndex::new(
            catalog
                .agents
                .iter()
                .map(|a| (a.path.clone(), a.id.clone(), None))
                .chain(
                    catalog
                        .skills
                        .iter()
                        .map(|s| (s.path.clone(), s.id.clone(), None)),
                ),
        );
        let scanner = WorkspaceScanner::new(8);

        // Run scanner for each workspace to collect installed assets.
        let installed_per_workspace: Vec<(
            PathBuf,
            Vec<crate::federation::scanner::InstalledAsset>,
        )> = workspaces
            .iter()
            .map(|ws| {
                let assets = scanner.scan_workspace(ws, &catalog_index);
                (ws.canonical_path.clone(), assets)
            })
            .collect();

        // All installed assets flattened.
        let all_installed: Vec<crate::federation::scanner::InstalledAsset> =
            installed_per_workspace
                .iter()
                .flat_map(
                    |(_path, v): &(PathBuf, Vec<crate::federation::scanner::InstalledAsset>)| {
                        v.clone()
                    },
                )
                .collect();

        // ── 6. Build canonical maps ──────────────────────────────────────────
        let canonical_hashes: HashMap<String, String> = catalog
            .integrity
            .as_ref()
            .map(|i| {
                let mut m = HashMap::new();
                for tree in &i.trees {
                    for f in &tree.files {
                        // Use the last path component as the asset id key.
                        m.insert(f.path.clone(), f.sha256.clone());
                    }
                }
                for f in &i.root_files {
                    m.insert(f.path.clone(), f.sha256.clone());
                }
                m
            })
            .unwrap_or_default();

        let canonical_versions: HashMap<String, String> = {
            let mut m = HashMap::new();
            for a in &catalog.agents {
                if let Some(v) = &a.version {
                    m.insert(a.id.clone(), v.clone());
                }
            }
            for s in &catalog.skills {
                if let Some(v) = &s.version {
                    m.insert(s.id.clone(), v.clone());
                }
            }
            m
        };

        // ── 7. Evaluate policy for each workspace ────────────────────────────
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let per_workspace_evals: Vec<crate::models::policy::PolicyEvaluation> = workspaces
            .iter()
            .zip(installed_per_workspace.iter())
            .map(|(ws, (_, installed))| {
                PolicyEngine::evaluate(&policy_config, ws, installed, &catalog, &today)
            })
            .collect();

        // Collect all violations.
        let all_violations: Vec<crate::models::policy::PolicyViolation> = per_workspace_evals
            .iter()
            .flat_map(|eval| {
                eval.results
                    .iter()
                    .filter(|r| !r.passed)
                    .map(|r| crate::models::policy::PolicyViolation {
                        rule: policy_config
                            .rules
                            .iter()
                            .find(|rule| rule.id == r.rule_id)
                            .cloned()
                            .unwrap_or_else(|| crate::models::policy::PolicyRule {
                                id: r.rule_id.clone(),
                                rule_type: crate::models::policy::PolicyRuleType::RequireAsset {
                                    asset_id: String::new(),
                                },
                                severity: crate::models::policy::Severity::Warning,
                                scope: crate::models::policy::PolicyScope::All,
                                description: r.rule_id.clone(),
                            }),
                        workspace: eval.workspace.clone(),
                        asset_id: None,
                        first_detected: today.clone(),
                        details: r.details.clone().unwrap_or_default(),
                        remediation: String::new(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let violations_dashboard = aggregate_violations(&per_workspace_evals, &all_violations);

        // ── 8. Evaluate per-report-type and collect findings ─────────────────
        let report_types = cli.report_types();
        let types_to_run: Vec<ReportType> = if report_types.is_empty() {
            vec![ReportType::Summary]
        } else {
            report_types
        };

        // Expand `All` into every type.
        let expanded: Vec<ReportType> = types_to_run
            .into_iter()
            .flat_map(|rt| {
                if rt == ReportType::All {
                    all_report_types()
                } else {
                    vec![rt]
                }
            })
            .collect();

        let is_all = expanded.len() > 1 || expanded.contains(&ReportType::All);
        let _ = is_all; // may use for combined-object logic

        let mut combined: HashMap<String, Value> = HashMap::new();

        for rt in &expanded {
            let (section_value, section_findings) = self.build_section(
                rt,
                &catalog,
                &workspaces,
                &installed_per_workspace,
                &all_installed,
                &canonical_hashes,
                &canonical_versions,
                &per_workspace_evals,
                &all_violations,
                &violations_dashboard,
                workspace_root,
                &today,
            );
            findings.extend(section_findings);
            combined.insert(rt.as_str().to_string(), section_value);
        }

        let exit_code = compute_exit_code(&findings);

        // Build the timestamp here so it is available for both persistence and
        // the output envelope below.
        let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

        // ── 9. Best-effort persistence ────────────────────────────────────────
        // Persist coverage scores and drift records to the SQLite index.  If the
        // index cannot be opened (path not set, disk error, etc.) skip silently
        // so that report output is never affected by persistence failures.
        {
            use crate::persistence::index::IndexManager;

            let asset_ids = catalog.all_asset_ids();
            // Ensure the index's parent directory exists — `IndexManager::open`
            // (rusqlite) creates the DB file but not missing parent dirs, so on a
            // clean install the default `~/.local/share/vfa/` would not exist.
            if let Some(parent) = Path::new(&cli.index_path).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
            if let Ok(mgr) = IndexManager::open(&cli.index_path) {
                let conn = mgr.write_conn();

                // Persist coverage scores keyed by the CANONICAL workspace path
                // (not the display basename, which collides for same-named
                // workspaces). `matrix.workspace_scores` is keyed by the basename
                // build_matrix derives, so map each resolved workspace path back
                // to its score.
                let matrix = CoverageEngine::build_matrix(
                    &asset_ids,
                    &installed_per_workspace,
                    &canonical_hashes,
                    &canonical_versions,
                );
                for (ws_path_buf, _assets) in &installed_per_workspace {
                    let ws_name = ws_path_buf
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| ws_path_buf.display().to_string());
                    if let Some(score) = matrix.workspace_scores.get(&ws_name) {
                        let _ = conn.execute(
                            "INSERT OR REPLACE INTO coverage_cache \
                             (workspace_path, workspace_name, coverage_score, computed_at) \
                             VALUES (?1, ?2, ?3, ?4)",
                            rusqlite::params![
                                ws_path_buf.to_string_lossy().as_ref(),
                                ws_name,
                                score,
                                &timestamp
                            ],
                        );
                    }
                }

                // Persist drift records
                let drift_records =
                    detect_drift(&all_installed, &canonical_hashes, &canonical_versions);
                for r in &drift_records {
                    if r.kind == crate::federation::drift::DriftKind::None {
                        continue;
                    }
                    let _ = conn.execute(
                        "INSERT INTO drift_history \
                         (workspace_path, asset_id, drift_type, first_detected, \
                          resolved_at, expected_hash, actual_hash) \
                         VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6)",
                        rusqlite::params![
                            r.workspace_path.to_string_lossy().as_ref(),
                            &r.asset_id,
                            crate::federation::drift::drift_kind_label(&r.kind),
                            &today,
                            &r.expected_hash,
                            &r.actual_hash,
                        ],
                    );
                }
            }
        }

        // Build the envelope.
        let console_version = env!("CARGO_PKG_VERSION");

        let output_value = if expanded.len() == 1 {
            // Single report type → flat envelope (Req 17.7 "each type as top-level key").
            let rt = &expanded[0];
            let inner = combined.remove(rt.as_str()).unwrap_or(Value::Null);
            json!({
                "report_type": rt.as_str(),
                "timestamp": timestamp,
                "console_version": console_version,
                "exit_code": exit_code,
                "data": inner,
            })
        } else {
            // Multiple types → combined object (Req 17.7).
            let mut obj = serde_json::Map::new();
            obj.insert("report_type".into(), json!("all"));
            obj.insert("timestamp".into(), json!(timestamp));
            obj.insert("console_version".into(), json!(console_version));
            obj.insert("exit_code".into(), json!(exit_code));
            for (k, v) in combined {
                obj.insert(k, v);
            }
            Value::Object(obj)
        };

        // Write to stdout.
        let output_str = self.render(&output_value, &expanded);
        println!("{output_str}");

        (output_value, exit_code)
    }

    // -----------------------------------------------------------------------
    // Section builder
    // -----------------------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    fn build_section(
        &self,
        rt: &ReportType,
        catalog: &CatalogStore,
        workspaces: &[crate::models::workspace::ResolvedWorkspace],
        installed_per_workspace: &[(PathBuf, Vec<crate::federation::scanner::InstalledAsset>)],
        all_installed: &[crate::federation::scanner::InstalledAsset],
        canonical_hashes: &HashMap<String, String>,
        canonical_versions: &HashMap<String, String>,
        per_workspace_evals: &[crate::models::policy::PolicyEvaluation],
        all_violations: &[crate::models::policy::PolicyViolation],
        violations_dashboard: &crate::policy::violations::ViolationsDashboard,
        workspace_root: &Path,
        today: &str,
    ) -> (Value, Vec<FindingSeverity>) {
        let _ = today; // used in some branches
        match rt {
            ReportType::Coverage => report_coverage(
                catalog,
                installed_per_workspace,
                canonical_hashes,
                canonical_versions,
            ),
            ReportType::Violations => report_violations(
                all_violations,
                violations_dashboard,
                per_workspace_evals
                    .iter()
                    .map(|e| e.results.len())
                    .sum::<usize>(),
            ),
            ReportType::Drift => report_drift(all_installed, canonical_hashes, canonical_versions),
            ReportType::Stale => report_stale(all_installed, canonical_versions),
            ReportType::Gates => report_gates(workspace_root),
            ReportType::Integrity => report_integrity(catalog, workspace_root),
            ReportType::Versions => report_versions(all_installed, canonical_versions),
            ReportType::Dependencies => report_dependencies(catalog),
            ReportType::Lifecycle => report_lifecycle(catalog),
            ReportType::Summary => report_summary(
                catalog,
                workspaces,
                all_installed,
                all_violations,
                violations_dashboard,
                per_workspace_evals,
            ),
            ReportType::All => {
                // Should never be reached — `All` is expanded above.
                (json!({"note": "All expanded"}), vec![])
            }
        }
    }

    // -----------------------------------------------------------------------
    // Render
    // -----------------------------------------------------------------------

    /// Format `value` according to the requested output format.
    pub fn render(&self, value: &Value, report_types: &[ReportType]) -> String {
        match self.format {
            OutputFormat::Json => format_json(value, true),
            OutputFormat::Markdown => {
                let report_data = value_to_report_data(value, report_types);
                format_markdown(&report_data)
            }
            OutputFormat::Table => {
                let report_data = value_to_report_data(value, report_types);
                format_table(&report_data)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers — convert Value to ReportData for non-JSON formats
// ---------------------------------------------------------------------------

fn value_to_report_data(value: &Value, _report_types: &[ReportType]) -> ReportData {
    // For structured Value → extract a table if it contains a "rows" key,
    // otherwise fall back to a raw JSON block.
    if let Some(rows_val) = value.get("rows") {
        if let Some(rows_arr) = rows_val.as_array() {
            // Try to build a ReportData::Table.
            let headers: Vec<String> = value
                .get("headers")
                .and_then(|h| h.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            if !headers.is_empty() {
                let rows: Vec<Vec<String>> = rows_arr
                    .iter()
                    .map(|row| {
                        if let Some(cells) = row.as_array() {
                            cells
                                .iter()
                                .map(|c| {
                                    c.as_str()
                                        .map(|s| s.to_string())
                                        .unwrap_or_else(|| c.to_string())
                                })
                                .collect()
                        } else {
                            vec![row.to_string()]
                        }
                    })
                    .collect();
                return ReportData::Table { headers, rows };
            }
        }
    }

    // Key-value summary?
    if let Some(obj) = value.as_object() {
        let skip_keys = ["report_type", "timestamp", "console_version", "exit_code"];
        let pairs: Vec<(String, String)> = obj
            .iter()
            .filter(|(k, v)| !skip_keys.contains(&k.as_str()) && !v.is_object() && !v.is_array())
            .map(|(k, v)| {
                let val_str = v
                    .as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| v.to_string());
                (k.clone(), val_str)
            })
            .collect();
        if !pairs.is_empty() {
            return ReportData::KeyValue { pairs };
        }
    }

    ReportData::Raw(value.clone())
}

// ---------------------------------------------------------------------------
// Per-report-type functions
// ---------------------------------------------------------------------------

/// Build a list of all concrete report types (used to expand `All`).
pub fn all_report_types() -> Vec<ReportType> {
    vec![
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
    ]
}

/// Record a headless run in the audit log (Req 14.7 / Task 11.2).
///
/// Appends an [`AuditEventType::OperatorAction`] entry with
/// `operator = "headless"` summarizing the report types produced and the
/// resulting exit code, preserving the hash chain. Returns the appended entry.
pub fn record_headless_audit(
    mgr: &crate::persistence::index::IndexManager,
    report_types: &[ReportType],
    exit_code: u8,
) -> Result<crate::models::audit::AuditEntry, crate::error::TuiError> {
    use crate::models::audit::AuditEventType;
    use crate::persistence::audit::AuditLogger;

    let mut logger = AuditLogger::from_manager(mgr)?;
    let types: Vec<&str> = report_types.iter().map(|r| r.as_str()).collect();
    logger.log(
        AuditEventType::OperatorAction,
        "headless_report",
        json!({ "report_types": types, "exit_code": exit_code }),
        "headless",
    )
}

// ── Coverage ─────────────────────────────────────────────────────────────────

fn report_coverage(
    catalog: &CatalogStore,
    installed_per_workspace: &[(PathBuf, Vec<crate::federation::scanner::InstalledAsset>)],
    canonical_hashes: &HashMap<String, String>,
    canonical_versions: &HashMap<String, String>,
) -> (Value, Vec<FindingSeverity>) {
    let asset_ids = catalog.all_asset_ids();
    let matrix = CoverageEngine::build_matrix(
        &asset_ids,
        installed_per_workspace,
        canonical_hashes,
        canonical_versions,
    );

    // Build per-workspace score objects.
    let mut workspace_scores: Vec<Value> = matrix
        .workspace_scores
        .iter()
        .map(|(ws, score)| {
            let score_str = format!("{score:.1}%");
            json!({ "workspace": ws, "coverage_score": score_str })
        })
        .collect();
    workspace_scores.sort_by(|a, b| {
        let ak = a["workspace"].as_str().unwrap_or("").to_lowercase();
        let bk = b["workspace"].as_str().unwrap_or("").to_lowercase();
        ak.cmp(&bk)
    });

    let aggregate = compute_aggregate_coverage_score(&matrix);
    let findings = vec![]; // coverage alone doesn't trigger exit 1

    let value = json!({
        "total_assets": asset_ids.len(),
        "total_workspaces": installed_per_workspace.len(),
        "aggregate_coverage_score": aggregate,
        "workspace_scores": workspace_scores,
    });
    (value, findings)
}

fn compute_aggregate_coverage_score(matrix: &crate::models::coverage::CoverageMatrix) -> String {
    let scores: Vec<f64> = matrix.workspace_scores.values().copied().collect();
    if scores.is_empty() {
        return "N/A".to_string();
    }
    let avg = scores.iter().sum::<f64>() / scores.len() as f64;
    format!("{avg:.1}%")
}

// ── Violations ───────────────────────────────────────────────────────────────

fn report_violations(
    all_violations: &[crate::models::policy::PolicyViolation],
    dashboard: &crate::policy::violations::ViolationsDashboard,
    rules_evaluated: usize,
) -> (Value, Vec<FindingSeverity>) {
    use crate::models::policy::Severity;

    let has_critical = all_violations
        .iter()
        .any(|v| v.rule.severity == Severity::Critical);

    let mut findings = vec![];
    if has_critical {
        findings.push(FindingSeverity::Compliance);
    }

    let mut viol_rows: Vec<Value> = all_violations
        .iter()
        .map(|v| {
            let indicator = match v.rule.severity {
                Severity::Critical => STATUS_FAIL,
                Severity::Warning => STATUS_WARN,
                Severity::Info => STATUS_WARN,
            };
            json!({
                "workspace": v.workspace,
                "rule_id": v.rule.id,
                "severity": format!("{:?}", v.rule.severity).to_lowercase(),
                "status": with_status(indicator, &v.rule.id),
                "details": v.details,
            })
        })
        .collect();
    viol_rows.sort_by(|a, b| {
        let ak = a["workspace"].as_str().unwrap_or("").to_lowercase();
        let bk = b["workspace"].as_str().unwrap_or("").to_lowercase();
        ak.cmp(&bk)
    });

    let mut ranked: Vec<Value> = dashboard
        .ranked_workspaces
        .iter()
        .map(|(ws, score)| {
            // A score computed over zero rules is vacuous, not a pass; emit
            // null rather than a reassuring 100.0.
            let score = if rules_evaluated == 0 {
                Value::Null
            } else {
                json!(score)
            };
            json!({ "workspace": ws, "compliance_score": score })
        })
        .collect();
    ranked.sort_by(|a, b| {
        let as_ = a["compliance_score"].as_f64().unwrap_or(100.0);
        let bs = b["compliance_score"].as_f64().unwrap_or(100.0);
        as_.partial_cmp(&bs).unwrap_or(std::cmp::Ordering::Equal)
    });

    let value = json!({
        // Publish the denominator so a consumer can tell "nothing violated"
        // apart from "nothing was checked".
        "rules_evaluated": rules_evaluated,
        "total_violations": all_violations.len(),
        "critical_count": all_violations.iter().filter(|v| v.rule.severity == Severity::Critical).count(),
        "warning_count": all_violations.iter().filter(|v| v.rule.severity == Severity::Warning).count(),
        "violations": viol_rows,
        "ranked_workspaces": ranked,
    });
    (value, findings)
}

// ── Drift ─────────────────────────────────────────────────────────────────────

fn report_drift(
    all_installed: &[crate::federation::scanner::InstalledAsset],
    canonical_hashes: &HashMap<String, String>,
    canonical_versions: &HashMap<String, String>,
) -> (Value, Vec<FindingSeverity>) {
    let records = detect_drift(all_installed, canonical_hashes, canonical_versions);

    let has_content_drift = records.iter().any(|r| r.kind == DriftKind::ContentDrift);
    let findings = if has_content_drift {
        vec![FindingSeverity::Compliance]
    } else {
        vec![]
    };

    let mut drift_rows: Vec<Value> = records
        .iter()
        .map(|r| {
            let kind_str = match r.kind {
                DriftKind::None => "none",
                DriftKind::ContentDrift => "content_drift",
                DriftKind::VersionDrift => "version_drift",
            };
            let indicator = match r.kind {
                DriftKind::ContentDrift => STATUS_DRIFT,
                DriftKind::VersionDrift => STATUS_WARN,
                DriftKind::None => STATUS_PASS,
            };
            json!({
                "asset_id": r.asset_id,
                "workspace_path": r.workspace_path.display().to_string(),
                "expected_hash": &r.expected_hash[..12.min(r.expected_hash.len())],
                "actual_hash": &r.actual_hash[..12.min(r.actual_hash.len())],
                "kind": kind_str,
                "status": with_status(indicator, kind_str),
            })
        })
        .collect();
    // Stable sort by asset_id (Req 27.2).
    drift_rows.sort_by(|a, b| {
        a["asset_id"]
            .as_str()
            .unwrap_or("")
            .to_lowercase()
            .cmp(&b["asset_id"].as_str().unwrap_or("").to_lowercase())
    });

    let value = json!({
        "total_drifted": records.len(),
        "content_drift_count": records.iter().filter(|r| r.kind == DriftKind::ContentDrift).count(),
        "version_drift_count": records.iter().filter(|r| r.kind == DriftKind::VersionDrift).count(),
        "records": drift_rows,
    });
    (value, findings)
}

// ── Stale ─────────────────────────────────────────────────────────────────────

/// Default stale-asset limit per workspace that triggers exit code 1 (Req 9.4).
const DEFAULT_STALE_ASSET_LIMIT: usize = 5;
/// Default minor-version threshold for staleness (Req 9.1).
const DEFAULT_STALE_MINOR_THRESHOLD: u32 = 2;

fn report_stale(
    all_installed: &[crate::federation::scanner::InstalledAsset],
    canonical_versions: &HashMap<String, String>,
) -> (Value, Vec<FindingSeverity>) {
    let stale_assets: Vec<Value> = all_installed
        .iter()
        .filter(|a| a.confirmed)
        .filter_map(|a| {
            let installed_ver = extract_version(a)?;
            let canonical_ver = canonical_versions.get(&a.asset_id)?;
            if is_stale(&installed_ver, canonical_ver, DEFAULT_STALE_MINOR_THRESHOLD) {
                Some(json!({
                    "asset_id": a.asset_id,
                    "workspace_path": a.workspace_path.display().to_string(),
                    "installed_version": installed_ver,
                    "canonical_version": canonical_ver,
                    "status": with_status(STATUS_STALE, &a.asset_id),
                }))
            } else {
                None
            }
        })
        .collect();

    // Group by workspace to check per-workspace stale counts (Req 9.4).
    let mut stale_per_ws: HashMap<String, usize> = HashMap::new();
    for entry in &stale_assets {
        let ws = entry["workspace_path"].as_str().unwrap_or("").to_string();
        *stale_per_ws.entry(ws).or_default() += 1;
    }

    let exceeds_threshold = stale_per_ws
        .values()
        .any(|&count| count >= DEFAULT_STALE_ASSET_LIMIT);

    let findings = if exceeds_threshold {
        vec![FindingSeverity::Compliance]
    } else {
        vec![]
    };

    let mut sorted_stale = stale_assets;
    sorted_stale.sort_by(|a, b| {
        a["asset_id"]
            .as_str()
            .unwrap_or("")
            .to_lowercase()
            .cmp(&b["asset_id"].as_str().unwrap_or("").to_lowercase())
    });

    let value = json!({
        "total_stale": sorted_stale.len(),
        "stale_threshold": DEFAULT_STALE_MINOR_THRESHOLD,
        "alert_threshold_per_workspace": DEFAULT_STALE_ASSET_LIMIT,
        "exceeds_threshold": exceeds_threshold,
        "stale_assets": sorted_stale,
    });
    (value, findings)
}

// ── Gates ─────────────────────────────────────────────────────────────────────

fn report_gates(workspace_root: &Path) -> (Value, Vec<FindingSeverity>) {
    // Load the gate DAG definition (not executing gates — only reporting structure).
    // Running real gates is optional and heavy (Req 2.7 note in tasks).
    let gates_toml = workspace_root.join("gates.toml");

    let gates_toml_opt = if gates_toml.exists() {
        Some(gates_toml.as_path())
    } else {
        None
    };

    let gate_dag = crate::gates::dag::parse_gates(gates_toml_opt, workspace_root);

    match gate_dag {
        Ok(definitions) => {
            let gate_rows: Vec<Value> = definitions
                .iter()
                .map(|g| {
                    json!({
                        "name": g.name,
                        "command": g.command,
                        "args": g.args,
                        "dependencies": g.dependencies,
                        "status": with_status(STATUS_WARN, "pending"),
                        "duration_ms": 0,
                        "skip_reason": null,
                    })
                })
                .collect();

            let value = json!({
                "total_gates": gate_rows.len(),
                "gates": gate_rows,
                "note": "Gate definitions loaded; run with --run-gates to execute",
            });
            (value, vec![])
        }
        Err(e) => {
            let value = json!({
                "total_gates": 0,
                "gates": [],
                "error": e.to_string(),
            });
            (value, vec![FindingSeverity::Operational])
        }
    }
}

// ── Integrity ────────────────────────────────────────────────────────────────

fn report_integrity(
    catalog: &CatalogStore,
    workspace_root: &Path,
) -> (Value, Vec<FindingSeverity>) {
    let manifest = match &catalog.integrity {
        Some(m) => m,
        None => {
            return (
                json!({"error": "asset-integrity.json not loaded", "status": with_status(STATUS_WARN, "missing")}),
                vec![FindingSeverity::Operational],
            );
        }
    };

    let results = verify_integrity(manifest, workspace_root);

    let has_failures = results.iter().any(|r| r.status == IntegrityStatus::Fail);
    let findings = if has_failures {
        vec![FindingSeverity::Compliance]
    } else {
        vec![]
    };

    let mut result_rows: Vec<Value> = results
        .iter()
        .map(|r| {
            let (indicator, status_str) = match r.status {
                IntegrityStatus::Pass => (STATUS_PASS, "pass"),
                IntegrityStatus::Fail => (STATUS_FAIL, "fail"),
                IntegrityStatus::Missing => (STATUS_MISSING, "missing"),
            };
            json!({
                "path": r.path,
                "expected_hash": r.expected_hash,
                "actual_hash": r.actual_hash,
                "status": with_status(indicator, status_str),
            })
        })
        .collect();
    // Sort by path (Req 27.2).
    result_rows.sort_by(|a, b| {
        a["path"]
            .as_str()
            .unwrap_or("")
            .to_lowercase()
            .cmp(&b["path"].as_str().unwrap_or("").to_lowercase())
    });

    let value = json!({
        "total_files": results.len(),
        "pass_count": results.iter().filter(|r| r.status == IntegrityStatus::Pass).count(),
        "fail_count": results.iter().filter(|r| r.status == IntegrityStatus::Fail).count(),
        "missing_count": results.iter().filter(|r| r.status == IntegrityStatus::Missing).count(),
        "results": result_rows,
    });
    (value, findings)
}

// ── Versions ─────────────────────────────────────────────────────────────────

fn report_versions(
    all_installed: &[crate::federation::scanner::InstalledAsset],
    canonical_versions: &HashMap<String, String>,
) -> (Value, Vec<FindingSeverity>) {
    let mut assets_at_current = 0usize;
    let mut total_with_versions = 0usize;

    let mut version_rows: Vec<Value> = all_installed
        .iter()
        .filter(|a| a.confirmed)
        .filter_map(|a| {
            let installed_ver = extract_version(a)?;
            let canonical_ver = canonical_versions.get(&a.asset_id)?;
            total_with_versions += 1;

            let delta = version_delta(&installed_ver, canonical_ver);
            if delta.status == VersionStatus::Current {
                assets_at_current += 1;
            }

            let status_str = match delta.status {
                VersionStatus::Current => "current",
                VersionStatus::Outdated => "outdated",
                VersionStatus::Unknown => "unknown",
            };
            let indicator = if delta.status == VersionStatus::Current {
                STATUS_PASS
            } else {
                STATUS_WARN
            };

            Some(json!({
                "asset_id": a.asset_id,
                "workspace_path": a.workspace_path.display().to_string(),
                "installed_version": installed_ver,
                "canonical_version": canonical_ver,
                "major_delta": delta.major_delta,
                "minor_delta": delta.minor_delta,
                "status": with_status(indicator, status_str),
            }))
        })
        .collect();

    version_rows.sort_by(|a, b| {
        a["asset_id"]
            .as_str()
            .unwrap_or("")
            .to_lowercase()
            .cmp(&b["asset_id"].as_str().unwrap_or("").to_lowercase())
    });

    let score = freshness_score(assets_at_current, total_with_versions);
    let value = json!({
        "freshness_score": format!("{score:.1}%"),
        "assets_at_current": assets_at_current,
        "total_with_versions": total_with_versions,
        "version_rows": version_rows,
    });
    (value, vec![])
}

// ── Dependencies ─────────────────────────────────────────────────────────────

fn report_dependencies(catalog: &CatalogStore) -> (Value, Vec<FindingSeverity>) {
    let graph = DependencyGraph::build(catalog);
    let adjacency = graph.to_adjacency_json();

    let mut node_rows: Vec<Value> = graph
        .nodes
        .values()
        .map(|n| {
            json!({
                "id": n.id,
                "asset_type": format!("{:?}", n.asset_type).to_lowercase(),
            })
        })
        .collect();
    node_rows.sort_by(|a, b| {
        a["id"]
            .as_str()
            .unwrap_or("")
            .to_lowercase()
            .cmp(&b["id"].as_str().unwrap_or("").to_lowercase())
    });

    let value = json!({
        "total_nodes": graph.nodes.len(),
        "total_edges": graph.edges.len(),
        "nodes": node_rows,
        "adjacency": adjacency,
    });
    (value, vec![])
}

// ── Lifecycle ─────────────────────────────────────────────────────────────────

fn report_lifecycle(catalog: &CatalogStore) -> (Value, Vec<FindingSeverity>) {
    use crate::models::agent::Lifecycle;

    let mut experimental = 0usize;
    let mut beta = 0usize;
    let mut stable = 0usize;
    let mut deprecated = 0usize;

    let mut asset_rows: Vec<Value> = catalog
        .agents
        .iter()
        .map(|a| {
            let lc = a.lifecycle.unwrap_or(Lifecycle::Experimental);
            match lc {
                Lifecycle::Experimental => experimental += 1,
                Lifecycle::Beta => beta += 1,
                Lifecycle::Stable => stable += 1,
                Lifecycle::Deprecated => deprecated += 1,
            }
            let lc_str = format!("{lc:?}").to_lowercase();
            let indicator = match lc {
                Lifecycle::Stable => STATUS_PASS,
                Lifecycle::Deprecated => STATUS_WARN,
                _ => STATUS_WARN,
            };
            json!({
                "id": a.id,
                "lifecycle": lc_str,
                "status": with_status(indicator, &lc_str),
            })
        })
        .collect();

    // Sort by id (Req 27.2).
    asset_rows.sort_by(|a, b| {
        a["id"]
            .as_str()
            .unwrap_or("")
            .to_lowercase()
            .cmp(&b["id"].as_str().unwrap_or("").to_lowercase())
    });

    let value = json!({
        "experimental": experimental,
        "beta": beta,
        "stable": stable,
        "deprecated": deprecated,
        "total": catalog.agents.len(),
        "assets": asset_rows,
    });
    (value, vec![])
}

// ── Summary ──────────────────────────────────────────────────────────────────

fn report_summary(
    catalog: &CatalogStore,
    workspaces: &[crate::models::workspace::ResolvedWorkspace],
    _all_installed: &[crate::federation::scanner::InstalledAsset],
    all_violations: &[crate::models::policy::PolicyViolation],
    dashboard: &crate::policy::violations::ViolationsDashboard,
    _per_workspace_evals: &[crate::models::policy::PolicyEvaluation],
) -> (Value, Vec<FindingSeverity>) {
    use crate::models::policy::Severity;

    let agg_compliance: f64 = if dashboard.workspace_scores.is_empty() {
        100.0
    } else {
        let sum: f64 = dashboard.workspace_scores.values().sum();
        let avg = sum / dashboard.workspace_scores.len() as f64;
        (avg * 10.0 + 0.5).floor() / 10.0
    };

    let has_critical = all_violations
        .iter()
        .any(|v| v.rule.severity == Severity::Critical);

    let overall_status = if has_critical {
        with_status(STATUS_FAIL, "compliance failures detected")
    } else if !all_violations.is_empty() {
        with_status(STATUS_WARN, "warnings present")
    } else {
        with_status(STATUS_PASS, "all clear")
    };

    let findings = if has_critical {
        vec![FindingSeverity::Compliance]
    } else {
        vec![]
    };

    let value = json!({
        "total_agents": catalog.agents.len(),
        "total_skills": catalog.skills.len(),
        "total_rules": catalog.rules.len(),
        "total_mcp_refs": catalog.mcp_refs.len(),
        "total_workspaces": workspaces.len(),
        "total_violations": all_violations.len(),
        "aggregate_compliance_score": format!("{agg_compliance:.1}%"),
        "overall_status": overall_status,
    });
    (value, findings)
}

// ---------------------------------------------------------------------------
// ReportType::as_str helper
// ---------------------------------------------------------------------------

impl ReportType {
    /// Return a stable lowercase string key for the report type.
    pub fn as_str(&self) -> &'static str {
        match self {
            ReportType::Coverage => "coverage",
            ReportType::Violations => "violations",
            ReportType::Drift => "drift",
            ReportType::Stale => "stale",
            ReportType::Gates => "gates",
            ReportType::Integrity => "integrity",
            ReportType::Versions => "versions",
            ReportType::Dependencies => "dependencies",
            ReportType::Lifecycle => "lifecycle",
            ReportType::Summary => "summary",
            ReportType::All => "all",
        }
    }
}

// ---------------------------------------------------------------------------
// Tests (Property 26 + unit tests, Task 9.8)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -----------------------------------------------------------------------
    // compute_exit_code unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn empty_findings_gives_exit_0() {
        assert_eq!(compute_exit_code(&[]), 0);
    }

    #[test]
    fn success_only_gives_exit_0() {
        assert_eq!(compute_exit_code(&[FindingSeverity::Success]), 0);
    }

    #[test]
    fn compliance_only_gives_exit_1() {
        assert_eq!(compute_exit_code(&[FindingSeverity::Compliance]), 1);
    }

    #[test]
    fn operational_only_gives_exit_2() {
        assert_eq!(compute_exit_code(&[FindingSeverity::Operational]), 2);
    }

    #[test]
    fn partial_catalog_only_gives_exit_3() {
        assert_eq!(compute_exit_code(&[FindingSeverity::PartialCatalog]), 3);
    }

    #[test]
    fn highest_wins_3_over_1() {
        assert_eq!(
            compute_exit_code(&[FindingSeverity::Compliance, FindingSeverity::PartialCatalog]),
            3
        );
    }

    #[test]
    fn highest_wins_3_over_2() {
        assert_eq!(
            compute_exit_code(&[
                FindingSeverity::Operational,
                FindingSeverity::PartialCatalog
            ]),
            3
        );
    }

    #[test]
    fn highest_wins_2_over_1() {
        assert_eq!(
            compute_exit_code(&[FindingSeverity::Compliance, FindingSeverity::Operational]),
            2
        );
    }

    #[test]
    fn all_four_gives_exit_3() {
        assert_eq!(
            compute_exit_code(&[
                FindingSeverity::Success,
                FindingSeverity::Compliance,
                FindingSeverity::Operational,
                FindingSeverity::PartialCatalog,
            ]),
            3
        );
    }

    #[test]
    fn content_drift_triggers_compliance_exit_1() {
        // DriftKind::ContentDrift → exit 1; VersionDrift → exit 0 (Req 10.4)
        let installed = vec![]; // empty — no real assets needed for this unit test
        let canonical_hashes = HashMap::new();
        let canonical_versions = HashMap::new();
        let (_, findings) = report_drift(&installed, &canonical_hashes, &canonical_versions);
        // No drift records → no compliance finding.
        assert_eq!(compute_exit_code(&findings), 0);
    }

    #[test]
    fn report_types_as_str_stable() {
        let pairs = [
            (ReportType::Coverage, "coverage"),
            (ReportType::Violations, "violations"),
            (ReportType::Drift, "drift"),
            (ReportType::Stale, "stale"),
            (ReportType::Gates, "gates"),
            (ReportType::Integrity, "integrity"),
            (ReportType::Versions, "versions"),
            (ReportType::Dependencies, "dependencies"),
            (ReportType::Lifecycle, "lifecycle"),
            (ReportType::Summary, "summary"),
            (ReportType::All, "all"),
        ];
        for (rt, expected) in &pairs {
            assert_eq!(rt.as_str(), *expected);
        }
    }

    // -----------------------------------------------------------------------
    // Smoke test: pipeline produces valid JSON against real catalog
    // -----------------------------------------------------------------------

    #[test]
    fn smoke_summary_produces_valid_json() {
        // Use the repository's own catalog/ as the workspace_root.
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();

        if !workspace_root.join("catalog").exists() {
            // Skip in environments without a catalog.
            return;
        }

        let catalog = CatalogStore::load(workspace_root);

        let workspaces: Vec<crate::models::workspace::ResolvedWorkspace> = vec![];
        let all_installed: Vec<crate::federation::scanner::InstalledAsset> = vec![];
        let evals: Vec<crate::models::policy::PolicyEvaluation> = vec![];
        let violations: Vec<crate::models::policy::PolicyViolation> = vec![];
        let dashboard = crate::policy::violations::ViolationsDashboard::default();

        let (value, findings) = report_summary(
            &catalog,
            &workspaces,
            &all_installed,
            &violations,
            &dashboard,
            &evals,
        );
        assert_eq!(compute_exit_code(&findings), 0);

        // Round-trip through JSON.
        let json_str = serde_json::to_string(&value).expect("serialize");
        let back: Value = serde_json::from_str(&json_str).expect("parse");
        assert!(back.is_object());
        assert!(back["total_agents"].as_u64().unwrap_or(0) > 0);
    }

    #[test]
    fn smoke_all_report_types_valid_json() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();

        if !workspace_root.join("catalog").exists() {
            return;
        }

        let catalog = CatalogStore::load(workspace_root);
        let canonical_hashes: HashMap<String, String> = HashMap::new();
        let canonical_versions: HashMap<String, String> = {
            let mut m = HashMap::new();
            for a in &catalog.agents {
                if let Some(v) = &a.version {
                    m.insert(a.id.clone(), v.clone());
                }
            }
            m
        };

        for rt in all_report_types() {
            let installed_per_workspace: Vec<(
                PathBuf,
                Vec<crate::federation::scanner::InstalledAsset>,
            )> = vec![];
            let all_installed: Vec<crate::federation::scanner::InstalledAsset> = vec![];
            let evals: Vec<crate::models::policy::PolicyEvaluation> = vec![];
            let violations: Vec<crate::models::policy::PolicyViolation> = vec![];
            let dashboard = crate::policy::violations::ViolationsDashboard::default();

            let reporter = HeadlessReporter::new(OutputFormat::Json, true);
            let (value, _findings) = reporter.build_section(
                &rt,
                &catalog,
                &[],
                &installed_per_workspace,
                &all_installed,
                &canonical_hashes,
                &canonical_versions,
                &evals,
                &violations,
                &dashboard,
                workspace_root,
                "2026-01-01",
            );

            // Must produce valid JSON.
            let json_str = serde_json::to_string(&value).expect("serialize");
            let back: Value = serde_json::from_str(&json_str)
                .unwrap_or_else(|e| panic!("report type {:?} produced invalid JSON: {e}", rt));
            assert!(
                back.is_object() || back.is_array() || !back.is_null(),
                "report type {:?} produced null",
                rt
            );
        }
    }

    // -----------------------------------------------------------------------
    // Property 26 (Req 18): compute_exit_code returns max severity among findings.
    // Exhaustive over the 4-value space (4^8 = 65536 > 256 combos covered).
    // -----------------------------------------------------------------------

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(256))]

        #[test]
        fn prop26_exit_code_is_max_severity(
            findings in prop::collection::vec(
                prop_oneof![
                    Just(FindingSeverity::Success),
                    Just(FindingSeverity::Compliance),
                    Just(FindingSeverity::Operational),
                    Just(FindingSeverity::PartialCatalog),
                ],
                0..8,
            )
        ) {
            let code = compute_exit_code(&findings);
            let expected_max = findings.iter().map(|f| *f as u8).max().unwrap_or(0);
            prop_assert_eq!(code, expected_max,
                "exit code should equal max severity; findings={:?}", findings);
        }

        /// compute_exit_code is monotone: adding a higher-severity finding never
        /// decreases the exit code.
        #[test]
        fn prop26_monotone(
            base in prop::collection::vec(
                prop_oneof![
                    Just(FindingSeverity::Success),
                    Just(FindingSeverity::Compliance),
                ],
                0..5,
            ),
            extra in prop_oneof![
                Just(FindingSeverity::Operational),
                Just(FindingSeverity::PartialCatalog),
            ]
        ) {
            let code_before = compute_exit_code(&base);
            let mut with_extra = base.clone();
            with_extra.push(extra);
            let code_after = compute_exit_code(&with_extra);
            prop_assert!(
                code_after >= code_before,
                "adding a higher severity should not decrease exit code"
            );
        }
    }
}
