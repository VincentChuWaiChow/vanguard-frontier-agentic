"""Actual offline helper tests. These do not execute an LLM or Istio."""
import ast
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
import istio_evidence_facts as f


def binding():
    return {"cluster_id": "lab-id", "context": "lab", "namespace": "shop",
            "resources": ["security.istio.io/AuthorizationPolicy/shop/restrict"],
            "verbs": ["apply"], "baseline_sha256": "a" * 64,
            "change_sha256": "b" * 64, "rollback_sha256": "c" * 64}


def approval():
    return {"approval_id": "approval-1", "approver_ref": "external-identity-ref",
            "review_ref": "review-1", "valid_from": "2026-09-08T10:00:00+04:00",
            "valid_until": "2026-09-08T11:00:00+04:00", "binding": binding()}


class JsonInputTests(unittest.TestCase):
    def test_valid(self):
        self.assertEqual(f.parse_json(b'{"a":1}'), {"a": 1})

    def test_duplicate_keys(self):
        with self.assertRaises(f.InputError): f.parse_json(b'{"a":1,"a":2}')

    def test_nonfinite(self):
        for text in (b'NaN', b'Infinity', b'-Infinity'):
            with self.subTest(text=text), self.assertRaises(f.InputError): f.parse_json(text)

    def test_float_overflow_rejected(self):
        with self.assertRaises(f.InputError): f.parse_json(b'1e9999')

    def test_brackets_inside_string_do_not_count_as_nesting(self):
        raw = json.dumps({"value": "[" * 100 + "\\\""}).encode()
        self.assertEqual(len(f.parse_json(raw)["value"]), 102)

    def test_explicit_depth_boundary(self):
        self.assertIsInstance(f.parse_json(b'[' * 64 + b']' * 64), list)
        with self.assertRaises(f.InputError): f.parse_json(b'[' * 65 + b']' * 65)

    def test_invalid_utf8(self):
        with self.assertRaises(f.InputError): f.parse_json(b'\xff')

    def test_oversized(self):
        with self.assertRaises(f.InputError): f.parse_json(b' ' * (f.MAX_INPUT_BYTES + 1))

    def test_deep_input(self):
        with self.assertRaises(f.InputError): f.parse_json(b'[' * 5000 + b']' * 5000)

    def test_excessive_integer(self):
        with self.assertRaises(f.InputError): f.parse_json(b'9' * 10000)

    def test_invalid_syntax(self):
        with self.assertRaises(f.InputError): f.parse_json(b'{"x":')


class RuleShapeTests(unittest.TestCase):
    def test_absent_rules(self):
        self.assertEqual(f.rule_list_shape({"action": "DENY"})["shape"], "absent")

    def test_empty_list(self):
        result = f.rule_list_shape({"action": "DENY", "rules": []})
        self.assertEqual(result["shape"], "empty-list")
        self.assertEqual(result["effective_access"], "not-evaluated")

    def test_empty_rule_differs(self):
        result = f.rule_list_shape({"action": "DENY", "rules": [{}]})
        self.assertEqual(result["shape"], "contains-empty-rule")
        self.assertEqual(result["empty_rule_count"], 1)

    def test_constrained_rule(self):
        self.assertEqual(f.rule_list_shape({"rules": [{"to": []}]})["shape"], "nonempty-constrained-rules")

    def test_default_action(self):
        self.assertEqual(f.rule_list_shape({})["action"], "ALLOW")

    def test_bad_action(self):
        for action in (None, "allow", "UNKNOWN", []):
            with self.subTest(action=action), self.assertRaises(f.InputError): f.rule_list_shape({"action": action})

    def test_null_and_scalar_rules(self):
        for rules in (None, "", {}, [None], [1]):
            with self.subTest(rules=rules), self.assertRaises(f.InputError): f.rule_list_shape({"rules": rules})


