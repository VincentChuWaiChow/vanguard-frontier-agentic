"""Local package/actual-selector/actual-snippet tests: never call real OCI."""
import ast
import builtins
import contextlib
import copy
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import socket
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]


def module(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / f"{name}.py")
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


selector = module("select_mode")
review = module("review_due")
recovery = module("check_recovery")


def fixture_contract(ddl, tf, limits, keys):
    """Controlled asset-equivalence check, NOT an HCL/Oracle DDL parser."""
    def require(condition, message):
        if not condition:
            raise ValueError(message)
    expected_columns = [("tenantId", "STRING"), ("orderId", "STRING"), ("status", "STRING"),
                        ("createdAt", "TIMESTAMP(3)"), ("amount", "NUMBER"), ("document", "JSON")]
    columns = re.findall(r"^\s*(\w+)\s+(STRING|TIMESTAMP\(3\)|NUMBER|JSON)\s*,", ddl, re.M)
    require(columns == expected_columns, "column schema changed")
    require(re.search(r"PRIMARY KEY\(SHARD\(tenantId\), orderId\)", ddl), "key mismatch")
    require("USING TTL 30 DAYS" in ddl, "TTL mismatch")
    require("IF NOT EXISTS" not in ddl and "DROP" not in ddl, "unsafe DDL")
    require('source = "oracle/oci"' in tf, "wrong provider")
    require('resource "oci_nosql_table" "orders"' in tf, "table missing")
    require('resource "oci_nosql_index" "tenant_status_created"' in tf, "index missing")
    require('file("${path.module}/orders.ddl")' in tf, "DDL not linked")
    require('"CREATE TABLE orders_v1", "CREATE TABLE ${var.table_name}"' in tf, "name substitution mismatch")
    require('ddl_statement       = local.ddl' in tf, "schema path unused")
    require(re.search(r'is_auto_reclaimable\s*=\s*false', tf), "reclaim risk")
    require(re.search(r'is_if_not_exists\s*=\s*false', tf), "adoption risk")
    require(tf.count('prevent_destroy = true') == 2, "destruction guard missing")
    require('kms_key_id' not in tf, "fictional table CMEK")
    mappings = {"max_read_units": "maxReadUnits", "max_write_units": "maxWriteUnits", "max_storage_in_gbs": "maxStorageInGBs"}
    for hcl, cli in mappings.items():
        value = re.search(rf"{hcl}\s*=\s*(\d+)", tf)
        require(value is not None and int(value[1]) == limits[cli], "capacity mismatch")
    require(limits["capacityMode"] == "PROVISIONED" and 'capacity_mode      = "PROVISIONED"' in tf, "capacity mode")
    actual_keys = re.findall(r'keys\s*\{\s*column_name\s*=\s*"([^"]+)"\s*\}', tf)
    require(actual_keys == [x["columnName"] for x in keys] == ["tenantId", "status", "createdAt"], "index order")
    return True


