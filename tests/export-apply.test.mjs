import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const repository = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const fixtures = path.join(repository, "tests/fixtures/export-plan");
const work = path.join(repository, "tools/vfa-tui/target/process-export-wave");
fs.mkdirSync(work, { recursive: true });
const contracts = JSON.parse(fs.readFileSync(path.join(fixtures, "legacy-contracts.json"), "utf8"));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object") return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  return JSON.stringify(value);
}
function resign(plan) {
  const { digest: _digest, ...body } = plan;
  plan.digest = hash(canonical(body));
  return plan;
}
function setup(t) {
  const root = fs.mkdtempSync(path.join(work, "apply-Ω "));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const source = path.join(root, "package 東京");
  fs.cpSync(path.join(fixtures, "package"), source, { recursive: true });
  fs.mkdirSync(path.join(source, "scripts"));
  const script = path.join(source, "scripts/export-marketplace-agents.mjs");
  fs.copyFileSync(path.join(repository, "scripts/export-marketplace-agents.mjs"), script);
  const consumer = path.join(root, "consumer Ω");
  const target = path.join(consumer, "not yet created");
  fs.mkdirSync(consumer);
  const plans = path.join(root, "plans");
  fs.mkdirSync(plans);
  const planFile = path.join(plans, "approved.json");
  function run(args, options = {}) {
    const result = spawnSync(process.execPath, [script, ...args], {
      cwd: root, encoding: "utf8", timeout: 30000, maxBuffer: 10 * 1024 * 1024, ...options,
    });
    assert.ifError(result.error);
    assert.equal(result.signal, null, result.stderr);
    return result;
  }
  function preview(args = ["--platform", "codex", "--agents", "alpha-agent"]) {
    const result = run([...args, "--repo", target, "--plan-json"]);
    assert.ok(result.status === 0 || result.status === 1, result.stderr);
    const plan = JSON.parse(result.stdout);
    fs.writeFileSync(planFile, result.stdout, { mode: 0o600 });
    return plan;
  }
  return { root, source, script, consumer, target, planFile, run, preview };
}
function snapshot(root) {
  if (!fs.existsSync(root)) return null;
  const entries = {};
  function visit(dir) {
    for (const item of fs.readdirSync(dir, { withFileTypes: true })) {
      const file = path.join(dir, item.name);
      const relative = path.relative(root, file).split(path.sep).join("/");
      const stat = fs.lstatSync(file, { bigint: true });
      if (stat.isSymbolicLink()) entries[relative] = { link: fs.readlinkSync(file) };
      else if (stat.isDirectory()) {
        entries[relative] = { directory: true, mode: Number(stat.mode & 0o7777n), mtime: String(stat.mtimeNs) };
        visit(file);
      } else entries[relative] = { base64: fs.readFileSync(file).toString("base64"), mode: Number(stat.mode & 0o7777n), mtime: String(stat.mtimeNs) };
    }
  }
  visit(root);
  return entries;
}
function expectedFiles(contract, target) {
  return Object.fromEntries(Object.entries(contract.files).map(([relative, expected]) => {
    let bytes = Buffer.from(expected.base64, "base64");
    if (relative.endsWith(".toml")) bytes = Buffer.from(bytes.toString("utf8").replace(/"<TARGET>\/([^"\n]+)"/g, (_match, suffix) => JSON.stringify(path.join(target, suffix))));
    return [relative, { bytes, mode: expected.mode }];
  }));
}
function assertFiles(env, expected) {
  const actual = snapshot(env.target);
  const files = Object.fromEntries(Object.entries(actual ?? {}).filter(([, value]) => !value.directory));
  assert.deepEqual(Object.keys(files).sort(), Object.keys(expected).sort());
  for (const [relative, value] of Object.entries(expected)) {
    assert.equal(files[relative].base64, value.bytes.toString("base64"), relative);
    assert.equal(files[relative].mode, value.mode, relative);
  }
}
function records(result, plan, success = true) {
  assert.equal(result.status, success ? 0 : 1, result.stderr);
  assert.ok(result.stdout.endsWith("\n"), "protocol must contain complete JSONL records");
  assert.ok(Buffer.byteLength(result.stdout) <= 8 * 1024 * 1024);
  const lines = result.stdout.trimEnd().split("\n");
  for (const line of lines) assert.ok(Buffer.byteLength(line + "\n") <= 64 * 1024);
  const output = lines.map((line) => JSON.parse(line));
  for (const [index, record] of output.entries()) {
    assert.equal(record.protocol, "vfa-export/1");
    assert.equal(record.plan_digest, plan.digest);
    assert.equal(record.seq, index + 1);
  }
  assert.deepEqual(output[0], { protocol: "vfa-export/1", plan_digest: plan.digest, seq: 1, type: "apply-start", file_count: plan.files.length });
  const summary = output.at(-1);
  assert.equal(output.filter((item) => item.type === "apply-summary").length, 1);
  assert.equal(summary.type, "apply-summary");
  if (success) assert.deepEqual(summary, {
    protocol: "vfa-export/1", plan_digest: plan.digest, seq: plan.files.length + 2, type: "apply-summary",
    status: "SUCCEEDED", planned: plan.files.length, written: plan.files.length,
    failed_index: null, unattempted: 0, uncertain: false, code: null,
  });
  return output;
}
function assertSuccess(result, plan, expected) {
  const output = records(result, plan);
  assert.equal(output.length, plan.files.length + 2);
  for (const [index, file] of plan.files.entries()) {
    const oracle = expected[file.destination.relative];
    assert.ok(oracle, "final bytes must exist in independent legacy oracle");
    assert.deepEqual(output[index + 1], {
      protocol: "vfa-export/1", plan_digest: plan.digest, seq: index + 2, type: "file-outcome",
      file_index: index, path: file.destination.relative, status: "WRITTEN",
      final: { sha256: hash(oracle.bytes), size: oracle.bytes.length, mode: oracle.mode },
    });
  }
}
function rejectedWithoutEffects(env, args = ["--apply-plan", env.planFile]) {
  const before = snapshot(env.target);
  const result = env.run(args);
  assert.notEqual(result.status, 0, result.stdout);
  assert.equal(result.stdout, "", result.stderr);
  assert.match(result.stderr, /PLAN_INVALID|PLAN_STALE|PLAN_LIMIT|PROTOCOL_LIMIT/);
  assert.deepEqual(snapshot(env.target), before);
  return result;
}
function instrument(env, mutate, markerName = "instrumentation.json") {
  const marker = path.join(env.root, markerName);
  let script = fs.readFileSync(env.script, "utf8");
  const changed = mutate(script, marker);
  assert.notEqual(changed, script, "inactive instrumentation");
  script = changed.replace("async function applyPlan(file) {", `async function applyPlan(file) {\n  fs.writeFileSync(${JSON.stringify(marker + ".started")}, "instrumented apply entered");`);
  fs.writeFileSync(env.script, script);
  return marker;
}
function assertStarted(marker) { assert.equal(fs.readFileSync(marker + ".started", "utf8"), "instrumented apply entered"); }
const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
function launch(env) {
  const child = spawn(process.execPath, [env.script, "--apply-plan", env.planFile], { cwd: env.root, stdio: ["ignore", "pipe", "pipe"], timeout: 20000 });
  let stderr = "";
  child.stderr.on("data", (chunk) => { stderr += chunk; });
  const completion = new Promise((resolve, reject) => {
    child.once("error", reject);
    child.once("close", (status, signal) => resolve({ status, signal, stderr }));
  });
  return { child, completion };
}

