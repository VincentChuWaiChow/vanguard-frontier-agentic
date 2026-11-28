# 🛡️ Kyverno Skills

<p align="center">
  <!-- 🖼️ Add a Kyverno logo to assets/logos/cnative/kyverno/ and update this path -->
  <span style="font-size:3.5em">🛡️</span>
</p>

Companion skills for the Kyverno policy engineering board (see `agents/kyverno/README.md`). Each skill is a lean entry point with triggers, the nearest competing trigger, minimum inputs, read-only operating rules, a decision procedure, a reference-loading map and a response minimum. Topic references load on demand.

| Skill | Decision it settles |
| --- | --- |
| `kyverno-policy-review` | Does the rule separate legitimate from prohibited writes on the installed API family, and does an image rule verify the approved signer and claims? |
| `kyverno-admission-reliability` | Can admission sustain the change and recover from dependency failures, and which hypothesis explains a failure? |
| `kyverno-exception-governance` | Is a deviation necessary, bounded to its approval, and verifiably removed? |
| `kyverno-policy-lifecycle` | Will mutate, generate or scheduled delete converge with other reconcilers, and what remains after reversal? |
| `kyverno-migration-readiness` | Which cohorts can move between policy families or releases with behavior preserved? |
| `kyverno-control-evidence` | What coverage and impact can the evidence support, for which population and window? |
| `kyverno-review-routing` | Who owns each decision, and what evidence would settle disagreement? |

## Release scope

Content is pinned to Kyverno v1.20.0, the release that deleted the legacy `kyverno.io` policy engine. At that tag the `policies.kyverno.io` CRDs serve v1, v1beta1 and v1alpha1, with v1beta1 as the storage version. Legacy ClusterPolicy guidance applies only to releases before v1.20.

## Shared evidence and sources

`references/evidence-vocabulary.md` (evidence types, claim classes, outcome-to-verdict mapping) and `references/evidence-ledger.md` (only the source rows that skill cites) are generated into every skill from `scripts/kyverno_data/`, so each skill works after an isolated export. Edit `scripts/kyverno_data/`, then run `python3 scripts/gen_kyverno_board.py`.

Run `npm run validate` and `npm run validate:kyverno-board` after changing these skills.
