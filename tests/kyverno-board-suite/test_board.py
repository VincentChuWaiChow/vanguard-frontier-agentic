"""Kyverno policy engineering board: structural, routing, decision-record and policy-fixture gates.

Verification layers (reported separately; none substitutes for another):
  1. Structural: bindings, read-only tiers, generated drift, entry-point and reference contracts,
     isolated export, stale-claim regressions.
  2. Deterministic: routing fixtures cross-checked against the repository grader, decision-record
     validator fixtures, and pinned-CLI policy fixtures (run only when the pinned CLI is available).
  3. Runtime (API server / controller): specified in tests/fixtures/kyverno-runtime-semantics and
     asserted NOT run. A passing CLI fixture never establishes webhook availability.
  4. Model behavior: tests/fixtures/kyverno-board-evals is a rubric corpus; it stays `not-run` until a
     recorded run (model, harness, fixture revision, repetitions, failures) is added to its manifest.
"""
from __future__ import annotations

import importlib.util
import json
import os
import re
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BOARD = json.loads((ROOT / "scripts/kyverno_data/board.json").read_text(encoding="utf-8"))
AGENTS = {a["id"]: a for a in BOARD["agents"]}
SKILLS = {a["skill"]: a for a in BOARD["agents"]}
FIX = ROOT / "tests/fixtures"
PINNED_CLI = "v1.20.0"
SKILL_H2 = ["Triggers", "Nearest competing trigger", "Minimum inputs", "Optional inputs", "Operating rules",
            "Decision procedure", "Reference loading", "Response minimum"]
REF_H2 = ["Question", "Release scope", "Assumptions", "Evidence to request", "Interpretation",
          "Worked example (synthetic)", "Counterexample", "Escalation threshold", "What would change the verdict"]
GENERATED_REFS = {"evidence-ledger.md", "evidence-vocabulary.md"}
SEAT_KINDS = {"positive-trigger", "near-neighbor-negative", "evidence-insufficient"}
HARM_KINDS = {"business-harm-counterexample", "oracle", "promotion-justified"}
# Claims the baseline skill made that primary sources at v1.20.0 contradict. They must not return.
STALE = [
    (re.compile(r"migrate-policy"), "invented CLI command"),
    (re.compile(r"Kyverno does not enforce expiry", re.I), "v1 PolicyException has expiresAt"),
    (re.compile(r"Audit[`\s]*in production is a critical", re.I), "categorical Audit severity"),
    (re.compile(r"failurePolicy[`\s]*is[`\s]*Ignore[`\s]*\(default\)", re.I), "ValidatingPolicy failurePolicy defaults to Fail"),
    (re.compile(r"stable `?policies\.kyverno\.io/v1`? API surface", re.I), "v1 is served but v1beta1 is storage at v1.20.0"),
]


