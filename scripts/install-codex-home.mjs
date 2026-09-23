#!/usr/bin/env node
/**
 * Reliable two-stage installer for Vanguard Frontier Agentic on Codex.
 *
 * Stage 1: register/refresh the Codex plugin marketplace.
 * Stage 2: export all Codex-capable agents and companion skills into a Codex home.
 */

import { spawnSync } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const exporter = path.join(repoRoot, "scripts", "export-marketplace-agents.mjs");

const args = process.argv.slice(2);
const opts = {
  marketplace: "VincentChuWaiChow/vanguard-frontier-agentic",
  repo: os.homedir(),
  // Default to refusing overwrites. This wrapper writes into the user's home
  // directory, where a collision is someone's customised agent or skill; the
  // exporter has no backup or rollback, so clobbering must be opt-in.
  force: false,
  skipMarketplace: false,
  dryRun: false,
};

function usage(exitCode = 0) {
  const out = exitCode === 0 ? console.log : console.error;
  out(`Usage: node scripts/install-codex-home.mjs [options]\n\nOptions:\n  --marketplace <source>   Codex marketplace source (default: VincentChuWaiChow/vanguard-frontier-agentic)\n  --repo <path>            Target home/repo path whose .codex folder receives agents/skills (default: $HOME)\n  --dry-run                Preview only: no marketplace mutation and no agent/skill writes\n  --skip-marketplace       Skip codex plugin marketplace add/upgrade\n  --force                  Overwrite existing agent/skill files (default: refuse on collision)\n  --no-force               Explicitly keep the refuse-on-collision default\n  -h, --help               Show this help\n`);
  process.exit(exitCode);
}

for (let i = 0; i < args.length; i++) {
  const arg = args[i];
  if (arg === "-h" || arg === "--help") usage(0);
  if (arg === "--marketplace") {
    const val = args[++i];
    if (!val || val.startsWith("-")) { console.error("--marketplace requires a non-flag value"); usage(1); }
    opts.marketplace = val;
  } else if (arg === "--repo") {
    const val = args[++i];
    if (!val || val.startsWith("-")) { console.error("--repo requires a non-flag value"); usage(1); }
    opts.repo = val;
  }
  else if (arg === "--dry-run") opts.dryRun = true;
  else if (arg === "--skip-marketplace") opts.skipMarketplace = true;
  else if (arg === "--force") opts.force = true;
  else if (arg === "--no-force") opts.force = false;
  else {
    console.error(`Unknown option: ${arg}`);
    usage(1);
  }
}

if (!opts.marketplace) {
  console.error("--marketplace cannot be empty");
  process.exit(1);
}
if (!opts.repo) {
  console.error("--repo cannot be empty");
  process.exit(1);
}

function run(label, command, commandArgs, options = {}) {
  console.error(`\n[${label}] ${command} ${commandArgs.join(" ")}`);
  const result = spawnSync(command, commandArgs, {
    cwd: repoRoot,
    stdio: "inherit",
    ...options,
  });
  if (result.error) {
    console.error(`[${label}] failed to start: ${result.error.message}`);
    process.exit(1);
  }
  if (result.status !== 0) {
    console.error(`[${label}] exited ${result.status}`);
    process.exit(result.status ?? 1);
  }
}

const exportArgs = ["--platform", "codex", "--all", "--repo", opts.repo];
if (opts.force) exportArgs.push("--force");

// Without --force, find out whether the export would collide before changing
// anything. The exporter's --dry-run checks every agent and skill destination
// and fails if any already exists, so a refused install leaves both the
// marketplace and the Codex home untouched rather than half-updated.
if (!opts.force && !opts.dryRun) {
  run("preflight", process.execPath, [exporter, ...exportArgs, "--dry-run"], {
    stdio: ["ignore", "ignore", "inherit"],
  });
}

if (!opts.skipMarketplace) {
  const marketplaceName = opts.marketplace
    .split("/").pop()
    ?.replace(/\.git$/, "")
    ?.replace(/@.+$/, "");

  if (opts.dryRun) {
    // --dry-run previously covered only the exporter stage, so a "preview"
    // still ran `codex plugin marketplace add/upgrade` and mutated real
    // marketplace state. A preview must not change anything.
    console.error(`[dry-run] would run: codex plugin marketplace add ${opts.marketplace}`);
    if (marketplaceName) {
      console.error(`[dry-run] would run: codex plugin marketplace upgrade ${marketplaceName}`);
    }
  } else {
    run("marketplace-add", "codex", ["plugin", "marketplace", "add", opts.marketplace]);
    if (marketplaceName) {
      run("marketplace-upgrade", "codex", ["plugin", "marketplace", "upgrade", marketplaceName]);
    }
  }
}

if (opts.dryRun) exportArgs.push("--dry-run");
run("export-agents-and-skills", process.execPath, [exporter, ...exportArgs]);

console.error("\nOK: two-stage Codex install completed");
