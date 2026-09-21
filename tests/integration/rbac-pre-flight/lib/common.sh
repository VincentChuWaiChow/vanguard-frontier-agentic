#!/usr/bin/env bash
# lib/common.sh — shared helpers for RBAC pre-flight integration tests.
# Guard scripts source this file; do NOT set -e here since denied checks
# intentionally return exit code 1 from kubectl.

# ---------------------------------------------------------------------------
# Color output (suppressed when not a TTY)
# ---------------------------------------------------------------------------
if [ -t 1 ]; then
  _GREEN='\033[0;32m'
  _RED='\033[0;31m'
  _YELLOW='\033[0;33m'
  _CYAN='\033[0;36m'
  _RESET='\033[0m'
else
  _GREEN=''
  _RED=''
  _YELLOW=''
  _CYAN=''
  _RESET=''
fi

# ---------------------------------------------------------------------------
# Global counters (reset per guard via report_guard)
# ---------------------------------------------------------------------------
GUARD_PASS=0
GUARD_FAIL=0
GUARD_SKIP=0

TOTAL_PASS=0
TOTAL_FAIL=0
TOTAL_SKIP=0

# ---------------------------------------------------------------------------
# require_kubectl — abort early if kubectl is not on PATH
# ---------------------------------------------------------------------------
require_kubectl() {
  if ! command -v kubectl &>/dev/null; then
    printf '%bFATAL: kubectl not found in PATH. Install kubectl >= 1.28 and retry.%b\n' \
      "$_RED" "$_RESET" >&2
    exit 1
  fi
}

# ---------------------------------------------------------------------------
# _run_can_i — internal helper
#   Usage: _run_can_i <verb> <resource> [extra kubectl flags...] --as=<SA>
#   Returns the raw output of kubectl auth can-i (yes/no) in CANI_OUTPUT.
#   Returns 0 on success, non-zero on kubectl error (distinct from denied).
# ---------------------------------------------------------------------------
_run_can_i() {
  # kubectl auth can-i exits 0 for "yes" and 1 for "no".
  # We capture output regardless of exit code.
  CANI_OUTPUT=$(kubectl auth can-i "$@" 2>&1)
  CANI_EXIT=$?
  # Propagate real errors (not the normal denied exit) to the caller.
  # kubectl prints "yes\n" or "no\n"; anything else is an error.
  case "$CANI_OUTPUT" in
    yes*|no*) return 0 ;;
    *) return 2 ;;   # unexpected output / server error
  esac
}

# ---------------------------------------------------------------------------
# assert_cannot — check that an SA does NOT have a permission
#   Usage: assert_cannot <verb> <resource> [kubectl-flags...] (SA must be in flags)
# ---------------------------------------------------------------------------
assert_cannot() {
  local description="$*"
  _run_can_i "$@"
  local rc=$?

  if [ $rc -eq 2 ]; then
    printf '  %b[SKIP]%b cannot %-60s  (kubectl error: %s)\n' \
      "$_YELLOW" "$_RESET" "$description" "$CANI_OUTPUT"
    (( GUARD_SKIP++ )) || true
    return
  fi

  if [ "$CANI_OUTPUT" = "no" ]; then
    printf '  %b[PASS]%b cannot %s\n' "$_GREEN" "$_RESET" "$description"
    (( GUARD_PASS++ )) || true
  else
    printf '  %b[FAIL]%b cannot %-60s  (got: %s — binding is over-scoped)\n' \
      "$_RED" "$_RESET" "$description" "$CANI_OUTPUT"
    (( GUARD_FAIL++ )) || true
  fi
}

# ---------------------------------------------------------------------------
# assert_can — check that an SA DOES have a permission
#   Usage: assert_can <verb> <resource> [kubectl-flags...] (SA must be in flags)
# ---------------------------------------------------------------------------
assert_can() {
  local description="$*"
  _run_can_i "$@"
  local rc=$?

  if [ $rc -eq 2 ]; then
    printf '  %b[SKIP]%b can    %-60s  (kubectl error: %s)\n' \
      "$_YELLOW" "$_RESET" "$description" "$CANI_OUTPUT"
    (( GUARD_SKIP++ )) || true
    return
  fi

  if [ "$CANI_OUTPUT" = "yes" ]; then
    printf '  %b[PASS]%b can    %s\n' "$_GREEN" "$_RESET" "$description"
    (( GUARD_PASS++ )) || true
  else
    printf '  %b[FAIL]%b can    %-60s  (got: %s — binding is under-scoped)\n' \
      "$_RED" "$_RESET" "$description" "$CANI_OUTPUT"
    (( GUARD_FAIL++ )) || true
  fi
}

