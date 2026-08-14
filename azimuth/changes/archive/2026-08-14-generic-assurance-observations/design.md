# Design: generic-assurance-observations

## Observation boundary

One observation is an immutable execution account. It owns the tool identity, report, configuration
inputs, observation time, optional expiry and payload fingerprint exactly once. Its bindings carry
the semantic interpretation because a single execution can answer several different propositions.

An `evidence` binding declares one claim-specific assertion, `satisfied` or `violated`, and the
scope, quantification and oracle of the observation. The loader projects it into the existing
`Covers` relation. This keeps checks and standards independent of whether evidence came from a code
tag, a manual tracker, a load tool or a chaos platform.

A `challenge` binding declares one claim-specific review question, `clean`, `findings` or
`inconclusive`, and the realization, evidence or mechanism subjects the tool examined. It enters the
judgment fingerprint but never `Covers`. Subject identity accepts the stable federated
`area|kind|address` key and the legacy `file#site|lang` key emitted before repository observation.

## Failure boundaries

- Duplicate observation ids fail because their execution identity is ambiguous.
- An evidence binding without an assertion, complete form, outcome, observation instant or expiry
  fails before it can count as coverage.
- A challenge with no subjects, an unknown claim or an unresolved subject is a hole.
- A failed evidence binding remains visible as failed evidence; it is never omitted.
- A broad tool run has no implicit claim scope. Importers derive bindings from existing linkage or
  consume explicit claim assertions.

## Extension boundary

Native adapters own schema and status interpretation. They fail closed on unknown native formats,
then emit the provider-neutral observation. The Rust core knows the two semantic roles but no tool
statuses, mutation scores, SARIF levels, k6 metrics or Chaos Mesh resource kinds. Tool-specific
detail remains in the fingerprinted report and optional opaque payload.
