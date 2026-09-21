#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parseArgs as utilParseArgs } from "node:util";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

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

// Security-critical validation patterns. These are exported so the
// property-based fuzz suite can exercise the *shipped* definitions instead of
// redeclaring its own copies, which silently drift from production.
export const AGENT_ID_PATTERN = /^[a-z0-9][a-z0-9-]*$/;

export const HARNESS_PATH_TRAVERSAL = /[\\/]\.\.[\\/]|^\.\.[\\/]|[\\/]\.\.$|^\.\.$/;

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

  return {
    repo:          v.repo ? path.resolve(v.repo) : process.cwd(),
    force:         v.force ?? false,
    list:          v.list ?? false,
    listRoles:     v["list-roles"] ?? false,
    listProviders: v["list-providers"] ?? false,
    all:           v.all ?? false,
    dryRun:        v["dry-run"] ?? false,
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
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name);
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
    const raw = fs.readFileSync(metadataPath, "utf8");
    const metadata = JSON.parse(raw);
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
  return { agents, byId };
}

export function normalizePlatform(platform) {
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

export function assertWithin(parent, child, label) {
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

function assertNoSymlinkAncestor(parent, child, label) {
  assertWithin(parent, child, label);

  const resolvedParent = path.resolve(parent);
  const resolvedChild = path.resolve(child);
  const relativeParent = path.dirname(path.relative(resolvedParent, resolvedChild));
  if (!relativeParent || relativeParent === ".") return;

  let current = resolvedParent;
  for (const part of relativeParent.split(path.sep)) {
    current = path.join(current, part);
    let stat;
    try {
      stat = fs.lstatSync(current);
    } catch (err) {
      if (err && err.code === "ENOENT") return;
      throw err;
    }
    if (stat.isSymbolicLink()) {
      throw new Error(
        `Refusing to ${label}: symbolic link ancestor '${current}' would redirect writes outside '${resolvedParent}'. ` +
        `Remove the symlink and retry.`
      );
    }
    if (!stat.isDirectory()) {
      throw new Error(
        `Refusing to ${label}: path ancestor '${current}' is not a directory.`
      );
    }
  }
}

function assertSafeWriteDestination(parent, destination, label) {
  // Lexical containment blocks traversal strings and absolute-path metadata.
  // The ancestor walk then rejects pre-existing symlinks such as `.codex -> /x`
  // before mkdir/copy can follow them. Node does not expose fully race-proof
  // O_NOFOLLOW directory creation, so this intentionally closes the reachable
  // planted-symlink case without pretending to solve privileged TOCTOU races.
  assertNoSymlinkAncestor(parent, destination, label);
}

function loadSkills() {
  const skillsRoot = path.join(repoRoot, "skills");
  if (!fs.existsSync(skillsRoot)) return new Map();
  const byName = new Map();
  for (const provider of fs.readdirSync(skillsRoot, { withFileTypes: true })) {
    if (!provider.isDirectory()) continue;
    const providerDir = path.join(skillsRoot, provider.name);
    for (const skill of fs.readdirSync(providerDir, { withFileTypes: true })) {
      if (!skill.isDirectory()) continue;
      const skillDir = path.join(providerDir, skill.name);
      const metaFile = path.join(skillDir, "metadata.json");
      if (fs.existsSync(path.join(skillDir, "SKILL.md"))) {
        let skillProvider = provider.name;
        if (fs.existsSync(metaFile)) {
          try {
            const meta = JSON.parse(fs.readFileSync(metaFile, "utf8"));
            if (meta.provider) {
              skillProvider = meta.provider;
            }
          } catch (err) {
            // Fall back to directory name if metadata.json is invalid.
          }
        }
        byName.set(skill.name, { dir: skillDir, provider: skillProvider });
      }
    }
  }
  return byName;
}

function copySkillTree(sourceDir, destDir, force, targetRoot) {
  assertWithin(repoRoot, sourceDir, "read skill source");
  for (const entry of fs.readdirSync(sourceDir, { withFileTypes: true })) {
    const src = path.join(sourceDir, entry.name);
    const dst = path.join(destDir, entry.name);
    assertSafeWriteDestination(targetRoot, dst, "write skill tree destination");
    if (entry.isSymbolicLink()) {
      throw new Error(`Refusing to copy symbolic link in skill tree: ${src}`);
    }
    let dstLstat = null;
    try { dstLstat = fs.lstatSync(dst); } catch { /* dst does not exist – fine */ }
    if (dstLstat && dstLstat.isSymbolicLink()) {
      throw new Error(
        `Refusing to write to symbolic link destination in skill tree: ${dst}. ` +
        `Remove the symlink and retry.`
      );
    }
    if (entry.isDirectory()) {
      copySkillTree(src, dst, force, targetRoot);
      continue;
    }
    if (!entry.isFile()) continue;
    if (!force && fs.existsSync(dst)) {
      throw new Error(`Refusing to overwrite existing file without --force: ${dst}`);
    }
    fs.mkdirSync(path.dirname(dst), { recursive: true });
    fs.copyFileSync(src, dst);
  }
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

function copyFile(source, destination, force, targetRoot) {
  assertSafeWriteDestination(targetRoot, destination, "write file destination");
  const sourceStat = fs.lstatSync(source);
  if (sourceStat.isSymbolicLink()) {
    throw new Error(`Refusing to copy symbolic link as harness source: ${source}`);
  }
  if (fs.existsSync(destination)) {
    // Reject symlink destinations regardless of --force. A symlink at the
    // destination would redirect the write outside the repo tree, bypassing
    // assertWithin(). lstatSync does not follow the symlink — exactly what we
    // want here to detect the link itself.
    const destStat = fs.lstatSync(destination);
    if (destStat.isSymbolicLink()) {
      throw new Error(
        `Refusing to write to symbolic link destination: ${destination}. ` +
        `Remove the symlink and retry.`
      );
    }
    if (!force) {
      throw new Error(`Refusing to overwrite existing file without --force: ${destination}`);
    }
  }
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.copyFileSync(source, destination);
}

function rewriteCodexAgentSkillPaths(agentFile, targetRoot) {
  const text = fs.readFileSync(agentFile, "utf8");
  const rewritten = text.replace(
    /^path = "skills\/[^"\n]+\/([^/"\n]+)\/SKILL\.md"$/gm,
    (_match, skillName) => {
      const skillDir = path.join(targetRoot, SKILLS_PLATFORM_CONFIG.codex, skillName);
      return `path = ${JSON.stringify(skillDir)}`;
    }
  );
  if (rewritten !== text) {
    fs.writeFileSync(agentFile, rewritten);
  }
}

function loadRoles() {
  const rolesPath = path.join(repoRoot, "catalog", "install-roles.json");
  if (!fs.existsSync(rolesPath)) {
    throw new Error("catalog/install-roles.json not found. Ensure the package is correctly installed.");
  }
  return JSON.parse(fs.readFileSync(rolesPath, "utf8"));
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
    if (typeof relativeSource !== "string" || HARNESS_PATH_TRAVERSAL.test(relativeSource) || path.isAbsolute(relativeSource)) {
      throw new Error(
        `Agent ${agent.id} ${variantKey} harness path '${relativeSource}' is invalid: ` +
        `must be a relative path within the repository, no '..' traversal, no absolute paths.`
      );
    }
    if (!AGENT_ID_PATTERN.test(agent.id)) {
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

  // Validate --provider early so the standalone path and the role-filter path
  // share the same error surface.
  if (args.provider && !AGENT_ID_PATTERN.test(args.provider)) {
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
      process.stderr.write(
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

  if (args.dryRun) {
    for (const agent of selectedAgents) {
      console.log(`export agent: ${agent.id} [provider=${agent.provider}]`);
    }
    const skillsDestRoot = SKILLS_PLATFORM_CONFIG[platform];
    let dryRunSkillCount = 0;
    if (!args.noSkills && skillsDestRoot) {
      const skillsByName = loadSkills();
      const includeAllSkills = args.all && !args.provider;
      const { skillNames } = resolveCompanionSkills(
        selectedAgents,
        skillsByName,
        selectedRole,
        includeAllSkills,
        args.provider ?? null
      );
      for (const skillName of skillNames) {
        console.log(`export skill: ${skillName}`);
        dryRunSkillCount += 1;
      }
    }
    process.stderr.write(
      `[vfa] --dry-run: ${selectedAgents.length} agent(s)` +
      (dryRunSkillCount > 0 ? `, ${dryRunSkillCount} skill(s)` : "") +
      ` planned, no files written.\n`
    );
    return;
  }

  const operations = [];
  for (const agent of selectedAgents) {
    for (const destination of buildDestinations(agent, platform)) {
      operations.push({
        ...destination,
        dest: path.join(args.repo, destination.destRelative),
        agentId: agent.id,
      });
    }
  }

  for (const operation of operations) {
    assertWithin(args.repo, operation.dest, "write destination");
    copyFile(operation.source, operation.dest, args.force, args.repo);
    if (platform === "codex") {
      rewriteCodexAgentSkillPaths(operation.dest, args.repo);
    }
    console.log(
      `installed\t${operation.agentId}\t${operation.variantKey}\t${path.relative(args.repo, operation.dest)}`
    );
  }

  const skillsDestRoot = SKILLS_PLATFORM_CONFIG[platform];
  if (args.noSkills) {
    process.stderr.write(`[vfa] --no-skills: companion skills not bundled.\n`);
  } else if (!skillsDestRoot) {
    const specificNotice = SKIP_SKILLS_PLATFORM_NOTICES[platform];
    if (specificNotice) {
      process.stderr.write(specificNotice);
    } else {
      process.stderr.write(
        `[vfa] Note: skills bundling is not yet supported on platform '${platform}'. ` +
        `Agents exported only. Pass --no-skills to silence.\n`
      );
    }
  } else {
    const skillsByName = loadSkills();
    // includeAll bundles every skill in the catalog. When --provider is set,
    // selectedAgents is already scoped to that provider — bundling every
    // skill would mix in hundreds of unrelated provider skills, violating
    // the documented "provider install" contract. Scope skills to the
    // selected agents' companion_skills in that case.
    const includeAllSkills = args.all && !args.provider;
    const { skillNames, orphans } = resolveCompanionSkills(
      selectedAgents,
      skillsByName,
      selectedRole,
      includeAllSkills,
      args.provider ?? null
    );
    let bundled = 0;
    for (const skillName of skillNames) {
      const sourceDir = skillsByName.get(skillName)?.dir;
      if (!sourceDir) continue;
      const destDir = path.join(args.repo, skillsDestRoot, skillName);
      assertWithin(args.repo, destDir, "write skill destination");
      copySkillTree(sourceDir, destDir, args.force, args.repo);
      console.log(`installed\tskill:${skillName}\t${platform}\t${path.relative(args.repo, destDir)}`);
      bundled += 1;
    }
    process.stderr.write(
      `[vfa] Bundled ${bundled} companion skill(s) alongside ${selectedAgents.length} agent(s)` +
      (orphans.length ? ` (no-skill agents: ${orphans.length})` : "") +
      `. Use --no-skills to opt out.\n`
    );
    if (orphans.length && orphans.length <= 10) {
      process.stderr.write(`[vfa] Agents without companion skill: ${orphans.join(", ")}\n`);
    }
  }
}

// Run the CLI only when this file is the entry point. Without this guard the
// module could not be imported by tests without executing an export.
const invokedDirectly =
  process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href;

if (invokedDirectly) {
  try {
    main();
  } catch (error) {
    console.error(error.message);
    process.exit(1);
  }
}
