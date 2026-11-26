use std::path::{Path, PathBuf};
use tempfile::TempDir;
use vfa_tui::headless::{ReportInputStatus, ReportProducer, ReportRequest, ReportService};
use vfa_tui::models::report::{OutputFormat, ReportType};

struct Fixture {
    _temp: TempDir,
    root: PathBuf,
    workspace: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("source");
        let workspace = temp.path().join("alpha");
        std::fs::create_dir_all(root.join("catalog")).unwrap();
        std::fs::create_dir_all(&workspace).unwrap();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/catalog");
        for name in [
            "agents.json",
            "skills.json",
            "rules.json",
            "mcp-references.json",
            "install-roles.json",
            "asset-integrity.json",
        ] {
            std::fs::copy(source.join(name), root.join("catalog").join(name)).unwrap();
        }
        std::fs::write(
            root.join("registry.toml"),
            format!("[[workspace]]\npath = {:?}\nname = \"alpha\"\n", workspace),
        )
        .unwrap();
        std::fs::write(root.join("policy.toml"), "").unwrap();
        Self {
            _temp: temp,
            root,
            workspace,
        }
    }
    fn request(&self) -> ReportRequest {
        ReportRequest {
            catalog_root: self.root.clone(),
            registry_path: self.root.join("registry.toml"),
            policy_path: self.root.join("policy.toml"),
            index_path: self.root.join("unused.db"),
            report_types: vec![
                ReportType::Coverage,
                ReportType::Violations,
                ReportType::Drift,
            ],
            format: OutputFormat::Json,
            workspace_filter: None,
            producer: ReportProducer {
                name: "test".into(),
                version: "1".into(),
            },
        }
    }
}
#[test]
fn review_regression_empty_registered_configuration_is_not_filtered_empty() {
    use vfa_tui::engine::governance::{GovernanceSnapshot, GovernanceStatus};

    let fixture = Fixture::new();
    let registry = fixture.root.join("registry.toml");
    let configured = std::fs::read(&registry).unwrap();
    std::fs::write(&registry, "workspace = []\n").unwrap();
    let request = fixture.request();
    let snapshot = GovernanceSnapshot::generate(&request, 1, 1, None);
    let quality = &snapshot.report().quality;
    assert_eq!(quality.registry, ReportInputStatus::Current);
    assert_eq!(quality.expected_workspaces, 0);
    assert_eq!(quality.workspaces, ReportInputStatus::NotConfigured);
    assert_eq!(snapshot.status(), GovernanceStatus::NotConfigured);
    assert!(!quality.legacy_scores_complete);
    for report in [ReportType::Coverage, ReportType::Violations] {
        assert_eq!(
            quality.for_report(&report),
            ReportInputStatus::NotConfigured
        );
    }

    std::fs::write(&registry, configured).unwrap();
    let mut filtered = request;
    filtered.workspace_filter = Some("no-matching-registered-workspace".into());
    let result = ReportService::generate(&filtered);
    assert_eq!(result.quality.expected_workspaces, 0);
    assert_eq!(result.quality.workspaces, ReportInputStatus::Current);
    assert!(result.quality.legacy_scores_complete);
}

#[cfg(unix)]
#[test]
fn review_regression_unlistable_root_preserves_independently_readable_assets() {
    use std::os::unix::fs::PermissionsExt;

    struct RestorePermissions(PathBuf, std::fs::Permissions);
    impl Drop for RestorePermissions {
        fn drop(&mut self) {
            let _ = std::fs::set_permissions(&self.0, self.1.clone());
        }
    }

    let fixture = Fixture::new();
    std::fs::create_dir(fixture.workspace.join(".claude")).unwrap();
    let asset = fixture.workspace.join(".claude/aws-iam-review-agent.md");
    let bytes = b"# VFA-EXPORT: {\"id\":\"aws-iam-review-agent\",\"version\":\"0.1.0\"}\nfixture\n";
    std::fs::write(&asset, bytes).unwrap();
    let before = ReportService::generate(&fixture.request());
    assert_eq!(before.scans[0].assets.len(), 1);
    let _restore = RestorePermissions(
        fixture.workspace.clone(),
        std::fs::metadata(&fixture.workspace).unwrap().permissions(),
    );
    std::fs::set_permissions(&fixture.workspace, std::fs::Permissions::from_mode(0o111)).unwrap();
    assert!(
        std::fs::read_dir(&fixture.workspace).is_err(),
        "this permission contract requires an unprivileged test process"
    );
    assert_eq!(std::fs::read(&asset).unwrap(), bytes);
    let after = ReportService::generate(&fixture.request());
    assert_eq!(after.scans[0].assets.len(), 1);
    assert_eq!(after.scans[0].assets[0].asset_id, "aws-iam-review-agent");
    assert_eq!(after.payload["coverage"], before.payload["coverage"]);
    assert_eq!(after.quality.workspaces, ReportInputStatus::Partial);
    assert!(after.scans[0]
        .errors
        .iter()
        .any(|error| error.path == fixture.workspace && error.operation == "read_workspace"));
}

