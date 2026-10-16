# Vanguard Mods

<div align="center">

  <img src="../assets/logos/vanguard-frontier-agentic-logo.png" alt="Vanguard Frontier Agentic" width="140" />

  <p><strong>Less “oops.” More visibility.</strong></p>
  <p>Tool-call guardrails and observers for Claude Code, Kiro and Codex.</p>

  <p>
    <a href="#pick-a-mod">Pick a mod</a> &nbsp;·&nbsp;
    <a href="#get-started">Get started</a> &nbsp;·&nbsp;
    <a href="#know-the-limits">Know the limits</a> &nbsp;·&nbsp;
    <a href="#under-the-hood">Under the hood</a>
  </p>

</div>

---

Skills guide the agent. **Mods inspect its tool calls.** Ask before risky actions,
redact supported output, or record what happened. Pick what you need—not everything at once.

<a id="pick-a-mod"></a>

## 🧰 Pick your guardrails

| Mod | The job | Reach |
| --- | --- | --- |
| [Secret Redactor](secret-redactor/README.md) | Spot credential-shaped input; redact supported output. | L0 |
| [Blast Radius Guard](blast-radius-guard/README.md) | Ask or deny on destructive commands and system-path writes. | L0 |
| [Live Guard Gate](live-guard-gate/README.md) | Flag cloud mutations and name guard agents to consult. | L0 |
| [Context Burn](context-burn/README.md) | Count tool calls; warn on reported context use where supported. | L1 |
| [Audit Trail](audit-trail/README.md) | Append call hashes and per-session sequence numbers. | L1 |

**Reach is a label, not a sandbox:** L0 = no declared writes/spawns/network;
L1 = file writes; L2 = subprocesses; L3 = network (warned, not blocked).
Reads are allowed at L0; starting the Node adapter is not a mod-spawned subprocess.

**Support differs:** Kiro/Codex deny instead of asking and cannot redact output.
Claude Code Audit Trail needs classic. Classic Post hooks miss failed calls.

<details>
<summary><strong>🔎 Same mod, different harness? Check the fine print.</strong></summary>

| Mod | Claude Code native | Claude Code classic | Kiro / Codex |
| --- | --- | --- | --- |
| Secret Redactor | Ask on input; redact or withhold output | Ask on input; replace successful output only | Deny matching input; no output redaction |
| Blast Radius Guard | Ask in confirm mode; deny in block mode | Same | Deny in either mode |
| Live Guard Gate | Ask and name guard agents | Same | Deny and name guard agents |
| Context Burn | Session context usage and threshold toasts | Tool-call counter only | Tool-call counter only |
| Audit Trail | Unsupported—use classic | Append after successful calls | Append; Kiro payload interpretation is degraded |

Guards use `PreToolUse`; supported redaction and observers use `PostToolUse`;
Context Burn also uses `SessionStart`.
Native equivalents are `tool.check`, `tool.call` and `session.start`.

Kiro's stdin fields are undocumented: guards scan the whole payload if input is
unrecognized; Audit Trail hashes that payload as arguments and `null` as the result.
Set `VANGUARD_AUDIT_TRAIL_SESSION_ID` to avoid its shared `unsessioned` fallback.
See generated `<harness>/UNSUPPORTED.md` for remaining limits.

</details>

---

<a id="get-started"></a>

## 🚀 Get started

Use a local checkout. Command adapters need **Node 22+**; native modules use the
harness runtime. No service needed.

```bash
source=/absolute/path/to/vanguard-frontier-agentic
target=/absolute/path/to/your/project
mod=blast-radius-guard
```

> [!IMPORTANT]
> **Native or classic. Not both for the same mod.** They duplicate checks and can
> double-count Context Burn. Audit Trail has no native adapter—use classic.

<details>
<summary><strong>🤖 Claude Code native</strong> — load a plugin root</summary>

```bash
cd "$target"
claude --plugin-dir "$source/mods/$mod"
```

