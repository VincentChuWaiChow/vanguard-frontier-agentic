//! Shared domain compatibility cases under the Desktop consumer lock graph.
//! Shared cases retain their independent reviewed fixtures and negative oracles.

use std::path::{Path, PathBuf};

#[path = "../../vfa-tui/tests/support/engine_cases.rs"]
mod engine_cases;

use engine_cases::TestResult;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../vfa-tui/tests/fixtures/compatibility/core")
}

#[test]
fn public_domain_imports_and_fixed_catalog_contract() -> TestResult {
    engine_cases::public_domain_imports_and_fixed_catalog_contract(&fixtures())
}

#[test]
fn same_oracle_rejects_payload_schema_order_and_missing_fixture_mutants() -> TestResult {
    engine_cases::same_oracle_rejects_payload_schema_order_and_missing_fixture_mutants(&fixtures())
}

#[test]
fn catalog_schema_rejects_unknown_fields_and_wrong_discriminator() -> TestResult {
    engine_cases::catalog_schema_rejects_unknown_fields_and_wrong_discriminator(&fixtures())
}

#[test]
fn malformed_collection_preserves_other_collections_and_reload_retains_last_good() -> TestResult {
    engine_cases::malformed_collection_preserves_other_collections_and_reload_retains_last_good(
        &fixtures(),
    )
}

#[test]
fn missing_catalog_file_is_an_error_not_a_healthy_empty_collection() -> TestResult {
    engine_cases::missing_catalog_file_is_an_error_not_a_healthy_empty_collection(&fixtures())
}
