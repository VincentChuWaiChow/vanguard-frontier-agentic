//! Integration test 13.5 — headless report generation end-to-end.
//! Validates: Requirements 17.x (headless reports), 11.4 (exit codes), 27.x
//!
//! Runs `HeadlessReporter::run` against the catalog fixtures and asserts the
//! JSON envelope shape, valid exit codes, deterministic structure, and that
//! rendered output is non-empty and valid JSON.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use clap::Parser;
use rusqlite::Connection;
use serde_json::{json, Value};
use tempfile::TempDir;

use vfa_tui::cli::Cli;
use vfa_tui::headless::reporter::HeadlessReporter;
use vfa_tui::models::report::OutputFormat;

const CATALOG_FILES: [&str; 6] = [
    "agents.json",
    "skills.json",
    "rules.json",
    "mcp-references.json",
    "install-roles.json",
    "asset-integrity.json",
];

struct Sandbox {
    _temp: TempDir,
    root: PathBuf,
    registry: PathBuf,
    policies: PathBuf,
    index: PathBuf,
    home: PathBuf,
}

impl Sandbox {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("isolated compatibility directory");
        let root = temp.path().join("checkout ü with spaces");
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        std::fs::create_dir_all(root.join("catalog")).unwrap();
        for file in CATALOG_FILES {
            std::fs::copy(
                source.join("catalog").join(file),
                root.join("catalog").join(file),
            )
            .expect("required catalog fixture must exist");
        }
        std::fs::copy(source.join("package.json"), root.join("package.json")).unwrap();
        let home = temp.path().join("isolated profile");
        std::fs::create_dir_all(&home).unwrap();
        Self {
            registry: root.join("absent-registry.toml"),
            policies: root.join("absent-policy.toml"),
            index: root.join("state ü").join("index.db"),
            root,
            home,
            _temp: temp,
        }
    }

    fn args(&self) -> Vec<std::ffi::OsString> {
        vec![
            "--workspace".into(),
            self.root.as_os_str().into(),
            "--registry".into(),
            self.registry.as_os_str().into(),
            "--policies".into(),
            self.policies.as_os_str().into(),
            "--index-path".into(),
            self.index.as_os_str().into(),
            "--no-color".into(),
        ]
    }

    fn cli(&self, extra: &[&str]) -> Cli {
        Cli::parse_from(
            std::iter::once(std::ffi::OsString::from("vfa-tui"))
                .chain(self.args())
                .chain(extra.iter().map(std::ffi::OsString::from)),
        )
    }

    fn launch(&self, extra: &[&str]) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_vfa-tui"));
        command
            .current_dir(&self.home)
            .env_clear()
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("APPDATA", &self.home)
            .env("LOCALAPPDATA", &self.home)
            .env("XDG_CONFIG_HOME", &self.home)
            .env("XDG_DATA_HOME", &self.home)
            .env("XDG_CACHE_HOME", &self.home)
            .env("USER", "compatibility")
            .env("USERNAME", "compatibility")
            .env("NO_COLOR", "1")
            .args(self.args())
            .args(extra);
        // Windows needs its system directory even with an isolated child profile.
        if let Some(system_root) = std::env::var_os("SystemRoot") {
            command.env("SystemRoot", system_root);
        }
        command.output().expect("launch Cargo-provided binary")
    }
}

fn golden() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/compatibility/reports/golden.json"
    ))
    .expect("required, reviewed golden JSON")
}

fn run(extra: &[&str]) -> (Value, u8) {
    let sandbox = Sandbox::new();
    let reporter = HeadlessReporter::new(OutputFormat::Json, true);
    reporter.run(&sandbox.cli(extra), &sandbox.root)
}

// Only these declared observation fields vary. Stable nested report data is
// never removed, reordered or generated from the implementation under test.
fn normalize_report(mut value: Value) -> Result<Value, &'static str> {
    if value["console_version"] != env!("CARGO_PKG_VERSION") {
        return Err("wrong report producer version");
    }
    let timestamp = value["timestamp"].as_str().ok_or("missing timestamp")?;
    chrono::DateTime::parse_from_rfc3339(timestamp).map_err(|_| "invalid timestamp")?;
    value["timestamp"] = json!("<timestamp>");
    value["console_version"] = json!("<producer-version>");
    Ok(value)
}

