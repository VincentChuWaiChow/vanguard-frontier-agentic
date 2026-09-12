#!/usr/bin/env node
/**
 * Standalone validator: fails CI if README.md count markers have drifted
 * from the real computed values derived from disk.
 *
 * Count contract:
 *   skills    = files matching skills/STAR-STAR/SKILL.md  (recursive)
 *   agents    = files matching agents/STAR-STAR/metadata.json  (recursive)
 *   providers = distinct `provider` values across all agents/STAR-STAR/metadata.json
 *   roles     = number of keys under .roles in catalog/install-roles.json
 *   rules     = length of JSON array in catalog/rules.json
 *   mcp       = length of JSON array in catalog/mcp-references.json
 *   provider agents = metadata provider values under agents/
 *   domain skills   = SKILL.md files under each skills/<domain>/ directory
 *
 * Run: node tests/validate-readme-counts.mjs
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Recursively collect files whose basename matches `basename`. */
function findFiles(dir, basename) {
  const results = [];
  if (!fs.existsSync(dir)) return results;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      results.push(...findFiles(full, basename));
    } else if (entry.isFile() && entry.name === basename) {
      results.push(full);
    }
  }
  return results;
}

// ---------------------------------------------------------------------------
// Compute counts from disk
// ---------------------------------------------------------------------------

const skillFiles = findFiles(path.join(repoRoot, "skills"), "SKILL.md");
const computedSkills = skillFiles.length;
const skillsPerDirectory = new Map();
for (const f of skillFiles) {
  const directory = path.relative(path.join(repoRoot, "skills"), f).split(path.sep)[0];
  skillsPerDirectory.set(directory, (skillsPerDirectory.get(directory) ?? 0) + 1);
}

const agentMetaFiles = findFiles(path.join(repoRoot, "agents"), "metadata.json");
const computedAgents = agentMetaFiles.length;

const providerSet = new Set();
const agentsPerProvider = new Map();
for (const f of agentMetaFiles) {
  try {
    const data = JSON.parse(fs.readFileSync(f, "utf8"));
    if (typeof data.provider === "string" && data.provider.length > 0) {
      providerSet.add(data.provider);
      agentsPerProvider.set(data.provider, (agentsPerProvider.get(data.provider) ?? 0) + 1);
    }
  } catch (error) {
    process.stderr.write(`FAIL [readme-counts] malformed agent metadata ${f}: ${error.message}\n`);
    process.exit(1);
  }
}
const computedProviders = providerSet.size;

const rolesDoc = JSON.parse(
  fs.readFileSync(path.join(repoRoot, "catalog", "install-roles.json"), "utf8")
);
const computedRoles = Object.keys(rolesDoc.roles ?? {}).length;

const rulesArr = JSON.parse(
  fs.readFileSync(path.join(repoRoot, "catalog", "rules.json"), "utf8")
);
const computedRules = Array.isArray(rulesArr) ? rulesArr.length : 0;

const mcpArr = JSON.parse(
  fs.readFileSync(path.join(repoRoot, "catalog", "mcp-references.json"), "utf8")
);
const computedMcp = Array.isArray(mcpArr) ? mcpArr.length : 0;

/** Map from canonical KEY (as used in inline spans) → computed value. */
const computed = {
  skills: computedSkills,
  agents: computedAgents,
  providers: computedProviders,
  roles: computedRoles,
  rules: computedRules,
  mcp: computedMcp,
};

// ---------------------------------------------------------------------------
// Read README.md
// ---------------------------------------------------------------------------

const readmePath = path.join(repoRoot, "README.md");
const readme = fs.readFileSync(readmePath, "utf8");

// ---------------------------------------------------------------------------
// Extract and validate marker block
// ---------------------------------------------------------------------------

const blockMatch = readme.match(
  /<!--\s*readme-counts:start\s*-->([\s\S]*?)<!--\s*readme-counts:end\s*-->/
);

if (!blockMatch) {
  process.stderr.write(
    "FAIL [readme-counts] marker block not found in README.md\n"
  );
  process.exit(1);
}

const blockContent = blockMatch[1];

/**
 * Map from table label → canonical KEY used in `computed`.
 * Keys are lower-cased for case-insensitive matching.
 */
