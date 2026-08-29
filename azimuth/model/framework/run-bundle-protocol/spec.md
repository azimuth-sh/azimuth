# Spec: framework/run-bundle-protocol

## Claim: subjects-have-exact-identity
Criticality: routine

Azimuth SHALL bind every Run to one exact typed Subject and accountable execution provenance.

### Case: source-state-distinguishes-workspaces-and-candidates
GIVEN two workspace or CI states with the same display revision but different relevant content
WHEN Azimuth derives their Subject fingerprints
THEN the content fingerprints keep the Subjects distinct

### Case: deployed-state-distinguishes-operational-subjects
GIVEN an artifact, deployment, service or monitoring window
WHEN its mutable provider locator stays the same but deployed content changes
THEN the Subject fingerprint changes with the immutable content identity

### Case: historical-import-retains-original-subject
WHEN an older native execution is imported after it occurred
THEN the Run retains its concrete original Subject
AND import time and provider execution identity remain provenance

## Claim: actual-selection-is-bounded-by-plan
Criticality: routine

Azimuth SHALL freeze planned and actual semantic selection and SHALL fail closed on substituted or
additional targets.

### Case: completed-run-matches-the-plan
GIVEN a completed Run
WHEN Azimuth compares its actual context, targets, implementations and work units with the plan
THEN the two selections are exactly equal

### Case: partial-run-retains-an-explicit-subset
GIVEN a partial, cancelled or timed-out Run
WHEN some planned work was never selected
THEN actual selection is an explicit subset
AND omitted work creates no synthetic result

### Case: selection-substitution-invalidates-the-bundle
WHEN actual selection adds a target or changes a fingerprint, implementation, unit or context
THEN protocol verification rejects the bundle for acceptance

## Claim: execution-reduces-to-cardinal-results
Criticality: routine

Azimuth SHALL reduce activities, work units and ordered attempts to exactly cardinal terminal
Observations and Challenge Results without conflating their meaning.

### Case: retries-cannot-erase-adverse-results
GIVEN several attempts for one work unit
WHEN any Check attempt violates its proposition or any Challenger attempt finds an objection
THEN the terminal result preserves the violation or findings

### Case: incomplete-execution-is-not-positive
GIVEN missing work units or an unfinished selected activity
WHEN Azimuth reduces the execution
THEN the terminal result is inconclusive rather than satisfied or clean

### Case: one-activity-can-have-two-semantic-roles
GIVEN one physical fault activity used by a Check and a Challenger
WHEN Azimuth reduces the Run
THEN it produces one Observation for the Check
AND one separately targeted Challenge Result without implication between them

## Claim: bundle-history-is-immutable-and-deterministic
Criticality: routine

Azimuth SHALL identify Run bundles by versioned canonical fingerprints and SHALL represent changed
content only through a linear immutable correction chain.

### Case: exact-replay-is-idempotent
WHEN the same bundle content is verified more than once
THEN one bundle fingerprint represents the duplicate inputs

### Case: correction-is-a-full-next-revision
GIVEN a late report for an existing Run
WHEN a corrected bundle is produced
THEN it increments the revision and names the immediate predecessor fingerprint
AND preserves the Subject, context, plan and source-execution anchors

### Case: ambiguous-history-fails-closed
WHEN a bundle set has a gap, fork, cycle, conflicting revision or changed anchor
THEN protocol verification rejects the correction history

## Claim: run-protocol-is-service-independent
Criticality: routine

Azimuth SHALL verify and inspect normalized Run bundles locally without an adapter or Assurance
Service.

### Case: negative-facts-remain-valid-bundles
GIVEN a protocol-consistent Run with a violated Observation or Challenge findings
WHEN `azimuth run verify` evaluates the bundle
THEN verification succeeds without reinterpreting the negative fact as a protocol failure

### Case: inspection-is-deterministic-and-nonauthoritative
WHEN `azimuth run inspect` reads the same bundle set repeatedly
THEN text and JSON accounts remain deterministic
AND neither account claims current repository acceptance or Assurance State

### Case: ingest-remains-absent
WHEN a user requests `azimuth run ingest`
THEN the command remains unknown until the Run-ledger change implements that authority
