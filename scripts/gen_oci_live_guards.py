#!/usr/bin/env python3
"""Generator: 6 OCI live-guard agents + 6 paired skills."""
import os, json, textwrap

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATE = "2026-04-30"

AGENTS = [
    {
        "id": "oci-live-resource-manager-stack-guard",
        "name": "OCI Live Resource Manager Stack Guard",
        "summary": "Guard OCI Resource Manager plan, apply, and destroy jobs with drift detection evidence, state-version audit, and stack-lock awareness before any mutation.",
        "focus": "Guard OCI Resource Manager stack plan/apply/destroy jobs by enforcing drift detection evidence, plan-job output review, state-version audit, and explicit approval before any apply or destroy.",
        "codex_role": "resource-manager-stack live operator",
        "skill_desc": "Guard OCI Resource Manager stack plan, apply, and destroy jobs with drift detection, state-version rollback, stack auto-lock awareness, and approval gates.",
        "skill_when": [
            "an OCI Resource Manager stack apply or destroy job must be run against a live environment",
            "drift has been detected on a stack and resolution requires an apply job with human approval",
            "a Resource Manager stack state must be inspected, imported, or rolled back after a partial apply",
        ],
        "response_shape": [
            "OCI tenancy and compartment confirmation (oci iam region list + stack OCID evidence)",
            "Drift detection output (oci resource-manager stack detect-drift result)",
            "Plan job output review (create-plan-job logs before approve)",
            "Stack auto-lock status (only one job at a time — Resource Manager enforces this)",
            "Approval status for apply or destroy",
            "Proposed or executed Resource Manager job action",
            "Post-job state verification and open risks (state-version rollback path if apply fails)",
        ],
        "official_docs": [
            "https://docs.oracle.com/en-us/iaas/Content/ResourceManager/Concepts/resourcemanager.htm",
            "https://docs.oracle.com/en-us/iaas/Content/ResourceManager/Tasks/detect-drift.htm",
            "https://docs.oracle.com/en-us/iaas/Content/ResourceManager/Tasks/create-job-lock-file.htm",
            "https://docs.oracle.com/en-us/iaas/Content/ResourceManager/home.htm",
        ],
        "security_notes": "OCI Resource Manager auto-locks a stack state during job execution. Never approve an apply or destroy job without a plan-job output review and drift detection evidence. Repo write access does not authorize live OCI infrastructure mutations.",
        "permissions_body": textwrap.dedent("""\
            # OCI IAM policy guidance for Resource Manager stack guard

            ## Identity model preference

            1. Named group in target compartment — never `any-user` or `any-group`
            2. Dynamic group matching the CI/CD runner instance by compartment and tag
            3. Short-lived session token via Instance Principal for automation
            4. Never grant `manage all-resources in tenancy`

            ## OCI IAM verb hierarchy reminder

            `inspect` ⊂ `read` ⊂ `use` ⊂ `manage`

            - `inspect` — list-only (no content details)
            - `read` — get + list (read details, no mutation)
            - `use` — limited mutation (no create/terminate)
            - `manage` — full CRUD (create, update, delete)

            ## Baseline read policy (auditors — no mutation rights)

            ```
            Allow group <rms-auditors> to inspect orm-stacks in compartment <prod-compartment>
            Allow group <rms-auditors> to read orm-stacks in compartment <prod-compartment>
            Allow group <rms-auditors> to inspect orm-jobs in compartment <prod-compartment>
            Allow group <rms-auditors> to read orm-jobs in compartment <prod-compartment>
            ```

            ## Plan-only policy (can create plan jobs, cannot apply or destroy)

            ```
            Allow group <rms-planners> to use orm-stacks in compartment <prod-compartment>
            Allow group <rms-planners> to use orm-jobs in compartment <prod-compartment>
            ```

            ## Full operator policy (apply + destroy — gate with approval workflow)

            ```
            Allow group <rms-operators> to manage orm-stacks in compartment <prod-compartment>
            Allow group <rms-operators> to manage orm-jobs in compartment <prod-compartment>
            ```

            ## Dynamic group for CI/CD instance principal

            ```
            Any {instance.compartment.id = '<compartment_ocid>', tag.Operations.Role.value = 'rms-runner'}

            Allow dynamic-group <rms-runners> to manage orm-stacks in compartment <prod-compartment>
            Allow dynamic-group <rms-runners> to manage orm-jobs in compartment <prod-compartment>
            ```

            ## Service-principal policies (Resource Manager service itself)

            OCI is policy-based IAM: managed services must hold explicit `Allow service ...`
            grants to act on your tenancy. Without these, stack jobs fail with `NotAuthorized`
            even when the human operator is correctly scoped.

            ```
            Allow service ResourceManager to manage orm-stacks in compartment <prod-compartment>
            Allow service ResourceManager to read secret-family in compartment <prod-compartment>
            Allow service ResourceManager to use tag-namespaces in tenancy
            ```

            Add resource-type rights for whatever the stack provisions, e.g.
            `Allow service ResourceManager to manage instance-family in compartment <X>`
            for stacks that create compute. Do not grant `manage all-resources` even to the
            service principal — scope by resource family.

            ## Do not use

            ```
            # FORBIDDEN
            Allow any-user to manage all-resources in tenancy
            Allow group <rms-operators> to manage all-resources in compartment prod
            ```

            Stack auto-lock: Resource Manager allows **only one running job at a time per stack**.
            This is platform-enforced — no additional concurrency control needed.
        """),
        "preflight_body": textwrap.dedent("""\
            # Resource Manager Stack — Preflight Commands

            ## 1. Confirm identity and region

            ```bash
            oci iam region list --output table
            oci iam user get --user-id <OPERATOR_OCID> --query 'data.name'
            ```

            ## 2. Inspect current stack state

            ```bash
            oci resource-manager stack get \\
              --stack-id <STACK_OCID> \\
              --query 'data.{state:"lifecycle-state", updated:"time-updated", terraform:"terraform-version", compartment:"compartment-id"}'
            ```

            ## 3. Detect drift (always before apply or destroy)

            ```bash
            oci resource-manager stack detect-drift \\
              --stack-id <STACK_OCID>

            # List drift details once job completes
            oci resource-manager stack list-resource-drift-details \\
              --stack-id <STACK_OCID>
            ```

            ## 4. Create a plan job and review output before any apply

            ```bash
            oci resource-manager job create-plan-job \\
              --stack-id <STACK_OCID> \\
              --display-name "preflight-plan-$(date +%Y%m%dT%H%M%S)"

            # Retrieve plan logs
            oci resource-manager job get-job-logs \\
              --job-id <PLAN_JOB_OCID> --all
            ```

            Stop and escalate if plan output shows unexpected resource deletions or replacements.

            ## 5. Verify no other job is currently running

            ```bash
            oci resource-manager job list \\
              --compartment-id <COMPARTMENT_OCID> \\
              --stack-id <STACK_OCID> \\
              --lifecycle-state IN_PROGRESS \\
              --query 'data[].{id:id, op:"operation", started:"time-created"}'
            ```
        """),
        "rollback_body": textwrap.dedent("""\
            # Resource Manager Stack — Rollback Playbook

            Resource Manager auto-locks the stack during jobs — concurrent apply/destroy is
            physically prevented. Rollback options depend on how far the failed apply progressed.

            ## Option 1: Apply previous configuration (re-upload prior config zip)

            ```bash
            oci resource-manager stack update \\
              --stack-id <STACK_OCID> \\
              --config-source-zip-file previous-config.zip

            oci resource-manager job create-apply-job \\
              --stack-id <STACK_OCID> \\
              --execution-plan-strategy FROM_PLAN_JOB_ID \\
              --execution-plan-job-id <PRIOR_PLAN_JOB_OCID> \\
              --display-name "rollback-apply-$(date +%Y%m%dT%H%M%S)"
            ```

            ## Option 2: Import a known-good Terraform state file

            ```bash
            oci resource-manager job create-import-tf-state-job \\
              --stack-id <STACK_OCID> \\
              --tf-state-base64 "$(base64 -i previous.tfstate)"
            ```

            ## Option 3: Targeted destroy of newly-created resources only

            ```bash
            oci resource-manager job create-destroy-job \\
              --stack-id <STACK_OCID> \\
              --execution-plan-strategy AUTO_APPROVED \\
              --display-name "targeted-destroy-$(date +%Y%m%dT%H%M%S)"
            ```

            Only use AUTO_APPROVED if human has already reviewed the destroy plan separately.

            ## Monitor rollback job

            ```bash
            oci resource-manager job get \\
              --job-id <JOB_OCID> \\
              --query 'data."lifecycle-state"'
            ```
        """),
    },
    {
        "id": "oci-live-iam-policy-compartment-guard",
        "name": "OCI Live IAM Policy Compartment Guard",
        "summary": "Guard OCI IAM policy changes and dynamic group mutations using verb-hierarchy audit and tag-condition review before write.",
        "focus": "Guard OCI IAM policy changes and dynamic group mutations by auditing verb-hierarchy (inspect < read < use < manage), compartment scope, and tag conditions before any policy write.",
        "codex_role": "iam-policy-compartment live operator",
        "skill_desc": "Guard OCI IAM policy writes and dynamic group changes with verb-hierarchy audit, compartment scope enforcement, anti-pattern detection (any-user/any-group), and rollback via statement restore.",
        "skill_when": [
            "an OCI IAM policy must be created or modified in a compartment or at tenancy root",
            "a dynamic group rule must be changed and blast-radius must be audited before write",
            "an IAM audit finds overly broad policies that must be narrowed with least-privilege verb selection",
        ],
        "response_shape": [
            "Compartment and tenancy identity confirmation",
            "Current policy statement inventory (oci iam policy list)",
            "Dynamic group rule audit and matching-instance check",
            "Verb-hierarchy assessment of proposed change (inspect/read/use/manage)",
            "Approval status and anti-pattern scan result (any-user/any-group flag)",
            "Proposed or executed policy write action",
            "Post-write policy verification and open risks",
        ],
        "official_docs": [
            "https://docs.oracle.com/en-us/iaas/Content/Identity/Concepts/policygetstarted.htm",
            "https://docs.oracle.com/en-us/iaas/Content/Identity/Tasks/managingdynamicgroups.htm",
            "https://docs.oracle.com/en-us/iaas/Content/Identity/Concepts/policysyntax.htm",
            "https://docs.oracle.com/en-us/iaas/Content/Identity/Reference/iampolicyreference.htm",
        ],
        "security_notes": "Any-user and any-group policies in tenancy root are the most common OCI security misconfiguration. Never approve manage-verb policies at tenancy scope without compartment scoping. Policy deletes take effect immediately with no grace period.",
        "permissions_body": textwrap.dedent("""\
            # OCI IAM policy for IAM policy compartment guard

            ## Identity model preference

            1. Named IAM-admin group scoped to an IAM-management compartment
            2. Dual-approval for tenancy-root policy changes (separate writer and approver)
            3. Never use `any-user` or `any-group` for policy management
            4. Tenancy-root policy changes require separate security-team sign-off

            ## Verb hierarchy reference

            ```
            inspect  = ListXxx APIs only. No resource content.
            read     = GetXxx + inspect. Can see resource details.
            use      = read + limited mutation (no create/terminate).
            manage   = full CRUD. Always scope to compartment, never tenancy for broad resources.
            ```

            ## Audit-only policy

            ```
            Allow group <iam-auditors> to inspect policies in tenancy
            Allow group <iam-auditors> to read policies in tenancy
            Allow group <iam-auditors> to inspect dynamic-groups in tenancy
            Allow group <iam-auditors> to read dynamic-groups in tenancy
            Allow group <iam-auditors> to inspect groups in tenancy
            Allow group <iam-auditors> to read users in tenancy
            ```

            ## Policy operator (compartment-scoped write, never tenancy root)

            ```
            Allow group <iam-operators> to manage policies in compartment <iam-compartment>
              where target.policy.name = /iam-managed-*/
            Allow group <iam-operators> to manage dynamic-groups in tenancy
              where target.dynamicGroup.name = /iam-managed-*/
            ```

            `dynamic-groups` are tenancy-scoped in OCI — they cannot be compartment-scoped.
            This is the minimum necessary `manage` at tenancy scope. The `where` name-pattern
            condition restricts which dynamic groups this role can create or modify, preventing
            privilege escalation through creation of an unrestricted dynamic group.

            **Critical syntax note**: OCI IAM uses **forward-slash regex pattern syntax** `= /pattern*/`
            for wildcard matching, **not** `= 'pattern-*'` (which is exact-string match for the
            literal `pattern-*`). Quoted-string equality in a `where` clause is a no-op security
            control if the operator can choose any name not matching the literal exact value.
            See [Oracle policy conditions docs](https://docs.oracle.com/en-us/iaas/Content/Identity/policysyntax/conditions.htm).

            ## Tag-condition for policy name pattern restriction

            ```
            Allow group <iam-operators> to manage policies in compartment <iam-compartment>
              where target.policy.name = /iam-managed-*/
            ```

            ## Tenancy-root admin (third tier — break-glass only)

            OCI policy-based IAM separates compartment-scoped operators from tenancy-root
            admins. The tenancy-root admin is a **break-glass** identity activated only for
            incidents that require touching tenancy-level policies (e.g., when an
            operator-managed policy would create a cycle or escalation path).

            ```
            Allow group <iam-tenancy-admins> to manage policies in tenancy
              where request.user.mfaTotpVerified = 'true'
            Allow group <iam-tenancy-admins> to manage groups in tenancy
              where target.group.name != 'Administrators'
            ```

            - MFA-TOTP gate enforced at policy-evaluation time (not just login).
            - Cannot modify the `Administrators` group from this role — that requires the
              bootstrap tenancy admin (no automation, no service principal).
            - Membership in `<iam-tenancy-admins>` should be empty by default; add only for
              the duration of an approved change window, then remove.

            ## Do not use

            ```
            # FORBIDDEN
            Allow any-group to manage policies in tenancy
            Allow group <iam-operators> to manage policies in tenancy
            Allow any-user to inspect all-resources in tenancy
            ```
        """),
        "preflight_body": textwrap.dedent("""\
            # IAM Policy Compartment — Preflight Commands

            ## 1. List all policies in target compartment

            ```bash
            oci iam policy list \\
              --compartment-id <COMPARTMENT_OCID> \\
              --all \\
              --query 'data[].{id:id, name:name, statements:statements}' \\
              --output json
            ```

            ## 2. Scan for any-user / any-group policies (red-flag detector)

            ```bash
            oci iam policy list \\
              --compartment-id <TENANCY_OCID> \\
              --all \\
              --query 'data[].statements[]' \\
              --output json | grep -i 'any-user\|any-group'
            ```

            Zero results expected. Any hit is a required review item before proceeding.

            ## 3. List dynamic groups and current matching rules

            ```bash
            oci iam dynamic-group list \\
              --compartment-id <TENANCY_OCID> \\
              --all \\
              --query 'data[].{name:name, rule:"matching-rule", id:id}'
            ```

            ## 4. Review the specific policy to be changed

            ```bash
            oci iam policy get \\
              --policy-id <POLICY_OCID> \\
              --query 'data.{name:name, statements:statements, version:"version-date"}'
            ```

            ## 5. Export current statements as rollback backup (ALWAYS before write)

            ```bash
            oci iam policy get \\
              --policy-id <POLICY_OCID> \\
              --query 'data.statements' > /tmp/policy-backup-$(date +%Y%m%dT%H%M%S).json
            echo "Backup saved. Proceed only after confirming backup is complete."
            ```
        """),
        "rollback_body": textwrap.dedent("""\
            # IAM Policy Compartment — Rollback Playbook

            ## Restore previous policy statements

            ```bash
            # Read backup statements from file saved in preflight step
            PREV_STATEMENTS=$(cat /tmp/policy-backup-<TIMESTAMP>.json)

            oci iam policy update \\
              --policy-id <POLICY_OCID> \\
              --statements "${PREV_STATEMENTS}" \\
              --version-date $(date +%Y-%m-%d) \\
              --force
            ```

            ## Verify policy restored correctly

            ```bash
            oci iam policy get \\
              --policy-id <POLICY_OCID> \\
              --query 'data.{name:name, statements:statements, version:"version-date"}'
            ```

            ## Delete a newly-created incorrect policy immediately

            ```bash
            oci iam policy delete \\
              --policy-id <POLICY_OCID> \\
              --force
            ```

            WARNING: policy delete is **immediate and total** — all access granted by the policy
            is revoked the moment the delete completes. This can cause service outages if the policy
            granted runtime access to compute or database resources. Confirm blast radius before delete.

            ## Disable a dynamic group (remove matching rule to prevent new matches)

            ```bash
            oci iam dynamic-group update \\
              --dynamic-group-id <DG_OCID> \\
              --matching-rule "None {instance.id = 'ocid1.instance.oc1.PLACEHOLDER'}"
            ```

            This effectively empties the group without deleting it.
        """),
    },
    {
        "id": "oci-live-oke-rollout-guard",
        "name": "OCI Live OKE Rollout Guard",
        "summary": "Guard OKE deployment rollouts through DevOps Service pipeline approval stages with blue-green and canary evidence, and kubectl rollout pause or undo gate.",
        "focus": "Guard OCI Kubernetes Engine deployment rollouts through DevOps Service pipeline approval stages, enforcing blue-green or canary evidence, kubectl rollout health checks, and explicit undo or advance decision.",
        "codex_role": "oke-rollout live operator",
        "skill_desc": "Guard OKE deployment rollouts via DevOps Service approval stages with canary and blue-green evidence, rollout health verification, and kubectl rollout undo gates.",
        "skill_when": [
            "an OKE deployment rollout must advance through a DevOps Service pipeline approval stage",
            "a blue-green or canary OKE deployment is in flight and the operator must decide to promote or rollback",
            "a kubectl rollout is paused on a live OKE cluster and an undo or resume decision is required",
        ],
        "response_shape": [
            "OKE cluster identity and DevOps pipeline confirmation",
            "Current rollout status and PDB health (kubectl rollout status + get pdb)",
            "DevOps pipeline stage and approval gate status",
            "Blue-green or canary traffic split evidence",
            "Approval status for advance, pause, or undo",
            "Proposed or executed rollout action",
            "Post-rollout pod health and service endpoint verification",
        ],
        "official_docs": [
            "https://docs.oracle.com/en-us/iaas/Content/devops/using/deploy_oke.htm",
            "https://docs.oracle.com/en-us/iaas/Content/devops/using/bgoke_deploy.htm",
            "https://docs.oracle.com/en-us/iaas/Content/devops/using/canaryoke_deploy.htm",
            "https://docs.oracle.com/en-us/iaas/Content/ContEng/Concepts/contengoverview.htm",
        ],
        "security_notes": "Never advance an OKE rollout past an approval stage without rollout status and PDB health evidence. kubectl rollout undo is irreversible in the sense that the prior version may not be identical to the deployed artifact — confirm target revision before undo.",
        "permissions_body": textwrap.dedent("""\
            # OCI IAM policy for OKE rollout guard

            ## Identity model preference

            1. DevOps Service pipeline with explicit approval stage — human must approve before deploy
            2. OKE cluster RBAC (Kubernetes-native) for in-cluster operations, not IAM only
            3. Separate read-only and deploy-operator groups at compartment scope

            ## OKE cluster read (no deploy rights)

            ```
            Allow group <oke-auditors> to read clusters in compartment <prod-compartment>
            Allow group <oke-auditors> to read cluster-node-pools in compartment <prod-compartment>
            ```

            ## DevOps pipeline read + deployment use

            ```
            Allow group <oke-operators> to read devops-pipelines in compartment <prod-compartment>
            Allow group <oke-operators> to read devops-deployments in compartment <prod-compartment>
            Allow group <oke-operators> to use devops-deployments in compartment <prod-compartment>
            ```

            ## OKE admin for rollback (use, NOT manage — cannot delete clusters)

            ```
            Allow group <oke-admins> to use clusters in compartment <prod-compartment>
            Allow group <oke-admins> to manage cluster-node-pools in compartment <prod-compartment>
            ```

            ## DevOps service dynamic group (pipeline automation)

            ```
            Allow dynamic-group <devops-pipeline-runners> to use cluster in compartment <prod-compartment>
            Allow dynamic-group <devops-pipeline-runners> to manage cluster-node-pools in compartment <prod-compartment>
            ```

            `use cluster` (not `manage cluster`) for the pipeline dynamic group: `manage` grants
            cluster termination rights, which must never be automated. Node pool management
            (`manage cluster-node-pools`) covers rolling updates, scaling, and version upgrades
            without exposing cluster deletion.

            ## Service-principal policies (OKE + DevOps services)

            OCI is policy-based IAM: the OKE control plane and the DevOps pipeline service
            each need their own `Allow service ...` grants. Without these, node pool scaling
            and pipeline execution fail with `NotAuthorized` even when human operators are
            correctly scoped.

            ```
            Allow service OKE to manage cluster-node-pools in compartment <prod-compartment>
            Allow service OKE to use virtual-network-family in compartment <prod-compartment>
            Allow service OKE to manage instance-family in compartment <prod-compartment>
              where target.resource.tag.Operations.OkeManaged.value = 'true'

            Allow service devops to use ons-topics in compartment <prod-compartment>
            Allow service devops to manage repos in compartment <prod-compartment>
            Allow service devops to read secret-family in compartment <prod-compartment>
            ```

            The `OkeManaged = 'true'` tag condition prevents OKE from acting on instances
            that are not part of a managed node pool — an extra least-privilege guard on
            the service principal itself.

            ## Do not use

            ```
            # FORBIDDEN
            Allow group <oke-operators> to manage clusters in compartment prod
              # "manage" allows cluster termination — use "use" for operators
            Allow dynamic-group <all-instances> to manage all-resources in compartment prod
            ```

            ## Kubernetes RBAC (in-cluster)

            Bind the OKE operator's OCID to a namespace-scoped Role, not ClusterRole:

            ```yaml
            rules:
            - apiGroups: ["apps"]
              resources: ["deployments", "replicasets"]
              verbs: ["get", "list", "watch", "patch", "update"]
            - apiGroups: [""]
              resources: ["pods", "pods/log", "services"]
              verbs: ["get", "list", "watch"]
            - apiGroups: ["policy"]
              resources: ["poddisruptionbudgets"]
              verbs: ["get", "list"]
            ```
        """),
        "preflight_body": textwrap.dedent("""\
            # OKE Rollout — Preflight Commands

            ## 1. Confirm OKE cluster state

            ```bash
            oci ce cluster get \\
              --cluster-id <CLUSTER_OCID> \\
              --query 'data.{name:name, state:"lifecycle-state", version:"kubernetes-version", endpoint:endpoints}'
            ```

            ## 2. Check DevOps pipeline status

            ```bash
            oci devops deploy-pipeline get \\
              --pipeline-id <PIPELINE_OCID> \\
              --query 'data.{name:name, state:"lifecycle-state"}'

            # List deployment stages with types
            oci devops deploy-stage list \\
              --pipeline-id <PIPELINE_OCID> \\
              --query 'data.items[].{name:"display-name", type:"deploy-stage-type", id:id}'
            ```

            ## 3. Fetch kubeconfig and confirm context

            ```bash
            oci ce cluster create-kubeconfig \\
              --cluster-id <CLUSTER_OCID> \\
              --file $HOME/.kube/oci-prod-config \\
              --region <REGION> \\
              --token-version 2.0.0
            export KUBECONFIG=$HOME/.kube/oci-prod-config
            kubectl config current-context
            ```

            ## 4. Audit rollout strategy and PDB

            ```bash
            kubectl rollout status deployment/<DEPLOY_NAME> -n <NAMESPACE> --timeout=30s || true
            kubectl get pdb -n <NAMESPACE> -o wide
            kubectl describe deployment <DEPLOY_NAME> -n <NAMESPACE> | grep -A 5 "RollingUpdateStrategy"
            ```

            ## 5. Blue-green: confirm stable service selector before swap

            ```bash
            kubectl get svc <SERVICE_NAME> -n <NAMESPACE> \\
              -o jsonpath='{.spec.selector}' | python3 -m json.tool
            ```
        """),
        "rollback_body": textwrap.dedent("""\
            # OKE Rollout — Rollback Playbook

            ## Option 1: kubectl rollback (in-cluster, immediate)

            ```bash
            kubectl rollout undo deployment/<DEPLOY_NAME> -n <NAMESPACE>
            kubectl rollout status deployment/<DEPLOY_NAME> -n <NAMESPACE>
            ```

            ## Option 2: Blue-green — switch service selector back to stable

            ```bash
            kubectl patch service <SERVICE_NAME> -n <NAMESPACE> \\
              -p '{"spec":{"selector":{"version":"<STABLE_VERSION>"}}}'

            # Confirm traffic is on stable
            kubectl get svc <SERVICE_NAME> -n <NAMESPACE> -o jsonpath='{.spec.selector}'
            ```

            ## Option 3: OCI DevOps — re-run previous successful deployment

            ```bash
            # Find last successful deployment
            oci devops deployment list \\
              --pipeline-id <PIPELINE_OCID> \\
              --query 'data.items[?contains("lifecycle-state", `SUCCEEDED`)][0].id'

            oci devops deployment create-single-deploy-stage-deployment \\
              --deploy-pipeline-id <PIPELINE_OCID> \\
              --deploy-stage-id <STABLE_STAGE_OCID> \\
              --display-name "rollback-$(date +%Y%m%dT%H%M%S)"
            ```

            ## Option 4: Node pool scale-down (if node-level instability is the root cause)

            ```bash
            oci ce node-pool update \\
              --node-pool-id <NODE_POOL_OCID> \\
              --node-config-details '{"size": <PREVIOUS_SIZE>}'
            ```

            ## Verify

            ```bash
            kubectl get pods -n <NAMESPACE> -l app=<APP_LABEL>
            kubectl top pods -n <NAMESPACE>
            ```
        """),
    },
    {
        "id": "oci-live-autonomous-db-lifecycle-guard",
        "name": "OCI Live Autonomous DB Lifecycle Guard",
        "summary": "Guard Autonomous Database scale, start, stop, clone, and terminate operations with protection-tag check, wallet backup, and connection-string audit before any lifecycle mutation.",
        "focus": "Guard OCI Autonomous Database lifecycle operations (scale, start, stop, clone, terminate) by verifying protection tags, wallet and backup state, and connection-string impact before any mutation.",
        "codex_role": "autonomous-db-lifecycle live operator",
        "skill_desc": "Guard Autonomous Database lifecycle changes — scale, start, stop, clone, terminate — with protection-tag enforcement, backup verification, and connection-string impact analysis before any mutation.",
        "skill_when": [
            "an Autonomous Database must be scaled, started, stopped, cloned, or terminated against a live OCI environment",
            "a protection tag must be audited before a lifecycle operation that could cause data loss or outage",
            "an Autonomous Database backup or wallet must be confirmed before a scale or clone operation",
        ],
        "response_shape": [
            "Autonomous Database identity and current lifecycle state",
            "Protection tag audit (defined tags and freeform tags for deletion guard)",
            "Backup inventory and most recent completed backup timestamp",
            "Connection string and consumer group impact assessment",
            "Approval status for the requested lifecycle operation",
            "Proposed or executed lifecycle action",
            "Post-operation state verification and open risks (non-reversible operations listed)",
        ],
        "official_docs": [
            "https://docs.oracle.com/en-us/iaas/Content/Database/Tasks/adbscaling.htm",
            "https://docs.oracle.com/en-us/iaas/Content/Database/Tasks/adbstopstart.htm",
            "https://docs.oracle.com/en-us/iaas/Content/Database/Tasks/adbcloning.htm",
            "https://docs.oracle.com/en-us/iaas/Content/Database/Tasks/adbbackingup.htm",
        ],
        "security_notes": "ADB termination is permanent — the database and all backups are deleted. Always verify protection tags before any terminate operation. ADB storage scale-up cannot be reversed. Termination blocked by defined-tag protection requires explicit tag removal approval.",
        "permissions_body": textwrap.dedent("""\
            # OCI IAM policy for Autonomous DB lifecycle guard

            ## Identity model preference

            1. Separate groups for readers, operators (start/stop/scale), and admins (clone/terminate)
            2. `use` verb for operators — prevents terminate and clone
            3. `manage` with tag condition for admins — allows terminate only when protection tag is absent
            4. Defined-tag namespace for protection tagging (use a protected namespace, not freeform)

            ## Baseline read (no mutation)

            ```
            Allow group <adb-auditors> to inspect autonomous-databases in compartment <prod-db-compartment>
            Allow group <adb-auditors> to read autonomous-databases in compartment <prod-db-compartment>
            Allow group <adb-auditors> to read autonomous-database-backups in compartment <prod-db-compartment>
            ```

            ## Operations — start, stop, scale (use verb, no terminate/clone)

            ```
            Allow group <adb-operators> to use autonomous-databases in compartment <prod-db-compartment>
            ```

            With `use` the operator can: start, stop, scale CPU/storage, generate wallet.
            The operator CANNOT: terminate, clone to new, change network-access type.

            ## Admin — clone and terminate (manage + tag condition)

            ```
            Allow group <adb-admins> to manage autonomous-databases in compartment <prod-db-compartment>
              where target.resource.tag.Operations.Lifecycle.value != 'protected'
            ```

            Tag condition: `manage` verbs only succeed if the ADB's defined tag
            `Operations.Lifecycle` is NOT set to `protected`. Set this tag on all production ADBs
            in a protected tag namespace (so only tag-namespace admins can remove it).

            > **IRREVERSIBILITY WARNING — read before granting `manage`:**
            >
            > - **Termination** is permanent. OCI does not recover terminated ADB instances.
            >   The 60-day automatic backup retention window expires; after that, no recovery path exists.
            > - **Storage scale-up** (`ocpuCount` or `dataStorageSizeInTBs` increase) cannot be reversed.
            >   You can scale CPU down, but storage can only grow — never shrink.
            > - Both operations must require dual-sign-off and a confirmed maintenance window
            >   before this role is used. The tag-condition gate is a necessary but insufficient control.

            ## Do not use

            ```
            # FORBIDDEN
            Allow group <adb-operators> to manage autonomous-databases in tenancy
            Allow any-user to use autonomous-databases in compartment prod-db
            ```
        """),
        "preflight_body": textwrap.dedent("""\
            # Autonomous DB Lifecycle — Preflight Commands

            ## 1. Get ADB state and confirm target

            ```bash
            oci db autonomous-database get \\
              --autonomous-database-id <ADB_OCID> \\
              --query 'data.{name:"display-name", state:"lifecycle-state", cpu:"cpu-core-count", storage:"data-storage-size-in-tbs", version:"db-version", workload:"db-workload"}'
            ```

            ## 2. Audit protection tags (CRITICAL — check before any lifecycle op)

            ```bash
            oci db autonomous-database get \\
              --autonomous-database-id <ADB_OCID> \\
              --query 'data.{definedTags:"defined-tags", freeformTags:"freeform-tags"}'
            ```

            Stop if `Operations.Lifecycle = protected` is set on a defined-tag namespace.
            Do not proceed with terminate or clone without explicit tag-removal approval.

            ## 3. Confirm recent backup exists

            ```bash
            oci db autonomous-database-backup list \\
              --autonomous-database-id <ADB_OCID> \\
              --all \\
              --query 'data[0:5].{id:id, type:type, state:"lifecycle-state", ended:"time-ended"}' \\
              --output table
            ```

            Fail-fast if no ACTIVE backup exists within RPO window before scale or stop operations.

            ## 4. Audit connection strings and consumer groups

            ```bash
            oci db autonomous-database get \\
              --autonomous-database-id <ADB_OCID> \\
              --query 'data."connection-strings".{high:high, medium:medium, low:low}'
            ```

            ## 5. Check data guard and APEX linkage (termination blockers)

            ```bash
            oci db autonomous-database get \\
              --autonomous-database-id <ADB_OCID> \\
              --query 'data.{dataGuard:"is-data-guard-enabled", autoScaling:"is-auto-scaling-enabled", apex:"apex-details"}'
            ```
        """),
        "rollback_body": textwrap.dedent("""\
            # Autonomous DB Lifecycle — Rollback Playbook

            ## Start a stopped ADB (fastest recovery from accidental stop)

            ```bash
            oci db autonomous-database start \\
              --autonomous-database-id <ADB_OCID>

            # Wait for AVAILABLE state
            oci db autonomous-database get \\
              --autonomous-database-id <ADB_OCID> \\
              --query 'data."lifecycle-state"'
            ```

            ## Scale CPU back to previous count (scale-down is supported)

            ```bash
            oci db autonomous-database update \\
              --autonomous-database-id <ADB_OCID> \\
              --cpu-core-count <PREVIOUS_CPU_COUNT>
            ```

            WARNING: **Storage scale-up cannot be reversed on ADB.** Verify storage size before
            scaling up — there is no reduce path once committed.

            ## Restore from backup after data-level issue

            ```bash
            # Point-in-time recovery
            oci db autonomous-database restore \\
              --autonomous-database-id <ADB_OCID> \\
              --timestamp "2026-04-29T10:00:00.000Z"
            ```

            ## Clone-to-new for investigation (non-destructive)

            ```bash
            oci db autonomous-database create-from-clone \\
              --compartment-id <COMPARTMENT_OCID> \\
              --db-name "<CLONE_NAME>" \\
              --source-id <ADB_OCID> \\
              --clone-type FULL
            ```

            ## CANNOT ROLL BACK

            - **Terminated ADB**: database and all backups are permanently deleted.
              No OCI Support recovery path exists.
            - **Storage scale-up**: ADB storage can only grow, never shrink.
            - **Prevention**: always verify `Operations.Lifecycle = protected` tag is set on prod ADBs.
        """),
    },
    {
        "id": "oci-live-vault-key-destruction-guard",
        "name": "OCI Live Vault Key Destruction Guard",
        "summary": "Guard OCI Vault master encryption key scheduled-deletion and HSM key rotation, refusing deletion without reviewing data associations and confirming the destruction window.",
        "focus": "Guard OCI Vault master encryption key scheduled-deletion and HSM rotation by auditing all data associations, key-usage references, and confirming the deletion window before any destruction scheduling.",
        "codex_role": "vault-key-destruction live operator",
        "skill_desc": "Guard Vault master encryption key scheduled-deletion and HSM rotation with data-association audits, key-usage reference checks, deletion-window enforcement, and cancellation playbooks.",
        "skill_when": [
            "an OCI Vault master encryption key must be scheduled for deletion or rotated to a new version",
            "a key scheduled for deletion must be cancelled before the destruction window expires",
            "an HSM-backed key usage must be audited before any key version lifecycle change",
        ],
        "response_shape": [
            "Vault and key identity confirmation (protection mode: HSM vs SOFTWARE)",
            "Key version inventory and current active version",
            "Data association audit (resources encrypted by this key version)",
            "Deletion window confirmation (minimum 7 days, default 30 days)",
            "Approval status for key rotation or deletion scheduling",
            "Proposed or executed vault key action",
            "Post-action state and irreversibility warning (point-of-no-return explicitly stated)",
        ],
        "official_docs": [
            "https://docs.oracle.com/en-us/iaas/Content/KeyManagement/Tasks/deletingkeys.htm",
            "https://docs.oracle.com/en-us/iaas/Content/KeyManagement/Tasks/rotatingkeys.htm",
            "https://docs.oracle.com/en-us/iaas/Content/KeyManagement/Concepts/keyoverview.htm",
            "https://docs.oracle.com/en-us/iaas/Content/KeyManagement/Tasks/managingkeys.htm",
        ],
        "security_notes": "After the scheduled deletion window expires, HSM-backed keys are cryptographically wiped. All data encrypted exclusively by that key version is permanently unrecoverable. Recovery SLA from OCI Support: NONE. Always use a 30-day window and audit data associations before scheduling.",
        "permissions_body": textwrap.dedent("""\
            # OCI IAM policy for Vault key destruction guard

            ## Identity model preference

            1. Separate groups for key auditors, key rotation operators, and key destruction admins
            2. `use` verb for rotation operators — creates new key versions, cannot schedule deletion
            3. `manage` for key destruction admins, restricted by tag condition (deletable tag required)
            4. Dual-control: key deletion requires a second approver group confirmation

            ## Key audit policy (read only, no mutation)

            ```
            Allow group <vault-auditors> to inspect vaults in compartment <prod-vault-compartment>
            Allow group <vault-auditors> to read vaults in compartment <prod-vault-compartment>
            Allow group <vault-auditors> to read keys in compartment <prod-vault-compartment>
            Allow group <vault-auditors> to inspect key-versions in compartment <prod-vault-compartment>
            ```

            ## Key rotation (use verb — new versions only, no deletion scheduling)

            ```
            Allow group <vault-key-operators> to use keys in compartment <prod-vault-compartment>
            Allow group <vault-key-operators> to use key-delegate in compartment <prod-vault-compartment>
            ```

            With `use` the operator can: create new key versions, enable/disable key versions.
            The operator CANNOT: schedule key deletion, delete the key, import key material.

            ## Key destruction (manage + tag condition — only for approved-deletable keys)

            ```
            Allow group <vault-key-admins> to manage keys in compartment <prod-vault-compartment>
              where target.resource.tag.Lifecycle.Deletable.value = 'approved'
            ```

            The `Lifecycle.Deletable = approved` tag must be set in a protected tag namespace.
            Production keys should never have this tag set unless they are actively being retired.

            ## CRITICAL timing note

            ```
            Minimum deletion window: 7 days
            Recommended deletion window: 30 days
            Cancel deadline: any time BEFORE time-of-deletion passes
            After deletion: PERMANENT. No recovery. No OCI Support escalation path.
            ```

            ## Do not use

            ```
            # FORBIDDEN
            Allow group <vault-operators> to manage all-resources in compartment prod-vault
            Allow any-user to manage keys in tenancy
            ```
        """),
        "preflight_body": textwrap.dedent("""\
            # Vault Key Destruction — Preflight Commands

            ## 1. Get key metadata and protection mode

            ```bash
            oci kms management key get \\
              --key-id <KEY_OCID> \\
              --endpoint <VAULT_MANAGEMENT_ENDPOINT> \\
              --query 'data.{name:"display-name", state:"lifecycle-state", protection:"protection-mode", algo:"key-shape".algorithm, scheduledDeletion:"time-of-deletion"}'
            ```

            **STOP** if `protection-mode = HSM` — HSM key destruction is irreversible.
            SOFTWARE keys can be re-imported; HSM keys cannot be recovered after destruction.

            ## 2. List all key versions (identify active and retired)

            ```bash
            oci kms management key-version list \\
              --key-id <KEY_OCID> \\
              --endpoint <VAULT_MANAGEMENT_ENDPOINT> \\
              --all \\
              --query 'data[].{version:"key-version-id", state:"lifecycle-state", created:"time-created"}' \\
              --output table
            ```

            ## 3. Audit data associations (resources encrypted by this key)

            ```bash
            # Note: OCI does not always provide a complete list via API.
            # Supplement with a resource search:
            oci resource search search-resources \\
              --query-text 'query all resources where freeformTags.EncryptionKeyId = '"'"'<KEY_OCID>'"'"'' \\
              --query 'data.items[].{type:"resource-type", name:"display-name", compartment:"compartment-id"}'
            ```

            If the association list is incomplete, perform a manual audit via tags before proceeding.

            ## 4. Check vault type (Virtual Private vs Shared HSM)

            ```bash
            oci kms vault get \\
              --vault-id <VAULT_OCID> \\
              --query 'data.{type:"vault-type", state:"lifecycle-state", endpoint:"management-endpoint"}'
            ```

            ## 5. Confirm the Lifecycle.Deletable tag is set (required by our IAM policy)

            ```bash
            oci kms management key get \\
              --key-id <KEY_OCID> \\
              --endpoint <VAULT_MANAGEMENT_ENDPOINT> \\
              --query 'data."defined-tags"'
            ```
        """),
        "rollback_body": textwrap.dedent("""\
            # Vault Key Destruction — Rollback Playbook

            ## Cancel a scheduled key deletion (before time-of-deletion)

            ```bash
            oci kms management key cancel-key-deletion \\
              --key-id <KEY_OCID> \\
              --endpoint <VAULT_MANAGEMENT_ENDPOINT>

            # Verify cancellation
            oci kms management key get \\
              --key-id <KEY_OCID> \\
              --endpoint <VAULT_MANAGEMENT_ENDPOINT> \\
              --query 'data.{state:"lifecycle-state", scheduledDeletion:"time-of-deletion"}'
            ```

            ## Re-enable the key after cancellation

            ```bash
            oci kms management key enable \\
              --key-id <KEY_OCID> \\
              --endpoint <VAULT_MANAGEMENT_ENDPOINT>
            ```

            ## Rotate to a new key version (non-destructive — old version remains available for decrypt)

            ```bash
            oci kms management key create-key-version \\
              --key-id <KEY_OCID> \\
              --endpoint <VAULT_MANAGEMENT_ENDPOINT>
            ```

            Old key versions remain ENABLED until explicitly disabled, allowing decryption of
            data encrypted by prior versions. This is the safe rotation pattern.

            ## POINT OF NO RETURN

            After `time-of-deletion` passes:

            - HSM key: cryptographic material is wiped from the HSM. **Permanent. No recovery.**
            - All data encrypted exclusively by this key version is **unrecoverable**.
            - OCI Support Recovery SLA: **NONE**.
            - Immediate escalation: open a P1 SR with OCI Support the moment accidental deletion is suspected.

            Prevention checklist before scheduling deletion:
            - [ ] All data encrypted by this key has been re-encrypted with the new key version
            - [ ] All services using this key version have been updated to the new version
            - [ ] A 30-day (not 7-day) deletion window was selected
            - [ ] A second approver has confirmed the data-association audit
        """),
    },
    {
        "id": "oci-live-cost-budget-runaway-guard",
        "name": "OCI Live Cost Budget Runaway Guard",
        "summary": "Gate OCI budget rule mutations, cost-tracking tag changes, and GPU or HPC shape provisioning against compartment spend limits before any cost-impacting mutation.",
        "focus": "Gate OCI budget rule mutations, cost-tracking tag changes, and GPU/HPC shape provisioning (BM.GPU4.8, A100, BM.HPC2.36) against compartment spend limits and approved quotas.",
        "codex_role": "cost-budget-runaway live operator",
        "skill_desc": "Gate OCI budget mutations and GPU/HPC shape provisioning against compartment spend limits, with inventory searches, quota audits, and emergency spend-stop playbooks.",
        "skill_when": [
            "an OCI budget rule threshold or alert must be modified for a tenancy or compartment",
            "a GPU or HPC shape provisioning request requires spend-limit approval before creating",
            "a runaway GPU cost event is detected and emergency quota reduction or instance stop is needed",
        ],
        "response_shape": [
            "Tenancy and compartment identity confirmation",
            "Active budget inventory and current spend vs threshold (oci budgets budget list)",
            "GPU/HPC shape quota usage and running instance inventory",
            "Cost-tracking tag namespace audit",
            "Approval status for budget change or GPU/HPC provisioning",
            "Proposed or executed cost-governance action",
            "Post-change budget alert confirmation and monitoring state",
        ],
        "official_docs": [
            "https://docs.oracle.com/en-us/iaas/Content/Billing/Tasks/managingbudgets.htm",
            "https://docs.oracle.com/en-us/iaas/Content/Compute/Tasks/managinginstances.htm",
            "https://docs.oracle.com/en-us/iaas/Content/Tagging/Tasks/managingtagsandtagnamespaces.htm",
            "https://docs.oracle.com/en-us/iaas/Content/General/Concepts/resourcequotas.htm",
        ],
        "security_notes": "GPU/HPC shapes (BM.GPU4.8, A100, BM.HPC2.36) can generate six-figure monthly costs when left running. Never approve quota increases or budget threshold raises without explicit financial-authority approval. Emergency stop requires Compute operator rights — escalate if not held.",
        "permissions_body": textwrap.dedent("""\
            # OCI IAM policy for cost budget runaway guard

            ## Identity model preference

            1. Named cost-governance group with tenancy-scoped budget management
            2. Separate cost-auditors (inspect/read only) from cost-admins (manage)
            3. GPU provisioning gates via compartment quota policies — not IAM `manage`
            4. Never grant `manage compute-instances in tenancy` to the cost-guard role

            ## Budget read (audit, no mutation)

            ```
            Allow group <cost-auditors> to inspect usage-budgets in tenancy
            Allow group <cost-auditors> to read usage-budgets in tenancy
            Allow group <cost-auditors> to inspect costs in tenancy
            Allow group <cost-auditors> to read costs in tenancy
            ```

            ## Budget write (manage — budgets are tenancy-scoped resources)

            ```
            Allow group <cost-admins> to manage usage-budgets in tenancy
            ```

            ## Quota inspection and resource search

            ```
            Allow group <cost-admins> to inspect quota in tenancy
            Allow group <cost-admins> to read quota in tenancy
            Allow group <cost-admins> to use resource-search in tenancy
            ```

            ## Cost operators (middle tier — adjust budgets, cannot delete)

            OCI policy-based IAM supports tier separation by verb. Cost operators can
            re-tune budget thresholds and notification rules without holding `manage`
            delete rights:

            ```
            Allow group <cost-operators> to use usage-budgets in tenancy
            Allow group <cost-operators> to read costs in tenancy
            Allow group <cost-operators> to use ons-topics in compartment <cost-alerts-compartment>
            ```

            `use usage-budgets` permits update + alert rule changes; it does NOT permit
            budget creation or deletion — those remain with `<cost-admins>`.

            ## Cost-tracking tag namespace management

            ```
            Allow group <cost-admins> to manage tag-namespaces in compartment <cost-tracking-compartment>
            Allow group <cost-admins> to use tag-namespaces in tenancy
            ```

            ## GPU/HPC shape gate via compartment quota (strongest control)

            Set a compartment-level quota to prevent GPU provisioning without explicit increase:

            ```
            set compute-core-count quota gpu-vm-count to 0 in compartment <default-compute>
            ```

            This physically prevents any GPU shape from being provisioned without a quota
            increase request — a harder gate than IAM deny policies.

            ## Do not use

            ```
            # FORBIDDEN
            Allow group <cost-admins> to manage all-resources in tenancy
            Allow any-group to manage compute-instances in tenancy
            Allow group <cost-admins> to manage compute-instances in tenancy
              # Cost guard should not have VM create/stop rights — escalate to compute operator
            ```
        """),
        "preflight_body": textwrap.dedent("""\
            # Cost Budget Runaway — Preflight Commands

            ## 1. List all budgets and current utilization

            ```bash
            oci budgets budget list \\
              --compartment-id <TENANCY_OCID> \\
              --all \\
              --query 'data[].{name:"display-name", amount:amount, spent:"actual-spend", forecast:"forecasted-spend", reset:"reset-period"}' \\
              --output table
            ```

            ## 2. Check compute GPU/HPC service limits

            ```bash
            oci limits value list \\
              --compartment-id <TENANCY_OCID> \\
              --service-name compute \\
              --all \\
              --query 'data[?contains(name, `gpu`) || contains(name, `hpc`)].{name:name, value:value, scope:"scope-type"}' \\
              --output table
            ```

            ## 3. Search for running GPU/HPC instances across tenancy

            ```bash
            oci resource search search-resources \\
              --query-text 'query instance resources where
                (shape = '"'"'BM.GPU4.8'"'"' ||
                 shape = '"'"'VM.GPU3.1'"'"' ||
                 shape = '"'"'BM.HPC2.36'"'"' ||
                 shape = '"'"'BM.GPU.H100.8'"'"') &&
                lifecycleState = '"'"'RUNNING'"'"'' \\
              --query 'data.items[].{id:"identifier", name:"display-name", compartment:"compartment-id"}'
            ```

            ## 4. Audit cost-tracking tag namespaces

            ```bash
            oci iam tag-namespace list \\
              --compartment-id <TENANCY_OCID> \\
              --all \\
              --query 'data[].{name:name, state:"lifecycle-state", isRetired:"is-retired"}' \\
              --output table
            ```

            ## 5. Check active budget alerts

            ```bash
            oci budgets alert list \\
              --compartment-id <TENANCY_OCID> \\
              --all \\
              --query 'data[].{budgetId:"budget-id", threshold:threshold, triggered:"time-first-triggered"}'
            ```
        """),
        "rollback_body": textwrap.dedent("""\
            # Cost Budget Runaway — Rollback Playbook

            ## Restore a raised budget threshold to previous value

            ```bash
            oci budgets budget update \\
              --budget-id <BUDGET_OCID> \\
              --amount <PREVIOUS_AMOUNT>

            # Verify
            oci budgets budget get \\
              --budget-id <BUDGET_OCID> \\
              --query 'data.{amount:amount, reset:"reset-period", spent:"actual-spend"}'
            ```

            ## Emergency: stop a runaway GPU instance (requires Compute operator — escalate if needed)

            ```bash
            # Soft stop (OCPU billing continues for stopped-but-preserved VMs until termination)
            oci compute instance action \\
              --instance-id <INSTANCE_OCID> \\
              --action STOP

            # For bare metal GPU (BM.GPU4.8) — billing stops only on TERMINATE
            # Escalate to Compute operator with appropriate compartment manage rights
            ```

            ## Lower a compartment GPU quota to prevent further provisioning

            ```bash
            oci limits quota create \\
              --compartment-id <COMPARTMENT_OCID> \\
              --name "emergency-gpu-cap-$(date +%Y%m%d)" \\
              --statements '["set compute-core-count quota gpu-count to 0 in compartment <COMPARTMENT>"]'
            ```

            ## Revert a budget alert threshold change

            ```bash
            oci budgets alert update \\
              --budget-id <BUDGET_OCID> \\
              --alert-id <ALERT_OCID> \\
              --threshold <PREVIOUS_THRESHOLD> \\
              --threshold-type ABSOLUTE
            ```

            ## Verify budget enforcement is restored

            ```bash
            oci budgets budget get \\
              --budget-id <BUDGET_OCID> \\
              --query 'data.{amount:amount, alerts:alerts[*].threshold}'
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
        f"- `skills/oci/{agent['id']}/SKILL.md`",
        "",
        f"Load files under `skills/oci/{agent['id']}/references/` only when the task needs that reference. Do not dump reference text into the response.",
        "",
        "## Focus",
        "",
        agent["focus"],
        "",
        "## Operating Rules",
        "",
        "- Load and follow the bound OCI skill first; do not drift into generic cloud advice.",
        "- This role is for repos or sessions that may be connected to live OCI credentials, CLI profiles, or real environments.",
        "- Before any live OCI mutation, confirm tenancy, compartment, active principal, exact target resource, expected impact, and explicit human approval.",
        "- Prefer plan, detect-drift, inspect, read, describe, and rollback evidence before mutation.",
        "- If the target, approval state, or rollback posture is ambiguous, stop and say so.",
        "- Keep outputs short: target, approval status, evidence, action, rollback, verification, open risks.",
        "- Never ask for secrets, credentials, private keys, tenancy OCIDs, or raw config dumps unless already sanitized and required.",
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
        "- Load and follow the bound OCI skill first; do not drift into generic cloud advice.",
        "- This role is for repos or sessions that may be connected to live OCI credentials, CLI profiles, or real environments.",
        "- Before any live OCI mutation, confirm tenancy, compartment, active principal, exact target resource, expected impact, and explicit human approval.",
        "- Prefer plan, detect-drift, inspect, read, describe, and rollback evidence before mutation.",
        "- If approval, identity, target, or rollback posture is ambiguous, stop and explain the blocker.",
        "- Never ask for secrets, credentials, private keys, tenancy OCIDs, or raw config dumps unless already sanitized and required.",
        "- Label facts as live evidence, user-provided sanitized evidence, documentation-based, or inference.",
    ])
    return f"""name = "{agent['id']}_agent"
description = "Specialized subagent for {agent['id']}. {agent['summary']}"
model = "gpt-5.4"
model_reasoning_effort = "high"
sandbox_mode = "workspace-write"

developer_instructions = \"\"\"
Load and follow the bound `{agent['id']}` skill first. This agent exists only for that guarded live-OCI role; do not drift into generic cloud advice.

Token discipline:
- Read only SKILL.md first; load references only when the task requires them.
- Keep answers compact: target, approval status, evidence, action, rollback, verification, open risks.
- Do not paste long docs, raw tool inventories, raw credential output, or full environment dumps.

Role focus: {agent['focus']}

Safety contract:
{rules}
\"\"\"

[[skills.config]]
path = "skills/oci/{agent['id']}/SKILL.md"
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
        "provider": "oci",
        "harnesses": ["codex", "copilot", "claude-code", "cursor", "gemini", "kiro"],
        "summary": agent["summary"],
        "source_type": "original",
        "official_docs": agent["official_docs"],
        "security_notes": agent["security_notes"],
        "last_verified": DATE,
        "path": f"agents/oci/{agent['id']}-agent",
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

Act as the guarded live OCI operator for {agent['id']} work. Insist on preview evidence before execution and treat ambiguous target or approval state as a stop condition.

## When to use

Use this skill when:

{when_items}

## Lean operating rules

- Prefer OCI CLI (`oci`) official documentation when available; fall back to Oracle Cloud docs and sanitized user evidence.
- Do not execute a live OCI change until tenancy, compartment, active principal, and resource ownership are explicit.
- Prefer plan, detect-drift, inspect, read, describe, and rollback evidence before execution.
- If the request skips preview or rollback design, push back.
- Never print secrets, API keys, tenancy OCIDs, private key contents, or raw config values. Summarize sanitized evidence only.
- Load references only when needed.

## References

Load these only when needed:

- [Preflight commands](references/preflight-commands.md) — OCI CLI commands to run before any mutation.
- [Rollback playbook](references/rollback-playbook.md) — concrete rollback steps for this service.
- [Permission model](references/permission-model.md) — OCI IAM policy statements and dynamic group guidance.
- [Official sources](references/official-sources.md) — authoritative OCI documentation links.

## Response minimum

Return, at minimum:

- confirmed tenancy, compartment, and active principal
- preflight evidence (plan output, drift result, inspect/read, health check)
- approval status for the proposed mutation
- rollback posture or explicit statement of what cannot be rolled back
- post-action verification steps or refusal reason
"""


def skill_metadata_json(agent):
    return json.dumps({
        "id": agent["id"],
        "name": agent["name"],
        "type": "skill",
        "provider": "oci",
        "harnesses": ["codex", "claude-code", "cursor", "gemini", "kiro", "other"],
        "summary": agent["skill_desc"],
        "source_type": "original",
        "official_docs": agent["official_docs"],
        "security_notes": agent["security_notes"],
        "last_verified": DATE,
        "path": f"skills/oci/{agent['id']}",
        "author": "github: VincentChuWaiChow",
        "version": "0.1.0",
    }, indent=2) + "\n"


def build():
    for ag in AGENTS:
        aid = ag["id"]
        adir = os.path.join(ROOT, "agents", "oci", f"{aid}-agent")
        hdir = os.path.join(adir, "harnesses")
        sdir = os.path.join(ROOT, "skills", "oci", aid)
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
        write(os.path.join(rdir, "preflight-commands.md"), f"# Preflight Commands\n\nSee `../../PREFLIGHT.md` in the agent directory for executable commands.\n")
        write(os.path.join(rdir, "rollback-playbook.md"), f"# Rollback Playbook\n\nSee `../../ROLLBACK.md` in the agent directory for the full rollback playbook.\n")
        write(os.path.join(rdir, "permission-model.md"), f"# Permission Model\n\nSee `../../PERMISSIONS.md` in the agent directory for OCI IAM policy statements and dynamic group guidance.\n")
        write(os.path.join(rdir, "official-sources.md"), f"# Official Sources\n\n" + "\n".join(f"- {u}" for u in ag["official_docs"]) + "\n")

    print("\nOCI live-guard agents + skills generated.")


if __name__ == "__main__":
    build()
