# Authorized live metadata audit

Before any read require exact principal, tenancy, compartment, region, endpoint,
table OCID/name and approved read-only evidence. Do not print auth configuration,
customer rows or broad inventory. Use scoped table/index/work-request/usage metadata
and selected Monitoring/key configuration only. A declared profile/principal is not
identity attestation; independently verify the active workload/human identity by
the approved host process. Stop on scope mismatch, authorization failure or drift;
never silently choose another compartment/region or mutate to repair. Document
readback time and gaps; ACTIVE/SUCCEEDED is not application/restore correctness.
