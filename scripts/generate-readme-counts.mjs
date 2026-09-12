#!/usr/bin/env node
/**
 * Generate catalog counts and inject them into README.md.
 *
 * Counts:
 *   skills    = SKILL.md files under skills/ (recursive)
 *   agents    = metadata.json files under agents/ (recursive)
 *   providers = distinct `provider` values across agents metadata files
 *   roles     = keys under `.roles` in catalog/install-roles.json
 *   rules     = length of JSON array in catalog/rules.json
 *   mcp       = length of JSON array in catalog/mcp-references.json
 *
 * Mode:
 *   (default)  overwrite README.md in place
 *   --check    compare expected vs actual, exit 1 if stale, 0 if current
 *
 * Run: npm run readme-counts:write
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const readmePath = path.join(repoRoot, "README.md");
const check = process.argv.includes("--check");

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Recursively list all files under `dir`. Returns relative paths from `dir`. */
function listFiles(dir) {
  return fs.readdirSync(dir, { recursive: true }).map(String);
}

// ---------------------------------------------------------------------------
// Compute counts
// ---------------------------------------------------------------------------

const skillFiles = listFiles(path.join(repoRoot, "skills"));
const skillEntryFiles = skillFiles.filter((f) => path.basename(f) === "SKILL.md");
const skillCount = skillEntryFiles.length;
const skillsPerDirectory = new Map();
for (const f of skillEntryFiles) {
  const directory = f.split(path.sep)[0];
  skillsPerDirectory.set(directory, (skillsPerDirectory.get(directory) ?? 0) + 1);
}

const agentFiles = listFiles(path.join(repoRoot, "agents"));
const agentMetaFiles = agentFiles.filter((f) => f.endsWith("metadata.json"));
const agentCount = agentMetaFiles.length;

const allProviders = new Set();
const agentsPerProvider = new Map();
for (const f of agentMetaFiles) {
  const fullPath = path.join(repoRoot, "agents", f);
  const m = JSON.parse(fs.readFileSync(fullPath, "utf8"));
  if (m.provider) {
    allProviders.add(m.provider);
    agentsPerProvider.set(m.provider, (agentsPerProvider.get(m.provider) ?? 0) + 1);
  }
}
const providerCount = allProviders.size;

// Display labels for the generated provider-reference table. A provider with no
// entry here falls back to its title-cased slug, so adding a provider never
// breaks the build — the label is a nicety, not another registration point.
// (CLAUDE.md already lists eight places a new provider must be registered;
// this deliberately is not a ninth.)
const PROVIDER_LABELS = {
  aws: "🟧 Amazon Web Services",
  azure: "🟦 Microsoft Azure",
  oci: "🟥 Oracle Cloud Infrastructure",
  gcp: "🟩 Google Cloud Platform",
  alibaba: "🟠 Alibaba Cloud",
  huawei: "🔴 Huawei Cloud",
  ovhcloud: "☁️ OVHcloud",
  ionos: "🌐 IONOS Cloud",
  scaleway: "🇫🇷 Scaleway",
  hetzner: "🇩🇪 Hetzner Cloud",
  contabo: "💰 Contabo",
  kubernetes: "☸️ Kubernetes (cross-cloud)",
  kyverno: "🛡️ Kyverno (admission policy)",
  argocd: "🔄 Argo CD + Argo Rollouts (GitOps)",
  istio: "🕸️ Istio (service mesh)",
  cilium: "🐝 Cilium (network policy)",
  opentelemetry: "📡 OpenTelemetry (observability)",
  terraform: "🟩 Terraform (cross-cloud)",
  "multi-cloud": "💰 FinOps / multi-cloud",
  prometheus: "📊 Prometheus (alerting + cardinality)",
  falco: "🦅 Falco (runtime threat detection)",
  sigstore: "🔏 Sigstore / Cosign (supply chain)",
  "cert-manager": "🔐 cert-manager (PKI / cert lifecycle)",
  fluxcd: "🔄 FluxCD (GitOps)",
  backstage: "🎭 Backstage (IDP / developer platform)",
  marketing: "📣 Marketing governance",
  microsoft: "🟦 Microsoft 365 / Dynamics 365",
  sap: "🟨 SAP S/4HANA + BTP",
  netsuite: "🟫 Oracle NetSuite ERP",
  salesforce: "☁️ Salesforce",
  databricks: "🧱 Databricks",
  snowflake: "❄️ Snowflake",
  nvidia: "🟩 NVIDIA (GPU / AI platform)",
  frontend: "🎨 Frontend (web platform)",
  dotnet: "🟣 .NET",
  java: "☕ Java",
  kotlin: "🟧 Kotlin",
  php: "🐘 PHP",
  python: "🐍 Python",
  typescript: "🔷 TypeScript",
  accounting: "📒 Accounting",
  finance: "💹 Finance",
  legal: "⚖️ Legal",
  hr: "👥 HR",
  generic: "🧰 Generic (cross-platform)",
};

