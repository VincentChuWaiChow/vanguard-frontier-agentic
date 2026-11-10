"""Check spec structure and evaluation coverage, not requirement semantics."""

import argparse
import hashlib
import json
import re
from pathlib import Path


def feature_rows(evaluations: str) -> str:
    heading = "## Feature acceptance scenarios\n"
    if heading not in evaluations:
        raise ValueError("missing feature acceptance section")
    return evaluations.split(heading, 1)[1].split("\n## ", 1)[0]


def check(requirements: str, evaluations: str) -> tuple[int, int]:
    rows = feature_rows(evaluations)
    for heading in (
        "# Requirements Document",
        "## Introduction",
        "## Glossary",
        "## User Roles",
        "## Requirements",
        "## Non-Functional Requirements",
        "## Out of Scope",
        "## Open Questions and Approval Gate",
    ):
        if heading not in requirements.splitlines():
            raise ValueError(f"missing heading: {heading}")

    blocks = list(re.finditer(r"^### Requirement (\d+): (.+)$", requirements, re.M))
    if not blocks:
        raise ValueError("no numbered requirements")
    numbers = [int(block.group(1)) for block in blocks]
    if numbers != list(range(1, len(blocks) + 1)):
        raise ValueError("requirement numbers must be unique and sequential")

    seen = set()
    for index, match in enumerate(blocks):
        end = blocks[index + 1].start() if index + 1 < len(blocks) else len(requirements)
        block = requirements[match.end() : end].split("\n## ", 1)[0]
        number = int(match.group(1))
        if "**User Story:**" not in block or "#### Acceptance Criteria" not in block:
            raise ValueError(f"R{number}: missing story or acceptance heading")
        criteria = re.findall(r"^\d+\. \*\*(R\d+\.\d+)\*\* (.+)$", block, re.M)
        if not criteria:
            raise ValueError(f"R{number}: no criteria")
        for offset, (identity, body) in enumerate(criteria, 1):
            if identity in seen or identity != f"R{number}.{offset}":
                raise ValueError(f"duplicate or nonsequential criterion: {identity}")
            seen.add(identity)
            if len(re.findall(r"\bSHALL\b", body)) != 1:
                raise ValueError(f"{identity}: expected one SHALL obligation")
            if not re.match(r"(?:WHEN|IF|WHILE|THE)\b", body):
                raise ValueError(f"{identity}: missing EARS trigger")
            if body.startswith(("WHEN ", "IF ")) and ", THEN " not in body:
                raise ValueError(f"{identity}: missing THEN response")
        row = re.compile(rf"^\| R{number} \| ([^\n]+) \| ([^\n]+) \|$", re.M)
        if not row.search(rows):
            raise ValueError(f"R{number}: missing positive/negative evaluation row")

    evaluated = re.findall(r"^\| R(\d+) \|", rows, re.M)
    if sorted(map(int, evaluated)) != numbers:
        raise ValueError("evaluation rows must cover each requirement exactly once")
    if "SHALL NEVER" not in requirements:
        raise ValueError("no explicit prohibition for adversarial analysis")
    return len(numbers), len(seen)


def self_test(requirements: str, evaluations: str) -> None:
    # Each mutant must fail the same production checker that accepts the real spec.
    rows = feature_rows(evaluations)
    without_first = re.sub(r"^\| R1 \|.*\n", "", rows, count=1, flags=re.M)
    mutants = {
        "missing heading": (requirements.replace("## Introduction\n", "", 1), evaluations),
        "duplicate ID": (requirements.replace("**R1.2**", "**R1.1**", 1), evaluations),
        "missing EARS response": (requirements.replace(", THEN ", ", ", 1), evaluations),
        "multiple obligations": (requirements.replace(" SHALL ", " SHALL SHALL ", 1), evaluations),
        "missing evaluation": (requirements, evaluations.replace(rows, without_first, 1)),
    }
    for label, (text, rows) in mutants.items():
        try:
            check(text, rows)
        except ValueError:
            print(f"PASS: rejects {label}")
        else:
            raise ValueError(f"checker incorrectly accepted mutant: {label}")


