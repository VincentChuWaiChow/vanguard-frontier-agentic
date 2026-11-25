# ImageValidatingPolicy starter kit

An ImageValidatingPolicy skeleton for the synthetic invoice service in namespace `payments` that requires images from `registry.example.invalid/payments/*` to be signed by the release pipeline and to carry a signed provenance attestation. Files: `policy.yaml`, `resources.yaml`, `kyverno-test.yaml`.

## What it demonstrates

- The policy loads and compiles with the pinned CLI, including the attestor and attestation declarations and the verification calls [KYV-E18].
- Explicit `failurePolicy` and `webhookConfiguration.timeoutSeconds`, because the CRD states no failurePolicy default for this kind [KYV-E36].
- Strict keyless identity (subject and issuer) rather than a broad regular expression [KYV-E17].
- Separate validations for the signature and for the attestation, so a signature check alone does not satisfy the attestation requirement [KYV-E18].
- Private-registry pull secrets belong in `spec.credentials.secrets`; the admitted Pod's imagePullSecrets are not used [KYV-E19].
- The one offline result pins the out-of-scope path: a Pod whose image is outside `matchImageReferences` reports `pass`, not `skip`.

## Verified with

- The pinned kyverno CLI v1.20.0, offline: load and compile, and the out-of-scope result only.
- Not verified: the signer path, the attestation path, registry access, the webhook and the controller. These need a `controlled-test` in a disposable cluster before promotion to Deny.
- The predicate-content check is commented out because the payload path is unverified: one published example reads fields at the top level of `extractPayload`, and whether predicate fields sit there or under `.predicate` is not settled [KYV-E51].

## Adapt it

1. Replace every `TEMPLATE` marker (control name, image globs, identities, attestation type).
2. Keep each field explicit; set `failurePolicy` and the timeout deliberately.
3. Keep one in-scope signed image, one unsigned image and one out-of-scope image in your controlled test, and keep the out-of-scope case here.
4. Rerun the kit with the pinned CLI, then run the signer and attestation paths in a disposable cluster; enable the predicate check only after reading one known-good attestation.

## Traps this kit avoids

- Reading the offline `pass` as "an image was verified": an out-of-scope image passes because the expressions run over an empty image list. A `pass` means nothing failed.
- A hyphenated attestor name such as `attestors.release-pipeline` fails to compile because CEL reads the hyphen as subtraction; the kit uses `releasePipeline`.
- Treating a signature check as the attestation check [KYV-E18].
- Trusting the predicate path in the commented-out check without evidence [KYV-E51].
