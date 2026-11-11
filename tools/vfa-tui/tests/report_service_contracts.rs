//! Shared report contracts: no Cli, terminal, global logging or implicit writes.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};
use tempfile::TempDir;
use vfa_tui::headless::{ReportInputStatus, ReportProducer, ReportRequest, ReportService};
use vfa_tui::models::report::{OutputFormat, ReportType};

const CATALOG_FILES: [&str; 6] = [
    "agents.json",
    "skills.json",
    "rules.json",
    "mcp-references.json",
    "install-roles.json",
    "asset-integrity.json",
];

struct Fixture {
    temp: TempDir,
    root: PathBuf,
    registry: PathBuf,
    policy: PathBuf,
    index: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("report source ü with spaces");
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        std::fs::create_dir_all(root.join("catalog")).unwrap();
        for name in CATALOG_FILES {
            std::fs::copy(
                source.join("catalog").join(name),
                root.join("catalog").join(name),
            )
            .expect("required baseline input fixture");
        }
        std::fs::copy(source.join("package.json"), root.join("package.json")).unwrap();
        Self {
            registry: root.join("registry.toml"),
            policy: root.join("policy.toml"),
            index: root.join("index ü").join("legacy.db"),
            root,
            temp,
        }
    }

    fn request(&self, types: Vec<ReportType>) -> ReportRequest {
        request(&self.root, &self.registry, &self.policy, &self.index, types)
    }
}

fn request(
    root: &Path,
    registry: &Path,
    policy: &Path,
    index: &Path,
    report_types: Vec<ReportType>,
) -> ReportRequest {
    ReportRequest {
        catalog_root: root.to_owned(),
        registry_path: registry.to_owned(),
        policy_path: policy.to_owned(),
        index_path: index.to_owned(),
        report_types,
        format: OutputFormat::Json,
        workspace_filter: None,
        producer: ReportProducer {
            name: "vfa-tui".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        },
    }
}

fn golden() -> Value {
    serde_json::from_str(include_str!("fixtures/compatibility/reports/golden.json"))
        .expect("required independently fixed report golden")
}

fn normalized(mut payload: Value, expected_version: &str) -> Value {
    assert_eq!(payload["console_version"], expected_version);
    chrono::DateTime::parse_from_rfc3339(payload["timestamp"].as_str().unwrap()).unwrap();
    payload["timestamp"] = json!("<timestamp>");
    payload["console_version"] = json!("<producer-version>");
    payload
}

#[test]
fn service_matches_fixed_single_combined_and_default_report_contracts() {
    let fixture = Fixture::new();
    for (types, expected) in [
        (vec![], golden()["summary"].clone()),
        (vec![ReportType::Summary], golden()["summary"].clone()),
        (vec![ReportType::Coverage], golden()["coverage"].clone()),
        (
            vec![ReportType::Coverage, ReportType::Summary],
            json!({
                "report_type": "all", "timestamp": "<timestamp>",
                "console_version": "<producer-version>", "exit_code": 0,
                "coverage": golden()["coverage"]["data"],
                "summary": golden()["summary"]["data"]
            }),
        ),
    ] {
        let request = fixture.request(types);
        let result = ReportService::generate(&request);
        assert_eq!(result.exit_code, 0);
        assert_eq!(
            normalized(result.payload, &request.producer.version),
            expected
        );
        assert_eq!(result.producer.name, "vfa-tui");
        assert_eq!(result.producer.version, env!("CARGO_PKG_VERSION"));
    }
    assert!(!fixture.index.exists());
    assert!(!fixture.index.parent().unwrap().exists());
}

#[test]
fn caller_identity_is_explicit_and_result_outlives_request_and_inputs() {
    let fixture = Fixture::new();
    let mut request = fixture.request(vec![ReportType::Summary]);
    request.producer = ReportProducer {
        name: "desktop-probe".into(),
        version: "9.8.7".into(),
    };
    let result = ReportService::generate(&request);
    assert_eq!(result.producer.name, "desktop-probe");
    assert_eq!(result.producer.version, "9.8.7");
    assert_eq!(result.payload["console_version"], "9.8.7");
    // Caller metadata is separate; the old machine-readable envelope is exact.
    assert_eq!(
        normalized(result.payload.clone(), "9.8.7"),
        golden()["summary"]
    );
    assert_eq!(result.provenance.catalog_root, request.catalog_root);
    assert_eq!(result.provenance.registry_path, request.registry_path);
    assert_eq!(result.provenance.policy_path, request.policy_path);
    assert_eq!(result.provenance.index_path, request.index_path);
    drop(request);
    drop(fixture);
    assert_eq!(result.payload["data"]["total_agents"], 5);
    assert_eq!(result.producer.name, "desktop-probe");
}

