# Work packages: claim-first-assurance-accounts

## Work package: account-design-parser
Status: complete
Depends on: none
Owns: tools/azimuth/src/account_design.rs, contracts/account-design.md, tools/azimuth/resources/skills/azimuth-propose/SKILL.md
Objective: parse and preserve Claim-first design declarations and prose
Evidence: isolated compilation and parser diagnostics inspected; no Azimuth tests run

## Work package: account-verification-parser
Status: complete
Depends on: none
Owns: tools/azimuth/src/account_verification.rs, contracts/account-verification.md
Objective: parse and preserve Claim-first verification scopes and method elements
Evidence: isolated compilation and parser diagnostics inspected; no Azimuth tests run

## Work package: account-delta-cli
Status: in-progress
Depends on: account-design-parser, account-verification-parser
Owns: tools/azimuth/src/account_delta.rs, contracts/account-delta.md, tools/azimuth/src/main.rs, tools/azimuth/src/lib.rs
Objective: project complete new module facets and expose guarded CLI preview and application
Evidence: exact comparison with twelve candidate facets, cargo check and CLI preview; no Azimuth tests run

## Work package: account-assembly
Status: complete
Depends on: account-design-parser, account-verification-parser
Owns: tools/azimuth/src/account_model.rs, contracts/account-model.md, tools/azimuth/src/account_support.rs, contracts/account-support.md
Objective: validate cross-facet references and report missing source contributions without positive assurance inference
Evidence: candidate account inspection and diagnostic review; no Azimuth tests run

## Work package: term-vocabulary
Status: complete
Depends on: none
Owns: tools/azimuth/src/spec.rs, contracts/spec.md, contracts/export.md
Objective: retain normative term vocabulary and include its changes in semantic fingerprints and export
Evidence: cargo check and local CLI inspection; no Azimuth tests run

## Work package: account-dotnet-emitter
Status: complete
Depends on: account-assembly
Owns: packages/dotnet/Azimuth.Annotations/Tags.cs, tools/extractors/dotnet/Azimuth.Emit/Collector.cs, tools/extractors/dotnet/Azimuth.Emit/Program.cs
Objective: extract fingerprinted verification Element support, Check definitions and Case contributions from compiled .NET code
Evidence: emitter build and manifest inspection; no Azimuth tests run

## Work package: model-integration
Status: in-progress
Depends on: account-assembly, term-vocabulary
Owns: tools/azimuth/src/model.rs, tools/azimuth/src/validation.rs, tools/azimuth/src/traceability.rs, tools/azimuth/src/assurance.rs, tools/azimuth/src/workspace.rs
Objective: make design-owned Claim Areas and prose participate in canonical validation, export and review freshness
Evidence: cargo check and product command inspection; no Azimuth tests run

## Work package: federation-support
Status: in-progress
Depends on: account-assembly
Owns: tools/azimuth/src/federation.rs
Objective: carry exact source/configuration support provenance through a complete project workset
Evidence: federation product-command inspection and cargo check; no Azimuth tests run

## Work package: integration
Status: pending
Depends on: account-delta-cli, account-assembly, term-vocabulary, account-dotnet-emitter, model-integration, federation-support
Owns: tools/azimuth/src/manifest.rs, azimuth/changes/claim-first-assurance-accounts
Objective: make the accepted account, source contributions and review freshness consistent across commands and model assembly
Evidence: product commands, compile checks and diff inspection; no Azimuth tests run
