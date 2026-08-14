# Verification: reusable-evidence-qualification

## Reuse relation

Given one unchanged definition and qualification, submit successful observations for candidate
revisions A and B. Both exact CI gates open, the qualification id and fingerprint remain unchanged,
and no agent work item is produced.

## Subject confinement

An observation for revision A must not open revision B's gate. An artifact-bound release
observation must not apply to another digest, deployment or stage.

## Time relation

Use an injected clock. A production observation opens the canary gate before its expiry and closes
it at expiry without sleeping. The feedback identifies renewal rather than semantic re-judgment.

## Failure and drift

- A violated observation closes the exact gate and creates diagnosis work.
- A changed definition fingerprint invalidates the existing qualification and creates
  qualification work before a new execution can count.
- A context mismatch closes the gate without changing the observation.
- A challenge finding creates judgment work and prevents an otherwise successful observation from
  opening the gate.

## Falsifier comparison

The experiment records semantic judgment count and repository writes for two successful recurring
executions. The target is one qualification judgment and zero result commits. If either successful
rerun requires another judgment or repository edit, do not proceed to the service.