class Package(unittest.TestCase):
    def test_entry_metadata_size_and_commands(self):
        text = (ROOT / "SKILL.md").read_text()
        self.assertIn("name: oci-nosql-pattern\n", text)
        self.assertIn('version: "1.0.0"', text)
        self.assertLessEqual(len(text.splitlines()), 90)
        self.assertLessEqual(len(text.split()), 900)
        self.assertGreater(text.count("|"), 50)
        for command in selector.COMMANDS:
            self.assertTrue((ROOT / f"references/commands/{command}.md").is_file())

    def test_json_and_all_local_markdown_targets(self):
        for file in ROOT.rglob("*.json"):
            json.loads(file.read_text())
        for file in ROOT.rglob("*.md"):
            for target in re.findall(r"\[[^\]]*\]\(([^)]+)\)", file.read_text()):
                if "://" in target or target.startswith("#"):
                    continue
                self.assertTrue((file.parent / target.split("#")[0]).exists(), f"{file}: {target}")

    def test_selector_import_is_stdlib_and_no_execution_primitives(self):
        source = (ROOT / "scripts/select_mode.py").read_text()
        tree = ast.parse(source)
        imports = {node.names[0].name.split(".")[0] for node in ast.walk(tree) if isinstance(node, ast.Import)}
        self.assertEqual(imports, {"argparse", "json", "sys"})
        forbidden = {"open", "read_text", "read_bytes", "write_text", "write_bytes", "exec", "eval", "system", "Popen", "run"}
        calls = {getattr(n.func, "id", getattr(n.func, "attr", "")) for n in ast.walk(tree) if isinstance(n, ast.Call)}
        self.assertFalse(calls & forbidden)

    def test_documented_cli_input_shapes(self):
        shapes = json.loads((ROOT / "evals/cli-shapes.json").read_text())
        self.assertEqual(shapes["cli_version"], "3.95.0")
        commands = {tuple(x["command"][1:]): x["input_shape"] for x in shapes["evidence"]}
        self.assertEqual(len(commands), 10)
        for command, shape in commands.items():
            if command[0] == "nosql":
                self.assertIn("compartmentId", shape)
        limits = json.loads((ROOT / "assets/table-limits.json").read_text())
        self.assertEqual(set(limits), set(commands[("nosql", "table", "create")]["tableLimits"]))
        query = json.loads((ROOT / "assets/query.json").read_text())
        self.assertTrue(query["isPrepared"])
        self.assertTrue(set(query) <= set(commands[("nosql", "query", "execute")]))
        sql = (ROOT / "assets/query.sql").read_text()
        for variable in query["variables"]:
            self.assertIn(variable, sql)
        self.assertIn("o.tenantId = $tenant", sql)

    def test_provisioning_fixtures_positive(self):
        self.assertTrue(fixture_contract((ROOT / "assets/orders.ddl").read_text(),
            (ROOT / "assets/orders.tf").read_text(), json.loads((ROOT / "assets/table-limits.json").read_text()),
            json.loads((ROOT / "assets/index-keys.json").read_text())))

    def test_provisioning_fixtures_negative(self):
        ddl = (ROOT / "assets/orders.ddl").read_text()
        tf = (ROOT / "assets/orders.tf").read_text()
        limits = json.loads((ROOT / "assets/table-limits.json").read_text())
        keys = json.loads((ROOT / "assets/index-keys.json").read_text())
        mutations = [(ddl.replace("SHARD(tenantId)", "SHARD(orderId)"), tf, limits, keys),
            (ddl.replace("NUMBER", "DOUBLE"), tf, limits, keys),
            (ddl.replace("30 DAYS", "3 DAYS"), tf, limits, keys),
            (ddl, tf.replace("max_read_units     = 50", "max_read_units     = 500"), limits, keys),
            (ddl, tf.replace("prevent_destroy = true", "prevent_destroy = false"), limits, keys),
            (ddl, tf.replace("is_if_not_exists = false", "is_if_not_exists = true"), limits, keys),
            (ddl, tf, limits, list(reversed(keys))),
            (ddl, tf + '\nkms_key_id = "unsupported"\n', limits, keys)]
        for mutation in mutations:
            with self.subTest(mutation=str(mutation)[:80]), self.assertRaises(ValueError):
                fixture_contract(*mutation)

    def test_migrator_templates_are_cloud_full_export_new_import(self):
        out = json.loads((ROOT / "assets/migrator-export.json").read_text())
        incoming = json.loads((ROOT / "assets/migrator-import.json").read_text())
        self.assertEqual(out["source"]["type"], "nosqldb_cloud")
        self.assertEqual(out["sink"]["type"], "object_storage_oci")
        self.assertEqual(incoming["source"]["type"], "object_storage_oci")
        self.assertEqual(incoming["sink"]["type"], "nosqldb_cloud")
        self.assertNotEqual(out["source"]["table"], incoming["sink"]["table"])
        self.assertNotIn("queryFilter", out["source"])
        self.assertTrue(out["source"]["includeTTL"])
        self.assertTrue(incoming["sink"]["includeTTL"])
        self.assertNotIn("ttlRelativeDate", incoming["sink"])
        self.assertFalse(incoming["sink"]["overwrite"])
        self.assertTrue(incoming["abortOnError"])
        self.assertEqual(set(incoming["sink"]["schemaInfo"]) & {"schemaPath", "useSourceSchema", "defaultSchema"}, {"schemaPath"})