#[test]
fn existing_report_must_not_call_failed_harness_a_successful_empty_scan() {
    let f = Fixture::new();
    std::fs::write(f.workspace.join(".claude"), "not a directory").unwrap();
    let result = ReportService::generate(&f.request());
    assert_eq!(result.quality.workspaces, ReportInputStatus::Unavailable);
}

#[test]
fn single_scan_observation_distinguishes_empty_missing_and_mixed_files() {
    use vfa_tui::federation::scanner::{CatalogIndex, WorkspaceScanner};
    use vfa_tui::models::workspace::{ResolvedWorkspace, WorkspaceStatus};
    let f = Fixture::new();
    let ws = ResolvedWorkspace {
        canonical_path: f.workspace.clone(),
        name: "alpha".into(),
        team: None,
        tags: vec![],
        status: WorkspaceStatus::Available,
    };
    let scanner = WorkspaceScanner::new(1);
    let index = CatalogIndex::new([("agent.md".into(), "agent".into(), None)]);
    let empty = scanner.observe_workspace(&ws, &index);
    assert!(empty.errors.is_empty());
    assert_eq!(empty.successful_directories, 1);
    assert!(empty.assets.is_empty());
    std::fs::create_dir(f.workspace.join(".claude")).unwrap();
    std::fs::write(
        f.workspace.join(".claude/agent.md"),
        "# VFA-EXPORT: {\"id\":\"agent\",\"version\":\"1\"}\nbody",
    )
    .unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink("absent", f.workspace.join(".claude/broken.md")).unwrap();
    let mixed = scanner.observe_workspace(&ws, &index);
    assert_eq!(mixed.assets.len(), 1);
    assert!(mixed.assets[0].confirmed);
    assert_eq!(mixed.successful_files, 1);
    #[cfg(unix)]
    {
        assert_eq!(mixed.errors.len(), 1);
        assert!(mixed.errors[0].path.ends_with("broken.md"));
        assert_eq!(mixed.expected_files, 2);
    }
    assert_eq!(scanner.scan_workspace(&ws, &index), mixed.assets);
    std::fs::remove_dir_all(&f.workspace).unwrap();
    let missing = scanner.observe_workspace(&ws, &index);
    assert_eq!(missing.errors.len(), 1);
    assert_eq!(missing.successful_directories, 0);
}

#[test]
fn report_quality_counts_and_exclusions_are_from_the_content_scan() {
    let f = Fixture::new();
    std::fs::create_dir(f.workspace.join(".claude")).unwrap();
    std::fs::write(
        f.workspace.join(".claude/aws-iam-review-agent.md"),
        "# VFA-EXPORT: {\"id\":\"aws-iam-review-agent\",\"version\":\"0.1.0\"}\nchanged",
    )
    .unwrap();
    std::fs::write(f.workspace.join(".cursor"), "bad harness").unwrap();
    let result = ReportService::generate(&f.request());
    assert_eq!(result.quality.workspaces, ReportInputStatus::Partial);
    assert_eq!(result.quality.successful_workspaces, 0);
    assert_eq!(result.scans.len(), 1);
    assert_eq!(result.scans[0].assets.len(), 1);
    assert_eq!(result.scans[0].errors.len(), 1);
    assert!(result.scans[0].errors[0].path.ends_with(".cursor"));
    assert!(!result.quality.legacy_scores_complete);
    assert_eq!(result.payload["coverage"]["total_workspaces"], 1);
    // Stored scan is an owned observation, not a filesystem-dependent accessor.
    std::fs::remove_dir_all(&f.workspace).unwrap();
    assert_eq!(result.scans[0].assets.len(), 1);
}

