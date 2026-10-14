#!/usr/bin/env node
/**
 * Generate catalog counts and inject them into the documentation that states them.
 *
 * Several docs quote agent, skill, provider, gate, and maestro totals. Typed by
 * hand they go stale silently — a board gains an agent, a gate is added, and the
 * prose keeps asserting last month's number with nothing to catch it. This
 * generator derives every one of them from the repository itself and rewrites
 * the marker spans, the same contract `generate-readme-counts.mjs` applies to
 * README.md.
 *
 * Literal numbers are written into the markdown rather than Liquid variables:
 * these files have no Jekyll front matter and several are read as raw markdown
 * on GitHub, where `{{ site.data... }}` would render as literal braces.
 *
 * Two marker namespaces, both replacing only the number between the markers:
 *
 *   <!-- count:board:snowflake:agents -->28<!-- /count -->     per provider
 *   <!-- count:global:gates -->25<!-- /count -->               repo-wide
 *
 * A board key may span providers by joining them with `+`, for a figure that is
 * genuinely about a pair of boards rather than either one:
 *
 *   <!-- count:board:legal+hr:agents -->28<!-- /count -->
 *
 * Board keys, derived per provider. Deprecated assets are separated out FIRST
 * and the tier keys then describe only the live board, so the parts sum to the
 * whole without double counting — a deprecated live guard is `deprecated`, not
 * `guards`:
 *
 *   agents      every agent for the provider, deprecated included
 *   deprecated  agents whose metadata sets lifecycle: deprecated
 *   skills      every skill for the provider
 *   router      non-deprecated `*-maestro-agent` agents, whatever their tier
 *   static      non-deprecated static-review agents
 *   review      non-deprecated static-review agents that are NOT the router
 *   readonly    non-deprecated read-only-runtime agents
 *   guards      non-deprecated mutating-runtime agents
 *   live        readonly + guards — the live control plane
 *   specialists non-deprecated, non-router agents regardless of declared tier
 *
 * `review` is computed by excluding maestros from the static-review set rather
 * than by subtracting the router count, because a maestro is not always a
 * static-review agent: across this repo maestros are variously static-review,
 * read-only-runtime, or untiered. Subtracting would under-count the specialists
 * on every board whose router is not static-review.
 *
 * Global keys: agents, skills, providers, roles, gates, maestros, scenarios, rules, mcp.
 *
 * Unknown providers and unknown keys are hard errors: a typo in a marker must
 * fail the gate rather than silently keep a stale number.
 *
 * Mode:
 *   (default)  rewrite the target documents in place
 *   --check    compare expected vs actual, exit 1 if stale, 0 if current
 *
 * Run: npm run board-counts:write
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const check = process.argv.includes("--check");

// Documents this generator owns. README.md is deliberately absent: it has its
// own generator and its own `count:<key>` namespace. Changelogs, ADRs, and
// dated plan/research documents are also absent on purpose — they record what
// was true when written, and rewriting their numbers would falsify the record.
const TARGETS = [
  "docs/language-stack-boards.md",
  "docs/configuration.md",
  "docs/marketplace-model.md",
  "docs/databricks-board.md",
  "tests/fixtures/README.md",
  "index.md",
  "agents/README.md",
  "agents/AGENTS.md",
];

// README.md is deliberately NOT a target. `manifest:write:all` runs this generator and
// generate-readme-counts.mjs concurrently (`&` … `wait`), so two writers on one file would
// race. README keeps a single owner; its per-provider figures use that generator's own
// `count:provider:<slug>` namespace, which this generator's marker regex cannot match.

// ---------------------------------------------------------------------------
// Collect stats from the repository
// ---------------------------------------------------------------------------

/** Read every metadata.json two levels under `root` (e.g. agents/<provider>/<asset>/). */
function readMetadata(root) {
  const out = [];
  const base = path.join(repoRoot, root);
  if (!fs.existsSync(base)) return out;
  for (const provider of fs.readdirSync(base)) {
    const providerDir = path.join(base, provider);
    if (!fs.statSync(providerDir).isDirectory()) continue;
    for (const asset of fs.readdirSync(providerDir)) {
      const metaPath = path.join(providerDir, asset, "metadata.json");
      if (!fs.existsSync(metaPath)) continue;
      out.push(JSON.parse(fs.readFileSync(metaPath, "utf8")));
    }
  }
  return out;
}

