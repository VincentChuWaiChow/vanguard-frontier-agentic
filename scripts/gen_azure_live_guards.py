#!/usr/bin/env python3
"""Generator: 6 Azure live-guard agents + 6 paired skills."""
import os, json, textwrap

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATE = "2026-04-30"

AGENTS = [
    {
        "id": "azure-live-arm-deployment-stack-guard",
        "name": "Azure Live ARM Deployment Stack Guard",
        "summary": "Guard ARM template and Deployment Stack changes with what-if evidence, denySettings review, and explicit approval before execute.",
        "focus": "Guard ARM/Bicep and Deployment Stack changes with `--what-if` evidence, `denySettings` audit, and explicit approval before any ARM execute.",
        "codex_role": "arm-deployment-stack live operator",
        "skill_desc": "Guard live ARM, Bicep, and Deployment Stack changes with what-if evidence, denySettings review, changeset diff, rollback posture, and approval gates.",
        "skill_when": [
            "an ARM or Bicep deployment must be previewed and possibly executed against a live Azure environment",
            "the session involves Deployment Stacks with denySettings and protected resource scopes",
            "a human needs guarded execution help with change evidence and rollback design",
        ],
        "response_shape": [
            "Target subscription, resource group, and active principal (az account show evidence)",
            "What-if diff output or deployment preview evidence",
            "denySettings posture and existing denyAssignments on target scope",
            "Approval status and change justification",
            "Proposed or executed ARM/Stack action",
            "Rollback posture (previous template ref or detach plan)",
            "Post-deploy verification steps and open risks",
        ],
        "official_docs": [
            "https://learn.microsoft.com/en-us/azure/azure-resource-manager/templates/deploy-what-if",
            "https://learn.microsoft.com/en-us/azure/azure-resource-manager/bicep/deployment-stacks",
            "https://learn.microsoft.com/en-us/azure/role-based-access-control/deny-assignments",
            "https://learn.microsoft.com/en-us/azure/azure-resource-manager/templates/best-practices",
        ],
        "security_notes": "Never execute an ARM or Deployment Stack change without what-if evidence, confirmed target scope, denySettings review, and explicit human approval. Repo write access does not authorize live Azure mutations.",
        "permissions_body": textwrap.dedent("""\
            # Least-privilege RBAC guidance

            ## Identity model preference

            1. PIM-eligible Contributor scoped to **target resource group only** — activated JIT for deploy windows
            2. Service principal with scoped Contributor for CI/CD pipelines — no standing access
            3. Do not use subscription-level Owner or Contributor for routine deployments

            ## Required Microsoft.* actions

            What-if and Deployment Stacks share the same permission boundary. There is no read-only what-if role;
            the operator must hold write permissions on the resources being deployed.

            ```json
            {
              "Name": "ARM Deployment Stack Guard",
              "IsCustom": true,
              "Description": "Minimum rights for guarded ARM what-if and Deployment Stack changes in one target resource group. Stack deletion is EXCLUDED — it requires a separate PIM-elevated role.",
              "Actions": [
                "Microsoft.Resources/deployments/read",
                "Microsoft.Resources/deployments/write",
                "Microsoft.Resources/deployments/whatIf/action",
                "Microsoft.Resources/deploymentStacks/read",
                "Microsoft.Resources/deploymentStacks/write",
                "Microsoft.Resources/subscriptions/resourceGroups/read"
              ],
              "NotActions": [
                "Microsoft.Resources/deploymentStacks/delete"
              ],
              "DataActions": [],
              "NotDataActions": [],
              "AssignableScopes": [
                "/subscriptions/<SUBSCRIPTION_ID>/resourceGroups/<TARGET_RG>"
              ]
            }
            ```

            `deploymentStacks/delete` is in `NotActions` above. Stack deletion requires a **separate
            PIM-eligible role** (see below) activated only for confirmed decommission windows.

            ### PIM-elevated delete role (activate only for planned decommission)

            ```json
            {
              "Name": "ARM Deployment Stack Delete (PIM)",
              "IsCustom": true,
              "Description": "Stack deletion only. Must be PIM-activated with approval and time-bound to a decommission window.",
              "Actions": [
                "Microsoft.Resources/deploymentStacks/read",
                "Microsoft.Resources/deploymentStacks/delete"
              ],
              "NotActions": [],
              "AssignableScopes": [
                "/subscriptions/<SUBSCRIPTION_ID>/resourceGroups/<TARGET_RG>"
              ]
            }
            ```

            Assign this role as **PIM-eligible** (not permanent active). Require manager approval
            and a maximum 2-hour activation window. Never combine with `deploymentStacks/write` in
            the same PIM activation unless you are replacing a stack.

            For each resource type touched by the template, add the matching write action, e.g.
            `Microsoft.Compute/virtualMachines/write` for VMs. This is unavoidable — what-if requires it.

            ## Deployment Stacks denySettings

            Recommended default for production stacks:

            ```bash
            az deployment-stack group create \\
              --deny-settings-mode denyDelete \\
              --deny-settings-apply-to-child-scopes \\
              ...
            ```

            `denyDelete` generates a platform-enforced `denyAssignment` on all managed resources.
            `denyWriteAndDelete` is stricter — use for compliance-mandated immutable resources.

            ## Do not assign

            - `Owner` at subscription scope
            - `Contributor` at management-group scope
            - Broad `Microsoft.Resources/*` wildcards
            - `Microsoft.Authorization/roleAssignments/write` (privilege escalation risk)
        """),
        "preflight_body": textwrap.dedent("""\
            # ARM Deployment Stack — Preflight Commands

            Run all of these before executing any live ARM or Deployment Stack change.

            ## 1. Confirm identity and active subscription

            ```bash
            az account show --query "{sub:id, tenant:tenantId, user:user.name, env:environmentName}"
            ```

            ## 2. What-if on ARM / Bicep template

            ```bash
            az deployment group what-if \\
              --resource-group <TARGET_RG> \\
              --template-file main.bicep \\
              --parameters @params.prod.json \\
              --result-format FullResourcePayloads
            ```

            Stop if what-if shows unexpected deletions or replacements. Deletions require separate approval.

            ## 3. Inspect current Deployment Stack state

            ```bash
            az deployment-stack group show \\
              --name <STACK_NAME> \\
              --resource-group <TARGET_RG> \\
              --query "{state:provisioningState, denySettings:denySettings, resourceCount:length(resources)}"
            ```

            ## 4. Review deny assignments on target scope

            ```bash
            az role assignment list \\
              --resource-group <TARGET_RG> \\
              --include-deny \\
              --query "[?type=='Microsoft.Authorization/denyAssignments'].{name:name,actions:denyAssignmentPermissions[0].actions}"
            ```

            ## 5. Validate template syntax

            ```bash
            az deployment group validate \\
              --resource-group <TARGET_RG> \\
              --template-file main.bicep \\
              --parameters @params.prod.json
            ```
        """),
        "rollback_body": textwrap.dedent("""\
            # ARM Deployment Stack — Rollback Playbook

            ## Option 1: Re-deploy previous known-good template (incremental)

            ```bash
            az deployment group create \\
              --resource-group <TARGET_RG> \\
              --template-file main.prev.bicep \\
              --parameters @params.prod.prev.json \\
              --mode Incremental \\
              --name "rollback-$(date +%Y%m%dT%H%M%S)"
            ```

            ## Option 2: Detach stack management without deleting resources

            ```bash
            az deployment-stack group delete \\
              --name <STACK_NAME> \\
              --resource-group <TARGET_RG> \\
              --action-on-unmanage detachAll \\
              --yes
            ```

            Use when the stack definition is wrong but the deployed resources are still healthy.

            ## Option 3: Full stack deletion (nuclear — bypass denySettings first)

            ```bash
            # REQUIRES: denySettings override or prior denyDelete removal
            az deployment-stack group delete \\
              --name <STACK_NAME> \\
              --resource-group <TARGET_RG> \\
              --action-on-unmanage deleteAll \\
              --bypass-stack-out-of-sync-error \\
              --yes
            ```

            WARNING: If `denySettings.mode = denyDelete`, this command fails by design.
            That failure is correct behavior — escalate to a Principal with deny-assignment write rights.

            ## Verify

            ```bash
            az deployment group show \\
              --resource-group <TARGET_RG> \\
              --name <DEPLOYMENT_NAME> \\
              --query "{state:properties.provisioningState, timestamp:properties.timestamp}"
            ```
        """),
    },
    {
        "id": "azure-live-pim-jit-activation-guard",
        "name": "Azure Live PIM JIT Activation Guard",
        "summary": "Gate PIM eligible role activations with justification, ticket binding, MFA verification, and time-bound scope before approval submission.",
        "focus": "Gate Entra ID PIM eligible role activations with justification, ticket reference, MFA verification, and time-bound scope before submission to the approval workflow.",
        "codex_role": "pim-jit-activation live operator",
        "skill_desc": "Gate Entra ID PIM eligible role activations with justification, MFA, ticket binding, time-bound scope, and approval workflow gates before any privileged Azure role becomes active.",
        "skill_when": [
            "a user or service principal must activate a PIM-eligible Azure or Entra ID role",
            "an approver must review and accept or reject a pending PIM activation request",
            "standing privileged access is being audited and time-bound JIT activation must be enforced",
        ],
        "response_shape": [
            "Eligible assignment confirmation (principal, role, scope, schedule)",
            "Existing active assignments check (avoid duplicate activation)",
            "Conditional Access and MFA posture verification",
            "Justification and ticket reference audit",
            "Activation request submission or approval action",
            "Time-bound window and expiry confirmation",
            "Post-activation access verification and open risks",
        ],
        "official_docs": [
            "https://learn.microsoft.com/en-us/entra/id-governance/privileged-identity-management/pim-deployment-plan",
            "https://learn.microsoft.com/en-us/entra/id-governance/privileged-identity-management/pim-resource-roles-configure-role-settings",
            "https://learn.microsoft.com/en-us/entra/id-governance/privileged-identity-management/pim-how-to-activate-role",
            "https://learn.microsoft.com/en-us/entra/id-governance/privileged-identity-management/pim-configure-azure-ad-roles",
        ],
        "security_notes": "Never activate a PIM role without justification, ticket reference, and MFA confirmation. An agent cannot activate another user's PIM role on their behalf — only the eligible principal may submit. Requires Entra ID P2 or equivalent license.",
        "permissions_body": textwrap.dedent("""\
            # Least-privilege RBAC guidance for PIM JIT operations

            ## Identity model

            PIM JIT is itself the least-privilege mechanism. The operator holds only an *eligible assignment*
            — not an active one. Activation is time-bounded, MFA-gated, and audit-logged natively.

            Preferred order:
            1. Entra ID PIM eligible assignment (not standing active)
            2. Time-bound maximum activation duration: 1–4 hours for break-glass, 8 hours maximum
            3. Require approval for roles with management-group or subscription scope
            4. Require justification and ticket reference for all activations

            ## Custom role to read eligible assignments and submit own activation

            ```json
            {
              "Name": "PIM JIT Activation Operator",
              "IsCustom": true,
              "Description": "Read PIM eligible assignments and submit own activation requests.",
              "Actions": [
                "Microsoft.Authorization/roleEligibilitySchedules/read",
                "Microsoft.Authorization/roleEligibilityScheduleRequests/read",
                "Microsoft.Authorization/roleAssignmentSchedules/read",
                "Microsoft.Authorization/roleAssignmentScheduleRequests/write",
                "Microsoft.Authorization/roleAssignments/read"
              ],
              "NotActions": [],
              "AssignableScopes": [
                "/subscriptions/<SUBSCRIPTION_ID>"
              ]
            }
            ```

            Note: `roleAssignmentScheduleRequests/write` only allows a principal to activate their *own*
            eligible assignment. It does not allow activating another user's role.

            ## Recommended PIM role settings (configure in Entra portal or Graph API)

            - Maximum activation duration: 8 hours
            - Require MFA on activation: **Yes**
            - Require justification: **Yes**
            - Require ticket information: **Yes** (link to change management system)
            - Require approval for: Owner, User Access Administrator, Global Administrator
            - Notification on activation: send to security team DL

            ## Graceful degradation (tenants without P2 license)

            Without PIM, use Conditional Access + Azure AD Group membership with time-bounded
            group assignment via Access Packages (Entra ID Governance) as the nearest equivalent.

            ## Do not assign

            - Standing `Owner` at subscription scope
            - Standing `User Access Administrator` (allows arbitrary role assignments)
            - `Microsoft.Authorization/roleAssignments/write` to non-PIM principals
        """),
        "preflight_body": textwrap.dedent("""\
            # PIM JIT Activation — Preflight Commands

            ## 1. Check eligible assignments for the current principal

            ```bash
            PRINCIPAL_OID=$(az ad signed-in-user show --query id -o tsv)
            SUB_ID=$(az account show --query id -o tsv)

            az rest \\
              --method GET \\
              --url "https://management.azure.com/subscriptions/${SUB_ID}/providers/Microsoft.Authorization/roleEligibilitySchedules?\\$filter=principalId+eq+'${PRINCIPAL_OID}'&api-version=2020-10-01" \\
              --query "value[].{role:properties.expandedProperties.roleDefinition.displayName, scope:properties.scope, status:properties.status, endTime:properties.endDateTime}"
            ```

            ## 2. Check for already-active assignments (prevent duplicate activation)

            ```bash
            az rest \\
              --method GET \\
              --url "https://management.azure.com/subscriptions/${SUB_ID}/providers/Microsoft.Authorization/roleAssignmentSchedules?\\$filter=principalId+eq+'${PRINCIPAL_OID}'&api-version=2020-10-01" \\
              --query "value[].{role:properties.expandedProperties.roleDefinition.displayName, status:properties.status, endTime:properties.endDateTime}"
            ```

            ## 3. Confirm Conditional Access and MFA status

            ```bash
            # Verify the signed-in user's MFA registration
            az rest \\
              --method GET \\
              --url "https://graph.microsoft.com/v1.0/me/authentication/methods" \\
              --resource "https://graph.microsoft.com/"
            ```

            ## 4. List pending approval requests (for approvers)

            ```bash
            az rest \\
              --method GET \\
              --url "https://management.azure.com/subscriptions/${SUB_ID}/providers/Microsoft.Authorization/roleAssignmentScheduleRequests?\\$filter=status+eq+'PendingApproval'&api-version=2020-10-01" \\
              --query "value[].{requestor:properties.expandedProperties.principal.displayName, role:properties.expandedProperties.roleDefinition.displayName, justification:properties.justification}"
            ```
        """),
        "rollback_body": textwrap.dedent("""\
            # PIM JIT Activation — Rollback Playbook

            ## Option 1: Self-deactivate an active role early

            ```bash
            SCHED_ID="<ROLE_ASSIGNMENT_SCHEDULE_ID>"
            SUB_ID=$(az account show --query id -o tsv)
            REQUEST_ID=$(uuidgen)

            az rest \\
              --method PUT \\
              --url "https://management.azure.com/subscriptions/${SUB_ID}/providers/Microsoft.Authorization/roleAssignmentScheduleRequests/${REQUEST_ID}?api-version=2020-10-01" \\
              --body "{
                \\"properties\\": {
                  \\"requestType\\": \\"SelfDeactivate\\",
                  \\"linkedRoleEligibilityScheduleId\\": \\"${SCHED_ID}\\",
                  \\"scheduleInfo\\": {
                    \\"expiration\\": { \\"type\\": \\"AfterDuration\\", \\"duration\\": \\"PT0S\\" }
                  }
                }
              }"
            ```

            ## Option 2: Cancel a pending activation request (before approval)

            ```bash
            az rest \\
              --method DELETE \\
              --url "https://management.azure.com/subscriptions/${SUB_ID}/providers/Microsoft.Authorization/roleAssignmentScheduleRequests/<REQUEST_ID>?api-version=2020-10-01"
            ```

            ## Option 3: Deny a pending approval request (approver action)

            ```bash
            az rest \\
              --method POST \\
              --url "https://management.azure.com/providers/Microsoft.Authorization/roleAssignmentApprovals/<APPROVAL_ID>/stages/<STAGE_ID>?api-version=2021-01-01-preview" \\
              --body "{\\"reviewResult\\": \\"Deny\\", \\"justification\\": \\"<REASON>\\"}"
            ```

            ## Verify deactivation

            ```bash
            az rest \\
              --method GET \\
              --url "https://management.azure.com/subscriptions/${SUB_ID}/providers/Microsoft.Authorization/roleAssignmentSchedules?\\$filter=principalId+eq+'${PRINCIPAL_OID}'&api-version=2020-10-01" \\
              --query "value[?properties.status=='Active'].{role:properties.expandedProperties.roleDefinition.displayName}"
            ```
        """),
    },
    {
        "id": "azure-live-aks-rollout-guard",
        "name": "Azure Live AKS Rollout Guard",
        "summary": "Guard AKS deployment rollouts with PDB audit, maxUnavailable and surge check, and explicit pause-before-proceed or undo gate before advancing.",
        "focus": "Guard AKS deployment rollouts by auditing PodDisruptionBudgets, rolling-update strategy, and replica health, then gating kubectl rollout advance or undo with explicit approval.",
        "codex_role": "aks-rollout live operator",
        "skill_desc": "Guard live AKS deployment rollouts with PDB audit, maxUnavailable/surge validation, rollout pause/undo gates, and post-rollout health verification.",
        "skill_when": [
            "a Kubernetes deployment rollout must proceed against a live AKS cluster",
            "a rollout is paused mid-flight and an operator must decide to resume or undo",
            "PDB violations or replica health issues are blocking a rollout and resolution is needed",
        ],
        "response_shape": [
            "AKS cluster identity confirmation (az aks show evidence)",
            "Current rollout status and replica health (kubectl rollout status)",
            "PodDisruptionBudget audit and rolling-update strategy review",
            "Approval status for advance, pause, or undo",
            "Proposed or executed kubectl rollout action",
            "Rollback posture (revision history and undo target)",
            "Post-rollout pod health verification and open risks",
        ],
        "official_docs": [
            "https://learn.microsoft.com/en-us/azure/aks/operator-best-practices-cluster-security",
            "https://learn.microsoft.com/en-us/azure/aks/concepts-clusters-workloads",
            "https://kubernetes.io/docs/concepts/workloads/controllers/deployment/#rolling-update-deployment",
            "https://kubernetes.io/docs/tasks/run-application/configure-pdb/",
        ],
        "security_notes": "Never advance an AKS rollout without PDB audit and replica health check. kubectl rollout undo is safe but must be confirmed before execution to avoid double-rollback churn.",
        "permissions_body": textwrap.dedent("""\
            # Least-privilege RBAC guidance for AKS rollouts

            ## Azure RBAC (control plane — getting credentials)

            ```json
            {
              "Name": "AKS Rollout Guard",
              "IsCustom": true,
              "Description": "Read AKS cluster state and fetch user-level kubeconfig. No cluster admin rights.",
              "Actions": [
                "Microsoft.ContainerService/managedClusters/read",
                "Microsoft.ContainerService/managedClusters/listClusterUserCredential/action"
              ],
              "NotActions": [
                "Microsoft.ContainerService/managedClusters/delete",
                "Microsoft.ContainerService/managedClusters/agentPools/write"
              ],
              "AssignableScopes": [
                "/subscriptions/<SUBSCRIPTION_ID>/resourceGroups/<TARGET_RG>/providers/Microsoft.ContainerService/managedClusters/<CLUSTER_NAME>"
              ]
            }
            ```

            Note: `listClusterUserCredential` gives a user-level kubeconfig. What that user can do
            *inside* the cluster is governed by AKS-integrated Entra ID RBAC, not this custom role.

            ## Kubernetes RBAC (data plane — inside the cluster)

            Bind the operator's Entra ID identity to a namespace-scoped Role:

            ```yaml
            apiVersion: rbac.authorization.k8s.io/v1
            kind: Role
            metadata:
              name: rollout-guard
              namespace: <NAMESPACE>
            rules:
            - apiGroups: ["apps"]
              resources: ["deployments", "replicasets"]
              verbs: ["get", "list", "watch", "patch", "update"]
            - apiGroups: [""]
              resources: ["pods", "pods/log"]
              verbs: ["get", "list", "watch"]
            - apiGroups: ["policy"]
              resources: ["poddisruptionbudgets"]
              verbs: ["get", "list"]
            ```

            ## Do not assign

            - `Azure Kubernetes Service Cluster Admin Role` (full cluster admin kubeconfig)
            - `cluster-admin` ClusterRoleBinding in Kubernetes
            - `Microsoft.ContainerService/managedClusters/agentPools/delete`
        """),
        "preflight_body": textwrap.dedent("""\
            # AKS Rollout — Preflight Commands

            ## 1. Confirm cluster identity and version

            ```bash
            az aks show \\
              --resource-group <TARGET_RG> \\
              --name <CLUSTER_NAME> \\
              --query "{k8sVersion:kubernetesVersion, state:provisioningState, fqdn:fqdn}"
            ```

            ## 2. Fetch user-level kubeconfig

            ```bash
            az aks get-credentials \\
              --resource-group <TARGET_RG> \\
              --name <CLUSTER_NAME> \\
              --overwrite-existing
            kubectl config current-context
            ```

            ## 3. Current rollout status (before apply)

            ```bash
            kubectl rollout status deployment/<DEPLOY_NAME> -n <NAMESPACE> --timeout=30s || true
            ```

            ## 4. Audit PodDisruptionBudget

            ```bash
            kubectl get pdb -n <NAMESPACE> -o wide
            ```

            Fail-fast if any PDB has `ALLOWED DISRUPTIONS = 0` and the rollout requires restarts.

            ## 5. Audit rolling-update strategy

            ```bash
            kubectl describe deployment <DEPLOY_NAME> -n <NAMESPACE> \\
              | grep -A 5 "RollingUpdateStrategy"
            ```

            ## 6. Check unhealthy pods before advancing

            ```bash
            kubectl get pods -n <NAMESPACE> -l app=<APP_LABEL> \\
              --field-selector="status.phase!=Running" -o wide
            ```
        """),
        "rollback_body": textwrap.dedent("""\
            # AKS Rollout — Rollback Playbook

            ## Option 1: Immediate undo (reverts to previous ReplicaSet)

            ```bash
            kubectl rollout undo deployment/<DEPLOY_NAME> -n <NAMESPACE>
            kubectl rollout status deployment/<DEPLOY_NAME> -n <NAMESPACE>
            ```

            ## Option 2: Undo to a specific revision

            ```bash
            # List revision history
            kubectl rollout history deployment/<DEPLOY_NAME> -n <NAMESPACE>

            # Undo to specific revision
            kubectl rollout undo deployment/<DEPLOY_NAME> \\
              --to-revision=<REVISION_NUMBER> \\
              -n <NAMESPACE>
            ```

            ## Option 3: Pause a stuck rollout mid-flight

            ```bash
            kubectl rollout pause deployment/<DEPLOY_NAME> -n <NAMESPACE>
            # Inspect, patch if needed, then resume or undo
            kubectl rollout resume deployment/<DEPLOY_NAME> -n <NAMESPACE>
            ```

            ## Verify rollback completed

            ```bash
            kubectl rollout status deployment/<DEPLOY_NAME> -n <NAMESPACE>
            kubectl get pods -n <NAMESPACE> -l app=<APP_LABEL>
            kubectl top pods -n <NAMESPACE>
            ```
        """),
    },
    {
        "id": "azure-live-app-service-slot-swap-guard",
        "name": "Azure Live App Service Slot Swap Guard",
        "summary": "Guard App Service slot swaps by auditing sticky settings, warmup probe readiness, and swap-with-preview evidence before final swap commit.",
        "focus": "Guard App Service production slot swaps by auditing sticky app settings, warmup probe readiness, and swap-with-preview staging evidence before final swap commit.",
        "codex_role": "app-service-slot-swap live operator",
        "skill_desc": "Guard live App Service slot swaps with sticky-settings audit, warmup probe verification, swap-with-preview staging, and instant rollback posture.",
        "skill_when": [
            "an App Service slot swap to production must be staged and committed against a live environment",
            "sticky settings or connection strings differ between slots and the operator must audit before swap",
            "a swap-with-preview is in progress and the operator must decide to complete or reset",
        ],
        "response_shape": [
            "App Service identity and slot inventory (az webapp deployment slot list)",
            "Sticky settings audit — differences between staging and production",
            "Warmup probe and startup health evidence",
            "Swap-with-preview staging confirmation",
            "Approval status for final swap commit",
            "Rollback posture (reset preview or re-swap back)",
            "Post-swap production health verification and open risks",
        ],
        "official_docs": [
            "https://learn.microsoft.com/en-us/azure/app-service/deploy-staging-slots",
            "https://learn.microsoft.com/en-us/azure/app-service/deploy-best-practices",
            "https://learn.microsoft.com/en-us/azure/app-service/configure-common",
        ],
        "security_notes": "Never perform a production slot swap without sticky-settings diff audit and warmup health confirmation. A bad swap with no rollback plan can take a production app offline instantly.",
        "permissions_body": textwrap.dedent("""\
            # Least-privilege RBAC guidance for App Service slot swaps

            ## Custom role (slot swap only, one App Service)

            ```json
            {
              "Name": "App Service Slot Swap Guard",
              "IsCustom": true,
              "Description": "Read App Service slot config and perform staged swap. No write to app settings or deployment config.",
              "Actions": [
                "Microsoft.Web/sites/read",
                "Microsoft.Web/sites/slots/read",
                "Microsoft.Web/sites/slots/config/read",
                "Microsoft.Web/sites/slots/slotsswap/action",
                "Microsoft.Web/sites/slotsswap/action",
                "Microsoft.Web/sites/config/read"
              ],
              "NotActions": [
                "Microsoft.Web/sites/config/write",
                "Microsoft.Web/sites/slots/config/write",
                "Microsoft.Web/sites/delete",
                "Microsoft.Web/sites/slots/delete"
              ],
              "AssignableScopes": [
                "/subscriptions/<SUBSCRIPTION_ID>/resourceGroups/<TARGET_RG>/providers/Microsoft.Web/sites/<APP_NAME>"
              ]
            }
            ```

            ## Nearest built-in role (broader than needed — prefer the custom role above)

            `Website Contributor` includes swap rights but also allows config writes.
            Use it only when the custom role assignment scope is too difficult to manage.

            ## Do not assign

            - `Owner` on the App Service — allows deletion
            - `Microsoft.Web/sites/config/write` without change-management gate
            - `Microsoft.Web/sites/slots/delete` — slot deletion is irreversible and excluded from the swap role
            - Subscription-level `Website Contributor` for routine swap operations
        """),
        "preflight_body": textwrap.dedent("""\
            # App Service Slot Swap — Preflight Commands

            ## 1. List all slots and current states

            ```bash
            az webapp deployment slot list \\
              --name <APP_NAME> \\
              --resource-group <TARGET_RG> \\
              --query "[].{name:name, state:state, host:defaultHostName}"
            ```

            ## 2. Audit sticky app settings (settings that do NOT swap with the slot)

            ```bash
            az webapp config appsettings list \\
              --name <APP_NAME> --slot staging \\
              --resource-group <TARGET_RG> \\
              --query "[?slotSetting==true].{name:name, value:value}"

            az webapp config appsettings list \\
              --name <APP_NAME> --slot production \\
              --resource-group <TARGET_RG> \\
              --query "[?slotSetting==true].{name:name, value:value}"
            ```

            Flag any mismatch in sticky connection strings before proceeding.

            ## 3. Verify warmup and startup health

            ```bash
            az webapp show \\
              --name <APP_NAME> --slot staging \\
              --resource-group <TARGET_RG> \\
              --query "{state:state, usageState:usageState, siteConfig:siteConfig.autoHealEnabled}"

            curl -I -s "https://<APP_NAME>-staging.azurewebsites.net/health" --max-time 30
            ```

            ## 4. Stage the swap-with-preview (does not complete the swap)

            ```bash
            az webapp deployment slot swap \\
              --name <APP_NAME> \\
              --resource-group <TARGET_RG> \\
              --slot staging \\
              --target-slot production \\
              --action preview
            ```

            Validate the preview URL before committing.
        """),
        "rollback_body": textwrap.dedent("""\
            # App Service Slot Swap — Rollback Playbook

            ## Option 1: Reset a swap-with-preview (safest — no prod change yet)

            ```bash
            az webapp deployment slot swap \\
              --name <APP_NAME> \\
              --resource-group <TARGET_RG> \\
              --slot staging \\
              --target-slot production \\
              --action reset
            ```

            This cancels the preview staging. No traffic was moved to the new version.

            ## Option 2: Re-swap back after a completed swap

            ```bash
            # Swap prod back to staging (restores previous production code)
            az webapp deployment slot swap \\
              --name <APP_NAME> \\
              --resource-group <TARGET_RG> \\
              --slot production \\
              --target-slot staging
            ```

            Speed: swap-back completes in seconds (no warmup required as staging was already warm).

            ## Option 3: Emergency scale-down if app is crashing post-swap

            ```bash
            az webapp stop --name <APP_NAME> --resource-group <TARGET_RG>
            # Fix the issue, then:
            az webapp start --name <APP_NAME> --resource-group <TARGET_RG>
            ```

            ## Verify production health after rollback

            ```bash
            az webapp show \\
              --name <APP_NAME> \\
              --resource-group <TARGET_RG> \\
              --query "{state:state, usageState:usageState}"

            curl -I -s "https://<APP_NAME>.azurewebsites.net/health" --max-time 30
            ```
        """),
    },
    {
        "id": "azure-live-keyvault-rotation-purge-guard",
        "name": "Azure Live Key Vault Rotation Purge Guard",
        "summary": "Guard Key Vault key and secret rotation, soft-delete enforcement, and purge-protection changes, with explicit irreversibility warning before any purge-protection enable.",
        "focus": "Guard Azure Key Vault key and secret rotation operations and purge-protection enablement, surfacing the irreversible nature of purge-protection and requiring explicit acknowledgment before any change.",
        "codex_role": "keyvault-rotation-purge live operator",
        "skill_desc": "Guard Key Vault key rotation, rotation policy changes, soft-delete enforcement, and purge-protection enablement with irreversibility warnings and rollback evidence.",
        "skill_when": [
            "a Key Vault key or secret rotation must be triggered or scheduled against a live vault",
            "soft-delete or purge-protection must be verified or enabled on a production vault",
            "a key or secret has been soft-deleted and recovery or permanent purge must be decided",
        ],
        "response_shape": [
            "Vault identity and current soft-delete/purge-protection state",
            "Key or secret version inventory and active version confirmation",
            "Current rotation policy audit",
            "Irreversibility warning for purge-protection (if enabling)",
            "Approval status for rotation or protection change",
            "Proposed or executed Key Vault action",
            "Post-action key version verification and open risks (unrecoverable scenarios listed explicitly)",
        ],
        "official_docs": [
            "https://learn.microsoft.com/en-us/azure/key-vault/general/key-vault-recovery",
            "https://learn.microsoft.com/en-us/azure/key-vault/keys/about-keys-details",
            "https://learn.microsoft.com/en-us/azure/key-vault/keys/how-to-configure-key-rotation",
            "https://learn.microsoft.com/en-us/azure/key-vault/general/best-practices",
        ],
        "security_notes": "Purge-protection enable is irreversible. Soft-deleted keys can be recovered within the retention window. HSM-backed hard-purged keys cannot be recovered. Never grant purge rights to routine rotation operators.",
        "permissions_body": textwrap.dedent("""\
            # Least-privilege RBAC guidance for Key Vault rotation and purge

            ## Rotation operator role (no delete, no purge)

            ```json
            {
              "Name": "Key Vault Rotation Guard",
              "IsCustom": true,
              "Description": "Rotate keys and update rotation policies. Cannot delete or purge keys/secrets/certificates. Cannot purge the vault itself. Cannot disable soft-delete.",
              "Actions": [
                "Microsoft.KeyVault/vaults/read",
                "Microsoft.KeyVault/vaults/keys/read",
                "Microsoft.KeyVault/vaults/secrets/read"
              ],
              "NotActions": [
                "Microsoft.KeyVault/vaults/purge/action",
                "Microsoft.KeyVault/vaults/delete",
                "Microsoft.KeyVault/vaults/write",
                "Microsoft.KeyVault/vaults/accessPolicies/write"
              ],
              "DataActions": [
                "Microsoft.KeyVault/vaults/keys/read",
                "Microsoft.KeyVault/vaults/keys/rotate/action",
                "Microsoft.KeyVault/vaults/keys/rotationpolicy/read",
                "Microsoft.KeyVault/vaults/keys/rotationpolicy/write",
                "Microsoft.KeyVault/vaults/secrets/getSecret/action"
              ],
              "NotDataActions": [
                "Microsoft.KeyVault/vaults/keys/delete",
                "Microsoft.KeyVault/vaults/keys/purge/action",
                "Microsoft.KeyVault/vaults/secrets/delete",
                "Microsoft.KeyVault/vaults/secrets/purge/action",
                "Microsoft.KeyVault/vaults/certificates/delete",
                "Microsoft.KeyVault/vaults/certificates/purge/action"
              ],
              "AssignableScopes": [
                "/subscriptions/<SUBSCRIPTION_ID>/resourceGroups/<TARGET_RG>/providers/Microsoft.KeyVault/vaults/<VAULT_NAME>"
              ]
            }
            ```

            **Action vs DataAction distinction (security-critical)**:
            `Microsoft.KeyVault/vaults/purge/action` is a **control-plane Action** that purges
            the soft-deleted **vault** itself (irreversible). It is **not** a DataAction and is
            not blocked by `NotDataActions`. It must be in `NotActions`. Certificate operations
            exist on both planes; this role blocks both. Do not assume `NotDataActions` covers
            all destructive Key Vault paths.

            Nearest built-in roles: `Key Vault Crypto Officer` (for keys), `Key Vault Secrets Officer` (for secrets).
            Both include delete — prefer the custom role above for rotation-only scenarios.

            ## Purge-protection enablement (separate, highly privileged operation)

            Requires: `Microsoft.KeyVault/vaults/write` on the vault resource.
            Assign via PIM with justification and at most 1-hour activation window.

            **IRREVERSIBILITY WARNING**: Once `enablePurgeProtection: true` is set on a vault,
            it cannot be unset. All soft-deleted objects in that vault are protected from permanent deletion
            until the soft-delete retention period (7–90 days) expires. This is a one-way door.

            ## Do not assign

            - `Key Vault Administrator` standing (includes purge rights)
            - `Microsoft.KeyVault/vaults/purge/action` to rotation operators
            - `Microsoft.KeyVault/vaults/accessPolicies/write` to non-admins (legacy access policy model)
        """),
        "preflight_body": textwrap.dedent("""\
            # Key Vault Rotation & Purge — Preflight Commands

            ## 1. Check vault recovery state

            ```bash
            az keyvault show \\
              --name <VAULT_NAME> \\
              --query "{softDeleteEnabled:properties.enableSoftDelete, purgeProtectionEnabled:properties.enablePurgeProtection, retentionDays:properties.softDeleteRetentionInDays, sku:sku.name}"
            ```

            **STOP** if `purgeProtectionEnabled` is `null` or `false` and you are about to enable it.
            Enabling purge-protection is **irreversible**. Get explicit written approval.

            ## 2. List key versions and active version

            ```bash
            az keyvault key list-versions \\
              --vault-name <VAULT_NAME> \\
              --name <KEY_NAME> \\
              --query "[].{version:kid, enabled:attributes.enabled, expires:attributes.expires, created:attributes.created}" \\
              --output table
            ```

            ## 3. Show current rotation policy

            ```bash
            az keyvault key rotation-policy show \\
              --vault-name <VAULT_NAME> \\
              --name <KEY_NAME>
            ```

            ## 4. List secrets with expiry audit

            ```bash
            az keyvault secret list \\
              --vault-name <VAULT_NAME> \\
              --query "[].{name:name, expires:attributes.expires, enabled:attributes.enabled}" \\
              --output table
            ```

            ## 5. Check for soft-deleted objects awaiting recovery or purge decision

            ```bash
            az keyvault key list-deleted --vault-name <VAULT_NAME> --output table
            az keyvault secret list-deleted --vault-name <VAULT_NAME> --output table
            ```
        """),
        "rollback_body": textwrap.dedent("""\
            # Key Vault Rotation & Purge — Rollback Playbook

            ## Recover a soft-deleted key (within retention window)

            ```bash
            az keyvault key recover \\
              --vault-name <VAULT_NAME> \\
              --name <KEY_NAME>
            ```

            ## Recover a soft-deleted secret

            ```bash
            az keyvault secret recover \\
              --vault-name <VAULT_NAME> \\
              --name <SECRET_NAME>
            ```

            ## Re-enable a previous key version (roll back to prior version as active)

            ```bash
            az keyvault key set-attributes \\
              --vault-name <VAULT_NAME> \\
              --name <KEY_NAME> \\
              --version <PREVIOUS_VERSION_ID> \\
              --enabled true
            ```

            ## Restore rotation policy to previous settings

            ```bash
            az keyvault key rotation-policy update \\
              --vault-name <VAULT_NAME> \\
              --name <KEY_NAME> \\
              --value @rotation-policy-backup.json
            ```

            ## CANNOT ROLL BACK

            - **Purge-protection enable**: once set, cannot be disabled on the vault.
            - **Hard-purged key**: permanently destroyed. Data encrypted exclusively by this
              key version is unrecoverable. Escalate to incident response immediately.
            - **Expired soft-delete retention + no purge-protection**: objects auto-purged
              after retention window expires with no recovery option.
        """),
    },
    {
        "id": "azure-live-cost-budget-action-guard",
        "name": "Azure Live Cost Budget Action Guard",
        "summary": "Gate subscription and management-group budget action changes and GPU or HPC SKU scale-up against approved spend thresholds before any cost-impacting mutation.",
        "focus": "Gate Azure subscription and management-group budget action changes and GPU/HPC SKU scale-up (NDv5, NCv3, H-series) against approved spend thresholds before any cost-impacting mutation.",
        "codex_role": "cost-budget-action live operator",
        "skill_desc": "Gate Azure budget action changes and GPU/HPC SKU provisioning against approved spend limits, with quota audits and emergency spend-stop playbooks.",
        "skill_when": [
            "a cost budget action threshold or notification must be modified for a subscription or management group",
            "a GPU or HPC VM SKU scale-up is requested and spend-limit approval is required",
            "a runaway cost event is detected and emergency quota reduction or VM deallocation is needed",
        ],
        "response_shape": [
            "Active subscription and budget inventory (az consumption budget list)",
            "Current spend vs threshold and forecast (actual vs budget amount)",
            "GPU/HPC quota usage in target region",
            "Approval status for budget change or SKU scale-up",
            "Proposed or executed cost-governance action",
            "Rollback posture (restore previous threshold, quota reduction)",
            "Post-change budget alert and monitoring confirmation",
        ],
        "official_docs": [
            "https://learn.microsoft.com/en-us/azure/cost-management-billing/costs/tutorial-acm-create-budgets",
            "https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/azure-subscription-service-limits",
            "https://learn.microsoft.com/en-us/azure/quotas/quickstart-increase-quota-portal",
            "https://learn.microsoft.com/en-us/azure/cost-management-billing/finops/overview-finops",
        ],
        "security_notes": "GPU/HPC SKUs (NDv5, H100, A100) can generate $50K+ daily costs. Never approve quota increases or budget threshold raises without explicit spend-approval sign-off from a financial authority.",
        "permissions_body": textwrap.dedent("""\
            # Least-privilege RBAC guidance for cost budget and GPU guard

            ## Custom role (budget read/write + quota read, NO VM creation)

            ```json
            {
              "Name": "Cost Budget Action Guard",
              "IsCustom": true,
              "Description": "Read and modify subscription budgets and read compute quotas. Cannot create VMs. Cannot delete budgets.",
              "Actions": [
                "Microsoft.Consumption/budgets/read",
                "Microsoft.Consumption/budgets/write",
                "Microsoft.CostManagement/budgets/read",
                "Microsoft.CostManagement/budgets/write",
                "Microsoft.CostManagement/query/action",
                "Microsoft.Compute/locations/usages/read",
                "Microsoft.Compute/locations/vmSizes/read",
                "Microsoft.Quota/quotas/read",
                "Microsoft.Quota/usages/read"
              ],
              "NotActions": [
                "Microsoft.Compute/virtualMachines/write",
                "Microsoft.Compute/virtualMachineScaleSets/write",
                "Microsoft.Quota/quotas/write",
                "Microsoft.Consumption/budgets/delete",
                "Microsoft.CostManagement/budgets/delete"
              ],
              "AssignableScopes": [
                "/subscriptions/<SUBSCRIPTION_ID>"
              ]
            }
            ```

            VM creation is explicitly excluded. `Microsoft.Quota/quotas/write` is also excluded:
            quota increase requests carry spending risk and must go through a separate approval
            workflow (e.g., Azure Support or an IT-ops request process), not through this role.
            GPU SKU approval flows through budget-action alerts only — not through quota write.

            **Budget deletion is excluded** (`Microsoft.Consumption/budgets/delete`,
            `Microsoft.CostManagement/budgets/delete`). Deleting budgets silently removes the
            only cross-region financial guardrail and disables every threshold alert on the
            subscription. Cleanup of test or stale budgets must go through a separate
            PIM-eligible "Cost Budget Cleanup" role, never the standing operational role.

            ## Separate PIM role: Cost Budget Cleanup (eligible-only)

            ```json
            {
              "Name": "Cost Budget Cleanup (PIM-eligible)",
              "IsCustom": true,
              "Description": "PIM-only role for deleting stale or test budgets. Eligible-only. Maximum 2-hour activation. MFA + justification required.",
              "Actions": [
                "Microsoft.Consumption/budgets/read",
                "Microsoft.Consumption/budgets/delete",
                "Microsoft.CostManagement/budgets/read",
                "Microsoft.CostManagement/budgets/delete"
              ],
              "AssignableScopes": [
                "/subscriptions/<SUBSCRIPTION_ID>"
              ]
            }
            ```

            Configure as PIM-eligible only (never standing active), MFA-gated, time-bounded.

            ## Azure Policy guardrail (deploy alongside the custom role)

            Deny GPU VM SKU provisioning without an approved budget tag:

            ```json
            {
              "if": {
                "allOf": [
                  {"field": "type", "equals": "Microsoft.Compute/virtualMachines"},
                  {"field": "Microsoft.Compute/virtualMachines/sku.name", "in": [
                    "Standard_ND96asr_v4", "Standard_NC24rs_v3", "Standard_ND40rs_v2",
                    "Standard_HB120rs_v3", "Standard_HB176rs_v4"
                  ]},
                  {"field": "tags.BudgetApproval", "exists": "false"}
                ]
              },
              "then": {"effect": "Deny"}
            }
            ```

            ## Do not assign

            - `Cost Management Contributor` at management-group scope (modifies all child subscriptions)
            - `Billing Account Contributor`
            - `Microsoft.Compute/virtualMachines/write` to this role
        """),
        "preflight_body": textwrap.dedent("""\
            # Cost Budget Action — Preflight Commands

            ## 1. List all budgets and current spend

            ```bash
            az consumption budget list \\
              --query "[].{name:name, amount:amount, currentSpend:currentSpend.amount, forecastSpend:forecastSpend.amount, timeGrain:timeGrain}" \\
              --output table
            ```

            ## 2. Check GPU/HPC quota usage in target region

            ```bash
            az vm list-usage \\
              --location <REGION> \\
              --query "[?contains(name.value, 'GPU') || contains(name.value, 'NC') || contains(name.value, 'ND') || contains(name.value, 'NV')].{name:name.localizedValue, used:currentValue, limit:limit}" \\
              --output table
            ```

            ## 3. Inventory running GPU/HPC VMs across subscription

            ```bash
            az vm list \\
              --query "[?contains(storageProfile.imageReference.sku, 'gpu') || starts_with(hardwareProfile.vmSize, 'Standard_NC') || starts_with(hardwareProfile.vmSize, 'Standard_ND')].{name:name, size:hardwareProfile.vmSize, rg:resourceGroup, state:powerState}" \\
              --show-details \\
              --output table
            ```

            ## 4. Show active budget alert thresholds

            ```bash
            az consumption budget show \\
              --budget-name <BUDGET_NAME> \\
              --query "{notifications:notifications, amount:amount, filter:filter, startDate:timePeriod.startDate}"
            ```

            ## 5. Check quota request history

            ```bash
            az quota request status list \\
              --scope "/subscriptions/<SUBSCRIPTION_ID>/providers/Microsoft.Compute/locations/<REGION>" \\
              --query "[].{name:name, status:properties.provisioningState, value:properties.value.limit}" \\
              --output table
            ```
        """),
        "rollback_body": textwrap.dedent("""\
            # Cost Budget Action — Rollback Playbook

            ## Restore a lowered budget to its previous threshold

            ```bash
            az consumption budget create \\
              --budget-name <BUDGET_NAME> \\
              --amount <PREVIOUS_AMOUNT> \\
              --time-grain Monthly \\
              --start-date <START_DATE> \\
              --end-date <END_DATE>
            ```

            ## Emergency: deallocate a runaway GPU VM (requires VM operator rights — escalate if needed)

            ```bash
            az vm deallocate \\
              --resource-group <RG> \\
              --name <VM_NAME> \\
              --no-wait
            # Verify deallocation
            az vm show --resource-group <RG> --name <VM_NAME> --query "powerState" -d
            ```

            ## Scale VMSS to zero during a runaway cost event

            ```bash
            az vmss scale \\
              --resource-group <RG> \\
              --name <VMSS_NAME> \\
              --new-capacity 0
            ```

            ## Revert a quota increase (reduce back to previous limit)

            ```bash
            az quota update \\
              --resource-name "cores" \\
              --scope "/subscriptions/<SUBSCRIPTION_ID>/providers/Microsoft.Compute/locations/<REGION>" \\
              --limit-object value=<PREVIOUS_LIMIT> value-type=Individual
            ```

            ## Verify budget alert is re-active

            ```bash
            az consumption budget show \\
              --budget-name <BUDGET_NAME> \\
              --query "{amount:amount, currentSpend:currentSpend.amount, notifications:notifications}"
            ```
        """),
    },
]


