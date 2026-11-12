#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { parseArgs as utilParseArgs } from "node:util";
import { createHash } from "node:crypto";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const MAX_PLAN_BYTES = 8 * 1024 * 1024;
const MAX_FILE_BYTES = 64 * 1024 * 1024;
const MAX_RENDER_BYTES = 256 * 1024 * 1024;
const MAX_FILES = 10000;
const sourceInputs = new Map();
const validId = (value) => typeof value === "string" && /^[a-z0-9][a-z0-9-]*$/.test(value);
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const relativePath = (root, file) => path.relative(root, file).split(path.sep).join("/");

// Protocol canonicalization: recursive object-key UTF-16 ordering, array order
// retained, compact JSON UTF-8. The digest excludes only the top-level digest.
function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function identity(stat) {
  return {
    device: String(stat.dev), inode: String(stat.ino), size: String(stat.size),
    mode: Number(stat.mode & 0o7777n), mtime_ns: String(stat.mtimeNs), links: String(stat.nlink),
  };
}

// Inspect every existing component, including ancestors outside the selected
// root. Missing suffixes are valid; links and non-directory ancestors are not.
// This is preflight, not protection against a malicious same-user filesystem race.
function checkPath(file, kind, label) {
  const absolute = path.resolve(file);
  const root = path.parse(absolute).root;
  const parts = absolute.slice(root.length).split(path.sep).filter(Boolean);
  let cursor = root;
  let anchor = { path: root, identity: identity(fs.lstatSync(root, { bigint: true })) };
  for (let index = 0; index < parts.length; index += 1) {
    cursor = path.join(cursor, parts[index]);
    let stat;
    try { stat = fs.lstatSync(cursor, { bigint: true }); }
    catch (error) {
      if (error.code !== "ENOENT") throw error;
      return { exists: false, identity: null, anchor, missing: parts.slice(index) };
    }
    if (stat.isSymbolicLink()) throw new Error(`Refusing to ${label}: symbolic link ${cursor}`);
    const last = index === parts.length - 1;
    if ((!last || kind === "directory") && !stat.isDirectory()) throw new Error(`Refusing to ${label}: non-directory path ${cursor}`);
    if (last && kind === "file" && !stat.isFile()) throw new Error(`Refusing to ${label}: non-regular file ${cursor}`);
    anchor = { path: cursor, identity: identity(stat) };
  }
  return { exists: true, identity: anchor.identity, anchor, missing: [] };
}

function readSource(file, input = false) {
  assertWithin(repoRoot, file, "read source");
  const state = checkPath(file, "file", "read source");
  if (!state.exists) throw new Error(`Missing source file: ${file}`);
  if (Number(state.identity.size) > MAX_FILE_BYTES) throw new Error(`PLAN_LIMIT: source file exceeds 64 MiB: ${file}`);
  const bytes = fs.readFileSync(file);
  const after = checkPath(file, "file", "read source");
  if (canonicalJson(state.identity) !== canonicalJson(after.identity)) throw new Error(`Source changed while reading: ${file}`);
  const record = { path: relativePath(repoRoot, file), identity: state.identity, sha256: sha256(bytes) };
  if (input) {
    sourceInputs.set(record.path, record);
    if (sourceInputs.size > MAX_FILES) throw new Error(`PLAN_LIMIT: source manifest exceeds ${MAX_FILES} inputs.`);
  }
  return { bytes, record };
}

function destinationState(file, skill = false) {
  // Retain the established skill-link diagnostic for legacy callers.
  const state = checkPath(file, "file", skill ? "write to symbolic link destination in skill tree" : "write destination");
  // Truncating a multiply-linked file also changes paths outside this plan,
  // including source metadata/executables or unrelated external sentinels.
  if (state.exists && state.identity.links !== "1") {
    throw new Error(`Destination collision: aliased hard-link file ${file}`);
  }
  if (state.exists && Number(state.identity.size) > MAX_FILE_BYTES) throw new Error(`PLAN_LIMIT: destination file exceeds 64 MiB: ${file}`);
  const digest = state.exists ? sha256(fs.readFileSync(file)) : null;
  const after = checkPath(file, "file", "inspect destination");
  if (canonicalJson(state.identity) !== canonicalJson(after.identity)) throw new Error(`Destination changed while reading: ${file}`);
  return { exists: state.exists, identity: state.identity, sha256: digest };
}

