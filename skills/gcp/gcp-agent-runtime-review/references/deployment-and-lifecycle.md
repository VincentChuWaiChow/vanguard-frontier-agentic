# Deployment and Lifecycle

Answers one question: how an agent reaches Agent Runtime, how it changes and how it is removed, and which of those steps are irreversible, quota-bound or easy to get wrong because of renamed APIs. Base for every page below: https://docs.cloud.google.com/gemini-enterprise-agent-platform/ (verified 2026-11-01).

## Name map

The product was renamed, and the old names survive in APIs, CLIs and Terraform. Reviewers map every name in the evidence to one resource before judging it.

| Where | Name in use | Source |
|---|---|---|
| Product | Agent Runtime, part of Gemini Enterprise Agent Platform; formerly Vertex AI Agent Engine (old URLs under `cloud.google.com/vertex-ai/generative-ai/docs/agent-engine/` redirect here) | `build/runtime` |
| API resource | `ReasoningEngine`, kept "to maintain backwards compatibility"; REST path `projects/PROJECT/locations/LOCATION/reasoningEngines/ID` on `LOCATION-aiplatform.googleapis.com` | `build/runtime`, `scale/runtime/manage-deployed-agents` |
| Python SDK, current | `agentplatform.Client(project=..., location=...)` with `client.runtimes.create/get/list/update/delete` | `scale/runtime/manage-deployed-agents`, `build/runtime/sdk-migration` |
| Python SDK, deprecated | `vertexai.agent_engines.*`; "In version 2.0.1 of the Agent Platform SDK, the agent_engines module within the SDK was refactored and renamed to runtimes under the standalone agentplatform package." No removal date is stated | `build/runtime/sdk-migration` |
| ADK CLI | `adk deploy agent_engine` (subcommand keeps the old name) | https://adk.dev/api-reference/cli/ |
| Terraform | `google_vertex_ai_reasoning_engine`, GA and beta providers, Terraform 1.5.0 or later | `scale/runtime/use-terraform` |

New code on `vertexai.agent_engines` is a medium finding (deprecated namespace). Mixed use of both SDKs in one pipeline is a high finding, because two code paths now own one resource.

## Deployment methods

`scale/runtime/deploy-an-agent` lists deployment from an agent object, local source files, a Dockerfile, a container image in Artifact Registry, and a connected Git repository (Developer Connect). The page repeats "You can use this deployment method with Python only." for four source-based methods; the container-image path is the language-neutral one. Which heading each sentence belongs to was inferred from adjacent text, so confirm on the page for a non-Python agent.

ADK CLI flags (https://adk.dev/api-reference/cli/, 2026-11-01): `adk deploy agent_engine [OPTIONS] AGENT`, with `--project`, `--region`, `--agent_engine_id` (update an existing instance instead of creating one), `--display_name`, `--service_account`, `--worker_pool` (private Cloud Build pool for VPC-SC or private networks), `--otel_to_cloud`, `--adk_version`, `--agent_engine_config_file`, `--session_service_uri` and `--memory_service_uri`.

Deprecated flags with a conflict in the docs: the CLI reference marks `--staging_bucket` "Deprecated. This argument is no longer required or used." and `--trace_to_cloud` deprecated in favour of `--otel_to_cloud`; it also notes `--adk_app` is deprecated. ADK's BigQuery Agent Analytics integration page still shows `--staging_bucket` and `--adk_app` in its deploy example. The CLI reference wins. A pipeline that passes `--staging_bucket` expecting an upload location is a low finding: the flag is ignored, so the artifact goes somewhere the author did not intend to control.

## Change and rollback

- Updates: `client.runtimes.update(name=RESOURCE_NAME, agent=..., config={...})`. "If you update versioned fields, you will create a new revision." (`scale/runtime/manage-deployed-agents`)
- Traffic: "You can then split traffic between the different active revisions." Percentages must sum to 100; the default is `trafficSplitAlwaysLatest`, so every update takes all traffic at once. "At this time, revisions and traffic splitting are available through the v1beta1 API." (`scale/runtime/manage-revisions-and-traffic`)
- Rollback: the docs name no rollback procedure. The available mechanism is a manual split, `config={"traffic_config": {"trafficSplitManual": {"targets": [...]}}}`, pointing traffic back at an earlier active revision. Archiving is not reversible: "archived revisions can't be restored." A rollback plan that depends on an archived revision is a blocking finding.
- A direct query to a revision path bypasses the traffic rules, so canary tests that call a revision directly do not prove what production traffic sees.
- Revision quota: a default of 100 revisions for new agents and a hard limit of 950 (same page).

Review rule: a production change must state the previous active revision it can return to, the traffic mode before and after, and who approves the switch.

## Deletion

- Calls: `client.runtimes.delete(name=RESOURCE_NAME, force=True)` or `remote_agent.delete(force=True)`. The only documentation of `force` is the code comment "Optional, if the agent has resources (e.g. sessions, memory)". Treat `force=True` as deleting the agent's sessions and memories with it until the docs say otherwise, and require an export decision first.
- Identity: "Deleting the agent runtime invalidates its agent identity. Any IAM Unified Access Policy (UAP) that grants permissions to that principal retains the defunct principal in its rules." Deletion therefore needs a follow-up to remove those bindings.
- Recreating an agent does not restore the old identity: the agent-identity principal embeds the reasoning engine ID (see [identity-and-network-boundaries.md](identity-and-network-boundaries.md)), so every grant must be re-made for the new principal.

Deletion is irreversible. This skill never runs it; it reviews a deletion plan and hands it to an accountable human.

## Quotas that shape the design

From `resources/agent-quotas`, per project and region unless raised:

- Create, delete or update Agent Platform resources: 10 per minute. A pipeline that redeploys many agents in a loop will hit this; plan batches.
- Query or StreamQuery: 90 per minute. Size this against peak traffic before launch; it is a hard ceiling on user requests until raised.
- A2A agent post requests (`sendMessage`, `cancelTask`): 60 per minute.

## Failure modes and recovery

| Failure | Safe response | Unsafe response |
|---|---|---|
| Create call times out or the acknowledgement is lost | List runtimes by display name and check for a new resource before any retry | Blind retry: a second create may produce a duplicate runtime with its own identity (inference; the docs do not state create idempotency) |
| Update produced a bad revision | Manual traffic split back to the previous active revision, then fix forward | Archiving the bad revision before traffic has moved |
| A container deployment never becomes ready | Check that the identity pulling the image can read that one Artifact Registry repository | Granting broad Artifact Registry roles at project level |
| Quota exhausted mid-rollout | Stop, record which agents changed, resume after the window | Retrying in a tight loop, which burns the shared write quota |
| Deprecated SDK removed in a future release | Migrate to `agentplatform` runtimes on a schedule | Pinning the old SDK with no owner or date |
| Agent deleted by mistake | Recreate, re-grant every permission to the new principal, restore data only from an export taken before deletion | Assuming the old identity or memories come back |
