#!/usr/bin/env bash
# tests/integration/rbac-pre-flight/run-all.sh
#
# Regression harness for all 7 live-guard least-privilege RBAC manifests.
# Creates a kind cluster, applies all RBAC manifests, runs every guard's
# pre-flight assertion matrix, tears down the cluster, and reports totals.
#
# Usage:
#   ./run-all.sh                         # full run (creates + destroys cluster)
#   ./run-all.sh --skip-cluster-create   # use an existing cluster
#   ./run-all.sh --guard=network-arch    # run one guard only
#   ./run-all.sh --skip-cluster-create --guard=rbac-mutation
#
# Environment variables:
#   KIND_K8S_VERSION   kind node image tag (default: v1.30.6)
#
# Exit codes:
#   0  every check ran and passed
#   1  one or more assertions failed (a manifest is over- or under-scoped)
#   2  INCOMPLETE — some checks never ran, so nothing was proven
#   3  HARNESS ERRORS — checks could not obtain a verdict from kubectl

set -euo pipefail

# ---------------------------------------------------------------------------
# Resolve paths relative to this script regardless of caller's cwd
# ---------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
LOG_FILE="/tmp/rbac-preflight-$(date +%Y%m%d-%H%M%S).log"

# ---------------------------------------------------------------------------
# Defaults
# ---------------------------------------------------------------------------
SKIP_CLUSTER_CREATE=false
ONLY_GUARD=""
CLUSTER_NAME="vanguard-rbac-test"
KIND_IMAGE="kindest/node:${KIND_K8S_VERSION:-v1.30.6}"

# ---------------------------------------------------------------------------
# Parse arguments
# ---------------------------------------------------------------------------
for arg in "$@"; do
  case "$arg" in
    --skip-cluster-create)
      SKIP_CLUSTER_CREATE=true
      ;;
    --guard=*)
      ONLY_GUARD="${arg#--guard=}"
      ;;
    --help|-h)
      grep '^#' "$0" | sed 's/^# \?//'
      exit 0
      ;;
    *)
      printf 'Unknown argument: %s\n' "$arg" >&2
      exit 1
      ;;
  esac
done

# ---------------------------------------------------------------------------
# Source the shared library
# ---------------------------------------------------------------------------
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

require_kubectl

# ---------------------------------------------------------------------------
# Cluster lifecycle
# ---------------------------------------------------------------------------
create_cluster() {
  if ! command -v kind &>/dev/null; then
    printf '%bFATAL: kind not found in PATH. Install kind >= 0.22 and retry.%b\n' \
      "$_RED" "$_RESET" >&2
    exit 1
  fi

  printf '%bCreating kind cluster %s (%s)...%b\n' \
    "$_CYAN" "$CLUSTER_NAME" "$KIND_IMAGE" "$_RESET"

  # ValidatingAdmissionPolicy went GA in Kubernetes 1.30.  On 1.28 and 1.29 it
  # is beta and, per the post-1.24 policy for beta APIs, both the feature gate
  # and the API group are OFF by default -- so `validatingadmissionpolicies` is
  # simply absent from discovery and the admission-policy guard cannot assert
  # anything about it.
  #
  # Measured, not assumed.  Against a stock kube-apiserver with RBAC,
  # /apis/admissionregistration.k8s.io/v1beta1 lists:
  #
  #   v1.28.15 default                     -> (nothing)
  #   v1.28.15 + gate + runtime-config     -> validatingadmissionpolicies, ...
  #   v1.29.10 default                     -> (nothing)
  #   v1.29.10 + gate + runtime-config     -> validatingadmissionpolicies, ...
  #
  # Turning them on keeps all four matrix legs asserting the same thing.  The
  # patch is scoped to those two versions: from 1.30 the gate is GA-locked and
  # passing it earns a deprecation warning, and eventually a fatal error.
  local extra_patches=""
  case "${KIND_K8S_VERSION:-v1.30.6}" in
    v1.28.*|v1.29.*)
      extra_patches='
kubeadmConfigPatches:
  - |
    kind: ClusterConfiguration
    apiServer:
      extraArgs:
        feature-gates: "ValidatingAdmissionPolicy=true"
        runtime-config: "admissionregistration.k8s.io/v1beta1=true"'
      printf '%b(enabling beta ValidatingAdmissionPolicy for %s)%b\n' \
        "$_YELLOW" "${KIND_K8S_VERSION}" "$_RESET"
      ;;
  esac

  kind create cluster \
    --name "$CLUSTER_NAME" \
    --image "$KIND_IMAGE" \
    --config - <<EOF
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
  - role: control-plane${extra_patches}
EOF

  printf '%bCluster ready.%b\n' "$_GREEN" "$_RESET"
}

delete_cluster() {
  if command -v kind &>/dev/null; then
    printf '\n%bDeleting kind cluster %s...%b\n' "$_CYAN" "$CLUSTER_NAME" "$_RESET"
    kind delete cluster --name "$CLUSTER_NAME" 2>/dev/null || true
  fi
}

# ---------------------------------------------------------------------------
# Cleanup trap — always remove the cluster on exit unless skipped
# ---------------------------------------------------------------------------
cleanup() {
  local exit_code=$?
  if [ "$SKIP_CLUSTER_CREATE" = "false" ]; then
    delete_cluster
  fi
  if [ -f "$LOG_FILE" ]; then
    printf '\nTest output saved to: %s\n' "$LOG_FILE"
  fi
  exit $exit_code
}
trap cleanup EXIT

