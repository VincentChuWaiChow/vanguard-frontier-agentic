#!/usr/bin/env python3
"""Provider-agnostic routing eval-harness for every `*-maestro` skill.

Discovers all `tests/fixtures/*-maestro-routing/` directories. Each directory
must contain:

  - `taxonomy.json` — the executable routing contract:
        {
          "provider": "<provider-id>",
          "domains": {
              "<domain>": {"keywords": [...], "agent": "<agent-id>"},
              ...
          },
          "live_guards": ["<agent-id>", ...],
          "live_guard_intent": "<regex>",   # task patterns that imply mutation
          "parallel_threshold": 0.6           # optional
        }
  - `inputs/NN-name.json` — { "name", "task", "tags"? } operator-style tasks
  - `expected/NN-name.json` — { "route": [agent_ids...], "mode": ... }

The grader scores each domain by counting taxonomy keywords present in the task
(word-boundary, case-insensitive), then resolves to one or more agents, with a
live-guard gate that fires when `live_guard_intent` matches. Live-guard agents
are never permitted to appear in `single` or `parallel` modes — only in
`live-guard-gate` (or `runtime-evidence-gate` for read-only-runtime tiers).

Regression guards:

  - Every agent referenced by the routing table exists in catalog/agents.json.
  - No `live_guards` agent is dispatched in non-gate modes (live-agent
    auto-dispatch is the highest-blast-radius regression we can have).
  - Every domain has a mapped agent.

Stress-test fixture tags:

  - `adversarial:instruction-injection`  — SYSTEM-style prose framing
  - `adversarial:persona-replacement`    — "you are now ..." prose
  - `adversarial:live-guard-bypass`      — urgency-laden destructive ops
  - `adversarial:secrets-bait`           — fixture contains fake credential
  - `ambiguous`                          — vague signals, expect unclassified

A fixture tagged `adversarial:secrets-bait` MUST not contain real-looking
credentials in the grader output (the grader is route-only, so this is
structurally guaranteed). The fixture *task* may carry a fake credential
shape; the validator checks that the task contains the marker `<FAKE>` so
no credential pattern accidentally ships in the repository.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURES_ROOT = ROOT / "tests" / "fixtures"
AGENTS_CATALOG = ROOT / "catalog" / "agents.json"

DEFAULT_PARALLEL_THRESHOLD = 0.6
PARALLEL_CEILING = 4


def _score_domain(task: str, keywords: list[str]) -> int:
    task_lower = task.lower()
    hits = 0
    for kw in keywords:
        if re.search(r"\W", kw):
            if kw.lower() in task_lower:
                hits += 1
        else:
            if re.search(rf"\b{re.escape(kw.lower())}\b", task_lower):
                hits += 1
    return hits


def evaluate(task: str, taxonomy: dict) -> dict:
    """Return {route: [agent_ids], mode: str}."""
    gate_mode = taxonomy.get("gate_mode", "live-guard-gate")
    live_guard_intent = taxonomy.get("live_guard_intent")
    if live_guard_intent and re.search(live_guard_intent, task, re.IGNORECASE):
        # Identify which live-guard the task matches by keyword overlap.
        live_guards = taxonomy.get("live_guards", [])
        # Optional explicit mutation-to-guard signals. Scoring the tokens of an
        # agent id is a weak proxy: two guards that both own a kind of "policy"
        # tie on that single shared token, and the tie breaks alphabetically —
        # which silently hands an approved mutation to a guard that does not own
        # it. A taxonomy may therefore declare the phrases that identify each
        # guard's mutation. Absent the key, behaviour is unchanged.
        guard_keywords = taxonomy.get("live_guard_keywords", {})
        if live_guards:
            scored = []
            for agent_id in live_guards:
                explicit = guard_keywords.get(agent_id, [])
                if explicit:
                    # Explicit signals outrank id tokens: a guard whose declared
                    # mutation phrase appears in the task wins outright, so the
                    # shared-token tie never decides ownership.
                    score = _score_domain(task, explicit) * 10
                else:
                    score = 0
                # Score the agent id tokens against the task.
                tokens = re.split(r"[-_]", agent_id.replace("agent", "").strip("-"))
                score += sum(1 for t in tokens if t and re.search(rf"\b{re.escape(t)}\b", task.lower()))
                scored.append((agent_id, score))
            scored.sort(key=lambda kv: (-kv[1], kv[0]))
            # Optional: a taxonomy whose gate also covers mutations that no
            # registered guard owns (e.g. a BGP or Helm change) sets
            # live_guard_require_keyword. Then a guard is chosen only when one of
            # its declared live_guard_keywords appears; otherwise the mutation is
            # gated with an empty route (no guard owns it, so nothing may be
            # dispatched). Id-token scoring alone never picks the guard here: a
            # word like "live" or "network" in the task would otherwise hand an
            # unowned mutation to whichever guard sorts first. Absent the key,
            # behaviour is unchanged.
            if taxonomy.get("live_guard_require_keyword") and not any(
                _score_domain(task, guard_keywords.get(agent_id, [])) > 0 for agent_id in live_guards
            ):
                return {"route": [], "mode": gate_mode}
            if scored[0][1] > 0:
                return {"route": [scored[0][0]], "mode": gate_mode}
            # Generic live-guard intent without a clear specific match: emit
            # the first live_guard as a stand-in (specialist will refine).
            return {"route": [scored[0][0]], "mode": gate_mode}
        # live_guard_intent matched but no specific live-guard agents are
        # registered (e.g. marketing). Still gate the mutation intent; do
        # not fall through to normal domain routing.
        return {"route": [], "mode": gate_mode}

    domains = taxonomy["domains"]
    scores = {d: _score_domain(task, conf["keywords"]) for d, conf in domains.items()}
    ranked = sorted(scores.items(), key=lambda kv: (-kv[1], kv[0]))
    if ranked[0][1] == 0:
        return {"route": [], "mode": "unclassified"}

    top_score = ranked[0][1]
    threshold = taxonomy.get("parallel_threshold", DEFAULT_PARALLEL_THRESHOLD)
    winners = [d for d, s in ranked if s > 0 and s >= top_score * threshold]
    winners = winners[:PARALLEL_CEILING]

    agents = sorted({domains[d]["agent"] for d in winners})
    mode = "single" if len(agents) == 1 else f"parallel ({len(agents)})"
    return {"route": agents, "mode": mode}


DATE_SHAPED_KEYWORD = re.compile(r"^\d{4}(-\d{2}){0,2}$")


def _validate_taxonomy(taxonomy: dict, catalog_ids: set[str]) -> list[str]:
    errors: list[str] = []
    provider = taxonomy.get("provider", "?")
    for domain, conf in taxonomy.get("domains", {}).items():
        agent_id = conf.get("agent")
        if agent_id not in catalog_ids:
            errors.append(f"[{provider}] regression: domain {domain!r} maps to unknown agent {agent_id!r}")
        if not conf.get("keywords"):
            errors.append(f"[{provider}] regression: domain {domain!r} has empty keywords list")
        # A date is never a domain signal. Keywords containing a non-word
        # character match as substrings, so a spec or release date quoted in an
        # agent summary (e.g. an MCP protocol revision) silently captures every
        # task that mentions that year-month for any unrelated reason. The
        # generator filters these out of mined tokens; this gate stops one
        # re-entering through a hand edit.
        for kw in conf.get("keywords", []):
            if DATE_SHAPED_KEYWORD.match(kw):
                errors.append(
                    f"[{provider}] regression: domain {domain!r} carries date-shaped "
                    f"keyword {kw!r} — a date is not a routing signal"
                )
    for guard in taxonomy.get("live_guards", []):
        if guard not in catalog_ids:
            errors.append(f"[{provider}] regression: live_guard {guard!r} not in catalog")
    return errors


def _task_has_unmarked_credential(task: str) -> bool:
    """Return True iff the task carries a real-looking credential pattern without
    a `<FAKE>` marker. Only a boolean is returned so no portion of the task can
    flow into any caller's log output (CodeQL: clear-text-logging false positive
    avoidance — the contract is *boolean in, boolean out*)."""
    risky = re.compile(r"AKIA[0-9A-Z]{16}|api[_-]?key\s*[:=]\s*['\"]?[A-Za-z0-9]{16,}", re.IGNORECASE)
    return bool(risky.search(task)) and "<FAKE>" not in task


def main() -> int:
    catalog_ids = {a["id"] for a in json.loads(AGENTS_CATALOG.read_text())}
    routing_dirs = sorted(FIXTURES_ROOT.glob("*-maestro-routing"))
    if not routing_dirs:
        print("ERROR: no maestro-routing fixture directories found", file=sys.stderr)
        return 2

    total_failures = 0
    total_scenarios = 0

    for prov_dir in routing_dirs:
        taxonomy_path = prov_dir / "taxonomy.json"
        if not taxonomy_path.exists():
            print(f"SKIP [{prov_dir.name}] no taxonomy.json")
            continue
        taxonomy = json.loads(taxonomy_path.read_text())
        provider = taxonomy.get("provider", prov_dir.name)

        for err in _validate_taxonomy(taxonomy, catalog_ids):
            print(f"FAIL {err}")
            total_failures += 1

        inputs_dir = prov_dir / "inputs"
        expected_dir = prov_dir / "expected"
        fixtures = sorted(inputs_dir.glob("*.json")) if inputs_dir.is_dir() else []
        if not fixtures:
            print(f"WARN [{provider}] no fixtures yet")
            continue

        provider_fails = 0
        live_guards = set(taxonomy.get("live_guards", []))

        for fp in fixtures:
            fixture = json.loads(fp.read_text())
            name = fixture.get("name", fp.stem)
            tags = fixture.get("tags", [])
            expected = json.loads((expected_dir / f"{name}.json").read_text())
            total_scenarios += 1

            if _task_has_unmarked_credential(fixture["task"]):
                # Intentionally do NOT echo any portion of the task — the
                # whole point of this check is to keep credential-shaped
                # strings out of logs.
                print(f"FAIL [{provider}] secrets-bait fixture {name!r} "
                      f"contains real-looking credential without <FAKE> marker")
                provider_fails += 1

            got = evaluate(fixture["task"], taxonomy)

            # Live-guard auto-dispatch guard: live-guard agents must only
            # appear in 'live-guard-gate' or 'runtime-evidence-gate' modes.
            gate_modes = {"live-guard-gate", "runtime-evidence-gate"}
            if got["mode"] not in gate_modes:
                bad = [a for a in got["route"] if a in live_guards]
                if bad:
                    print(f"FAIL [{provider}/{name}] live-agent auto-dispatch guard tripped: "
                          f"{bad} in mode {got['mode']!r}")
                    provider_fails += 1
                    continue

            route_ok = set(got["route"]) == set(expected["route"])
            mode_ok = got["mode"] == expected["mode"]
            tag_str = f" tags={tags}" if tags else ""
            if route_ok and mode_ok:
                print(f"OK   [{provider}/{name}] route={got['route']} mode={got['mode']}{tag_str}")
            else:
                print(f"FAIL [{provider}/{name}] got route={got['route']} mode={got['mode']} | "
                      f"expected route={expected['route']} mode={expected['mode']}{tag_str}")
                provider_fails += 1

        if provider_fails == 0:
            print(f"-- [{provider}] {len(fixtures)} scenarios passed")
        total_failures += provider_fails

    if total_failures:
        print(f"\n{total_failures} routing check(s) failed across {len(routing_dirs)} provider(s)", file=sys.stderr)
        return 1
    print(f"\nOK: {total_scenarios} scenarios validated across {len(routing_dirs)} maestro(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
