# SYNTHETIC runbook: investigate a growing queue backlog

**SYNTHETIC EXAMPLE — NOT EXECUTED.** The observations below are invented to demonstrate an evidence-based investigation. They are not output from a cluster. Commands are read-only examples with unresolved target placeholders; no cluster access, recovery action, or operational validation has occurred.

| Metadata | Value |
|---|---|
| Document lifecycle | draft |
| Operational owner | unknown |
| Reviewer | unknown |
| Revision / date | synthetic revision 0.1 / unknown |
| Sources | Invented observations SIM-01 through SIM-04; official kubectl references below |
| Jira / related documentation | unknown / unknown |
| Review trigger | Worker rollout, queue contract change, incident learning, or completed recovery review |
| Procedure validation state | NOT RUN |

The lifecycle is document review status, not Confluence publication or evidence of successful execution.

## Purpose and scope

Determine whether queue growth is associated with unavailable workers, downstream throttling, an ingress increase, or an unresolved measurement gap. The runbook stops at a bounded diagnosis and recovery handoff; it does not authorize changing replicas, queue contents, deployment versions, or processing concurrency.

Synthetic impact: jobs are waiting longer. Actual customer impact, severity, processing deadline, queue provider, worker version, and service-level objective are **unknown**.

## Preconditions and exact target

| Required input | Value | Verification source / state |
|---|---|---|
| Approved environment / account | {{environment_and_account}} | {{inventory_or_incident_reference}}; NOT CHECKED |
| Kubeconfig context / expected API endpoint | {{kube_context}} / {{expected_api_endpoint}} | {{approved_inventory_reference}}; NOT CHECKED |
| Namespace / deployment | {{namespace}} / {{worker_deployment}} | {{service_inventory_reference}}; NOT CHECKED |
| Worker selector | {{worker_selector}} | Must match the deployment's selector and verified pod ownership; NOT CHECKED |
| Worker pod / ReplicaSet / container | {{worker_pod}} / {{worker_replicaset}} / {{worker_container}} | Select only from verified workload ownership; NOT CHECKED |
| Queue identity and metrics | {{queue_name}} / {{metrics_dashboard_url}} | {{queue_mapping_and_metric_definition_source}}; NOT CHECKED |
| Minimum access | Read the target deployment, ReplicaSet, pods, events, and selected pod logs | {{access_evidence}}; NOT CHECKED |
| Operator / incident authority | {{operator_and_authorized_read_scope}} | {{task_or_incident_reference}}; unknown |
| kubectl / cluster versions | {{client_and_server_versions}} | Compatibility not verified |

Resolve all placeholders before adapting the commands. Do not treat a context name, matching label, or `Running` pod phase alone as proof of correct target or healthy processing. Read bounded, redacted diagnostics; do not paste credentials or full sensitive payloads into the incident record.

## Stop conditions

Stop the affected investigation branch if the endpoint or workload identity differs from the approved inventory, access is denied, selected pods belong to another workload, the queue-to-worker mapping is unknown, or evidence is stale or contradictory. Do not infer an SLO breach or incident severity without its threshold and source. Recovery steps remain gated until their exact target, authority, procedure, and reversal limits are established.

## Procedure

| Step | Action | Expected observation | If different |
|---|---|---|---|
| 1 | Read-only: inspect the selected context's endpoint using command A. | Endpoint matches the independently approved target. | Stop; resolve the inventory/context mismatch. |
| 2 | Read-only: inspect deployment, selector, pods, and ReplicaSet ownership using commands B–E. | Generation is observed; ownership chain points to the intended deployment; readiness can be assessed. | Branch to workload identity or availability investigation. Do not count unrelated pods as capacity. |
| 3 | Read-only: record queue depth, oldest age, arrivals, completions, metric definitions, and timestamps from the verified dashboard. | Metrics refer to the same queue and interval; the direction of backlog growth can be explained. | Treat telemetry as incomplete; do not calculate a drain estimate from incomparable rates. |
| 4 | Read-only: inspect bounded events and logs for the verified pod/container using commands F–G. | Evidence can support or contradict unavailable-worker and downstream-failure hypotheses. | Record unavailable evidence or contradictory results; avoid assuming absence of log entries means no failure. |
| 5 | Compare evidence against the branch table below. | A bounded next check or escalation is selected, with remaining uncertainty explicit. | Keep multiple hypotheses open and collect the missing discriminating evidence. |
| 6 | Record target, evidence window, hypotheses, and required recovery owner. | Another operator can reproduce the reasoning and understand what has not been done. | Keep the investigation open; do not label the queue recovered. |

