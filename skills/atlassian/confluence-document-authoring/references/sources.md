# Sources and compatibility

Reviewed: 2026-11-11. Skill version: 1.2.0.

## Inspiration and authorship

The user supplied the document categories and an excerpt from [Confluence Documentation Patterns](https://skills.sh/thelobbi/claude/confluence-documentation-patterns), maintained in [TheLobbi/claude](https://github.com/TheLobbi/claude/blob/main/plugins/delivery-orchestrator/skills/confluence/SKILL.md). The linked listing provides pattern outlines. The complete templates and working procedures in this skill are independently authored; the upstream package is not bundled or installed.

[Atlassian's AI ticket article](https://www.atlassian.com/blog/jira/writing-tickets-for-ai-agents) informs the companion workflow connecting design, decisions, scoped Jira work, and verification. It does not validate these templates or guarantee a productivity gain.

## Verified product references

- CQL: [fields](https://developer.atlassian.com/cloud/confluence/cql-fields/), [functions](https://developer.atlassian.com/cloud/confluence/cql-functions/), [operators](https://developer.atlassian.com/cloud/confluence/cql-operators/), and [text search](https://developer.atlassian.com/cloud/confluence/performing-text-searches-using-cql/).
- Editor: [wiki markup limitations](https://support.atlassian.com/confluence-cloud/docs/insert-confluence-wiki-markup/).
- Jira embeds: [work items](https://support.atlassian.com/confluence-cloud/docs/insert-the-jira-issues-macro/), [charts](https://support.atlassian.com/confluence-cloud/docs/insert-the-jira-chart-macro/), and [timeline](https://support.atlassian.com/confluence-cloud/docs/insert-the-jira-timeline-macro/).
- Publishing: [page API](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/) and [official v2 schema](https://dac-static.atlassian.com/cloud/confluence/openapi-v2.v3.json).
- Native templates: [create a template](https://support.atlassian.com/confluence-cloud/docs/create-a-template/), [global templates](https://support.atlassian.com/confluence-cloud/docs/edit-a-global-template/), and [content-template API](https://developer.atlassian.com/cloud/confluence/rest/v1/api-group-template/).
- Decision-history convention: [AWS ADR process](https://docs.aws.amazon.com/prescriptive-guidance/latest/architectural-decision-records/adr-process.html).

## Known documentation discrepancies

Official CQL examples use `now()` offsets although the function catalogue omits them. Do not interpret that omission as proof of invalid syntax. The native-template API's generated sample is broader than its prose body-format requirement. The specific references explain both cases and require target verification where appropriate.

Cloud is the reference integration. Data Center, tenant features, connector schemas, macro renderers, permissions, and API versions can vary. The skill remains useful for drafting without a connection. Recheck the active contract before live writes, preserve existing content, and accurately report what was verified.

Template structures, review gates, evidence classification, document lifecycle conventions, and proposed labels are design choices, not mandatory Atlassian fields. Example documents are explicitly fictional and are not evidence about the user's infrastructure.

## Product, business and support expansion

The user-supplied [Notion engineering catalogue](https://www.notion.com/templates/search?query=engineering) was inspected on 2026-11-11 as inspiration for engineering, product explanations in business language, bug reporting, new features and support. The fetched catalogue was partial and dynamic; no exhaustive count or quality ranking is claimed. Added templates and synthetic examples are independently authored, not copies or imports of Notion templates. The new routing and audience conventions are design choices, not mandated Jira or Confluence fields.

## Public Atlassian catalogue expansion

On 2026-11-11, the user-supplied [Confluence template gallery](https://www.atlassian.com/software/confluence/templates) and eight linked detail pages were inspected. The [extended catalogue](template-catalogue.md) maps each selected pattern to its public source. This is a bounded selection, not exhaustive gallery coverage.

The new Markdown templates and prompts are independently authored using this skill's evidence and lifecycle conventions. They are not copies of native template bodies, MCP exports, official Atlassian deliverables, or proof of native installation. Source inspiration does not validate a filled document's claims, approvals, or operational outcomes. Recheck the active connector's actual capabilities and permissions before attempting any native template read or write.

## Upstream comparison for template improvements

Reviewed on 2026-11-11 at `borghei/Claude-Skills` revision `25bf86b89c6c1e63f433a5b65e8cfcb6cf0b7ad3`:

- [Confluence template guidance](https://github.com/borghei/Claude-Skills/blob/25bf86b89c6c1e63f433a5b65e8cfcb6cf0b7ad3/project-management/confluence-expert/references/templates.md): comparable document patterns and maintenance metadata. Its sibling `assets/` contains sample audit JSON, not native template exports.
- [Atlassian Confluence patterns](https://github.com/borghei/Claude-Skills/blob/25bf86b89c6c1e63f433a5b65e8cfcb6cf0b7ad3/project-management/atlassian-templates/references/confluence-templates.md): project-charter purpose, stakeholder responsibilities, and funding proposals.
- [Template-change governance](https://github.com/borghei/Claude-Skills/blob/25bf86b89c6c1e63f433a5b65e8cfcb6cf0b7ad3/project-management/atlassian-templates/references/workflows-and-governance.md): review impact, compatibility, migration, and communications when changing shared templates.
- [Notion PM assets](https://github.com/borghei/Claude-Skills/tree/25bf86b89c6c1e63f433a5b65e8cfcb6cf0b7ad3/project-management/notion-pm/assets): audience-oriented roadmap summaries and dated OKR updates. These concepts are adapted as plain authoring sections, not Notion database or sync features.

The upstream [license](https://github.com/borghei/Claude-Skills/blob/25bf86b89c6c1e63f433a5b65e8cfcb6cf0b7ad3/LICENSE) is MIT plus Commons Clause, not plain MIT. No upstream files, scripts, native template bodies, macros, or schemas are bundled. New content is independently authored; future copying requires its own license/attribution review. Fixed contingency rates, confidence scales, review cadences, populated example facts, automatic sign-off/status changes, assumed integrations, and score-based readiness were not adopted. Filtering a roadmap view is not an access-control guarantee.