#[test]
fn snapshot_stale_retains_original_provenance_but_never_crosses_request_or_source() {
    use vfa_tui::engine::governance::{GovernanceSnapshot, GovernanceStatus};
    let f = Fixture::new();
    let request = f.request();
    let first = GovernanceSnapshot::generate(&request, 1, 7, None);
    assert_eq!(first.status(), GovernanceStatus::Current);
    let original = first.report().provenance.observed_at.clone();
    std::fs::write(&request.policy_path, "invalid [").unwrap();
    let failed = GovernanceSnapshot::generate(&request, 1, 7, Some(&first));
    assert_eq!(failed.status(), GovernanceStatus::Stale);
    assert_eq!(failed.report().provenance.observed_at, original);
    assert_eq!(
        failed.attempt().quality.policy,
        ReportInputStatus::Unavailable
    );
    let source_changed = GovernanceSnapshot::generate(&request, 2, 7, Some(&failed));
    assert_eq!(source_changed.status(), GovernanceStatus::Partial);
    let generation_changed = GovernanceSnapshot::generate(&request, 1, 8, Some(&failed));
    assert_eq!(generation_changed.status(), GovernanceStatus::Partial);
    let mut changed = request.clone();
    changed.workspace_filter = Some("does-not-match".into());
    let request_changed = GovernanceSnapshot::generate(&changed, 1, 7, Some(&failed));
    assert_eq!(request_changed.status(), GovernanceStatus::Partial);
    std::fs::remove_file(&request.policy_path).unwrap();
    let absent = GovernanceSnapshot::generate(&request, 1, 7, None);
    assert_eq!(absent.status(), GovernanceStatus::Partial);
    assert_eq!(
        absent.report().quality.policy,
        ReportInputStatus::NotConfigured
    );
}

#[test]
fn two_fixed_workspaces_have_exact_coverage_drift_and_suppressed_policy_results() {
    use serde_json::json;
    let f = Fixture::new();
    let beta = f.workspace.parent().unwrap().join("beta");
    std::fs::create_dir_all(beta.join(".claude")).unwrap();
    std::fs::create_dir_all(f.workspace.join(".claude")).unwrap();
    std::fs::write(f.root.join("registry.toml"), format!("[[workspace]]\npath = {:?}\nname = \"alpha\"\n[[workspace]]\npath = {:?}\nname = \"beta\"\n", f.workspace, beta)).unwrap();
    let agents: serde_json::Value =
        serde_json::from_slice(&std::fs::read(f.root.join("catalog/agents.json")).unwrap())
            .unwrap();
    std::fs::write(
        f.root.join("catalog/agents.json"),
        serde_json::to_vec(&json!([agents[0], agents[1]])).unwrap(),
    )
    .unwrap();
    for name in ["skills.json", "rules.json", "mcp-references.json"] {
        std::fs::write(f.root.join("catalog").join(name), "[]").unwrap();
    }
    let alpha_content =
        "# VFA-EXPORT: {\"id\":\"aws-iam-review-agent\",\"version\":\"0.1.0\"}\nalpha copy\n";
    // Independent fixed digest from the fixture bytes, not from the scanner.
    let alpha_hash = "7cc4391839bba04a433b70528012002879daf72c842b4e14d2e87a35f07d8dea";
    std::fs::write(
        f.workspace.join(".claude/aws-iam-review-agent.md"),
        alpha_content,
    )
    .unwrap();
    std::fs::write(
        beta.join(".claude/aws-s3-security-agent.md"),
        "# VFA-EXPORT: {\"id\":\"aws-s3-security-agent\",\"version\":\"0.2.0\"}\nchanged s3\n",
    )
    .unwrap();
    std::fs::write(f.root.join("catalog/asset-integrity.json"), serde_json::to_vec(&json!({"manifest_version":1,"algorithm":"sha256","scope":{"trees":[],"root_files":[]},"trees":[],"root_files":[{"path":"aws-iam-review-agent","sha256":alpha_hash,"bytes":1},{"path":"aws-s3-security-agent","sha256":"0000000000000000000000000000000000000000000000000000000000000000","bytes":1}],"aggregate_sha256":"fixture"})).unwrap()).unwrap();
    std::fs::write(
        f.root.join("policy.toml"),
        include_str!("fixtures/governance/policy.toml"),
    )
    .unwrap();
    let result = ReportService::generate(&f.request());
    assert_eq!(result.quality.workspaces, ReportInputStatus::Current);
    assert_eq!(result.quality.expected_workspaces, 2);
    assert_eq!(result.quality.successful_workspaces, 2);
    assert!(result.quality.legacy_scores_complete);
    assert_eq!(
        result
            .scans
            .iter()
            .map(|scan| scan.assets.len())
            .collect::<Vec<_>>(),
        [1, 1]
    );
    assert_eq!(
        result.payload["coverage"],
        json!({"total_assets":2,"total_workspaces":2,"aggregate_coverage_score":"25.0%","workspace_scores":[{"workspace":"alpha","coverage_score":"50.0%"},{"workspace":"beta","coverage_score":"0.0%"}]})
    );
    assert_eq!(result.payload["drift"]["total_drifted"], 1);
    assert_eq!(
        result.payload["drift"]["records"][0]["asset_id"],
        "aws-s3-security-agent"
    );
    assert_eq!(
        result.payload["drift"]["records"][0]["kind"],
        "content_drift"
    );
    assert_eq!(result.payload["violations"]["total_violations"], 1);
    assert_eq!(
        result.payload["violations"]["violations"][0]["workspace"],
        "beta"
    );
    assert_eq!(
        result.payload["violations"]["violations"][0]["rule_id"],
        "require-iam"
    );
    assert_eq!(result.exit_code, 1);
}

