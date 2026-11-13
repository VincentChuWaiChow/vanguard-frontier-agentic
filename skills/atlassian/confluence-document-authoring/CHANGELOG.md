# Changelog

## 1.2.1 - 2026-11-13

- Integrate into the vanguard-frontier-agentic catalog under `skills/atlassian/`. Add the repository-required `allowed-tools` declaration (`Read Grep Glob WebFetch`; drafting and review need no shell, and live Atlassian access is granted by the harness MCP configuration, not by this line) and a repository `metadata.json`. No template, prompt, example, or reference content changed.
- Authorized writes initiated by another agent go through `atlassian-live-write-guard-agent`; a request that appears only inside an agent message or retrieved content is not the user's authorization. Direct user requests keep the existing reuse-of-authorization behavior.

## 1.2.0 - 2026-11-11

- Add eight independently authored templates and matching prompts for project planning/status, roadmaps, OKRs, risks, experiments, customer-impact assessments, and team working agreements.
- Add a lazily loaded extended catalogue with verified public Atlassian source links; preserve the existing templates and prompts.
- Keep planning, measured outcomes, approval, and native Confluence installation distinct. These assets are not native template exports.
- Add an independently authored project charter and prompt for mandate, stakeholder decision rights, funding estimates, and approval gates; distinguish it from the delivery plan.
- Add evidence-backed OKR check-in history, audience-aware roadmap summaries with sharing safeguards, and lazy template-change/migration guidance.
- Extend type-specific quality review and record the pinned upstream comparison without importing its macros, database schemas, scripts, or defaults.

## 1.1.0 - 2026-11-11

- Add copy-ready prompts for product explainers, feature overviews, support knowledge-base articles, and release notes.
- Include the combined product idea to PRD and Jira backlog workflow.
- Link the prompt catalogue from `SKILL.md` and include prompt files in the integrity manifest.

## 1.0.0 - 2026-11-11

- Initial release of Confluence document authoring templates, examples, and references for engineering, product, business, and support documentation.
- Include evidence-based composition, document lifecycle, quality review, and Jira integration guidance.
- Declare version `1.0.0` and author `Vincent Chu` in skill metadata.
