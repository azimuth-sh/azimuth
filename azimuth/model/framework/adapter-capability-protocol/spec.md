# Spec: framework/adapter-capability-protocol

## Claim: adapter-configuration-is-explicit-and-pinned
Criticality: routine

Azimuth SHALL invoke only explicitly configured adapters whose protocol, description, limits and
capabilities are pinned before provider interaction.

### Case: executable-resolution-never-searches-path
- Event: an adapter executable is configured
- Required outcome: its locator is absolute or relative to the configuration file
- Additional condition or outcome: Azimuth invokes it without a shell or `PATH` discovery

### Case: executable-and-resources-match-pinned-content
- Event: an executable or declared resource content differs from its configured digest
- Required outcome: Azimuth rejects the adapter before spawning it

### Case: child-environment-is-an-explicit-allowlist
- Event: Azimuth invokes an adapter
- Required outcome: it clears the ambient child environment
- Additional condition or outcome: it passes only exact configured non-secret literals
- Additional condition or outcome: it performs no ambient inheritance, secret reference or interpolation

### Case: configured-content-is-staged-from-one-byte-stream
- Event: core prepares an executable, declared resource or import input
- Required outcome: it copies and hashes one opened byte stream into a private invocation stage
- Additional condition or outcome: it invokes or supplies only the staged content after digest validation
- Additional condition or outcome: the stage makes no filesystem or network sandbox claim

### Case: configured-description-drift-fails-closed
- Event: a running adapter reports an id, version, build, descriptor or capability dictionary
- Additional condition or outcome: the reported value differs from configuration
- Required outcome: Azimuth rejects the adapter before accepting a provider result

### Case: capability-identity-tracks-semantics
- Event: adapter content, adapter identity or capability-local declarations and settings change
- Required outcome: the capability fingerprint changes
- Additional condition or outcome: adapter-wide environment or process-limit changes affect configuration identity instead
- Additional condition or outcome: locator relocation or prose does not change either semantic fingerprint

## Claim: semantic-planning-uses-the-complete-model
Criticality: routine

Azimuth SHALL derive Check and Challenge execution plans from one complete unselected current
model without delegating semantic resolution to an adapter.

### Case: planning-resolves-complete-check-identity
- Context: a request names sorted Checks, finite work units and capability addresses
- Event: Azimuth plans the Run
- Required outcome: it pins the complete-model fingerprint and each Check's current fingerprint
- Additional condition or outcome: it includes every stable implementation of each selected Check

### Case: check-execution-does-not-require-current-decisions
- Event: a selected Check has no current Method Qualification or Applicability Decision, or its request context differs from a binding
- Required outcome: Azimuth may still plan that Check execution
- Additional condition or outcome: it does not infer evidentiary applicability

### Case: challenge-planning-resolves-current-decisions
- Context: a request names Challenge Plans, finite work units and explicit capability addresses
- Event: Azimuth plans the Run
- Required outcome: it resolves exact current accepted Method Qualification, Applicability Decision or Claim Judgment targets
- Additional condition or outcome: it freezes traceability-derived semantic scope before adapter translation

## Claim: launch-plans-bind-capability-routing
Criticality: routine

Azimuth SHALL bind an exact Subject, operation and complete Run-bundle semantic plan to explicit
capability routes in one canonical launch plan.

### Case: capability-substitution-changes-launch-identity
- Event: a route substitutes a capability while the Run-bundle semantic plan stays unchanged
- Required outcome: the launch fingerprint changes
- Additional condition or outcome: the derived Run id changes

### Case: one-adapter-routes-several-capabilities
- Context: one configured adapter exposes several capabilities
- Event: a Run needs more than one supported route
- Required outcome: one launch plan may route its selections through those capabilities
- Additional condition or outcome: every route retains its exact capability fingerprint

## Claim: adapter-invocation-is-strict-and-bounded
Criticality: routine

Azimuth SHALL invoke an adapter as a bounded short-lived process with one strict JSON request and
exactly one strict JSON response.

### Case: process-resources-are-bounded
- Event: an adapter exceeds its timeout, standard-output limit or standard-error limit
- Required outcome: Azimuth terminates the exchange and reports a transport failure

