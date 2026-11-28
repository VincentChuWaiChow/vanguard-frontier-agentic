//! Gate DAG execution — parallel gate runner with cascading failure propagation.
//!
//! # Design: injectable runner
//!
//! The core execution logic is parameterised over a `GateRunner` trait so that:
//! - **Production** path uses [`SubprocessRunner`] which delegates to
//!   [`SubprocessExecutor::spawn`] (no shell, sanitized env).
//! - **Tests / property tests** inject a [`MockRunner`] (or any closure) to
//!   produce deterministic pass/fail outcomes without spawning real processes.
//!
//! `GateRunner` uses `BoxFuture` (via `Pin<Box<dyn Future>>`) to remain dyn-compatible
//! without requiring the `async_trait` crate.

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::future::join_all;
use tokio::sync::Semaphore;

use crate::models::gate::{DagGateStatus, GateDAG, GateDefinition, GateResult};
use crate::subprocess::SubprocessExecutor;

// ─────────────────────────────────────────────────────────────────────────────
// GateRunOutcome — headless report row (Req 2.7)
// ─────────────────────────────────────────────────────────────────────────────

/// Per-gate report row suitable for headless `--report gates` JSON output.
///
/// All fields are serde-serializable. `duration_ms` uses `u64` so it is easy
/// to consume from JSON (no need to parse a duration string).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GateRunOutcome {
    /// Unique gate name within the DAG.
    pub name: String,
    /// Terminal status after execution (snake_case serialization).
    pub status: DagGateStatus,
    /// Wall-clock execution time in milliseconds.
    pub duration_ms: u64,
    /// Names of prerequisite gates.
    pub dependencies: Vec<String>,
    /// Human-readable reason the gate was skipped, or `None`.
    pub skip_reason: Option<String>,
}