// Render the exact final buffer once. Identity artifacts remain binary-safe;
// Codex's historical UTF-8/newline behavior is retained, without an intermediate
// unrewritten destination file. copyFile historically drops setuid/setgid bits.
function renderArtifact(source, platform, kind, targetRoot) {
  const { bytes, record } = readSource(source);
  let content = bytes;
  const transformation = { id: "identity/v1", options: {} };
  if (platform === "codex" && kind === "agent") {
    const original = bytes.toString("utf8");
    const rewritten = original.replace(
      /^path = "skills\/[^"\n]+\/([^/"\n]+)\/SKILL\.md"$/gm,
      (_match, skillName) => `path = ${JSON.stringify(path.join(targetRoot, SKILLS_PLATFORM_CONFIG.codex, skillName))}`
    );
    if (rewritten !== original) content = Buffer.from(rewritten, "utf8");
    transformation.id = "codex-skill-paths/v1";
    transformation.options = { targetRoot, skillsDirectory: SKILLS_PLATFORM_CONFIG.codex };
  }
  return {
    content, source: record, transformation,
    final: { sha256: sha256(content), size: content.length, mode: record.identity.mode & 0o1777 },
  };
}

function writeArtifact(artifact) {
  const file = artifact.file;
  const source = path.join(repoRoot, file.source.path);
  const currentSource = readSource(source).record;
  if (canonicalJson(currentSource) !== canonicalJson(file.source)) throw new Error(`Source changed before write: ${source}`);
  const destination = file.destination.path;
  const currentDestination = destinationState(destination, file.kind === "skill");
  const expected = file.destination;
  if (canonicalJson(currentDestination) !== canonicalJson({ exists: expected.exists, identity: expected.identity, sha256: expected.sha256 })) {
    throw new Error(`Destination changed before write: ${destination}`);
  }
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  checkPath(destination, "file", "write destination");
  const flags = fs.constants.O_WRONLY | fs.constants.O_CREAT |
    (file.action === "create" ? fs.constants.O_EXCL : fs.constants.O_TRUNC) |
    (fs.constants.O_NOFOLLOW ?? 0);
  const descriptor = fs.openSync(destination, flags, file.final.mode);
  try {
    fs.writeFileSync(descriptor, artifact.content);
    fs.fchmodSync(descriptor, file.final.mode);
  } finally {
    fs.closeSync(descriptor);
  }
}


const PLATFORM_CONFIG = {
  codex: {
    variants: [["codex", ".codex/agents", ".toml"]],
  },
  copilot: {
    variants: [["copilot", ".github/agents", ".agent.md"]],
  },
  "claude-code": {
    variants: [["claude-code", ".claude/agents", ".md"]],
  },
  cursor: {
    variants: [["cursor", ".cursor/agents", ".md"]],
  },
  gemini: {
    variants: [["gemini", ".gemini/agents", ".md"]],
  },
  "kiro-ide": {
    variants: [["kiro-ide", ".kiro/agents", ".md"]],
  },
  "kiro-cli": {
    variants: [["kiro-cli", ".kiro/agents", ".json"]],
  },
  kiro: {
    variants: [
      ["kiro-ide", ".kiro/agents", ".md"],
      ["kiro-cli", ".kiro/agents", ".json"],
    ],
  },
};

const PLATFORM_ALIASES = {
  claude: "claude-code",
  kiroide: "kiro-ide",
  kirocli: "kiro-cli",
};

const SKILLS_PLATFORM_CONFIG = {
  codex: ".codex/skills",
  "claude-code": ".claude/skills",
  copilot: ".github/skills",
  gemini: ".gemini/skills",
};

/**
 * Platforms that will NEVER support skill bundling because they have no native
 * skill primitive. The value is an explicit notice that replaces the generic
 * "not yet supported" fallback for these platforms.
 *
 * Design rationale: docs/cross-harness-skills.md
 *   Cursor — uses Project Rules (.cursor/rules/*.mdc), not skills.
 *   Kiro   — uses Steering files (.kiro/steering/*.md), not skills.
 * Both mismatches are large enough that skill export is intentionally omitted
 * as a permanent design decision, not a pending TODO.
 */
const SKIP_SKILLS_PLATFORM_NOTICES = {
  cursor:
    "[vfa] Skill export is not supported on Cursor. Cursor uses Project Rules " +
    "(.cursor/rules/*.mdc), not skills. The semantics (style guides, glob-based " +
    "triggers) differ significantly from our multi-section operating playbooks; " +
    "this is a permanent design decision, not a pending TODO. " +
    "See docs/cross-harness-skills.md for the full rationale.\n",
  kiro:
    "[vfa] Skill export is not supported on Kiro. Kiro uses Steering files " +
    "(.kiro/steering/*.md), not skills. Steering is single-file guidance with " +
    "plural-by-default inclusion; our SKILL packages bundle scripts/ and " +
    "references/ siblings that Steering cannot accommodate. " +
    "This is a permanent design decision, not a pending TODO. " +
    "See docs/cross-harness-skills.md for the full rationale.\n",
  "kiro-ide":
    "[vfa] Skill export is not supported on Kiro. Kiro uses Steering files " +
    "(.kiro/steering/*.md), not skills. Steering is single-file guidance with " +
    "plural-by-default inclusion; our SKILL packages bundle scripts/ and " +
    "references/ siblings that Steering cannot accommodate. " +
    "This is a permanent design decision, not a pending TODO. " +
    "See docs/cross-harness-skills.md for the full rationale.\n",
  "kiro-cli":
    "[vfa] Skill export is not supported on Kiro. Kiro uses Steering files " +
    "(.kiro/steering/*.md), not skills. Steering is single-file guidance with " +
    "plural-by-default inclusion; our SKILL packages bundle scripts/ and " +
    "references/ siblings that Steering cannot accommodate. " +
    "This is a permanent design decision, not a pending TODO. " +
    "See docs/cross-harness-skills.md for the full rationale.\n",
};

function usage(exitCode = 0) {
  const message = `
Export selected marketplace agents into a consumer repository.

Usage:
  vfa-export-agents --platform <platform> --agents <agent-id[,agent-id...]> [--repo <path>] [--force]
  vfa-export-agents --platform <platform> --role <role-id> [--provider <provider>] [--repo <path>] [--force]
  vfa-export-agents --platform <platform> --provider <provider> [--repo <path>] [--force]
  vfa-export-agents --platform <platform> --all [--repo <path>] [--force]
  vfa-export-agents --list
  vfa-export-agents --list-roles
  vfa-export-agents --list-providers

Platforms:
  codex, copilot, claude-code, cursor, gemini, kiro, kiro-ide, kiro-cli

Roles:
  cloud-security-engineer, cloud-platform-engineer, cloud-dba,
  cloud-finops-analyst, cloud-solutions-architect, cloud-devops-engineer,
  cloud-ai-platform-engineer, plus kubernetes-* specialisations.

Selectors (mutually exclusive):
  --agents <ids>       Install one or more named agent ids (comma-separated).
  --role <role-id>     Install every agent in the role's bundled list.
  --provider <name>    Install every agent whose provider field equals <name>.
  --all                Install every agent in the catalog.

  --provider <p> --role <r>  → narrow the role to agents whose provider == p.
  --provider <p>             → standalone; equivalent to --all filtered to p.

Options:
  --repo <path>          Target repository path (default: cwd).
  --force                Overwrite existing files without prompting.
  --list-providers        List all providers with agent counts; then exit.
  --dry-run              Print the export plan without writing files.
  --plan-json             Emit a bounded vfa-export/1 plan; never write targets.
  --no-skills            Skip companion skill bundling.

Companion skills:
  By default, when --platform supports skill bundling (claude-code, copilot, gemini),
  each agent's same-named SKILL.md companion is also exported into the
  platform skill directory (e.g. <repo>/.claude/skills/, <repo>/.github/skills/,
  or <repo>/.gemini/skills/).
  Pairing rule: agent id '<name>-agent' bundles skill '<name>' if it exists.
  Use --no-skills to export agents only.

Examples:
  vfa-export-agents --list
  vfa-export-agents --list-roles
  vfa-export-agents --list-providers
  vfa-export-agents --platform claude-code --agents azure-cosmosdb-platform-operator-agent
  vfa-export-agents --platform claude-code --role cloud-security-engineer
  vfa-export-agents --platform claude-code --role cloud-security-engineer --provider azure
  vfa-export-agents --platform claude-code --provider aws
  vfa-export-agents --platform claude-code --provider azure --dry-run
  vfa-export-agents --platform claude-code --all --no-skills --repo /path/to/project
  vfa-export-agents --platform kiro --agents azure-cosmosdb-platform-operator-agent --repo ../consumer-repo
  vfa-export-agents --platform copilot --all --repo /path/to/project --force
`.trim();
  console[exitCode === 0 ? "log" : "error"](message);
  process.exit(exitCode);
}

function parseArgs(argv) {
  // Use Node.js built-in util.parseArgs (stable since v18.3, available in v22):
  // - handles --key=value inline form natively
  // - returns null-prototype values object (prototype pollution safe)
  // - strict mode throws a real Error for unknown flags (not a silent usage() exit)
  let parsed;
  try {
    parsed = utilParseArgs({
      args: argv,
      strict: true,
      allowPositionals: false,
      options: {
        help:             { type: "boolean", short: "h", default: false },
        list:             { type: "boolean", default: false },
        "list-roles":     { type: "boolean", default: false },
        "list-providers": { type: "boolean", default: false },
        force:            { type: "boolean", default: false },
        all:              { type: "boolean", default: false },
        "dry-run":        { type: "boolean", default: false },
        "plan-json":      { type: "boolean", default: false },
        "no-skills":      { type: "boolean", default: false },
        platform:         { type: "string" },
        role:             { type: "string" },
        provider:         { type: "string" },
        repo:             { type: "string" },
        agents:           { type: "string" },
      },
    });
  } catch (err) {
    // Unknown or mistyped flags surface here with a clear message.
    console.error(err.message);
    usage(1);
  }

  const v = parsed.values;
  if (v["plan-json"] && v.help) {
    throw new Error("--plan-json cannot be combined with --help.");
  }
  if (v.help) usage(0);

  // Validate --provider: empty string and whitespace-only are rejected.
  // unicode zero-width chars (e.g. U+200B) pass trim() in some engines so
  // the downstream format regex /^[a-z0-9][a-z0-9-]*$/ acts as a second gate.
  const providerRaw = v.provider ?? null;
  if (providerRaw !== null) {
    const provVal = providerRaw.trim();
    if (!provVal) {
      throw new Error(
        "--provider requires a non-empty value. " +
        "Run 'vfa-export-agents --list-providers' for valid options."
      );
    }
  }

  if (v["plan-json"] && (v["dry-run"] || v.list || v["list-roles"] || v["list-providers"])) {
    throw new Error("--plan-json cannot be combined with --dry-run or listing modes.");
  }
  if ((v.role && (v.agents !== undefined || v.all)) || (v.agents !== undefined && (v.provider !== undefined || v.all))) {
    throw new Error("Ambiguous selection: use --agents, --role [--provider], --provider [--all], or --all.");
  }
  if (v.agents !== undefined && (!v.agents.trim() || v.agents.split(",").some((id) => !id.trim()) || new Set(v.agents.split(",").map((id) => id.trim())).size !== v.agents.split(",").length)) {
    throw new Error("--agents requires non-empty, unique comma-separated agent ids.");
  }
  if (v.role !== undefined && !/^[a-z0-9][a-z0-9-]*$/.test(v.role)) {
    throw new Error("Invalid --role value.");
  }

  return {
    repo:          v.repo ? path.resolve(v.repo) : process.cwd(),
    force:         v.force ?? false,
    list:          v.list ?? false,
    listRoles:     v["list-roles"] ?? false,
    listProviders: v["list-providers"] ?? false,
    all:           v.all ?? false,
    dryRun:        v["dry-run"] ?? false,
    planJson:      v["plan-json"] ?? false,
    noSkills:      v["no-skills"] ?? false,
    platform:      v.platform ?? null,
    role:          v.role ?? null,
    provider:      providerRaw !== null ? providerRaw.trim() : null,
    agents:        v.agents
      ? v.agents.split(",").map((s) => s.trim()).filter(Boolean)
      : [],
  };
}

function walk(dir, matcher, results = []) {
  checkPath(dir, "directory", "read source directory");
  for (const entry of fs.readdirSync(dir, { withFileTypes: true }).sort((a, b) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0)) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isSymbolicLink()) throw new Error(`Refusing symbolic link in source tree: ${fullPath}`);
    if (entry.isDirectory()) {
      walk(fullPath, matcher, results);
      continue;
    }
    if (matcher(fullPath)) results.push(fullPath);
  }
  return results;
}

