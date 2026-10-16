"""Focused Kiro-format/traceability checks, not semantic or SMT verification."""
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[3]
SPEC = ROOT / ".kiro/specs/cross-harness-mods"


def check(req, design, tasks):
    assert req.startswith("# Requirements Document\n")
    assert design.startswith("# Design: ")
    assert tasks.startswith("# Implementation Plan: ")
    for heading in ("Introduction", "Glossary", "Requirements"):
        assert f"## {heading}\n" in req, heading
    for heading in ("Overview", "Components and Interfaces", "Correctness Properties", "Testing Strategy"):
        assert f"## {heading}\n" in design, heading
    for heading in ("Overview", "Tasks", "Notes", "Task Dependency Graph"):
        assert f"## {heading}\n" in tasks, heading
    blocks = re.split(r"^### Requirement \d+:.*$", req, flags=re.M)[1:]
    assert len(blocks) == 18
    for block in blocks:
        assert "**User Story:**" in block
        assert "#### Acceptance Criteria" in block
    r18 = req.split("### Requirement 18:", 1)[1].split("## Correctness Properties", 1)[0]
    criteria = re.findall(r"^(\d+)\. ", r18, re.M)
    assert criteria == [str(i) for i in range(1, 21)]
    t9 = tasks.split("- [ ] 9. T9", 1)[1].split("## Notes", 1)[0]
    for criterion in criteria:
        ref = f"18.{criterion}"
        pattern = rf"(?<![\d.]){re.escape(ref)}(?![\d.])"
        assert re.search(pattern, t9), f"T9 missing {ref}"
        assert re.search(pattern, design), f"design missing {ref}"
    model = design.split("pub struct Mod {", 1)[1].split("\n}", 1)[0]
    assert "entity_type" not in model and "ModType" not in model
    assert "pub security_notes: Option<String>" in model
    assert "pub footprint: Option<ModFootprint>" in model
    assert "pub struct ModFootprint" in design
    assert "### ADR-7:" in design
    assert "### Property 8: TUI Mod Load Completeness" in design
    assert re.findall(r"^  - \[x\] (9\.\d+)\b", tasks, re.M) == ["9.0"]
    for status, parent in re.findall(r"^- \[([ x])\] (\d+)\.", tasks, re.M):
        if re.search(rf"^  - \[ \] {parent}\.\d+\b", tasks, re.M):
            assert status == " ", f"Completed parent {parent} has pending children"
    leaves = re.findall(r"^  - \[[ x]\] (\d+\.\d+)\b", tasks, re.M)
    graph_text = tasks.split("## Task Dependency Graph", 1)[1]
    graph = json.loads(re.search(r"```json\n(.*?)\n```", graph_text, re.S)[1])
    waves = graph["waves"]
    assert [w["id"] for w in waves] == list(range(len(waves)))
    scheduled = [task for wave in waves for task in wave["tasks"]]
    assert len(scheduled) == len(set(scheduled))
    assert set(scheduled) == set(leaves), "Missing or invented graph leaves"
    assert all(len(w["tasks"]) == 1 for w in waves), "Shared writers must serialize"
    assert scheduled.index("6.7") < scheduled.index("7.4")
    assert scheduled.index("8.6") < scheduled.index("8.4")
    assert scheduled.index("9.0") < scheduled.index("9.1")
    assert scheduled.index("9.7") < scheduled.index("9.8") < scheduled.index("9.9")
    assert scheduled.index("9.9") < scheduled.index("9.10") < scheduled.index("9.6")
    assert scheduled[-1] == "8.5"


req, design, tasks = [(SPEC / f"{name}.md").read_text() for name in ("requirements", "design", "tasks")]
check(req, design, tasks)
for bad_design, bad_tasks in (
    (design.replace("pub struct Mod {", "pub struct Mod {\n    pub entity_type: ModType,"), tasks),
    (design, tasks.replace('"tasks": ["9.3"]', '"tasks": ["9.4"]')),
):
    try:
        check(req, bad_design, bad_tasks)
    except AssertionError:
        continue
    raise AssertionError("Negative checker probe unexpectedly passed")
print("PASS: focused format, R18.1-20 traceability, unimplemented T9 status, serialized graph")
print("PASS: negative discriminator and graph mutations rejected")
print("PASS: approved full-browser and typed legacy-compatible footprint contract present")