fn report_matches(value: Value, exit: i32, expected: &Value) -> bool {
    expected["exit_code"].as_i64() == Some(i64::from(exit))
        && normalize_report(value).is_ok_and(|normalized| normalized == *expected)
}

fn assert_report(output: &Output, expected: &Value) -> Value {
    let value: Value = serde_json::from_slice(&output.stdout)
        .expect("stdout must contain exactly one JSON value, with no progress/log text");
    assert!(
        report_matches(value.clone(), output.status.code().unwrap(), expected),
        "unexpected report/exit: {value:#}; status {:?}",
        output.status.code()
    );
    value
}

fn stderr_lines(output: &Output) -> Vec<String> {
    String::from_utf8(output.stderr.clone())
        .expect("UTF-8 stderr")
        .lines()
        .map(String::from)
        .collect()
}

fn assert_initial_log(line: &str) {
    let mut record: Value = serde_json::from_str(line).expect("initial JSON log on stderr");
    chrono::DateTime::parse_from_rfc3339(record["timestamp"].as_str().unwrap()).unwrap();
    let session = uuid::Uuid::parse_str(record["fields"]["session_id"].as_str().unwrap()).unwrap();
    assert_eq!(session.get_version_num(), 4);
    record["timestamp"] = json!("<timestamp>");
    record["fields"]["session_id"] = json!("<session-id>");
    assert_eq!(record, golden()["logging"]);
}

fn assert_quiet_stderr(output: &Output) {
    let lines = stderr_lines(output);
    assert_eq!(
        lines.len(),
        1,
        "legacy quiet mode emits only the startup log"
    );
    assert_initial_log(&lines[0]);
}

fn schema_snapshot(conn: &Connection) -> Value {
    let mut result = json!({
        "schema_version": conn.query_row(
            "SELECT value FROM schema_meta WHERE key = 'schema_version'",
            [], |row| row.get::<_, String>(0)
        ).unwrap()
    });
    for table in ["audit_log", "coverage_cache", "drift_history"] {
        let mut statement = conn
            .prepare(&format!("PRAGMA table_info({table})"))
            .unwrap();
        let columns: Vec<Value> = statement
            .query_map([], |row| {
                Ok(json!([
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, i64>(5)?
                ]))
            })
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        result[table] = json!(columns);
    }
    let mut statement = conn.prepare(
        "SELECT name, sql FROM sqlite_master WHERE type='trigger' AND tbl_name='audit_log' ORDER BY name"
    ).unwrap();
    let mut triggers = serde_json::Map::new();
    for entry in statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .unwrap()
    {
        let (name, sql) = entry.unwrap();
        triggers.insert(name, json!(sql));
    }
    result["audit_triggers"] = json!(triggers);
    result
}

fn envelope_keys_present(v: &Value) {
    let obj = v.as_object().expect("envelope must be a JSON object");
    for k in ["report_type", "timestamp", "console_version", "exit_code"] {
        assert!(obj.contains_key(k), "envelope missing key {k}: {v}");
    }
}

#[test]
fn default_report_is_summary_envelope() {
    let (value, code) = run(&[]);
    envelope_keys_present(&value);
    assert_eq!(value["report_type"], "summary");
    assert!(code <= 3, "exit code {code} out of documented 0..=3 range");
}

#[test]
fn typed_coverage_report_round_trips() {
    let (value, code) = run(&["--report", "coverage"]);
    envelope_keys_present(&value);
    assert_eq!(value["report_type"], "coverage");
    assert!(
        value.get("data").is_some(),
        "single-type report carries a data section"
    );
    assert!(code <= 3);
}

#[test]
fn all_report_combines_multiple_sections() {
    let (value, _code) = run(&["--report", "all"]);
    let obj = value.as_object().unwrap();
    assert_eq!(obj["report_type"], "all");
    // The combined object carries more than just the envelope's 4 fixed keys.
    assert!(
        obj.len() > 4,
        "combined report should include multiple section keys"
    );
}