function loadAgents() {
  const metadataPaths = walk(path.join(repoRoot, "agents"), (fullPath) =>
    fullPath.endsWith(`${path.sep}metadata.json`)
  );

  const agents = metadataPaths.map((metadataPath) => {
    const metadata = JSON.parse(readSource(metadataPath, true).bytes.toString("utf8"));
    if (!metadata || !validId(metadata.id) || !validId(metadata.provider) ||
        !metadata.harness_variants || typeof metadata.harness_variants !== "object" || Array.isArray(metadata.harness_variants) ||
        (metadata.companion_skills !== undefined && (!Array.isArray(metadata.companion_skills) || metadata.companion_skills.some((id) => !validId(id))))) {
      throw new Error(`Malformed agent metadata: ${metadataPath}`);
    }
    return {
      id: metadata.id,
      name: metadata.name,
      provider: metadata.provider,
      summary: metadata.summary,
      harness_variants: metadata.harness_variants ?? {},
      companion_skills: Array.isArray(metadata.companion_skills) ? metadata.companion_skills : undefined,
      metadataPath,
    };
  });

  const byId = new Map(agents.map((agent) => [agent.id, agent]));
  if (byId.size !== agents.length) throw new Error("Duplicate agent id in metadata.");
  return { agents, byId };
}