class WeightTests(unittest.TestCase):
    def test_proportional_equivalence(self):
        self.assertEqual(f.relative_weights([9, 1])["shares"], f.relative_weights([90, 10])["shares"])

    def test_exact_fractions(self):
        self.assertEqual(f.relative_weights([1, 2])["shares"], ["1/3", "2/3"])

    def test_zero_component(self):
        self.assertEqual(f.relative_weights([0, 10])["shares"], ["0", "1"])

    def test_no_observation_claim(self):
        result = f.relative_weights([9, 1])
        self.assertEqual(result["observed_distribution"], "not-measured")
        self.assertEqual(result["api_validation"], "not-performed")

    def test_invalid_weights(self):
        for weights in ([], [0, 0], [-1, 3], [True, 2], [0.2, 0.8], ["9", 1], None):
            with self.subTest(weights=weights), self.assertRaises(f.InputError): f.relative_weights(weights)


class StatusTests(unittest.TestCase):
    def result(self, generation, observed):
        return f.condition_freshness(generation, {"type": "Accepted", "status": "True", "observedGeneration": observed})

    def test_stale(self): self.assertEqual(self.result(2, 1)["generation_relation"], "stale")
    def test_current(self): self.assertEqual(self.result(2, 2)["generation_relation"], "current")
    def test_missing(self): self.assertEqual(self.result(2, None)["generation_relation"], "unknown")
    def test_future(self): self.assertEqual(self.result(2, 3)["generation_relation"], "inconsistent-future-generation")

    def test_current_is_not_traffic_proof(self):
        self.assertEqual(self.result(2, 2)["end_to_end_behavior"], "not-proven")
        self.assertEqual(self.result(2, 2)["parent_and_controller_ownership"], "not-checked")

    def test_invalid_generation(self):
        for generation in (0, -1, True, "2"):
            with self.subTest(generation=generation), self.assertRaises(f.InputError): self.result(generation, 1)

    def test_boolean_status_rejected(self):
        with self.assertRaises(f.InputError): f.condition_freshness(2, {"type": "Accepted", "status": True})


class VersionTests(unittest.TestCase):
    def test_direction(self):
        self.assertEqual(f.directional_skew("1.30.0", "1.29.5")["numeric_relation"], "control-plane-one-minor-ahead")
        self.assertEqual(f.directional_skew("1.29.5", "1.30.0")["numeric_relation"], "data-plane-minor-ahead")

    def test_more_than_one_minor(self):
        self.assertEqual(f.directional_skew("1.30.0", "1.28.0")["numeric_relation"], "control-plane-more-than-one-minor-ahead")

    def test_patch(self):
        self.assertEqual(f.directional_skew("1.30.0", "1.30.1")["numeric_relation"], "same-minor-data-plane-newer-patch")

    def test_same_minor(self):
        self.assertEqual(f.directional_skew("1.30.2", "1.30.1")["numeric_relation"], "same-minor")

    def test_different_major(self):
        self.assertEqual(f.directional_skew("2.0.0", "1.30.0")["numeric_relation"], "different-major-requires-specific-review")

    def test_numeric_only_not_support(self):
        self.assertEqual(f.directional_skew("1.20.0", "1.19.0")["support_status"], "not-determined")

    def test_invalid_versions(self):
        for version in ("latest", "1.30", "v1.30.0", "01.30.0", "1.30.0-rc.1", None):
            with self.subTest(version=version), self.assertRaises(f.InputError): f.directional_skew(version, "1.30.0")


class StorageTests(unittest.TestCase):
    def crd(self):
        return {"spec": {"versions": [{"name": "v1", "storage": True, "served": True}, {"name": "v1beta1", "storage": False, "served": False}]}, "status": {"storedVersions": ["v1", "v1beta1"]}}

    def test_historical_storage(self):
        result = f.crd_storage_summary(self.crd())
        self.assertEqual(result["storage_write_version"], "v1")
        self.assertEqual(result["historical_versions_not_served"], ["v1beta1"])
        self.assertEqual(result["stored_object_migration"], "not-proven")

    def test_missing_history(self):
        crd = self.crd(); del crd["status"]
        self.assertIsNone(f.crd_storage_summary(crd)["historical_stored_versions"])

    def test_multiple_storage(self):
        crd = self.crd(); crd["spec"]["versions"][1]["storage"] = True
        with self.assertRaises(f.InputError): f.crd_storage_summary(crd)

    def test_no_storage(self):
        crd = self.crd(); crd["spec"]["versions"][0]["storage"] = False
        with self.assertRaises(f.InputError): f.crd_storage_summary(crd)

    def test_duplicate_version(self):
        crd = self.crd(); crd["spec"]["versions"][1]["name"] = "v1"
        with self.assertRaises(f.InputError): f.crd_storage_summary(crd)


