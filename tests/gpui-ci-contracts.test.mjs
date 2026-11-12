import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const tuiFile = ".github/workflows/vfa-tui-ci.yml";
const desktopFile = ".github/workflows/gpui-desktop-ci.yml";
const title = "Desktop static contracts—not native acceptance";
const checkout = "actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10";
// Captured before the path-only TUI edit. This protects ALL existing permissions,
// concurrency, environment, job/step settings, action pins and command bytes.
const preservedTuiHash = "1675fc6ec1b5806c1c07d2b3ecfbcd1adbc44febc7010ca9c64bcc6fa9af7a76";
const sharedInputs = [
  "tools/vfa-tui/src/engine/mod.rs",
  "scripts/export-marketplace-agents.mjs",
  "scripts/install-codex-home.mjs",
  "schemas/agent.schema.json",
  "catalog/agents.json",
  "package.json",
  "tests/export-plan.test.mjs",
  "tests/fixtures/export-plan/legacy-contracts.json",
  "tests/gpui-ci-contracts.test.mjs",
];
const commands = [
  "python3 tools/vfa-desktop/tests/check_bootstrap.py --manifest-path tools/vfa-desktop/Cargo.toml --documents-only --self-test",
  "python3 tools/vfa-tui/tests/check_feature_boundary.py --self-test",
  "node --test tests/gpui-ci-contracts.test.mjs",
];

