---
metadata:
  author: "github: VincentChuWaiChow"
---

# Cloud Provenance Rule

When answering cloud, IAM, Terraform, Kubernetes, database, or MCP questions:

- Verify drift-prone facts against official documentation or live configuration where possible.
- Label facts, inferences, assumptions, and uncertainty separately.
- Do not invent ARNs, subscription IDs, tenancy OCIDs, regions, resource names, or account IDs.
- Prefer least privilege and read-only inspection before mutation.
- If official docs and live state disagree, report the conflict instead of forcing a false conclusion.
