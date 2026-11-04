---
name: vue-router-navigation-security-review
description: "Statically review Vue Router navigation guards, redirect flows, and template bindings for client-side guards used as the sole authorization boundary, open redirects via route.query.redirect/returnUrl, javascript:/data: scheme injection through dynamic :to/:href bindings, route params/query reaching v-html/innerHTML (reflected XSS), guard-induced redirect loops, and catch-all/history-mode misconfiguration, grounded in Vue Router's own documentation."
allowed-tools: Read Grep Glob
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
  updated: "2026-07-03"
  category: security
---

# Vue Router Navigation Security Review

## Purpose

Review Vue Router navigation guards, redirect flows, dynamic link bindings, and route
configuration for the security-critical defect classes specific to *client-side routing as a
security surface* — not general route-tree architecture, not component design, and not the
state-store or SSR-hydration concerns owned by sibling skills. This skill exists to keep the
review anchored to five documented defect classes: (a) a `beforeEach`/`beforeEnter` guard used
as the sole authorization boundary with no server-side enforcement behind it, (b) open redirect
via `route.query.redirect`/`returnUrl` passed unvalidated into `router.push()`/
`window.location`, (c) `javascript:`/`data:` scheme injection via a dynamic `:to`/`:href`
binding, (d) `route.params`/`route.query` interpolated into `v-html`/`innerHTML` (reflected XSS
through the router), and (e) `next(unvalidatedPath)`/guard-returned redirects creating loops or
bypasses, overly-broad catch-all routes, and `history`-mode server misconfiguration. It does not
re-litigate general route-tree/code-splitting architecture, Pinia/Vuex store security, or SSR
cross-request pollution in every response.

## When to use

Use this skill when the user asks to:

- review a `router.beforeEach`/`beforeEnter` guard (or a Nuxt `definePageMeta`/route-middleware
  equivalent) that gates access to a route,
- audit a login/logout flow's post-authentication redirect handling,
- assess whether a `<router-link :to="...">` or dynamic `:href` binding is safe from scheme
  injection,
- investigate whether `route.params`/`route.query` reaching a `v-html` or `.innerHTML` sink is
  exploitable,
- review a route table for redirect-loop risk, catch-all overreach, or `history`-mode
  server-fallback exposure,
- perform a pre-launch security review of a Vue Router-based application's navigation layer.

Do not use this skill for:

- general route-tree structure, loader/code-splitting, or focus-management review with no
  security angle — use `routing-navigation-review` instead (and note it targets React
  Router/Next.js, not Vue Router),
- Pinia/Vuex store security (persisted sensitive data, SSR store-singleton pollution, untrusted
  hydration payloads, client-held role flags as an authorization source) — use
  `vue-state-store-security-review` instead,
- SSR entry-point cross-request state pollution or general `v-html`/dynamic-URL injection
  unrelated to router data — use `vue-ssr-security-review` instead; this skill's injection scope
  is specifically `route.params`/`route.query` reaching a sink, not every `v-html` in the app,
- broad token-storage, cookie-flag, CSRF, or OAuth/OIDC flow review with no router-specific
  mechanism in play — use `frontend-auth-session-security-review` instead; this skill covers
  only the router-mediated open-redirect mechanism (`route.query.redirect`/`returnUrl`), not
  session/token architecture generally,
- confirming a bypass has already been exploited in production (concurrent-request reproduction,
  live penetration testing, a captured attack) — static analysis proves the structural risk, not
  that it has already been exploited.

## Context7 Documentation Protocol

