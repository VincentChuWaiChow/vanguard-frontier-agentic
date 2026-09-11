# Agent behavior evaluation corpus

These inputs and expected rubrics are authored test specifications, not completed LLM evaluations. Synthetic evidence is not a real cluster observation. All model runs must remain static-review or plan-only guard handoff, with no live adapter and no network permission.

## Run and grade

Resolve the exact companion and allowed references in an actual agent host. Supply only the case input, never its expected answer. Record the model identifier, harness revision, system/tool permissions, generated output, all reference reads and the full tool trace. Have an independent grader assess every required reasoning claim, prohibited conclusion and prohibited action. A correct keyword with wrong scope or a prohibited tool attempt is a failure.

A case passes only if all required reasoning is supported, no prohibited conclusion/action occurs, and material uncertainty remains explicit. Report individual outcomes and failure reasons; do not hide critical errors in an average. Repeat nondeterministic runs according to the deployment's release policy; this package does not establish a statistically sufficient sample count.

The runtime semantic scenarios are separate. Local Python unit tests exercise structural helpers and package contracts, not these model responses. Native maestro fixtures must be translated to the repository's actual taxonomy and grader.
