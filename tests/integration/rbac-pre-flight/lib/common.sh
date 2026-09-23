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
# GUARD_ERROR counts checks that produced no verdict for a reason that is NOT
# an expected property of the environment -- an unreachable API server, a
# refused impersonation, a malformed flag.  Kept apart from GUARD_SKIP so a
# broken harness is never filed as "this cluster simply lacks that CRD".
GUARD_ERROR=0

TOTAL_PASS=0
TOTAL_FAIL=0
TOTAL_SKIP=0
TOTAL_ERROR=0

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
#
#   Sets:
#     CANI_OUTPUT  kubectl's stdout (the verdict: "yes" or "no...")
#     CANI_STDERR  kubectl's stderr (warnings and errors)
#     CANI_EXIT    kubectl's exit status
#     CANI_REASON  when no verdict was obtained: 'unresolved' or 'error'
#
#   Returns 0 on a verdict, 2 when the resource could not be resolved against
#   discovery, 3 on any other failure to obtain a verdict.
#
#   Why stdout and stderr are kept apart
#   -----------------------------------
#   `kubectl auth can-i` writes the verdict to stdout and every diagnostic to
#   stderr.  Verified against kubernetes/kubernetes
#   staging/src/k8s.io/kubectl/pkg/cmd/auth/cani.go, identical at v1.28.15 and
#   v1.31.2 (both ends of this suite's CI matrix): the verdict goes to `o.Out`
#   (RunAccessCheck), while the WarningPrinter is constructed over `o.ErrOut`
#   (Complete) and carries "resource 'X' is not namespace scoped" and "the
#   server doesn't have a resource type 'X'".
#
#   This helper used to capture `2>&1`.  A cluster-scoped resource therefore
#   arrived as "Warning: resource 'clusterrolebindings' is not namespace scoped
#   in group 'rbac.authorization.k8s.io'\nno", which matches neither yes* nor
#   no*, so a perfectly good verdict was discarded as a kubectl error.  In CI
#   that silently converted 176 of 294 assertions into skips.
# ---------------------------------------------------------------------------
_run_can_i() {
  local stderr_file
  stderr_file=$(mktemp "${TMPDIR:-/tmp}/cani-stderr.XXXXXX")

  # kubectl auth can-i exits 0 for "yes" and 1 for "no"; both are verdicts.
  CANI_OUTPUT=$(kubectl auth can-i "$@" 2>"$stderr_file")
  CANI_EXIT=$?
  CANI_STDERR=$(cat "$stderr_file")
  rm -f "$stderr_file"
  CANI_REASON=""

  # An unresolved resource is checked BEFORE the verdict, because kubectl
  # still prints one and it is worthless.  resourceFor() warns and then
  # returns schema.GroupVersionResource{Resource: resourceArg} -- the whole
  # dotted argument as the resource name, with an EMPTY group.  The
  # SelfSubjectAccessReview therefore asks about
  # {group: "", resource: "schedules.velero.io"}, which no RBAC rule can ever
  # match, so kubectl answers "no" whatever the manifests grant.
  #
  # Accepting that verdict would be a fail-open: every `assert_cannot_or_skip`
  # on an uninstalled CRD would PASS vacuously, while every
  # `assert_can_or_skip` would FAIL spuriously.  Treat it as no verdict.
  case "$CANI_STDERR" in
    *"doesn't have a resource type"*|*"server could not find the requested resource"*|*"no matches for kind"*)
      CANI_REASON="unresolved"
      return 2
      ;;
  esac

  # "resource 'X' is not namespace scoped" is benign -- the GVR resolved, the
  # verdict is real, and only the -n flag was redundant.  Do not let it
  # suppress an answer.
  case "$CANI_OUTPUT" in
    yes*|no*) return 0 ;;
  esac

  # No verdict and no recognised cause.  Report kubectl's own words rather
  # than attributing it to the call site -- filing every failure as "CRD not
  # found" is how a broken cluster passes for an expected limitation.
  CANI_REASON="error"
  return 3
}

# ---------------------------------------------------------------------------
# _report_no_verdict — shared handling for a check that produced no verdict
#   Usage: _report_no_verdict <rc> <"can"|"cannot"> <description> <tolerate_unresolved>
#   Returns 0 if the caller should stop (it was counted), 1 to continue.
# ---------------------------------------------------------------------------
_report_no_verdict() {
  local rc="$1" word="$2" description="$3" tolerate="$4"

  [ "$rc" -eq 0 ] && return 1

  if [ "$rc" -eq 2 ] && [ "$tolerate" = "true" ]; then
    # Expected environment limitation: the resource is absent from discovery.
    # Still a check that never ran, so it counts as a skip and keeps the run
    # INCOMPLETE -- it is not evidence that the binding is correctly scoped.
    printf '  %b[SKIP]%b %-6s %-60s  (not in discovery: %s)\n' \
      "$_YELLOW" "$_RESET" "$word" "$description" "$CANI_STDERR"
    (( GUARD_SKIP++ )) || true
    return 0
  fi

  # Everything else is a harness or cluster problem.  Report kubectl's own
  # words rather than a guess about the cause.
  printf '  %b[ERROR]%b %-5s %-60s  (no verdict [%s]: %s)\n' \
    "$_RED" "$_RESET" "$word" "$description" "${CANI_REASON:-unknown}" "$CANI_STDERR"
  (( GUARD_ERROR++ )) || true
  return 0
}