test("actual apply protocol and independent final bytes/modes cover all legacy selections", (t) => {
  for (const name of ["codex", "claude", "no-skills", "role-provider", "provider", "cursor", "kiro"]) {
    const env = setup(t);
    const contract = contracts.find((item) => item.name === name);
    const plan = env.preview(contract.args);
    const expected = expectedFiles(contract, env.target);
    const result = env.run(["--apply-plan", env.planFile]);
    assertSuccess(result, plan, expected);
    assertFiles(env, expected);
    assert.equal(result.stderr, "", name);
  }
});

test("force apply restores fixed legacy bytes/modes; no-force conflict plans are unconfirmable", (t) => {
  const env = setup(t);
  const contract = contracts.find((item) => item.name === "codex");
  const expected = expectedFiles(contract, env.target);
  for (const [relative] of Object.entries(expected)) {
    const file = path.join(env.target, relative);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, "old destination bytes\r\n", { mode: 0o700 });
  }
  const blocked = env.preview(contract.args);
  assert.equal(blocked.confirmable, false);
  rejectedWithoutEffects(env);
  const plan = env.preview([...contract.args, "--force"]);
  assert.ok(plan.files.every((file) => file.action === "overwrite"));
  assertSuccess(env.run(["--apply-plan=" + env.planFile]), plan, expected);
  assertFiles(env, expected);
});

