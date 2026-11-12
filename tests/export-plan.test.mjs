import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const repository = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const fixtures = path.join(repository, "tests/fixtures/export-plan");
// Captured with the unmodified legacy CLI BEFORE implementation. These final
// bytes, modes, stdout and stderr are fixed oracles, not planner-generated data.
const contracts = JSON.parse(fs.readFileSync(path.join(fixtures, "legacy-contracts.json"), "utf8"));
const hash = (value) => createHash("sha256").update(value).digest("hex");
function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object") return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  return JSON.stringify(value);
}
function digest(plan) {
  const { digest: _digest, ...body } = plan;
  return hash(canonical(body));
}
function setup(t) {
  const root = fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()), "vfa-plan-Ω "));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const source = path.join(root, "package 東京");
  fs.cpSync(path.join(fixtures, "package"), source, { recursive: true });
  fs.mkdirSync(path.join(source, "scripts"));
  const script = path.join(source, "scripts/export-marketplace-agents.mjs");
  fs.copyFileSync(path.join(repository, "scripts/export-marketplace-agents.mjs"), script);
  const target = path.join(root, "consumer Ω", "not yet created");
  function run(args, options = {}) {
    const result = spawnSync(process.execPath, [script, ...args, "--repo", target], {
      cwd: root, encoding: "utf8", timeout: 30000, maxBuffer: 10 * 1024 * 1024, ...options,
    });
    assert.ifError(result.error);
    assert.equal(result.signal, null, result.stderr);
    return result;
  }
  return { root, source, script, target, run };
}
function snapshot(root, directories = false) {
  if (!fs.existsSync(root)) return null;
  const result = {};
  function visit(dir) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const file = path.join(dir, entry.name);
      const relative = path.relative(root, file).split(path.sep).join("/");
      const stat = fs.lstatSync(file, { bigint: true });
      if (stat.isSymbolicLink()) result[relative] = { link: fs.readlinkSync(file) };
      else if (stat.isDirectory()) {
        if (directories) result[relative] = { directory: true, mtime: String(stat.mtimeNs), mode: Number(stat.mode & 0o777n) };
        visit(file);
      } else result[relative] = { base64: fs.readFileSync(file).toString("base64"), mode: Number(stat.mode & 0o777n), ...(directories ? { mtime: String(stat.mtimeNs) } : {}) };
    }
  }
  visit(root);
  return result;
}
function expectedFiles(contract, target) {
  return Object.fromEntries(Object.entries(contract.files).map(([relative, value]) => {
    const original = Buffer.from(value.base64, "base64");
    const bytes = relative.endsWith(".toml") ? Buffer.from(original.toString("utf8").replace(/"<TARGET>\/([^"\n]+)"/g, (_match, suffix) => JSON.stringify(path.join(target, suffix)))) : original;
    return [relative, { base64: bytes.toString("base64"), mode: value.mode }];
  }));
}
function expectedStdout(value) {
  return value.replace(/^(installed\t[^\n]*\t)([^\t\n]+)$/gm, (_match, prefix, relative) => prefix + relative.split("/").join(path.sep));
}
function argsFor(contract) { return contract.args.filter((arg) => arg !== "--dry-run"); }
function readPlan(result, status = 0) {
  assert.equal(result.status, status, result.stderr);
  const plan = JSON.parse(result.stdout);
  assert.equal(plan.protocol, "vfa-export/1");
  assert.equal(plan.type, "export-plan");
  assert.equal(plan.canonicalization, "sorted-keys-json/v1");
  assert.equal(result.stdout, canonical(plan) + "\n", "stdout is canonical protocol only");
  assert.equal(plan.digest, digest(plan));
  return plan;
}
function verifyFixedPlan(plan, env, contract) {
  assert.equal(plan.protocol, "vfa-export/1");
  assert.equal(plan.digest, digest(plan));
  assert.equal(plan.destination.root, env.target);
  assert.equal(plan.source.root, env.source);
  assert.equal(plan.source.sha256, hash(canonical(plan.source.inputs)));
  for (const input of plan.source.inputs) assert.equal(input.sha256, hash(fs.readFileSync(path.join(env.source, input.path))));
  assert.deepEqual(plan.source.catalogs.map((item) => [item.path, item.exists]), [
    ["catalog/agents.json", true], ["catalog/skills.json", true],
    ["catalog/install-roles.json", true], ["catalog/skill-manifest.json", false],
  ]);
  assert.equal(plan.selection_sha256, hash(canonical(plan.selection)));
  assert.equal(plan.options_sha256, hash(canonical(plan.options)));
  const platform = contract.name === "codex" ? "codex" : contract.name === "cursor" ? "cursor" : contract.name === "kiro" ? "kiro" : "claude-code";
  assert.deepEqual(plan.options, { platform, force: false, noSkills: contract.name === "no-skills" });
  const agents = contract.name === "provider" ? ["alpha-agent", "gamma-agent"] : ["alpha-agent"];
  assert.deepEqual(plan.selection.agents, agents);
  const skills = contract.name === "role-provider" ? ["alpha", "gamma", "shared-ref"] : contract.name === "provider" ? ["alpha", "gamma", "shared-ref"] : ["alpha", "rival", "shared-ref"];
  assert.deepEqual(plan.selection.skills, ["cursor", "kiro", "no-skills"].includes(contract.name) ? [] : skills);
  const expected = expectedFiles(contract, env.target);
  assert.deepEqual(plan.files.map((file) => file.destination.relative).sort(), Object.keys(expected).sort());
  for (const file of plan.files) {
    const rel = file.destination.relative;
    assert.equal(file.destination.path, path.join(env.target, rel));
    assert.equal(file.destination.exists, false);
    assert.equal(file.destination.sha256, null);
    assert.equal(file.action, "create");
    const bytes = Buffer.from(expected[rel].base64, "base64");
    assert.deepEqual(file.final, { sha256: hash(bytes), size: bytes.length, mode: expected[rel].mode });
    assert.equal(file.source.sha256, hash(fs.readFileSync(path.join(env.source, file.source.path))));
    if (file.kind === "agent") {
      const variantFile = { codex: "codex.toml", "claude-code": "claude.md", cursor: "cursor.md", "kiro-ide": "kiro.md", "kiro-cli": "kiro.json" }[file.variant];
      assert.equal(file.source.path, `agents/aws/${file.id}/${variantFile}`);
      assert.deepEqual(file.transformation, platform === "codex" ? {
        id: "codex-skill-paths/v1", options: { targetRoot: env.target, skillsDirectory: ".codex/skills" },
      } : { id: "identity/v1", options: {} });
    } else {
      const provider = file.id === "rival" ? "azure" : file.id === "shared-ref" ? "shared" : "aws";
      assert.equal(file.source.path, `skills/${provider}/${file.id}/${rel.split(`/skills/${file.id}/`)[1]}`);
      assert.deepEqual(file.transformation, { id: "identity/v1", options: {} });
    }
  }
  assert.equal(plan.confirmable, true);
  assert.deepEqual(plan.conflicts, []);
}