HARNESS_TEMPLATE_COPILOT = """\
---
description: "{summary}"
name: "{name}"
tools:
  - "read"
  - "search"
  - "search/codebase"
  - "web/githubRepo"
  - "web/fetch"
  - "read/problems"
  - "execute/runInTerminal"
  - "execute/getTerminalOutput"
  - "read/terminalLastCommand"
  - "read/terminalSelection"
disable-model-invocation: false
user-invocable: true
---

{body}
"""


def harness_body(agent):
    lines = [
        f"# {agent['name']}",
        "",
        f"Use this canonical agent only for `{agent['id']}` work.",
        "",
        "## Required Skill",
        "",
        "Before answering, read and follow:",
        "",
        f"- `skills/azure/{agent['id']}/SKILL.md`",
        "",
        f"Load files under `skills/azure/{agent['id']}/references/` only when the task needs that reference. Do not dump reference text into the response.",
        "",
        "## Focus",
        "",
        agent["focus"],
        "",
        "## Operating Rules",
        "",
        f"- Load and follow the bound Azure skill first; do not drift into generic cloud advice.",
        "- This role is for repos or sessions that may be connected to live Azure credentials, CLI profiles, or real environments.",
        "- Before any live Azure mutation, confirm subscription, resource group, active principal, exact target resource, expected impact, and explicit human approval.",
        "- Prefer what-if, dry-run, preview, describe, status, plan, and rollback evidence before mutation.",
        "- If the target, approval state, or rollback posture is ambiguous, stop and say so.",
        "- Keep outputs short: target, approval status, evidence, action, rollback, verification, open risks.",
        "- Never ask for secrets, credentials, access tokens, private keys, or raw environment dumps unless already sanitized and required.",
        "",
        "## Response Shape",
        "",
    ]
    for i, step in enumerate(agent["response_shape"], 1):
        lines.append(f"{i}. {step}")
    return "\n".join(lines)