def check_design(requirements: str, design: str) -> int:
    for heading in ("## Overview", "## Architecture", "## Components and Interfaces",
                    "## Data Models", "## Error Handling", "## Testing Strategy",
                    "## Delivery and Migration", "## Requirement Traceability"):
        if heading not in design.splitlines():
            raise ValueError(f"design missing heading: {heading}")
    digest = hashlib.sha256(requirements.encode()).hexdigest()
    if digest not in design:
        raise ValueError("design missing approved requirements hash")
    anchors = re.findall(r"^### (D\d+):", design, re.M)
    tests = re.findall(r"^#### (V\d+):", design, re.M)
    for kind, values in (("D", anchors), ("V", tests)):
        if not values or values != [f"{kind}{n}" for n in range(1, len(values) + 1)]:
            raise ValueError(f"design {kind} anchors must be unique and sequential")
    expected = set(re.findall(r"^\d+\. \*\*(R\d+\.\d+)\*\*", requirements, re.M))
    table = design.split("## Requirement Traceability\n", 1)[1].split("\n## ", 1)[0]
    covered, used_anchors, used_tests = set(), set(), set()
    for row in table.splitlines():
        if not row.startswith("| R"):
            continue
        columns = [item.strip() for item in row.strip("|").split("|")]
        if len(columns) != 3:
            raise ValueError("traceability row must contain three columns")
        span = re.fullmatch(r"R(\d+)\.(\d+)(?:-R(\d+)\.(\d+))?", columns[0])
        if not span:
            raise ValueError(f"invalid criterion range: {columns[0]}")
        group, start = int(span[1]), int(span[2])
        end_group = int(span[3]) if span[3] else group
        end = int(span[4]) if span[4] else start
        if group != end_group or end < start or end - start > len(expected):
            raise ValueError("invalid or excessive criterion range")
        for identity in (f"R{group}.{n}" for n in range(start, end + 1)):
            if identity not in expected or identity in covered:
                raise ValueError(f"unknown or duplicate design criterion: {identity}")
            covered.add(identity)
        for column, known, used in ((columns[1], anchors, used_anchors),
                                    (columns[2], tests, used_tests)):
            references = column.split(", ")
            if not references or any(ref not in known for ref in references):
                raise ValueError(f"unknown design/test anchor: {column}")
            used.update(references)
    if covered != expected:
        raise ValueError(f"missing design criteria: {sorted(expected - covered)}")
    if used_anchors != set(anchors) or used_tests != set(tests):
        raise ValueError("unreferenced design/test anchor")
    return len(covered)


def self_test_design(requirements: str, design: str) -> None:
    row = next(line for line in design.splitlines() if line.startswith("| R1."))
    digest = hashlib.sha256(requirements.encode()).hexdigest()
    mutants = {
        "missing design trace row": design.replace(row + "\n", "", 1),
        "duplicate design coverage": design.replace(row, row + "\n" + row, 1),
        "missing design section": re.sub(r"^### D3:.*$", "### Runtime bridge", design, count=1, flags=re.M),
        "unknown verification anchor": design.replace(row, row.replace("V3", "V999"), 1),
        "wrong approved hash": design.replace(digest, "0" * 64),
    }
    for label, mutant in mutants.items():
        try:
            check_design(requirements, mutant)
        except ValueError:
            print(f"PASS: rejects {label}")
        else:
            raise ValueError(f"checker incorrectly accepted mutant: {label}")