### Case: output-streams-cannot-deadlock-each-other
- Event: an adapter fills standard output and standard error concurrently
- Required outcome: Azimuth drains both capped streams without waiting for either stream to close first

### Case: transport-status-is-not-a-product-outcome
- Event: an adapter exits nonzero, times out or returns a malformed exchange
- Required outcome: Azimuth does not reinterpret exit status or diagnostics as an Observation or Challenge Result

### Case: timed-out-execute-is-not-retried-implicitly
- Event: an execute operation times out after native work may have started
- Required outcome: Azimuth does not automatically retry the operation

### Case: unavailable-process-group-isolation-fails-before-spawn
- Context: a fresh process group and bounded core exchange are required
- Event: a host cannot establish them before adapter code runs
- Required outcome: Azimuth rejects the invocation before spawn as an exit-one transport failure
- Additional condition or outcome: it creates no adapter process or output bundle

## Claim: execute-validates-normalized-run-semantics
Criticality: routine

Azimuth SHALL accept an executed adapter response only after validating its request identity,
provenance, actual selection and complete Run bundle.

### Case: execution-output-is-atomic-after-validation
- Event: returned execution identity or selection differs from the launch plan
- Required outcome: Azimuth rejects the response
- Additional condition or outcome: it leaves no output bundle

### Case: negative-execution-facts-remain-successful-exchanges
- Event: a valid response reports violation, findings, partial, cancellation or timeout
- Required outcome: Azimuth preserves that fact and exits successfully

### Case: dual-role-results-remain-separate
- Context: one adapter activity supports Check and Challenge capability routes
- Event: it returns both result kinds for one physical activity
- Required outcome: the bundle retains a separate Observation and Challenge Result
- Additional condition or outcome: neither result implies the other

## Claim: import-inputs-are-content-addressed
Criticality: routine

Azimuth SHALL bind every native import input to the exact file content core supplied to the
configured adapter.

### Case: core-identifies-import-by-content
- Event: a caller supplies an import input file
- Required outcome: core computes its digest and byte size before adapter invocation
- Additional condition or outcome: the adapter response repeats that exact input identity
- Additional condition or outcome: returned bundle provenance retains the sorted input identities

### Case: native-locators-cannot-replace-input-identity
- Event: a native report has a mutable path, URI or provider execution id
- Required outcome: those values remain provenance
- Additional condition or outcome: they do not substitute for the core-computed content identity

### Case: import-corrections-require-verified-predecessors
- Event: an import or execution supplies predecessor bundles
- Required outcome: core validates their complete linear correction chain before adapter invocation
- Additional condition or outcome: the request binds the sorted revision and bundle-fingerprint identities
- Additional condition or outcome: it carries the complete verified terminal predecessor for exact anchor preservation
- Additional condition or outcome: the response is exactly the next revision correcting the terminal predecessor

## Claim: capability-classes-and-provider-identities-stay-separate
Criticality: routine

Azimuth SHALL use a closed semantic capability-class vocabulary with open configured-adapter
capability addresses, separate open provider-family identities and open challenge forms.

### Case: one-capability-supports-several-classes
- Event: one provider capability can execute or import more than one semantic role
- Required outcome: its declaration may name several supported capability classes

### Case: model-extraction-is-not-a-run-operation
- Event: configuration declares a `model.extract` capability
- Required outcome: the E Run command surface does not plan or invoke it as a Run operation

## Claim: runtime-command-results-have-stable-exit-classes
Criticality: routine

Azimuth SHALL distinguish valid execution facts from semantic or transport mismatches and from
input schema failures across the adapter command surface.

### Case: honest-negative-facts-exit-zero
- Event: execute or import returns a protocol-consistent adverse or incomplete Run fact
- Required outcome: the command exits zero and writes the validated bundle atomically

### Case: invalid-exchanges-produce-no-output
- Event: a mismatch exits one or a schema failure exits two
- Required outcome: the command writes no output bundle

### Case: durable-ingest-remains-absent
- Event: a user requests `azimuth run ingest`
- Required outcome: the command remains unknown until the Run-ledger change implements that authority