- Resolve the Vue Router library ID with `resolve-library-id` (matched result: `/vuejs/router`,
  Vue Router's own source-and-docs repository) before citing any guard-mechanism, redirect-API,
  or dynamic-matching claim.
- Use `query-docs` against `/vuejs/router` to corroborate: navigation-guard return semantics
  (`beforeEach`/`beforeEnter` returning `false`/a location/`undefined`, and the legacy `next`
  argument's call-exactly-once contract), the documented `to.name !== 'Login'` self-exclusion
  pattern, redirect function shapes (`redirect: to => ({ path, query })`), `route.params`/
  `route.query` exposure with no built-in escaping, `router.push`'s acceptance of bare string
  paths, `createWebHistory()`'s server-fallback requirement, and the `/:pathMatch(.*)*`
  catch-all pattern. These are all `repo evidence` claims — cite the specific guide file
  (navigation-guards, redirect-and-alias, dynamic-matching, history-mode) alongside the claim.
- The conclusion that a client-side guard must not be the *sole* authorization boundary is not
  itself a Vue-Router API fact — Vue Router's docs describe guards strictly as navigation
  control flow (cancel/redirect/proceed), never as an access-control mechanism. Label this
  specific conclusion `inference`, grounded in general server-side-enforcement security practice
  (`documentation-based` via OWASP), layered on the `repo evidence` fact that guards are
  browser-executed callbacks.
- The claim that `v-html` bypasses Vue's default template auto-escaping is general Vue
  template-compiler behavior, not a Vue-Router API — ground it in the Vue security guide
  (`documentation-based`, listed in this skill's `official_docs`), not in `/vuejs/router`.
- Read `package.json` first to confirm the Vue Router major version in use — guard signatures
  (return-value guards vs. the legacy `next` third argument), redirect object shape, and
  catch-all syntax (`/:pathMatch(.*)*` in v4 vs. legacy `*` in v3) differ across majors; do not
  apply v4 API names to a v3 codebase or vice versa.
- If Context7 is unavailable, fall back to the `official_docs` URLs in this skill's
  `metadata.json` and label the claim `documentation-based, unverified against current release`.

## Lean operating rules

- Every finding in this skill's scope defaults to HIGH severity: broken client-side-only access
  control, open redirect, scheme injection, and reflected XSS are all directly exploitable
  without further chaining. Do not downgrade to MEDIUM because "it hasn't been reported
  exploited yet" — the risk is structural.
- Trace every finding to a concrete file:line and a concrete data-flow path (guard → endpoint
  gap, or origin → sink hops for injection/redirect findings). A finding that says "this guard
  might not be enough" or "this redirect might be exploitable" without the specific trace is a
  guess, not a finding.
- Never flag "a `beforeEach`/`beforeEnter` guard exists" by itself. The finding requires the
  *absence* of confirmed server-side enforcement on the endpoint the protected route depends on
  — look for evidence (a 401/403 check, a BFF layer, an API contract note) before concluding
  it's missing, and state explicitly what you found or didn't find.
- Never accept a type check or substring blocklist as clearing an open-redirect or
  scheme-injection finding. The only acceptable control is an allowlist: same-origin/
  relative-path validation for redirects, and an explicit protocol allowlist
  (`http:`/`https:`/`mailto:`) for dynamic link bindings.
- Do not approve a `v-html`/`.innerHTML` sink fed by `route.params`/`route.query` unless a named
  sanitizer call is visibly present on that exact traced path — a sanitizer existing elsewhere
  in the codebase does not clear this bar.
- Check every unconditional redirect-on-guard for the framework's own documented self-exclusion
  pattern (`to.name !== 'Login'` or equivalent). Its absence is a HIGH redirect-loop finding, not
  a style nit — Vue Router's docs treat it as required.
- Read the full `routes` array in registration order before clearing a catch-all route; a
  broad/catch-all pattern registered before a route that should require auth can shadow it.
- Never execute, build, or run application code, and never send live requests, as part of this
  review; this is a static-review skill (Read/Grep/Glob only).
- Load only the reference needed for the concern in scope.

## References

Load these only when needed:

- [Review workflow and findings contract](references/workflow-and-output.md) — use for the
  step-by-step review procedure, the full decision tree across all five defect classes, and the
  required output shape.
- [Client-side guards as an authorization boundary, and navigation control-flow risks](references/client-guards-and-control-flow.md)
  — load when reviewing a `beforeEach`/`beforeEnter` guard, a guard-returned redirect, a
  catch-all route, or `history`-mode configuration.
- [Open redirect, scheme injection, and reflected XSS through the router](references/open-redirect-and-injection.md)
  — load when reviewing a post-login redirect flow, a dynamic `:to`/`:href` binding, or a
  `v-html`/`innerHTML` sink fed by `route.params`/`route.query`.

## Response minimum

Return, at minimum:

- the guard(s), redirect flow(s), dynamic link binding(s), route table, and/or `v-html`/
  `innerHTML` sink(s) in scope,
- ranked findings with file:line evidence, defect category
  (`client-guard-as-sole-authz` / `open-redirect` / `scheme-injection` / `reflected-xss` /
  `redirect-loop` / `catch-all-misconfig` / `history-mode-server-misconfig`), the concrete
  data-flow trace (the guard-to-endpoint gap, or the full origin-to-sink path naming every hop),
  and a fix sketch matching Vue Router's documented pattern,
- for every guard finding, an explicit statement of whether server-side enforcement evidence was
  found and where — never infer it exists without evidence on the traced path,
- evidence level per finding (`repo evidence`, `documentation-based`, or `inference`), with
  structural-risk findings explicitly labeled as structural risk, not confirmed-exploited,
- verdict (approve / approve-with-notes / block),
- open questions or scope the review could not cover (e.g., "confirming the API layer actually
  rejects unauthorized requests requires a live request or reading server-side code outside this
  diff's scope").