test("fixed pre-change legacy stdout/stderr, final bytes and modes remain compatible", (t) => {
  for (const contract of contracts) {
    const env = setup(t);
    const result = env.run(contract.args);
    assert.equal(result.status, 0, `${contract.name}: ${result.stderr}`);
    assert.equal(result.stdout.replaceAll(env.target, "<TARGET>"), expectedStdout(contract.stdout), contract.name);
    assert.equal(result.stderr.replaceAll(env.target, "<TARGET>"), contract.stderr, contract.name);
    assert.deepEqual(snapshot(env.target) ?? {}, expectedFiles(contract, env.target), contract.name);
    if (contract.overwrite) {
      for (const relative of Object.keys(contract.overwrite.files)) {
        const file = path.join(env.target, relative);
        fs.writeFileSync(file, "stale overwrite bytes\r\n");
        fs.chmodSync(file, 0o700);
      }
      const forced = env.run([...contract.args, "--force"]);
      assert.equal(forced.status, 0, forced.stderr);
      assert.equal(forced.stdout, expectedStdout(contract.overwrite.stdout));
      assert.equal(forced.stderr, contract.overwrite.stderr);
      assert.deepEqual(snapshot(env.target), expectedFiles(contract.overwrite, env.target), `${contract.name} overwrite`);
    }
  }
});

