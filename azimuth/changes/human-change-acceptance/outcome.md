# Outcome: Record human acceptance of non-routine Requirements

Status: accepted

## Delivered

A change directory may carry a strict `acceptance.json`. `change finalize` and `change archive` accept only Requirements the change adds, remove exactly their `missing-requirement-assurance` findings and include the decider, date and Requirement set in the finalization receipt. Archived records keep applying to `azimuth validate --scope acceptance` and to federated project finalization. Azimuth contacts no Assurance State Service.

## Departures

None.

## Verification performed

The release build succeeds. Per repository rules no tests were written or run; the behavior was checked by running the built CLI against a consumer change.

## Residual decisions

An acceptance has no expiry, revocation or second approver. Who may write the record is a team convention, not enforced by the tool.