### Read-only command examples — NOT RUN

These commands follow the referenced kubectl syntax. They have not been executed or validated against a target version. Missing output fields are **unknown**, not automatically zero.

**A — selected context endpoint only:**

```sh
kubectl --context="{{kube_context}}" config view --minify -o jsonpath='{.clusters[0].cluster.server}'
```

**B — deployment identity and observed generation:**

```sh
kubectl --context="{{kube_context}}" --namespace="{{namespace}}" get deployment "{{worker_deployment}}" -o custom-columns='NAME:.metadata.name,UID:.metadata.uid,GENERATION:.metadata.generation,OBSERVED:.status.observedGeneration,DESIRED:.spec.replicas,READY:.status.readyReplicas,AVAILABLE:.status.availableReplicas,UPDATED:.status.updatedReplicas'
```

**C — deployment selector to compare with the supplied selector:**

```sh
kubectl --context="{{kube_context}}" --namespace="{{namespace}}" get deployment "{{worker_deployment}}" -o jsonpath='{.spec.selector}'
```

**D — selected pods and their owners:**

```sh
kubectl --context="{{kube_context}}" --namespace="{{namespace}}" get pods --selector="{{worker_selector}}" -o custom-columns='NAME:.metadata.name,OWNER_KIND:.metadata.ownerReferences[0].kind,OWNER:.metadata.ownerReferences[0].name,READY:.status.containerStatuses[*].ready,RESTARTS:.status.containerStatuses[*].restartCount,PHASE:.status.phase'
```

**E — ReplicaSet owner must match the deployment name and UID from B:**

```sh
kubectl --context="{{kube_context}}" --namespace="{{namespace}}" get replicaset "{{worker_replicaset}}" -o custom-columns='NAME:.metadata.name,OWNER_KIND:.metadata.ownerReferences[0].kind,OWNER:.metadata.ownerReferences[0].name,OWNER_UID:.metadata.ownerReferences[0].uid'
```

Repeat the ownership check for each distinct ReplicaSet before treating all selected pods as part of the deployment. Confirm the controller owner if multiple owner references are present.

**F — events for one verified worker pod:**

```sh
kubectl --context="{{kube_context}}" --namespace="{{namespace}}" get events --field-selector="involvedObject.name={{worker_pod}}" --sort-by='.metadata.creationTimestamp'
```

**G — bounded logs for one verified container:**

```sh
kubectl --context="{{kube_context}}" --namespace="{{namespace}}" logs "{{worker_pod}}" --container="{{worker_container}}" --since=10m --tail=200 --limit-bytes=65536 --timestamps=true
```

Record the returned event object's identity and time window; a reused pod name or an empty event list is not proof that no incident occurred. Log sampling is bounded and may omit relevant events.

## Invented observations for this worked example

**Every entry in this section is simulated.** It does not populate the real execution evidence record.

| ID | Synthetic observation | Interpretation / limitation |
|---|---|---|
| SIM-01 | Four desired workers; four ready and available; observed generation matches the deployment generation; sampled pod ownership matches. | No unavailability is shown in this snapshot. Readiness alone does not demonstrate completed jobs. |
| SIM-02 | Queue depth rises from 1,200 at 10:00 to 5,700 at 10:05 to 10,200 at 10:10 in a fictional observation window. | The simulated average increase is 15 jobs/second. Actual metric definitions remain unverified. |
| SIM-03 | Simulated arrival rate is 20 jobs/second and completion rate is 5 jobs/second throughout that window. | Their difference is consistent with SIM-02 under the scenario's simple queue accounting assumption; it is not a measured production rate. |
| SIM-04 | Two sampled worker containers show downstream HTTP `429` responses during the same window; no restarts appear in the sampled pod status. | Throttling is a plausible contributor. The scope, authoritative retry contract, and causal effect remain unknown. |

