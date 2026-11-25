# Red Team and Release Readiness

> Status: **BUILT — design record.** Previous: [05-value-model-and-kpis.md](./05-value-model-and-kpis.md)

## 1. What was actually verified

| Check | Result | What it does not prove |
| --- | --- | --- |
| `npm run validate` (every `validate:*` gate, including the new `validate:cilium-board`) | pass | that agents answer well |
| `npm run lint:spell`, markdownlint over every Markdown file | pass | — |
| `cargo test` in `tools/vfa-tui` against the new catalog (no Rust change) | pass | — |
| Routing fixture, 28 scenarios graded by `evaluate()` | pass | that a model follows the routing table; the grader is a keyword router |
| Negative probes: unclassified command, `bash` fence, mutating tier, widened Claude Code tools, stale adapter | each failed with its rule ID (CB01, CB03, CB02, CB06, CB08, CB10) | — |
| Negative probes: dangling ledger citation, off-allowlist source | each failed in `validate:skill-references` | that VERIFIED rows are behaviorally true |
| Primary-source verification | three Sonnet research passes; load-bearing claims re-verified by the orchestrator through Context7 (`/cilium/cilium` 1.20.4) | ledger rows are documentation checks, not tests on a cluster |
| Independent refuter pass over every reference | 7 major and 5 minor finding groups, no critical; dispositions in §3 | a second refuter pass was not run after the fixes |

**Not run, and not claimed:** no behavioral evaluation of any agent answering a real or synthetic Cilium case; no live cluster; no measurement of any KPI. The brief's requirement of "zero observed unauthorized mutation attempts in the release-gating corpus" is met only structurally (no write tools, no execution authority, live guards never auto-dispatched), not by observing model behavior.

## 2. Stakeholder challenge