function normalizePlatform(platform) {
  const lowered = platform.toLowerCase();
  return Object.hasOwn(PLATFORM_ALIASES, lowered) ? PLATFORM_ALIASES[lowered] : lowered;
}

function ensurePlatform(platform) {
  if (!platform) usage(1);
  const normalized = normalizePlatform(platform);
  if (!PLATFORM_CONFIG[normalized]) {
    console.error(`Unsupported platform: ${platform}`);
    usage(1);
  }
  return normalized;
}

function assertWithin(parent, child, label) {
  // Lexical containment complements checkPath's ancestor inspection; neither
  // promises atomicity against concurrent malicious same-user modifications.
  const resolvedParent = path.resolve(parent);
  const resolvedChild = path.resolve(child);
  const sep = path.sep;
  const parentWithSep = resolvedParent.endsWith(sep) ? resolvedParent : resolvedParent + sep;
  if (resolvedChild !== resolvedParent && !resolvedChild.startsWith(parentWithSep)) {
    throw new Error(
      `Refusing to ${label}: path '${resolvedChild}' escapes '${resolvedParent}'. ` +
      `This indicates a malformed metadata.json or path traversal attempt.`
    );
  }
}

function loadSkills() {
  const skillsRoot = path.join(repoRoot, "skills");
  const rootState = checkPath(skillsRoot, "directory", "read skill directory");
  if (!rootState.exists) return new Map();
  const byName = new Map();
  for (const provider of fs.readdirSync(skillsRoot, { withFileTypes: true })) {
    if (provider.isSymbolicLink()) throw new Error(`Refusing symbolic link in skill source: ${provider.name}`);
    if (!provider.isDirectory()) continue;
    const providerDir = path.join(skillsRoot, provider.name);
    for (const skill of fs.readdirSync(providerDir, { withFileTypes: true })) {
      if (skill.isSymbolicLink()) throw new Error(`Refusing symbolic link in skill source: ${skill.name}`);
      if (!skill.isDirectory()) continue;
      const skillDir = path.join(providerDir, skill.name);
      const skillFile = path.join(skillDir, "SKILL.md");
      if (!checkPath(skillFile, "file", "read skill source").exists) continue;
      if (!validId(skill.name)) throw new Error(`Malformed skill directory name: ${skill.name}`);
      const metaFile = path.join(skillDir, "metadata.json");
      let skillProvider = provider.name;
      if (checkPath(metaFile, "file", "read skill metadata").exists) {
        const meta = JSON.parse(readSource(metaFile, true).bytes.toString("utf8"));
        if (!meta || typeof meta !== "object" || Array.isArray(meta) || (meta.provider !== undefined && !validId(meta.provider))) throw new Error(`Malformed skill metadata: ${metaFile}`);
        skillProvider = meta.provider || provider.name;
      }
      if (byName.has(skill.name)) throw new Error(`Duplicate skill name: ${skill.name}`);
      byName.set(skill.name, { dir: skillDir, provider: skillProvider });
    }
  }
  return byName;
}