test("real CLI previews all legacy selections with fixed artifact oracles and no missing-root effects", (t) => {
  for (const contract of contracts.filter((item) => item.name !== "dry-run")) {
    const env = setup(t);
    const before = snapshot(env.root, true);
    const plan = readPlan(env.run([...argsFor(contract), "--plan-json"]));
    verifyFixedPlan(plan, env, contract);
    assert.equal(fs.existsSync(path.dirname(env.target)), false);
    assert.deepEqual(snapshot(env.root, true), before, contract.name);
    const second = readPlan(env.run([...argsFor(contract), "--plan-json"]));
    assert.equal(second.digest, plan.digest, "unchanged input identities are stable");
  }
});

test("existing trees have explicit no-force conflicts; human dry-run remains compatible; force matches final bytes", (t) => {
  const env = setup(t);
  const contract = contracts.find((item) => item.name === "claude");
  assert.equal(env.run(contract.args).status, 0);
  const destination = path.join(env.target, ".claude/agents/alpha-agent.md");
  fs.writeFileSync(destination, "existing user bytes\n");
  fs.chmodSync(destination, 0o600);
  const before = snapshot(env.target, true);
  const blocked = readPlan(env.run([...contract.args, "--plan-json"]), 1);
  assert.equal(blocked.confirmable, false);
  assert.equal(blocked.conflicts.length, 10);
  assert.ok(blocked.files.every((file) => file.action === "conflict"));
  assert.equal(blocked.files[0].destination.sha256, hash("existing user bytes\n"));
  assert.equal(env.run([...contract.args, "--dry-run"]).status, 0, "legacy preview may list would-be exports");
  assert.equal(env.run(contract.args).status, 1);
  assert.deepEqual(snapshot(env.target, true), before);
  const forced = readPlan(env.run([...contract.args, "--plan-json", "--force"]));
  assert.ok(forced.files.every((file) => file.action === "overwrite"));
  assert.deepEqual(snapshot(env.target, true), before);
  const result = env.run([...contract.args, "--force"]);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, expectedStdout(contract.stdout));
  assert.deepEqual(snapshot(env.target), expectedFiles(contract, env.target));
});

test("Codex rendering binds final Unicode destination; binary, CRLF, no-final-newline and executable companions are exact", (t) => {
  const env = setup(t);
  const args = ["--platform", "codex", "--agents", "alpha-agent", "--plan-json"];
  const plan = readPlan(env.run(args, { cwd: os.tmpdir() }));
  const changedRoot = path.join(env.root, "another 東京 path");
  // Use the real CLI for an independently located destination.
  const relocatedResult = spawnSync(process.execPath, [env.script, ...args, "--repo", changedRoot], { cwd: os.tmpdir(), encoding: "utf8" });
  const relocated = readPlan(relocatedResult);
  assert.notEqual(plan.files[0].source.sha256, plan.files[0].final.sha256);
  assert.equal(relocated.files[0].source.sha256, plan.files[0].source.sha256);
  assert.notEqual(relocated.files[0].final.sha256, plan.files[0].final.sha256);
  assert.notEqual(relocated.digest, plan.digest);
  assert.equal(fs.existsSync(changedRoot), false);
  assert.equal(env.run(args.slice(0, -1)).status, 0);
  for (const file of plan.files) {
    const bytes = fs.readFileSync(file.destination.path);
    assert.equal(hash(bytes), file.final.sha256);
    assert.equal(bytes.length, file.final.size);
    assert.equal(fs.statSync(file.destination.path).mode & 0o777, file.final.mode);
  }
});

