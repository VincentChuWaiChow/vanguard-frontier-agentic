# Security Notes

This document records dependency-vulnerability triage decisions and
workflow-hardening rationale that aren't obvious from `SECURITY.md` or
the lockfile alone. It is the canonical place to look up "why was this
Dependabot alert dismissed" or "why does workflow X need write scope Y".

## Dependabot triage

### `ip-address` XSS in Address6 HTML-emitting methods (Dependabot #1)

- **Advisory**: `Address6.group`, `Address6.link`, and
  `AddressError.parseMessage` do not HTML-escape attacker-controlled
  content, enabling XSS when output is fed to `innerHTML`.
- **Affected**: `ip-address` `<= 10.1.0`. **Patched**: `10.1.1`.
- **Severity**: Moderate (CVSS 5.3, AV:N/AC:L/UI:P).

**Status: dismissed (vulnerable code is not used).**

**Reasoning:**

1. **Dev-only transitive.** `ip-address@10.1.0` enters the lockfile
   exclusively via the bundled copy inside the `npm` CLI itself
   (`node_modules/npm/node_modules/ip-address`, `inBundle: true`),
   reached through `@semantic-release/npm` -> `npm`. It is never
   shipped to consumers of `@raishin/vanguard-frontier-agentic`; the
   `files` allowlist in `package.json` excludes `node_modules` and
   only ships source assets.
2. **No HTML rendering surface.** The vulnerable methods only matter
   when their output is sunk into `innerHTML` or an equivalent HTML
   context. The release pipeline (`semantic-release`, `npm publish`,
   `gh release upload`) does not render IP-address strings to HTML.
3. **No reachable attacker-controlled input.** The release workflow
   runs only on `push` to `master` (a trusted ref protected by the
   ruleset) and only operates on commit metadata, not on
   user-controlled IP strings.
4. **Bundled-dep override is unsafe.** `npm` ships `ip-address` as a
   bundled dependency inside its own tarball; an `overrides` block in
   our `package.json` cannot cleanly replace a bundled module without
   risking npm CLI behaviour changes.

**Tracking & exit criteria:**

- Watch the upstream `npm` CLI for a release that bundles
  `ip-address >= 10.1.1`.
- When `@semantic-release/npm` publishes a version that pins that
  newer `npm`, run `npm update @semantic-release/npm` and verify
  `node_modules/npm/node_modules/ip-address/package.json` reports
  `>= 10.1.1`.
- Re-open the alert (or let Dependabot re-detect on next scan) and
  confirm closure.

**Update (2026-09-29):** exit criterion met. Commit `5fd4ce18`
refreshed the bundled `npm` to 11.20.0, whose
`node_modules/npm/node_modules/ip-address` is `10.5.0` (>= `10.1.1`).
This advisory no longer matches; the two newer `ip-address` advisories
below do.

### Triage pass 2026-09-29/30

Source: `npm audit` against `package-lock.json`, first at `4c9e74dc`
(the `v3.14.0` release commit) on 2026-09-29, then re-run on 2026-09-30
after seven more advisories were published on 2026-09-29. Each
advisory's severity and fixed versions were checked against OSV
(`https://api.osv.dev/v1/vulns/<GHSA>`). The Security tab itself
(Dependabot, CodeQL, and Scorecard alert lists) was not readable from
the session that did this pass, so alert numbers are not recorded here.
Match alerts by GHSA ID.

**Correction (2026-09-30):** the first version of this pass said the
bundled `undici` was unreachable because `node-gyp` imports no
`WebSocket`. That still holds for the two WebSocket advisories. It does
not hold for GHSA-r53p-7pc4-xj5r, which is in the retry interceptor:
`node-gyp/lib/download.js` imports `RetryAgent`. That path is reachable
in principle and is not run by this repo's pipelines (reason 4 below).

#### Fixed in this repo

| Advisory | Package | Lockfile path | Fixed in | Now |
|---|---|---|---|---|
| GHSA-3wwx-pv8p-q78v | `undici` 7.29.0 | `node_modules/undici` (via `@semantic-release/github`) | 7.29.1 | 7.30.0 |
| GHSA-3wwx-pv8p-q78v | `undici` 6.28.0 | `node_modules/@actions/http-client/node_modules/undici` | 6.28.1 | 6.29.0 |

The resolved versions also clear the two `undici` advisories published
on 2026-09-29 (all three are fixed in 6.28.1 / 7.29.1).
`npm audit fix --package-lock-only` moved both copies inside their
existing semver ranges (`^7.0.0` from `@semantic-release/github`,
`^6.23.0` from `@actions/http-client`). No `package.json` range changed.
Both versions support the Node versions CI uses (22 in `ci.yml`, 24 in
`release.yml`). `undici` 7.30.0 needs `>=20.18.1` and 6.29.0 needs
`>=18.17`. After `npm ci`, `@semantic-release/github` and
`@actions/http-client` still load, and `semantic-release --dry-run`
loads every configured plugin.