#[test]
fn run_is_structurally_deterministic() {
    // Timestamps differ between runs, so compare the exit code and the set of
    // top-level keys rather than the whole value.
    let (a, ca) = run(&["--report", "coverage"]);
    let (b, cb) = run(&["--report", "coverage"]);
    assert_eq!(ca, cb, "exit code must be deterministic");

    let mut ka: Vec<&String> = a.as_object().unwrap().keys().collect();
    let mut kb: Vec<&String> = b.as_object().unwrap().keys().collect();
    ka.sort();
    kb.sort();
    assert_eq!(ka, kb, "top-level key set must be deterministic");
    // The data payloads (excluding timestamp) must match exactly.
    assert_eq!(a["data"], b["data"], "coverage data must be deterministic");
}

#[test]
fn output_is_valid_json_and_renders_nonempty() {
    let (value, _) = run(&["--report", "summary"]);
    // Serialize → re-parse round trip.
    let s = serde_json::to_string(&value).expect("serialize");
    let reparsed: Value = serde_json::from_str(&s).expect("valid JSON");
    assert_eq!(reparsed["report_type"], value["report_type"]);

    let reporter = HeadlessReporter::new(OutputFormat::Json, true);
    let sandbox = Sandbox::new();
    let rendered = reporter.render(
        &value,
        &sandbox.cli(&["--report", "summary"]).report_types(),
    );
    assert!(
        !rendered.trim().is_empty(),
        "rendered report must be non-empty"
    );
}

#[test]
fn cli_fixed_summary_coverage_and_combined_envelopes() {
    for kind in ["summary", "coverage"] {
        let sandbox = Sandbox::new();
        let output = sandbox.launch(&["--quiet", "--report", kind]);
        assert_report(&output, &golden()[kind]);
        assert_quiet_stderr(&output);
        assert_eq!(std::fs::read_dir(&sandbox.home).unwrap().count(), 0);
        assert!(!sandbox.registry.exists());
        assert!(!sandbox.policies.exists());
    }
    let sandbox = Sandbox::new();
    let expected = json!({
        "report_type": "all", "timestamp": "<timestamp>",
        "console_version": "<producer-version>", "exit_code": 0,
        "coverage": golden()["coverage"]["data"],
        "summary": golden()["summary"]["data"]
    });
    let output = sandbox.launch(&["--quiet", "--report", "coverage,summary"]);
    assert_report(&output, &expected);
    assert_quiet_stderr(&output);
}

#[test]
fn cli_progress_is_stderr_only_and_quiet_preserves_startup_log() {
    let sandbox = Sandbox::new();
    let output = sandbox.launch(&["--report", "summary"]);
    assert_report(&output, &golden()["summary"]);
    let lines = stderr_lines(&output);
    assert_eq!(lines.len(), 4);
    assert_initial_log(&lines[0]);
    assert_eq!(
        lines[1],
        format!("[vfa-tui] loading catalog from {}", sandbox.root.display())
    );
    assert_eq!(
        lines[2],
        format!(
            "[vfa-tui] workspace registry not found at {}; proceeding with 0 workspaces",
            sandbox.registry.display()
        )
    );
    assert_eq!(lines[3], "[vfa-tui] 0 workspace(s) in scope");
}

#[test]
fn cli_format_flags_preserve_legacy_json_and_markdown_fallback() {
    for format in ["json", "table", "markdown"] {
        let sandbox = Sandbox::new();
        let output = sandbox.launch(&["--quiet", "--report", "summary", "--format", format]);
        assert_eq!(output.status.code(), Some(0));
        assert_quiet_stderr(&output);
        let text = std::str::from_utf8(&output.stdout).unwrap();
        // These envelope-shaped reports currently render JSON even in table
        // mode, with a fenced JSON block for Markdown. Capture, do not fix it.
        let body = if format == "markdown" {
            text.strip_prefix("```json\n")
                .and_then(|body| body.strip_suffix("\n```\n\n"))
                .expect("legacy fenced Markdown fallback")
        } else {
            text.strip_suffix('\n').expect("one trailing newline")
        };
        let value: Value = serde_json::from_str(body).unwrap();
        assert!(report_matches(value.clone(), 0, &golden()["summary"]));
        assert_eq!(body, serde_json::to_string_pretty(&value).unwrap());
    }
}

