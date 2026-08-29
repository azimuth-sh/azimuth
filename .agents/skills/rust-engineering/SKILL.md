---
name: rust-engineering
description: Implement, refactor and review Rust in this repository with explicit domain types, straightforward control flow, deterministic behavior and fail-closed boundaries. Use for Rust work in tools/azimuth, services/assurance and tools/extractors/rust, especially ledger, projection, protocol, parser, adapter-host and PostgreSQL code. Pair with the applicable Azimuth change skill when the work changes accepted behavior; do not use it to invent unresolved product semantics.
---

# Engineer boring Rust

Write Rust whose correctness is visible from its types and control flow. Let the compiler provide fast feedback, but leave product authority in the repository model and approved change.

## Establish the boundary

1. Read `AGENTS.md`, the nearest `Cargo.toml`, affected source and the authoritative contract or model package before editing.
2. Read the approved change when behavior is changing. If no artifact decides a material semantic question, stop at that question instead of encoding a convenient answer.
3. Identify the invariant, owning module, inputs, outputs, failure classes and durable effects.
4. Preserve the repository authority order. Do not use docs or this skill to override code, contracts, standards or accepted model intent.
5. Keep consumers, provider adapters and external orchestration outside the authoritative Rust boundary unless the approved change explicitly brings them in.

## Model the domain explicitly

- Represent distinct identities and states with named structs and enums, not interchangeable strings, tuples or boolean combinations.
- Prefer closed enums and exhaustive `match` for vocabularies owned by Azimuth. Preserve open strings only where the contract deliberately leaves an extension point open.
- Parse and validate untrusted representations once at the boundary. Pass validated domain values inward.
- Make invalid transitions unrepresentable where a small local type can do so. Do not construct a type-state framework for a one-function invariant.
- Keep domain functions synchronous, deterministic and side-effect free where practical. Inject time, identifiers, filesystem input and external results.
- Use `BTreeMap`, `BTreeSet` or explicit sorting when order reaches a fingerprint, protocol, diagnostic or persisted decision.
- Keep fingerprints, provenance and exact Subject or Claim identity explicit. Never infer semantic identity from a path, line number, ambient process state or provider label.

## Prefer concrete code

- Start with concrete structs, functions and modules.
- Add a trait only for an actual boundary with multiple implementations, a deliberate test seam, or an accepted extension point. State which case requires it.
- Add a generic parameter only when the same algorithm genuinely operates over several types.
- Prefer a repeated three-line transformation to a premature framework.
- Avoid service locators, generic repositories, event buses hidden behind broad traits, macro DSLs, builder pyramids and configurable pipelines without a current consumer.
- Keep functions small enough that failure and state transition paths are visible, but do not split sequential logic into one-line indirection.
- Use comments for invariants, authority and surprising constraints. Let names explain mechanics.
- Keep public APIs narrow. Default items and fields to private, and expose behavior rather than storage layout.

## Control effects and concurrency

- Keep async at HTTP, database, process and channel boundaries. Do not make pure domain logic async.
- Give every spawned task an owner, completion path, deadline and cancellation behavior.
- Do not detach work whose completion changes an authoritative result.
- Bound queues, streams, response sizes, retries and parallelism. Make overload and timeout outcomes explicit.
- Preserve atomic publication: validate complete content before committing or replacing output.
- Treat cancellation and partial provider output as named outcomes; never translate absence into success.
- Do not use `unsafe` unless an approved design requires it and states the invariant that makes it sound. Prefer a safe dependency or a simpler design.

## Handle errors plainly

- Return errors from request, storage, protocol and execution paths. Do not `unwrap` or `expect` on external input, database state, clocks, process results or configuration.
- Reserve `expect` for a local invariant that construction already proves and make the message name that invariant. Prefer removing even that expectation when the return type stays simple.
- Use a small concrete error enum when callers must distinguish outcomes. Use contextual strings at an application boundary when no caller branches on the type.
- Preserve the repository's exit and protocol failure classes. Do not collapse semantic mismatch, invalid shape and transport failure for convenience.
- Make every parse failure name the file, line and expected form.
- Log stable identifiers and outcome classes, not secrets or unrestricted payloads.

## Preserve repository boundaries

- Keep `tools/azimuth` dependency-free unless an accepted change explicitly revises that constraint.
- Put AST, compiler, call-graph and schema analysis in an ecosystem extractor; core reads manifests.
- Never discover adapter executables through `PATH`, invoke a shell or inherit ambient environment.
- Treat adapters as bounded provider translations, not semantic authorities, daemons or supervisors.
- Preserve valid adverse facts. A violated Observation, finding, partial Run or clean Challenge is not a transport failure and is not evidence of a stronger proposition.
- Do not give repository declarations, standalone bundles or legacy service records authority they do not currently possess.

## Keep a modular service honest

- Give each module one domain responsibility and ownership of its writes.
- Prevent modules from changing another module's tables directly. Cross a typed command, event or query boundary.
- Store immutable facts append-only. Express correction by an explicit linked record, not mutation.
- Make ingestion idempotent on declared identity and reject conflicting content under the same id.
- Compute projections from accepted facts. Do not expose authoritative derived state as editable CRUD fields.
- Keep transactions short and make the atomicity boundary match the domain decision.
- Keep SQL explicit and inspectable. Avoid a generic persistence layer that erases uniqueness, ordering, locking or conflict semantics.
- Let the application/server crate compose modules and infrastructure. Do not let transport types become the domain model.

## Use the compiler as the feedback loop

1. Make the smallest coherent edit.
2. Format only the affected Rust package or files.
3. Run the narrowest applicable `cargo check` and fix the first causal diagnostic rather than patching downstream symptoms.
4. Run focused Clippy when the package already supports it; do not add broad allowances to silence a design problem.
5. Inspect the final diff for accidental public surface, hidden nondeterminism and needless abstraction.
6. Run Azimuth's own relevant validation/report/export commands when the change affects their inputs or outputs.

Do not create, run or offer tests unless the user explicitly asks, as required by repository guidance. Record an observed coverage gap without filling it. When tests are explicitly requested, prefer behavior at the public boundary and synthetic fixtures over tests coupled to private function shape.

## Review before handoff

Confirm that:

- every new abstraction has a current consumer and named purpose;
- states and failure classes remain distinguishable;
- deterministic output is independent of hash order, wall clock and ambient environment;
- async work is bounded and cancellation-safe;
- database writes preserve immutability, idempotency and transaction intent;
- diagnostics expose enough identity to reconstruct a decision without leaking sensitive data;
- the code can be understood during an incident without reconstructing a type trick;
- the implementation does not claim authority or behavior beyond the approved change.