const labelToKey = {
  "skills": "skills",
  "agents": "agents",
  "providers": "providers",
  "install roles": "roles",
  "rules": "rules",
  "mcp references": "mcp",
};

const failures = [];

const requiredIstioMarkers = [
  ["Istio agent provider", /<!--\s*count:provider:istio\s*-->\d+<!--\s*\/count\s*-->/g],
  [
    "Istio skill directory",
    /<!--\s*count:skill-directory:istio\s*-->\d+<!--\s*\/count\s*-->/g,
  ],
];
for (const [label, marker] of requiredIstioMarkers) {
  const occurrences = readme.match(marker)?.length ?? 0;
  if (occurrences !== 1) {
    failures.push(
      `FAIL [readme-counts] README must contain exactly one ${label} count marker; found ${occurrences}`
    );
  }
}

// Parse table rows: | Label | Number |
// Tolerate extra whitespace around cell content.
const tableRowRe = /\|\s*([^|]+?)\s*\|\s*(\d+)\s*\|/g;
let rowMatch;
while ((rowMatch = tableRowRe.exec(blockContent)) !== null) {
  const rawLabel = rowMatch[1].trim().toLowerCase();
  const shownValue = parseInt(rowMatch[2], 10);
  const key = labelToKey[rawLabel];
  if (key === undefined) continue; // header row or unknown label — skip
  const computedValue = computed[key];
  if (shownValue !== computedValue) {
    failures.push(
      `FAIL [readme-counts] ${rowMatch[1].trim()}: README shows ${shownValue}, computed ${computedValue}`
    );
  }
}

// ---------------------------------------------------------------------------
// Find and validate all inline count spans
// ---------------------------------------------------------------------------

// <!-- count:KEY -->NUMBER<!-- /count -->
const spanRe = /<!--\s*count:(skills|agents|providers|roles|rules|mcp)\s*-->(\d+)<!--\s*\/count\s*-->/g;
let spanMatch;
while ((spanMatch = spanRe.exec(readme)) !== null) {
  const key = spanMatch[1];
  const shownValue = parseInt(spanMatch[2], 10);
  const computedValue = computed[key];
  if (shownValue !== computedValue) {
    failures.push(
      `FAIL [readme-counts] ${key} (inline span): README shows ${shownValue}, computed ${computedValue}`
    );
  }
}

const providerSpanRe =
  /<!--\s*count:provider:([a-z0-9-]+)\s*-->(\d+)<!--\s*\/count\s*-->/g;
while ((spanMatch = providerSpanRe.exec(readme)) !== null) {
  const slug = spanMatch[1];
  const shownValue = parseInt(spanMatch[2], 10);
  const computedValue = agentsPerProvider.get(slug);
  if (computedValue === undefined) {
    failures.push(`FAIL [readme-counts] unknown agent provider marker: ${slug}`);
  } else if (shownValue !== computedValue) {
    failures.push(
      `FAIL [readme-counts] provider ${slug}: README shows ${shownValue}, computed ${computedValue}`
    );
  }
}

const skillDirectorySpanRe =
  /<!--\s*count:skill-directory:([a-z0-9-]+)\s*-->(\d+)<!--\s*\/count\s*-->/g;
while ((spanMatch = skillDirectorySpanRe.exec(readme)) !== null) {
  const slug = spanMatch[1];
  const shownValue = parseInt(spanMatch[2], 10);
  const computedValue = skillsPerDirectory.get(slug);
  if (computedValue === undefined) {
    failures.push(`FAIL [readme-counts] unknown skill directory marker: ${slug}`);
  } else if (shownValue !== computedValue) {
    failures.push(
      `FAIL [readme-counts] skill directory ${slug}: README shows ${shownValue}, computed ${computedValue}`
    );
  }
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

if (failures.length > 0) {
  for (const line of failures) {
    process.stderr.write(line + "\n");
  }
  process.exit(1);
}

console.log(
  `OK: README counts match catalog (skills=${computedSkills} agents=${computedAgents} ` +
  `providers=${computedProviders} roles=${computedRoles} rules=${computedRules} mcp=${computedMcp})`
);
process.exit(0);