function resolveCompanionSkills(selectedAgents, skillsByName, role, includeAll, selectedProvider) {
  const skillNames = new Set();
  if (includeAll) {
    for (const [name, meta] of skillsByName.entries()) {
      if (!selectedProvider || meta.provider === selectedProvider || meta.provider === "shared") {
        skillNames.add(name);
      }
    }
  }
  if (role && Array.isArray(role.skills)) {
    for (const id of role.skills) {
      const meta = skillsByName.get(id);
      // Exclude skills not found on disk — do not promote undefined through the
      // provider gate (the old `!meta` branch made dry-run output lie about what
      // would actually be written, masking catalog rot).
      if (!meta) continue;
      if (!selectedProvider || meta.provider === selectedProvider || meta.provider === "shared") {
        skillNames.add(id);
      }
    }
  }
  const orphans = [];
  for (const agent of selectedAgents) {
    // Prefer explicit companion_skills if declared (even if empty — that means intentional no-pair)
    if (Array.isArray(agent.companion_skills)) {
      for (const skillId of agent.companion_skills) {
        const meta = skillsByName.get(skillId);
        // Apply the same provider scope gate used for role.skills — prevents a
        // cross-provider companion_skills declaration from leaking rival skills.
        if (meta && (!selectedProvider || meta.provider === selectedProvider || meta.provider === "shared")) {
          skillNames.add(skillId);
        }
      }
      // companion_skills: [] is intentional no-pair — do NOT count as orphan
      continue;
    }
    // Fall back to name-stripping convention
    const skillName = agent.id.endsWith("-agent")
      ? agent.id.slice(0, -"-agent".length)
      : agent.id;
    const meta = skillsByName.get(skillName);
    if (meta && (!selectedProvider || meta.provider === selectedProvider || meta.provider === "shared")) {
      skillNames.add(skillName);
    } else if (!role) {
      orphans.push(agent.id);
    }
  }
  return { skillNames: [...skillNames].sort(), orphans };
}

function loadRoles() {
  const rolesPath = path.join(repoRoot, "catalog", "install-roles.json");
  if (!checkPath(rolesPath, "file", "read roles metadata").exists) {
    throw new Error("catalog/install-roles.json not found. Ensure the package is correctly installed.");
  }
  const data = JSON.parse(readSource(rolesPath, true).bytes.toString("utf8"));
  if (!data || !data.roles || typeof data.roles !== "object" || Array.isArray(data.roles)) throw new Error("Malformed install roles metadata.");
  return data;
}

function listAgents(agents) {
  for (const agent of agents.sort((a, b) => a.id.localeCompare(b.id))) {
    console.log(`${agent.id}\t${agent.provider}\t${agent.name}`);
  }
}

function listRoles(rolesData) {
  for (const [roleId, role] of Object.entries(rolesData.roles)) {
    const agentCount = role.agents.length;
    const skillCount = (role.skills ?? []).length;
    console.log(`${roleId}\t${role.label}\t${agentCount} agents, ${skillCount} skills`);
  }
}