| Reviewer | Challenge | Finding | Resolution | Residual |
| --- | --- | --- | --- | --- |
| CTO | Is a five-agent board overengineering? | Remove half and what is lost? Removing the maestro loses only routing on ambiguous symptoms; removing any specialist loses an unowned decision (R2's BGP gap, lifecycle) | Kept four specialists; maestro conditional on measured value ([02 §5](./02-opportunity-scoring.md)) | Maestro value unmeasured |
| CISO | New trust-boundary risk? | Board holds no write tools; Kiro adapters are prompt-only | Stated in every contract | Kiro residual (R-04) |
| SRE director | Will it shorten incidents? | Only if users supply comparison cases and drop counters | Input contracts demand them; `needs-review` otherwise | Unmeasured (R-01) |
| Principal network engineer | Are mechanisms right? | Refuter found overstated and unsupported claims | Fixed or relabeled (§3) | Ledgers are point-in-time |
| Platform engineering director | Can it be maintained? | Ledgers go stale each minor | Maintenance rule in the skills README | Depends on discipline (R-05) |
| FinOps | Does value justify cost? | Not provable yet | Value model with disconfirming signals ([05 §6](./05-value-model-and-kpis.md)) | — |
| Compliance reviewer | Auditable evidence? | `approved` could be misread as certification | Defined as bounded, never certification; control lineage delegated | Readers can still misquote |
| Repository maintainer | Coherence? | Reused envelope, verdicts, ledger gate, renderer, grader; no new schema | — | Istio adapters still differ in Codex sandbox handling (R-06) |
| Adversarial agentic architect | How does orchestration fail? | Keyword grader cannot express "first owner, then X"; injected instructions route to a gate | Recorded as fixture tags; gate mode is fail-safe | R-02 |

## 3. Refuter findings and dispositions

| # | Finding | Disposition |
| --- | --- | --- |
| 1 | Policy agent declared `read-only-runtime` and live reads through an authorized tool, contradicting the shared skill; the rendered adapters grant only read tools anyway | **Accepted, orchestrator's error.** Board is now uniformly `static-review`; CB06 enforces it. See the correction in [04 §1](./04-threat-model-and-governance.md) |
| 2 | `cilium clustermesh status` cited without a source | Accepted; removed and replaced with an unverified placeholder |
| 3 | `cilium-dbg bgp` recommended without its 1.20 deprecation | Accepted; caveat and ledger row added |
| 4a | "A full Hubble event queue does not impact traffic" called unsupported | **Rejected.** The orchestrator verified the sentence in the 1.20.4 tuning documentation via Context7; it was missing only from the delegate fact sheet |
| 4b-d | Aggregation effect, L7 visibility, cardinality stated as fact | Accepted; restated as the tuning guidance's wording, derived, and hypothesis respectively |
| 5 | Ledger rows VERIFIED where the fact sheet said unverified | Accepted; four rows changed to UNVERIFIED |
| 6 | Misclassified commands (resource deletion as C4, failure drills as C4, in-pod `hubble observe` as C1, `helm get values` as C1) | Accepted; reclassified |
| 7 | Ownership loops (egress-gateway reply encryption; kernel feature choice; a stray Tetragon item) | Accepted; reply-path encryption is policy-owned coverage, kernel feature choice goes to the Kubernetes architecture agent, Tetragon item removed |
| 8 | Pre-existing policy-reference claims not in any source (gRPC path matcher, wildcard and empty-selector semantics, reload ordering, Kafka without the 1.20 removal) | Accepted; the gRPC claim was removed, the rest relabeled as unverified against the installed version's policy language page |
| 9-11 | Claims stated more strongly than the sources | Accepted; labeled derived or reworded to the source |
| 12 | Duplicated paragraphs across skills | Partly accepted: the key-leak advisory, absence-of-flow rule and Istio socket-LB scenario now have one owner each; short `policy-default-local-cluster` mentions in the lifecycle and boundary skills are kept as one-line handoffs |

**Correction to the record during this work:** two arithmetic errors in the first scorecard draft (candidates C2 and C4) were found by a recomputation script and fixed before commit; the scores in [02](./02-opportunity-scoring.md) are recomputed values.

## 4. Brief §17 scenario coverage

Coverage below is contract-level (the rule exists in the named file) unless a fixture is named.

| Scenario | Where it is handled |
| --- | --- |
| Default-deny would interrupt DNS | policy `default-deny-rollout.md`; policy agent adversarial checks |
| Hubble shows no malicious flows | evidence-boundary `flow-telemetry.md` scenario |
| BGP sessions healthy, service unreachable | external `bgp-control-plane.md`; fixture `003-happy-external-connectivity-review` |
| Latency after upgrade | dataplane hypothesis tree 5; fixture `multi-02-broke-after-upgrade` |
| Version unknown | every agent's failure handling |
| ClusterMesh failure affects a global service | external `clustermesh-resilience.md`; fixture `010-happy-clustermesh-dr` |
| IPAM pool near exhaustion | **Gap.** Pod CIDR sizing belongs to `kubernetes-network-architecture-review-agent`; the lifecycle skill covers which IPAM changes are safe in place. No board reference computes pool headroom (R-07) |
| Encryption is enabled | policy `encryption-coverage.md`; fixture `005-happy-encryption-coverage` |
| Multiple Gateway API controllers | out of board: Kubernetes architecture agent |
| Context7 returns conflicting documentation | ledger CONFLICT rows (WireGuard MTU constant versus comment); lifecycle and boundary fact-sheet conflicts recorded |
| Jira contains embedded instructions | handoff rule 4; fixture `adv-jira-injection` (routes to a gate, never to execution) |
| Immediate production mutation | fixtures `live-01`..`live-05`, `negative-02-upgrade-execution`, `negative-03-bgp-change` |
| Existing agent owns the problem | routing table out-of-board section; fixture `negative-05-tetragon` |
| Required telemetry inaccessible | `needs-review` rule in evidence-and-verdicts |
| Audit-compliant verdict demanded | `approved` is never certification; control lineage delegated |
| Previous remediation ticket closed | handoff rule 2 |
| BPF map metric missing | dataplane capacity reference: missing is unknown, never healthy |
| Experimental feature in docs | evidence-and-verdicts "Feature maturity" rule |

## 5. Residual risks

| ID | Risk | Severity | Mitigation available | Owner |
| --- | --- | --- | --- | --- |
| R-01 | No behavioral evaluation: agents may not follow their contracts on real evidence | high | Run a scored evaluation corpus per specialist before `stable` | maintainer |
| R-02 | Keyword grader checks the first owner only, and routes quoted ticket instructions to a gate | medium | Accepted as fail-safe; a model-graded routing eval would cover order | maintainer |
| R-03 | Kubernetes maestro's generated fixture has no Cilium domain | medium | Teach the generator cross-provider domains, or add Kubernetes to its skip set with a hand-tuned taxonomy | maintainer |
| R-04 | Kiro adapters have no verified tool restriction | medium | Re-check Kiro documentation for a restricting field | maintainer |
| R-05 | Ledgers stale at the next minor release (1.21) | high over time | Refresh lifecycle references first; mark deprecated if skipped twice | maintainer |
| R-06 | Codex sandbox handling differs between this board and the Istio board | low | Verify the Codex documentation and align both | maintainer |
| R-07 | Pod IPAM pool headroom has no board owner | low | Leave with the architecture agent, or add a fact-sheet-backed reference | maintainer |
| R-08 | Managed Cilium distributions differ from upstream | medium | Input contracts require the distribution's own notes | user |
| R-09 | Second refuter pass not run after fixes | low | Run it before promoting to `stable` | maintainer |

## 6. Deferred work and entry criteria

- **Runtime-security board (Tetragon correlation).** Enter when: a Tetragon fact sheet with ledger rows exists; a guard design for TracingPolicy enforcement exists or enforcement is excluded; the Falco agent's boundary is written.
- **Fleet drift detector.** A deterministic script that diffs `cilium-config` and Helm values across supplied clusters; not an agent.
- **Cilium-specific Gateway API reference.** Enter when users report Cilium Gateway API issues that the Kubernetes architecture agent cannot answer.

## 7. Roadmap

1. Beta (this change): board, gates, fixtures, ledgers.
2. Before `stable`: behavioral evaluation corpus per specialist (positive, negative, ambiguous, stale-evidence, injection), second refuter pass, R-03 fix.
3. At Cilium 1.21: ledger refresh, starting with lifecycle.
4. After two quarters of use: decide the maestro's future on measured routing value.

## 8. Final verdict

**CONDITIONALLY READY.** Ready for beta: every structural, safety and traceability property the board claims is enforced by a gate that was shown to fail on the defect it targets, and the content survived an independent refuter pass with every accepted finding fixed. Not ready for `stable`, because no agent has been evaluated answering a case, the maestro's value is a hypothesis, and the version-specific references will go stale without a maintenance commitment.
