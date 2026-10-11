# Work packages

## Work package: workflow-engine
Status: complete
Depends on: none
Owns: tools/azimuth/src/workflow_state.rs
Objective: Implement explicit durable workflow coordination and review/correction loops through strict configured task graphs, without launching agents.
Evidence: Cargo build and product-command inspection, no tests.

## Work package: model-revisions
Status: complete
Depends on: none
Owns: services/assurance/server/src/run_ledger.rs, services/assurance/server/migrations/0004_model_revisions.sql, services/assurance/README.md
Objective: Register and assess candidates within an existing ASS project without replacing accepted authority; preserve authenticated review provenance.
Evidence: Service compilation and SQL/endpoint inspection, no tests.
