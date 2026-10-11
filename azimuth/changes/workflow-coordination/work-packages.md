# Work packages

## Work package: workflow-engine
Status: complete
Depends on: none
Owns: tools/azimuth/src/workflow_state.rs, tools/azimuth/src/main.rs, tools/azimuth/src/resources.rs, tools/azimuth/resources/workflows/, tools/azimuth/resources/manifest.json, tools/azimuth/resources/references/workflow.md, tools/azimuth/resources/skills/, contracts/workflow.md, docs/change-process.md
Objective: Implement explicit durable workflow coordination, reusable declarative definitions, the shipped standard change-development workflow and validated inventory expansion with review/correction loops, without launching agents.
Evidence: Cargo build and product-command inspection, no tests.

## Work package: model-revisions
Status: complete
Depends on: none
Owns: services/assurance/server/src/run_ledger.rs, services/assurance/server/migrations/0004_model_revisions.sql, services/assurance/README.md
Objective: Register and assess candidates within an existing ASS project without replacing accepted authority; preserve authenticated review provenance.
Evidence: Service compilation and SQL/endpoint inspection, no tests.