test("source/catalog/script/destination and native ancestor changes reject before effects", (t) => {
  for (const kind of ["source", "catalog", "script", "destination", "ancestor", "missing ancestor", "ancestor mode"]) {
    const env = setup(t);
    const agentsDir = path.join(env.target, ".codex/agents");
    fs.mkdirSync(agentsDir, { recursive: true });
    if (kind === "destination") fs.writeFileSync(path.join(agentsDir, "alpha-agent.toml"), "before preview");
    env.preview(["--platform", "codex", "--agents", "alpha-agent", "--force"]);
    if (kind === "source") fs.appendFileSync(path.join(env.source, "agents/aws/alpha-agent/codex.toml"), "\n# stale");
    if (kind === "catalog") fs.appendFileSync(path.join(env.source, "catalog/agents.json"), "\n");
    if (kind === "script") fs.appendFileSync(env.script, "\n// changed execution manifest\n");
    if (kind === "destination") fs.writeFileSync(path.join(agentsDir, "alpha-agent.toml"), "after preview");
    if (kind === "ancestor") { fs.renameSync(agentsDir, agentsDir + ".old"); fs.mkdirSync(agentsDir); }
    if (kind === "missing ancestor") fs.rmdirSync(agentsDir);
    if (kind === "ancestor mode") fs.chmodSync(agentsDir, 0o700);
    rejectedWithoutEffects(env);
  }
});

test("unrelated sibling writes do not stale stable directory identity bindings", (t) => {
  const env = setup(t);
  const contract = contracts.find((item) => item.name === "codex");
  const plan = env.preview(contract.args);
  fs.writeFileSync(path.join(env.consumer, "unrelated.txt"), "sibling changes directory mtime/size, not identity");
  const expected = expectedFiles(contract, env.target);
  assertSuccess(env.run(["--apply-plan", env.planFile]), plan, expected);
  assertFiles(env, expected);
});

test("digest/shape and recomputed-digest semantic tampering cannot bypass the current authoritative plan", (t) => {
  for (const mutate of [
    (p) => { p.digest = "0".repeat(64); return false; },
    (p) => { p.files[0].final.sha256 = "0".repeat(64); },
    (p) => { p.files[0].destination.relative = "../escape.toml"; },
    (p) => { p.files[0].destination.path = path.join(path.dirname(p.destination.root), "escape.toml"); },
    (p) => { p.files[0].transformation.options.targetRoot += "-different"; },
    (p) => { p.selection.agents = ["gamma-agent"]; p.selection_sha256 = hash(canonical(p.selection)); },
    (p) => { p.files[0].unknown_effect = "delete"; },
    (p) => { delete p.files[0].destination.parent_chain; },
    (p) => { p.confirmable = false; },
    (p) => { p.source.root += "-foreign"; },
  ]) {
    const env = setup(t);
    const plan = env.preview();
    if (mutate(plan) !== false) resign(plan);
    fs.writeFileSync(env.planFile, JSON.stringify(plan));
    rejectedWithoutEffects(env);
  }
});