def write(path, content):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        f.write(content)
    print(f"  wrote {path.replace(ROOT+'/', '')}")


def agent_md(agent):
    body = harness_body(agent)
    return f"""---
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
---

# {agent['name']}

> Agent for `{agent['id']}`. {agent['summary']}

## Harness Variants

- `harnesses/codex.toml` — Codex native agent configuration.
- `harnesses/copilot.agent.md` — GitHub Copilot / VS Code custom agent definition.
- `harnesses/claude-code.agent.md` — Claude Code Markdown-family adapter.
- `harnesses/cursor.agent.md` — Cursor Markdown-family adapter.
- `harnesses/gemini.agent.md` — Gemini CLI Markdown-family adapter.
- `harnesses/kiro-ide.agent.md` — Kiro IDE Markdown-family adapter.
- `harnesses/kiro-cli.agent.json` — Kiro CLI JSON adapter.

## Canonical Contract

{body}
"""


def codex_toml(agent):
    rules = "\n".join([
        "- Load and follow the bound Azure skill first; do not drift into generic cloud advice.",
        "- This role is for repos or sessions that may be connected to live Azure credentials, CLI profiles, or real environments.",
        "- Before any live Azure mutation, confirm subscription, resource group, active principal, exact target, expected impact, and explicit human approval.",
        "- Prefer what-if, dry-run, preview, describe, status, plan, and rollback evidence before mutation.",
        "- If approval, identity, target, or rollback posture is ambiguous, stop and explain the blocker.",
        "- Never ask for secrets, credentials, access tokens, account numbers, private keys, or raw environment dumps unless already sanitized and required.",
        "- Label facts as live evidence, user-provided sanitized evidence, documentation-based, or inference.",
    ])
    return f"""name = "{agent['id']}_agent"
description = "Specialized subagent for {agent['id']}. {agent['summary']}"
model = "gpt-5.4"
model_reasoning_effort = "high"
sandbox_mode = "workspace-write"

developer_instructions = \"\"\"
Load and follow the bound `{agent['id']}` skill first. This agent exists only for that guarded live-Azure role; do not drift into generic cloud advice.

Token discipline:
- Read only SKILL.md first; load references only when the task requires them.
- Keep answers compact: target, approval status, evidence, action, rollback, verification, open risks.
- Do not paste long docs, raw tool inventories, raw credential output, or full environment dumps.

Role focus: {agent['focus']}

Safety contract:
{rules}
\"\"\"

[[skills.config]]
path = "skills/azure/{agent['id']}/SKILL.md"
enabled = true

[metadata]
author = "github: VincentChuWaiChow"
"""


