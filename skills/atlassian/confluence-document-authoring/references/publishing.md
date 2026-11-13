# Publishing and native templates

Use this reference only for actual page/template operations or for preparing an explicit publishing plan. A useful content draft needs no live write.

## Match the operation and representation

| Surface | Handling |
| --- | --- |
| Portable skill template | Fill Markdown; remove author instructions from the finished document |
| Cloud editor | Use supported editor blocks, actual URLs/Smart Links, and supported macros |
| Connector | Inspect its input format, conversion, supported objects, and preservation limits |
| Page REST API | Follow the selected endpoint's body schema and representation |
| Native content-template API | Follow the template-specific contract; do not send a page payload |
| Data Center | Verify the installed edition/version before choosing syntax or APIs |

The Cloud editor does not support the legacy Insert → Markup workflow. A brace expression such as `{jira:PROJ-123}` is not a reliable paste-to-render instruction there. This editor limitation is not a statement that every REST endpoint rejects wiki representation. [Wiki markup documentation](https://support.atlassian.com/confluence-cloud/docs/insert-confluence-wiki-markup/)

The current v2 page body schema lists `storage`, `atlas_doc_format`, and `wiki`; `body.value` is a string. Use the exact active contract. For `atlas_doc_format`, serialize the ADF document into that string. ADF, storage XML, Markdown, and legacy wiki text are distinct formats; serialize/escape the chosen representation correctly. Do not wrap unconverted Markdown in a storage body. [Page API](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/) · [Official v2 schema](https://dac-static.atlassian.com/cloud/confluence/openapi-v2.v3.json)

## Create or update a page

1. Resolve the site, space, parent, and page ID from actual context. Read current content, metadata, restrictions, and version for an update. Check for existing matching content before creation, without claiming a limited search proves uniqueness.
2. Preserve unrelated sections, macros, attachments, links, and restrictions. Use a representation that retains existing structured content. A Markdown round trip may lose rich content.
3. Prepare the exact requested change. Use existing authorization when target and intent are clear. A request for a draft authorizes drafting; “draft” in conversation does not require changing a published page's API status. For page creation, set the intended publication state explicitly: v2 creates published content by default unless `status: "draft"` is supplied.
4. For an ordinary v2 update, use the freshly retrieved version plus one and preserve publication state. The current v2 schema requires `version.number: 1` when updating status to `draft`; do not apply the plus-one rule to that case. Changing `current` to `draft` can replace an existing draft. Updating current content may also reconcile with, or override significantly diverged content in, an existing draft. Inspect current drafts and the active contract before a potentially destructive transition. A connector may handle these details itself; inspect its input contract first. [Page API](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/) · [v2 schema](https://dac-static.atlassian.com/cloud/confluence/openapi-v2.v3.json)
5. On conflict, fetch again, compare the intervening change, and recompute the delta. Never merely increment a stale payload's version. After a timeout with an uncertain outcome, read back before retrying a create.
6. Verify the resulting body, version, intended labels/parent, and links. Verify formatting/rendering where available. Report what could not be inspected. Resolve only failed follow-up operations after partial success.

Steps 1–6 are engineering safeguards, not a promise that every connector exposes conditional updates or draft inspection. If safe preservation is impossible, keep the prepared content and identify the specific unresolved operation.

## Install a native template when requested

A space template requires the corresponding admin capability; a global template requires the global administrator capability. Native templates support variables, placeholder instructions, and labels. Variable types include Text, Multi-line Text, and List; repeated variable names share values. Images and attachments have documented template limitations. [Create a template](https://support.atlassian.com/confluence-cloud/docs/create-a-template/) · [Global templates](https://support.atlassian.com/confluence-cloud/docs/edit-a-global-template/)

Map the portable `{{PLACEHOLDERS}}` to native variables where supported and author instructions to native placeholder text. Do not assume Markdown markers become interactive variables automatically.

The content-template REST endpoint's prose requires storage-format content and does not support creating/updating blueprints. Its generated examples include broader body representations; follow the specific prose contract and validate the payload instead of copying the full example. [Template API](https://developer.atlassian.com/cloud/confluence/rest/v1/api-group-template/)

Resolve template scope and existing template identity before writing. Use the requested create/update authorization; ask only for unresolved scope or a missing material choice. Verify the template's presence, labels, and variable behavior when the available surface permits it. If capability or access is missing, finish the reusable content and state that native installation remains incomplete.

Keep these claims distinct: reusable template authored; page created from template; native template installed. Only report the operation actually verified.