class RoutingTests(unittest.TestCase):
    def test_minimal_owner(self):
        self.assertEqual(f.route_review("review", ["authorization"])["review_owners"], ["istio-authorization-policy-review-agent"])

    def test_ambient_precedes_authorization(self):
        self.assertEqual(f.route_review("review", ["authorization", "ambient"])["review_owners"], ["istio-ambient-mesh-review-agent", "istio-authorization-policy-review-agent"])

    def test_mutation_never_dispatches(self):
        result = f.route_review("mutate", ["authorization"])
        self.assertEqual(result["mode"], "live-guard-handoff")
        self.assertEqual(result["auto_dispatch"], [])
        self.assertEqual(result["handoff_owner"], "kubernetes-live-mesh-policy-guard-agent")

    def test_collection_handoff(self):
        self.assertEqual(f.route_review("collect", [])["mode"], "runtime-collection-handoff")

    def test_empty_is_unresolved(self): self.assertEqual(f.route_review("review", [])["mode"], "needs-review")

    def test_unknown_owner(self):
        with self.assertRaises(f.InputError): f.route_review("review", ["imagined-owner"])

    def test_duplicate_owner(self):
        with self.assertRaises(f.InputError): f.route_review("review", ["ambient", "ambient"])

    def test_unknown_blocks_approval(self):
        result = f.aggregate_reviews([{"id": "a", "scope_id": "s", "verdict": "approved"}, {"id": "b", "scope_id": "s", "verdict": "needs-review"}])
        self.assertEqual(result["verdict"], "needs-review")

    def test_blocker_preserved(self):
        result = f.aggregate_reviews([{"id": "a", "scope_id": "s", "verdict": "approved"}, {"id": "b", "scope_id": "s", "verdict": "blocked"}])
        self.assertEqual(result["verdict"], "blocked")

    def test_cross_scope_not_aggregated(self):
        self.assertEqual(f.aggregate_reviews([{"id": "a", "scope_id": "s1", "verdict": "approved"}, {"id": "b", "scope_id": "s2", "verdict": "approved"}])["verdict"], "needs-review")

    def test_approved_not_authorization(self):
        result = f.aggregate_reviews([{"id": "a", "scope_id": "s", "verdict": "approved"}])
        self.assertEqual(result["verdict"], "approved")
        self.assertEqual(result["execution_authorization"], "not-granted")

    def test_duplicate_reviews(self):
        item = {"id": "a", "scope_id": "s", "verdict": "approved"}
        with self.assertRaises(f.InputError): f.aggregate_reviews([item, item])

    def test_empty_reviews(self):
        with self.assertRaises(f.InputError): f.aggregate_reviews([])


