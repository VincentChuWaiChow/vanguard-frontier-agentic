# Official Sources

Answers one question: which primary source supports each claim in this skill, how it was checked and what it does not prove. All entries were verified on 2026-11-01. "Verbatim" means the quoted text was read on the page or in its published source; "Corroborated" means Context7 returned the same call shape. Google Cloud pages are relative to https://docs.cloud.google.com/gemini-enterprise-agent-platform/.

## Google Cloud: Agent Runtime

| Page | Claim supported | Stage or version | Check | Limitation |
|---|---|---|---|---|
| `build/runtime` | Product name Agent Runtime; resource kept as `ReasoningEngine` for backwards compatibility; VPC-SC compliance; A2A marked preview | Page carries no GA label for the runtime | Verbatim | Old `cloud.google.com/vertex-ai/.../agent-engine/` URLs 301-redirect here |
| `build/runtime/setup` | `roles/aiplatform.user` to use an agent; `roles/iam.serviceAccountUser` with a custom service account; default Reasoning Engine Service Agent and `roles/aiplatform.reasoningEngineServiceAgent` | n/a | Verbatim | Names no separate deploy-only role |
| `build/runtime/sdk-migration` | `vertexai.agent_engines` renamed to `agentplatform` runtimes in SDK 2.0.1; old namespaces in deprecation | Deprecation phase | Verbatim | No removal date stated |
| `scale/runtime/deploy-an-agent` | `client.runtimes.create`; deployment methods; four "Python only" methods; `identity_type=AGENT_IDENTITY`; Secret Manager grant to the Agent Platform Service Agent | No Preview label on `identity_type` | Verbatim | Heading-to-sentence mapping for "Python only" was inferred |
| `scale/runtime/manage-deployed-agents` | `agentplatform.Client(...)`; `client.runtimes.update`; new revision on versioned-field updates; `delete(force=True)`; deletion invalidates agent identity and leaves defunct UAP principals | n/a | Verbatim | `force` is documented only by a code comment |
| `scale/runtime/manage-revisions-and-traffic` | Traffic split across active revisions; default `trafficSplitAlwaysLatest`; archived revisions cannot be restored; revision quota 100 default, 950 hard | v1beta1 API | Verbatim | No named rollback procedure |
| `scale/runtime/agent-identity` | Per-agent principal format; default roles; Legacy Bucket role limitation | Access to own sessions, memories, sandboxes marked Preview | Verbatim | Launch stage of the identity feature itself not stated |
| `scale/runtime/private-service-connect-interface` | Default Google-managed network with no VPC access; PSC interface bridges to a VPC | n/a | Verbatim | Configuration steps not summarized here |
| `scale/sandbox/configure-vpc-sc` | PSC-E and PSC-I make sandboxes VPC-SC compliant | n/a | Verbatim | Sandbox only, not the runtime |
| `scale/runtime/tracing` | `GOOGLE_CLOUD_AGENT_ENGINE_ENABLE_TELEMETRY` enables traces, logs and metrics; OpenTelemetry for AdkApp via environment variables; export to Cloud Trace | n/a | Verbatim | Env var values may change with the GenAI conventions |
| `scale/memory-bank` | Memories isolated per scope dictionary | No stage label found | Verbatim | Retention and deletion semantics not reviewed here |
| `resources/agent-quotas` | 10 resource writes per minute; 90 queries per minute; 60 A2A posts per minute | n/a | Verbatim | Project and region defaults; may be raised |
| `scale/runtime/use-terraform` | `google_vertex_ai_reasoning_engine`; Terraform 1.5.0 or later | GA and beta providers | Verbatim | Beta-only arguments not enumerated here |

## Agent Development Kit

| Source | Claim supported | Stage or version | Check | Limitation |
|---|---|---|---|---|
| https://pypi.org/project/google-adk/ | 2.13.0 latest (2026-10-23); 1.39.1 on the 1.x line; 1.27.0 yanked "Broke Agent Engine Deployments"; Python 3.11+ | 2.13.0 | Verbatim | Bi-weekly cadence; recheck on every review |
| https://github.com/google/adk-python | `Agent` and `Workflow` as the 2.x building blocks; 2.13.0 telemetry default change and revert variable | 2.13.0 | Verbatim | Release notes, not a stability promise |
| https://adk.dev/api-reference/cli/ | `adk deploy agent_engine` options; `--staging_bucket`, `--trace_to_cloud`, `--adk_app` deprecated; `adk eval` usage | 2.13.0 docs | Verbatim and Corroborated (Context7 `/google/adk-docs`) | An integration page still uses two deprecated flags |
| https://adk.dev/agents/workflow-agents/loop-agents/ | Templated workflows superseded in 2.0; `LoopAgent` needs an explicit termination; `max_iterations` | 2.x | Verbatim | No default `max_iterations` stated |
| https://adk.dev/workflows/collaboration/ | `chat`, `task`, `single_turn` modes; task mode leaf-only | Python and Go v2.0.0 | Verbatim | Task mode disabled in graph workflows on 2.0.0 |
| https://adk.dev/sessions/session/ and https://adk.dev/sessions/state/ | Session services; `InMemorySessionService` loses data on restart; state prefixes | 2.x | Verbatim | Database schema migration notes not covered |
| https://adk.dev/sessions/memory/ | Memory service implementations and interface | 2.x | Verbatim | Optional methods vary by service |
| https://adk.dev/callbacks/types-of-callbacks/ | Callback names and parameters; exact parameter names in Python; model and tool callbacks on `LlmAgent` | 2.x | Verbatim | Full page was truncated during review; return semantics taken from the next row |
| Context7 `/google/adk-python` (`contributing/samples/core/callbacks/README.md`, `src/google/adk/integrations/mongodb/README.md`) | Returning a dict from `before_tool_callback` skips the tool; `None` proceeds | main branch | Corroborated | Context7 listed versions only up to v2.6.1 |
| https://github.com/google/adk-docs/blob/main/docs/tools-custom/confirmation.md | Tool confirmation is experimental and does not support `DatabaseSessionService` or `VertexAiSessionService` | ADK Python 1.14.0 and later | Verbatim (Known limitations section) | Rendered at https://adk.dev/tools-custom/confirmation/ |
| https://adk.dev/safety/ | Risk sources incl. indirect injection; Agent-Auth versus User-Auth; plugins for cross-agent policy; sandboxed code execution | 2.x | Verbatim | Tail of the page was summarized, not read verbatim |
| https://adk.dev/evaluate/ and https://adk.dev/evaluate/criteria/ | Python-only evaluation; file suffixes; criteria names | 2.x | Verbatim | Default thresholds not verified, so not stated |
| https://adk.dev/a2a/quickstart-exposing/ | `to_a2a`; card path; experimental in Python | Experimental | Verbatim | Consuming side not reviewed here |
| https://adk.dev/observability/traces/ | OpenTelemetry GenAI spans emitted by ADK | Python v1.17.0 and later | Verbatim | Conventions are Development status (https://github.com/open-telemetry/semantic-conventions-genai) |