def kiro_cli_json(agent):
    body = harness_body(agent)
    prompt = body.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")
    return f'{{"name": "{agent["name"]}", "description": "{agent["summary"]}", "prompt": "{prompt}"}}\n'


def metadata_json(agent):
    return json.dumps({
        "id": f"{agent['id']}-agent",
        "name": agent["name"],
        "type": "agent",
        "provider": "azure",
        "harnesses": ["codex", "copilot", "claude-code", "cursor", "gemini", "kiro"],
        "summary": agent["summary"],
        "source_type": "original",
        "official_docs": agent["official_docs"],
        "security_notes": agent["security_notes"],
        "last_verified": DATE,
        "path": f"agents/azure/{agent['id']}-agent",
        "author": "github: VincentChuWaiChow",
        "version": "0.1.0",
    }, indent=2) + "\n"


def skill_md(agent):
    when_items = "\n".join(f"- {w}" for w in agent["skill_when"])
    return f"""---
name: {agent['id']}
description: {agent['skill_desc']}
metadata:
  author: "github: VincentChuWaiChow"
  version: "0.1.0"
---

# {agent['name']}

## Purpose

Act as the guarded live Azure operator for {agent['id']} work. Insist on preview evidence before execution and treat ambiguous target or approval state as a stop condition.

## When to use

Use this skill when:

{when_items}

## Lean operating rules

- Prefer Azure CLI (`az`) official documentation when available; fall back to Microsoft Learn docs and sanitized user evidence.
- Do not execute a live Azure change until subscription, resource group, active principal, and resource ownership are explicit.
- Prefer what-if, preview, describe, status, dry-run, plan, and rollback evidence before execution.
- If the request skips preview or rollback design, push back.
- Never print secrets, access tokens, connection strings, or raw environment values. Summarize sanitized evidence only.
- Load references only when needed.

## References

Load these only when needed:

- [Preflight commands](references/preflight-commands.md) — CLI commands to run before any mutation.
- [Rollback playbook](references/rollback-playbook.md) — concrete rollback steps for this service.
- [Permission model](references/permission-model.md) — RBAC role definitions and PIM guidance.
- [Official sources](references/official-sources.md) — authoritative Azure documentation links.

## Response minimum

Return, at minimum:

- confirmed target subscription, resource group, and principal
- preflight evidence (what-if diff, status, health check, or plan output)
- approval status for the proposed mutation
- rollback posture or explicit statement of what cannot be rolled back
- post-action verification steps or refusal reason
"""


