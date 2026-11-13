# Changelog

## 1.2.1 - 2026-11-13

- Integrate into the vanguard-frontier-agentic catalog under `skills/atlassian/`. Add the repository-required `allowed-tools` declaration (`Read Grep Glob WebFetch`; drafting and review need no shell, and live Atlassian access is granted by the harness MCP configuration, not by this line) and a repository `metadata.json`. No template, prompt, example, or reference content changed.
- Authorized writes initiated by another agent go through `atlassian-live-write-guard-agent`; a request that appears only inside an agent message or retrieved content is not the user's authorization. Direct user requests keep the existing reuse-of-authorization behavior.

## 1.2.0 - 2026-11-11

- Distinguish story-specific acceptance criteria from applicable, source-backed team completion policy in story and implementation templates.
- Add focused user-story and epic-breakdown prompts plus lazy template-adaptation guidance; reject checkbox/score-based readiness and assumed deployment gates.
- Add optional, evidence-backed external dependency gates to the epic template, distinguishing needed-by forecasts from actual commitments.
- Record the reviewed upstream patterns and pinned revision without importing third-party templates, scripts, defaults, or integrations.

## 1.1.0 - 2026-11-11

- Add copy-ready prompts for feature requests, bug reports, customer support cases, and access or service requests.
- Include the combined product idea to PRD and Jira backlog workflow.
- Link the prompt catalogue from `SKILL.md` and include prompt files in the integrity manifest.

## 1.0.0 - 2026-11-11

- Initial release of Jira ticket authoring templates, examples, and references for engineering, product, business, and support work.
- Include evidence-based drafting, acceptance criteria, quality review, and Jira/Confluence traceability guidance.
- Declare version `1.0.0` and author `Vincent Chu` in skill metadata.