test("apply is a sole mode: help, options, selectors and duplicated modes cannot be mixed", (t) => {
  const env = setup(t);
  env.preview();
  for (const args of [
    ["--apply-plan", env.planFile, "--help"], ["--help", "--apply-plan", env.planFile],
    ["--apply-plan", env.planFile, "--force"], ["--apply-plan", env.planFile, "--no-skills"],
    ["--apply-plan", env.planFile, "--plan-json"], ["--apply-plan", env.planFile, "--dry-run"],
    ["--apply-plan", env.planFile, "--platform", "codex"], ["--apply-plan", env.planFile, "--repo", env.target],
    ["--apply-plan", env.planFile, "--agents", "alpha-agent"], ["--apply-plan", env.planFile, "--operation-id", "anything"],
    ["--apply-plan", env.planFile, "--apply-plan", env.planFile], ["--apply-plan"], ["--apply-plan="],
  ]) rejectedWithoutEffects(env, args);
});

test("bounded plan reader rejects malformed, oversized, non-regular, symlink and hard-linked plans", (t) => {
  for (const kind of ["malformed", "null", "UTF8", "oversize", "directory", "symlink", "parent symlink", "hardlink", "missing"]) {
    const env = setup(t);
    env.preview();
    if (kind === "malformed") fs.writeFileSync(env.planFile, "{bad JSON");
    if (kind === "null") fs.writeFileSync(env.planFile, "null");
    if (kind === "UTF8") fs.writeFileSync(env.planFile, Buffer.from([0x7b, 0xff, 0x7d]));
    if (kind === "oversize") fs.truncateSync(env.planFile, 8 * 1024 * 1024 + 1);
    if (kind === "directory") { fs.unlinkSync(env.planFile); fs.mkdirSync(env.planFile); }
    if (kind === "symlink") { fs.renameSync(env.planFile, env.planFile + ".real"); fs.symlinkSync(env.planFile + ".real", env.planFile); }
    if (kind === "parent symlink") { const parent = path.dirname(env.planFile); fs.renameSync(parent, parent + ".real"); fs.symlinkSync(parent + ".real", parent, "dir"); }
    if (kind === "hardlink") fs.linkSync(env.planFile, env.planFile + ".alias");
    if (kind === "missing") fs.unlinkSync(env.planFile);
    rejectedWithoutEffects(env);
  }
});

test("real failure after one successful write reports uncertain current effects without rollback or invented outcomes", (t) => {
  const env = setup(t);
  const marker = instrument(env, (script, marker) => script.replace("    fs.writeFileSync(descriptor, artifact.content);", `    if (file.kind === "skill") {\n      fs.writeFileSync(descriptor, "partial current write");\n      fs.writeFileSync(${JSON.stringify(marker)}, "actual partial write occurred");\n      throw new Error("PRIVATE FAILURE CONTENT MUST NOT LEAK");\n    }\n    fs.writeFileSync(descriptor, artifact.content);`));
  const plan = env.preview();
  const result = env.run(["--apply-plan", env.planFile]);
  const output = records(result, plan, false);
  assertStarted(marker);
  assert.equal(fs.readFileSync(marker, "utf8"), "actual partial write occurred");
  assert.equal(output.length, 3);
  assert.equal(output[1].file_index, 0);
  assert.equal(output[1].status, "WRITTEN");
  assert.deepEqual(output[2], {
    protocol: "vfa-export/1", plan_digest: plan.digest, seq: 3, type: "apply-summary", status: "FAILED",
    planned: plan.files.length, written: 1, failed_index: 1, unattempted: plan.files.length - 2, uncertain: true, code: "WRITE_FAILED",
  });
  const expected = expectedFiles(contracts.find((item) => item.name === "codex"), env.target);
  assert.deepEqual(fs.readFileSync(plan.files[0].destination.path), expected[plan.files[0].destination.relative].bytes);
  assert.equal(fs.readFileSync(plan.files[1].destination.path, "utf8"), "partial current write");
  for (const file of plan.files.slice(2)) assert.equal(fs.existsSync(file.destination.path), false);
  assert.ok(!result.stdout.includes("PRIVATE FAILURE") && !result.stdout.includes("partial current"));
});