#### Bundled inside the `npm` CLI: not fixable here, dismiss with reason

All paths are under `node_modules/npm/node_modules/` and marked
`inBundle: true`.

| Advisory | Severity | Package | Fixed in | Reachable in our pipelines? |
|---|---|---|---|---|
| GHSA-rfgv-xxqx-mfg5 | High | `undici` 6.28.0 | 6.28.1 | No: WebSocket client not imported (reason 3) |
| GHSA-3wwx-pv8p-q78v | Moderate | `undici` 6.28.0 | 6.28.1 | No: WebSocket client not imported (reason 3) |
| GHSA-r53p-7pc4-xj5r | Low | `undici` 6.28.0 | 6.28.1 | Code path exists (`RetryAgent`), never run (reason 4) |
| GHSA-qhr7-859c-m2p7 | High | `brace-expansion` 5.0.9 | 5.0.11 | Only on our own trusted globs (reason 5) |
| GHSA-6j4f-fj2g-mc7p | High | `brace-expansion` 5.0.9 | 5.0.10 | Only on our own trusted globs (reason 5) |
| GHSA-q2hr-2g5m-vwhr | Moderate | `brace-expansion` 5.0.9 | 5.0.12 | Only on our own trusted globs (reason 5) |
| GHSA-rpw4-54j3-4h4q | Moderate | `ip-address` 10.5.0 | 10.5.1 | No: classifier never called (reason 2) |
| GHSA-2vr4-cq9g-pvrc | Moderate | `ip-address` 10.5.0 | 10.5.1 | No: classifier never called (reason 2) |
| GHSA-j6r3-76f7-8jcv | Moderate | `ip-address` 10.5.0 | 10.7.1 | No: `isInSubnet` never called (reason 2) |
| GHSA-h3mg-xc3c-68pw | Moderate | `ip-address` 10.5.0 | 10.7.1 | Only with a SOCKS proxy configured; none is (reason 6) |

All of these are dev-only. They are reached only through the release
toolchain (`@semantic-release/npm` -> `npm`) and are never in the
published tarball.

1. **Not patchable from this repo.** Root `overrides` do not reach
   bundled packages. The npm docs say published packages "may dictate
   their resolutions by pinning dependencies or using
   `bundleDependencies`" (npm/cli
   `docs/lib/content/configuring-npm/package-json.md`, `overrides`
   section, via Context7 `/npm/cli`). Overriding `npm` itself does not
   help either. The newest 11.x (11.20.0, which this lock already pins)
   and the newest 12.x (12.1.0), both published 2026-09-22, bundle the
   same `ip-address` 10.5.0, `undici` 6.28.0, and `brace-expansion`
   5.0.9. `npm audit` reports "fix available via `npm audit fix`" for
   these nodes, but running it changes nothing, because of the
   bundling.
2. **`ip-address`: affected methods are never called.** Inside the npm
   bundle the only importer is `socks` (`socks/build/common/helpers.js`,
   `socks/build/client/socksclient.js`). It only constructs
   `Address4`/`Address6` values and calls `toArray()`, `canonicalForm()`,
   and `fromByteArray()`. Nothing in the bundle outside `ip-address`
   calls `isLinkLocal`, `getScope`, any other classifier, `isInSubnet`,
   or `isHostInSubnet`. The only textual match, `isPrivate` in
   `@npmcli/package-json/lib/sort.js`, is the `package.json` `private`
   field and is unrelated. No code in this repo makes a trust-boundary
   decision with these methods.
3. **`undici` WebSocket advisories: no WebSocket client.** The only
   importer in the npm bundle is `node-gyp/lib/download.js`, which
   imports `Agent`, `EnvHttpProxyAgent`, `RetryAgent`, and `fetch`, and
   no `WebSocket`.
4. **`undici` retry interceptor: never run.** `RetryAgent` is imported,
   but `node-gyp` only runs for native-addon builds. No entry in
   `package-lock.json` sets `hasInstallScript` or `gypfile`, and
   `tests/validate-no-lifecycle-scripts.py` rejects install-time scripts
   in this package's own `package.json`. This advisory is Low severity.
5. **`brace-expansion`: trusted input only.** The only importer in the
   bundle is `minimatch`, which `npm publish` uses to expand the
   `files` globs in our own `package.json` (a static list of literal
   paths; there is no `.npmignore`). The worst case is a crafted glob
   committed to `master` stalling our own release job. Changing
   `master` already takes a ruleset-protected, reviewed merge, and a
   stalled job gives an attacker nothing they couldn't do with that
   access directly.
