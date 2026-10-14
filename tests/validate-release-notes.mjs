import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const require = createRequire(import.meta.url);
const config = require("../.releaserc.js");

// Resolve each plugin exactly as semantic-release does: from its own
// lib/plugins directory first, falling back to the project
// (semantic-release/lib/plugins/utils.js, loadPlugin). Importing the bare
// specifier from here would resolve from the project root instead, so if npm
// ever nested a different plugin copy under semantic-release, this gate would
// pass against a module the release never runs.
const semanticReleaseDir = path.dirname(require.resolve("semantic-release"));
const loadPluginRequire = createRequire(
  path.join(semanticReleaseDir, "lib", "plugins", "utils.js"),
);

function resolvePlugin(name) {
  try {
    return loadPluginRequire.resolve(name);
  } catch {
    return require.resolve(name);
  }
}

// Which copy of `name` Node would load for code in `fromFile`: the first
// node_modules/<name> walking up from its directory.
function installedVersion(fromFile, name) {
  for (let dir = path.dirname(fromFile); ; dir = path.dirname(dir)) {
    const manifest = path.join(dir, "node_modules", name, "package.json");
    if (fs.existsSync(manifest)) {
      return JSON.parse(fs.readFileSync(manifest, "utf8")).version;
    }
    if (path.dirname(dir) === dir) {
      throw new Error(`${name} is not resolvable from ${fromFile}`);
    }
  }
}

function pluginConfigFor(name) {
  const entry = config.plugins.find(([pluginName]) => pluginName === name)?.[1];
  if (!entry) {
    throw new Error(`${name} configuration is missing`);
  }
  return entry;
}

const failures = [];
const quietLogger = { log() {}, warn() {}, error() {}, success() {} };

// conventional-changelog-conventionalcommits@10 renders only through
// conventional-changelog-writer@9 and refuses older writers outright. The
// stable plugins declare writer@8 and parser@6; package.json `overrides`
// raise them to the set their own next majors publish. Assert the copies the
// release will actually load, so a lockfile or override regression names
// itself instead of surfacing as a template error at release time.
const presetMajor = Number(
  installedVersion(require.resolve("../package.json"), "conventional-changelog-conventionalcommits").split(".")[0],
);
const REQUIRED_MAJORS = { "conventional-changelog-writer": 9, "conventional-commits-parser": 7 };
const pluginFiles = {};
for (const name of ["@semantic-release/commit-analyzer", "@semantic-release/release-notes-generator"]) {
  pluginFiles[name] = resolvePlugin(name);
  if (presetMajor < 10) continue;
  for (const [dep, major] of Object.entries(REQUIRED_MAJORS)) {
    const version = installedVersion(pluginFiles[name], dep);
    if (Number(version.split(".")[0]) < major) {
      failures.push(
        `${name} would load ${dep}@${version}; conventionalcommits@${presetMajor} needs ${major}+`,
      );
    }
  }
}

// Report a wrong writer or parser before rendering: the preset's own guard
// would otherwise throw a template error that hides which copy was loaded.
if (failures.length) {
  for (const failure of failures) console.error(`FAIL ${failure}`);
  process.exit(1);
}

const { generateNotes } = await import(
  pathToFileURL(pluginFiles["@semantic-release/release-notes-generator"]).href
);
const { analyzeCommits } = await import(
  pathToFileURL(pluginFiles["@semantic-release/commit-analyzer"]).href
);

const notes = await generateNotes(pluginConfigFor("@semantic-release/release-notes-generator"), {
  commits: [
    {
      hash: "1234567890abcdef",
      message: "feat(istio): add evidence-bounded review suite",
    },
  ],
  lastRelease: { gitTag: "v3.11.1", gitHead: "old" },
  nextRelease: { version: "3.12.0", gitTag: "v3.12.0", gitHead: "new" },
  options: {
    repositoryUrl:
      "https://github.com/VincentChuWaiChow/vanguard-frontier-agentic.git",
  },
  cwd: process.cwd(),
  logger: console,
});

if (!notes.includes("v3.12.0") || !notes.includes("evidence-bounded review suite")) {
  failures.push("release notes are missing the expected version or feature");
}

// The analyzer decides the version bump, so a preset or parser change that
// misreads a commit ships the wrong version with no error. One commit per
// case, against the configured releaseRules.
const ANALYZER_CASES = [
  ["feat(api): add endpoint", "minor"],
  ["fix(api): handle empty body", "patch"],
  ["feat(api)!: drop the v1 routes", "major"],
  ["fix(api): tighten parsing\n\nBREAKING CHANGE: the v1 payload is rejected", "major"],
  ["security(auth): rotate the signing key", "patch"],
  ["perf(export): stream large catalogs", "patch"],
  ["refactor(exporter): split the writer", "patch"],
  ["build(deps): bump fast-check", "patch"],
  ["revert: feat(api): add endpoint", "patch"],
  ["docs(readme): fix a link", null],
  ["chore(deps): bump the lockfile", null],
  ["feat(no-release): internal tooling only", null],
];
const analyzerConfig = pluginConfigFor("@semantic-release/commit-analyzer");
for (const [message, expected] of ANALYZER_CASES) {
  const got = await analyzeCommits(analyzerConfig, {
    commits: [{ hash: "1234567890abcdef", message }],
    cwd: process.cwd(),
    logger: quietLogger,
  });
  if ((got ?? null) !== expected) {
    failures.push(
      `commit-analyzer: ${JSON.stringify(message.split("\n")[0])} -> ${got ?? null}, expected ${expected}`,
    );
  }
}

if (failures.length) {
  for (const failure of failures) console.error(`FAIL ${failure}`);
  process.exit(1);
}

console.log(
  `OK: semantic-release notes generated and ${ANALYZER_CASES.length} commit-analysis cases hold with conventionalcommits@${presetMajor}`,
);
