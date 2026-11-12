---
metadata:
  author: "github: Raishin"
---

# Rust Safety Rule

When Codex implements, reviews, refactors or tests Rust, its Cargo configuration,
or an FFI/native boundary, apply these rules. Discover the selected checkout,
manifests, toolchain and fixture/evidence roots; never assume fixed host paths.

## RUST-1: Prove unsafe invariants

Prefer checked indexing, owned buffers and validated strings. Introduce unsafe
only for a necessary boundary or justified optimization. Document initialization,
bounds, alignment, aliasing, ownership, lifetimes and thread affinity where
relevant; a SAFETY comment must establish facts, not repeat an assumption.
Safe callers must not be able to violate the abstraction's memory-safety contract.

## RUST-2: Treat panic as an exit path

Before partially changing state, account for panicking clone/drop/comparison,
iterator and callback code. Preserve unwind-safe invariants with ordering or
guards. Never drop uninitialized values or unwind through an unsupported FFI ABI.
Do not use blanket catch_unwind/AssertUnwindSafe to justify continuing damaged
state; distinguish unwinding from aborts and ordinary Result errors.

## RUST-3: Validate caller-controlled behavior

Safe traits and typed results need not be logically correct or repeatable. Retain
the observation used for validation; check bounds, encoding, lengths and request
identity before using it. Do not check one callback result then use another
unchecked result. Revalidate mutable external state at the action boundary;
an earlier path/hash check does not eliminate filesystem races.

## RUST-4: Avoid process-global environment mutation

For non-Windows multithreaded code and parallel tests, avoid set_var/remove_var.
Use injected lookups or a child configured with Command::env/envs/env_clear.
A test-only mutex cannot synchronize unknown foreign environment readers.
Do not replace a running multithreaded host with CommandExt::exec as a shortcut
for spawning a helper; review its OS-specific contract if genuinely required.

## RUST-5: Own cleanup through completion

Keep resource/lock/process ownership until the resource is actually closed or
reaped. Root-PID exit, future cancellation and kill-on-drop do not prove descendant
cleanup. Require OS-specific evidence. Keep blocking joins off event-loop threads;
coordinate worker shutdown instead of calling process::exit from workers or
destructors. Do not rely on Drop for acknowledged persistence or async shutdown.

## RUST-6: Separate outcome from evidence

Require actual results; enqueue/flush or receiver loss is not success. Where
persistence exists, require its documented durable-write acknowledgment, such as
successful COMMIT under the required durability settings. For event journals,
validate projection/event consistency and reject conflicting retries. Preserve a
known execution outcome when evidence fails; never silently replay or repair an action.
Memory safety alone does not establish authorization, atomicity or correctness.

## RUST-7: Maintain dependencies deliberately

Verify reported issues against primary reports, affected APIs, exact toolchain,
dependency/lock identities and platform reachability. Pinning provides
reproducibility, not proof of patch coverage. Do not infer application
exploitability from a std soundness report or upgrade pins without authorization
and compatibility/regression evidence. Never install tools implicitly.

## RUST-8: Test failures and report limits

Define relevant negative tests before changes: panicking/inconsistent callbacks,
invalid identities/encoding, dropped receivers, bounded queues, transaction/commit
failures, owner death and surviving descendants. Run the selected project's format,
clippy and test gates; preserve known baseline failures explicitly. Use supported
Miri/sanitizer probes when appropriate, but do not equate them with complete
FFI/OS/native coverage. Report PASS/MISS/UNVERIFIED with exact commands, toolchain,
locks and exercised platforms. Compilation or one host's tests are not universal proof.

## Sources and reuse

Primary references: [unsafe contracts](https://doc.rust-lang.org/nomicon/safe-unsafe-meaning.html),
[exception safety](https://doc.rust-lang.org/nomicon/exception-safety.html),
[catch_unwind](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html),
[environment safety](https://doc.rust-lang.org/std/env/fn.set_var.html),
[process exit](https://doc.rust-lang.org/std/process/fn.exit.html),
[Miri limits](https://github.com/rust-lang/miri).
Motivation: [FORGE's article](https://forge-blog.github.io/posts/rust-std-memory-safety/)
by Yechan Bae (November 9, 2026). Independently authored guidance; no upstream
code or proof of concept copied. Reference an explicitly located copy from
another project's guidance; cataloging alone does not activate it globally.
Load [evals](rust-safety-rule.evals.md) only when maintaining/evaluating this rule.