function providerLabel(slug) {
  if (PROVIDER_LABELS[slug]) return PROVIDER_LABELS[slug];
  return slug
    .split("-")
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

const rolesData = JSON.parse(
  fs.readFileSync(path.join(repoRoot, "catalog", "install-roles.json"), "utf8"),
);
const roleCount = Object.keys(rolesData.roles).length;

const rulesData = JSON.parse(
  fs.readFileSync(path.join(repoRoot, "catalog", "rules.json"), "utf8"),
);
const ruleCount = Array.isArray(rulesData) ? rulesData.length : 0;

const mcpData = JSON.parse(
  fs.readFileSync(path.join(repoRoot, "catalog", "mcp-references.json"), "utf8"),
);
const mcpCount = Array.isArray(mcpData) ? mcpData.length : 0;

const counts = {
  skills: skillCount,
  agents: agentCount,
  providers: providerCount,
  roles: roleCount,
  rules: ruleCount,
  mcp: mcpCount,
};

// ---------------------------------------------------------------------------
// Build the marker block
// ---------------------------------------------------------------------------

const markerBlock =
  `<!-- readme-counts:start -->\n` +
  `<!-- Generated by scripts/generate-readme-counts.mjs — do not edit by hand. Run: npm run readme-counts:write -->\n` +
  `| Catalog | Count |\n` +
  `| --- | --- |\n` +
  `| Skills | ${skillCount} |\n` +
  `| Agents | ${agentCount} |\n` +
  `| Providers | ${providerCount} |\n` +
  `| Install roles | ${roleCount} |\n` +
  `| Rules | ${ruleCount} |\n` +
  `| MCP references | ${mcpCount} |\n` +
  `<!-- readme-counts:end -->`;

// The provider-reference table is derived, never hand-maintained. It drifted
// badly while it was hand-written: it listed 27 of 45 providers, understated
// kubernetes and multi-cloud, and carried a `velero` row for a provider that
// has skills but no agents at all. Sorted by agent count descending, then slug,
// so the ordering is deterministic and the diff is stable.
const providerRows = [...agentsPerProvider.entries()]
  .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
  .map(([slug, n]) => `| \`${slug}\` | ${providerLabel(slug)} | ${n} |`)
  .join("\n");

const providerTableBlock =
  `<!-- provider-table:start -->\n` +
  `<!-- Generated by scripts/generate-readme-counts.mjs — do not edit by hand. Run: npm run readme-counts:write -->\n` +
  `| \`--provider\` value | Domain | 🔢 Agents in catalog |\n` +
  `| --- | --- | ---: |\n` +
  `${providerRows}\n` +
  `<!-- provider-table:end -->`;

// ---------------------------------------------------------------------------
// Transform README content
// ---------------------------------------------------------------------------

// Agent counts per DIRECTORY under agents/, which is not the same thing as per
// provider. agents/finops/ holds agents whose provider is kubernetes or multi-cloud,
// and agents/qa/ holds agents whose provider is generic; conversely `generic` and
// `multi-cloud` are provider values with no directory of their own. The README tree
// documents the directory layout, so it must be keyed on directories — keying it on
// provider would "correct" four currently-accurate lines into being wrong.
const agentsPerDirectory = new Map();
for (const entry of fs.readdirSync(path.join(repoRoot, "agents"), { withFileTypes: true })) {
  if (!entry.isDirectory()) continue;
  const dir = path.join(repoRoot, "agents", entry.name);
  let n = 0;
  for (const sub of fs.readdirSync(dir, { withFileTypes: true })) {
    if (sub.isDirectory() && fs.existsSync(path.join(dir, sub.name, "metadata.json"))) n++;
  }
  // A directory with no agents (agents/velero/ is a README-only leftover) is not part
  // of the tree, and must not be reported as missing from it.
  if (n > 0) agentsPerDirectory.set(entry.name, n);
}

/** Tree slugs naming a directory that holds no agents. */
const treeUnknownDirs = new Set();
/** Directories holding agents that the tree never lists. */
let treeMissingDirs = [];

/** Provider slugs referenced by a count:provider marker but absent from the catalog. */
const unknownProviders = new Set();
/** Skill-directory slugs referenced by a marker but absent from skills/. */
const unknownSkillDirectories = new Set();

function buildExpectedContent(original) {
  let content = original;

  // 1. Replace the marker block (markers preserved, inner content replaced)
  const markerStartRe = /<!-- readme-counts:start -->[\s\S]*?<!-- readme-counts:end -->/;
  if (markerStartRe.test(content)) {
    content = content.replace(markerStartRe, markerBlock);
  }

  // 1b. Replace the provider-reference table block, same marker contract.
  const providerTableRe = /<!-- provider-table:start -->[\s\S]*?<!-- provider-table:end -->/;
  if (providerTableRe.test(content)) {
    content = content.replace(providerTableRe, providerTableBlock);
  }

  // 2. Replace inline count spans <!-- count:KEY -->OLDNUMBER<!-- /count -->
  const inlineRe = /<!-- count:(skills|agents|providers|roles|rules|mcp) -->\d+<!-- \/count -->/g;
  content = content.replace(inlineRe, (_, key) => {
    return `<!-- count:${key} -->${counts[key]}<!-- /count -->`;
  });

  // 3. Per-provider agent counts: <!-- count:provider:SLUG -->N<!-- /count -->
  //
  // README's two narrative provider tables carry a count column alongside a
  // hand-written description. The descriptions are prose and stay hand-written
  // (CLAUDE.md), but the numbers next to them are catalog facts and drifted
  // exactly as you would expect: both the databricks and snowflake rows sat at
  // "3" for boards that had grown to 20 and 28 agents respectively, because
  // nothing checked them.
  //
  // This namespace is deliberately separate from generate-board-counts.mjs.
  // That generator owns `count:board:*` and `count:global:*`, and its TARGETS
  // list does NOT include README.md — `manifest:write:all` runs the two
  // generators concurrently (`&` … `wait`), so two writers on one file would
  // race. Keeping README single-owner is what makes that safe; the two marker
  // regexes are disjoint, so the split is enforceable rather than conventional.
  const providerRe =
    /<!--\s*count:provider:([a-z0-9-]+)\s*-->\d+<!--\s*\/count\s*-->/g;
  content = content.replace(providerRe, (match, slug) => {
    if (!agentsPerProvider.has(slug)) {
      // Fail closed. A typo'd or removed provider must not silently freeze at
      // whatever number happened to be typed there.
      unknownProviders.add(slug);
      return match;
    }
    return `<!-- count:provider:${slug} -->${agentsPerProvider.get(slug)}<!-- /count -->`;
  });

  // 3b. Per-directory skill counts: <!-- count:skill-directory:SLUG -->N<!-- /count -->
  //
  // Skill domains in the README follow the on-disk skills/<domain>/ layout,
  // not the metadata provider. This matters for cross-provider companions such
  // as skills/istio/istio-live-policy-change, whose provider remains kubernetes.
  const skillDirectoryRe =
    /<!--\s*count:skill-directory:([a-z0-9-]+)\s*-->\d+<!--\s*\/count\s*-->/g;
  content = content.replace(skillDirectoryRe, (match, slug) => {
    if (!skillsPerDirectory.has(slug)) {
      unknownSkillDirectories.add(slug);
      return match;
    }
    return `<!-- count:skill-directory:${slug} -->${skillsPerDirectory.get(slug)}<!-- /count -->`;
  });

  // 4. Repository-tree block: rewrite the agent count on each directory line.
  //
  // The tree lives inside a ```text fence, where an HTML comment marker would render
  // as visible text — so the markers wrap the fence and the counts are rewritten in
  // place instead. Descriptions are hand-written prose and are preserved verbatim;
  // only the number and its agent/agents pluralisation are generated.
  const treeRe = /<!-- agent-tree:start -->[\s\S]*?<!-- agent-tree:end -->/;
  const treeMatch = content.match(treeRe);
  if (treeMatch) {
    const seen = new Set();
    const rewritten = treeMatch[0].replace(
      /^([├└]── )([a-z0-9-]+)(\/\s*)\((\d+) (agents?)\b/gm,
      (match, branch, slug, sep, _n, _word) => {
        if (!agentsPerDirectory.has(slug)) {
          treeUnknownDirs.add(slug);
          return match;
        }
        seen.add(slug);
        const n = agentsPerDirectory.get(slug);
        return `${branch}${slug}${sep}(${n} ${n === 1 ? "agent" : "agents"}`;
      },
    );
    treeMissingDirs = [...agentsPerDirectory.keys()].filter((d) => !seen.has(d)).sort();
    content = content.replace(treeRe, rewritten);
  }

  return content;
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

const original = fs.readFileSync(readmePath, "utf8");
const requiredIstioMarkers = [
  ["Istio agent provider", /<!--\s*count:provider:istio\s*-->\d+<!--\s*\/count\s*-->/g],
  [
    "Istio skill directory",
    /<!--\s*count:skill-directory:istio\s*-->\d+<!--\s*\/count\s*-->/g,
  ],
];
for (const [label, marker] of requiredIstioMarkers) {
  const occurrences = original.match(marker)?.length ?? 0;
  if (occurrences !== 1) {
    process.stderr.write(
      `ERROR: README.md must contain exactly one ${label} count marker; found ${occurrences}.\n`,
    );
    process.exit(1);
  }
}
const expected = buildExpectedContent(original);

// Fail closed on a count:provider marker naming a provider the catalog does not
// have. Rewriting it is impossible and leaving it alone would freeze a stale
// number behind a marker that looks generated — the worst of both worlds.
if (treeUnknownDirs.size > 0 || treeMissingDirs.length > 0) {
  const parts = [];
  if (treeUnknownDirs.size > 0) {
    parts.push(
      `tree lists director(y|ies) with no agents: ${[...treeUnknownDirs].sort().join(", ")}`,
    );
  }
  if (treeMissingDirs.length > 0) {
    parts.push(`agents/ director(y|ies) missing from the tree: ${treeMissingDirs.join(", ")}`);
  }
  process.stderr.write(
    `ERROR: README.md repository tree is out of sync with agents/.\n  - ${parts.join("\n  - ")}\n` +
      `The counts are generated, but each line's description is hand-written — ` +
      `add or remove the line by hand, then re-run.\n`,
  );
  process.exit(1);
}

if (unknownProviders.size > 0) {
  process.stderr.write(
    `ERROR: README.md references unknown provider(s) in count:provider markers: ` +
      `${[...unknownProviders].sort().join(", ")}\n` +
      `Valid providers: ${[...agentsPerProvider.keys()].sort().join(", ")}\n`,
  );
  process.exit(1);
}

if (unknownSkillDirectories.size > 0) {
  process.stderr.write(
    `ERROR: README.md references unknown skill director(y|ies) in count:skill-directory markers: ` +
      `${[...unknownSkillDirectories].sort().join(", ")}\n` +
      `Valid directories: ${[...skillsPerDirectory.keys()].sort().join(", ")}\n`,
  );
  process.exit(1);
}

if (check) {
  if (original === expected) {
    console.log("OK: README counts current");
    process.exit(0);
  }

  // Print a basic diff-style report to stderr
  const origLines = original.split("\n");
  const expLines = expected.split("\n");
  const maxLen = Math.max(origLines.length, expLines.length);
  const diffLines = [];
  for (let i = 0; i < maxLen; i++) {
    const o = origLines[i];
    const e = expLines[i];
    if (o !== e) {
      diffLines.push(`Line ${i + 1}:`);
      if (o !== undefined) diffLines.push(`  - ${o}`);
      if (e !== undefined) diffLines.push(`  + ${e}`);
    }
  }
  process.stderr.write(
    `ERROR: README.md counts are stale. Run: npm run readme-counts:write\n\n` +
      diffLines.join("\n") +
      "\n",
  );
  process.exit(1);
} else {
  if (original === expected) {
    console.log("OK: README.md already up to date — no changes written.");
  } else {
    fs.writeFileSync(readmePath, expected, "utf8");
    console.log(
      `OK: README.md updated (skills=${skillCount}, agents=${agentCount}, ` +
        `providers=${providerCount}, roles=${roleCount}, rules=${ruleCount}, mcp=${mcpCount})`,
    );
  }
}