const agentMeta = readMetadata("agents");
const skillMeta = readMetadata("skills");

const BOARD_KEYS = [
  "agents", "deprecated", "skills",
  "router", "static", "review", "readonly", "guards", "live", "specialists",
];

/** @type {Map<string, Record<string, number>>} */
const stats = new Map();

function bucket(provider) {
  if (!stats.has(provider)) {
    stats.set(provider, Object.fromEntries(BOARD_KEYS.map((k) => [k, 0])));
  }
  return stats.get(provider);
}

for (const m of agentMeta) {
  if (!m.provider) continue;
  const s = bucket(m.provider);
  s.agents += 1;

  // Deprecated first, and exclusively: a deprecated live guard is not part of
  // the live board, so counting it under `guards` would both overstate the
  // mutation surface and break the sum against `agents`.
  if (m.lifecycle === "deprecated") {
    s.deprecated += 1;
    continue;
  }

  const isRouter = typeof m.id === "string" && m.id.endsWith("-maestro-agent");
  if (isRouter) s.router += 1;
  // Every non-router agent on the live board, whatever tier it declares — or
  // none at all. Many boards predate the tier field, so `review` (which counts
  // only declared static-review agents) reads 0 for them; `specialists` is the
  // figure those boards actually mean.
  else s.specialists += 1;

  if (m.execution_tier === "static-review") {
    s.static += 1;
    // Specialists are the static-review agents that are not the router. Derived
    // by exclusion, not by subtracting `router`, because a router is not always
    // static-review — see the header note.
    if (!isRouter) s.review += 1;
  } else if (m.execution_tier === "read-only-runtime") {
    s.readonly += 1;
  } else if (m.execution_tier === "mutating-runtime") {
    s.guards += 1;
  }
}

for (const m of skillMeta) {
  if (!m.provider) continue;
  bucket(m.provider).skills += 1;
}

for (const s of stats.values()) s.live = s.readonly + s.guards;

// ---------------------------------------------------------------------------
// Repo-wide figures
// ---------------------------------------------------------------------------

function jsonLen(rel, pick) {
  const p = path.join(repoRoot, rel);
  if (!fs.existsSync(p)) return 0;
  const parsed = JSON.parse(fs.readFileSync(p, "utf8"));
  const v = pick ? pick(parsed) : parsed;
  return Array.isArray(v) ? v.length : Object.keys(v).length;
}

const pkg = JSON.parse(fs.readFileSync(path.join(repoRoot, "package.json"), "utf8"));
// Gates = the distinct validate:* scripts the `validate` chain actually runs.
// Counting the chain rather than every validate:* key means a script that
// exists but was never wired in is not advertised as an enforced gate.
const gateCount = new Set(
  (pkg.scripts?.validate ?? "").match(/validate:[a-z0-9-]+/g) ?? [],
).size;

const fixturesDir = path.join(repoRoot, "tests", "fixtures");
const maestroCount = fs.existsSync(fixturesDir)
  ? fs.readdirSync(fixturesDir).filter((d) => d.endsWith("-maestro-routing")).length
  : 0;

// Scenarios = what validate-maestro-routing.py counts: every inputs/*.json in a
// *-maestro-routing directory that has a taxonomy.json (it skips the rest).
const scenarioCount = fs.existsSync(fixturesDir)
  ? fs
      .readdirSync(fixturesDir)
      .filter((d) => d.endsWith("-maestro-routing"))
      .filter((d) => fs.existsSync(path.join(fixturesDir, d, "taxonomy.json")))
      .map((d) => path.join(fixturesDir, d, "inputs"))
      .filter((dir) => fs.existsSync(dir))
      .reduce(
        (sum, dir) => sum + fs.readdirSync(dir).filter((f) => f.endsWith(".json")).length,
        0,
      )
  : 0;

const globals = {
  agents: agentMeta.length,
  skills: skillMeta.length,
  providers: new Set(agentMeta.map((m) => m.provider).filter(Boolean)).size,
  roles: jsonLen("catalog/install-roles.json", (d) => d.roles ?? d),
  gates: gateCount,
  maestros: maestroCount,
  scenarios: scenarioCount,
  rules: jsonLen("catalog/rules.json"),
  mcp: jsonLen("catalog/mcp-references.json"),
};