#[test]
fn fixed_oracle_rejects_producer_envelope_exit_and_nested_data_mutations() {
    let fixture = Fixture::new();
    let request = fixture.request(vec![ReportType::Summary]);
    let result = ReportService::generate(&request);
    let accepts = |payload: Value, exit_code: u8, producer: &ReportProducer| {
        exit_code == 0
            && producer == &request.producer
            && payload["console_version"] == request.producer.version
            && payload["timestamp"]
                .as_str()
                .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
                .is_some()
            && normalized(payload, &request.producer.version) == golden()["summary"]
    };
    assert!(accepts(
        result.payload.clone(),
        result.exit_code,
        &result.producer
    ));
    for (pointer, value) in [
        ("/console_version", json!("wrong-version")),
        ("/timestamp", json!("not-a-timestamp")),
        ("/report_type", json!("coverage")),
        ("/exit_code", json!(2)),
        ("/data/total_agents", json!(999)),
        ("/data/total_violations", json!(99)),
    ] {
        let mut mutant = result.payload.clone();
        *mutant.pointer_mut(pointer).unwrap() = value;
        assert!(
            !accepts(mutant, result.exit_code, &result.producer),
            "accepted {pointer}"
        );
    }
    let mut flat_envelope = result.payload.clone();
    let data = flat_envelope
        .as_object_mut()
        .unwrap()
        .remove("data")
        .unwrap();
    flat_envelope
        .as_object_mut()
        .unwrap()
        .extend(data.as_object().unwrap().clone());
    assert!(!accepts(flat_envelope, result.exit_code, &result.producer));
    assert!(!accepts(result.payload.clone(), 2, &result.producer));
    let wrong_caller = ReportProducer {
        name: "implicit-caller".into(),
        ..result.producer.clone()
    };
    assert!(!accepts(result.payload, result.exit_code, &wrong_caller));
}

#[test]
fn all_report_routes_keep_the_combined_keyset_without_index_writes() {
    let fixture = Fixture::new();
    let request = fixture.request(vec![ReportType::All]);
    let result = ReportService::generate(&request);
    let mut keys: Vec<_> = result
        .payload
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "console_version",
            "coverage",
            "dependencies",
            "drift",
            "exit_code",
            "gates",
            "integrity",
            "lifecycle",
            "report_type",
            "stale",
            "summary",
            "timestamp",
            "versions",
            "violations",
        ]
    );
    assert_eq!(result.payload["report_type"], "all");
    assert_eq!(result.payload["summary"], golden()["summary"]["data"]);
    assert_eq!(result.payload["coverage"], golden()["coverage"]["data"]);
    assert!(!fixture.index.parent().unwrap().exists());
}

#[test]
fn missing_invalid_and_partial_input_quality_is_not_healthy_zero() {
    let fixture = Fixture::new();
    let request = fixture.request(vec![ReportType::Summary]);
    let result = ReportService::generate(&request);
    assert_eq!(result.quality.catalog, ReportInputStatus::Current);
    assert_eq!(result.quality.registry, ReportInputStatus::NotConfigured);
    assert_eq!(result.quality.policy, ReportInputStatus::NotConfigured);
    assert_eq!(result.quality.workspaces, ReportInputStatus::NotConfigured);
    assert_eq!(result.exit_code, 0); // Legacy exit semantics are not quality.
    assert_eq!(result.payload["data"]["total_violations"], 0);

    std::fs::write(&fixture.registry, "[[workspace]\n").unwrap();
    std::fs::write(&fixture.policy, "[not valid TOML").unwrap();
    let result = ReportService::generate(&request);
    assert_eq!(result.quality.registry, ReportInputStatus::Unavailable);
    assert_eq!(result.quality.policy, ReportInputStatus::Unavailable);
    assert_eq!(result.quality.workspaces, ReportInputStatus::Unavailable);
    assert_eq!(result.exit_code, 2);
    assert!(!result.diagnostics.is_empty());

    std::fs::write(fixture.root.join("catalog/agents.json"), "not JSON").unwrap();
    let result = ReportService::generate(&request);
    assert_eq!(result.quality.catalog, ReportInputStatus::Partial);
    assert_eq!(result.exit_code, 3);
    assert!(!fixture.index.exists());
}

