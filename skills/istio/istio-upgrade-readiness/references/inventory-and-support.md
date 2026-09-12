# Inventory and support

## UPGRADE-01: inventory without topology assumptions

Record exact Istio control/data-plane versions, connected revisions, Kubernetes version, installation ownership, Helm releases or istioctl configuration, image provenance, CRD bundle, gateways, waypoints, CNI, ztunnel and extensions. Identify multi-cluster/network boundaries only from evidence.

Obtain the proposed target and actual source-to-target release notes. Confirm supported Kubernetes versions, installation methods, feature state, known security advisories and intermediate transition requirements from release-specific primary sources. Do not freeze a mutable support table into a timeless rule.

## Evidence limits

A list of deployments is not proof of all connected proxy versions. A desired image tag is not necessarily the running binary. Failed/Forbidden inventory requires needs-review. No source or target version means no upgrade-readiness approval, but a preparation checklist can still be produced.

## Readiness dimensions

Report support, compatibility, availability, security, operational ownership, test coverage and rollback separately. Never convert missing blockers into a percentage chance of success. Unknown is not a positive score.

Source: SKEW and installation-specific documentation in official-sources.md. Test: UPGRADE-01.