test("fixed semantic and canonical-digest oracles reject negative mutations even with recomputed digests", (t) => {
  const env = setup(t);
  const contract = contracts.find((item) => item.name === "codex");
  const plan = readPlan(env.run([...contract.args, "--plan-json"]));
  verifyFixedPlan(plan, env, contract);
  for (const mutate of [
    (p) => { p.files[0].final.sha256 = p.files[0].source.sha256; },
    (p) => { p.files[0].destination.relative = "../escaped.toml"; },
    (p) => { p.files[0].transformation.id = "identity/v1"; },
    (p) => { p.selection.agents = ["beta-agent"]; },
    (p) => { p.source.inputs[0].sha256 = "0".repeat(64); },
  ]) {
    const mutated = structuredClone(plan);
    mutate(mutated);
    mutated.digest = digest(mutated);
    assert.throws(() => verifyFixedPlan(mutated, env, contract));
  }
  const changed = structuredClone(plan);
  changed.options.force = true;
  assert.notEqual(changed.digest, digest(changed));
});

test("provider, role, fallback and no-pair companion rules also cover --all and no-skills", (t) => {
  const env = setup(t);
  const all = readPlan(env.run(["--platform", "claude-code", "--all", "--plan-json"]));
  assert.deepEqual(all.selection.agents, ["alpha-agent", "gamma-agent", "beta-agent"]);
  assert.deepEqual(all.selection.skills, ["alpha", "gamma", "rival", "shared-ref"]);
  const role = readPlan(env.run(["--platform", "claude-code", "--role", "reviewers", "--plan-json"]));
  assert.deepEqual(role.selection.agents, ["alpha-agent", "beta-agent"]);
  assert.deepEqual(role.selection.skills, ["alpha", "gamma", "rival", "shared-ref"]);
  const beta = readPlan(env.run(["--platform", "claude-code", "--agents", "beta-agent", "--plan-json"]));
  assert.deepEqual(beta.selection.skills, []);
  const gamma = readPlan(env.run(["--platform", "claude-code", "--agents", "gamma-agent", "--plan-json"]));
  assert.deepEqual(gamma.selection.skills, ["gamma"]);
  const noSkills = readPlan(env.run(["--platform", "codex", "--all", "--no-skills", "--plan-json"]));
  assert.equal(noSkills.files.length, 3);
  assert.deepEqual(noSkills.selection.skills, []);
});

test("unknown/malformed/ambiguous selections and unavailable apply mode fail before stdout or writes", (t) => {
  const env = setup(t);
  for (const args of [
    ["--agents", "unknown"], ["--provider", "unknown"], ["--role", "unknown"],
    ["--agents", "alpha-agent,"], ["--agents", "alpha-agent,alpha-agent"],
    ["--agents", "alpha-agent", "--all"], ["--role", "reviewers", "--all"],
    ["--agents", "alpha-agent", "--provider", "aws"], [],
    ["--agents", "alpha-agent", "--dry-run"], ["--list"],
    ["--apply-plan", "anything.json"],
  ]) {
    const result = env.run(["--platform", "codex", "--plan-json", ...args]);
    assert.notEqual(result.status, 0, JSON.stringify(args));
    assert.equal(result.stdout, "", JSON.stringify(args));
    assert.ok(result.stderr.length > 0);
    assert.equal(fs.existsSync(path.dirname(env.target)), false);
  }
});

function mutateJson(file, mutate) {
  const data = JSON.parse(fs.readFileSync(file, "utf8"));
  mutate(data);
  fs.writeFileSync(file, JSON.stringify(data));
}

