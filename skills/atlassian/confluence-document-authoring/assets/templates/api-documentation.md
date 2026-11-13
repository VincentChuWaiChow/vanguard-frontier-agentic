# API Reference: {{SERVICE_AND_API_VERSION}}

<!-- Describe verified contracts. Mark proposed routes or behaviors Proposed; never invent endpoints, schemas, status codes, or successful test results. Duplicate the operation section only as needed. Remove comments and unused optional sections from the filled document. -->

| Metadata | Value |
| --- | --- |
| API / documentation owner | {{OWNER_AND_TEAM}} |
| Contract status | {{PROPOSED_CURRENT_DEPRECATED_OR_UNKNOWN}} |
| API version / document version | {{API_VERSION}} / {{DOCUMENT_VERSION}} |
| Last verified | {{DATE_ENVIRONMENT_AND_METHOD_OR_NOT_VERIFIED}} |
| Authoritative specification | {{SPEC_REFERENCE_REVISION_OR_UNKNOWN}} |
| Jira / related design | {{VERIFIED_REFERENCES_OR_NOT_LINKED}} |
| Audience | {{CLIENT_DEVELOPERS_OPERATORS_OR_PARTNERS}} |

## Purpose and scope

{{DESCRIBE_SUPPORTED_USE_CASES_AND_EXCLUDED_BEHAVIOR}}

## Environments and authentication

| Environment | Verified base address | Authentication / identity | Required permission / scope |
| --- | --- | --- | --- |
| {{ENVIRONMENT}} | {{VERIFIED_BASE_ADDRESS_OR_UNKNOWN}} | {{AUTHENTICATION_MECHANISM}} | {{EXACT_VERIFIED_PERMISSION_OR_UNKNOWN}} |

{{DESCRIBE_CREDENTIAL_HANDLING_TOKEN_LIFECYCLE_AND_AUTHORIZATION_BOUNDARIES_WITHOUT_SECRETS}}

## Operation: {{METHOD}} {{VERIFIED_PATH_OR_PROPOSED_PATH}}

{{DESCRIBE_PURPOSE_PRECONDITIONS_AND_SIDE_EFFECTS}}

| Contract field | Value / evidence |
| --- | --- |
| Method and path | {{VERIFIED_METHOD_AND_PATH_WITH_SPEC_POINTER}} |
| Permission / resource ownership | {{VERIFIED_AUTHORIZATION_REQUIREMENTS}} |
| Request / response media type | {{VERIFIED_MEDIA_TYPES}} |
| Idempotency / concurrency | {{VERIFIED_BEHAVIOR_OR_UNKNOWN}} |

### Request contract

| Location | Name | Type / required | Validation / default | Source |
| --- | --- | --- | --- | --- |
| {{PATH_QUERY_HEADER_OR_BODY}} | {{FIELD}} | {{TYPE_AND_REQUIRED_FLAG}} | {{CONSTRAINT_OR_NO_DEFAULT}} | {{SPEC_POINTER}} |

{{LINK_OR_INCLUDE_THE_RELEVANT_SCHEMA_REVISION_AND_DESCRIBE_CROSS_FIELD_BUSINESS_RULES}}

### Request example

Example provenance: {{SPEC_EXAMPLE_OR_SANITIZED_CAPTURE_OR_PROPOSED_ILLUSTRATION}}.

```text
{{REQUEST_EXAMPLE_CONSISTENT_WITH_THE_CONTRACT_WITHOUT_REAL_CREDENTIALS}}
```

### Success response

| Status / condition | Schema / fields | Semantics / source |
| --- | --- | --- |
| {{VERIFIED_STATUS_AND_CONDITION}} | {{SCHEMA_REFERENCE}} | {{MEANING_AND_SPEC_POINTER}} |

```text
{{RESPONSE_EXAMPLE_WITH_EXPLICIT_SPEC_OR_CAPTURE_PROVENANCE}}
```

### Errors and client recovery

| Verified status / code | Trigger | Response shape | Client action / retry safety |
| --- | --- | --- | --- |
| {{STATUS_OR_CODE}} | {{CONDITION}} | {{SCHEMA_OR_EXAMPLE_REFERENCE}} | {{VERIFIED_RECOVERY_OR_UNKNOWN}} |

<!-- Optional: retain for collection endpoints, asynchronous work, or constrained throughput. -->
### Pagination, limits, and asynchronous behavior

{{DESCRIBE_CURSOR_OR_OFFSET_ORDERING_TERMINATION_RATE_LIMITS_AND_POLLING_ONLY_WHEN_VERIFIED}}

## Versioning and compatibility

{{STATE_VERSION_SELECTION_SUPPORTED_VERSIONS_BREAKING_CHANGE_RULES_AND_DEPRECATION_EVIDENCE}}

## Verification and known limitations

| Contract / scenario | Evidence and revision | Environment / date | Result / limit |
| --- | --- | --- | --- |
| {{SCENARIO}} | {{SPEC_REVIEW_OR_TEST_ARTIFACT}} | {{ENVIRONMENT_AND_DATE}} | {{OBSERVED_RESULT_OR_NOT_RUN}} |

| Unknown / assumption | Consumer impact | Owner / resolution |
| --- | --- | --- |
| {{UNKNOWN_OR_ASSUMPTION}} | {{IMPACT}} | {{OWNER_AND_CHECK}} |

## Source register and change history

| Reference | Revision / checked | Supports / limitation |
| --- | --- | --- |
| {{VERIFIED_SPEC_CODE_OR_TEST_REFERENCE}} | {{REVISION_AND_DATE}} | {{CLAIM_AND_LIMITATION}} |

| Document version / date | Change | Related Jira / owner |
| --- | --- | --- |
| {{VERSION_AND_DATE}} | {{CONTRACT_OR_DOC_CHANGE}} | {{VERIFIED_REFERENCE_AND_OWNER}} |