impl GateRunOutcome {
    /// Build a `GateRunOutcome` from a completed `GateResult` and its definition.
    pub fn from_result(result: &GateResult, def: &GateDefinition) -> Self {
        Self {
            name: result.name.clone(),
            status: result.status.clone(),
            duration_ms: result.duration.as_millis() as u64,
            dependencies: def.dependencies.clone(),
            skip_reason: result.skip_reason.clone(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GateRunner trait
// ─────────────────────────────────────────────────────────────────────────────

/// Abstraction over the mechanism used to actually execute a single gate.
///
/// Uses a `BoxFuture` return type to remain dyn-compatible without the
/// `async_trait` crate.
pub trait GateRunner: Send + Sync {
    /// Execute a single gate and return its result.
    fn run<'a>(
        &'a self,
        def: &'a GateDefinition,
        workspace_root: &'a Path,
    ) -> Pin<Box<dyn Future<Output = GateResult> + Send + 'a>>;
}

// ─────────────────────────────────────────────────────────────────────────────
// SubprocessRunner — production implementation
// ─────────────────────────────────────────────────────────────────────────────

/// Production gate runner that delegates to [`SubprocessExecutor`].
///
/// Runs `def.command` with `def.args` in `workspace_root`, bounded by
/// `def.timeout`.  Captures combined stdout + stderr as the gate output.
pub struct SubprocessRunner;

impl GateRunner for SubprocessRunner {
    fn run<'a>(
        &'a self,
        def: &'a GateDefinition,
        workspace_root: &'a Path,
    ) -> Pin<Box<dyn Future<Output = GateResult> + Send + 'a>> {
        Box::pin(async move {
            let start = Instant::now();
            let timestamp = crate::clock::now_millis();

            let spawn_result =
                SubprocessExecutor::spawn(&def.command, &def.args, workspace_root, def.timeout)
                    .await;

            let mut handle = match spawn_result {
                Ok(h) => h,
                Err(e) => {
                    return GateResult {
                        name: def.name.clone(),
                        status: DagGateStatus::Failed,
                        exit_code: None,
                        duration: start.elapsed(),
                        timestamp,
                        output: format!("failed to spawn: {e}"),
                        skip_reason: None,
                    };
                }
            };

            // Wait with a grace-period timeout.
            let wait_result =
                tokio::time::timeout(def.timeout + Duration::from_secs(5), handle.wait()).await;

            let duration = start.elapsed();

            // Drain any buffered output lines (best-effort).
            let mut output_lines: Vec<String> = Vec::new();
            while let Some(line) = handle.try_recv_stdout() {
                output_lines.push(line.content);
            }
            while let Some(line) = handle.try_recv_stderr() {
                output_lines.push(line.content);
            }
            let output = output_lines.join("\n");

            match wait_result {
                Ok(Ok(code)) => {
                    let status = if code == 0 {
                        DagGateStatus::Passed
                    } else {
                        DagGateStatus::Failed
                    };
                    GateResult {
                        name: def.name.clone(),
                        status,
                        exit_code: Some(code),
                        duration,
                        timestamp,
                        output,
                        skip_reason: None,
                    }
                }
                Ok(Err(e)) => GateResult {
                    name: def.name.clone(),
                    status: DagGateStatus::Failed,
                    exit_code: None,
                    duration,
                    timestamp,
                    output: format!("wait error: {e}"),
                    skip_reason: None,
                },
                Err(_) => {
                    handle.cancel().await.ok();
                    GateResult {
                        name: def.name.clone(),
                        status: DagGateStatus::TimedOut,
                        exit_code: None,
                        duration,
                        timestamp,
                        output,
                        skip_reason: None,
                    }
                }
            }
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MockRunner — test double
// ─────────────────────────────────────────────────────────────────────────────

/// Deterministic gate runner for tests.
///
/// Given a closure `F: Fn(&str) -> DagGateStatus + Send + Sync + 'static`,
/// produces instant results without spawning any processes.
///
/// # Example
/// ```ignore
/// let runner = MockRunner::new(|name| {
///     if name == "lint" { DagGateStatus::Failed } else { DagGateStatus::Passed }
/// });
/// ```
pub struct MockRunner<F>
where
    F: Fn(&str) -> DagGateStatus + Send + Sync + 'static,
{
    outcome_fn: F,
}

impl<F> MockRunner<F>
where
    F: Fn(&str) -> DagGateStatus + Send + Sync + 'static,
{
    /// Create a mock runner from an outcome function.
    pub fn new(outcome_fn: F) -> Self {
        Self { outcome_fn }
    }
}

impl<F> GateRunner for MockRunner<F>
where
    F: Fn(&str) -> DagGateStatus + Send + Sync + 'static,
{
    fn run<'a>(
        &'a self,
        def: &'a GateDefinition,
        _workspace_root: &'a Path,
    ) -> Pin<Box<dyn Future<Output = GateResult> + Send + 'a>> {
        let status = (self.outcome_fn)(&def.name);
        let name = def.name.clone();
        Box::pin(async move {
            GateResult {
                name,
                status,
                exit_code: None,
                duration: Duration::ZERO,
                timestamp: crate::clock::now_millis(),
                output: String::new(),
                skip_reason: None,
            }
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// execute_all
// ─────────────────────────────────────────────────────────────────────────────

/// Execute all gates in the DAG, respecting dependency order and concurrency.
///
/// Gates within the same topological layer are started sequentially while
/// honouring the semaphore-bounded `concurrency_limit`.  Dependent gates only
/// start after their prerequisites complete.
///
/// # Prerequisite failure cascade (Req 2.4)
/// When a gate FAILS or TIMES_OUT, all transitive downstream gates are marked
/// [`DagGateStatus::Skipped`] with a `skip_reason` of `"dependency failed: <gate>"`.
/// Skipped gates are **not** passed to the runner.
///
/// # Parameters
/// - `dag`: the gate graph (from [`build_dag`]).
/// - `concurrency_limit`: max gates running simultaneously (0 → default 4).
/// - `workspace_root`: working directory for subprocess invocations.
/// - `runner`: injectable runner (use [`SubprocessRunner`] in production,
///   [`MockRunner`] in tests).
pub async fn execute_all(
    dag: &GateDAG,
    concurrency_limit: usize,
    workspace_root: &Path,
    runner: &dyn GateRunner,
) -> Vec<GateResult> {
    let limit = if concurrency_limit == 0 {
        4
    } else {
        concurrency_limit
    };
    let semaphore = Arc::new(Semaphore::new(limit));

    // Map gate name → completed result.
    let mut results: HashMap<String, GateResult> = HashMap::new();
    // Map gate name → skip reason (populated on failure cascade).
    let mut skip_because: HashMap<String, String> = HashMap::new();

    for layer in &dag.execution_order {
        // Before executing this layer, resolve any new skips caused by failures
        // from previous layers.
        for gate_name in layer {
            if skip_because.contains_key(gate_name.as_str()) {
                continue;
            }
            let def = dag.gates.iter().find(|g| &g.name == gate_name).unwrap();
            for dep in &def.dependencies {
                // If the dep was already skipped, propagate.
                if let Some(reason) = skip_because.get(dep.as_str()) {
                    skip_because.insert(gate_name.clone(), reason.clone());
                    break;
                }
                // If the dep failed/timed-out, cascade.
                if let Some(dep_result) = results.get(dep.as_str()) {
                    if matches!(
                        dep_result.status,
                        DagGateStatus::Failed | DagGateStatus::TimedOut
                    ) {
                        skip_because.insert(gate_name.clone(), format!("dependency failed: {dep}"));
                        break;
                    }
                }
            }
        }

        let (to_run, to_skip): (Vec<_>, Vec<_>) = layer
            .iter()
            .partition(|name| !skip_because.contains_key(name.as_str()));

        // Record skipped gates immediately and propagate further.
        for name in &to_skip {
            let reason = skip_because.get(name.as_str()).cloned().unwrap_or_default();
            results.insert(
                (*name).clone(),
                GateResult {
                    name: (*name).clone(),
                    status: DagGateStatus::Skipped,
                    exit_code: None,
                    duration: Duration::ZERO,
                    timestamp: crate::clock::now_millis(),
                    output: String::new(),
                    skip_reason: Some(reason.clone()),
                },
            );
            propagate_skip(&mut skip_because, dag, name, &reason);
        }

        // Execute non-skipped gates in this layer concurrently, bounded by the semaphore.
        let layer_futures: Vec<_> = to_run
            .iter()
            .map(|name| {
                let def = dag.gates.iter().find(|g| &g.name == *name).unwrap();
                let semaphore = Arc::clone(&semaphore);
                let name_clone = (*name).clone();
                async move {
                    let _permit = semaphore.acquire().await.expect("semaphore never closed");
                    let result = runner.run(def, workspace_root).await;
                    (name_clone, result)
                }
            })
            .collect();

        let layer_results = join_all(layer_futures).await;
        for (name, result) in layer_results {
            // If this gate failed, propagate skip to its dependents.
            if matches!(
                result.status,
                DagGateStatus::Failed | DagGateStatus::TimedOut
            ) {
                let reason = format!("dependency failed: {}", result.name);
                propagate_skip(&mut skip_because, dag, &result.name, &reason);
            }
            results.insert(name, result);
        }
    }

    // Return results in flattened topological order.
    dag.execution_order
        .iter()
        .flatten()
        .filter_map(|name| results.remove(name))
        .collect()
}

/// Recursively mark all transitive dependents of `failed_gate` as skipped in
/// `skip_because`, using the DAG adjacency map (downstream edges).
fn propagate_skip(
    skip_because: &mut HashMap<String, String>,
    dag: &GateDAG,
    failed_gate: &str,
    reason: &str,
) {
    let mut stack: Vec<String> = Vec::new();
    if let Some(dependents) = dag.adjacency.get(failed_gate) {
        stack.extend(dependents.iter().cloned());
    }
    while let Some(dep) = stack.pop() {
        if skip_because.contains_key(dep.as_str()) {
            continue;
        }
        skip_because.insert(dep.clone(), reason.to_string());
        if let Some(further) = dag.adjacency.get(dep.as_str()) {
            stack.extend(further.iter().cloned());
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// execute_single
// ─────────────────────────────────────────────────────────────────────────────

/// Execute a single named gate, running its unsatisfied prerequisites first.
///
/// Prerequisites are considered satisfied (and skipped) when the cache map
/// contains a valid cached result for that gate.  Cache validity is determined
/// by [`is_cache_valid`].
///
/// # Parameters
/// - `gate_name`: the target gate to execute.
/// - `dag`: the full gate graph.
/// - `cached_results`: map from gate name → `(GateResult, content_hash)`.
///   Pass an empty map to run all prerequisites unconditionally.
/// - `current_hashes`: map from gate name → current content hash for validation.
/// - `workspace_root`: working directory.
/// - `runner`: injectable gate runner.
pub async fn execute_single(
    gate_name: &str,
    dag: &GateDAG,
    cached_results: &HashMap<String, (GateResult, String)>,
    current_hashes: &HashMap<String, String>,
    workspace_root: &Path,
    runner: &dyn GateRunner,
) -> Vec<GateResult> {
    // Collect the transitive closure of prerequisites (including the target gate)
    // in topological order.
    let needed: Vec<String> = required_gates(gate_name, dag);

    let mut results: Vec<GateResult> = Vec::new();

    for name in &needed {
        let def = dag.gates.iter().find(|g| &g.name == name).unwrap();

        // Check if any prerequisite failed before running this gate.
        let mut prereq_failed = false;
        for dep in &def.dependencies {
            if let Some(dep_result) = results.iter().find(|r| &r.name == dep) {
                if matches!(
                    dep_result.status,
                    DagGateStatus::Failed | DagGateStatus::TimedOut
                ) {
                    prereq_failed = true;
                    break;
                }
            }
        }

        // If a prerequisite failed, skip this gate.
        if prereq_failed {
            let failed_dep = def
                .dependencies
                .iter()
                .find(|d| {
                    results
                        .iter()
                        .find(|r| &r.name == *d)
                        .map(|r| {
                            matches!(r.status, DagGateStatus::Failed | DagGateStatus::TimedOut)
                        })
                        .unwrap_or(false)
                })
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            results.push(GateResult {
                name: name.clone(),
                status: DagGateStatus::Skipped,
                exit_code: None,
                duration: Duration::ZERO,
                timestamp: crate::clock::now_millis(),
                output: String::new(),
                skip_reason: Some(format!("dependency failed: {failed_dep}")),
            });
            continue;
        }

        // Check cache validity.
        if let Some((cached_result, cached_hash)) = cached_results.get(name.as_str()) {
            if matches!(cached_result.status, DagGateStatus::Passed) {
                let current_hash = current_hashes
                    .get(name.as_str())
                    .map(|s| s.as_str())
                    .unwrap_or("");
                if is_cache_valid(cached_hash, current_hash) {
                    results.push(cached_result.clone());
                    continue;
                }
            }
        }

        let result = runner.run(def, workspace_root).await;
        results.push(result);
    }

    results
}

/// Return the ordered list of gates that must run to satisfy `gate_name`,
/// including itself, in topological order (prerequisites first).
fn required_gates(gate_name: &str, dag: &GateDAG) -> Vec<String> {
    let mut needed: HashSet<String> = HashSet::new();
    let mut stack = vec![gate_name.to_string()];
    while let Some(current) = stack.pop() {
        if needed.contains(&current) {
            continue;
        }
        needed.insert(current.clone());
        if let Some(def) = dag.gates.iter().find(|g| g.name == current) {
            for dep in &def.dependencies {
                stack.push(dep.clone());
            }
        }
    }

    // Return in topological order as defined in the DAG.
    dag.execution_order
        .iter()
        .flatten()
        .filter(|name| needed.contains(name.as_str()))
        .cloned()
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// is_cache_valid
// ─────────────────────────────────────────────────────────────────────────────

/// Return `true` if the cached content hash still matches the current hash.
///
/// An empty `current_hash` always invalidates the cache to avoid silently
/// accepting a stale result when the hash cannot be computed.
pub fn is_cache_valid(cached_hash: &str, current_hash: &str) -> bool {
    !current_hash.is_empty() && cached_hash == current_hash
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gates::dag::{build_dag, GateDagExecutor};
    use std::time::Duration;
    use tempfile::TempDir;

    fn make_gate(name: &str, deps: &[&str]) -> GateDefinition {
        GateDefinition {
            name: name.to_string(),
            command: "echo".to_string(),
            args: vec![name.to_string()],
            dependencies: deps.iter().map(|s| s.to_string()).collect(),
            timeout: Duration::from_secs(60),
            description: String::new(),
        }
    }

    fn all_pass() -> MockRunner<impl Fn(&str) -> DagGateStatus + Send + Sync + 'static> {
        MockRunner::new(|_| DagGateStatus::Passed)
    }

    fn fail_on(
        failing: &'static str,
    ) -> MockRunner<impl Fn(&str) -> DagGateStatus + Send + Sync + 'static> {
        MockRunner::new(move |name| {
            if name == failing {
                DagGateStatus::Failed
            } else {
                DagGateStatus::Passed
            }
        })
    }

    // ── is_cache_valid ────────────────────────────────────────────────────────

    #[test]
    fn cache_valid_same_hash() {
        assert!(is_cache_valid("abc123", "abc123"));
    }

    #[test]
    fn cache_invalid_different_hash() {
        assert!(!is_cache_valid("abc123", "def456"));
    }

    #[test]
    fn cache_invalid_empty_current() {
        assert!(!is_cache_valid("abc123", ""));
    }

    #[test]
    fn cache_invalid_both_empty() {
        assert!(!is_cache_valid("", ""));
    }

    // ── GateRunOutcome ────────────────────────────────────────────────────────

    #[test]
    fn gate_run_outcome_from_result() {
        let def = make_gate("lint", &[]);
        let result = GateResult {
            name: "lint".to_string(),
            status: DagGateStatus::Passed,
            exit_code: Some(0),
            duration: Duration::from_millis(250),
            timestamp: "2025-01-01T00:00:00.000Z".to_string(),
            output: String::new(),
            skip_reason: None,
        };
        let outcome = GateRunOutcome::from_result(&result, &def);
        assert_eq!(outcome.name, "lint");
        assert_eq!(outcome.duration_ms, 250);
        assert!(outcome.dependencies.is_empty());
    }

    #[test]
    fn gate_run_outcome_serde_round_trip() {
        let def = make_gate("types", &["lint"]);
        let result = GateResult {
            name: "types".to_string(),
            status: DagGateStatus::Skipped,
            exit_code: None,
            duration: Duration::ZERO,
            timestamp: "2025-01-01T00:01:00.000Z".to_string(),
            output: String::new(),
            skip_reason: Some("dependency failed: lint".to_string()),
        };
        let outcome = GateRunOutcome::from_result(&result, &def);
        let json = serde_json::to_string(&outcome).unwrap();
        let decoded: GateRunOutcome = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, "types");
        assert_eq!(decoded.status, DagGateStatus::Skipped);
        assert_eq!(
            decoded.skip_reason.as_deref(),
            Some("dependency failed: lint")
        );
        assert_eq!(decoded.dependencies, vec!["lint"]);
    }

    // ── execute_all — basic execution ─────────────────────────────────────────

    #[tokio::test]
    async fn execute_all_empty_dag_returns_empty() {
        let dag = build_dag(vec![]).unwrap();
        let tmp = TempDir::new().unwrap();
        let runner = all_pass();
        let results = execute_all(&dag, 4, tmp.path(), &runner).await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn execute_all_single_gate_passes() {
        let dag = build_dag(vec![make_gate("lint", &[])]).unwrap();
        let tmp = TempDir::new().unwrap();
        let runner = all_pass();
        let results = execute_all(&dag, 4, tmp.path(), &runner).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "lint");
        assert_eq!(results[0].status, DagGateStatus::Passed);
    }

    #[tokio::test]
    async fn execute_all_chain_all_pass() {
        let gates = vec![
            make_gate("a", &[]),
            make_gate("b", &["a"]),
            make_gate("c", &["b"]),
        ];
        let dag = build_dag(gates).unwrap();
        let tmp = TempDir::new().unwrap();
        let runner = all_pass();
        let results = execute_all(&dag, 4, tmp.path(), &runner).await;
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.status == DagGateStatus::Passed));
    }

    // ── execute_all — prerequisite failure cascade ────────────────────────────

    #[tokio::test]
    async fn execute_all_cascade_skip_on_failure() {
        // lint → types → integration
        // lint FAILS → types + integration should be Skipped
        let gates = vec![
            make_gate("lint", &[]),
            make_gate("types", &["lint"]),
            make_gate("integration", &["types"]),
        ];
        let dag = build_dag(gates).unwrap();
        let tmp = TempDir::new().unwrap();
        let runner = fail_on("lint");
        let results = execute_all(&dag, 4, tmp.path(), &runner).await;

        let lint = results.iter().find(|r| r.name == "lint").unwrap();
        let types = results.iter().find(|r| r.name == "types").unwrap();
        let integration = results.iter().find(|r| r.name == "integration").unwrap();

        assert_eq!(lint.status, DagGateStatus::Failed);
        assert_eq!(types.status, DagGateStatus::Skipped);
        assert_eq!(integration.status, DagGateStatus::Skipped);
        assert!(types.skip_reason.as_deref().unwrap_or("").contains("lint"));
    }

    #[tokio::test]
    async fn execute_all_unrelated_gate_not_skipped_on_failure() {
        // lint → types   AND  build (independent)
        // lint FAILS → types Skipped, but build PASSES
        let gates = vec![
            make_gate("lint", &[]),
            make_gate("types", &["lint"]),
            make_gate("build", &[]),
        ];
        let dag = build_dag(gates).unwrap();
        let tmp = TempDir::new().unwrap();
        let runner = fail_on("lint");
        let results = execute_all(&dag, 4, tmp.path(), &runner).await;

        let types = results.iter().find(|r| r.name == "types").unwrap();
        let build = results.iter().find(|r| r.name == "build").unwrap();

        assert_eq!(types.status, DagGateStatus::Skipped);
        assert_eq!(build.status, DagGateStatus::Passed);
    }

    #[tokio::test]
    async fn execute_all_skip_reason_contains_failing_gate_name() {
        let gates = vec![
            make_gate("a", &[]),
            make_gate("b", &["a"]),
            make_gate("c", &["b"]),
        ];
        let dag = build_dag(gates).unwrap();
        let tmp = TempDir::new().unwrap();
        let runner = fail_on("a");
        let results = execute_all(&dag, 4, tmp.path(), &runner).await;

        let b = results.iter().find(|r| r.name == "b").unwrap();
        let c = results.iter().find(|r| r.name == "c").unwrap();

        assert_eq!(b.status, DagGateStatus::Skipped);
        assert_eq!(c.status, DagGateStatus::Skipped);
        assert!(b
            .skip_reason
            .as_deref()
            .unwrap_or("")
            .contains("dependency failed"));
        assert!(c
            .skip_reason
            .as_deref()
            .unwrap_or("")
            .contains("dependency failed"));
    }

    // ── execute_all — result ordering ─────────────────────────────────────────

    #[tokio::test]
    async fn execute_all_results_in_dag_order() {
        // diamond: a → {b, c} → d
        let gates = vec![
            make_gate("a", &[]),
            make_gate("b", &["a"]),
            make_gate("c", &["a"]),
            make_gate("d", &["b", "c"]),
        ];
        let dag = build_dag(gates).unwrap();
        let tmp = TempDir::new().unwrap();
        let runner = all_pass();
        let results = execute_all(&dag, 4, tmp.path(), &runner).await;
        assert_eq!(results.len(), 4);
        let names: Vec<&str> = results.iter().map(|r| r.name.as_str()).collect();
        let a_pos = names.iter().position(|&n| n == "a").unwrap();
        let b_pos = names.iter().position(|&n| n == "b").unwrap();
        let c_pos = names.iter().position(|&n| n == "c").unwrap();
        let d_pos = names.iter().position(|&n| n == "d").unwrap();
        assert!(a_pos < b_pos);
        assert!(a_pos < c_pos);
        assert!(b_pos < d_pos);
        assert!(c_pos < d_pos);
    }

    // ── execute_single + is_cache_valid ──────────────────────────────────────

    #[tokio::test]
    async fn execute_single_uses_cache_when_valid() {
        // lint → types
        // lint has a valid cached result → should NOT be re-run
        let gates = vec![make_gate("lint", &[]), make_gate("types", &["lint"])];
        let dag = build_dag(gates).unwrap();
        let tmp = TempDir::new().unwrap();

        let cached_lint = GateResult {
            name: "lint".to_string(),
            status: DagGateStatus::Passed,
            exit_code: Some(0),
            duration: Duration::from_millis(100),
            timestamp: "2025-01-01T00:00:00.000Z".to_string(),
            output: String::new(),
            skip_reason: None,
        };

        let mut cached = HashMap::new();
        cached.insert("lint".to_string(), (cached_lint, "abc".to_string()));
        let mut current_hashes = HashMap::new();
        current_hashes.insert("lint".to_string(), "abc".to_string());

        // Fail if "lint" is actually run — cache should prevent execution.
        let runner = fail_on("lint");
        let results =
            execute_single("types", &dag, &cached, &current_hashes, tmp.path(), &runner).await;

        let lint = results.iter().find(|r| r.name == "lint").unwrap();
        assert_eq!(
            lint.status,
            DagGateStatus::Passed,
            "cached result should be used"
        );
    }

    #[tokio::test]
    async fn execute_single_reruns_when_cache_stale() {
        let gates = vec![make_gate("lint", &[])];
        let dag = build_dag(gates).unwrap();
        let tmp = TempDir::new().unwrap();

        let cached_lint = GateResult {
            name: "lint".to_string(),
            status: DagGateStatus::Passed,
            exit_code: Some(0),
            duration: Duration::from_millis(100),
            timestamp: "2025-01-01T00:00:00.000Z".to_string(),
            output: String::new(),
            skip_reason: None,
        };

        let mut cached = HashMap::new();
        cached.insert("lint".to_string(), (cached_lint, "abc".to_string()));
        let mut current_hashes = HashMap::new();
        current_hashes.insert("lint".to_string(), "xyz".to_string()); // different → stale

        let runner = all_pass();
        let results =
            execute_single("lint", &dag, &cached, &current_hashes, tmp.path(), &runner).await;

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, DagGateStatus::Passed);
    }

    // ── GateDagExecutor construction ─────────────────────────────────────────

    #[test]
    fn gate_dag_executor_default_concurrency() {
        let ex = GateDagExecutor::new(0);
        assert_eq!(ex.concurrency_limit, 4);
    }

    #[test]
    fn gate_dag_executor_custom_concurrency() {
        let ex = GateDagExecutor::new(8);
        assert_eq!(ex.concurrency_limit, 8);
    }
}
