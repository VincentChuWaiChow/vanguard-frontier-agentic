# Native Windows prerequisite handoff

## Verified current state

Native Windows x64 is reachable through WSL interop with explicit host escalation.
The sandboxed interop attempt fails at its socket boundary. Native OS reports
10.0.26300.0. Winget exists; cargo/rustc are absent from PATH, default user cargo/
rustup binaries are absent, vswhere is absent, and known MSVC BuildTools/Windows
SDK library paths are absent. This is standard-path discovery, not an exhaustive
disk inventory. No Windows build or bootstrap has run.

## Setup admission still required

Choose an appropriately licensed Visual Studio edition/C++ toolchain and Windows
SDK. License eligibility is unknown; do not silently choose Community or BuildTools
or accept an agreement. Confirm scope/admin/install behavior before automated
installation. No installation/download/configuration occurred during discovery.

Official rustup MSVC guidance names Desktop Development with C++ and a Windows SDK;
its minimal component example is MSVC x64/x86 build tools plus Windows 11 SDK.
The current GPUI consumer is x64, so no ARM64 tool installation is inferred.

Once an approved Windows Rustup installation exists, install/use the project's
exact numbered toolchain, not an unverified Windows stable alias:

```powershell
rustup toolchain install 1.99.1 --profile minimal --component rustfmt --component clippy
rustc +1.99.1 -Vv
```

Do not substitute Linux compiler evidence, cross-compilation, WSLg or a Windows
package manager success for actual Windows native bootstrap. Existing source,
source hashes, locks, same-version dev-only test support and real MSVC target
must remain intact. Select a controlled native checkout/work directory and
caller-owned evidence directory before running existing check/UI/native targets.
Do not invoke external CI or copy unreviewed secrets/account configuration.

## Sources

Fetched with Context7 after resolving /rust-lang/rustup:
[Windows MSVC prerequisites](https://github.com/rust-lang/rustup/blob/main/doc/user-guide/src/installation/windows-msvc.md)
and [Rustup installer/toolchain flags](https://github.com/rust-lang/rustup/blob/main/src/cli/setup_mode.rs).
These establish setup mechanics, not the user's licensing eligibility.

## Dependency admission

Current task-list audit finds only unchecked task 1.4 has every numbered dependency
complete. Task 1.6's actual Linux consumer tests pass, but its full dependency 1.4
remains open; later full implementation cannot start by relabeling partial evidence.
The G1 requirement retains real macOS ARM64 and Windows x64 native evidence.