class ApprovalBindingTests(unittest.TestCase):
    def check(self, apr=None, current=None, as_of="2026-09-08T10:30:00+04:00"):
        return f.approval_binding_check(apr or approval(), current or binding(), as_of)

    def test_matching_not_authority(self):
        result = self.check()
        self.assertTrue(result["binding_matches"])
        self.assertEqual(result["approval_authenticity"], "not-verified")
        self.assertEqual(result["execution_authorization"], "not-granted")

    def test_every_bound_field_detects_drift(self):
        for key in f.BINDING_KEYS:
            current = binding()
            if key in ("resources", "verbs"):
                current[key] = ["patch"] if key == "verbs" else ["security.istio.io/AuthorizationPolicy/shop/different"]
            elif key.endswith("sha256"):
                current[key] = "d" * 64
            else:
                current[key] = "different"
            with self.subTest(field=key):
                self.assertIn(key, self.check(current=current)["mismatches"])

    def test_expiry_exclusive(self):
        self.assertIn("validity_window", self.check(as_of="2026-09-08T11:00:00+04:00")["mismatches"])

    def test_not_yet_valid(self):
        self.assertIn("validity_window", self.check(as_of="2026-09-08T09:59:59+04:00")["mismatches"])

    def test_start_inclusive(self):
        self.assertTrue(self.check(as_of="2026-09-08T10:00:00+04:00")["binding_matches"])

    def test_timezone_equivalence(self):
        self.assertTrue(self.check(as_of="2026-09-08T06:30:00Z")["binding_matches"])

    def test_naive_time_rejected(self):
        with self.assertRaises(f.InputError): self.check(as_of="2026-09-08T10:30:00")

    def test_reversed_window(self):
        apr = approval(); apr["valid_until"] = apr["valid_from"]
        with self.assertRaises(f.InputError): self.check(apr=apr)

    def test_unknown_approval_field(self):
        apr = approval(); apr["approved"] = True
        with self.assertRaises(f.InputError): self.check(apr=apr)

    def test_bad_digest(self):
        current = binding(); current["change_sha256"] = "abc"
        with self.assertRaises(f.InputError): self.check(current=current)

    def test_unknown_verb(self):
        current = binding(); current["verbs"] = ["force-delete-all"]
        with self.assertRaises(f.InputError): self.check(current=current)

    def test_wildcard_resource_is_rejected(self):
        apr = approval(); apr["binding"]["resources"] = ["*"]
        current = binding(); current["resources"] = ["*"]
        with self.assertRaises(f.InputError): self.check(apr=apr, current=current)

    def test_wildcard_resource_segment_is_rejected(self):
        resource = "security.istio.io/AuthorizationPolicy/*/restrict"
        apr = approval(); apr["binding"]["resources"] = [resource]
        current = binding(); current["resources"] = [resource]
        with self.assertRaises(f.InputError): self.check(apr=apr, current=current)


class CliTests(unittest.TestCase):
    def run_cli(self, data, *args, env=None):
        return subprocess.run([sys.executable, str(ROOT / "scripts/istio_evidence_facts.py"), *args], input=data, capture_output=True, env=env, timeout=10)

    def test_stdin_success(self):
        run = self.run_cli(b'{"operation":"relative-weights","input":{"weights":[9,1]}}')
        self.assertEqual(run.returncode, 0)
        self.assertEqual(json.loads(run.stdout)["shares"], ["9/10", "1/10"])

    def test_failure_does_not_echo_secret(self):
        run = self.run_cli(b'{"SECRET_TOKEN_ABC":')
        self.assertEqual(run.returncode, 2)
        self.assertNotIn(b'SECRET_TOKEN_ABC', run.stderr)
        self.assertNotIn(b'Traceback', run.stderr)

    def test_operation_extra_keys_rejected(self):
        run = self.run_cli(b'{"operation":"relative-weights","input":{"weights":[1],"command":"unexpected"}}')
        self.assertEqual(run.returncode, 2)

    def test_symlink_file_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp); (path / "real").write_text('{}'); (path / "alias").symlink_to(path / "real")
            self.assertEqual(self.run_cli(b'', str(path / "alias")).returncode, 2)

    def test_present_kubeconfig_ignored(self):
        with tempfile.TemporaryDirectory() as tmp:
            kube = Path(tmp) / "config"; kube.write_text('PRIVATE_CANARY')
            env = {**os.environ, "KUBECONFIG": str(kube)}
            run = self.run_cli(b'{"operation":"relative-weights","input":{"weights":[9,1]}}', env=env)
            self.assertEqual(run.returncode, 0)
            self.assertNotIn(b'PRIVATE_CANARY', run.stdout + run.stderr)
            self.assertEqual(kube.read_text(), 'PRIVATE_CANARY')

    def test_helper_has_no_network_shell_or_environment_import(self):
        tree = ast.parse((ROOT / "scripts/istio_evidence_facts.py").read_text())
        imports = []
        for node in ast.walk(tree):
            if isinstance(node, ast.Import): imports.extend(alias.name.split('.')[0] for alias in node.names)
            if isinstance(node, ast.ImportFrom): imports.append((node.module or '').split('.')[0])
        self.assertFalse(set(imports) & {"socket", "requests", "urllib", "http", "subprocess", "os", "kubernetes"})


if __name__ == '__main__': unittest.main()
