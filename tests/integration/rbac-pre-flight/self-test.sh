#!/usr/bin/env bash
# tests/integration/rbac-pre-flight/self-test.sh
#
# Tests the harness itself, with no cluster.
#
# The guards assert things about RBAC manifests; nothing asserted anything
# about the guards.  That gap shipped a fail-open: `_run_can_i` captured
# `kubectl auth can-i` with `2>&1`, so any warning on stderr arrived glued to
# the front of the verdict on stdout, matched neither `yes*` nor `no*`, and was
# filed as a kubectl error.  In CI that turned 176 of 294 assertions into skips
# while the log claimed the CRDs merely were not installed.
#
# Every expectation below is grounded in kubernetes/kubernetes
# staging/src/k8s.io/kubectl/pkg/cmd/auth/cani.go, which is identical at
# v1.28.15 and v1.31.2 (both ends of this suite's CI matrix):
#
#   * the verdict is written to o.Out   (RunAccessCheck)
#   * every warning to o.ErrOut         (Complete, via WarningPrinter)
#   * resourceFor() warns about an unresolvable resource and then returns
#     GroupVersionResource{Resource: resourceArg} -- the full dotted argument
#     as the resource, with an empty group -- so kubectl answers "no" to a
#     question no RBAC rule can match.
#
# A stub `kubectl` on PATH replays those exact shapes.  Deterministic: no
# cluster, no network, no clock.
#
# Usage: ./self-test.sh        (exit 0 = harness behaves correctly)

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STUB_DIR="$(mktemp -d "${TMPDIR:-/tmp}/rbac-selftest.XXXXXX")"
trap 'rm -rf "$STUB_DIR"' EXIT

# ---------------------------------------------------------------------------
# Stub kubectl: replays CANI_STUB_OUT on stdout, CANI_STUB_ERR on stderr,
# and exits CANI_STUB_RC.
# ---------------------------------------------------------------------------
# Order matters: kubectl emits warnings during Complete()/Validate() and the
# verdict later, in RunAccessCheck().  Under the old `2>&1` capture the warning
# therefore landed IN FRONT of the verdict, which is precisely why `no*` stopped
# matching.  A stub that printed stdout first would hide the bug it is here to
# catch, so stderr is written first.
cat > "$STUB_DIR/kubectl" <<'STUB'
#!/usr/bin/env bash
[ -n "${CANI_STUB_ERR:-}" ] && printf '%s\n' "$CANI_STUB_ERR" >&2
[ -n "${CANI_STUB_OUT:-}" ] && printf '%s\n' "$CANI_STUB_OUT"
exit "${CANI_STUB_RC:-0}"
STUB
chmod +x "$STUB_DIR/kubectl"
PATH="$STUB_DIR:$PATH"

# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

SELFTEST_FAIL=0

# ---------------------------------------------------------------------------
# expect — run one assertion against stubbed kubectl output and check which
#          counter it landed in.
#   Usage: expect <label> <stdout> <stderr> <rc> <assert_fn> <want-counter>
# ---------------------------------------------------------------------------
expect() {
  local label="$1" out="$2" err="$3" rc="$4" fn="$5" want="$6"

  GUARD_PASS=0; GUARD_FAIL=0; GUARD_SKIP=0; GUARD_ERROR=0
  CANI_STUB_OUT="$out" CANI_STUB_ERR="$err" CANI_STUB_RC="$rc" \
    "$fn" get widgets --as=probe >/dev/null 2>&1

  local got=""
  [ "$GUARD_PASS"  -eq 1 ] && got="PASS"
  [ "$GUARD_FAIL"  -eq 1 ] && got="FAIL"
  [ "$GUARD_SKIP"  -eq 1 ] && got="SKIP"
  [ "$GUARD_ERROR" -eq 1 ] && got="ERROR"

  if [ "$got" = "$want" ]; then
    printf '  ok   %-58s -> %s\n' "$label" "$got"
  else
    printf '  FAIL %-58s -> %s (wanted %s)\n' "$label" "${got:-none}" "$want"
    SELFTEST_FAIL=$((SELFTEST_FAIL + 1))
  fi
}

# These are kubectl's real strings, not paraphrases.
WARN_SCOPE="Warning: resource 'clusterrolebindings' is not namespace scoped in group 'rbac.authorization.k8s.io'"
WARN_UNRESOLVED="Warning: the server doesn't have a resource type 'schedules' in group 'velero.io'"
ERR_REFUSED="The connection to the server localhost:8080 was refused - did you specify the right host or port?"

printf '=== verdicts on a clean stderr ===\n'
expect 'cannot / no'                    'no'  ''  1 assert_cannot         PASS
expect 'cannot / yes (over-scoped)'     'yes' ''  0 assert_cannot         FAIL
expect 'can / yes'                      'yes' ''  0 assert_can            PASS
expect 'can / no (under-scoped)'        'no'  ''  1 assert_can            FAIL
expect 'cannot / no with RBAC reason'   'no - RBAC: denied' '' 1 assert_cannot PASS

printf '\n=== the regression: a benign warning must not eat the verdict ===\n'
expect 'cannot / no + not-namespace-scoped'  'no'  "$WARN_SCOPE" 1 assert_cannot PASS
expect 'cannot / yes + not-namespace-scoped' 'yes' "$WARN_SCOPE" 0 assert_cannot FAIL
expect 'can / yes + not-namespace-scoped'    'yes' "$WARN_SCOPE" 0 assert_can    PASS

