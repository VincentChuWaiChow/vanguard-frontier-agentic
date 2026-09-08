#!/usr/bin/env node
/**
 * Per-harness model + reasoning-effort policy engine for agent harness variants.
 *
 * Canonical intent lives in catalog/model-policy.json. This script projects
 * that policy into the executable harness files (codex.toml `model` /
 * `model_reasoning_effort` / `model_provider`, claude-code `.agent.md`
 * frontmatter `model:` / `effort:`, cursor `.agent.md` frontmatter `model:`)
 * and regenerates the resolved index catalog/model-assignments.json that
 * read-side tooling (vfa-tui) displays.
 *
 * Every model name and reasoning-effort value a rule projects is checked
 * against catalog/model-registry.json (schema: schemas/model-registry.schema.json),
 * a verified per-harness capability matrix. The engine fails closed: a model
 * that isn't registered, or a reasoning effort a model/namespace doesn't
 * support, is a validation error rather than a value silently projected that
 * would 404 or be dropped at request time. See docs/model-policy-matrix.md
 * for the human-readable matrix and .claude/skills/model-registry-refresh/
 * SKILL.md for the refresh workflow.
 *
 * codex model_provider: derived from the registry namespace that classifies
 * the assigned model (openai -> no line projected/default provider;
 * ollama/openrouter -> `model_provider = "ollama"`/`"openrouter"` projected).
 * The operator's codex config must define a matching [model_providers.<id>]
 * table for the route to actually work — the registry only flags the intent.
 *
 * Reasoning-effort vocabularies are per harness (catalog/model-registry.json
 * `reasoning_key` / `reasoning_efforts`) and, where the registry narrows
 * further, per model/namespace: codex projects `model_reasoning_effort` in
 * codex.toml; claude-code now also projects reasoning via the subagent
 * frontmatter `effort:` key; cursor has no reasoning passthrough.
 *
 * Model lifecycle (catalog/model-registry.json `status` / `retirement_date` /
 * `successor` on model entries): behavior is driven ONLY by the committed
 * `status` field, never Date.now(). "retiring" projects the pinned model
 * unchanged and emits a warning naming the documented successor. "retired"
 * projects the documented successor instead (chain-followed through any
 * further "retired" links) in place of the pinned model, and emits a louder
 * warning until the policy rule is migrated to pin the successor directly.
 * Warnings never fail `check` or change the exit code; they are aggregated
 * by message text and printed once each, with an affected-assignment count.
 *
 * Policy semantics:
 *   - Scopes: "all", "provider:<id>", "role:<id>", "agent:<id>".
 *   - Precedence: agent > role > provider > all, per field (model and
 *     reasoning_effort resolve independently).
 *   - Two rules in the same tier that disagree on the same agent+harness+field
 *     are a hard error (roles overlap by design, so equal values are allowed;
 *     conflicting values must be settled by an agent-level rule).
 *   - "auto" clears the field: the managed line is removed and the harness
 *     runtime default applies. Absence of any rule means "auto".
 *   - Fields are only projected into harnesses that support them (see
 *     HARNESS_CAPABILITIES). A rule targeting an unsupported field fails
 *     `check` — intent that cannot be enforced is treated as an error, not
 *     silently recorded.
 *
 * Commands:
 *   report [--json]     print resolved assignments per agent x harness
 *   check               validate policy + detect drift (exit 1 on violation)
 *   apply [--dry-run]   project policy into harness files + assignments index
 *   set --scope <all|provider=ID|role=ID|agent=ID|agents=a,b> --harness <id>
 *       [--model <name|auto>] [--reasoning <effort|auto>] [--dry-run]
 *                       upsert rule(s), then apply
 *   import-current [--force]
 *                       seed the policy from the values currently present in
 *                       harness files (bootstrap; refuses to overwrite an
 *                       existing policy without --force)
 *
 * After a non-dry-run apply/set, refresh the integrity manifest:
 *   npm run asset-integrity:write
 */

import { readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { dirname, isAbsolute, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const policyPath = join(repoRoot, "catalog", "model-policy.json");
const assignmentsPath = join(repoRoot, "catalog", "model-assignments.json");
const agentsCatalogPath = join(repoRoot, "catalog", "agents.json");
const rolesCatalogPath = join(repoRoot, "catalog", "install-roles.json");
const registryPath = join(repoRoot, "catalog", "model-registry.json");

/** Which policy fields each harness's executable format can express.
 * Only verified, officially supported keys are projected; inventing metadata
 * fields in executable agent files is forbidden (see CLAUDE.md cross-platform
 * asset rule). Extend this table only with documented harness support.
 * `reasoning_key` is the config key used to project reasoning_effort
 * (codex.toml key / frontmatter key); null = no projectable reasoning field. */
const HARNESS_CAPABILITIES = {
  codex: {
    model: true,
    reasoning_effort: true,
    reasoning_key: "model_reasoning_effort",
    variant: "codex",
    file: "codex.toml",
  },
  "claude-code": {
    model: true,
    reasoning_effort: true,
    reasoning_key: "effort",
    variant: "claude-code",
    file: "claude-code.agent.md",
  },
  cursor: { model: true, reasoning_effort: false, reasoning_key: null, variant: "cursor", file: "cursor.agent.md" },
  copilot: { model: false, reasoning_effort: false, reasoning_key: null, variant: "copilot", file: "copilot.agent.md" },
  gemini: { model: false, reasoning_effort: false, reasoning_key: null, variant: "gemini", file: "gemini.agent.md" },
  kiro: { model: false, reasoning_effort: false, reasoning_key: null, variant: "kiro-ide", file: "kiro-ide.agent.md" },
};

/** Harness-specific model-name character shape. codex model names may carry
 * ':' (Ollama name:tag) and '/' (OpenRouter author/model); every other
 * harness falls back to SAFE_VALUE. Values are embedded only inside
 * double-quoted TOML strings / YAML frontmatter and passed as argv, never
 * through a shell, so this is a shape check, not a shell-safety check. */
const MODEL_CHARSETS = {
  codex: /^[A-Za-z0-9][A-Za-z0-9._:/-]*$/,
  // Cursor documents per-model parameters appended to the model id in square
  // brackets (cursor.com/docs/subagents), e.g. claude-opus-5[effort=high].
  // The extra characters are confined to this harness so every other model
  // value keeps the tighter charset.
  cursor: /^[A-Za-z0-9][A-Za-z0-9._-]*(\[[A-Za-z0-9.=,]+\])?$/,
};

/** Cursor per-model parameter keys documented for subagent frontmatter.
 * Fail closed on anything else: an unknown key is silently ignored by Cursor,
 * which is exactly the "looks applied but isn't" failure this registry exists
 * to catch. */
const CURSOR_MODEL_PARAMS = new Set(["fast", "effort", "context"]);

/** A trailing "[k=v,...]" parameter group, the only bracketed form accepted. */
const MODEL_PARAM_SUFFIX_RE = /\[[^[\]]*\]$/;

/** The bare model id a value refers to, with any trailing parameter group
 * removed. Registry allowlists enumerate base ids only — the parameters are a
 * per-rule modifier, not a different model — so every closed-membership
 * lookup goes through this. Values without a suffix are returned unchanged. */
function modelBase(value) {
  return typeof value === "string" ? value.replace(MODEL_PARAM_SUFFIX_RE, "") : value;
}

/** Validate a value's parameter group, if it has one. Returns an error string
 * or null. */
function modelParamsError(value) {
  if (typeof value !== "string") return null;
  const match = value.match(MODEL_PARAM_SUFFIX_RE);
  if (!match) return null;
  const body = match[0].slice(1, -1);
  if (body.length === 0) return "empty parameter group \"[]\"";
  const seen = new Set();
  for (const pair of body.split(",")) {
    const eq = pair.indexOf("=");
    if (eq <= 0 || eq === pair.length - 1) return `malformed parameter "${pair}" (expected key=value)`;
    const key = pair.slice(0, eq);
    if (!CURSOR_MODEL_PARAMS.has(key)) {
      return `unknown parameter "${key}" (documented: ${[...CURSOR_MODEL_PARAMS].join(", ")})`;
    }
    if (seen.has(key)) return `duplicate parameter "${key}"`;
    seen.add(key);
  }
  return null;
}

const SAFE_VALUE = /^[A-Za-z0-9][A-Za-z0-9._-]*$/;
const SCOPE_TIERS = ["all", "provider", "role", "agent"];

/** Registry-hardening limits. The registry is attacker-controllable catalog
 * data in a PR; every value below is compiled/interpolated by this script, so
 * it is validated fail-closed before use. */
const MAX_MATCH_PATTERN_LENGTH = 200;
/** A quantifier ( + * {n,} ) applied to a group that itself ends in an
 * unbounded quantifier — the catastrophic-backtracking shape, e.g. (a+)+. */
const NESTED_QUANTIFIER_RE = /\([^()]*[+*][^()]*\)[+*{]/;
/** Provider ids and reasoning-effort tokens embedded into codex.toml /
 * frontmatter. Lowercase-alnum with internal separators; no quotes, spaces,
 * newlines, or shell/markup metacharacters. */
const SAFE_TOKEN = /^[a-z0-9][a-z0-9._-]*$/;

// ── shared loading ───────────────────────────────────────────────────────────

/** Resolve a harness-variant path and confirm it resolves to a file under the
 * `agents/` tree. Returns the absolute path, or null if it escapes the
 * repository (`..` / absolute) OR resolves anywhere outside `agents/`. Catalog
 * data (agent.path / harness_variants) is attacker-controllable in a PR, and
 * these paths are both read and written; repo-containment alone is not enough
 * because sensitive files like `.git/config`, `package.json`, or a root file
 * are lexically *inside* the repo. Every one of the 1,100+ real variant paths
 * lives under `agents/`, so scoping the guard there is both correct and tight —
 * it blocks `.git/`, root files, and traversal in one check. Stricter than the
 * `path_is_inside_repo` sibling validators (which this hereby supersedes for
 * the codex/read-write path); lexical only, no symlink resolution. */
function resolveInsideRepo(relPath) {
  if (typeof relPath !== "string" || relPath.length === 0) return null;
  const abs = join(repoRoot, relPath);
  const rel = relative(repoRoot, abs);
  if (rel === "" || rel.startsWith("..") || isAbsolute(rel)) return null;
  if (rel.split(/[\\/]/)[0] !== "agents") return null;
  return abs;
}

/** Read a file as UTF-8, or return null if it does not exist. Avoids the
 * TOCTOU race of existsSync-then-read: a single syscall either returns the
 * contents or reports ENOENT, leaving no window between check and use. */
function readFileOrNull(path) {
  try {
    return readFileSync(path, "utf8");
  } catch (e) {
    if (e && e.code === "ENOENT") return null;
    throw e;
  }
}

function loadJson(path, label) {
  const raw = readFileOrNull(path);
  if (raw === null) {
    fail(2, `ERROR: ${label} not found at ${relative(repoRoot, path)}`);
  }
  try {
    return JSON.parse(raw);
  } catch (e) {
    fail(2, `ERROR: ${label} is not valid JSON: ${e.message}`);
  }
}

/** Load JSON, returning `fallback` when the file is absent. */
function loadJsonOrDefault(path, label, fallback) {
  const raw = readFileOrNull(path);
  if (raw === null) return fallback;
  try {
    return JSON.parse(raw);
  } catch (e) {
    fail(2, `ERROR: ${label} is not valid JSON: ${e.message}`);
  }
}

function fail(code, ...lines) {
  for (const l of lines) console.error(l);
  process.exit(code);
}

function loadAgents() {
  const catalog = loadJson(agentsCatalogPath, "agent catalog");
  return catalog.filter((e) => e.type === "agent");
}

function loadRoles() {
  return loadJson(rolesCatalogPath, "role catalog").roles;
}

function sha256Hex(text) {
  return createHash("sha256").update(text).digest("hex");
}

// ── model registry (catalog/model-registry.json) ────────────────────────────

/** Structural validation of the raw registry document. Returns a list of
 * error strings (empty = valid). Does not build the compiled lookup
 * structure — call compileRegistry() once this returns no errors. */
function validateRegistry(doc) {
  const errors = [];
  if (!doc || typeof doc !== "object" || Array.isArray(doc)) {
    return ["model registry root must be an object"];
  }
  if (doc.manifest_version !== 1) {
    errors.push("manifest_version must be 1");
  }
  if (!doc.harnesses || typeof doc.harnesses !== "object" || Array.isArray(doc.harnesses)) {
    errors.push("harnesses must be an object");
    return errors;
  }
  const requiredHarnesses = Object.entries(HARNESS_CAPABILITIES)
    .filter(([, caps]) => caps.model)
    .map(([h]) => h);
  for (const h of requiredHarnesses) {
    if (!doc.harnesses[h]) errors.push(`harnesses.${h} is required`);
  }
  for (const [harness, hdef] of Object.entries(doc.harnesses)) {
    const where = `harnesses.${harness}`;
    if (!hdef || typeof hdef !== "object") {
      errors.push(`${where} must be an object`);
      continue;
    }
    if (hdef.reasoning_key !== null && typeof hdef.reasoning_key !== "string") {
      errors.push(`${where}.reasoning_key must be a string or null`);
    }
    if (!Array.isArray(hdef.reasoning_efforts)) {
      errors.push(`${where}.reasoning_efforts must be an array of strings`);
    } else {
      // Each vocab element is interpolated into codex.toml / frontmatter
      // (`model_reasoning_effort = "..."` / `effort: "..."`); charset-check it
      // so a crafted registry can't smuggle a quote/newline through membership.
      for (const eff of hdef.reasoning_efforts) {
        if (typeof eff !== "string" || !SAFE_TOKEN.test(eff)) {
          errors.push(
            `${where}.reasoning_efforts contains an unsafe value ${JSON.stringify(eff)}`,
          );
        }
      }
    }
    if (!Array.isArray(hdef.namespaces) || hdef.namespaces.length === 0) {
      errors.push(`${where}.namespaces must be a non-empty array`);
      continue;
    }
    const efforts = Array.isArray(hdef.reasoning_efforts) ? hdef.reasoning_efforts : [];
    hdef.namespaces.forEach((ns, i) => {
      const nsWhere = `${where}.namespaces[${i}]`;
      if (!ns || typeof ns.id !== "string") {
        errors.push(`${nsWhere}.id is required`);
        return;
      }
      if (typeof ns.match !== "string") {
        errors.push(`${nsWhere}.match must be a string`);
        return;
      }
      if (!ns.match.startsWith("^") || !ns.match.endsWith("$")) {
        errors.push(`${nsWhere}.match must be anchored (^...$): "${ns.match}"`);
        return;
      }
      // ReDoS guard: registry data is attacker-controllable in a PR, and this
      // pattern is compiled and run in validate:model-policy (CI). Reject
      // over-long patterns and nested unbounded quantifiers (the classic
      // catastrophic-backtracking shape, e.g. (a+)+, (a*)+ ) before compiling.
      // Node's RegExp has no step budget, so this is the only line of defense.
      if (ns.match.length > MAX_MATCH_PATTERN_LENGTH) {
        errors.push(
          `${nsWhere}.match is too long (${ns.match.length} > ${MAX_MATCH_PATTERN_LENGTH}); simplify the namespace pattern`,
        );
        return;
      }
      if (NESTED_QUANTIFIER_RE.test(ns.match)) {
        errors.push(
          `${nsWhere}.match contains a nested unbounded quantifier (ReDoS risk): "${ns.match}"`,
        );
        return;
      }
      let re;
      try {
        re = new RegExp(ns.match);
      } catch (e) {
        errors.push(`${nsWhere}.match is not a valid regular expression: ${e.message}`);
        return;
      }
      if (ns.membership !== "closed" && ns.membership !== "open") {
        errors.push(`${nsWhere}.membership must be "closed" or "open"`);
        return;
      }
      if (ns.membership === "closed" && !Array.isArray(ns.models)) {
        errors.push(`${nsWhere}: membership "closed" requires a models array`);
      }
      // model_provider is interpolated into codex.toml (`model_provider = "..."`).
      if (
        ns.model_provider !== undefined &&
        ns.model_provider !== null &&
        (typeof ns.model_provider !== "string" || !SAFE_TOKEN.test(ns.model_provider))
      ) {
        errors.push(
          `${nsWhere}.model_provider must be null or a safe token, got ${JSON.stringify(ns.model_provider)}`,
        );
      }
      if (Array.isArray(ns.models)) {
        const nsModelIds = new Set(
          ns.models.filter((m) => m && typeof m.id === "string").map((m) => m.id),
        );
        const modelsById = new Map(
          ns.models.filter((m) => m && typeof m.id === "string").map((m) => [m.id, m]),
        );
        ns.models.forEach((model, j) => {
          const mWhere = `${nsWhere}.models[${j}]`;
          if (!model || typeof model.id !== "string") {
            errors.push(`${mWhere}.id is required`);
            return;
          }
          if (!re.test(model.id)) {
            errors.push(
              `${mWhere}: model id "${model.id}" does not match namespace "${ns.id}" match pattern "${ns.match}"`,
            );
          }
          // model.id is interpolated into `model = "..."`; a crafted registry
          // could pair a permissive namespace `match` with a quote/newline in
          // the id, so charset-check it against the harness's own shape.
          const idCharset = MODEL_CHARSETS[harness] ?? SAFE_VALUE;
          if (!idCharset.test(model.id)) {
            errors.push(
              `${mWhere}: model id "${model.id}" contains characters unsafe for ${harness} projection`,
            );
          }
          if (model.reasoning_efforts !== undefined) {
            if (!Array.isArray(model.reasoning_efforts)) {
              errors.push(`${mWhere}.reasoning_efforts must be an array`);
            } else {
              for (const eff of model.reasoning_efforts) {
                if (!efforts.includes(eff)) {
                  errors.push(
                    `${mWhere}.reasoning_efforts value "${eff}" is not in the "${harness}" reasoning_efforts vocabulary`,
                  );
                }
              }
            }
          }
          if (
            model.status !== undefined &&
            !["available", "retiring", "retired"].includes(model.status)
          ) {
            errors.push(`${mWhere}.status must be one of available|retiring|retired`);
          }
          if (model.retirement_date !== undefined) {
            if (
              typeof model.retirement_date !== "string" ||
              !/^\d{4}-\d{2}-\d{2}$/.test(model.retirement_date)
            ) {
              errors.push(`${mWhere}.retirement_date must match YYYY-MM-DD`);
            }
          }
          if (model.successor !== undefined) {
            if (typeof model.successor !== "string" || !nsModelIds.has(model.successor)) {
              errors.push(
                `${mWhere}.successor "${model.successor}" must name an existing model id in namespace "${ns.id}"`,
              );
            }
          }
          if (model.status === "retired" && !model.successor) {
            errors.push(`${mWhere}: status "retired" requires a successor`);
          }
        });
        // Successor chains must terminate at a model whose status is not
        // "retired" within 5 hops, with no cycles — mirrors resolveLifecycle's
        // rule of following successor links only while the current entry is
        // "retired".
        ns.models.forEach((model, j) => {
          if (!model || typeof model.id !== "string" || model.successor === undefined) return;
          const mWhere = `${nsWhere}.models[${j}]`;
          const visited = new Set([model.id]);
          let current = model;
          let hops = 0;
          while (current.status === "retired") {
            const nextId = current.successor;
            if (nextId === undefined || !modelsById.has(nextId)) break; // reported separately
            if (visited.has(nextId)) {
              errors.push(`${mWhere}: successor chain from "${model.id}" has a cycle at "${nextId}"`);
              break;
            }
            hops++;
            if (hops > 5) {
              errors.push(
                `${mWhere}: successor chain from "${model.id}" does not terminate within 5 hops`,
              );
              break;
            }
            visited.add(nextId);
            current = modelsById.get(nextId);
          }
        });
      }
      if (ns.reasoning_efforts !== undefined) {
        if (!Array.isArray(ns.reasoning_efforts)) {
          errors.push(`${nsWhere}.reasoning_efforts must be an array`);
        } else {
          for (const eff of ns.reasoning_efforts) {
            if (!efforts.includes(eff)) {
              errors.push(
                `${nsWhere}.reasoning_efforts value "${eff}" is not in the "${harness}" reasoning_efforts vocabulary`,
              );
            }
          }
        }
      }
    });
  }
  return errors;
}

/** Build the compiled lookup structure from an already-validated registry
 * document: Map harness -> { reasoningKey, efforts, namespaces }, where each
 * namespace carries a compiled RegExp and a Map(id -> model entry). */
function compileRegistry(doc) {
  const compiled = new Map();
  for (const [harness, hdef] of Object.entries(doc.harnesses)) {
    const namespaces = (hdef.namespaces || []).map((ns) => {
      const models = new Map();
      for (const model of ns.models || []) {
        models.set(model.id, {
          ...model,
          status: model.status ?? "available",
          retirement_date: model.retirement_date ?? null,
          successor: model.successor ?? null,
        });
      }
      return {
        id: ns.id,
        re: new RegExp(ns.match),
        membership: ns.membership,
        modelProvider: ns.model_provider ?? null,
        nsEfforts: ns.reasoning_efforts ?? null,
        models,
      };
    });
    compiled.set(harness, {
      reasoningKey: hdef.reasoning_key ?? null,
      efforts: hdef.reasoning_efforts || [],
      namespaces,
    });
  }
  return compiled;
}

function loadRegistry() {
  const doc = loadJson(registryPath, "model registry");
  const errors = validateRegistry(doc);
  if (errors.length > 0) {
    fail(2, "ERROR: model registry is invalid:", ...errors.map((e) => "  " + e));
  }
  return compileRegistry(doc);
}

/** First namespace (in registry order) whose match regex accepts `value`,
 * or null if none does. */
function classifyModel(harnessReg, value) {
  if (!harnessReg) return null;
  for (const ns of harnessReg.namespaces) {
    if (ns.re.test(value)) return ns;
  }
  return null;
}

/** Verified reasoning-effort vocabulary for a specific model value, or null
 * if the model doesn't classify into any namespace at all (distinct from an
 * empty array, which means "classified, but no reasoning passthrough"). */
function effortsForModel(harnessReg, value) {
  const ns = classifyModel(harnessReg, value);
  if (!ns) return null;
  const base = modelBase(value);
  if (ns.membership === "closed" && ns.models.has(base)) {
    const entry = ns.models.get(base);
    if (entry.reasoning_efforts !== undefined) return entry.reasoning_efforts;
  }
  return ns.nsEfforts ?? harnessReg.efforts;
}

/** Resolve provider lifecycle for one classified model value: follow
 * `successor` links while the current entry's `status` is "retired"
 * (registry validation already guarantees these chains terminate at a
 * non-retired model within 5 hops with no cycles), producing the effective
 * model to project plus an operator-facing warning. Open namespaces and
 * values that don't classify into a closed namespace, or aren't listed in
 * one, are returned unchanged with no warning — lifecycle tracking only
 * applies to the verified allowlist. Never consults the wall clock. */
function resolveLifecycle(harnessReg, ns, value) {
  const base = modelBase(value);
  const suffix = value.slice(base.length);
  if (!ns || ns.membership !== "closed" || !ns.models.has(base)) {
    return { model: value, fallbackFrom: null, warning: null };
  }
  const original = ns.models.get(base);
  let current = original;
  while (current.status === "retired" && current.successor && ns.models.has(current.successor)) {
    current = ns.models.get(current.successor);
  }
  if (current.id !== base) {
    // Carry the parameter group onto the successor: the operator's intent for
    // effort/context is independent of which snapshot serves the request.
    const projected = `${current.id}${suffix}`;
    let warning = `model "${value}" was retired by the provider — projecting documented successor "${projected}"; migrate the policy rule`;
    if (current.status === "retiring") {
      const chainedDatePart = current.retirement_date ? ` on ${current.retirement_date}` : "";
      warning += ` (note: "${current.id}" is itself scheduled for retirement${chainedDatePart} — successor: ${current.successor ?? "none announced"})`;
    }
    return {
      model: projected,
      fallbackFrom: value,
      warning,
    };
  }
  if (original.status === "retiring") {
    const datePart = original.retirement_date ? ` on ${original.retirement_date}` : "";
    return {
      model: value,
      fallbackFrom: null,
      warning: `model "${value}" is scheduled for retirement${datePart} — documented successor: ${original.successor ?? "none announced"}; plan a policy migration`,
    };
  }
  return { model: value, fallbackFrom: null, warning: null };
}

// ── policy parsing + validation ──────────────────────────────────────────────

function parseScope(scope) {
  if (scope === "all") return { tier: "all", id: null };
  const m = /^(provider|role|agent):([a-z0-9][a-z0-9-]*)$/.exec(scope);
  if (!m) return null;
  return { tier: m[1], id: m[2] };
}

/** Structural + referential validation. Returns a list of error strings. */
function validatePolicy(policy, agents, roles, registry) {
  const errors = [];
  if (!policy || typeof policy !== "object" || Array.isArray(policy)) {
    return ["policy root must be an object"];
  }
  if (policy.manifest_version !== 1) {
    errors.push("manifest_version must be 1");
  }
  if (policy.defaults) {
    if (policy.defaults.model !== "auto" || policy.defaults.reasoning_effort !== "auto") {
      errors.push('defaults must be {"model": "auto", "reasoning_effort": "auto"}');
    }
  }
  if (!Array.isArray(policy.rules)) {
    errors.push("rules must be an array");
    return errors;
  }
  const agentIds = new Set(agents.map((a) => a.id));
  const providers = new Set(agents.map((a) => a.provider));
  const seen = new Set();
  policy.rules.forEach((rule, i) => {
    const where = `rules[${i}]`;
    const known = new Set(["scope", "harness", "model", "reasoning_effort"]);
    for (const k of Object.keys(rule)) {
      if (!known.has(k)) errors.push(`${where}: unknown field "${k}"`);
    }
    const scope = typeof rule.scope === "string" ? parseScope(rule.scope) : null;
    if (!scope) {
      errors.push(`${where}: invalid scope "${rule.scope}"`);
      return;
    }
    const caps = HARNESS_CAPABILITIES[rule.harness];
    if (!caps) {
      errors.push(`${where}: unknown harness "${rule.harness}"`);
      return;
    }
    if (rule.model === undefined && rule.reasoning_effort === undefined) {
      errors.push(`${where}: must set model and/or reasoning_effort`);
    }
    if (rule.model !== undefined) {
      if (!caps.model) {
        errors.push(`${where}: harness "${rule.harness}" does not support model pinning`);
      } else if (rule.model !== "auto") {
        const charset = MODEL_CHARSETS[rule.harness] ?? SAFE_VALUE;
        if (typeof rule.model !== "string" || !charset.test(rule.model)) {
          errors.push(`${where}: unsafe model value "${rule.model}"`);
        } else {
          const harnessReg = registry.get(rule.harness);
          if (!harnessReg) {
            errors.push(`${where}: no model registry entry for harness "${rule.harness}"`);
          } else {
            const ns = classifyModel(harnessReg, rule.model);
            const paramsError = modelParamsError(rule.model);
            if (!ns) {
              const nsList = harnessReg.namespaces.map((n) => `${n.id}: ${n.re.source}`).join("; ");
              errors.push(
                `${where}: model "${rule.model}" does not match any ${rule.harness} model namespace (${nsList}); see docs/model-policy-matrix.md`,
              );
            } else if (paramsError) {
              errors.push(
                `${where}: model "${rule.model}" has an invalid parameter group — ${paramsError}`,
              );
            } else if (ns.membership === "closed" && !ns.models.has(modelBase(rule.model))) {
              errors.push(
                `${where}: model "${rule.model}" is not in the verified model registry for ${rule.harness} namespace "${ns.id}" (catalog/model-registry.json); verify it against official docs and add it via the model-registry-refresh workflow (.claude/skills/model-registry-refresh/SKILL.md)`,
              );
            } else if (ns.membership === "closed" && ns.models.has(modelBase(rule.model))) {
              // Defense-in-depth: registry validation already requires a
              // successor whenever status is "retired", but a rule pinning
              // such a model with no successor to fall back to must still be
              // rejected here so it can never be applied.
              const entry = ns.models.get(modelBase(rule.model));
              if (entry.status === "retired" && !entry.successor) {
                errors.push(
                  `${where}: model "${rule.model}" was retired with no documented successor — the rule must be migrated`,
                );
              }
            }
          }
        }
      }
    }
    if (rule.reasoning_effort !== undefined) {
      if (!caps.reasoning_effort) {
        errors.push(`${where}: harness "${rule.harness}" does not support reasoning_effort`);
      } else if (rule.reasoning_effort !== "auto") {
        const harnessReg = registry.get(rule.harness);
        const efforts = harnessReg ? harnessReg.efforts : [];
        if (!efforts.includes(rule.reasoning_effort)) {
          errors.push(`${where}: reasoning_effort "${rule.reasoning_effort}" must be auto|${efforts.join("|")}`);
        }
      }
    }
    if (scope.tier === "provider" && !providers.has(scope.id)) {
      errors.push(`${where}: unknown provider "${scope.id}"`);
    }
    if (scope.tier === "role" && !roles[scope.id]) {
      errors.push(`${where}: unknown role "${scope.id}"`);
    }
    if (scope.tier === "agent" && !agentIds.has(scope.id)) {
      errors.push(`${where}: unknown agent "${scope.id}"`);
    }
    const dupKey = `${rule.scope}::${rule.harness}`;
    if (seen.has(dupKey)) {
      errors.push(`${where}: duplicate rule for scope "${rule.scope}" harness "${rule.harness}"`);
    }
    seen.add(dupKey);
  });
  return errors;
}

// ── resolution ───────────────────────────────────────────────────────────────

/** Resolve effective {model, reasoning_effort, sources} for one agent+harness.
 *
 * Only the highest-priority tier (agent > role > provider > all) that actually
 * matches a field decides that field. A conflict is reported *only* when the
 * winning tier itself carries two different values — lower-tier disagreements
 * are irrelevant because a higher tier overrides them. This preserves the
 * documented escape hatch: adding a single `agent:<id>` rule resolves a
 * role-overlap conflict instead of being rejected alongside it. */
function resolveAgentHarness(agent, harness, policy, roleIndex, errors) {
  const result = {
    model: "auto",
    reasoning_effort: "auto",
    model_source: "default",
    reasoning_source: "default",
  };
  // Collect matching rules per field, grouped by tier.
  const hitsByTier = { model: new Map(), reasoning_effort: new Map() };
  for (const rule of policy.rules) {
    if (rule.harness !== harness) continue;
    const scope = parseScope(rule.scope);
    if (!scope) continue;
    const matches =
      scope.tier === "all" ||
      (scope.tier === "provider" && scope.id === agent.provider) ||
      (scope.tier === "role" && (roleIndex.get(scope.id) || new Set()).has(agent.id)) ||
      (scope.tier === "agent" && scope.id === agent.id);
    if (!matches) continue;
    for (const field of ["model", "reasoning_effort"]) {
      if (rule[field] === undefined) continue;
      if (!hitsByTier[field].has(scope.tier)) hitsByTier[field].set(scope.tier, []);
      hitsByTier[field].get(scope.tier).push({ rule, value: rule[field] });
    }
  }
  for (const field of ["model", "reasoning_effort"]) {
    // Highest-priority tier with any matching rule wins outright.
    let winningTier = null;
    for (const tier of SCOPE_TIERS) {
      if (hitsByTier[field].has(tier)) winningTier = tier;
    }
    if (winningTier === null) continue;
    const hits = hitsByTier[field].get(winningTier);
    const values = new Set(hits.map((h) => h.value));
    if (values.size > 1) {
      errors.push(
        `conflict: agent "${agent.id}" harness "${harness}" gets ${field} values ` +
          `[${[...values].join(", ")}] from ${winningTier}-tier rules ` +
          `(${hits.map((h) => h.rule.scope).join(", ")}); add an agent-level override`,
      );
      continue;
    }
    result[field] = hits[0].value;
    const src = field === "model" ? "model_source" : "reasoning_source";
    result[src] = hits[0].rule.scope;
  }
  return result;
}

function buildRoleIndex(roles) {
  const index = new Map();
  for (const [roleId, role] of Object.entries(roles)) {
    index.set(roleId, new Set(role.agents || []));
  }
  return index;
}

/** Resolve the full assignment table. Returns { assignments, errors, warnings }.
 * `warnings` is a flat array of message strings, one per assignment carrying
 * a lifecycle warning (duplicates included; callers aggregate for display). */
function resolveAll(policy, agents, roles, registry) {
  const errors = [];
  const warnings = [];
  const roleIndex = buildRoleIndex(roles);
  const assignments = [];
  const sorted = [...agents].sort((a, b) => a.id.localeCompare(b.id));
  for (const agent of sorted) {
    for (const [harness, caps] of Object.entries(HARNESS_CAPABILITIES)) {
      if (!caps.model && !caps.reasoning_effort) continue;
      const variantRel =
        agent.harness_variants?.[caps.variant] ?? `${agent.path}/harnesses/${caps.file}`;
      const variantAbs = resolveInsideRepo(variantRel);
      if (variantAbs === null) {
        errors.push(
          `unsafe variant path for agent "${agent.id}" harness "${harness}": "${variantRel}" escapes the repository`,
        );
        continue;
      }
      if (readFileOrNull(variantAbs) === null) continue;
      const r = resolveAgentHarness(agent, harness, policy, roleIndex, errors);
      const harnessReg = registry.get(harness);
      // Resolve provider lifecycle (status: available|retiring|retired) on
      // the winning model BEFORE model_provider/efforts derivation, so a
      // "retired" pin projects its documented successor everywhere else.
      let effectiveModel = r.model;
      let modelFallbackFrom = null;
      let modelWarning = null;
      if (r.model !== "auto") {
        const pinnedNs = classifyModel(harnessReg, r.model);
        const lifecycle = resolveLifecycle(harnessReg, pinnedNs, r.model);
        effectiveModel = lifecycle.model;
        modelFallbackFrom = lifecycle.fallbackFrom;
        modelWarning = lifecycle.warning;
        if (modelWarning) warnings.push(modelWarning);
      }
      const modelProvider =
        effectiveModel === "auto"
          ? null
          : classifyModel(harnessReg, effectiveModel)?.modelProvider ?? null;
      if (caps.reasoning_effort && effectiveModel !== "auto" && r.reasoning_effort !== "auto") {
        const efforts = effortsForModel(harnessReg, effectiveModel);
        if (efforts !== null && !efforts.includes(r.reasoning_effort)) {
          errors.push(
            `incompatible: agent "${agent.id}" harness "${harness}" model "${effectiveModel}" (from ${r.model_source}) ` +
              `does not support reasoning_effort "${r.reasoning_effort}" (from ${r.reasoning_source}); ` +
              (efforts.length
                ? `supported: ${efforts.join("|")}`
                : "no verified reasoning passthrough for this route — set reasoning to auto"),
          );
        }
      }
      assignments.push({
        agent_id: agent.id,
        harness,
        model: effectiveModel === "auto" ? null : effectiveModel,
        model_provider: modelProvider,
        model_fallback_from: modelFallbackFrom,
        model_warning: modelWarning,
        reasoning_effort: caps.reasoning_effort
          ? r.reasoning_effort === "auto"
            ? null
            : r.reasoning_effort
          : null,
        model_source: r.model_source,
        reasoning_source: caps.reasoning_effort ? r.reasoning_source : "default",
        file: variantRel,
      });
    }
  }
  return { assignments, errors, warnings };
}

// ── surgical file editing ────────────────────────────────────────────────────

/** Set/replace/remove a managed top-level key in a codex.toml file.
 * Only lines before the first table header ([...]) and outside triple-quoted
 * strings are considered, so prose inside developer_instructions can never be
 * mistaken for a config key. `value === null` removes the line. */
function editTomlKey(content, key, value) {
  const lines = content.split("\n");
  let inTriple = false;
  const keyRe = new RegExp(`^${key}\\s*=`);
  let keyLine = -1;
  let nameLine = -1;
  let descLine = -1;
  let modelLine = -1;
  for (let i = 0; i < lines.length; i++) {
    const tripleCount = (lines[i].match(/"""/g) || []).length;
    if (!inTriple) {
      // Stop at the first table header ([...]): only top-level keys are managed.
      if (/^\[/.test(lines[i])) {
        break;
      }
      if (keyRe.test(lines[i]) && keyLine === -1) keyLine = i;
      if (/^name\s*=/.test(lines[i])) nameLine = i;
      if (/^description\s*=/.test(lines[i]) && descLine === -1) descLine = i;
      // Capture the real top-level `model =` line in the SAME triple-quote-aware
      // pass, so a decoy `model = ...` line inside a triple-quoted string can
      // never be chosen as the insertion anchor (which would splice the new key
      // into the middle of that string).
      if (/^model\s*=/.test(lines[i]) && modelLine === -1) modelLine = i;
    }
    if (tripleCount % 2 === 1) inTriple = !inTriple;
  }
  const rendered = value === null ? null : `${key} = "${value}"`;
  if (keyLine >= 0) {
    if (rendered === null) {
      lines.splice(keyLine, 1);
    } else if (lines[keyLine] !== rendered) {
      lines[keyLine] = rendered;
    } else {
      return content;
    }
    return lines.join("\n");
  }
  if (rendered === null) return content;
  // Insert after `model =` for the reasoning key, else after description/name.
  let anchor = descLine >= 0 ? descLine : nameLine;
  if (key === "model_reasoning_effort" && modelLine >= 0) {
    anchor = modelLine;
  }
  lines.splice(anchor + 1, 0, rendered);
  return lines.join("\n");
}

/** Set/replace/remove a managed key in a YAML frontmatter block (the block
 * between the leading `---` fence pair). `value === null` removes the line.
 * Insertion order: `model` is inserted just before the closing fence (its
 * long-standing position); any other key (e.g. `effort`) is inserted directly
 * after an existing `model:` line when present, else also before the closing
 * fence. */
function editFrontmatterKey(content, key, value) {
  const lines = content.split("\n");
  if (lines[0] !== "---") return content;
  let close = -1;
  for (let i = 1; i < lines.length; i++) {
    if (lines[i] === "---") {
      close = i;
      break;
    }
  }
  if (close === -1) return content;
  const keyRe = new RegExp(`^${key}:`);
  const rendered = value === null ? null : `${key}: "${value}"`;
  for (let i = 1; i < close; i++) {
    if (keyRe.test(lines[i])) {
      if (rendered === null) {
        lines.splice(i, 1);
      } else if (lines[i] !== rendered) {
        lines[i] = rendered;
      } else {
        return content;
      }
      return lines.join("\n");
    }
  }
  if (rendered === null) return content;
  let insertAt = close;
  if (key !== "model") {
    for (let i = 1; i < close; i++) {
      if (/^model:/.test(lines[i])) {
        insertAt = i + 1;
        break;
      }
    }
  }
  lines.splice(insertAt, 0, rendered);
  return lines.join("\n");
}

/** Compute the projected content for one assignment. */
function projectFile(content, assignment) {
  if (assignment.harness === "codex") {
    let next = editTomlKey(content, "model", assignment.model);
    next = editTomlKey(next, "model_reasoning_effort", assignment.reasoning_effort);
    next = editTomlKey(next, "model_provider", assignment.model_provider);
    return next;
  }
  const caps = HARNESS_CAPABILITIES[assignment.harness];
  let next = editFrontmatterKey(content, "model", assignment.model);
  if (caps.reasoning_effort && caps.reasoning_key) {
    next = editFrontmatterKey(next, caps.reasoning_key, assignment.reasoning_effort);
  }
  return next;
}

// ── assignments index ────────────────────────────────────────────────────────

function renderAssignments(policyText, assignments) {
  const capabilities = {};
  for (const [harness, caps] of Object.entries(HARNESS_CAPABILITIES)) {
    capabilities[harness] = { model: caps.model, reasoning_effort: caps.reasoning_effort };
  }
  const doc = {
    manifest_version: 1,
    generated_by: "scripts/model-policy.mjs",
    policy_sha256: sha256Hex(policyText),
    capabilities,
    assignments: assignments.map(({ file, ...rest }) => rest),
  };
  return JSON.stringify(doc, null, 2) + "\n";
}

function canonicalPolicyText(policy) {
  const tierRank = Object.fromEntries(SCOPE_TIERS.map((t, i) => [t, i]));
  const rules = [...policy.rules].sort((a, b) => {
    if (a.harness !== b.harness) return a.harness.localeCompare(b.harness);
    const ta = parseScope(a.scope);
    const tb = parseScope(b.scope);
    if (tierRank[ta.tier] !== tierRank[tb.tier]) return tierRank[ta.tier] - tierRank[tb.tier];
    return a.scope.localeCompare(b.scope);
  });
  const doc = {
    manifest_version: 1,
    description:
      "Per-harness model and reasoning-effort policy for agent harness variants. " +
      "Managed by scripts/model-policy.mjs (vfa-tui Model Policy view or CLI). " +
      "'auto' omits the field so the harness runtime default applies. " +
      "Precedence: agent > role > provider > all.",
    defaults: { model: "auto", reasoning_effort: "auto" },
    rules: rules.map((r) => {
      const out = { scope: r.scope, harness: r.harness };
      if (r.model !== undefined) out.model = r.model;
      if (r.reasoning_effort !== undefined) out.reasoning_effort = r.reasoning_effort;
      return out;
    }),
  };
  return JSON.stringify(doc, null, 2) + "\n";
}

// ── plan / apply ─────────────────────────────────────────────────────────────

/** Compute the full projection plan: file changes + new assignments text. */
function computePlan(policy, agents, roles, registry) {
  const { assignments, errors, warnings } = resolveAll(policy, agents, roles, registry);
  const changes = [];
  for (const a of assignments) {
    const abs = join(repoRoot, a.file);
    const current = readFileSync(abs, "utf8");
    const next = projectFile(current, a);
    if (next !== current) {
      changes.push({ file: a.file, assignment: a, next });
    }
  }
  return { assignments, errors, warnings, changes };
}

/** Print aggregated lifecycle warnings to stdout: identical warning texts
 * are grouped and printed once each, with the count of assignments they
 * affect. Warnings never change the exit code. */
function printWarnings(warnings) {
  const counts = new Map();
  for (const w of warnings) counts.set(w, (counts.get(w) || 0) + 1);
  for (const [text, count] of counts) {
    console.log(`WARNING: ${text} [affects ${count} assignment(s)]`);
  }
}

function describeChange(c) {
  const model = c.assignment.model ?? "auto";
  const reasoning = c.assignment.reasoning_effort ?? "auto";
  let detail;
  if (c.assignment.harness === "codex") {
    detail = `model=${model} provider=${c.assignment.model_provider ?? "default"} reasoning=${reasoning}`;
  } else if (c.assignment.harness === "claude-code") {
    detail = `model=${model} effort=${reasoning}`;
  } else {
    detail = `model=${model}`;
  }
  return `file update: ${c.file} (${detail})`;
}

function runApply({ dryRun }) {
  const registry = loadRegistry();
  const agents = loadAgents();
  const roles = loadRoles();
  const policy = loadJson(policyPath, "model policy");
  const structuralErrors = validatePolicy(policy, agents, roles, registry);
  if (structuralErrors.length > 0) {
    fail(1, "ERROR: model policy is invalid:", ...structuralErrors.map((e) => "  " + e));
  }
  const { assignments, errors, warnings, changes } = computePlan(policy, agents, roles, registry);
  if (errors.length > 0) {
    fail(1, "ERROR: model policy conflicts:", ...[...new Set(errors)].map((e) => "  " + e));
  }
  const policyText = canonicalPolicyText(policy);
  const assignmentsText = renderAssignments(policyText, assignments);
  const assignmentsStale = readFileOrNull(assignmentsPath) !== assignmentsText;

  for (const c of changes) console.log(describeChange(c));
  printWarnings(warnings);
  if (dryRun) {
    console.log(
      `dry-run: ${changes.length} file(s) would change; assignments index ${assignmentsStale ? "would be rewritten" : "already in sync"}`,
    );
    return;
  }
  for (const c of changes) writeFileSync(join(repoRoot, c.file), c.next);
  const currentPolicyText = readFileSync(policyPath, "utf8");
  if (currentPolicyText !== policyText) writeFileSync(policyPath, policyText);
  if (assignmentsStale) writeFileSync(assignmentsPath, assignmentsText);
  console.log(
    `OK: applied model policy (${changes.length} file(s) changed, ${assignments.length} assignments)`,
  );
  if (changes.length > 0 || assignmentsStale) {
    console.log("reminder: run `npm run asset-integrity:write` before committing");
  }
}

function runCheck() {
  const registry = loadRegistry();
  const agents = loadAgents();
  const roles = loadRoles();
  const policy = loadJson(policyPath, "model policy");
  const structuralErrors = validatePolicy(policy, agents, roles, registry);
  if (structuralErrors.length > 0) {
    fail(1, "ERROR: model policy is invalid:", ...structuralErrors.map((e) => "  " + e));
  }
  const { assignments, errors, warnings, changes } = computePlan(policy, agents, roles, registry);
  const problems = [...new Set(errors)];
  for (const c of changes) {
    problems.push(`drift: ${c.file} does not match the model policy`);
  }
  const policyText = canonicalPolicyText(policy);
  const assignmentsText = renderAssignments(policyText, assignments);
  const currentAssignments = readFileOrNull(assignmentsPath);
  if (currentAssignments === null) {
    problems.push("catalog/model-assignments.json is missing; run npm run model-policy:apply");
  } else if (currentAssignments !== assignmentsText) {
    problems.push("catalog/model-assignments.json is stale; run npm run model-policy:apply");
  }
  if (problems.length > 0) {
    fail(
      1,
      "ERROR: model policy check failed:",
      ...problems.map((p) => "  " + p),
      "fix: adjust catalog/model-policy.json or run npm run model-policy:apply",
    );
  }
  // Warnings are informational only — they never affect the exit code, even
  // when problems is otherwise empty (check stays green with warnings).
  printWarnings(warnings);
  console.log(
    `OK: model policy in sync (${policy.rules.length} rules, ${assignments.length} assignments)`,
  );
}

function runReport({ json }) {
  const registry = loadRegistry();
  const agents = loadAgents();
  const roles = loadRoles();
  const policy = loadJson(policyPath, "model policy");
  const structuralErrors = validatePolicy(policy, agents, roles, registry);
  if (structuralErrors.length > 0) {
    fail(1, "ERROR: model policy is invalid:", ...structuralErrors.map((e) => "  " + e));
  }
  const { assignments, errors, warnings } = resolveAll(policy, agents, roles, registry);
  if (errors.length > 0) {
    fail(1, "ERROR: model policy conflicts:", ...[...new Set(errors)].map((e) => "  " + e));
  }
  if (json) {
    console.log(renderAssignments(canonicalPolicyText(policy), assignments).trimEnd());
    return;
  }
  for (const a of assignments) {
    const model = a.model ?? "auto";
    const reasoning = a.reasoning_effort ?? "auto";
    console.log(`${a.agent_id} ${a.harness} model=${model} reasoning=${reasoning}`);
  }
  printWarnings(warnings);
}

// ── set ──────────────────────────────────────────────────────────────────────

function parseSetArgs(argv) {
  const args = { scopes: [], harness: null, model: undefined, reasoning: undefined, dryRun: false };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--dry-run") {
      args.dryRun = true;
    } else if (a === "--scope") {
      const v = argv[++i];
      if (v === undefined) fail(2, "ERROR: --scope requires a value");
      if (v === "all") {
        args.scopes.push("all");
      } else {
        const m = /^(provider|role|agent|agents)=(.+)$/.exec(v);
        if (!m) fail(2, `ERROR: invalid --scope "${v}"`);
        if (m[1] === "agents") {
          for (const id of m[2].split(",").filter(Boolean)) args.scopes.push(`agent:${id}`);
        } else {
          args.scopes.push(`${m[1]}:${m[2]}`);
        }
      }
    } else if (a === "--harness") {
      args.harness = argv[++i];
    } else if (a === "--model") {
      args.model = argv[++i];
    } else if (a === "--reasoning") {
      args.reasoning = argv[++i];
    } else {
      fail(2, `ERROR: unknown argument "${a}"`);
    }
  }
  if (args.scopes.length === 0) fail(2, "ERROR: --scope is required");
  if (!args.harness) fail(2, "ERROR: --harness is required");
  if (args.model === undefined && args.reasoning === undefined) {
    fail(2, "ERROR: provide --model and/or --reasoning");
  }
  return args;
}

function runSet(argv) {
  const args = parseSetArgs(argv);
  const registry = loadRegistry();
  const agents = loadAgents();
  const roles = loadRoles();
  const policy = loadJsonOrDefault(policyPath, "model policy", {
    manifest_version: 1,
    rules: [],
  });

  for (const scope of args.scopes) {
    const existing = policy.rules.find((r) => r.scope === scope && r.harness === args.harness);
    const rule = existing ?? { scope, harness: args.harness };
    if (args.model !== undefined) rule.model = args.model;
    if (args.reasoning !== undefined) rule.reasoning_effort = args.reasoning;
    if (!existing) policy.rules.push(rule);
    console.log(
      `policy rule: ${scope} ${args.harness}` +
        (args.model !== undefined ? ` model=${args.model}` : "") +
        (args.reasoning !== undefined ? ` reasoning=${args.reasoning}` : ""),
    );
  }

  const structuralErrors = validatePolicy(policy, agents, roles, registry);
  if (structuralErrors.length > 0) {
    fail(1, "ERROR: resulting policy would be invalid:", ...structuralErrors.map((e) => "  " + e));
  }
  const { assignments, errors, warnings, changes } = computePlan(policy, agents, roles, registry);
  if (errors.length > 0) {
    fail(1, "ERROR: resulting policy has conflicts:", ...[...new Set(errors)].map((e) => "  " + e));
  }
  for (const c of changes) console.log(describeChange(c));
  printWarnings(warnings);
  if (args.dryRun) {
    console.log(`dry-run: ${changes.length} file(s) would change; policy not written`);
    return;
  }
  const policyText = canonicalPolicyText(policy);
  writeFileSync(policyPath, policyText);
  for (const c of changes) writeFileSync(join(repoRoot, c.file), c.next);
  writeFileSync(assignmentsPath, renderAssignments(policyText, assignments));
  console.log(
    `OK: policy updated (${policy.rules.length} rules), ${changes.length} file(s) changed`,
  );
  console.log("reminder: run `npm run asset-integrity:write` before committing");
}

// ── import-current ───────────────────────────────────────────────────────────

function readCurrentValue(content, harness, key) {
  if (harness === "codex") {
    const lines = content.split("\n");
    let inTriple = false;
    const keyRe = new RegExp(`^${key}\\s*=\\s*"(.*)"\\s*$`);
    for (const line of lines) {
      const tripleCount = (line.match(/"""/g) || []).length;
      if (!inTriple) {
        if (/^\[/.test(line)) break;
        const m = keyRe.exec(line);
        if (m) return m[1];
      }
      if (tripleCount % 2 === 1) inTriple = !inTriple;
    }
    return "auto";
  }
  const lines = content.split("\n");
  if (lines[0] !== "---") return "auto";
  const keyRe = new RegExp(`^${key}:\\s*"?([^"]*)"?\\s*$`);
  for (let i = 1; i < lines.length; i++) {
    if (lines[i] === "---") break;
    const m = keyRe.exec(lines[i]);
    if (m) return m[1];
  }
  return "auto";
}

/** Pick the most common value; ties prefer "auto", then lexicographic. */
function majority(counter) {
  let best = null;
  for (const [value, count] of [...counter.entries()].sort((a, b) => {
    if (b[1] !== a[1]) return b[1] - a[1];
    if (a[0] === "auto") return -1;
    if (b[0] === "auto") return 1;
    return a[0].localeCompare(b[0]);
  })) {
    best = { value, count };
    break;
  }
  return best;
}

/** Derive a minimal rule set (all -> provider -> agent) reproducing the
 * observed per-agent values for one harness+field. */
function minimizeRules(observed, harness, field) {
  const rules = [];
  const global = new Map();
  for (const { value } of observed) global.set(value, (global.get(value) || 0) + 1);
  const globalBest = majority(global);
  const allValue = globalBest ? globalBest.value : "auto";
  if (allValue !== "auto") {
    rules.push({ scope: "all", harness, [field]: allValue });
  }
  const byProvider = new Map();
  for (const o of observed) {
    if (!byProvider.has(o.provider)) byProvider.set(o.provider, []);
    byProvider.get(o.provider).push(o);
  }
  for (const [provider, entries] of [...byProvider.entries()].sort()) {
    const counter = new Map();
    for (const { value } of entries) counter.set(value, (counter.get(value) || 0) + 1);
    const best = majority(counter);
    let effective = allValue;
    if (best.value !== allValue && best.count > entries.length / 2) {
      rules.push({ scope: `provider:${provider}`, harness, [field]: best.value });
      effective = best.value;
    }
    for (const o of entries.sort((a, b) => a.agent_id.localeCompare(b.agent_id))) {
      if (o.value !== effective) {
        rules.push({ scope: `agent:${o.agent_id}`, harness, [field]: o.value });
      }
    }
  }
  return rules;
}

function runImportCurrent({ force, dryRun }) {
  if (readFileOrNull(policyPath) !== null && !force) {
    fail(2, "ERROR: catalog/model-policy.json already exists; pass --force to regenerate");
  }
  const registry = loadRegistry();
  const agents = loadAgents();
  const roles = loadRoles();
  const rawRules = [];
  for (const [harness, caps] of Object.entries(HARNESS_CAPABILITIES)) {
    if (!caps.model && !caps.reasoning_effort) continue;
    const observedModel = [];
    const observedReasoning = [];
    for (const agent of agents) {
      const variantRel =
        agent.harness_variants?.[caps.variant] ?? `${agent.path}/harnesses/${caps.file}`;
      const abs = resolveInsideRepo(variantRel);
      if (abs === null) continue;
      const content = readFileOrNull(abs);
      if (content === null) continue;
      if (caps.model) {
        observedModel.push({
          agent_id: agent.id,
          provider: agent.provider,
          value: readCurrentValue(content, harness, "model"),
        });
      }
      if (caps.reasoning_effort) {
        observedReasoning.push({
          agent_id: agent.id,
          provider: agent.provider,
          value: readCurrentValue(content, harness, caps.reasoning_key),
        });
      }
    }
    if (caps.model) rawRules.push(...minimizeRules(observedModel, harness, "model"));
    if (caps.reasoning_effort) {
      rawRules.push(...minimizeRules(observedReasoning, harness, "reasoning_effort"));
    }
  }
  // Merge model + reasoning rules that share scope+harness.
  const merged = new Map();
  for (const r of rawRules) {
    const key = `${r.scope}::${r.harness}`;
    merged.set(key, { ...(merged.get(key) || {}), ...r });
  }
  const policy = { manifest_version: 1, rules: [...merged.values()] };
  const structuralErrors = validatePolicy(policy, agents, roles, registry);
  if (structuralErrors.length > 0) {
    fail(1, "ERROR: imported policy is invalid:", ...structuralErrors.map((e) => "  " + e));
  }
  const { assignments, errors, changes } = computePlan(policy, agents, roles, registry);
  if (errors.length > 0) {
    fail(1, "ERROR: imported policy has conflicts:", ...[...new Set(errors)].map((e) => "  " + e));
  }
  if (changes.length > 0) {
    fail(
      1,
      "ERROR: imported policy does not reproduce the current tree (bug):",
      ...changes.map((c) => "  " + describeChange(c)),
    );
  }
  if (dryRun) {
    console.log(`dry-run: would write policy with ${policy.rules.length} rules (0 file changes)`);
    return;
  }
  const policyText = canonicalPolicyText(policy);
  writeFileSync(policyPath, policyText);
  writeFileSync(assignmentsPath, renderAssignments(policyText, assignments));
  console.log(
    `OK: imported model policy (${policy.rules.length} rules, ${assignments.length} assignments, 0 file changes)`,
  );
}

// ── main ─────────────────────────────────────────────────────────────────────

const [command, ...rest] = process.argv.slice(2);
switch (command) {
  case "report":
    runReport({ json: rest.includes("--json") });
    break;
  case "check":
    runCheck();
    break;
  case "apply":
    runApply({ dryRun: rest.includes("--dry-run") });
    break;
  case "set":
    runSet(rest);
    break;
  case "import-current":
    runImportCurrent({ force: rest.includes("--force"), dryRun: rest.includes("--dry-run") });
    break;
  default:
    fail(
      2,
      "usage: node scripts/model-policy.mjs <report|check|apply|set|import-current> [options]",
    );
}
