# Spec: system/assurance

## Requirement: service-performance
Criticality: standard

The service SHALL remain within its declared latency and error-rate objectives under expected load.

### Scenario: latency-objective
GIVEN the declared expected workload
WHEN the load interval completes
THEN p95 latency is below 300 milliseconds

### Scenario: error-rate-objective
GIVEN the declared expected workload
WHEN the load interval completes
THEN the error rate is below 0.5 percent

## Requirement: broker-loss-resilience
Criticality: standard

The service SHALL degrade visibly during broker loss and recover queued work after restoration.

### Scenario: degraded-service-is-bounded
GIVEN the broker connection is interrupted
WHEN requests arrive during the interruption
THEN failures remain below the declared degraded-service threshold

### Scenario: queued-work-recovers
GIVEN work accumulated while the broker was unavailable
WHEN the broker connection is restored
THEN the queue drains within 120 seconds

### Scenario: interruption-raises-alert
GIVEN the broker connection remains interrupted
WHEN the backlog exceeds its alert interval
THEN the backlog alert arrives within 90 seconds

## Requirement: static-analysis-context
Criticality: routine

Static analysis MAY challenge claims realized in an analyzed artifact without becoming their
covering evidence.

### Scenario: analyzed-sites-enter-judgment
GIVEN a SARIF report names an analyzed realization site
WHEN Azimuth imports the report
THEN the affected claim's judgment inputs include the observation