test("unsafe source metadata, traversal and duplicate destination definitions fail closed", (t) => {
  for (const mutate of [
    (e) => mutateJson(path.join(e.source, "agents/aws/alpha-agent/metadata.json"), (m) => { m.harness_variants.codex = "../outside.toml"; }),
    (e) => mutateJson(path.join(e.source, "agents/aws/alpha-agent/metadata.json"), (m) => { m.harness_variants.codex = "/tmp/outside.toml"; }),
    (e) => mutateJson(path.join(e.source, "agents/aws/alpha-agent/metadata.json"), (m) => { m.companion_skills = "alpha"; }),
    (e) => mutateJson(path.join(e.source, "agents/azure/beta-agent/metadata.json"), (m) => { m.id = "alpha-agent"; }),
    (e) => fs.writeFileSync(path.join(e.source, "skills/aws/alpha/metadata.json"), "{broken"),
    (e) => fs.cpSync(path.join(e.source, "skills/aws/alpha"), path.join(e.source, "skills/azure/alpha"), { recursive: true }),
    (e) => fs.rmSync(path.join(e.source, "agents/aws/alpha-agent/codex.toml")),
  ]) {
    const env = setup(t);
    mutate(env);
    const before = snapshot(env.root, true);
    const result = env.run(["--platform", "codex", "--agents", "alpha-agent", "--plan-json", "--force"]);
    assert.notEqual(result.status, 0, result.stdout);
    assert.equal(result.stdout, "");
    assert.ok(result.stderr.length > 0);
    assert.deepEqual(snapshot(env.root, true), before);
  }
  const env = setup(t);
  mutateJson(path.join(env.source, "catalog/install-roles.json"), (m) => { m.roles.reviewers.agents.push("missing-agent"); });
  assert.notEqual(env.run(["--platform", "codex", "--role", "reviewers", "--provider", "aws", "--plan-json"]).status, 0);
});

test("symlink ancestors/leaves and non-regular destinations are rejected without target effects", (t) => {
  for (const prepare of [
    (e) => { fs.mkdirSync(path.dirname(e.target), { recursive: true }); fs.symlinkSync(e.source, e.target, "dir"); },
    (e) => { fs.symlinkSync(e.source, path.dirname(e.target), "dir"); },
    (e) => { fs.mkdirSync(path.join(e.target, ".codex"), { recursive: true }); fs.symlinkSync(e.source, path.join(e.target, ".codex/agents"), "dir"); },
    (e) => { fs.mkdirSync(path.join(e.target, ".codex/agents"), { recursive: true }); fs.symlinkSync(path.join(e.root, "missing"), path.join(e.target, ".codex/agents/alpha-agent.toml")); },
    (e) => { fs.mkdirSync(path.join(e.target, ".codex/agents/alpha-agent.toml"), { recursive: true }); },
    (e) => { fs.mkdirSync(e.target, { recursive: true }); fs.writeFileSync(path.join(e.target, ".codex"), "not a directory"); },
    (e) => { const source = path.join(e.source, "agents/aws/alpha-agent/codex.toml"); fs.unlinkSync(source); fs.symlinkSync(path.join(e.source, "agents/aws/gamma-agent/codex.toml"), source); },
    (e) => { const source = path.join(e.source, "skills/aws/alpha/references"); fs.rmSync(source, { recursive: true }); fs.symlinkSync(e.root, source, "dir"); },
    (e) => { const source = path.join(e.source, "agents/aws/alpha-agent"); fs.rmSync(source, { recursive: true }); fs.symlinkSync(path.join(e.source, "agents/aws/gamma-agent"), source, "dir"); },
  ]) {
    const env = setup(t);
    prepare(env);
    const before = snapshot(env.root, true);
    for (const preview of [true, false]) {
      const result = env.run(["--platform", "codex", "--agents", "alpha-agent", "--force", ...(preview ? ["--plan-json"] : [])]);
      assert.notEqual(result.status, 0, result.stdout);
      assert.equal(result.stdout, "");
      assert.match(result.stderr, /symbolic link|non-directory|non-regular/i);
      assert.deepEqual(snapshot(env.root, true), before);
    }
  }
});