# ---------------------------------------------------------------------------
# assert_can_or_skip — like assert_can but treats "NotFound" CRD errors as SKIP
#   Use for Gateway API / Cilium / Istio / Argo CD / Velero / Kyverno CRDs that
#   may not exist in a vanilla kind cluster.
# ---------------------------------------------------------------------------
assert_can_or_skip() {
  local description="$*"
  _run_can_i "$@"
  local rc=$?

  if [ $rc -eq 2 ]; then
    # CRD not installed — skip rather than fail
    printf '  %b[SKIP]%b can    %-60s  (CRD not found — install CRDs to test)\n' \
      "$_YELLOW" "$_RESET" "$description"
    (( GUARD_SKIP++ )) || true
    return
  fi

  if [ "$CANI_OUTPUT" = "yes" ]; then
    printf '  %b[PASS]%b can    %s\n' "$_GREEN" "$_RESET" "$description"
    (( GUARD_PASS++ )) || true
  else
    printf '  %b[FAIL]%b can    %-60s  (got: %s — binding is under-scoped)\n' \
      "$_RED" "$_RESET" "$description" "$CANI_OUTPUT"
    (( GUARD_FAIL++ )) || true
  fi
}

# ---------------------------------------------------------------------------
# assert_cannot_or_skip — like assert_cannot but treats "NotFound" as SKIP
#   Use for domain-specific must-not checks on CRDs that may not be installed.
# ---------------------------------------------------------------------------
assert_cannot_or_skip() {
  local description="$*"
  _run_can_i "$@"
  local rc=$?

  if [ $rc -eq 2 ]; then
    printf '  %b[SKIP]%b cannot %-60s  (CRD not found — install CRDs to test)\n' \
      "$_YELLOW" "$_RESET" "$description"
    (( GUARD_SKIP++ )) || true
    return
  fi

  if [ "$CANI_OUTPUT" = "no" ]; then
    printf '  %b[PASS]%b cannot %s\n' "$_GREEN" "$_RESET" "$description"
    (( GUARD_PASS++ )) || true
  else
    printf '  %b[FAIL]%b cannot %-60s  (got: %s — binding is over-scoped)\n' \
      "$_RED" "$_RESET" "$description" "$CANI_OUTPUT"
    (( GUARD_FAIL++ )) || true
  fi
}

# ---------------------------------------------------------------------------
# run_universal_must_not — runs the 21 universal must-not-be-yes checks
#   Usage: run_universal_must_not <SA>
# ---------------------------------------------------------------------------
run_universal_must_not() {
  local SA="$1"
  printf '%b  -- universal must-not-be-yes --%b\n' "$_CYAN" "$_RESET"

  assert_cannot '*' '*' --all-namespaces "--as=$SA"
  assert_cannot delete namespaces "--as=$SA"
  assert_cannot delete pods -n kube-system "--as=$SA"
  assert_cannot create pods/exec -n kube-system "--as=$SA"
  assert_cannot create pods/portforward --all-namespaces "--as=$SA"
  assert_cannot delete daemonsets -n kube-system "--as=$SA"
  assert_cannot delete deployments -n kube-system "--as=$SA"
  assert_cannot create customresourcedefinitions "--as=$SA"
  assert_cannot delete customresourcedefinitions "--as=$SA"
  assert_cannot get secrets --all-namespaces "--as=$SA"
  assert_cannot create clusterrolebindings "--as=$SA"
  assert_cannot create mutatingwebhookconfigurations.admissionregistration.k8s.io "--as=$SA"
  assert_cannot delete mutatingwebhookconfigurations.admissionregistration.k8s.io "--as=$SA"
  assert_cannot create validatingwebhookconfigurations.admissionregistration.k8s.io "--as=$SA"
  assert_cannot create apiservices.apiregistration.k8s.io "--as=$SA"
  assert_cannot update certificatesigningrequests.certificates.k8s.io --subresource=approval "--as=$SA"
  assert_cannot create serviceaccounts/token --all-namespaces "--as=$SA"
  assert_cannot delete priorityclasses.scheduling.k8s.io "--as=$SA"
  assert_cannot delete ingressclasses.networking.k8s.io "--as=$SA"
  assert_cannot delete leases.coordination.k8s.io -n kube-node-lease "--as=$SA"
  assert_cannot update namespaces/finalize "--as=$SA"
}

