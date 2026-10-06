# Work packages: compact-verification-packages

## Work package: package-tooling
Status: complete
Depends on: none
Owns: tools/azimuth/src, tools/azimuth/tests, tools/extractors, packages, release/qualify.py, release/isolate_experiments.py, release/orchestrate.py, release/test_qualify.py, release/test_isolate_experiments.py, release/test_orchestrate.py
Objective: Implement compact package parsing, typed producer manifests, SDK source tags and bounded Check/Run integration with strict generic validation
Evidence: Current SDK/emitter and CLI builds, 107 relevant Rust regressions, 62 TypeScript tests, 9 Python Run helper tests, 37 release tests, real native import interoperability and source-account product validation; see outcome.md

## Work package: run-ledger
Status: complete
Depends on: none
Owns: services/assurance
Objective: Persist current accountable Run records and expose exact subject assurance state without synthesizing decisions
Evidence: Successful service build, core review regressions, independent service review and remediation record, project-scoped authority/Run/review product commands; see outcome.md

## Work package: integration
Status: complete
Depends on: package-tooling, run-ledger
Owns: contracts, docs, tools/azimuth/resources, azimuth, AGENTS.md, release/acceptance.py, experiments/polyglot/check.sh
Objective: Keep public contracts, author skills and canonical authority consistent with implemented package and assurance semantics
Evidence: Updated contracts and embedded resources; current canonical source-account validation reports 46 Claims and zero findings; diff inspection and residuals in outcome.md