test("per-write original and apply-created parent replacements are rejected, never adopted or recreated", (t) => {
  for (const kind of ["bound replacement", "bound missing", "owned replacement", "external new"]) {
    const env = setup(t);
    const skillParent = path.join(env.target, ".codex/skills");
    if (kind.startsWith("bound")) fs.mkdirSync(skillParent, { recursive: true });
    const marker = instrument(env, (script, marker) => script.replace("      writeArtifact(artifacts[index], prepareParents);", `      if (index === 1) {\n        ${kind === "bound missing" ? `fs.rmdirSync(${JSON.stringify(skillParent)});` : kind === "bound replacement" ? `fs.renameSync(${JSON.stringify(skillParent)}, ${JSON.stringify(skillParent + ".old")}); fs.mkdirSync(${JSON.stringify(skillParent)});` : kind === "owned replacement" ? `const dir = path.dirname(artifacts[0].file.destination.path); fs.renameSync(dir, dir + ".old"); fs.mkdirSync(dir);` : `fs.mkdirSync(${JSON.stringify(skillParent)});`}\n        fs.writeFileSync(${JSON.stringify(marker)}, "parent fault executed");\n      }\n      writeArtifact(artifacts[index], prepareParents);`));
    const plan = env.preview();
    const result = env.run(["--apply-plan", env.planFile]);
    const output = records(result, plan, false);
    assertStarted(marker);
    assert.equal(fs.readFileSync(marker, "utf8"), "parent fault executed");
    assert.equal(output.at(-1).written, 1);
    assert.equal(output.at(-1).failed_index, 1);
    assert.equal(output.at(-1).uncertain, true);
    assert.equal(output.at(-1).code, "WRITE_FAILED");
    assert.equal(fs.existsSync(plan.files[1].destination.path), false);
    if (kind === "bound missing") assert.equal(fs.existsSync(skillParent), false, "a missing bound directory must not be recreated");
  }
});

test("record/cumulative limits and reserved terminal capacity fail preflight before any target effects", (t) => {
  for (const kind of ["record", "cumulative"]) {
    const env = setup(t);
    const marker = instrument(env, (script) => kind === "record" ? script.replace("const MAX_RECORD_BYTES = 64 * 1024;", "const MAX_RECORD_BYTES = 250;") : script.replace("const MAX_PROTOCOL_BYTES = 8 * 1024 * 1024;", "const MAX_PROTOCOL_BYTES = 64 * 1024 + 1;"));
    env.preview();
    const result = rejectedWithoutEffects(env);
    assert.match(result.stderr, /PROTOCOL_LIMIT/);
    assertStarted(marker);
    assert.equal(fs.existsSync(env.target), false);
  }
});

test("closed stdout fails before any target write and handles callback-error-before-error-event safely", async (t) => {
  const env = setup(t);
  const marker = instrument(env, (script) => script.replace("  const output = protocolOutput();", "  const output = protocolOutput(); // copied closed-output fixture"));
  env.preview();
  const { child, completion } = launch(env);
  child.stdout.destroy();
  const result = await completion;
  assertStarted(marker);
  assert.equal(result.status, 1, result.stderr);
  assert.equal(result.signal, null);
  assert.match(result.stderr, /OUTPUT_FAILED/);
  assert.ok(!result.stderr.includes("Unhandled 'error' event"));
  assert.equal(fs.existsSync(env.target), false);
});

test("slow stdout applies backpressure before further effects and resumes to one complete success summary", async (t) => {
  const env = setup(t);
  const extra = path.join(env.source, "skills/aws/alpha/extra");
  fs.mkdirSync(extra);
  for (let index = 0; index < 1800; index += 1) fs.writeFileSync(path.join(extra, `${index}.txt`), "fixed extra fixture bytes\n");
  const marker = instrument(env, (script, marker) => script.replace("function writeArtifact(artifact, prepareParents = null) {", `function writeArtifact(artifact, prepareParents = null) {\n  fs.appendFileSync(${JSON.stringify(marker)}, "write attempted\\n");`));
  const plan = env.preview();
  const { child, completion } = launch(env);
  child.stdout.pause();
  t.after(() => { if (child.exitCode === null) child.kill(); });
  const deadline = Date.now() + 10000;
  while (!fs.existsSync(marker) && Date.now() < deadline) await delay(25);
  assertStarted(marker);
  assert.ok(fs.existsSync(marker), "instrumented actual writes must start");
  await delay(800);
  const stalled = fs.readFileSync(marker, "utf8").split("\n").length - 1;
  await delay(200);
  assert.equal(fs.readFileSync(marker, "utf8").split("\n").length - 1, stalled, "blocked output must stop target effects");
  assert.ok(stalled > 0 && stalled < plan.files.length, `${stalled}/${plan.files.length} writes before drain`);
  let stdout = "";
  child.stdout.on("data", (chunk) => { stdout += chunk; });
  child.stdout.resume();
  const result = { ...await completion, stdout };
  const output = records(result, plan);
  assert.equal(output.length, plan.files.length + 2);
  assert.equal(fs.readFileSync(marker, "utf8").split("\n").length - 1, plan.files.length);
});

