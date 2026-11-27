# Packet 05: Publication follow-up platform research
## Objective
Answer the new user question: can Docker emulate macOS sufficiently to satisfy the GPUI macOS ARM64 native bootstrap/Metal gate?
## Context
New follow-up explicitly authorizes Git commit/push and asks about Docker; no authorization to deploy, change native gates, install emulators, rent cloud hosts or invoke remote CI. Prior installation run used three read-only agents. This separate follow-up admits one bounded read-only research agent only; parent owns Git and diff integration. No new competing run root.
## Sources
Official Docker documentation, Apple virtualization documentation and, if useful, GitHub-hosted macOS runner documentation. Use Context7 resolve then query for Docker, with primary official web documentation fallback.
## Ownership
Read-only external-documentation research only; no files/tests/builds/installs/Git operations.
## Do
Separate Linux containers, macOS virtual machines, architecture and GPUI Metal/native proof. State renderer limitations/uncertainty. Return 2-4 concise facts with primary URLs and practical options; do not estimate pricing without verification.
## Do not
Spawn agents, execute Docker, read credentials, install anything, change files, recommend licensing bypasses or claim unexecuted CI/VM satisfies acceptance.
## Expected output
Concise answer, citations, uncertainty and recommended next step.
## Verification
Documentation evidence only, not runtime evidence.
## Handoff format
Facts; sources; caveats; recommendation.
