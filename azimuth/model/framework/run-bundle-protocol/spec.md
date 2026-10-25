# Spec: framework/run-bundle-protocol

## Requirement: subjects-have-exact-identity
Title: Verification Targets have exact identity

Azimuth SHALL bind every Run to one exact typed Verification Target and accountable execution provenance.

### Scenario: source-state-distinguishes-workspaces-and-candidates
- Context: two workspace or CI states with the same display revision but different relevant content
- Event: Azimuth derives their Verification Target fingerprints
- Required outcome: the content fingerprints keep the Verification Targets distinct

### Scenario: deployed-state-distinguishes-operational-subjects
- Context: an artifact, deployment, service or monitoring window
- Event: its mutable provider locator stays the same but deployed content changes
- Required outcome: the Verification Target fingerprint changes with the immutable content identity

### Scenario: historical-import-retains-original-subject
- Event: an older native execution is imported after it occurred
- Required outcome: the Run retains its concrete original Verification Target
- Additional condition or outcome: import time and provider execution identity remain provenance

## Requirement: actual-selection-is-bounded-by-plan
Title: Actual selection is bounded by plan

Azimuth SHALL freeze planned and actual semantic selection and SHALL fail closed on substituted or
additional targets.

### Scenario: completed-run-matches-the-plan
- Context: a completed Run
- Event: Azimuth compares its actual context, targets, implementations and work units with the plan
- Required outcome: the two selections are exactly equal

### Scenario: partial-run-retains-an-explicit-subset
- Context: a partial, cancelled or timed-out Run
- Event: some planned work was never selected
- Required outcome: actual selection is an explicit subset
- Additional condition or outcome: omitted work creates no synthetic result

### Scenario: selection-substitution-invalidates-the-bundle
- Event: actual selection adds a target or changes a fingerprint, implementation, unit or context
- Required outcome: protocol verification rejects the bundle for acceptance

## Requirement: execution-reduces-to-cardinal-results
Title: Execution reduces to cardinal results

Azimuth SHALL reduce activities, work units and ordered attempts to exactly cardinal terminal
Check Results and Challenge Results without conflating their meaning.

### Scenario: retries-cannot-erase-adverse-results
- Context: several attempts for one work unit
- Event: any Check attempt violates its proposition or any Verification Challenge attempt finds an objection
- Required outcome: the terminal result preserves the violation or findings

### Scenario: incomplete-execution-is-not-positive
- Context: missing work units or an unfinished selected activity
- Event: Azimuth reduces the execution
- Required outcome: the terminal result is inconclusive rather than satisfied or clean

### Scenario: one-activity-can-have-two-semantic-roles
- Context: one physical fault activity used by a Check and a Verification Challenge
- Event: Azimuth reduces the Run
- Required outcome: it produces one Check Result for the Check
- Additional condition or outcome: one separately targeted Challenge Result without implication between them

## Requirement: bundle-history-is-immutable-and-deterministic
Title: Bundle history is immutable and deterministic

Azimuth SHALL identify Run bundles by versioned canonical fingerprints and SHALL represent changed
content only through a linear immutable correction chain.

### Scenario: exact-replay-is-idempotent
- Event: the same bundle content is verified more than once
- Required outcome: one bundle fingerprint represents the duplicate inputs

### Scenario: correction-is-a-full-next-revision
- Context: a late report for an existing Run
- Event: a corrected bundle is produced
- Required outcome: it increments the revision and names the immediate predecessor fingerprint
- Additional condition or outcome: preserves the Verification Target, Execution Context, plan and source-execution anchors

### Scenario: ambiguous-history-fails-closed
- Event: a bundle set has a gap, fork, cycle, conflicting revision or changed anchor
- Required outcome: protocol verification rejects the correction history

## Requirement: run-protocol-is-service-independent
Title: Run protocol is service independent

Azimuth SHALL verify and inspect normalized Run bundles locally without an adapter or Assurance
Service.

### Scenario: negative-facts-remain-valid-bundles
- Context: a protocol-consistent Run with a violated Check Result or Challenge findings
- Event: `azimuth run verify` evaluates the bundle
- Required outcome: verification succeeds without reinterpreting the negative fact as a protocol failure

### Scenario: inspection-is-deterministic-and-nonauthoritative
- Event: `azimuth run inspect` reads the same bundle set repeatedly
- Required outcome: text and JSON accounts remain deterministic
- Additional condition or outcome: neither account claims current repository acceptance or Assurance State

### Scenario: ingest-remains-absent
- Event: a user requests `azimuth run ingest`
- Required outcome: the command remains unknown until the Run-ledger change implements that authority