test("PLAN_LIMIT rejects oversized artifacts before output; never emits truncated success", (t) => {
  const env = setup(t);
  const source = path.join(env.source, "agents/aws/alpha-agent/codex.toml");
  fs.truncateSync(source, 64 * 1024 * 1024 + 1);
  const result = env.run(["--platform", "codex", "--agents", "alpha-agent", "--plan-json"]);
  assert.notEqual(result.status, 0);
  assert.equal(result.stdout, "");
  assert.match(result.stderr, /PLAN_LIMIT/);
  assert.equal(fs.existsSync(path.dirname(env.target)), false);
});


test("existing hard-link destination aliases cannot mutate sources or collide with other artifacts", (t) => {
  for (const aliasSource of [true, false]) {
    const env = setup(t);
    const agentsDir = path.join(env.target, ".codex/agents");
    fs.mkdirSync(agentsDir, { recursive: true });
    const destination = path.join(agentsDir, "alpha-agent.toml");
    const source = path.join(env.source, "agents/aws/alpha-agent/codex.toml");
    if (aliasSource) fs.linkSync(source, destination);
    else {
      fs.writeFileSync(destination, "shared existing destination\n");
      fs.linkSync(destination, path.join(agentsDir, "gamma-agent.toml"));
    }
    const before = snapshot(env.root, true);
    const result = env.run(["--platform", "codex", "--agents", aliasSource ? "alpha-agent" : "alpha-agent,gamma-agent", "--force", "--plan-json"]);
    assert.notEqual(result.status, 0);
    assert.equal(result.stdout, "");
    assert.match(result.stderr, /collision.*aliased/i);
    assert.deepEqual(snapshot(env.root, true), before);
  }
});

test("serialized protocol overflow fails explicitly with empty stdout and no target directory", (t) => {
  const env = setup(t);
  const references = path.join(env.source, "skills/aws/alpha/references");
  for (let index = 0; index < 7000; index += 1) {
    fs.writeFileSync(path.join(references, `${String(index).padStart(4, "0")}-${"x".repeat(180)}.txt`), "x");
  }
  const result = env.run(["--platform", "codex", "--agents", "alpha-agent", "--plan-json"], { timeout: 60000 });
  assert.notEqual(result.status, 0);
  assert.equal(result.stdout, "");
  assert.match(result.stderr, /PLAN_LIMIT: export plan exceeds 8 MiB/);
  assert.equal(fs.existsSync(path.dirname(env.target)), false);
});

test("hard-linked metadata, catalogs, executable and outside-target sentinels are never writable effects", (t) => {
  for (const sourceName of ["metadata", "catalog", "executable", "outside"]) {
    const env = setup(t);
    const protectedFile = {
      metadata: path.join(env.source, "agents/aws/alpha-agent/metadata.json"),
      catalog: path.join(env.source, "catalog/agents.json"),
      executable: env.script,
      outside: path.join(env.root, "outside-target-sentinel"),
    }[sourceName];
    if (sourceName === "outside") fs.writeFileSync(protectedFile, "outside content must survive\r\n");
    const destination = path.join(env.target, ".codex/agents/alpha-agent.toml");
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.linkSync(protectedFile, destination);
    const before = snapshot(env.root, true);
    for (const preview of [true, false]) {
      const result = env.run(["--platform", "codex", "--agents", "alpha-agent", "--force", ...(preview ? ["--plan-json"] : [])]);
      assert.notEqual(result.status, 0, `${sourceName}: unexpected success`);
      assert.equal(result.stdout, "");
      assert.match(result.stderr, /collision.*aliased/i);
      assert.deepEqual(snapshot(env.root, true), before);
    }
  }
});

test("JSON plan and help cannot report a successful human-only response", (t) => {
  const env = setup(t);
  const result = env.run(["--plan-json", "--help"]);
  assert.notEqual(result.status, 0);
  assert.equal(result.stdout, "");
  assert.match(result.stderr, /plan-json.*help/);
  assert.equal(fs.existsSync(path.dirname(env.target)), false);
  const help = env.run(["--help"]);
  assert.equal(help.status, 0);
  assert.match(help.stdout, /Usage:/);
});