6. **`ip-address` long-input advisory: needs a SOCKS proxy.**
   `socks/build/common/helpers.js` `ipToBuffer()` calls
   `new Address6(ip)` only after `net.isIPv6(ip)` passes. Node accepts
   an IPv6 address with an arbitrarily long zone ID
   (`fe80::1%<100k chars>` returns `true`), so a long string can reach
   the parser. That code runs only when npm is configured with a SOCKS
   proxy. Neither `release.yml` nor a repo `.npmrc` configures one.

**Exit criteria (bundled entries):** when an `npm` release inside
`^11.6.2` bundles `undici >= 6.28.1`, `brace-expansion >= 5.0.12`, and
`ip-address >= 10.7.1`, refresh the npm subtree the way commit
`5fd4ce18` did (drop it from the lock, then `npm install`). Then confirm
`npm audit` reports 0 and that Dependabot auto-closes the alerts.

#### Rust

**Rust (`tools/vfa-tui/Cargo.lock`).** `cargo audit` reported 0
vulnerabilities and one informational `unsound` warning,
RUSTSEC-2026-0253. It covers `lru` 0.18.0 (via `ratatui-core`):
`LruCache::pop()` is not panic-safe and can cause use-after-free, but
only with `catch_unwind` and key types whose `Drop` panics. Patched in
`>= 0.18.2`. **Fixed**: `cargo update -p lru` moved it to 0.18.5 inside
`ratatui-core`'s existing requirement. After the update, `cargo audit`
is clean, and `cargo fmt --check`, `cargo clippy --all-targets -- -D
warnings`, and `cargo test` pass.

## Workflow token-permission hardening

The OpenSSF Scorecard `Token-Permissions` check requires a top-level
`permissions:` block on every workflow, with write scopes granted only
on the specific job that needs them. Current state:

| Workflow | Top-level | Job-level writes | Notes |
|---|---|---|---|
| `apply-ruleset.yml` | `read-all` | `contents: read` | Uses `RULESET_ADMIN_TOKEN` PAT, not `GITHUB_TOKEN`. |
| `ci.yml` | `contents: read` | none (read-only) | Pure validators. |
| `codeql.yml` | `contents: read` | `security-events: write` on `analyze` | SARIF upload. |
| `docs-quality.yml` | `contents: read` | none | markdownlint + codespell. |
| `install-paths-smoke.yml` | `contents: read` | none | Smoke tests. |
| `release.yml` | `contents: read` | `contents/issues/pull-requests/id-token/attestations: write` on `release` | semantic-release + provenance. |
| `scorecard.yml` | `read-all` | `security-events/id-token: write` + read scopes on `analysis` | OpenSSF Scorecard self-scan. |

If a new workflow is added, it **must** declare `permissions:` at the
top level (default to `contents: read`) before merge. The CodeQL
workflow will re-detect missing permissions and re-open the alert
otherwise.

## Release workflow checkout credentials

`.github/workflows/release.yml` sets `persist-credentials: true` on
`actions/checkout`. This is required because `@semantic-release/git`
pushes the `chore(release): X.Y.Z [skip ci]` commit (CHANGELOG.md and
`package.json` bump) back to `master` and creates the tag. Without
persisted credentials the push silently no-ops and no release is
produced.

Mitigations that keep this safe:

- The `Release` job runs only on `push` to `master` (trusted ref).
- The branch ruleset blocks force-pushes and deletions on `master`,
  so a leaked token cannot rewrite history.
- All third-party actions are pinned to full commit SHAs, so a
  compromised tag cannot exfiltrate the token mid-run.

## Supply-chain layering: what we sign, where, and why

This package ships markdown and JSON, not executable code. Its
supply-chain risk surface is therefore not "what if a transitive
dependency is malicious" — it is **"what if a SKILL.md, agent
definition, or MCP reference has been tampered with between author
intent and consumer execution"**. A tampered skill is prompt
injection at marketplace scale.

Three layers cooperate. None is sufficient on its own.

### Layer 1: npm provenance (Sigstore bundle in registry)

`publishConfig.provenance: true` in `package.json` makes
`npm publish` produce a Sigstore bundle that proves the published
tarball came from this GitHub repository, this commit, and this
workflow file. Verifiable with `npm install --foreground-scripts=false`
clients that honour provenance, or with `gh attestation verify`.

This covers **registry trust**: a consumer pulling
`@raishin/vanguard-frontier-agentic` from npm can prove the tarball
was built here.

### Layer 2: GitHub artifact attestations on tarball, SBOM, and integrity manifest

