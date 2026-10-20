# Work packages

## Work package: observation-service
Status: complete
Depends on: none
Owns: services/assurance/server
Objective: Retain subject-free target observations and resolve to exact observed deployments and native evidence within ASS, preserving relational storage and unmatched facts.
Evidence: Clean cargo check; full server suite 18 tests passed, including relational preservation, historical matching and poison isolation; updated real HTTP/PostgreSQL integration passed two different releases with the same Check definition. Independent implementation review approved.

## Work package: collection-transport
Status: complete
Depends on: none
Owns: services/collector
Objective: Collect runtime target identity without reading ASS and durably publish subject-free measurements and contexts under the agreed protocol.
Evidence: 13 focused real HTTP/context/outbox/configuration tests passed with resource warnings treated as errors; final independent rerun passed. Prometheus stale-alert/replacement-process attribution corrected and verified.