function listProviders(agents) {
  const counts = new Map();
  for (const agent of agents) {
    counts.set(agent.provider, (counts.get(agent.provider) ?? 0) + 1);
  }
  const sorted = [...counts.entries()].sort(([a], [b]) => a.localeCompare(b));
  for (const [provider, count] of sorted) {
    console.log(`${provider}\t${count} agent(s)`);
  }
}

function buildDestinations(agent, platform) {
  const config = PLATFORM_CONFIG[platform];
  const destinations = [];

  for (const [variantKey, folder, extension] of config.variants) {
    const relativeSource = agent.harness_variants[variantKey];
    if (!relativeSource) {
      throw new Error(`Agent ${agent.id} does not have a ${variantKey} harness variant.`);
    }
    if (typeof relativeSource !== "string" || /[\\/]\.\.[\\/]|^\.\.[\\/]|[\\/]\.\.$|^\.\.$/.test(relativeSource) || path.isAbsolute(relativeSource) || path.win32.isAbsolute(relativeSource)) {
      throw new Error(
        `Agent ${agent.id} ${variantKey} harness path '${relativeSource}' is invalid: ` +
        `must be a relative path within the repository, no '..' traversal, no absolute paths.`
      );
    }
    if (!/^[a-z0-9][a-z0-9-]*$/.test(agent.id)) {
      throw new Error(
        `Agent id '${agent.id}' fails schema pattern ^[a-z0-9][a-z0-9-]*$. ` +
        `Cannot derive a safe destination filename.`
      );
    }
    const source = path.join(repoRoot, relativeSource);
    assertWithin(repoRoot, source, "read source");
    destinations.push({
      variantKey,
      source,
      destRelative: path.join(folder, `${agent.id}${extension}`),
    });
  }

  return destinations;
}

