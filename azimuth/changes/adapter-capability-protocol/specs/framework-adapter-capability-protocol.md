# Intent delta: framework/adapter-capability-protocol

## Add requirement: adapter-configuration-is-explicit-and-pinned
Criticality: routine

Azimuth SHALL invoke only explicitly configured adapters whose protocol, description, limits and
capabilities are pinned before provider interaction.

### Add scenario: executable-resolution-never-searches-path
WHEN an adapter executable is configured
THEN its locator is absolute or relative to the configuration file
AND Azimuth invokes it without a shell or `PATH` discovery

### Add scenario: executable-and-resources-match-pinned-content
WHEN an executable or declared resource content differs from its configured digest
THEN Azimuth rejects the adapter before spawning it

### Add scenario: child-environment-is-an-explicit-allowlist
WHEN Azimuth invokes an adapter
THEN it clears the ambient child environment
AND it passes only exact configured non-secret literals
AND it performs no ambient inheritance, secret reference or interpolation

### Add scenario: configured-content-is-staged-from-one-byte-stream
WHEN core prepares an executable, declared resource or import input
THEN it copies and hashes one opened byte stream into a private invocation stage
AND it invokes or supplies only the staged content after digest validation
AND the stage makes no filesystem or network sandbox claim

### Add scenario: configured-description-drift-fails-closed
WHEN a running adapter reports an id, version, build, descriptor or capability dictionary
AND the reported value differs from configuration
THEN Azimuth rejects the adapter before accepting a provider result

### Add scenario: capability-identity-tracks-semantics
WHEN pinned behavior or declared capability semantics change
THEN the capability fingerprint changes
AND locator relocation, prose or secret values do not change that fingerprint

## Add requirement: semantic-planning-uses-the-complete-model
Criticality: routine

Azimuth SHALL derive Check execution plans from one complete unselected current model without
delegating semantic resolution to an adapter.

### Add scenario: planning-resolves-complete-check-identity
GIVEN a request names sorted Checks, finite work units and capability addresses
WHEN Azimuth plans the Run
THEN it pins the complete-model fingerprint and each Check's current fingerprint
AND it includes every stable implementation of each selected Check

### Add scenario: check-execution-does-not-require-current-qualification
WHEN a selected Check has no current Qualification or its request context differs from a binding
THEN Azimuth may still plan that Check execution
AND it does not infer evidentiary applicability

### Add scenario: challenge-planning-remains-unresolved
WHEN the E planner receives a Check planning request
THEN it emits an empty Challenge selection
AND it does not resolve Challenge Plans, Qualifications or Claim Judgments

## Add requirement: launch-plans-bind-capability-routing
Criticality: routine

Azimuth SHALL bind an exact Subject, operation and complete D46 semantic plan to explicit
capability routes in one canonical launch plan.

### Add scenario: capability-substitution-changes-launch-identity
WHEN a route substitutes a capability while the D46 semantic plan stays unchanged
THEN the launch fingerprint changes
AND the derived Run id changes

### Add scenario: one-adapter-routes-several-capabilities
GIVEN one configured adapter exposes several capabilities
WHEN a Run needs more than one supported route
THEN one launch plan may route its selections through those capabilities
AND every route retains its exact capability fingerprint

## Add requirement: adapter-invocation-is-strict-and-bounded
Criticality: routine

Azimuth SHALL invoke an adapter as a bounded short-lived process with one strict JSON request and
exactly one strict JSON response.

### Add scenario: process-resources-are-bounded
WHEN an adapter exceeds its timeout, standard-output limit or standard-error limit
THEN Azimuth terminates the exchange and reports a transport failure

### Add scenario: output-streams-cannot-deadlock-each-other
WHEN an adapter fills standard output and standard error concurrently
THEN Azimuth drains both capped streams without waiting for either stream to close first

### Add scenario: transport-status-is-not-a-product-outcome
WHEN an adapter exits nonzero, times out or returns a malformed exchange
THEN Azimuth does not reinterpret exit status or diagnostics as an Observation or Challenge Result

### Add scenario: timed-out-execute-is-not-retried-implicitly
WHEN an execute operation times out after native work may have started
THEN Azimuth does not automatically retry the operation

## Add requirement: execute-validates-normalized-run-semantics
Criticality: routine

Azimuth SHALL accept an executed adapter response only after validating its request identity,
provenance, actual selection and complete D46 Run bundle.

### Add scenario: execution-output-is-atomic-after-validation
WHEN returned execution identity or selection differs from the launch plan
THEN Azimuth rejects the response
AND it leaves no output bundle

### Add scenario: negative-execution-facts-remain-successful-exchanges
WHEN a valid response reports violation, findings, partial, cancellation or timeout
THEN Azimuth preserves that fact and exits successfully

### Add scenario: dual-role-results-remain-separate
GIVEN one adapter activity supports Check and Challenge capability routes
WHEN it returns both result kinds for one physical activity
THEN the bundle retains a separate Observation and Challenge Result
AND neither result implies the other

## Add requirement: import-inputs-are-content-addressed
Criticality: routine

Azimuth SHALL bind every native import input to the exact file content core supplied to the
configured adapter.

### Add scenario: core-identifies-import-by-content
WHEN a caller supplies an import input file
THEN core computes its digest and byte size before adapter invocation
AND the adapter response repeats that exact input identity
AND returned bundle provenance retains the sorted input identities

### Add scenario: native-locators-cannot-replace-input-identity
WHEN a native report has a mutable path, URI or provider execution id
THEN those values remain provenance
AND they do not substitute for the core-computed content identity

### Add scenario: import-corrections-require-verified-predecessors
WHEN an import or execution supplies predecessor bundles
THEN core validates their complete linear correction chain before adapter invocation
AND the request binds the sorted revision and bundle-fingerprint identities
AND it carries the complete verified terminal predecessor for exact anchor preservation
AND the response is exactly the next revision correcting the terminal predecessor

## Add requirement: capability-classes-and-provider-identities-stay-separate
Criticality: routine

Azimuth SHALL use a closed semantic capability-class vocabulary with open configured-adapter
capability addresses, separate open provider-family identities and open challenge forms.

### Add scenario: one-capability-supports-several-classes
WHEN one provider capability can execute or import more than one semantic role
THEN its declaration may name several supported capability classes

### Add scenario: model-extraction-is-not-a-run-operation
WHEN configuration declares a `model.extract` capability
THEN the E Run command surface does not plan or invoke it as a Run operation

## Add requirement: runtime-command-results-have-stable-exit-classes
Criticality: routine

Azimuth SHALL distinguish valid execution facts from semantic or transport mismatches and from
input schema failures across the adapter command surface.

### Add scenario: honest-negative-facts-exit-zero
WHEN execute or import returns a protocol-consistent adverse or incomplete Run fact
THEN the command exits zero and writes the validated bundle atomically

### Add scenario: invalid-exchanges-produce-no-output
WHEN a mismatch exits one or a schema failure exits two
THEN the command writes no output bundle

### Add scenario: durable-ingest-remains-absent
WHEN a user requests `azimuth run ingest`
THEN the command remains unknown until the Run-ledger change implements that authority
