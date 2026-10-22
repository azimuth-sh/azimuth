# Change: Record human acceptance of non-routine Requirements

Status: accepted and complete

Intent delta: none
Because: the change adds a recorded human decision at the existing finalize and archive boundary; no accepted Requirement of the framework changes.

## Problem

`azimuth change finalize` and `archive` clear a compact non-routine Requirement only when the local snapshot holds an accepted Requirement Review and a supported exact-target Requirement Status. Requirement Status is computed by the Assurance State Service from applicable evidence and is never authored, so the gate could not be satisfied without writing a positive status by hand. The first federated change with critical Requirements (`private-api-service-ingress` in a consumer project) was blocked at finalization although its reviews were accepted, its release was published and accepted in the service, and both Requirements computed `supported` for the deployed subject.

## Outcome

Acceptance of non-routine Requirements becomes an explicit, attributable human decision recorded in the change, not a derived fact. Azimuth does not contact an Assurance State Service to accept a change.

## Scope

- Add `acceptance.json` to a change directory: decider, date, the accepted Requirements the change adds, and a free-text basis.
- `change finalize` and `change archive` apply the record to the `missing-requirement-assurance` findings of exactly those Requirements and list the record in the finalization receipt.
- Archived records keep applying to `azimuth validate --scope acceptance` and to federated project finalization.

Excluded: any ASS client, automatic acceptance, other finding kinds, expiry or revocation, and authorship rules beyond the strict record.

## Completion conditions

- A strict record clears only the named Requirements' missing-assurance findings and nothing else.
- A record naming a Requirement the change does not add, or an invalid record, fails finalization.
- Editing or removing the record makes the finalization receipt stale.