#[test]
fn mixed_policy_and_workspace_inputs_retain_resolution_evidence() {
    let fixture = Fixture::new();
    let available = fixture.temp.path().join("available workspace");
    let missing = fixture.temp.path().join("missing workspace");
    std::fs::create_dir(&available).unwrap();
    let registry = json!({"workspace": [
        {"path": available, "name": "available"},
        {"path": missing, "name": "missing"},
    ]});
    std::fs::write(&fixture.registry, toml::to_string(&registry).unwrap()).unwrap();
    std::fs::write(
        &fixture.policy,
        r#"
[[rule]]
id = "valid-rule"
description = "Valid rule retained beside an invalid rule"
type = "require_asset"
asset_id = "aws-iam-review-agent"
severity = "warning"
scope = "all"
[[rule]]
id = "invalid-rule"
description = "Unknown rule type is rejected"
type = "not-a-rule-type"
severity = "warning"
scope = "all"
"#,
    )
    .unwrap();
    let request = fixture.request(vec![ReportType::Summary]);
    let result = ReportService::generate(&request);
    assert_eq!(result.quality.registry, ReportInputStatus::Current);
    assert_eq!(result.quality.policy, ReportInputStatus::Partial);
    assert_eq!(result.quality.workspaces, ReportInputStatus::Partial);
    assert_eq!(result.quality.expected_workspaces, 2);
    assert_eq!(result.quality.available_workspaces, 1);
    let unresolved = result
        .provenance
        .workspaces
        .iter()
        .find(|workspace| workspace.name == "missing")
        .unwrap();
    assert_eq!(unresolved.path, missing);
    assert_eq!(unresolved.status, ReportInputStatus::Unavailable);
    assert!(unresolved
        .diagnostic
        .as_ref()
        .is_some_and(|message| !message.is_empty()));
    assert_eq!(result.exit_code, 2);
    std::fs::remove_dir(available).unwrap();
    let result = ReportService::generate(&request);
    assert_eq!(result.quality.workspaces, ReportInputStatus::Unavailable);
    assert_eq!(result.quality.available_workspaces, 0);
    assert!(!fixture.index.exists());
}

#[cfg(unix)]
#[test]
fn all_failed_non_utf8_catalog_paths_remain_unavailable() {
    use std::os::unix::ffi::OsStringExt;
    let temp = tempfile::tempdir().unwrap();
    let root = temp
        .path()
        .join(std::ffi::OsString::from_vec(b"native-root-\xff".to_vec()));
    std::fs::create_dir_all(root.join("catalog")).unwrap();
    for name in CATALOG_FILES {
        std::fs::write(root.join("catalog").join(name), "invalid JSON").unwrap();
    }
    let request = request(
        &root,
        &root.join("registry.toml"),
        &root.join("policy.toml"),
        &root.join("index.db"),
        vec![ReportType::Summary],
    );
    let result = ReportService::generate(&request);
    assert_eq!(result.quality.catalog, ReportInputStatus::Unavailable);
    assert_eq!(result.provenance.catalog_root, root);
    assert_eq!(result.exit_code, 3);
    assert!(!request.index_path.exists());
}

fn directory_bytes(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    std::fs::read_dir(root)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            (
                path.file_name().unwrap().into(),
                std::fs::read(path).unwrap(),
            )
        })
        .collect()
}

#[test]
fn existing_index_and_sidecars_are_untouched_by_read_only_generation() {
    let fixture = Fixture::new();
    std::fs::create_dir_all(fixture.index.parent().unwrap()).unwrap();
    let conn = rusqlite::Connection::open(&fixture.index).unwrap();
    conn.execute_batch(
        "PRAGMA journal_mode=WAL; CREATE TABLE preserved(value INTEGER); INSERT INTO preserved VALUES(41);",
    )
    .unwrap();
    // Keep a real WAL writer open so pre-existing SQLite sidecars, not merely
    // their absence, are included in the byte-for-byte write-effect guard.
    assert!(fixture.index.with_extension("db-wal").exists());
    assert!(fixture.index.with_extension("db-shm").exists());
    let before = directory_bytes(fixture.index.parent().unwrap());
    let result = ReportService::generate(&fixture.request(vec![ReportType::All]));
    assert_eq!(result.exit_code, 0);
    assert_eq!(directory_bytes(fixture.index.parent().unwrap()), before);
    assert_eq!(
        conn.query_row("SELECT value FROM preserved", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        41
    );
    assert!(
        conn.prepare("SELECT * FROM coverage_cache").is_err(),
        "service must not migrate the old index"
    );
}

#[test]
fn format_and_workspace_filter_do_not_change_shared_calculation_semantics() {
    let fixture = Fixture::new();
    // Keep registered path ASCII because the separately documented legacy
    // registry expander does not yet preserve non-ASCII bytes.
    let managed = fixture.temp.path().join("managed project");
    std::fs::create_dir(&managed).unwrap();
    let config = json!({"workspace": [{"path": managed.canonicalize().unwrap(), "name": "managed project"}]});
    std::fs::write(&fixture.registry, toml::to_string(&config).unwrap()).unwrap();
    for format in [
        OutputFormat::Json,
        OutputFormat::Markdown,
        OutputFormat::Table,
    ] {
        let mut request = fixture.request(vec![ReportType::Coverage]);
        request.format = format;
        let result = ReportService::generate(&request);
        assert_eq!(result.payload["data"]["total_workspaces"], 1);
        assert_eq!(result.payload["data"]["aggregate_coverage_score"], "0.0%");
        assert_eq!(
            result.payload["data"]["workspace_scores"],
            json!([{"workspace": "managed project", "coverage_score": "0.0%"}])
        );
        request.workspace_filter = Some("no-match".into());
        let filtered = ReportService::generate(&request);
        assert_eq!(
            normalized(filtered.payload, &request.producer.version),
            golden()["coverage"]
        );
    }
    assert!(!fixture.index.exists());
}

#[cfg(feature = "terminal")]
#[test]
fn pure_service_and_actual_legacy_cli_match_without_sharing_an_oracle() {
    let fixture = Fixture::new();
    let request = fixture.request(vec![ReportType::Summary]);
    let result = ReportService::generate(&request);
    let output = Command::new(env!("CARGO_BIN_EXE_vfa-tui"))
        .arg("--workspace")
        .arg(&fixture.root)
        .arg("--registry")
        .arg(&fixture.registry)
        .arg("--policies")
        .arg(&fixture.policy)
        .arg("--index-path")
        .arg(&fixture.index)
        .args(["--quiet", "--no-color", "--report", "summary"])
        .env("USER", "compatibility")
        .env("USERNAME", "compatibility")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(i32::from(result.exit_code)));
    let cli: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        normalized(result.payload, &request.producer.version),
        golden()["summary"]
    );
    assert_eq!(
        normalized(cli, &request.producer.version),
        golden()["summary"]
    );
    assert!(
        fixture.index.exists(),
        "only legacy adapter applies persistence"
    );
}