def check_tasks(requirements: str, design: str, tasks: str) -> int:
    for heading in ("# Implementation Plan: GPUI Desktop", "## Check Registry",
                    "## Tasks", "## Portable Visual Quality Workflow",
                    "## Admission and Completion Gates"):
        if heading not in tasks.splitlines():
            raise ValueError(f"task plan missing heading: {heading}")
    if tasks.index("## Tasks\n") > tasks.index("## Check Registry\n"):
        raise ValueError("task checklist must precede the check registry")
    fixed_path = r"(?<![\w.-])/(?:[\w.@{}$-]+/)+|[A-Za-z]:[\\/]|\\\\[^\\\s]+\\|(?:\.agents|\.codex)/skills/impeccable"
    if re.search(fixed_path, tasks):
        raise ValueError("task plan contains a fixed machine or skill-install path")
    fence = None
    for line in tasks.splitlines():
        marker = re.match(r"^\s*(`{3,}|~{3,})", line)
        if marker:
            if fence is None:
                fence = marker[1][0]
            elif marker[1][0] == fence:
                fence = None
        elif fence and re.match(r"^- \[[ x]\] \d", line):
            raise ValueError("task checkboxes must not be inside a code fence")
    for text in (requirements, design):
        if hashlib.sha256(text.encode()).hexdigest() not in tasks:
            raise ValueError("task plan missing frozen approved input hash")
    checks = re.findall(r"^### (C\d+):", tasks, re.M)
    if not checks or checks != [f"C{n}" for n in range(1, len(checks) + 1)]:
        raise ValueError("check registry must be unique and sequential")
    expected = set(re.findall(r"^\d+\. \*\*(R\d+\.\d+)\*\*", requirements, re.M))
    design_ids = set(re.findall(r"^### (D\d+):", design, re.M))
    test_ids = set(re.findall(r"^#### (V\d+):", design, re.M))
    area = tasks.split("## Tasks\n", 1)[1].split("\n## ", 1)[0]
    blocks = list(re.finditer(r"^- \[([ x])\] (\d+\.\d+) (.+)$", area, re.M))
    epics = re.findall(r"^- \[[ x]\] (\d+)\. (.+)$", area, re.M)
    if [int(n) for n, _ in epics] != list(range(1, len(epics) + 1)):
        raise ValueError("epic IDs must be unique and sequential")
    if not blocks:
        raise ValueError("no implementation subtasks")
    ids = [block[2] for block in blocks]
    if len(ids) != len(set(ids)):
        raise ValueError("duplicate task ID")
    fields_required = {"Depends", "Paths", "Design", "Requirements", "Verification",
                       "Checks", "Positive", "Negative", "Estimate"}
    graph, covered, used_design, used_tests, used_checks = {}, set(), set(), set(), set()
    methods = {}
    numbers = {}
    for index, match in enumerate(blocks):
        identity = match[2]
        major, minor = map(int, identity.split("."))
        numbers.setdefault(major, []).append(minor)
        end = blocks[index + 1].start() if index + 1 < len(blocks) else len(area)
        pairs = re.findall(r"^  - ([A-Za-z ]+): (.*)$", area[match.end():end], re.M)
        data = dict(pairs)
        if len(data) != len(pairs) or not fields_required.issubset(data):
            raise ValueError(f"{identity}: missing or duplicate task fields")
        if any(not data[key].strip() for key in fields_required):
            raise ValueError(f"{identity}: empty task field")
        graph[identity] = [] if data["Depends"] == "none" else data["Depends"].split(", ")
        if "Method" in data:
            if data["Method"] in methods:
                raise ValueError("duplicate visual quality phase")
            methods[data["Method"]] = identity
        for key, known, used in (("Design", design_ids, used_design),
                                 ("Verification", test_ids, used_tests),
                                 ("Checks", set(checks), used_checks)):
            refs = data[key].split(", ")
            if any(ref not in known for ref in refs):
                raise ValueError(f"{identity}: unknown {key} reference")
            used.update(refs)
        for token in data["Requirements"].split(", "):
            span = re.fullmatch(r"R(\d+)\.(\d+)(?:-R(\d+)\.(\d+))?", token)
            if not span:
                raise ValueError(f"{identity}: invalid requirement reference {token}")
            group, start = int(span[1]), int(span[2])
            end_group = int(span[3]) if span[3] else group
            end = int(span[4]) if span[4] else start
            if group != end_group or end < start or end - start > len(expected):
                raise ValueError(f"{identity}: invalid requirement range")
            refs = {f"R{group}.{n}" for n in range(start, end + 1)}
            if not refs.issubset(expected):
                raise ValueError(f"{identity}: unknown requirement")
            covered.update(refs)
    if sorted(numbers) != [int(n) for n, _ in epics]:
        raise ValueError("tasks and epic headers differ")
    if any(ns != list(range(1, len(ns) + 1)) for ns in numbers.values()):
        raise ValueError("subtask IDs must be sequential within each epic")
    if covered != expected:
        raise ValueError(f"uncovered task criteria: {sorted(expected-covered)}")
    if used_design != design_ids or used_tests != test_ids or used_checks != set(checks):
        raise ValueError("uncovered design, verification or check anchors")
    if not {"impeccable critique + audit", "impeccable polish"}.issubset(methods):
        raise ValueError("missing Impeccable audit or refinement task")
    audit, polish = methods["impeccable critique + audit"], methods["impeccable polish"]
    if audit not in graph[polish] or polish not in graph.get("8.3", []):
        raise ValueError("final readiness must depend on ordered visual audit/refinement")
    visiting, visited = set(), set()

    def visit(node: str) -> None:
        if node not in graph:
            raise ValueError(f"unknown dependency {node}")
        if node in visiting:
            raise ValueError(f"dependency cycle at {node}")
        if node in visited:
            return
        visiting.add(node)
        for parent in graph[node]:
            visit(parent)
        visiting.remove(node)
        visited.add(node)

    for identity in ids:
        visit(identity)
    positions = {identity: index for index, identity in enumerate(ids)}
    if any(positions[parent] >= positions[node] for node in ids for parent in graph[node]):
        raise ValueError("task listing is not a valid topological order")
    return len(ids)