// Agent counts per DIRECTORY under agents/, which is deliberately NOT the same as per
// provider. agents/finops/ holds 4 agents whose provider is kubernetes or multi-cloud, so
// `finops` has a directory but no provider entry at all; agents/qa/ holds agents whose
// provider is generic. Navigation headings in agents/AGENTS.md link to a DIRECTORY, so a
// provider-keyed marker would either report the wrong number (kubernetes: 15 on disk, 16
// by provider) or fail closed on a directory that is not a provider (finops). Hence a
// third scope.
const dirStats = new Map();
{
  const agentsRoot = path.join(repoRoot, "agents");
  for (const entry of fs.readdirSync(agentsRoot, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const dir = path.join(agentsRoot, entry.name);
    let n = 0;
    for (const sub of fs.readdirSync(dir, { withFileTypes: true })) {
      if (sub.isDirectory() && fs.existsSync(path.join(dir, sub.name, "metadata.json"))) n++;
    }
    if (n > 0) dirStats.set(entry.name, n);
  }
}

// ---------------------------------------------------------------------------
// Rewrite the marker spans
// ---------------------------------------------------------------------------

const markerRe =
  /<!-- count:(board:([a-z0-9+-]+)|dir:([a-z0-9-]+)|global):([a-z]+) -->(\d+)<!-- \/count -->/g;

const errors = [];
const pending = [];
let seen = 0;

for (const rel of TARGETS) {
  const file = path.join(repoRoot, rel);
  if (!fs.existsSync(file)) {
    errors.push(`target document missing: ${rel}`);
    continue;
  }
  const original = fs.readFileSync(file, "utf8");

  const updated = original.replace(markerRe, (match, scope, providerList, dirName, key, current) => {
    seen += 1;
    let expected;

    if (scope.startsWith("dir:")) {
      if (key !== "agents") {
        errors.push(`${rel}: unknown dir key "${key}" in count:${scope}:${key} — only "agents" is defined`);
        return match;
      }
      if (!dirStats.has(dirName)) {
        errors.push(
          `${rel}: unknown directory "${dirName}" in count:${scope}:${key} — ` +
            `no agents/${dirName}/*/metadata.json found`,
        );
        return match;
      }
      expected = String(dirStats.get(dirName));
    } else if (scope === "global") {
      if (!(key in globals)) {
        errors.push(`${rel}: unknown global key "${key}" — valid: ${Object.keys(globals).sort().join(", ")}`);
        return match;
      }
      expected = String(globals[key]);
    } else {
      if (!BOARD_KEYS.includes(key)) {
        errors.push(`${rel}: unknown board key "${key}" — valid: ${BOARD_KEYS.slice().sort().join(", ")}`);
        return match;
      }
      const providers = providerList.split("+");
      let total = 0;
      let bad = false;
      for (const p of providers) {
        const s = stats.get(p);
        if (!s) {
          errors.push(`${rel}: unknown provider "${p}" in count:board:${providerList}:${key} — no agents or skills found on disk for it`);
          bad = true;
          continue;
        }
        total += s[key];
      }
      if (bad) return match;
      expected = String(total);
    }

    if (check && expected !== current) {
      errors.push(`${rel}: stale count:${scope}:${key} — file says ${current}, repository says ${expected}`);
    }
    return `<!-- count:${scope}:${key} -->${expected}<!-- /count -->`;
  });

  if (updated !== original) pending.push([file, rel, updated]);
}

if (errors.length) {
  console.error("FAIL: generated documentation counts");
  for (const e of errors) console.error(`  - ${e}`);
  if (check) console.error("\nRun `npm run board-counts:write` to refresh.");
  process.exit(1);
}

if (seen === 0) {
  console.error("FAIL: no count:board:*, count:dir:* or count:global:* markers found in any target document.");
  console.error("      The generator is wired in but the documents no longer carry markers,");
  console.error("      which means their counts are unguarded. Restore the markers.");
  process.exit(1);
}

if (check) {
  console.log(`OK: documentation counts current (${seen} marker(s) across ${TARGETS.length} document(s))`);
  process.exit(0);
}

for (const [file, rel, content] of pending) {
  fs.writeFileSync(file, content);
  console.log(`  updated ${rel}`);
}
console.log(
  pending.length
    ? `OK: documentation counts written (${seen} marker(s), ${pending.length} document(s) changed)`
    : `OK: documentation counts already up to date (${seen} marker(s))`,
);