def load(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def h2s(text: str) -> list[str]:
    out, fence = [], False
    for line in text.splitlines():
        if line.startswith("```"):
            fence = not fence
        elif not fence and line.startswith("## "):
            out.append(line[3:].strip())
    return out


class Layer1Structure(unittest.TestCase):
    def test_every_agent_binds_its_skill_and_stays_read_only(self):
        for aid, spec in AGENTS.items():
            with self.subTest(agent=aid):
                adir = ROOT / "agents/kyverno" / aid
                meta = json.loads((adir / "metadata.json").read_text())
                self.assertEqual(meta["companion_skills"][0], spec["skill"])
                self.assertEqual(meta["execution_tier"], "static-review")
                self.assertIn('sandbox_mode = "read-only"', (adir / "harnesses/codex.toml").read_text())
                copilot = (adir / "harnesses/copilot.agent.md").read_text()
                tools = re.findall(r'^  - "([^"]+)"$', copilot.split("---")[1], re.M)
                self.assertEqual(set(tools), {"read", "search", "search/codebase"})
                body = (adir / "AGENT.md").read_text()
                self.assertIn(BOARD["live_guard"], body)
                self.assertNotIn("## Canonical Contract\n\n## ", body, "empty Canonical Contract section")

    def test_every_skill_is_static_and_has_entry_point_contract(self):
        for skill in SKILLS:
            with self.subTest(skill=skill):
                sdir = ROOT / "skills/kyverno" / skill
                text = (sdir / "SKILL.md").read_text()
                self.assertRegex(text, r"(?m)^allowed-tools: Read Grep Glob$")
                self.assertRegex(text, r"(?m)^  execution_tier: static-review$")
                self.assertEqual(h2s(text), SKILL_H2)
                self.assertNotRegex(text, r"```(bash|sh|shell|console)\b")
                self.assertTrue((sdir / "agents/openai.yaml").is_file())
                self.assertEqual(json.loads((sdir / "metadata.json").read_text())["id"], skill)

    def test_topic_references_are_independently_useful(self):
        for skill in SKILLS:
            for ref in sorted((ROOT / "skills/kyverno" / skill / "references").glob("*.md")):
                if ref.name in GENERATED_REFS:
                    continue
                with self.subTest(ref=str(ref.relative_to(ROOT))):
                    text = ref.read_text()
                    got = h2s(text)
                    missing = [h for h in REF_H2 if h not in got]
                    self.assertFalse(missing, f"missing sections {missing}")
                    self.assertRegex(text.splitlines()[0], r"^# .+\?\s*$", "H1 must be the question the reference answers")
                    self.assertIn("**Cheapest discriminating check**", text)
                    self.assertIn("**Keep out of evidence**", text)
                    self.assertNotRegex(text, r"https?://", "URLs belong in the generated ledger")

    def test_each_skill_entry_point_links_every_topic_reference(self):
        for skill in SKILLS:
            sdir = ROOT / "skills/kyverno" / skill
            text = (sdir / "SKILL.md").read_text()
            for ref in (sdir / "references").glob("*.md"):
                with self.subTest(skill=skill, ref=ref.name):
                    self.assertIn(f"](references/{ref.name})", text, "orphan reference: not on the loading map")

    def test_generated_outputs_are_in_sync(self):
        res = subprocess.run(["python3", "scripts/gen_kyverno_board.py", "--check"], cwd=ROOT, capture_output=True, text=True)
        self.assertEqual(res.returncode, 0, res.stderr)

    def test_generator_rejects_a_citation_to_an_unknown_source(self):
        gen = load(ROOT / "scripts/gen_kyverno_board.py", "gen_kyverno_board")
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / "fake-skill/references").mkdir(parents=True)
            (Path(tmp) / "fake-skill/SKILL.md").write_text("A claim [KYV-E99].\n")
            gen.SKILLS_DIR = Path(tmp)
            _, sources, vocab = gen.load()
            with self.assertRaises(SystemExit):
                gen.skill_outputs("fake-skill", sources, vocab)

    def test_generator_preserves_policy_managed_markdown_model_fields(self):
        # model-policy:apply owns `model:`/`effort:` in the claude-code and cursor adapters;
        # regenerating the board must not silently drop an applied override.
        gen = load(ROOT / "scripts/gen_kyverno_board.py", "gen_kyverno_board_md")
        board, sources, _ = gen.load()
        agent = board["agents"][0]
        fm = '---\nname: "x"\ndescription: "y"\nmodel: "{m}"\n{e}---\n\nbody\n'
        existing = {
            "harnesses/claude-code.agent.md": fm.format(m="opus", e='effort: "high"\n'),
            "harnesses/cursor.agent.md": fm.format(m="inherit", e=""),
            "harnesses/kiro-ide.agent.md": fm.format(m="hand-added", e=""),
        }
        files = gen.render_agent(agent, board, sources, existing)
        claude = files["harnesses/claude-code.agent.md"].split("\n---\n", 1)[0]
        self.assertTrue(claude.endswith('model: "opus"\neffort: "high"'), claude)
        self.assertIn('\nmodel: "inherit"\n---\n', files["harnesses/cursor.agent.md"])
        # Negative: a harness model-policy does not manage keeps no model line.
        self.assertNotIn("model:", files["harnesses/kiro-ide.agent.md"].split("\n---\n", 1)[0])
        # And with nothing on disk, no model line is invented.
        bare = gen.render_agent(agent, board, sources, {})
        self.assertNotIn("model:", bare["harnesses/claude-code.agent.md"].split("\n---\n", 1)[0])

    def test_ledgers_carry_only_cited_rows(self):
        marker = re.compile(r"\[(KYV-E\d+)\]")
        for skill in SKILLS:
            sdir = ROOT / "skills/kyverno" / skill
            cited = set()
            for md in sdir.rglob("*.md"):
                if md.name not in GENERATED_REFS:
                    cited |= set(marker.findall(md.read_text()))
            rows = set(re.findall(r"^\| (KYV-E\d+) \|", (sdir / "references/evidence-ledger.md").read_text(), re.M))
            with self.subTest(skill=skill):
                self.assertEqual(rows, cited)

    def test_stale_baseline_claims_do_not_return(self):
        roots = [ROOT / "skills/kyverno", ROOT / "agents/kyverno", ROOT / "powers/vanguard-kyverno"]
        for base in roots:
            for path in base.rglob("*"):
                if path.suffix not in {".md", ".json", ".toml", ".yaml"} or not path.is_file():
                    continue
                text = path.read_text()
                for pattern, why in STALE:
                    with self.subTest(path=str(path.relative_to(ROOT)), why=why):
                        self.assertIsNone(pattern.search(text), why)

    def test_seat_inventory_is_complete_and_merges_are_explained(self):
        seats = BOARD["seats"]
        self.assertEqual(len(seats), 8)
        for seat in seats:
            with self.subTest(seat=seat["seat"]):
                self.assertIn(seat["agent"], AGENTS)
                if seat["choice"] == "merged":
                    self.assertGreater(len(seat.get("merge_note", "")), 80)

    @unittest.skipUnless(shutil.which("node"), "node is required for the export check")
    def test_isolated_export_is_self_contained(self):
        with tempfile.TemporaryDirectory() as tmp:
            res = subprocess.run(
                ["node", "scripts/export-marketplace-agents.mjs", "--platform", "claude-code",
                 "--agents", ",".join(AGENTS), "--repo", tmp],
                cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(res.returncode, 0, res.stderr)
            skills_root = Path(tmp) / ".claude/skills"
            installed = {p.name for p in skills_root.iterdir()}
            self.assertTrue(set(SKILLS) | {"jira-ticket-authoring", "confluence-document-authoring"} <= installed, installed)
            for md in skills_root.rglob("*.md"):
                own = skills_root / md.relative_to(skills_root).parts[0]
                for link in re.findall(r"\]\(([^)\s#]+)", md.read_text()):
                    if link.startswith(("http://", "https://", "mailto:")):
                        continue
                    target = (md.parent / link).resolve()
                    with self.subTest(file=str(md.relative_to(skills_root)), link=link):
                        self.assertTrue(target.exists(), "dangling link after export")
                        self.assertTrue(str(target).startswith(str(own.resolve())), "link escapes the exported skill")


class Layer1Templates(unittest.TestCase):
    """Templates are part of the skill contract: discoverable, runnable, and free of known traps."""

    def kits(self):
        return sorted(p.parent for p in (ROOT / "skills/kyverno").glob("*/assets/templates/*/kyverno-test.yaml"))

    def test_every_template_is_on_its_skill_loading_map(self):
        for skill in SKILLS:
            tdir = ROOT / "skills/kyverno" / skill / "assets/templates"
            if not tdir.is_dir():
                continue
            text = (ROOT / "skills/kyverno" / skill / "SKILL.md").read_text()
            for item in sorted(tdir.iterdir()):
                link = f"assets/templates/{item.name}/README.md" if item.is_dir() else f"assets/templates/{item.name}"
                with self.subTest(skill=skill, template=item.name):
                    self.assertIn(f"]({link})", text, "template not reachable from the entry point")

    def test_every_skill_ships_at_least_one_template(self):
        for skill in SKILLS:
            with self.subTest(skill=skill):
                self.assertTrue(any((ROOT / "skills/kyverno" / skill / "assets/templates").glob("*")))

    def test_starter_kits_are_complete(self):
        kits = self.kits()
        self.assertGreaterEqual(len(kits), 6)
        for kit in kits:
            with self.subTest(kit=str(kit.relative_to(ROOT))):
                self.assertTrue((kit / "README.md").is_file())
                yaml_text = "".join(p.read_text() for p in kit.rglob("*.yaml"))
                self.assertIn("TEMPLATE", yaml_text, "kit must mark what to replace")

    def test_starter_kits_avoid_known_traps(self):
        traps = [
            (re.compile(r"controllers:\s*\[[^\]]*\b[A-Z][a-zA-Z]+"), "autogen controllers must be lowercase plural resource names (KYV-E38/E39)"),
            (re.compile(r"\.registry\b(?!\()"), "image().registry() is a method; field style fails to compile"),
            (re.compile(r"attestors\.[a-zA-Z0-9]*-"), "attestor names must be CEL identifiers (no hyphens)"),
            (re.compile(r"^\s*validationFailureAction:", re.M), "legacy ClusterPolicy field in a v1 starter kit"),
        ]
        for kit in self.kits():
            for path in kit.rglob("*.yaml"):
                text = path.read_text()
                for pattern, why in traps:
                    with self.subTest(file=str(path.relative_to(ROOT)), why=why):
                        self.assertIsNone(pattern.search(text), why)

    def test_policy_kits_set_lifecycle_switches_explicitly(self):
        for kit in self.kits():
            policy = kit / "policy.yaml"
            if not policy.is_file():
                continue
            text = policy.read_text()
            with self.subTest(kit=kit.name):
                if "kind: ValidatingPolicy" in text or "kind: ImageValidatingPolicy" in text:
                    self.assertRegex(text, r"(?m)^  validationActions: ")
                    self.assertRegex(text, r"(?m)^  failurePolicy: (Fail|Ignore)$")
                if "kind: GeneratingPolicy" in text:
                    for switch in ("generateExisting", "synchronize", "orphanDownstreamOnPolicyDelete"):
                        self.assertIn(switch, text)

    def test_decision_record_template_is_valid(self):
        mod = load(ROOT / "skills/kyverno/kyverno-review-routing/scripts/validate_board_decision.py", "vbd_template")
        record = json.loads((ROOT / "skills/kyverno/kyverno-review-routing/assets/templates/decision-record.template.json").read_text())
        self.assertEqual(mod.check(record, mod.load_validator()), [])


class Layer2Deterministic(unittest.TestCase):
    def test_routing_fixtures_cover_every_seat_with_trigger_and_near_neighbor(self):
        grader = load(ROOT / "tests/validate-maestro-routing.py", "validate_maestro_routing")
        tax = json.loads((FIX / "kyverno-maestro-routing/taxonomy.json").read_text())
        leads = {}
        for inp in sorted((FIX / "kyverno-maestro-routing/inputs").glob("*.json")):
            task = json.loads(inp.read_text())
            expected = json.loads((FIX / "kyverno-maestro-routing/expected" / inp.name).read_text())
            self.assertEqual(grader.evaluate(task["task"], tax), expected, inp.name)
            for agent in expected["route"]:
                leads.setdefault(agent, set()).update(task["tags"])
        for aid in AGENTS:
            if aid == "kyverno-maestro-agent":
                continue
            with self.subTest(agent=aid):
                self.assertIn("happy-path", leads.get(aid, set()))
                self.assertIn("near-neighbor", leads.get(aid, set()))

    def test_decision_record_validator_rejects_persuasive_wrong_records(self):
        mod = load(ROOT / "skills/kyverno/kyverno-review-routing/scripts/validate_board_decision.py", "vbd")
        examples = sorted((ROOT / "skills/kyverno/kyverno-review-routing/assets/examples").glob("*/decision.json"))
        self.assertEqual(mod.run_suite(FIX / "kyverno-board-decisions", extra_valid=examples), 0)

    def test_schema_diagnostics_never_echo_credentials(self):
        mod = load(ROOT / "skills/kyverno/kyverno-review-routing/scripts/validate_board_decision.py", "vbd_secret")
        rec = json.loads((FIX / "kyverno-board-decisions/valid/02-invoice-coverage-conditional.json").read_text())
        # Built at runtime so no credential-shaped literal sits in the repository.
        fake_value = "Bear" + "er " + "q" * 40
        rec["request_kind"] = fake_value  # schema-invalid field carrying a credential-shaped value
        found = mod.check(rec, mod.load_validator())
        self.assertEqual({r for r, _ in found}, {"KYB-SECRET", "KYB-SCHEMA"})
        self.assertFalse(any("q" * 20 in msg for _, msg in found), found)

    def test_invoice_fixture_is_the_published_worked_record(self):
        a = (ROOT / "skills/kyverno/kyverno-review-routing/assets/examples/KYV-DEC-0001/decision.json").read_text()
        b = (FIX / "kyverno-board-decisions/valid/02-invoice-coverage-conditional.json").read_text()
        self.assertEqual(a, b)
        rec = json.loads(a)
        cov = rec["coverage"]
        self.assertEqual((cov["claimed"]["numerator"], cov["claimed"]["denominator"]), (100, 120))
        self.assertIsNone(cov["compliance_percentage"])
        drafts = {p["path"] for p in rec["artifacts"]}
        for draft in drafts:
            text = (ROOT / "skills/kyverno/kyverno-review-routing/assets/examples/KYV-DEC-0001" / draft).read_text()
            self.assertIn("KYV-DEC-0001", text)
            self.assertRegex(text, r"100(/| of )120")

    def test_policy_fixtures_declare_expectations(self):
        dirs = sorted(p for p in (FIX / "kyverno-policy-cli").iterdir() if p.is_dir())
        self.assertGreaterEqual(len(dirs), 4)
        for d in dirs:
            self.assertTrue((d / "kyverno-test.yaml").is_file(), d.name)

    def test_policy_fixtures_pass_with_pinned_cli(self):
        cli = os.environ.get("KYVERNO_CLI") or shutil.which("kyverno")
        if not cli:
            self.skipTest("pinned kyverno CLI not available (set KYVERNO_CLI); layer 2 policy fixtures NOT verified in this run")
        version = subprocess.run([cli, "version"], capture_output=True, text=True).stdout
        self.assertIn(f"Version: {PINNED_CLI}", version, "fixtures record behavior of the pinned CLI only")
        for target in (FIX / "kyverno-policy-cli", ROOT / "skills/kyverno"):
            with self.subTest(target=str(target.relative_to(ROOT))):
                res = subprocess.run([cli, "test", str(target)], capture_output=True, text=True)
                self.assertEqual(res.returncode, 0, res.stdout[-3000:] + res.stderr[-2000:])


class Layer3And4Status(unittest.TestCase):
    def test_runtime_specs_are_explicitly_unperformed(self):
        specs = sorted((FIX / "kyverno-runtime-semantics").glob("*.json"))
        self.assertGreaterEqual(len(specs), 4)
        for path in specs:
            data = json.loads(path.read_text())
            with self.subTest(spec=path.name):
                self.assertEqual(data["status"], "not-run")
                self.assertTrue(data["preconditions"] and data["stop_conditions"] and data["acceptance"])

    def test_eval_corpus_covers_every_seat_and_rejects_universal_blocking(self):
        root = FIX / "kyverno-board-evals"
        manifest = json.loads((root / "manifest.json").read_text())
        cases = [json.loads(p.read_text()) for p in sorted((root / "expected").glob("*.json"))]
        self.assertEqual(manifest["case_count"], len(cases))
        self.assertEqual(len(list((root / "inputs").glob("*.json"))), len(cases))
        recorded = {case for run in manifest["model_runs"] for case, result in run["results"].items()}
        for c in cases:
            with self.subTest(case=c["case_id"]):
                if c["case_id"] in recorded:
                    self.assertIn(c["run_status"], {"pilot-passed", "pilot-failed"})
                else:
                    self.assertEqual(c["run_status"], "not-run", "a case cannot claim a run the manifest does not record")
        for run in manifest["model_runs"]:
            self.assertTrue(run["model"] and run["harness"] and run["fixture_revision"] and run["repetitions"] >= 1)
            self.assertIn("observed_failures", run)
        if not manifest["model_runs"]:
            self.assertEqual(manifest["evaluation_status"], "not-run")
        by_skill = {}
        for c in cases:
            by_skill.setdefault(c["skill_id"], set()).add(c["kind"])
            self.assertTrue(c["required_assertions"] and c["forbidden_conclusions"], c["case_id"])
        for skill in SKILLS:
            if skill == "kyverno-review-routing":
                continue
            with self.subTest(skill=skill):
                self.assertTrue(SEAT_KINDS <= by_skill[skill], by_skill[skill])
                self.assertTrue(HARM_KINDS & by_skill[skill])
        self.assertTrue(any(c["kind"] == "promotion-justified" for c in cases), "universal blocking must fail the suite")
        self.assertTrue(any(c["kind"] == "mixed-intent" for c in cases))
        routed = {json.loads(p.read_text())["name"] for p in (FIX / "kyverno-maestro-routing/inputs").glob("*.json")}
        for c in cases:
            if c["routing_fixture"]:
                with self.subTest(case=c["case_id"]):
                    self.assertIn(c["routing_fixture"], routed)


if __name__ == "__main__":
    unittest.main()