printf '\n=== an unresolved resource: the verdict is worthless, not a result ===\n'
# kubectl answers "no" here, against {group:"", resource:"schedules.velero.io"}.
# Accepting it would pass every cannot-check vacuously and fail every can-check.
expect 'cannot_or_skip / unresolved'    'no'  "$WARN_UNRESOLVED" 1 assert_cannot_or_skip SKIP
expect 'can_or_skip / unresolved'       'no'  "$WARN_UNRESOLVED" 1 assert_can_or_skip    SKIP
# A core resource missing from discovery is a broken cluster, not a limitation.
expect 'cannot / unresolved is an error' 'no' "$WARN_UNRESOLVED" 1 assert_cannot         ERROR
expect 'can / unresolved is an error'    'no' "$WARN_UNRESOLVED" 1 assert_can            ERROR

printf '\n=== infrastructure faults are never filed as a missing CRD ===\n'
expect 'cannot / connection refused'       '' "$ERR_REFUSED" 1 assert_cannot         ERROR
expect 'can_or_skip / connection refused'  '' "$ERR_REFUSED" 1 assert_can_or_skip    ERROR
expect 'cannot_or_skip / connection refused' '' "$ERR_REFUSED" 1 assert_cannot_or_skip ERROR
expect 'can / empty output, empty stderr'  '' ''             1 assert_can            ERROR

# ---------------------------------------------------------------------------
# report_total exit codes
# ---------------------------------------------------------------------------
printf '\n=== report_total exit codes ===\n'
check_total() {
  local label="$1" want="$2"
  local rc=0
  report_total >/dev/null 2>&1 || rc=$?
  if [ "$rc" -eq "$want" ]; then
    printf '  ok   %-58s -> %d\n' "$label" "$rc"
  else
    printf '  FAIL %-58s -> %d (wanted %d)\n' "$label" "$rc" "$want"
    SELFTEST_FAIL=$((SELFTEST_FAIL + 1))
  fi
}

TOTAL_PASS=10 TOTAL_FAIL=0 TOTAL_SKIP=0 TOTAL_ERROR=0; check_total 'all ran and passed'       0
TOTAL_PASS=10 TOTAL_FAIL=1 TOTAL_SKIP=0 TOTAL_ERROR=0; check_total 'a real failure'           1
TOTAL_PASS=10 TOTAL_FAIL=0 TOTAL_SKIP=0 TOTAL_ERROR=1; check_total 'a harness error'          3
TOTAL_PASS=10 TOTAL_FAIL=0 TOTAL_SKIP=1 TOTAL_ERROR=0; check_total 'a skip'                   2
TOTAL_PASS=0  TOTAL_FAIL=0 TOTAL_SKIP=0 TOTAL_ERROR=0; check_total 'zero checks is not a pass' 2
TOTAL_PASS=10 TOTAL_FAIL=1 TOTAL_SKIP=1 TOTAL_ERROR=1; check_total 'failure outranks the rest' 1

# ---------------------------------------------------------------------------
# Subresource syntax lint
#
# `kubectl auth can-i <verb> pods/exec` does NOT ask about the exec
# subresource.  It parses `pods/exec` as resource=pods, name=exec, and the
# SelfSubjectAccessReview goes out as
#   {"verb":"create","resource":"pods","name":"exec"}
# -- verified by inspecting the request body against a live apiserver.
#
# So every must-not check written that way was vacuous: measured on
# kube-apiserver v1.30.6, a service account explicitly granted `pods/exec`
# create still answered "no" to `can-i create pods/exec`, and "yes" only to
# `can-i create pods --subresource=exec`.  Nine assertions across the guards
# were in that state, silently unable to detect the escalation they named.
#
# `resource/name` is a legitimate and intentional form here -- it is how the
# resourceName tests pin a named object (configmaps/coredns,
# namespaces/kube-system).  Only the names that are real subresources are
# flagged.
# ---------------------------------------------------------------------------
printf '\n=== subresource syntax lint ===\n'

# Real subresources, taken from the discovery document of a live apiserver
# rather than from memory.
SUBRESOURCES='proxy|status|exec|log|attach|portforward|binding|eviction|ephemeralcontainers|token|finalize|scale|approval|resize'

LINT_HITS=$(grep -rnE "assert_(can|cannot)(_or_skip)? +[a-z*]+ +[a-z]+/($SUBRESOURCES)\b" \
  "$SCRIPT_DIR/guards" "$SCRIPT_DIR/lib" 2>/dev/null || true)

if [ -n "$LINT_HITS" ]; then
  printf '  FAIL resource/subresource written as a name; use --subresource=\n'
  printf '%s\n' "$LINT_HITS" | sed 's/^/       /'
  SELFTEST_FAIL=$((SELFTEST_FAIL + 1))
else
  printf '  ok   no assertion writes a subresource as a resource name\n'
fi

printf '\n'
if [ "$SELFTEST_FAIL" -gt 0 ]; then
  printf 'HARNESS SELF-TEST FAILED (%d case(s))\n' "$SELFTEST_FAIL" >&2
  exit 1
fi
printf 'harness self-test: all cases behaved correctly\n'
