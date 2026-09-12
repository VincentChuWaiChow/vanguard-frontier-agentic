"""Native Istio review-suite contract and integration tests."""
import json
from pathlib import Path
import unittest
import jsonschema
import yaml

ROOT = Path(__file__).resolve().parents[2]
SKILLS = [
    "istio-ambient-mesh-review", "istio-authorization-policy-review",
    "istio-traffic-resilience-review", "istio-gateway-api-review",
    "istio-upgrade-readiness", "istio-dataplane-diagnostics",
    "istio-review-routing", "istio-live-policy-change",
]
AGENTS = {
    "istio-ambient-mesh-review": ("istio", "istio-ambient-mesh-review-agent"),
    "istio-authorization-policy-review": ("istio", "istio-authorization-policy-review-agent"),
    "istio-traffic-resilience-review": ("istio", "istio-traffic-resilience-review-agent"),
    "istio-gateway-api-review": ("istio", "istio-gateway-api-review-agent"),
    "istio-upgrade-readiness": ("istio", "istio-upgrade-readiness-agent"),
    "istio-dataplane-diagnostics": ("istio", "istio-dataplane-diagnostics-agent"),
    "istio-review-routing": ("istio", "istio-maestro-agent"),
    "istio-live-policy-change": ("kubernetes", "kubernetes-live-mesh-policy-guard-agent"),
}

def review():
    return {
        "schema_version": "1.0", "skill_id": "istio-authorization-policy-review",
        "verdict": "needs-review", "execution_tier": "static-review",
        "scope": {"question": "Which rule-list shape is present?", "environment": "synthetic fixture", "evidence_boundary": "one supplied policy; no runtime evidence"},
        "claims": [{"id": "claim-1", "classification": "derived", "statement": "The supplied list has zero rules.", "evidence_refs": ["policy.json#/spec/rules"], "applicability": "Structural list shape only", "rule_id": "AUTH-02"}],
        "tests": [{"id": "test-1", "state": "proposed", "expected": "Record intended positive and negative traffic behavior.", "observed": None, "evidence_refs": []}],
        "limitations": ["No runtime enforcement was observed."],
    }

class ReviewContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.validator=jsonschema.Draft202012Validator(json.loads((ROOT/"schemas/istio-review-contract.schema.json").read_text()))
    def rejects(self,data): self.assertTrue(list(self.validator.iter_errors(data)))
    def test_bounded_review_valid(self): self.validator.validate(review())
    def test_derived_claim_requires_evidence(self):
        data=review(); data["claims"][0]["evidence_refs"]=[]; self.rejects(data)
    def test_proposed_test_cannot_claim_observed_result(self):
        data=review(); data["tests"][0]["observed"]="passed"; self.rejects(data)
    def test_passed_test_requires_observation_and_evidence(self):
        data=review(); data["tests"][0]["state"]="passed"; self.rejects(data)
    def test_review_cannot_claim_execution_authority(self):
        data=review(); data["authorized"]=True; self.rejects(data)

class NativeIntegrationTests(unittest.TestCase):
    def test_all_skill_agent_bindings_resolve(self):
        for skill in SKILLS:
            with self.subTest(skill=skill):
                sm=json.loads((ROOT/f"skills/istio/{skill}/metadata.json").read_text())
                provider,agent=AGENTS[skill]
                am=json.loads((ROOT/f"agents/{provider}/{agent}/metadata.json").read_text())
                self.assertEqual(sm["id"],skill)
                self.assertEqual(am["companion_skills"],[skill])
                self.assertTrue((ROOT/f"skills/istio/{skill}/SKILL.md").is_file())
    def test_static_agents_are_read_only(self):
        for skill,(provider,agent) in AGENTS.items():
            if skill=="istio-live-policy-change": continue
            text=(ROOT/f"agents/{provider}/{agent}/harnesses/codex.toml").read_text()
            self.assertIn('sandbox_mode = "read-only"',text)
    def test_live_guard_is_not_duplicated(self):
        matches=list((ROOT/"agents").glob("**/kubernetes-live-mesh-policy-guard-agent"))
        self.assertEqual(len(matches),1)
        meta=json.loads((matches[0]/"metadata.json").read_text())
        self.assertEqual(meta["companion_skills"],["istio-live-policy-change"])
    def test_live_guard_preflight_covers_every_writable_istio_resource(self):
        guard=ROOT/"agents/kubernetes/kubernetes-live-mesh-policy-guard-agent"
        docs=(guard/"references/rbac-pre-flight.md").read_text()
        script=(ROOT/"tests/integration/rbac-pre-flight/guards/mesh-policy.sh").read_text()
        manifests=yaml.safe_load_all((guard/"references/least-privilege-rbac.yaml").read_text())
        role=next(item for item in manifests if item.get("kind")=="ClusterRole")
        writable={
            f"{resource}.{rule['apiGroups'][0]}"
            for rule in role["rules"] if set(rule["verbs"]) & {"create", "patch"}
            for resource in rule["resources"]
        }
        self.assertEqual(len(writable),5)
        for resource in writable:
            with self.subTest(resource=resource):
                for source in (docs, script):
                    self.assertIn(f"delete {resource}", source)
                    self.assertIn(f"create {resource}", source)
                    self.assertIn(f"patch {resource}", source)
    def test_semantic_eval_corpus_remains_unperformed(self):
        paths=list((ROOT/"tests/fixtures/istio-review-evals/expected").glob("*.json"))
        self.assertEqual(len(paths),55)
        for path in paths:
            self.assertEqual(json.loads(path.read_text())["run_status"],"not-run",path)
    def test_runtime_specs_remain_unperformed(self):
        paths=list((ROOT/"tests/fixtures/runtime-semantics").glob("*.json"))
        self.assertEqual(len(paths),7)
        for path in paths:
            data=json.loads(path.read_text()); self.assertEqual(data["status"],"not-run",path)
            self.assertTrue(data["preconditions"]); self.assertTrue(data["stop_conditions"])

if __name__ == "__main__": unittest.main()