def self_test_tasks(requirements: str, design: str, tasks: str) -> None:
    def remove_first_criterion(match: re.Match) -> str:
        refs = []
        for token in match[1].split(", "):
            span = re.fullmatch(r"R(\d+)\.(\d+)(?:-R(\d+)\.(\d+))?", token)
            end = int(span[4]) if span[4] else int(span[2])
            refs.extend(f"R{span[1]}.{n}" for n in range(int(span[2]), end + 1))
        kept = [identity for identity in refs if identity != "R1.1"]
        return "  - Requirements: " + ", ".join(kept or ["R1.2"])

    uncovered = re.sub(r"^  - Requirements: (.+)$", remove_first_criterion, tasks, flags=re.M)
    mutants = {
        "uncovered task criterion": uncovered,
        "task dependency cycle": tasks.replace("  - Depends: none", "  - Depends: 1.1", 1),
        "unknown task check": re.sub(r"^  - Checks:.*$", "  - Checks: C999", tasks, count=1, flags=re.M),
        "missing task paths": re.sub(r"^  - Paths:.*\n", "", tasks, count=1, flags=re.M),
        "wrong task design hash": tasks.replace(hashlib.sha256(design.encode()).hexdigest(), "0"*64),
        "missing task checkbox": tasks.replace("- [ ] 1.1 ", "- 1.1 ", 1),
        "code-fenced checkboxes": tasks.replace("## Tasks\n", "## Tasks\n\n```text\n", 1).replace("## Execution Contract", "```\n\n## Execution Contract", 1),
        "buried checklist": tasks.replace("## Tasks", "## Check Registry", 1).replace("## Check Registry\n\nCommands", "## Tasks\n\nCommands", 1),
        "missing Impeccable audit": tasks.replace("  - Method: impeccable critique + audit\n", "", 1),
        "missing Impeccable refinement": tasks.replace("  - Method: impeccable polish\n", "", 1),
        "fixed Unix skill path": tasks + "\nSkill: /Users/example/skills/impeccable\n",
        "fixed Windows output path": tasks + "\nEvidence: C:\\fixed\\evidence\n",
        "fixed relative skill path": tasks + "\nSkill: .agents/skills/impeccable\n",
        "missing final visual gate": tasks.replace("  - Depends: 7.1, 7.2, 7.3, 7.5, 8.1, 8.2", "  - Depends: 7.1, 7.2, 7.3, 8.1, 8.2", 1),
    }
    for label, mutant in mutants.items():
        if mutant == tasks:
            raise ValueError(f"task mutant did not change the input: {label}")
        try:
            check_tasks(requirements, design, mutant)
        except ValueError:
            print(f"PASS: rejects {label}")
        else:
            raise ValueError(f"checker incorrectly accepted mutant: {label}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--design", action="store_true", help="check approved design traceability")
    parser.add_argument("--tasks", action="store_true", help="check task coverage and dependency DAG")
    args = parser.parse_args()
    root = Path(__file__).resolve().parent
    requirements = (root / "requirements.md").read_text()
    evaluations = (root / "evals.md").read_text()
    count, criteria = check(requirements, evaluations)
    print(f"PASS: {count} requirements, {criteria} unique EARS criteria, evaluation rows present")
    if args.self_test:
        self_test(requirements, evaluations)
    if args.design or args.tasks:
        state_file = root.parents[2] / ".workflow/ultracode/gpui-desktop/state.json"
        approval = json.loads(state_file.read_text())["approvals"]["requirements"]
        digest = hashlib.sha256(requirements.encode()).hexdigest()
        if approval["decision"] != "approved" or approval["sha256"] != digest:
            raise ValueError("requirements do not match their recorded approval")
        design = (root / "design.md").read_text()
        traced = check_design(requirements, design)
        print(f"PASS: {traced} approved criteria mapped to existing design/test anchors")
        if args.self_test:
            self_test_design(requirements, design)
        if args.tasks:
            state = json.loads(state_file.read_text())
            design_approval = state["approvals"]["design"]
            if design_approval["decision"] != "approved" or design_approval["sha256"] != hashlib.sha256(design.encode()).hexdigest():
                raise ValueError("design does not match its recorded approval")
            tasks = (root / "tasks.md").read_text()
            total = check_tasks(requirements, design, tasks)
            if state["implementation"] == "not_started" and re.search(r"^- \[x\]", tasks, re.M):
                raise ValueError("planning cannot mark unimplemented tasks complete")
            print(f"PASS: {total} owned/testable tasks, complete criteria/anchors, acyclic topological dependency graph")
            if args.self_test:
                self_test_tasks(requirements, design, tasks)
    print("Scope: structural checks only; no semantic, SMT or runtime proof")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError) as error:
        raise SystemExit(f"FAIL: {error}") from error