For the synthetic rates only, 20 arrivals minus 5 completions equals 15 additional jobs per second, or 4,500 jobs per five minutes. A drain-time claim would be unjustified while completions remain below arrivals. The actual rates and any recovery estimate require real observations and consistent metric definitions.

## Evidence branches

| Evidence | Working hypothesis | Next discriminating check | Recovery implication |
|---|---|---|---|
| Ready/available replicas below the intended baseline, with matching ownership and recent warning events | Worker availability may limit throughput. | Check scheduling, readiness failure, image/startup errors, and active rollout evidence. | Use a reviewed workload recovery procedure only after identifying the failure; none is supplied here. |
| Workers appear available; downstream `429` responses coincide with low completions | A downstream limit may constrain useful work. | Obtain the actual downstream rate-limit scope, `Retry-After` behavior, worker retry policy, and request-rate telemetry. | Increasing worker replicas could increase rejected traffic. A reviewed concurrency or admission change may be appropriate, but is not authorized by this example. |
| Arrival rate increases while completion rate remains near a verified baseline | Ingress may exceed processing capacity. | Compare arrivals and completions over matching windows; verify per-tenant distribution and agreed capacity limits. | Capacity or admission decisions need measured limits and service-owner authority. |
| Dashboard identity, metric definitions, or queue mapping is missing | Diagnosis remains unknown. | Resolve the telemetry mapping before relying on the trend. | No recovery command follows from ambiguous metrics. |

**Synthetic working conclusion:** downstream throttling is a supported hypothesis to investigate next. It is not a confirmed root cause, and no recovery is demonstrated.

## Recovery / rollback handoff

| Candidate situation | Required procedure and authority | Reversal / data constraints | Current state |
|---|---|---|---|
| Downstream throttling is confirmed | Reviewed change to concurrency, retry behavior, or admission; exact target and service-owner authorization required. | Preserve queued jobs, ordering/idempotency guarantees, and original settings; verify the downstream contract first. | Procedure, owner, and approval unknown; NOT RUN |
| A rollout-related worker failure is confirmed | Reviewed version-recovery plan tied to an actual deployment record. | Check data and message compatibility; a previous image is not automatically safe to restore. | Procedure and rollback compatibility unknown; NOT RUN |
| Queue drain or replay is proposed | Queue-provider procedure and data-owner decision, with loss/duplication analysis. | Message deletion or replay can change business outcomes; no such operation is included here. | NOT RUN |

No executable mutation or recovery command is supplied because the necessary target evidence and recovery contract are absent.

## Escalation and validation

Escalation role: worker service owner; if downstream throttling is supported, include the downstream service owner through the verified incident process. Names, contact route, incident link, and next checkpoint are **unknown**. Provide target identity, timestamps, queue definitions, bounded redacted evidence, and the explicit statement that no mutation was performed.

| Completion or recovery claim | Required evidence | Actual evidence / state |
|---|---|---|
| Correct target and permissions established | Independent inventory comparison and authorized access record | unknown; NOT RUN |
| Backlog stabilizing or shrinking | Comparable queue metrics over an agreed observation window | unknown; NOT RUN |
| Job timeliness restored | Oldest age and customer-impact evidence against a verified objective | objective unknown; NOT RUN |
| No adverse effects from recovery | Verified duplicate/loss/error checks appropriate to the real contract | recovery not performed; NOT RUN |

Execution record: run ID `{{run_id}}`; operator **unknown**; observation time **unknown**; actual outcome **NOT RUN**. Follow-up: establish the real service contract, owner, and operational baseline before adapting this example.

## Command reference sources

These references establish command syntax, not the truth of the simulated observations or compatibility with a particular cluster:

- [kubectl get](https://kubernetes.io/docs/reference/kubectl/generated/kubectl_get/)
- [kubectl logs](https://kubernetes.io/docs/reference/kubectl/generated/kubectl_logs/)
- [kubectl config view](https://kubernetes.io/docs/reference/kubectl/generated/kubectl_config/kubectl_config_view/)