#[test]
fn service_has_no_stdio_or_global_subscriber_side_effects() {
    const MODE: &str = "VFA_SERVICE_PROBE_CHILD";
    const BEGIN: &str = "VFA_SERVICE_BEGIN";
    const END: &str = "VFA_SERVICE_END";
    if std::env::var_os(MODE).is_some() {
        use std::io::Write;
        let path = |key| PathBuf::from(std::env::var_os(key).expect("parent-provided probe path"));
        let request = request(
            &path("VFA_SOURCE"),
            &path("VFA_REGISTRY"),
            &path("VFA_POLICY"),
            &path("VFA_INDEX"),
            vec![ReportType::All],
        );
        println!("{BEGIN}");
        eprintln!("{BEGIN}");
        std::io::stdout().flush().unwrap();
        std::io::stderr().flush().unwrap();
        let result = ReportService::generate(&request);
        assert_eq!(result.exit_code, 0);
        let invalid_inputs = ReportRequest {
            registry_path: request.registry_path.with_extension("invalid.toml"),
            policy_path: request.policy_path.with_extension("invalid.toml"),
            ..request
        };
        assert_eq!(ReportService::generate(&invalid_inputs).exit_code, 2);
        tracing::subscriber::set_global_default(tracing::subscriber::NoSubscriber::default())
            .expect("service must not install a global tracing subscriber");
        println!("{END}");
        eprintln!("{END}");
        return;
    }
    let fixture = Fixture::new();
    std::fs::write(
        fixture.registry.with_extension("invalid.toml"),
        "not valid TOML [",
    )
    .unwrap();
    std::fs::write(
        fixture.policy.with_extension("invalid.toml"),
        "not valid TOML [",
    )
    .unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap());
    child
        .args([
            "--exact",
            "service_has_no_stdio_or_global_subscriber_side_effects",
            "--nocapture",
            "--test-threads=1",
        ])
        .env_clear()
        .env(MODE, "1")
        .env("VFA_SOURCE", &fixture.root)
        .env("VFA_REGISTRY", &fixture.registry)
        .env("VFA_POLICY", &fixture.policy)
        .env("VFA_INDEX", &fixture.index)
        .env("HOME", fixture.temp.path())
        .env("USERPROFILE", fixture.temp.path())
        .env("XDG_CONFIG_HOME", fixture.temp.path())
        .env("XDG_DATA_HOME", fixture.temp.path())
        .env("USER", "compatibility")
        .env("USERNAME", "compatibility");
    if let Some(system_root) = std::env::var_os("SystemRoot") {
        child.env("SystemRoot", system_root);
    }
    let output = child.output().unwrap();
    assert!(
        output.status.success(),
        "child failure: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for stream in [&output.stdout, &output.stderr] {
        let text = std::str::from_utf8(stream).unwrap();
        let between = text
            .split_once(&format!("{BEGIN}\n"))
            .expect("probe entered service call")
            .1
            .split_once(&format!("{END}\n"))
            .expect("probe returned from service call")
            .0;
        assert_eq!(between, "", "service wrote to a process stream");
    }
    assert!(!fixture.index.exists());
}
