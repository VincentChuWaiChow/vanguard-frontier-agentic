# Advisory applicability

## Problem

An advisory page states affected version ranges, but exposure needs a second condition: the vulnerable feature must be enabled and reachable. Treating every in-range cluster as exploitable wastes urgent change windows; treating "we do not think we use it" as proof leaves real exposure.

Scenario: a payments platform on 1.19.3 receives a scanner finding for several advisories at once and asks which need an emergency change and which can ride the next planned upgrade.

## Method

1. Fix the installed version exactly (x.y.z). Unknown version: every row below is `unknown`.
2. Compare it with each affected range. Out of range: not affected by that advisory.
3. In range: establish the feature precondition from the values diff or live configuration (observed), not from memory of the design.
4. In range and feature enabled: exposure is real; remediation is a normal lifecycle change to a fixed version on the same minor or the next consecutive minor, subject to the upgrade path rules.
5. In range and feature state unknown: `needs-review`.
6. Advisory fixes are usually patch releases of a maintained minor; prefer the latest patch of the current minor before a minor hop [CIL-LC-E02].

## Dated snapshot (retrieved 2026-11-25)

GHSA IDs are the primary key. CVE numbers are UNVERIFIED unless noted: the extraction tool printed some that look unusual, so recheck before citing [CIL-LC-E26]. This table is a snapshot; re-fetch the list at github.com/cilium/cilium/security/advisories before relying on it [CIL-LC-E25].

| GHSA | Published, severity | Affected | Fixed | Feature precondition |
| --- | --- | --- | --- | --- |
| GHSA-p5q6-g354-mxrg | 2026-11-03, Moderate | 1.20.0 to below 1.20.2; 1.19.0 to below 1.19.8; below 1.18.14 | 1.20.2, 1.19.8, 1.18.14 | CNP L7 header match using a secret reference (`secret.name` with `mismatch: REPLACE`): path traversal gives local file disclosure |
| GHSA-33qq-jq9c-6gcc | 2026-08-12, High | 1.19.0 to below 1.19.6; 1.18.0 to below 1.18.12; 1.14.0 to below 1.17.18 | 1.19.6, 1.18.12, 1.17.18 | Mutual authentication in use: any valid client certificate can impersonate any identity |
| GHSA-xqhm-7xhv-6ppj | 2026-08-12, High | below 1.17.18; 1.18.0 to below 1.18.12; 1.19.0 to below 1.19.6 | 1.17.18, 1.18.12, 1.19.6 | Secret sync (SDS) enabled: name collision allows cross-namespace L7 policy bypass |
| GHSA-vh48-r624-p8v7 | 2026-08-12, High | 1.18.0 to below 1.18.9; 1.17.0 to below 1.17.15; 1.16.0 to 1.16.19 | 1.18.9, 1.17.15 (no 1.16 fix listed; 1.16 is EOL) | VLAN interface plus its parent: ingress host and L7 policies bypassed |
| GHSA-fm8w-2m5w-9j7r | 2026-07-06, Moderate | 1.19.0 to 1.19.4 | 1.19.5 | Kubernetes NetworkPolicy using `ipBlock` without selectors: wrongly allows same-namespace ingress |
| GHSA-w7c2-w76w-5hmj | 2026-07-06, Moderate | below 1.17.17; 1.18.0 to below 1.18.11; 1.19.0 to below 1.19.5 | 1.17.17, 1.18.11, 1.19.5 | Gateway API HTTPRoute with traffic mirroring: Services in any namespace, bypassing ReferenceGrant |
| GHSA-q6h5-q3q6-f87x | 2026-06-16, Moderate | 1.19.0 to below 1.19.4; 1.18.2 to below 1.18.10; below 1.17.16 | 1.19.4, 1.18.10, 1.17.16 | CiliumLocalRedirectPolicy using `addressMatcher`: cross-namespace Service traffic hijack |
| GHSA-3fcv-jvfp-m4q9 | 2026-06-01, Critical | 1.19.0 to below 1.19.2; 1.18.0 to below 1.18.8; below 1.17.14 | 1.19.2, 1.18.8, 1.17.14 | L7 enabled: world-accessible Envoy admin socket, local information disclosure or cluster disruption |


Related older advisory GHSA-gj49-89wh-h4gj (High, 2026-04-22; bugtool output contained the WireGuard private key; affected below 1.17.15, 1.18.0 to 1.18.8, 1.19.0 to 1.19.2; fixed 1.19.3, 1.18.9, 1.17.15) is in [CIL-LC-E27]; bundles from affected versions are sensitive [CIL-LC-E28], and the companion skill `cilium-evidence-boundary` owns bundle handling.

Policy: supported branches get advisories and CVEs for stable features; beta-feature issues go to public issues [CIL-LC-E11].

## Worked example (scenario, derived)

Installed 1.19.3:

- GHSA-3fcv-jvfp-m4q9 (fixed 1.19.2): not in range, because 1.19.3 is above the fixed version. Not affected.
- GHSA-q6h5-q3q6-f87x (fixed 1.19.4): in range; exposure depends on any CiliumLocalRedirectPolicy using `addressMatcher`.
- GHSA-fm8w-2m5w-9j7r (1.19.0 to 1.19.4): in range; exposure depends on NetworkPolicy `ipBlock` without selectors.
- GHSA-w7c2-w76w-5hmj (fixed 1.19.5): in range; depends on HTTPRoute mirroring.
- GHSA-xqhm-7xhv-6ppj and GHSA-33qq-jq9c-6gcc (fixed 1.19.6): in range; depend on SDS and mutual auth.
- GHSA-p5q6-g354-mxrg (fixed 1.19.8): in range; depends on L7 secret references.
- GHSA-vh48-r624-p8v7: 1.19 is not listed as affected.

The feature column then decides the urgency. For the eight advisories in this snapshot, a patch to the latest 1.19.x covers every fixed line for this minor, and is a separate change from any minor hop.

## Evidence that discriminates

| Question | Evidence | Class |
| --- | --- | --- |
| Is the feature present | `kubectl get` of the relevant object kind (CiliumLocalRedirectPolicy, HTTPRoute, NetworkPolicy) across namespaces [C1] | observational |
| Is mutual auth or SDS enabled | Helm values or ConfigMap content supplied for review | observed |
| Was a bugtool or sysdump archive shared | Ticket and file-share records | observed |

## Contradictions

- Scanner says affected, installed version above the fixed version: version string may be a vendor build; ask for the distribution's backport statement. Do not call it fixed.
- Distribution claims fixed without a version bump: needs the vendor advisory mapping; `needs-review`.

## Handoff

Policy bypass consequences: cilium-network-policy-review-agent. Gateway API HTTPRoute design: kubernetes-network-architecture-review-agent. Upgrade execution: human change process.