// A deliberately constrained reader for these small workflows, not a general
// YAML parser. Unsupported structures fail closed rather than guessing their
// GitHub semantics. It reads active event filters and actual scalar run steps;
// comments and detached lists cannot satisfy the contract.
function linesOf(text) {
  assert.ok(!text.includes("\t"), "workflow indentation must use spaces");
  return text.split(/\r?\n/).map((line) => line.replace(/\s+#.*$/, "")).filter((line) => line.trim() && !line.trimStart().startsWith("#"));
}
function fields(lines, indent) {
  const result = new Map();
  const pattern = new RegExp(`^ {${indent}}([a-zA-Z][a-zA-Z0-9_-]*):(?: +(.*))?$`);
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    if (line.search(/\S/) !== indent) continue;
    const match = line.match(pattern);
    assert.ok(match, `unsupported workflow mapping: ${line}`);
    const [, key, value = ""] = match;
    assert.ok(!result.has(key), `duplicate workflow mapping: ${key}`);
    let end = index + 1;
    while (end < lines.length && lines[end].search(/\S/) > indent) end += 1;
    result.set(key, { value, body: lines.slice(index + 1, end) });
  }
  return result;
}
function keys(mapping, expected) {
  assert.deepEqual([...mapping.keys()].sort(), [...expected].sort(), "unexpected or missing workflow keys");
}
function scalar(value) {
  if ((value.startsWith("'") && value.endsWith("'")) || (value.startsWith('"') && value.endsWith('"'))) return value.slice(1, -1);
  return value;
}
function filters(text) {
  const document = fields(linesOf(text), 0);
  assert.ok(document.has("on"), "missing active event mapping");
  for (const key of ["on", "permissions", "concurrency", "env", "jobs"]) {
    if (document.has(key)) assert.equal(document.get(key).value, "", `${key} must be a block mapping`);
  }
  const events = fields(document.get("on").body, 2);
  keys(events, ["pull_request", "push"]);
  return Object.fromEntries([...events].map(([event, block]) => {
    const settings = fields(block.body, 4);
    keys(settings, event === "push" ? ["branches", "paths"] : ["paths"]);
    if (event === "push") assert.equal(settings.get("branches").value, "[develop]");
    assert.equal(settings.get("paths").value, "", "path filters must be explicit YAML lists");
    const paths = settings.get("paths").body.map((line) => {
      const match = line.match(/^ {6}- (.+)$/);
      assert.ok(match, `unsupported path filter: ${line}`);
      const pattern = scalar(match[1]);
      assert.ok(pattern && !/[!\[\]{}?&*]/.test(pattern.replaceAll("*", "")), `unsupported filter pattern: ${pattern}`);
      return pattern;
    });
    assert.ok(paths.length > 0);
    return [event, paths];
  }));
}
function matches(pattern, file) {
  const escaped = pattern.split("**").map((part) => part.split("*").map((piece) => piece.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("[^/]*")).join(".*");
  return new RegExp(`^${escaped}$`).test(file);
}
function requireCoverage(text, inputs) {
  for (const [event, paths] of Object.entries(filters(text))) {
    for (const input of inputs) {
      assert.ok(fs.statSync(path.join(root, input)).isFile(), `missing consumer input: ${input}`);
      assert.ok(paths.some((pattern) => matches(pattern, input)), `${event} does not trigger for ${input}`);
    }
  }
}
function stepsOf(lines) {
  const steps = [];
  let active;
  let nested;
  for (const line of lines) {
    const item = line.match(/^ {6}- ([a-z-]+): (.+)$/);
    if (item) {
      active = { [item[1]]: scalar(item[2]) };
      steps.push(active);
      nested = null;
      continue;
    }
    const property = line.match(/^ {8}([a-z-]+):(?: (.*))?$/);
    if (property) {
      assert.ok(active, "step property without a step");
      assert.ok(!Object.hasOwn(active, property[1]), "duplicate step property");
      nested = property[2] === undefined ? property[1] : null;
      active[property[1]] = nested ? {} : scalar(property[2]);
      continue;
    }
    const child = line.match(/^ {10}([a-z-]+): (.+)$/);
    assert.ok(child && active && nested === "with", `unsupported step syntax: ${line}`);
    assert.ok(!Object.hasOwn(active.with, child[1]), "duplicate action input");
    active.with[child[1]] = scalar(child[2]);
  }
  return steps;
}
function grade(tui, desktop) {
  const tuiDocument = fields(linesOf(tui), 0);
  keys(tuiDocument, ["name", "on", "permissions", "concurrency", "env", "jobs"]);
  assert.equal(scalar(tuiDocument.get("name").value), "vfa-tui CI");
  const marker = tui.indexOf("permissions:\n");
  assert.ok(marker >= 0, "TUI permission baseline missing");
  assert.equal(createHash("sha256").update(tui.slice(marker)).digest("hex"), preservedTuiHash, "existing TUI policy/jobs/steps changed");
  requireCoverage(tui, [...sharedInputs, tuiFile]);
  requireCoverage(desktop, [...sharedInputs, tuiFile, desktopFile,
    "tools/vfa-desktop/Cargo.toml", "tools/vfa-desktop/tests/check_bootstrap.py",
    "tools/vfa-tui/tests/check_feature_boundary.py"]);
  assert.ok(!linesOf(desktop).some((line) => /^\s*(?:- )?continue-on-error:/.test(line)), "static contracts cannot continue on error");
  const document = fields(linesOf(desktop), 0);
  keys(document, ["name", "on", "permissions", "concurrency", "jobs"]);
  assert.equal(scalar(document.get("name").value), title, "static-only workflow must not imply native acceptance");
  const permissions = fields(document.get("permissions").body, 2);
  keys(permissions, ["contents"]);
  assert.equal(permissions.get("contents").value, "read");
  const concurrency = fields(document.get("concurrency").body, 2);
  keys(concurrency, ["group", "cancel-in-progress"]);
  assert.equal(concurrency.get("group").value, "gpui-desktop-static-${{ github.ref }}");
  assert.equal(concurrency.get("cancel-in-progress").value, "true");
  const jobs = fields(document.get("jobs").body, 2);
  keys(jobs, ["static-contracts"]);
  const job = fields(jobs.get("static-contracts").body, 4);
  keys(job, ["name", "runs-on", "steps"]);
  assert.equal(scalar(job.get("name").value), title, "static-only job must not imply native acceptance");
  assert.equal(job.get("runs-on").value, "ubuntu-latest");
  assert.deepEqual(stepsOf(job.get("steps").body), [
    { uses: checkout, with: { "persist-credentials": "false" } },
    { name: "Declared bootstrap contracts (documents only)", run: commands[0] },
    { name: "Feature-boundary negative fixtures (offline self-test)", run: commands[1] },
    { name: "Workflow trigger and static-command contracts", run: commands[2] },
  ], "only checkout plus the three blocking installation-free static commands are allowed");
}

const tui = fs.readFileSync(path.join(root, tuiFile), "utf8");
const desktop = fs.readFileSync(path.join(root, desktopFile), "utf8");
test("actual workflow filters cover shared consumers and preserve the static-only execution boundary", () => grade(tui, desktop));

function reject(name, change) {
  test(`negative mutation rejected: ${name}`, () => {
    const candidate = change({ tui, desktop });
    assert.ok(candidate.tui !== tui || candidate.desktop !== desktop, "inactive mutation");
    assert.throws(() => grade(candidate.tui, candidate.desktop));
  });
}
for (const workflow of ["tui", "desktop"]) {
  for (const pattern of ["tools/vfa-tui/**", "scripts/**", "schemas/**"]) {
    for (const event of ["pull_request", "push"]) {
      reject(`${workflow} ${event} missing ${pattern}`, (pair) => {
        const heading = `  ${event}:`;
        const start = pair[workflow].indexOf(heading);
        const line = `      - '${pattern}'\n`;
        const index = pair[workflow].indexOf(line, start);
        assert.ok(index >= start);
        pair[workflow] = pair[workflow].slice(0, index) + pair[workflow].slice(index + line.length);
        return pair;
      });
    }
  }
}
for (const command of commands) reject(`dropped command ${command}`, (pair) => ({ ...pair, desktop: pair.desktop.replace(`        run: ${command}`, "        run: true") }));
reject("job continue-on-error", (pair) => ({ ...pair, desktop: pair.desktop.replace("    steps:", "    continue-on-error: true\n    steps:") }));
reject("step continue-on-error", (pair) => ({ ...pair, desktop: pair.desktop.replace(`        run: ${commands[0]}`, `        continue-on-error: true\n        run: ${commands[0]}`) }));
reject("false native workflow label", (pair) => ({ ...pair, desktop: pair.desktop.replace(`name: ${title}`, "name: Desktop native acceptance") }));
reject("false native job label", (pair) => ({ ...pair, desktop: pair.desktop.replace(`    name: ${title}`, "    name: Desktop native acceptance") }));
reject("invented runner capability", (pair) => ({ ...pair, desktop: pair.desktop.replace("runs-on: ubuntu-latest", "runs-on: desktop-native-runner") }));
reject("install/setup action", (pair) => ({ ...pair, desktop: pair.desktop.replace(`uses: ${checkout}`, "uses: actions/setup-node@unapproved") }));
reject("upload action", (pair) => ({ ...pair, desktop: pair.desktop.replace(`uses: ${checkout}`, "uses: actions/upload-artifact@unapproved") }));
reject("remote dispatch trigger", (pair) => ({ ...pair, desktop: pair.desktop.replace("on:\n", "on:\n  workflow_dispatch:\n") }));
reject("dropped bootstrap documents-only boundary", (pair) => ({ ...pair, desktop: pair.desktop.replace(" --documents-only", "") }));

reject("TUI top-level alternate shell bypass", (pair) => ({ ...pair, tui: pair.tui.replace("permissions:\n", "defaults:\n  run:\n    shell: bash --noprofile --norc -c true {0}\npermissions:\n") }));
reject("inline event mapping ambiguity", (pair) => ({ ...pair, tui: pair.tui.replace("on:\n", "on: [workflow_dispatch]\n") }));