#[test]
fn mixed_workspace_failures_name_every_exclusion_and_keep_denominators() {
    let f = Fixture::new();
    let missing = f.workspace.parent().unwrap().join("missing");
    std::fs::write(f.root.join("registry.toml"), format!("[[workspace]]\npath = {:?}\nname = \"alpha\"\n[[workspace]]\npath = {:?}\nname = \"missing\"\n", f.workspace, missing)).unwrap();
    let result = ReportService::generate(&f.request());
    assert_eq!(result.quality.workspaces, ReportInputStatus::Partial);
    assert_eq!(result.quality.expected_workspaces, 2);
    assert_eq!(result.quality.available_workspaces, 1);
    assert_eq!(result.quality.successful_workspaces, 1);
    assert!(!result.quality.legacy_scores_complete);
    assert_eq!(result.scans[1].workspace, missing);
    assert_eq!(result.scans[1].errors.len(), 1);
    assert_eq!(result.scans[1].errors[0].operation, "read_workspace");
    assert_eq!(
        result.provenance.workspaces[1].status,
        ReportInputStatus::Unavailable
    );
    assert!(result.provenance.workspaces[1].diagnostic.is_some());
    // Legacy payload still includes its historical two-workspace denominator.
    assert_eq!(result.payload["coverage"]["total_workspaces"], 2);
}

#[test]
fn absent_unreadable_and_invalid_policy_and_registry_are_not_healthy_zeros() {
    let f = Fixture::new();
    let request = f.request();
    std::fs::remove_file(&request.registry_path).unwrap();
    let absent = ReportService::generate(&request);
    assert_eq!(absent.quality.registry, ReportInputStatus::NotConfigured);
    assert_eq!(absent.quality.workspaces, ReportInputStatus::NotConfigured);
    assert!(!absent.quality.legacy_scores_complete);
    std::fs::write(&request.registry_path, "invalid [").unwrap();
    let invalid = ReportService::generate(&request);
    assert_eq!(invalid.quality.registry, ReportInputStatus::Unavailable);
    assert_eq!(invalid.quality.workspaces, ReportInputStatus::Unavailable);
    std::fs::remove_file(&request.policy_path).unwrap();
    let absent = ReportService::generate(&request);
    assert_eq!(absent.quality.policy, ReportInputStatus::NotConfigured);
    std::fs::create_dir(&request.policy_path).unwrap();
    let unreadable = ReportService::generate(&request);
    assert_eq!(unreadable.quality.policy, ReportInputStatus::Unavailable);
    assert!(!unreadable.quality.legacy_scores_complete);
}

#[test]
fn repeated_failed_refresh_is_bounded_and_success_replaces_stale_data() {
    use vfa_tui::engine::governance::{GovernanceSnapshot, GovernanceStatus};
    let f = Fixture::new();
    let request = f.request();
    let mut snapshot = GovernanceSnapshot::generate(&request, 4, 2, None);
    let original = snapshot.report().provenance.observed_at.clone();
    std::fs::write(f.workspace.join(".claude"), "bad harness").unwrap();
    for _ in 0..10 {
        snapshot = GovernanceSnapshot::generate(&request, 4, 2, Some(&snapshot));
        assert_eq!(snapshot.status(), GovernanceStatus::Stale);
        assert_eq!(snapshot.report().provenance.observed_at, original);
        assert_eq!(snapshot.attempt().scans[0].errors.len(), 1);
    }
    std::fs::remove_file(f.workspace.join(".claude")).unwrap();
    let recovered = GovernanceSnapshot::generate(&request, 4, 2, Some(&snapshot));
    assert_eq!(recovered.status(), GovernanceStatus::Current);
    assert!(recovered.report().scans[0].errors.is_empty());
}

