# Which image, signer identity and attestation claim does admission actually verify?

## Question

Does the image policy check the approved signer and the required attestation claims, or only that some signature or attestation exists?

## Release scope

ImageValidatingPolicy is Stable in v1.20.0 [KYV-E19]; field names below are from the v1 CRD at tag v1.20.0 [KYV-E17]. Legacy ClusterPolicy `verifyImages` is a different API and is not installable from v1.20.0 [KYV-E01]; do not transfer its fields to ImageValidatingPolicy.

## Assumptions

- The approved signer identity (subject and issuer, or key) and the required attestation types are supplied by the requester; choosing them is not this skill's job.
- Offline results do not show registry reachability or admission-time behavior.

## Evidence to request

- `manifest`: the ImageValidatingPolicy, including `attestors`, `attestations` and `validations`.
- `manifest`: `spec.validationConfigurations` if set, and `spec.credentials.secrets` for private registries.
- `manifest`: the approved signer identity and issuer, and the required predicate type, as stated by the owner.
- `offline-test`: results for one image signed by the approved signer with the required claim, one signed by another signer, and one missing the claim.
- `admission-trace`: a real admission response for a signed image, if available.

## Interpretation

1. Verification happens only through CEL validations that call `verifyImageSignatures(image, attestors)` and `verifyAttestationSignatures(image, attestation, attestors)`; both return counts, and `extractPayload(image, attestation)` exposes predicate content [KYV-E18]. A policy that declares an attestor but never calls it in a validation verifies nothing.
2. A count is not a requirement. The validation must compare the count to the number the requirement needs; a check that the count is non-negative or loosely compared does not enforce anything.
3. `validationConfigurations` fields `mutateDigest`, `verifyDigest` and `required` each default to true [KYV-E17]. Confirm they were not turned off, and note that digest handling changes what a tag-based reference resolves to.
4. Keyless identity matching is strict on subject and issuer by default; `subjectRegExp`, `issuerRegExp` or a CEL `subjectExpression` are alternatives [KYV-E17]. A broad regular expression widens the set of acceptable signers; read the pattern against the approved identity.
5. An in-toto attestation declares its type under `intoto.type` [KYV-E17]. A signature on an attestation of a different predicate type does not satisfy a requirement for the required type.
6. A signature check alone never satisfies an attestation requirement or a signer restriction [KYV-E18]. The attestation call and a predicate comparison, plus an identity that matches only the approved signer, are each separate things to find in the policy.
7. Private registry credentials are not taken from the admitted Pod's imagePullSecrets; they must be listed in `spec.credentials.secrets` [KYV-E19]. A policy that works against a public registry can fail or mis-verify against a private one.
8. What happens when the registry is unreachable at admission is not documented in the evidence set: treat it as unverified and route it to `kyverno-admission-reliability-agent`.
9. Whether the approved signer, issuer and trust root are the right ones belongs to `sigstore-cosign-supply-chain-review-agent`; keep only whether the policy enforces the stated requirement.

## Worked example (synthetic)

**Requester supplied**: ImageValidatingPolicy for `registry.example.invalid/payments/*`, requirement: images in `payments` must be signed by the release pipeline identity (a stated subject and issuer) and carry a SLSA provenance attestation.

**Agent derived**: The policy calls `verifyImageSignatures` with the approved attestor and fails closed when the count is zero. It has no call to `verifyAttestationSignatures`, and no `attestations` entry for the provenance type. An image signed by the approved signer but with no provenance predicate, or with a predicate of a different type, therefore passes while the provenance requirement is unmet. The provenance requirement is unmet: `blocked` for that requirement, `needs-review` overall.

**Still needed**: `manifest` of the attestation entry and the CEL validation that would check `extractPayload` content, or confirmation that the requirement was dropped.

**Decision today**: `blocked` / `blocked` for the provenance requirement; the signer requirement is `conditional` pending the identity test below.

**Cheapest discriminating check**: `offline-test` with three images: approved signer plus provenance (must pass), approved signer with no provenance (must fail), different signer (must fail).

**Developer-facing denial message**: "payments-image-trust: image must be signed by the release pipeline and carry a SLSA provenance attestation for its digest. Rebuild through the release pipeline or escalate to the supply-chain owner." It tells the developer what to produce and whom to ask.

**Escalation owner**: supply-chain owner for the signer and attestation requirement; platform security for the policy.

**Keep out of evidence**: signing keys, registry credentials, internal registry hostnames beyond the synthetic one.

## Counterexample

A correctly signed image from an unapproved signer passes because the identity uses a broad `subjectRegExp` that also matches the other signer's subject. Signature verification succeeded, but the signer restriction was not expressed: the policy proves "some matching identity signed this", not "the approved signer signed this". Strict `subject` and `issuer` [KYV-E17], or a tightly anchored expression checked in the negative test, closes the gap.

## Escalation threshold

Send trust-root, signer and attestation design questions to `sigstore-cosign-supply-chain-review-agent`. Send registry-outage or timeout questions to `kyverno-admission-reliability-agent`. Stop and ask the supply-chain owner when the requirement text and the policy disagree on which claim is mandatory.

## What would change the verdict

- A validation that calls `verifyAttestationSignatures` and compares `extractPayload` content to the required predicate type and fields [KYV-E18].
- A negative `offline-test` showing the unapproved signer is rejected.
- `spec.credentials.secrets` evidence for the private registry [KYV-E19].
- An `admission-trace` for a signed and an unsigned image.