function planExport(args, agents, byId, platform) {
  const notices = [];
  // Validate --provider early so the standalone path and the role-filter path
  // share the same error surface.
  if (args.provider && !/^[a-z0-9][a-z0-9-]*$/.test(args.provider)) {
    throw new Error(`Invalid --provider value '${args.provider}'. Must match /^[a-z0-9][a-z0-9-]*$/.`);
  }
  if (args.provider) {
    const providersInCatalog = new Set(agents.map((a) => a.provider));
    if (!providersInCatalog.has(args.provider)) {
      throw new Error(
        `Unknown --provider '${args.provider}'. Run 'vfa-export-agents --list-providers' for the list.`
      );
    }
  }

  let selectedAgents;
  let selectedRole = null;
  if (args.role) {
    const rolesData = loadRoles();
    const role = Object.hasOwn(rolesData.roles, args.role) ? rolesData.roles[args.role] : undefined;
    selectedRole = role;
    if (!role) {
      const validRoles = Object.keys(rolesData.roles).join(", ");
      throw new Error(`Unknown role: ${args.role}. Valid roles: ${validRoles}`);
    }
    if (!Array.isArray(role.agents) || role.agents.some((id) => !validId(id)) || new Set(role.agents).size !== role.agents.length ||
        (role.skills !== undefined && (!Array.isArray(role.skills) || role.skills.some((id) => !validId(id))))) {
      throw new Error(`Malformed role metadata: ${args.role}`);
    }
    for (const id of role.agents) {
      if (!byId.has(id)) throw new Error(`Role '${args.role}' references unknown agent id: ${id}. Run npm run validate to check catalog integrity.`);
    }
    let roleAgentIds = role.agents;
    if (args.provider) {
      roleAgentIds = roleAgentIds.filter((id) => {
        const agent = byId.get(id);
        return agent && agent.provider === args.provider;
      });
      if (roleAgentIds.length === 0) {
        throw new Error(`No agents found for role '${args.role}' with --provider '${args.provider}'.`);
      }
    }
    selectedAgents = roleAgentIds.map((agentId) => {
      const agent = byId.get(agentId);
      if (!agent) {
        throw new Error(`Role '${args.role}' references unknown agent id: ${agentId}. Run npm run validate to check catalog integrity.`);
      }
      return agent;
    });
  } else if (args.provider) {
    // Standalone --provider: equivalent to --all filtered to that provider.
    selectedAgents = agents.filter((a) => a.provider === args.provider);
    if (selectedAgents.length === 0) {
      throw new Error(`No agents found for --provider '${args.provider}'.`);
    }
  } else if (args.all) {
    // --all targets every catalog agent, but only those that actually
    // declare harness variants for the requested platform can be exported.
    // Agents authored without per-harness adapter files (e.g., declaring
    // harnesses: ["other"] as a Wave-1 placeholder) are silently skipped
    // with a stderr notice so smoke tests don't fail on incomplete portfolios.
    const platformConfig = PLATFORM_CONFIG[platform];
    const requiredVariants = platformConfig.variants.map(([variantKey]) => variantKey);
    const supports = (agent) =>
      requiredVariants.every((variantKey) => agent.harness_variants?.[variantKey]);
    selectedAgents = agents.filter(supports);
    const skipped = agents.length - selectedAgents.length;
    if (skipped > 0) {
      notices.push(
        `[vfa] --all on platform '${platform}': skipped ${skipped} agent(s) ` +
        `without ${requiredVariants.join("+")} harness variant(s).\n`
      );
    }
  } else {
    selectedAgents = args.agents.map((agentId) => {
      const agent = byId.get(agentId);
      if (!agent) {
        throw new Error(`Unknown agent id: ${agentId}`);
      }
      return agent;
    });
  }

  if (selectedAgents.length === 0) {
    throw new Error("No agents selected. Use --agents, --role, --provider, or --all.");
  }

  const rootState = checkPath(args.repo, "directory", "use destination root");
  const targetRoot = path.join(fs.realpathSync(rootState.anchor.path), ...rootState.missing);
  const skillsDestRoot = SKILLS_PLATFORM_CONFIG[platform];
  let skillsByName = new Map();
  let skillNames = [];
  let orphans = [];
  if (!args.noSkills && skillsDestRoot) {
    skillsByName = loadSkills();
    ({ skillNames, orphans } = resolveCompanionSkills(
      selectedAgents, skillsByName, selectedRole, args.all && !args.provider, args.provider ?? null
    ));
  }

  const files = [];
  const groups = [];
  const conflicts = [];
  const destinations = new Set();
  let renderedBytes = 0;
  function addArtifact(source, destRelative, kind, id, variant, group) {
    const dest = path.join(targetRoot, destRelative);
    assertWithin(targetRoot, dest, "write destination");
    const collisionKey = process.platform === "win32" ? dest.toLowerCase() : dest;
    if (destinations.has(collisionKey)) throw new Error(`Destination collision: ${dest}`);
    destinations.add(collisionKey);
    if (files.length >= MAX_FILES) throw new Error(`PLAN_LIMIT: export exceeds ${MAX_FILES} files.`);
    const rendered = renderArtifact(source, platform, kind, targetRoot);
    renderedBytes += rendered.content.length;
    if (renderedBytes > MAX_RENDER_BYTES) throw new Error("PLAN_LIMIT: rendered artifacts exceed 256 MiB.");
    const destination = { path: dest, relative: relativePath(targetRoot, dest), ...destinationState(dest, kind === "skill") };
    const action = destination.exists ? (args.force ? "overwrite" : "conflict") : "create";
    if (action === "conflict") conflicts.push({
      code: "EXISTS_WITHOUT_FORCE", path: destination.relative,
      message: `Refusing to overwrite existing file without --force: ${dest}`,
    });
    const file = {
      kind, id, variant, source: rendered.source, destination, action,
      transformation: rendered.transformation, final: rendered.final,
    };
    files.push(file);
    group.artifacts.push({ file, content: rendered.content });
  }
  for (const agent of selectedAgents) {
    for (const operation of buildDestinations(agent, platform)) {
      const group = { kind: "agent", artifacts: [], message: `installed\t${agent.id}\t${operation.variantKey}\t${operation.destRelative}` };
      addArtifact(operation.source, operation.destRelative, "agent", agent.id, operation.variantKey, group);
      groups.push(group);
    }
  }
  for (const skillName of skillNames) {
    const sourceDir = skillsByName.get(skillName).dir;
    const destRelative = path.join(skillsDestRoot, skillName);
    const group = { kind: "skill", artifacts: [], message: `installed\tskill:${skillName}\t${platform}\t${destRelative}` };
    function visit(dir) {
      checkPath(dir, "directory", "read skill source");
      for (const entry of fs.readdirSync(dir, { withFileTypes: true }).sort((a, b) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0)) {
        const source = path.join(dir, entry.name);
        if (entry.isSymbolicLink()) throw new Error(`Refusing to copy symbolic link in skill tree: ${source}`);
        if (entry.isDirectory()) visit(source);
        else if (entry.isFile()) addArtifact(source, path.join(destRelative, path.relative(sourceDir, source)), "skill", skillName, platform, group);
        else throw new Error(`Refusing non-regular file in skill tree: ${source}`);
      }
    }
    visit(sourceDir);
    groups.push(group);
  }
  const sourceIdentities = new Set(files.map((file) => `${file.source.identity.device}:${file.source.identity.inode}`));
  const existingDestinations = new Set();
  for (const file of files.filter((item) => item.destination.exists)) {
    const key = `${file.destination.identity.device}:${file.destination.identity.inode}`;
    if (sourceIdentities.has(key) || existingDestinations.has(key)) {
      throw new Error(`Destination collision through an aliased file: ${file.destination.path}`);
    }
    existingDestinations.add(key);
  }

  const skillNoticesBefore = [];
  const skillNoticesAfter = [];
  if (args.noSkills) skillNoticesBefore.push(`[vfa] --no-skills: companion skills not bundled.\n`);
  else if (!skillsDestRoot) skillNoticesBefore.push(SKIP_SKILLS_PLATFORM_NOTICES[platform] ??
    `[vfa] Note: skills bundling is not yet supported on platform '${platform}'. Agents exported only. Pass --no-skills to silence.\n`);
  else {
    skillNoticesAfter.push(
      `[vfa] Bundled ${skillNames.length} companion skill(s) alongside ${selectedAgents.length} agent(s)` +
      (orphans.length ? ` (no-skill agents: ${orphans.length})` : "") + `. Use --no-skills to opt out.\n`
    );
    if (orphans.length && orphans.length <= 10) skillNoticesAfter.push(`[vfa] Agents without companion skill: ${orphans.join(", ")}\n`);
  }

  // Bind the actual package/catalog and metadata used by selection, not a Git
  // branch label. Absent optional manifests are recorded rather than invented.
  readSource(fileURLToPath(import.meta.url), true);
  const catalogs = [];
  for (const name of ["agents.json", "skills.json", "install-roles.json", "skill-manifest.json"]) {
    const file = path.join(repoRoot, "catalog", name);
    if (checkPath(file, "file", "read catalog").exists) {
      const record = readSource(file, true).record;
      catalogs.push({ ...record, exists: true });
    } else catalogs.push({ path: `catalog/${name}`, exists: false });
  }
  const inputs = [...sourceInputs.values()].sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0);
  const selection = {
    kind: args.role ? "role" : args.provider ? "provider" : args.all ? "all" : "agents",
    role: args.role, provider: args.provider, all: args.all,
    agents: selectedAgents.map((agent) => agent.id), skills: skillNames,
  };
  const options = { platform, force: args.force, noSkills: args.noSkills };
  const source = { root: fs.realpathSync(repoRoot), catalogs, inputs, sha256: sha256(canonicalJson(inputs)) };
  const plan = {
    protocol: "vfa-export/1", type: "export-plan", canonicalization: "sorted-keys-json/v1",
    source, selection, selection_sha256: sha256(canonicalJson(selection)),
    options, options_sha256: sha256(canonicalJson(options)),
    destination: { root: targetRoot, ...rootState },
    files, conflicts, confirmable: conflicts.length === 0,
  };
  plan.digest = sha256(canonicalJson(plan));
  return { plan, groups, selectedAgents, skillNames, notices, skillNoticesBefore, skillNoticesAfter };
}