#[test]
fn policy_quality_is_report_specific_and_zero_selected_workspaces_is_current_empty() {
    use vfa_tui::engine::governance::{GovernanceSnapshot, GovernanceStatus};
    let f = Fixture::new();
    let mut request = f.request();
    request.report_types = vec![ReportType::Coverage, ReportType::Drift];
    std::fs::write(&request.policy_path, "invalid [").unwrap();
    let invalid = GovernanceSnapshot::generate(&request, 1, 1, None);
    assert_eq!(invalid.status(), GovernanceStatus::Current);
    assert_eq!(invalid.sections()["coverage"], ReportInputStatus::Current);
    assert_eq!(
        invalid.attempt().quality.policy,
        ReportInputStatus::Unavailable
    );
    assert!(invalid.attempt().quality.legacy_scores_complete);
    request.report_types.push(ReportType::Violations);
    let mixed = GovernanceSnapshot::generate(&request, 1, 1, None);
    assert_eq!(mixed.status(), GovernanceStatus::Partial);
    assert_eq!(
        mixed.sections()["violations"],
        ReportInputStatus::Unavailable
    );
    assert_eq!(mixed.sections()["coverage"], ReportInputStatus::Current);
    std::fs::remove_file(&request.policy_path).unwrap();
    let absent = GovernanceSnapshot::generate(&request, 1, 1, None);
    assert_eq!(absent.status(), GovernanceStatus::Partial);
    assert_eq!(
        absent.sections()["violations"],
        ReportInputStatus::NotConfigured
    );
    request.report_types = vec![ReportType::Coverage];
    request.workspace_filter = Some("no-match".into());
    let empty = GovernanceSnapshot::generate(&request, 1, 1, None);
    assert_eq!(empty.status(), GovernanceStatus::Current);
    assert_eq!(empty.report().quality.expected_workspaces, 0);
    assert_eq!(empty.report().quality.successful_workspaces, 0);
    assert_eq!(
        empty.report().quality.workspaces,
        ReportInputStatus::Current
    );
    assert_eq!(empty.report().payload["data"]["total_workspaces"], 0);
    request.report_types = vec![ReportType::Violations];
    let policy_only = GovernanceSnapshot::generate(&request, 1, 1, None);
    assert_eq!(policy_only.status(), GovernanceStatus::NotConfigured);
}

#[cfg(feature = "terminal")]
#[test]
fn legacy_layout_warning_is_preserved_without_a_second_directory_scan() {
    use std::io::Write;
    use std::sync::{Arc, Mutex};
    use vfa_tui::federation::scanner::{CatalogIndex, WorkspaceScanner};
    use vfa_tui::workspace::harness_layout::HarnessDir;
    #[derive(Clone)]
    struct Capture(Arc<Mutex<Vec<u8>>>);
    impl Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let f = Fixture::new();
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let output = Arc::clone(&bytes);
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_writer(move || Capture(Arc::clone(&output)))
        .finish();
    let scanner = WorkspaceScanner::new(1);
    let index = CatalogIndex::new([]);
    tracing::subscriber::with_default(subscriber, || {
        // Successfully empty and failed directories retain the legacy warning.
        scanner.scan_harness_dir(&f.workspace, &HarnessDir::Claude, &f.workspace, &index);
        scanner.scan_harness_dir(
            &f.workspace,
            &HarnessDir::Claude,
            &f.workspace.join("absent"),
            &index,
        );
        std::fs::write(f.workspace.join("notes.txt"), "kiro layout signal").unwrap();
        scanner.scan_harness_dir(&f.workspace, &HarnessDir::Kiro, &f.workspace, &index);
    });
    let text = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
    assert_eq!(
        text.matches("harness directory does not match expected layout — skipping")
            .count(),
        2
    );
    assert!(text.contains("absent"));
    assert!(text.contains(".claude"));
}