class Routes(unittest.TestCase):
    def parse(self, args):
        return selector.build_plan(args.split())

    def test_help_and_prefix(self):
        expected = ["references/commands/help.md"]
        for args in [[], ["help"], ["/oci-nosql-pattern"], ["/oci-nosql-pattern", "help"]]:
            self.assertEqual(selector.build_plan(args)["read"], expected)

    def test_commands_and_every_selected_feature_have_exact_paths(self):
        for command in selector.COMMANDS[1:]:
            for feature, guides in selector.FEATURES.items():
                args = [command, "--features", feature]
                if command == "feature-plan":
                    if feature not in selector.ACTIONS:
                        continue
                    args += ["--action", "configure"]
                result = selector.build_plan(args)
                expected = [f"references/commands/{command}.md"]
                if command == "audit": expected += ["references/commands/audit/offline.md"]
                if command in {"backup", "restore"}: expected += [f"references/commands/{command}/migrator.md"]
                expected += guides
                self.assertEqual(result["read"], list(dict.fromkeys(expected)))
                self.assertEqual(result["execution"], "none")
                self.assertTrue(all((ROOT / x).is_file() for x in result["read"]))

    def test_selected_variants_do_not_load_siblings_or_indirect_methods(self):
        for mode in ["offline", "live"]:
            p = self.parse(f"audit --mode {mode} --features embeddings,backup")
            self.assertEqual(p["live_read_approval_required"], mode == "live")
            self.assertFalse(any(f"audit/{'live' if mode == 'offline' else 'offline'}" in x for x in p["read"]))
            self.assertFalse(any('/backup/' in x or '/restore/' in x for x in p['read']))
        for command in ["backup", "restore"]:
            for method in ["sdk", "migrator"]:
                p = self.parse(f"{command} --method {method} --features backup,encryption")
                self.assertIn(f"references/commands/{command}/{method}.md", p["read"])
                self.assertFalse(any(f"/{'sdk' if method == 'migrator' else 'migrator'}.md" in x for x in p["read"]))
                self.assertTrue(p["change_approval_required"])

    def test_routes_never_read_files_or_call_network_processes(self):
        with patch.object(builtins, "open", side_effect=AssertionError("read")), \
             patch.object(Path, "read_text", side_effect=AssertionError("read")), \
             patch.object(Path, "write_text", side_effect=AssertionError("write")), \
             patch.object(socket, "socket", side_effect=AssertionError("network")), \
             patch.object(subprocess, "run", side_effect=AssertionError("process")):
            self.parse("audit --mode live --features backup,embeddings")
            self.parse("restore --method sdk")

    def test_invalid_actions_and_arguments_fail_closed(self):
        cases = ["unknown", "/wrong help", "select --features ''", "select --features core,", "select --features pitr", "select --features streams",
                 "backup --method pitr", "restore --method on-demand", "help --features core", "audit --mode production",
                 "feature-plan --features vectors --action enable", "feature-plan --features core --action configure",
                 "feature-plan --features ttl,encryption --action disable", "feature-plan --features iam --action enable",
                 "performance-test --environment production", "design --mode live", "select --feat core"]
        for flag in ["--duration-seconds", "--max-requests", "--max-concurrency"]:
            cases += [f"performance-test {flag} {x}" for x in ["0", "-1", "nan", "1.5"]]
        for args in cases:
            with self.subTest(args=args), contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit) as exc:
                self.parse(args)
            self.assertEqual(exc.exception.code, 2)

    def test_actual_cli_selector(self):
        cli = subprocess.run([sys.executable, str(ROOT / 'scripts/select_mode.py'), '/oci-nosql-pattern', 'audit', '--mode', 'live', '--features', 'iam,encryption'], capture_output=True, text=True, check=True)
        result = json.loads(cli.stdout)
        self.assertEqual(result['execution'], 'none')
        self.assertTrue(result['live_read_approval_required'])
        bad = subprocess.run([sys.executable, str(ROOT / 'scripts/select_mode.py'), 'restore', '--method', 'pitr'], capture_output=True, text=True)
        self.assertEqual(bad.returncode, 2)
        self.assertFalse(bad.stdout)