# ---------------------------------------------------------------------------
# Apply RBAC manifests
# ---------------------------------------------------------------------------
apply_crd_stubs() {
  printf '\n%bInstalling stub CRDs...%b\n' "$_CYAN" "$_RESET"

  # `kubectl auth can-i` resolves its resource argument through discovery
  # first.  Without these, it asks the API server about {group: "", resource:
  # "schedules.velero.io"} and answers "no" regardless of what the manifests
  # grant -- measured against kube-apiserver v1.30.6.  See fixtures/crds.yaml.
  kubectl apply -f "$SCRIPT_DIR/fixtures/crds.yaml"

  # Discovery is cached and refreshed asynchronously; give the aggregated
  # discovery document a chance to pick the new groups up before asserting.
  kubectl wait --for=condition=Established --timeout=60s \
    -f "$SCRIPT_DIR/fixtures/crds.yaml"

  printf '%bStub CRDs established.%b\n' "$_GREEN" "$_RESET"
}

apply_rbac_manifests() {
  printf '\n%bApplying RBAC manifests...%b\n' "$_CYAN" "$_RESET"

  # Ensure the namespaces the guards impersonate into exist.
  for ns in vanguard-system velero argocd istio-system kyverno; do
    kubectl create namespace "$ns" --dry-run=client -o yaml | kubectl apply -f -
  done

  local manifests=(
    "skills/kubernetes/kubernetes-live-network-architecture-mutation-guard/references/least-privilege-rbac.yaml"
    "agents/kubernetes/kubernetes-live-network-policy-guard-agent/references/least-privilege-rbac.yaml"
    "agents/kubernetes/kubernetes-live-mesh-policy-guard-agent/references/least-privilege-rbac.yaml"
    "agents/kubernetes/kubernetes-live-admission-policy-guard-agent/references/least-privilege-rbac.yaml"
    "agents/kubernetes/kubernetes-live-argocd-sync-guard-agent/references/least-privilege-rbac.yaml"
    "agents/kubernetes/kubernetes-live-rbac-mutation-guard-agent/references/least-privilege-rbac.yaml"
    "agents/kubernetes/kubernetes-live-velero-restore-guard-agent/references/least-privilege-rbac.yaml"
  )

  for manifest in "${manifests[@]}"; do
    local full_path="$REPO_ROOT/$manifest"
    if [ ! -f "$full_path" ]; then
      printf '%bWARN: manifest not found, skipping: %s%b\n' \
        "$_YELLOW" "$full_path" "$_RESET" >&2
      continue
    fi
    printf '  Applying %s\n' "$manifest"
    kubectl apply -f "$full_path"
  done

  printf '%bManifests applied.%b\n' "$_GREEN" "$_RESET"
}

# ---------------------------------------------------------------------------
# Source guard files and run them
# ---------------------------------------------------------------------------
# shellcheck source=guards/network-arch.sh
source "$SCRIPT_DIR/guards/network-arch.sh"
# shellcheck source=guards/network-policy.sh
source "$SCRIPT_DIR/guards/network-policy.sh"
# shellcheck source=guards/mesh-policy.sh
source "$SCRIPT_DIR/guards/mesh-policy.sh"
# shellcheck source=guards/admission-policy.sh
source "$SCRIPT_DIR/guards/admission-policy.sh"
# shellcheck source=guards/argocd-sync.sh
source "$SCRIPT_DIR/guards/argocd-sync.sh"
# shellcheck source=guards/rbac-mutation.sh
source "$SCRIPT_DIR/guards/rbac-mutation.sh"
# shellcheck source=guards/velero-restore.sh
source "$SCRIPT_DIR/guards/velero-restore.sh"

run_guards() {
  # Map of guard names to their runner functions
  declare -A GUARD_FN=(
    [network-arch]=run_guard_network_arch
    [network-policy]=run_guard_network_policy
    [mesh-policy]=run_guard_mesh_policy
    [admission-policy]=run_guard_admission_policy
    [argocd-sync]=run_guard_argocd_sync
    [rbac-mutation]=run_guard_rbac_mutation
    [velero-restore]=run_guard_velero_restore
  )

  # Ordered list (bash 3 compat — no associative array ordering guarantee)
  local GUARD_ORDER=(
    network-arch
    network-policy
    mesh-policy
    admission-policy
    argocd-sync
    rbac-mutation
    velero-restore
  )

  if [ -n "$ONLY_GUARD" ]; then
    if [ -z "${GUARD_FN[$ONLY_GUARD]+_}" ]; then
      printf '%bUnknown guard: %s%b\n' "$_RED" "$ONLY_GUARD" "$_RESET" >&2
      printf 'Available guards: %s\n' "${GUARD_ORDER[*]}" >&2
      exit 1
    fi
    "${GUARD_FN[$ONLY_GUARD]}"
  else
    for guard in "${GUARD_ORDER[@]}"; do
      "${GUARD_FN[$guard]}" || true  # failures tracked in counters; don't abort
    done
  fi
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
{
  printf '%b=== Vanguard RBAC Pre-flight Integration Test ===%b\n' "$_CYAN" "$_RESET"
  printf 'Cluster: %s | Image: %s\n' "$CLUSTER_NAME" "$KIND_IMAGE"
  printf 'Log: %s\n' "$LOG_FILE"

  if [ "$SKIP_CLUSTER_CREATE" = "false" ]; then
    create_cluster
  else
    printf '%b(--skip-cluster-create: using current kubeconfig context)%b\n' \
      "$_YELLOW" "$_RESET"
  fi

  apply_crd_stubs
  apply_rbac_manifests
  run_guards

  # Propagate report_total's verdict verbatim so callers can distinguish the
  # three states: 0 = every check ran and passed, 1 = real failures,
  # 2 = INCOMPLETE (checks were skipped, so nothing was proven).
  rc=0
  report_total || rc=$?
  exit "$rc"
} 2>&1 | tee "$LOG_FILE"