test("a terminal stdout failure retains successful file effects but cannot exit zero or imply complete acceptance", (t) => {
  const env = setup(t);
  const marker = instrument(env, (script, marker) => script.replace("    async emit(record) {", `    async emit(record) {\n      if (record.type === "apply-summary") {\n        fs.writeFileSync(${JSON.stringify(marker)}, "terminal output failure executed");\n        fs.closeSync(process.stdout.fd);\n      }`));
  const plan = env.preview();
  const result = env.run(["--apply-plan", env.planFile]);
  assertStarted(marker);
  assert.equal(fs.readFileSync(marker, "utf8"), "terminal output failure executed");
  assert.equal(result.status, 1);
  assert.match(result.stderr, /OUTPUT_FAILED/);
  const output = result.stdout.trimEnd().split("\n").map((line) => JSON.parse(line));
  assert.equal(output.length, plan.files.length + 1);
  assert.equal(output.filter((item) => item.type === "file-outcome").length, plan.files.length);
  assert.equal(output.some((item) => item.type === "apply-summary"), false);
  assertFiles(env, expectedFiles(contracts.find((item) => item.name === "codex"), env.target));
});


test("fully resigned array platform values cannot bypass strict platform schema or Codex rendering", (t) => {
  for (const platform of ["codex", "claude-code"]) {
    const env = setup(t);
    const plan = env.preview(["--platform", platform, "--agents", "alpha-agent", "--no-skills"]);
    plan.options.platform = [platform];
    plan.options_sha256 = hash(canonical(plan.options));
    for (const file of plan.files) {
      const bytes = fs.readFileSync(path.join(env.source, file.source.path));
      file.transformation = { id: "identity/v1", options: {} };
      file.final.sha256 = hash(bytes);
      file.final.size = bytes.length;
    }
    resign(plan);
    fs.writeFileSync(env.planFile, JSON.stringify(plan));
    rejectedWithoutEffects(env);
  }
});

test("a legitimate rendered artifact may exceed its source read limit within the renderer budget", (t) => {
  const env = setup(t);
  const limit = 128 * 1024;
  const original = fs.readFileSync(env.script, "utf8");
  assert.ok(Buffer.byteLength(original) < limit, "reduced bound must accommodate exporter input");
  const changed = original.replace("const MAX_FILE_BYTES = 64 * 1024 * 1024;", "const MAX_FILE_BYTES = 128 * 1024;");
  assert.notEqual(changed, original, "copied-script boundary instrumentation must run");
  fs.writeFileSync(env.script, changed);
  const args = ["--platform", "codex", "--agents", "alpha-agent", "--no-skills"];
  const small = env.preview(args);
  const source = path.join(env.source, small.files[0].source.path);
  const line = 'path = "skills/example/alpha/SKILL.md"\n';
  const padding = Buffer.alloc(limit - Buffer.byteLength(line), 32);
  fs.writeFileSync(source, Buffer.concat([Buffer.from(line), padding]));
  fs.chmodSync(source, 0o644);
  const plan = env.preview(args);
  assert.ok(plan.files[0].final.size > limit, "destination rewrite must genuinely cross the source limit");
  const expected = {
    ".codex/agents/alpha-agent.toml": {
      bytes: Buffer.concat([Buffer.from("path = " + JSON.stringify(path.join(env.target, ".codex/skills/alpha")) + "\n"), padding]),
      mode: 0o644,
    },
  };
  assertSuccess(env.run(["--apply-plan", env.planFile]), plan, expected);
  assertFiles(env, expected);
});