# ---------------------------------------------------------------------------
# begin_guard — print guard header and reset per-guard counters
# ---------------------------------------------------------------------------
begin_guard() {
  local guard_name="$1"
  GUARD_PASS=0
  GUARD_FAIL=0
  GUARD_SKIP=0
  printf '\n%b=== Guard: %s ===%b\n' "$_CYAN" "$guard_name" "$_RESET"
}

# ---------------------------------------------------------------------------
# report_guard — print per-guard summary and accumulate totals
#   Returns 1 if this guard had any failures (for run-all.sh to track).
# ---------------------------------------------------------------------------
report_guard() {
  local guard_name="$1"
  local status_color="$_GREEN"
  local status_word="PASS"

  # A skipped check is an assertion that was never made.  This harness is the
  # privilege-creep regression gate for the least-privilege RBAC manifests that
  # ship with the Kubernetes live-guard agents, so "could not check" must never
  # render as "PASS" — an unreachable cluster or a failing impersonation would
  # otherwise report a clean bill of health over zero executed checks.
  if [ "$GUARD_SKIP" -gt 0 ] || [ "$GUARD_PASS" -eq 0 ]; then
    status_color="$_YELLOW"
    status_word="INCOMPLETE"
  fi

  # A real failure outranks an incomplete run.
  if [ "$GUARD_FAIL" -gt 0 ]; then
    status_color="$_RED"
    status_word="FAIL"
  fi

  printf '%b  %s: %d passed, %d failed, %d skipped%b\n' \
    "$status_color" "$status_word" \
    "$GUARD_PASS" "$GUARD_FAIL" "$GUARD_SKIP" \
    "$_RESET"

  (( TOTAL_PASS += GUARD_PASS )) || true
  (( TOTAL_FAIL += GUARD_FAIL )) || true
  (( TOTAL_SKIP += GUARD_SKIP )) || true

  # Non-zero unless every check in this guard actually ran and passed.
  [ "$GUARD_FAIL" -eq 0 ] && [ "$GUARD_SKIP" -eq 0 ] && [ "$GUARD_PASS" -gt 0 ]
}

# ---------------------------------------------------------------------------
# report_total — print final summary across all guards
# ---------------------------------------------------------------------------
report_total() {
  printf '\n%b========================================%b\n' "$_CYAN" "$_RESET"
  if [ "$TOTAL_FAIL" -gt 0 ]; then
    printf '%bFAILURES DETECTED%b  (%d passed, %d failed, %d skipped)\n' \
      "$_RED" "$_RESET" "$TOTAL_PASS" "$TOTAL_FAIL" "$TOTAL_SKIP"
    printf '%b========================================%b\n' "$_CYAN" "$_RESET"
    return 1
  fi

  # Exit 2 == INCOMPLETE: distinct from both PASS and FAIL so a caller can tell
  # "the manifests are clean" from "the checks never ran".
  if [ "$TOTAL_SKIP" -gt 0 ] || [ "$TOTAL_PASS" -eq 0 ]; then
    printf '%bINCOMPLETE%b  (%d passed, %d failed, %d skipped)\n' \
      "$_YELLOW" "$_RESET" "$TOTAL_PASS" "$TOTAL_FAIL" "$TOTAL_SKIP"
    printf '  %d check(s) did not execute; this run proves nothing about\n' "$TOTAL_SKIP"
    printf '  privilege creep in the shipped RBAC manifests.\n'
    printf '%b========================================%b\n' "$_CYAN" "$_RESET"
    return 2
  fi

  printf '%bALL GUARDS PASSED%b  (%d passed, %d skipped)\n' \
    "$_GREEN" "$_RESET" "$TOTAL_PASS" "$TOTAL_SKIP"
  printf '%b========================================%b\n' "$_CYAN" "$_RESET"
  return 0
}