def skill_metadata_json(agent):
    return json.dumps({
        "id": agent["id"],
        "name": agent["name"],
        "type": "skill",
        "provider": "azure",
        "harnesses": ["codex", "claude-code", "cursor", "gemini", "kiro", "other"],
        "summary": agent["skill_desc"],
        "source_type": "original",
        "official_docs": agent["official_docs"],
        "security_notes": agent["security_notes"],
        "last_verified": DATE,
        "path": f"skills/azure/{agent['id']}",
        "author": "github: VincentChuWaiChow",
        "version": "0.1.0",
    }, indent=2) + "\n"


def build():
    for ag in AGENTS:
        aid = ag["id"]
        adir = os.path.join(ROOT, "agents", "azure", f"{aid}-agent")
        hdir = os.path.join(adir, "harnesses")
        sdir = os.path.join(ROOT, "skills", "azure", aid)
        rdir = os.path.join(sdir, "references")
        os.makedirs(hdir, exist_ok=True)
        os.makedirs(rdir, exist_ok=True)

        print(f"\n[{aid}]")
        body = harness_body(ag)

        write(os.path.join(adir, "AGENT.md"), agent_md(ag))
        write(os.path.join(adir, "PERMISSIONS.md"), f"# Permissions: {ag['name']}\n\n{ag['permissions_body']}\n")
        write(os.path.join(adir, "PREFLIGHT.md"), ag["preflight_body"])
        write(os.path.join(adir, "ROLLBACK.md"), ag["rollback_body"])
        write(os.path.join(adir, "metadata.json"), metadata_json(ag))

        write(os.path.join(hdir, "claude-code.agent.md"), f"---\nname: \"{ag['name']}\"\ndescription: \"{ag['summary']}\"\n---\n\n{body}\n")
        write(os.path.join(hdir, "cursor.agent.md"), f"---\nname: \"{ag['name']}\"\ndescription: \"{ag['summary']}\"\n---\n\n{body}\n")
        # Gemini CLI requires a slug `name`; see gemini_harness in gen_python_agents.py.
        write(os.path.join(hdir, "gemini.agent.md"), f"---\nname: \"{aid}-agent\"\ndisplay_name: \"{ag['name']}\"\ndescription: \"{ag['summary']}\"\n---\n\n{body}\n")
        write(os.path.join(hdir, "kiro-ide.agent.md"), f"---\nname: \"{ag['name']}\"\ndescription: \"{ag['summary']}\"\n---\n\n{body}\n")
        write(os.path.join(hdir, "copilot.agent.md"), HARNESS_TEMPLATE_COPILOT.format(name=ag["name"], summary=ag["summary"], body=body))
        write(os.path.join(hdir, "codex.toml"), codex_toml(ag))
        write(os.path.join(hdir, "kiro-cli.agent.json"), kiro_cli_json(ag))

        write(os.path.join(sdir, "SKILL.md"), skill_md(ag))
        write(os.path.join(sdir, "metadata.json"), skill_metadata_json(ag))
        # Stub references (operators load full content from PREFLIGHT/ROLLBACK/PERMISSIONS)
        write(os.path.join(rdir, "preflight-commands.md"), f"# Preflight Commands\n\nSee `../../PREFLIGHT.md` in the agent directory for executable commands.\n")
        write(os.path.join(rdir, "rollback-playbook.md"), f"# Rollback Playbook\n\nSee `../../ROLLBACK.md` in the agent directory for the full rollback playbook.\n")
        write(os.path.join(rdir, "permission-model.md"), f"# Permission Model\n\nSee `../../PERMISSIONS.md` in the agent directory for RBAC role definitions and PIM guidance.\n")
        write(os.path.join(rdir, "official-sources.md"), f"# Official Sources\n\n" + "\n".join(f"- {u}" for u in ag["official_docs"]) + "\n")

    print("\nAzure live-guard agents + skills generated.")


if __name__ == "__main__":
    build()