# ---------------------------------------------------------------------------
# assert_cannot — check that an SA does NOT have a permission
#   Usage: assert_cannot <verb> <resource> [kubectl-flags...] (SA must be in flags)
# ---------------------------------------------------------------------------
assert_cannot() {
  local description="$*"
  local rc=0
  _run_can_i "$@" || rc=$?

  # Core Kubernetes resources: absence from discovery is a broken cluster,
  # not an expected limitation, so it is never tolerated here.
  _report_no_verdict "$rc" cannot "$description" false && return

  case "$CANI_OUTPUT" in
    no*)
      printf '  %b[PASS]%b cannot %s\n' "$_GREEN" "$_RESET" "$description"
      (( GUARD_PASS++ )) || true
      ;;
    *)
      printf '  %b[FAIL]%b cannot %-60s  (got: %s — binding is over-scoped)\n' \
        "$_RED" "$_RESET" "$description" "$CANI_OUTPUT"
      (( GUARD_FAIL++ )) || true
      ;;
  esac
}

# ---------------------------------------------------------------------------
# assert_can — check that an SA DOES have a permission
#   Usage: assert_can <verb> <resource> [kubectl-flags...] (SA must be in flags)
# ---------------------------------------------------------------------------
assert_can() {
  local description="$*"
  local rc=0
  _run_can_i "$@" || rc=$?

  _report_no_verdict "$rc" can "$description" false && return

  case "$CANI_OUTPUT" in
    yes*)
      printf '  %b[PASS]%b can    %s\n' "$_GREEN" "$_RESET" "$description"
      (( GUARD_PASS++ )) || true
      ;;
    *)
      printf '  %b[FAIL]%b can    %-60s  (got: %s — binding is under-scoped)\n' \
        "$_RED" "$_RESET" "$description" "$CANI_OUTPUT"
      (( GUARD_FAIL++ )) || true
      ;;
  esac
}

# ---------------------------------------------------------------------------
# assert_can_or_skip — like assert_can but tolerates a resource that is absent
#   from discovery (Gateway API / Cilium / Istio / Argo CD / Velero / Kyverno
#   CRDs that a vanilla kind cluster does not install).
#
#   Tolerated means SKIP, not PASS: the run stays INCOMPLETE.  When the CRDs
#   ARE installed the check runs for real, which is why the suite ships
#   fixtures/crds.yaml.
# ---------------------------------------------------------------------------
assert_can_or_skip() {
  local description="$*"
  local rc=0
  _run_can_i "$@" || rc=$?

  _report_no_verdict "$rc" can "$description" true && return

  case "$CANI_OUTPUT" in
    yes*)
      printf '  %b[PASS]%b can    %s\n' "$_GREEN" "$_RESET" "$description"
      (( GUARD_PASS++ )) || true
      ;;
    *)
      printf '  %b[FAIL]%b can    %-60s  (got: %s — binding is under-scoped)\n' \
        "$_RED" "$_RESET" "$description" "$CANI_OUTPUT"
      (( GUARD_FAIL++ )) || true
      ;;
  esac
}

# ---------------------------------------------------------------------------
# assert_cannot_or_skip — like assert_cannot but tolerates a resource that is
#   absent from discovery.  See assert_can_or_skip.
# ---------------------------------------------------------------------------
assert_cannot_or_skip() {
  local description="$*"
  local rc=0
  _run_can_i "$@" || rc=$?

  _report_no_verdict "$rc" cannot "$description" true && return

  case "$CANI_OUTPUT" in
    no*)
      printf '  %b[PASS]%b cannot %s\n' "$_GREEN" "$_RESET" "$description"
      (( GUARD_PASS++ )) || true
      ;;
    *)
      printf '  %b[FAIL]%b cannot %-60s  (got: %s — binding is over-scoped)\n' \
        "$_RED" "$_RESET" "$description" "$CANI_OUTPUT"
      (( GUARD_FAIL++ )) || true
      ;;
  esac
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
  assert_cannot create pods --subresource=exec -n kube-system "--as=$SA"
  assert_cannot create pods --subresource=portforward --all-namespaces "--as=$SA"
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
  assert_cannot create serviceaccounts --subresource=token --all-namespaces "--as=$SA"
  assert_cannot delete priorityclasses.scheduling.k8s.io "--as=$SA"
  assert_cannot delete ingressclasses.networking.k8s.io "--as=$SA"
  assert_cannot delete leases.coordination.k8s.io -n kube-node-lease "--as=$SA"
  assert_cannot update namespaces --subresource=finalize "--as=$SA"
}