Not for `audit-trail`. Native API: early access. Check the verified build in the
[implementation notes](#under-the-hood) before upgrading.

</details>

<details>
<summary><strong>🤖 Claude Code classic</strong> — copy, then merge hooks</summary>

```bash
mkdir -p "$target/.claude/vanguard-mods/$mod"
cp -R "$source/mods/$mod/bin" "$source/mods/$mod/lib" "$target/.claude/vanguard-mods/$mod/"
```

Append `mods/<id>/claude-code/settings.hooks.json` entries under `hooks` in the
target's `.claude/settings.json`. **Keep existing hooks/settings.** Generated command:

```text
node "$CLAUDE_PROJECT_DIR/.claude/vanguard-mods/<id>/bin/hook.mjs" --harness claude-code --event <event>
```

`CLAUDE_PROJECT_DIR` stays at the initial project root, even across worktree changes.
Observers resolve relative writes from the installed adapter location.

</details>

<details>
<summary><strong>🔮 Kiro</strong> — copy the adapters and hook JSON</summary>

```bash
mkdir -p "$target/.kiro/hooks" "$target/.kiro/vanguard-mods/$mod"
cp -R "$source/mods/$mod/bin" "$source/mods/$mod/lib" "$target/.kiro/vanguard-mods/$mod/"
cp "$source/mods/$mod/kiro/"*-hook.json "$target/.kiro/hooks/"
```

Copy only `*-hook.json`, not `UNSUPPORTED.md`. Replace the relative script path
inside each hook with its **absolute installed path**: hook cwd is undocumented.
Commands use `--harness kiro --event <event>`; confirmation requests deny with exit 2.

</details>

<details>
<summary><strong>⚡ Codex</strong> — copy, merge, then review trust</summary>

```bash
mkdir -p "$target/.codex/vanguard-mods/$mod"
cp -R "$source/mods/$mod/bin" "$source/mods/$mod/lib" "$target/.codex/vanguard-mods/$mod/"
```

Append event entries from `mods/<id>/codex/hooks.json` to the target's
`.codex/hooks.json`, preserving existing hooks. Replace relative script paths with
**absolute installed paths**; hook cwd is undocumented. Complete Codex's hook trust review.

Commands use `--harness codex --event <event>`; asks deny with exit 2.
Redactor Post is unsupported; the narrower MCP output field is not substituted.

</details>

**Try it before trusting it:** run ordinary and harmless pattern-matching calls in
the harness. Inspect the decision, output or observer file—not just exit codes.
Repeat after path, permission, Node or harness changes.

---

<a id="know-the-limits"></a>

## ⚠️ Guardrails, not guarantees

- **A hook must run to help.** Guards deny handler/input errors; missing Node,
  moved scripts, startup crashes and timeouts can leave calls unguarded.
  CLI errors may not block; Kiro IDE handling differs.
- **Regex is not complete DLP.** Pre checks never rewrite arguments; approved input
  still reaches the tool. Post redaction does not scrub other logs or network traffic.
  Redactor errors attempt withholding; classic shape mismatches may be ignored.
  [Withholding has limits, too](secret-redactor/README.md#limitations).
- **A review prompt is not authorization.** Live Guard Gate names agents—it never
  spawns them or verifies production accounts. Guards scan strings, not shell semantics;
  relative paths stay unresolved. Bypass modes can auto-approve an ask.
- **Observers are not compliance proof.** Node counters track calls, not tokens or
  spend; only native Context Burn uses reported context data. Unlocked updates can
  lose increments. Audit logs are mutable, unsigned and not tamper-evident.
  Observer failures do not block tools.

<details>
<summary><strong>🗂️ State files and custom patterns</strong></summary>

- **Context Burn:** `.<harness>/context-burn-status/<session-hash>.json`
  (`.claude` for Claude Code). Filenames use the full lowercase SHA-256 of the UTF-8
  session ID, never raw IDs. Missing/empty Node IDs share `unsessioned` state.
  `VANGUARD_CONTEXT_BURN_STATUS_PATH` is an exact override; sharing it opts out of
  isolation. Old single-file state is not migrated automatically.
- **Audit Trail:** `.audit/<UTC-date>/<session>.jsonl`; an earlier file is reused
  across dates. `VANGUARD_AUDIT_TRAIL_LOG_DIR` overrides the directory.
- **Footprints disclose defaults, not containment.** Overrides can write outside
  declared globs; overlap checks do not constrain operator overrides.
- **Patterns append, never replace.** `VANGUARD_SECRET_REDACTOR_PATTERNS` and
  `VANGUARD_BLAST_RADIUS_GUARD_PATTERNS` accept extra pattern files. Use absolute paths.
  Unreadable/invalid configured files deny guards or withhold redactor output.
  Each mod README documents schemas and environment names.

</details>

<a id="under-the-hood"></a>

## 🔧 Under the hood

`mod.toml` declares support and footprint; `lib/core.mjs` holds shared behavior.
Edit those sources, `patterns.json`, entries and native modules—not generated adapters.
The generator produces manifests, shared runtime copies, pattern modules and `catalog/mods.json`.

<details>
<summary><strong>🛠️ Contributing? Run the gates.</strong></summary>

```bash
npm run mods:write
npm run validate:mods
npm run test:fuzz
npm run asset-integrity:write  # last generator, on its own
npm run validate
```

The Python gate needs **Python 3.11+**. For each supported native mod, also run
`claude plugin validate mods/<id>` and `claude plugin test mods/<id>` locally;
CI has no native CLI. Spelling and Markdown lint are separate gates.
Static checks are not proof that an installed hook runs.

</details>

[Implementation plan](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/blob/dc8b29c823fb89dc4246413f38a7b82fdd34aa18/.claude/plan/cross-harness-mods.md)
· [Spec deviations](https://github.com/VincentChuWaiChow/vanguard-frontier-agentic/blob/dc8b29c823fb89dc4246413f38a7b82fdd34aa18/.kiro/specs/cross-harness-mods/deviations.md)