function main() {
  const args = parseArgs(process.argv.slice(2));

  const cwd = process.cwd();
  const cwdWithSep = cwd.endsWith(path.sep) ? cwd : cwd + path.sep;
  if (args.repo !== cwd && !args.repo.startsWith(cwdWithSep)) {
    process.stderr.write(
      `[vfa] Warning: --repo '${args.repo}' is outside the current working directory.\n` +
      `[vfa] Verify this is the intended target before continuing.\n`
    );
  }

  const { agents, byId } = loadAgents();

  if (args.list) {
    listAgents(agents);
    return;
  }

  if (args.listRoles) {
    const rolesData = loadRoles();
    listRoles(rolesData);
    return;
  }

  if (args.listProviders) {
    listProviders(agents);
    return;
  }

  const platform = ensurePlatform(args.platform);

  const result = planExport(args, agents, byId, platform);
  for (const notice of result.notices) process.stderr.write(notice);
  if (args.planJson) {
    const output = canonicalJson(result.plan);
    if (Buffer.byteLength(output + "\n") > MAX_PLAN_BYTES) throw new Error("PLAN_LIMIT: export plan exceeds 8 MiB.");
    process.stdout.write(output + "\n");
    if (!result.plan.confirmable) {
      process.stderr.write(result.plan.conflicts.map((conflict) => conflict.message).join("\n") + "\n");
      process.exitCode = 1;
    }
    return;
  }
  if (args.dryRun) {
    for (const agent of result.selectedAgents) console.log(`export agent: ${agent.id} [provider=${agent.provider}]`);
    for (const skillName of result.skillNames) console.log(`export skill: ${skillName}`);
    process.stderr.write(
      `[vfa] --dry-run: ${result.selectedAgents.length} agent(s)` +
      (result.skillNames.length > 0 ? `, ${result.skillNames.length} skill(s)` : "") +
      ` planned, no files written.\n`
    );
    return;
  }
  if (!result.plan.confirmable) throw new Error(result.plan.conflicts[0].message);
  const lastAgentGroup = result.groups.findLast((group) => group.kind === "agent");
  for (const group of result.groups) {
    for (const artifact of group.artifacts) writeArtifact(artifact);
    console.log(group.message);
    if (group === lastAgentGroup) {
      for (const notice of result.skillNoticesBefore) process.stderr.write(notice);
    }
  }
  for (const notice of result.skillNoticesAfter) process.stderr.write(notice);
}

try {
  main();
} catch (error) {
  console.error(error.message);
  process.exit(1);
}