#[test]
fn cli_registered_workspace_coverage_writes_canonical_cache_and_filter_retains_it() {
    let sandbox = Sandbox::new();
    // CLI argv paths exercise Unicode. The legacy registry expander corrupts
    // non-ASCII bytes, so keep this cache-side-effect baseline ASCII; that
    // independent production defect is recorded in task 1.1's eval report.
    let managed = sandbox._temp.path().join("managed project");
    std::fs::create_dir(&managed).unwrap();
    let canonical = managed.canonicalize().unwrap();
    let registry = json!({"workspace": [{"path": canonical, "name": "managed project"}]});
    std::fs::write(&sandbox.registry, toml::to_string(&registry).unwrap()).unwrap();
    let output = sandbox.launch(&["--quiet", "--report", "coverage"]);
    let mut expected = golden()["coverage"].clone();
    expected["data"] = json!({
        "aggregate_coverage_score": "0.0%", "total_assets": 12,
        "total_workspaces": 1,
        "workspace_scores": [{"coverage_score": "0.0%", "workspace": "managed project"}]
    });
    let report = assert_report(&output, &expected);
    let conn = Connection::open(&sandbox.index).unwrap();
    let row: (String, String, f64, String) = conn.query_row(
        "SELECT workspace_path, workspace_name, coverage_score, computed_at FROM coverage_cache",
        [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    ).unwrap();
    assert_eq!(
        row,
        (
            canonical.to_str().unwrap().into(),
            "managed project".into(),
            0.0,
            report["timestamp"].as_str().unwrap().into()
        )
    );
    drop(conn);
    let output = sandbox.launch(&[
        "--quiet",
        "--report",
        "coverage",
        "--workspace-filter",
        "no-match",
    ]);
    assert_report(&output, &golden()["coverage"]);
    let conn = Connection::open(&sandbox.index).unwrap();
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM coverage_cache", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn cli_operational_partial_and_compliance_exit_codes_are_exact() {
    let sandbox = Sandbox::new();
    std::fs::write(&sandbox.registry, "[[workspace]\n").unwrap();
    let output = sandbox.launch(&["--quiet", "--report", "summary"]);
    let mut expected = golden()["summary"].clone();
    expected["exit_code"] = json!(2);
    assert_report(&output, &expected);
    assert_quiet_stderr(&output);

    // The file remains a workspace marker even though its catalog is corrupt.
    // Partial catalog failure takes precedence over the simultaneous bad registry.
    std::fs::write(sandbox.root.join("catalog/agents.json"), "not JSON").unwrap();
    expected["exit_code"] = json!(3);
    expected["data"]["total_agents"] = json!(0);
    let output = sandbox.launch(&["--quiet", "--report", "summary"]);
    assert_report(&output, &expected);
    assert_quiet_stderr(&output);

    let sandbox = Sandbox::new();
    std::fs::write(sandbox.root.join("README.md"), "changed fixture\n").unwrap();
    let output = sandbox.launch(&["--quiet", "--report", "integrity"]);
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(value["exit_code"], 1);
    assert_eq!(value["data"]["fail_count"], 1);
    assert_eq!(value["data"]["missing_count"], 2);
    assert_eq!(value["data"]["total_files"], 3);
    assert_quiet_stderr(&output);
}

#[test]
fn cli_usage_and_workspace_failures_emit_no_report_or_database() {
    let sandbox = Sandbox::new();
    let output = sandbox.launch(&["--report", "summary", "--format", "invalid"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("invalid value 'invalid'"));
    assert!(stderr.contains("--format"));
    assert!(!sandbox.index.exists());

    std::fs::remove_file(sandbox.root.join("catalog/agents.json")).unwrap();
    let output = sandbox.launch(&["--quiet", "--report", "summary"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("[vfa-tui] workspace detection failed:"));
    assert!(stderr.contains("catalog/agents.json"));
    assert!(!sandbox.index.exists());
}

#[test]
fn cli_preserves_existing_audit_and_appends_once_per_invocation() {
    let sandbox = Sandbox::new();
    let first = sandbox.launch(&["--quiet", "--report", "summary"]);
    assert_report(&first, &golden()["summary"]);
    let conn = Connection::open(&sandbox.index).unwrap();
    assert_eq!(schema_snapshot(&conn), golden()["schema"]);
    let first_row: (i64, String, String, String, String) = conn
        .query_row(
            "SELECT id, timestamp, details, entry_hash, prev_hash FROM audit_log",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .unwrap();
    assert_eq!(first_row.0, 1);
    assert!(first_row.4.is_empty());
    assert_eq!(first_row.3.len(), 64);
    assert!(first_row.3.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_eq!(
        serde_json::from_str::<Value>(&first_row.2).unwrap(),
        json!({"report_types": ["summary"], "exit_code": 0})
    );
    drop(conn);

    let second = sandbox.launch(&["--quiet", "--report", "summary"]);
    assert_report(&second, &golden()["summary"]);
    let conn = Connection::open(&sandbox.index).unwrap();
    assert_eq!(schema_snapshot(&conn), golden()["schema"]);
    let preserved: (i64, String, String, String, String) = conn
        .query_row(
            "SELECT id, timestamp, details, entry_hash, prev_hash FROM audit_log WHERE id=1",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .unwrap();
    assert_eq!(preserved, first_row);
    let appended: (i64, String, String, String, String) = conn
        .query_row(
            "SELECT id, event_type, subject, operator, prev_hash FROM audit_log WHERE id=2",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .unwrap();
    assert_eq!(
        appended,
        (
            2,
            "operator_action".into(),
            "headless_report".into(),
            "headless".into(),
            first_row.3
        )
    );
    for (table, count) in [
        ("audit_log", 2),
        ("coverage_cache", 0),
        ("drift_history", 0),
    ] {
        assert_eq!(
            conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            count
        );
    }
}

#[test]
fn cli_index_failure_does_not_change_legacy_report_success() {
    let mut sandbox = Sandbox::new();
    sandbox.index = sandbox.root.join("directory-not-database");
    std::fs::create_dir(&sandbox.index).unwrap();
    let output = sandbox.launch(&["--quiet", "--report", "summary"]);
    assert_report(&output, &golden()["summary"]);
    assert_quiet_stderr(&output);
    assert_eq!(std::fs::read_dir(&sandbox.index).unwrap().count(), 0);
}

#[test]
fn golden_oracle_rejects_envelope_payload_version_exit_and_schema_mutants() {
    let expected = golden()["summary"].clone();
    let mut baseline = expected.clone();
    baseline["timestamp"] = json!("2000-01-01T00:00:00Z");
    baseline["console_version"] = json!(env!("CARGO_PKG_VERSION"));
    assert!(report_matches(baseline.clone(), 0, &expected));
    let mut envelope = baseline.clone();
    let data = envelope.as_object_mut().unwrap().remove("data").unwrap();
    envelope["payload"] = data;
    let mut payload = baseline.clone();
    payload["data"]["total_agents"] = json!(999);
    let mut version = baseline.clone();
    version["console_version"] = json!("wrong-producer");
    let mut exit = baseline.clone();
    exit["exit_code"] = json!(3);
    let mut timestamp = baseline.clone();
    timestamp["timestamp"] = json!("not a timestamp");
    for mutant in [envelope, payload, version, exit, timestamp] {
        assert!(!report_matches(mutant, 0, &expected));
    }
    assert!(!report_matches(baseline, 2, &expected));
    let sandbox = Sandbox::new();
    let output = sandbox.launch(&["--quiet", "--report", "summary"]);
    assert_report(&output, &expected);
    let conn = Connection::open(&sandbox.index).unwrap();
    let actual_schema = schema_snapshot(&conn);
    assert_eq!(actual_schema, golden()["schema"]);
    let mut wrong_version = golden()["schema"].clone();
    wrong_version["schema_version"] = json!("999");
    assert_ne!(actual_schema, wrong_version);
    let mut missing_column = golden()["schema"].clone();
    missing_column["coverage_cache"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert_ne!(actual_schema, missing_column);
    for statement in [
        "UPDATE audit_log SET subject='mutant' WHERE id=1",
        "DELETE FROM audit_log WHERE id=1",
    ] {
        assert!(conn
            .execute(statement, [])
            .unwrap_err()
            .to_string()
            .contains("append-only"));
    }
    conn.execute("DROP TRIGGER audit_log_no_delete", [])
        .unwrap();
    assert_ne!(schema_snapshot(&conn), golden()["schema"]);
}
