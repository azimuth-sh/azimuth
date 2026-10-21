# Intent delta: framework/adapter-capability-protocol

## Replace requirement: semantic-planning-uses-the-complete-model
From: sha256:f4968e8c2ffa794b10c1c4be4c32a8778ebf814b45e380b790f3143b026a2e55
Because: Claim Review and Claim Health replace removed review targets while Check execution remains independent of approval.
Criticality: routine

Azimuth SHALL derive Check and machine Challenge execution plans from one complete unselected current model without delegating semantic resolution to an adapter.

### Scenario: planning-resolves-complete-check-identity

Given a request naming sorted Checks, finite work units and capability addresses, when the Run is planned, then it pins the complete-model fingerprint and each Check's current fingerprint and includes every stable implementation of each selected Check.

### Scenario: check-execution-does-not-require-current-decisions

Given a selected Check without a current accepted Claim Review or Claim Health, when execution is planned, then the Check may still execute against an exact Subject. Execution planning does not establish method credibility, contribution sufficiency or Claim support.

### Scenario: challenge-planning-resolves-current-decisions

Given a machine Challenger request with finite work units and an explicit capability address, when the Run is planned, then it resolves an exact retained current eligible Claim Review or Claim Health, pins the declaration and implementation configuration, and freezes semantic scope and its source-resolution report before adapter translation. Historical review kinds do not substitute for a current target.