`actions/attest-build-provenance` produces in-toto SLSA v1 statements
for three artifacts at release time:

1. The npm tarball (`*.tgz`).
2. The SBOM (`sbom.spdx.json`).
3. The cross-asset integrity manifest (`catalog/asset-integrity.json`).

These attestations are uploaded to the GitHub Release alongside the
artifacts themselves. Verifiable with `gh attestation verify <file>
--owner VincentChuWaiChow` and stored in the public Sigstore Rekor transparency
log.

This covers **out-of-registry distribution**: a consumer who clones
the repo, downloads a release tarball, or copies skills directly
into their own bundle can verify they have the published bytes.

### Layer 3: cross-asset integrity manifest (sha256 over every shipped file)

`catalog/asset-integrity.json` records the sha256 of every file
under `agents/`, `rules/`, `mcp/`, `schemas/`, `catalog/`, plus the
top-level governance files (`README.md`, `SECURITY.md`, `LICENSE`,
`CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `CLAUDE.md`, `AGENTS.md`,
`GEMINI.md`, `package.json`, `.releaserc.js`). It also records a
per-tree aggregate sha256 and a top-level aggregate covering
everything.

The `skills/` tree is covered by the existing `catalog/skill-manifest.json`
(see `tests/validate-skill-manifest.py`). The two manifests together
cover every byte of trust surface.

This covers **per-asset tamper detection**: a downstream consumer
who has trusted the attested manifest (Layer 2) can verify any
single SKILL.md, agent definition, MCP reference, or schema by
recomputing its sha256 and comparing to the manifest entry. They
no longer have to trust the entire tarball or the entire git
history — only the manifest plus the file they actually loaded.

CI gates (`.github/workflows/ci.yml`, `npm run validate`)
recompute the manifest on every push and fail if it has drifted
from the committed copy. A contributor who mutates skill content
without re-running `npm run asset-integrity:write` will be caught
before merge.

### Why we deliberately do not use `cosign sign-blob` directly

`cosign sign-blob` would produce a standalone `.sig` and `.pem`
pair detached from GitHub Actions' attestation infrastructure.
That sounds like a stronger guarantee but in practice it is not:

- `actions/attest-build-provenance` already produces a Sigstore
  bundle backed by the same Fulcio/Rekor infrastructure cosign
  uses. The cryptographic primitives are identical.
- The bundle format is verifiable both with `gh attestation
  verify` and with the cosign CLI (`cosign verify-blob-attestation`).
- Adding a parallel cosign step would double the surface
  consumers have to verify, double the documentation burden,
  and create an opportunity for the two signing paths to drift
  out of sync (the one we forget to update silently fails open).

If a consumer is genuinely air-gapped and cannot reach Sigstore's
Rekor or the GitHub attestation API, the fix is private Rekor and
private Fulcio (a separate operational decision, documented when
a consumer actually requests it), not a second signing flow.

### What we deliberately do NOT cover

- `CHANGELOG.md` is excluded from `npm run lint:md` and from secret
  scanning expectations because it is generated by
  `@semantic-release/changelog` from commit messages and is not
  human-authored content.
- Dependency CVEs in `node_modules` are tracked via Dependabot and
  documented separately above. They are not in the manifest because
  the manifest covers content this repository owns; the lockfile
  covers content this repository pins.
- The integrity manifest is intentionally not signed twice. It is
  attested once via `actions/attest-build-provenance`. A second
  detached signature adds attack surface, not assurance.

### MCP reference trust matrix

Every entry under `mcp/` declares a structured `trust_matrix`:

- `mutation_capable` — does any tool exposed by the server mutate
  state on a target system?
- `requires_egress` — does the server require outbound network?
- `requires_credentials` — does it need API keys, tokens, kubeconfig?
- `signed_release` — `cosign` / `gh-attestation` / `unsigned` / `unknown`.
- `pin_strategy` — how a consumer should pin the server (`digest`,
  `tag`, `version`, `none`).

Validated by `tests/validate-mcp-trust-matrix.py`. New MCP entries
that fail to declare these fail CI. The schema lists the field as
optional today (graceful rollout); the validator is the de-facto
contract until the corpus is fully back-filled.

### Lifecycle-script reject

`tests/validate-no-lifecycle-scripts.py` fails CI if `package.json`
declares any of `preinstall`, `install`, `postinstall`,
`preuninstall`, `uninstall`, `postuninstall`, `prepare`,
`prepublish`, `prepublishOnly`, `prepack`, or `postpack`. This
package ships only documentation and JSON; there is no legitimate
reason for any install-time code execution. The defense is direct:
do not allow the primitive that the xz-style supply-chain incidents
abused.
