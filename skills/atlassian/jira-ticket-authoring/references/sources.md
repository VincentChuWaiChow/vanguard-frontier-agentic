# Sources and compatibility

Reviewed: 2026-11-11. Skill version: 1.2.0.

## Primary sources

- [Atlassian: Why better tickets help agents write better code](https://www.atlassian.com/blog/jira/writing-tickets-for-ai-agents), published 2026-08-09. Source inspiration for bounded work, explicit criteria, contextual planning, and capturing incidental findings. This is a reported team comparison, not evidence that this skill or any template guarantees a productivity multiplier.
- [Jira Cloud REST v3: issues](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-issues/). Body formats, create metadata, parent and subtasks.
- [Jira Cloud work types](https://support.atlassian.com/jira-cloud-administration/docs/what-are-issue-types/). Work-type terminology and hierarchy context.
- [Issue linking model](https://developer.atlassian.com/cloud/jira/platform/issue-linking-model/) and [issue link API](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-issue-links/). Relationship semantics and operation behavior.
- [Rovo MCP supported tools](https://developer.atlassian.com/cloud/rovo-mcp/guides/supported-tools/). Dynamic connector capability discovery.

## Scope of claims

The integrated templates, readiness verdicts, evidence classification, acceptance-to-verification mapping, and optional modules are original engineering conventions. They are not mandated Atlassian fields or official certifications. The supplied article motivates the workflow; the detailed templates are independently authored.

The skill is useful without a connected Atlassian account. Authoring is portable; live execution depends on the installed connector, access, project configuration, and current API. Cloud is the reference integration. Data Center requires a version-specific check before writes.

Do not treat a successfully filled template or a static review as proof of test execution, safe deployment, business approval, or actual productivity gains. Inspect the relevant evidence when making those claims.

## Product, business and support expansion

The user-supplied [Notion engineering catalogue](https://www.notion.com/templates/search?query=engineering) was inspected on 2026-11-11 as inspiration for engineering, product explanations in business language, bug reporting, new features and support. The fetched catalogue was partial and dynamic; no exhaustive count or quality ranking is claimed. Added templates and synthetic examples are independently authored, not copies or imports of Notion templates. The new routing and audience conventions are design choices, not mandated Jira or Confluence fields.

## Upstream comparison for template improvements

Reviewed on 2026-11-11 at `borghei/Claude-Skills` revision `25bf86b89c6c1e63f433a5b65e8cfcb6cf0b7ad3`:

- [Jira expert](https://github.com/borghei/Claude-Skills/tree/25bf86b89c6c1e63f433a5b65e8cfcb6cf0b7ad3/project-management/jira-expert): intake quality and review pitfalls.
- [Atlassian Jira template patterns](https://github.com/borghei/Claude-Skills/blob/25bf86b89c6c1e63f433a5b65e8cfcb6cf0b7ad3/project-management/atlassian-templates/references/jira-templates.md): story, bug, epic, and completion-checklist concepts.
- [Template-change governance](https://github.com/borghei/Claude-Skills/blob/25bf86b89c6c1e63f433a5b65e8cfcb6cf0b7ad3/project-management/atlassian-templates/references/workflows-and-governance.md): impact review and compatibility before updating shared templates.

The upstream [license](https://github.com/borghei/Claude-Skills/blob/25bf86b89c6c1e63f433a5b65e8cfcb6cf0b7ad3/LICENSE) is MIT plus Commons Clause, not plain MIT. This comparison informs independently authored guidance and prompts; no upstream files, scripts, or template bodies are bundled. Do not infer license clearance for future copying. Fixed field/checkbox scores, automatic production-deployment checks, sample issue keys, and assumed tenant workflows were not adopted. Existing local evidence, stable-ID, and authorization safeguards remain controlling.