class ReviewAndRecovery(unittest.TestCase):
    def test_review_dates_do_not_roll_forward_or_write(self):
        path = ROOT / 'assets/content-review.json'
        before = path.read_bytes()
        ledger = json.loads(before)
        for day, status in [('2026-11-08', 'not-due'), ('2026-12-01', 'due'), ('2026-12-02', 'overdue'), ('2030-06-03', 'overdue')]:
            result = review.review_status(day, ledger)
            self.assertEqual(result['status'], status)
            self.assertEqual(result['next_review'], '2026-12-01')
            self.assertFalse(result['writes'])
        run = subprocess.run([sys.executable, str(ROOT / 'scripts/review_due.py'), '--as-of', '2028-01-01'], capture_output=True, text=True, check=True)
        self.assertEqual(json.loads(run.stdout)['status'], 'overdue')
        self.assertEqual(path.read_bytes(), before)
        for day in ['2026-02-31', 'not-a-date']:
            with self.assertRaises(ValueError): review.review_status(day, ledger)

    def valid_manifest(self):
        m = json.loads((ROOT / 'assets/recovery-manifest.example.json').read_text())
        m['source'] = {'tenancy':'ocid1.tenancy.demo','region':'us-ashburn-1','compartment':'ocid1.compartment.demo','table':'orders_v1','table_ocid':'ocid1.nosqltable.source'}
        m['destination'] = {'region':'us-ashburn-1','compartment':'ocid1.compartment.demo','table':'orders_recovery','exists':False}
        m['backup'].update(namespace='demo', bucket='archive', prefix='unique-export', verified=True, failed_records=0,
            writes_quiesced=True, object_versions=['immutable-version-id'], schema_sha256='a'*64, data_sha256={'part1.json':'b'*64})
        return m

    def test_recovery_manifest_positive_and_actual_cli(self):
        m = self.valid_manifest()
        result = recovery.validate(m)
        self.assertEqual(result['execution'], 'none')
        self.assertEqual(result['live_evidence'], 'UNVERIFIED')
        self.assertFalse(result['cutover_authorized'])
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / 'manifest.json'; path.write_text(json.dumps(m))
            run = subprocess.run([sys.executable, str(ROOT / 'scripts/check_recovery.py'), str(path)], capture_output=True, text=True, check=True)
            self.assertEqual(json.loads(run.stdout)['status'], 'locally-consistent')

    def test_recovery_rejects_unsafe_inputs_and_incomplete_example(self):
        mutations = [('destination','table','orders_v1'), ('destination','table','ORDERS_V1'), ('destination','exists',True), ('destination','exists',None),
            ('backup','failed_records',1), ('backup','failed_records',False), ('backup','verified',False), ('backup','writes_quiesced',False),
            ('backup','object_versions',[]), ('backup','object_versions',['']), ('backup','data_sha256',{}), ('backup','schema_sha256','bad'),
            ('backup','export_end','2026-11-07T00:00:00Z'), ('backup','export_start','2026-11-08T00:00:00'),
            ('backup','include_ttl',False), ('source','table_ocid','REPLACE_SOURCE')]
        for group, key, value in mutations:
            m = self.valid_manifest(); m[group][key] = value
            with self.subTest(key=key), self.assertRaises((ValueError, KeyError)):
                recovery.validate(m)
        with self.assertRaises(ValueError):
            recovery.validate(json.loads((ROOT/'assets/recovery-manifest.example.json').read_text()))


FAKE_OCI = r"""#!/usr/bin/env python3
import json,os,sys
from pathlib import Path
args=sys.argv[1:]
with open(os.environ['CALL_LOG'],'a') as f:f.write(json.dumps(args)+'\n')
if args[:4] != ['--profile','approved-profile','--region','us-ashburn-1']:sys.exit(90)
cmd=args[4:7]
scenario=os.environ.get('SCENARIO','ok')
if cmd==['iam','tenancy','get']:
    if scenario=='tenancy-error':sys.exit(5)
    print('wrong-tenancy' if scenario=='tenancy-mismatch' else 'ocid1.tenancy.demo')
elif cmd==['iam','compartment','get']:
    if scenario=='compartment-error':sys.exit(5)
    print('wrong-compartment' if scenario=='compartment-mismatch' else 'ocid1.compartment.demo')
elif cmd==['nosql','table','create']:
    if scenario=='table-error':sys.exit(6)
    print('{"data":{"state":"SUCCEEDED"}}')
elif cmd==['nosql','index','create']:
    if scenario=='index-error':sys.exit(7)
    print('{"data":{"state":"SUCCEEDED"}}')
elif cmd in [['nosql','table','get'],['nosql','index','get']]:
    print('{"data":{"state":"ACTIVE"}}')
else:sys.exit(99)
"""