# ---------------------------------------------------------------------------
# begin_guard — print guard header and reset per-guard counters
# ---------------------------------------------------------------------------
begin_guard() {
  local guard_name="$1"
  GUARD_PASS=0
  GUARD_FAIL=0
  GUARD_SKIP=0
  GUARD_ERROR=0
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

  # A harness or cluster problem outranks an incomplete run: an expected
  # environment limitation is survivable, a broken check is not.
  if [ "$GUARD_ERROR" -gt 0 ]; then
    status_color="$_RED"
    status_word="ERROR"
  fi

  # A real failure outranks everything: it is the only outcome that says
  # something about the manifests themselves.
  if [ "$GUARD_FAIL" -gt 0 ]; then
    status_color="$_RED"
    status_word="FAIL"
  fi

  printf '%b  %s: %d passed, %d failed, %d skipped, %d errored%b\n' \
    "$status_color" "$status_word" \
    "$GUARD_PASS" "$GUARD_FAIL" "$GUARD_SKIP" "$GUARD_ERROR" \
    "$_RESET"

  (( TOTAL_PASS += GUARD_PASS )) || true
  (( TOTAL_FAIL += GUARD_FAIL )) || true
  (( TOTAL_SKIP += GUARD_SKIP )) || true
  (( TOTAL_ERROR += GUARD_ERROR )) || true

  # Non-zero unless every check in this guard actually ran and passed.
  [ "$GUARD_FAIL" -eq 0 ] && [ "$GUARD_SKIP" -eq 0 ] && [ "$GUARD_ERROR" -eq 0 ] \
    && [ "$GUARD_PASS" -gt 0 ]
}

# ---------------------------------------------------------------------------
# report_total — print final summary across all guards
# ---------------------------------------------------------------------------
report_total() {
  printf '\n%b========================================%b\n' "$_CYAN" "$_RESET"
  if [ "$TOTAL_FAIL" -gt 0 ]; then
    printf '%bFAILURES DETECTED%b  (%d passed, %d failed, %d skipped, %d errored)\n' \
      "$_RED" "$_RESET" "$TOTAL_PASS" "$TOTAL_FAIL" "$TOTAL_SKIP" "$TOTAL_ERROR"
    printf '%b========================================%b\n' "$_CYAN" "$_RESET"
    return 1
  fi

  # Exit 3 == ERROR: checks failed to produce a verdict for a reason that is
  # not a property of this environment.  Distinct from INCOMPLETE so a broken
  # cluster is never read as "those CRDs just are not installed here".
  if [ "$TOTAL_ERROR" -gt 0 ]; then
    printf '%bHARNESS ERRORS%b  (%d passed, %d failed, %d skipped, %d errored)\n' \
      "$_RED" "$_RESET" "$TOTAL_PASS" "$TOTAL_FAIL" "$TOTAL_SKIP" "$TOTAL_ERROR"
    printf '  %d check(s) could not obtain a verdict from kubectl.  Each line above\n' "$TOTAL_ERROR"
    printf '  carries kubectl'"'"'s own stderr; this is a harness or cluster fault, not\n'
    printf '  a statement about the RBAC manifests.\n'
    printf '%b========================================%b\n' "$_CYAN" "$_RESET"
    return 3
  fi

  # Exit 2 == INCOMPLETE: distinct from both PASS and FAIL so a caller can tell
  # "the manifests are clean" from "the checks never ran".
  if [ "$TOTAL_SKIP" -gt 0 ] || [ "$TOTAL_PASS" -eq 0 ]; then
    printf '%bINCOMPLETE%b  (%d passed, %d failed, %d skipped, %d errored)\n' \
      "$_YELLOW" "$_RESET" "$TOTAL_PASS" "$TOTAL_FAIL" "$TOTAL_SKIP" "$TOTAL_ERROR"
    printf '  %d check(s) did not execute; this run proves nothing about\n' "$TOTAL_SKIP"
    printf '  privilege creep in the shipped RBAC manifests.\n'
    printf '%b========================================%b\n' "$_CYAN" "$_RESET"
    return 2
  fi

  printf '%bALL GUARDS PASSED%b  (%d passed, %d skipped, %d errored)\n' \
    "$_GREEN" "$_RESET" "$TOTAL_PASS" "$TOTAL_SKIP" "$TOTAL_ERROR"
  printf '%b========================================%b\n' "$_CYAN" "$_RESET"
  return 0
}
