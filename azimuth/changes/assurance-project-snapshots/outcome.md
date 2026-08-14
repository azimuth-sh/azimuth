# Outcome: assurance-project-snapshots

Status: implemented, pending acceptance

## Result

Azimuth now exports one immutable assurance project snapshot from a complete, hole-free accepted
model. The current fixture yields 85 stable contracts from 90 claims; routine claims remain outside
the service. Contract fingerprints include claim and effective verification semantics plus
surfaces and area realization obligations. Exact model fingerprints continue to include source
relations and enumerated members.

The assurance service stores snapshots immutably and recomputes every content identity. Evidence
definitions use structured claim references and cannot be registered before their contract is
known. Observations cannot be stored for an unknown snapshot or one where their definition no
longer applies. Gate evaluation distinguishes missing provenance from contract drift and emits
focused snapshot-registration, definition-revision and qualification work.

The diagnostic client shows the latest imported snapshot, complete model identity, definition
contract, surface and obligated areas. It consumes service projections and contains no independent
applicability rules.

## Evidence executed

- Four CLI projection regressions passed: architectural and verification drift change a contract;
  set-like reordering does not; realization-body drift changes only the exact snapshot.
- The CLI-to-service wire-contract regression deserialized a real CLI projection and recomputed
  both snapshot and contract identities successfully.
- Eleven pure lifecycle cases passed, including missing snapshot and inapplicable-contract gates.
- The HTTP component lifecycle passed against real PostgreSQL. It covered immutable snapshot
  replay, definition-before-snapshot refusal, observation provenance refusal, qualification reuse
  across an unchanged contract and architectural drift requiring a revised definition.
- The Next.js diagnostic client passed strict TypeScript checking and a production build.
- A real fixture export produced 85 non-routine claim contracts from a zero-finding accepted model.
- The agent-tier worklist contained no stale or adverse non-routine judgment. The five unjudged
  entries are the expected routine claims, which owe no judgment.

## Departures

The proposal described inapplicable-contract work as requalification. Implementation reports both
`revise-definition` and `qualify-definition`, because a definition bound to an obsolete contract
cannot be repaired by attaching a fresh verdict to the same semantic record.

The wire format uses the service's camel-case JSON vocabulary. An early implementation emitted
snake-case fields from the dependency-free CLI; an executable cross-crate round-trip exposed and
removed that mismatch before the repository gate.

## Residual decisions

- Authenticate and sign snapshot provenance before production deployment. Recomputed hashes prove
  integrity of submitted content, not producer authority.
- Add a federated-project snapshot projection after one multi-repository assurance workflow needs
  it; the present command intentionally rejects partial `--only` accounts.
- Build a real CI publisher rather than making `azimuth check` perform network writes.
- Decide retention and supersession presentation once imported snapshot history grows beyond a
  diagnostic reference account.

## Measurements

- accepted claims: 90;
- exported claim contracts: 85;
- routine claims added to service ceremony: 0;
- repository mechanics reimplemented in the service: 0;
- pure lifecycle cases: 11;
- PostgreSQL-backed lifecycle cases: 1 composed public-API scenario;
- final accepted-model findings: 0.