class ActualShellSnippet(unittest.TestCase):
    def run_snippet(self, scenario='ok', table='safe_orders', confirmation='CREATE safe_orders\n', remove=None):
        with tempfile.TemporaryDirectory() as temp:
            directory = Path(temp); fake = directory / 'oci'; fake.write_text(FAKE_OCI); fake.chmod(0o755)
            log = directory / 'calls.jsonl'
            env = {**os.environ, 'PATH':str(directory)+os.pathsep+os.environ['PATH'], 'CALL_LOG':str(log), 'SCENARIO':scenario,
                'OCI_PROFILE':'approved-profile','OCI_REGION':'us-ashburn-1','TENANCY_OCID':'ocid1.tenancy.demo',
                'COMPARTMENT_OCID':'ocid1.compartment.demo','PRINCIPAL_OCID':'ocid1.user.approved','TABLE_NAME':table}
            if remove:env.pop(remove,None)
            run = subprocess.run(['bash', str(ROOT/'assets/create-table.sh')], input=confirmation, env=env, capture_output=True, text=True, timeout=10)
            calls = [json.loads(x) for x in log.read_text().splitlines()] if log.exists() else []
            return run, calls

    def test_actual_script_positive_exact_targets_schema_limits_and_order(self):
        run, calls = self.run_snippet()
        self.assertEqual(run.returncode, 0, run.stderr)
        self.assertEqual([x[4:7] for x in calls], [['iam','tenancy','get'],['iam','compartment','get'],['nosql','table','create'],['nosql','index','create'],['nosql','table','get'],['nosql','index','get']])
        for call in calls[2:]:
            self.assertEqual(call[call.index('--compartment-id')+1], 'ocid1.compartment.demo')
        table, index = calls[2:4]
        ddl = table[table.index('--ddl-statement')+1]
        expected = (ROOT/'assets/orders.ddl').read_text().rstrip().replace('CREATE TABLE orders_v1','CREATE TABLE safe_orders')
        self.assertEqual(ddl, expected)
        self.assertEqual(table[table.index('--name')+1], 'safe_orders')
        limits = table[table.index('--table-limits')+1]
        self.assertEqual(json.loads(Path(limits.removeprefix('file://')).read_text()), json.loads((ROOT/'assets/table-limits.json').read_text()))
        self.assertEqual(index[index.index('--table-name-or-id')+1], 'safe_orders')
        self.assertEqual(index[index.index('--is-if-not-exists')+1], 'false')
        self.assertEqual(index[index.index('--keys')+1], f'file://{ROOT}/assets/index-keys.json')
        self.assertEqual(table[table.index('--wait-for-state')+1], 'SUCCEEDED')

    def test_wrong_target_metadata_and_failed_identity_stop_before_write(self):
        for scenario in ['tenancy-error','tenancy-mismatch','compartment-error','compartment-mismatch']:
            run,calls = self.run_snippet(scenario=scenario)
            self.assertNotEqual(run.returncode,0)
            self.assertFalse(any(x[4:7]==['nosql','table','create'] for x in calls))

    def test_confirmation_name_and_missing_scope_fail_closed(self):
        for options in [{'confirmation':'yes\n'},{'table':'orders;rm_bad'},{'table':'../orders'},{'remove':'COMPARTMENT_OCID'},{'remove':'PRINCIPAL_OCID'}]:
            run,calls = self.run_snippet(**options)
            self.assertNotEqual(run.returncode,0)
            self.assertFalse(any(x[4]=='nosql' for x in calls))

    def test_failed_mutation_never_adopts_deletes_or_continues(self):
        for scenario, expected_calls in [('table-error',3),('index-error',4)]:
            run,calls = self.run_snippet(scenario=scenario)
            self.assertNotEqual(run.returncode,0)
            self.assertEqual(len(calls),expected_calls)
            self.assertFalse(any('delete' in x for x in calls))
            self.assertFalse(any(x[4:7]==['nosql','table','get'] for x in calls))


if __name__ == '__main__':
    unittest.main()
