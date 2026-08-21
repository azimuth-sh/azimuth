# Work packages: verification-evidence-bindings

## Work package: semantic-authority
Status: complete
Depends on: none
Owns: docs/decisions.md, azimuth/formats/verification.md, azimuth/standards/verification.md, azimuth/standards/judgment.md, azimuth/model/framework/verification-evidence-bindings, azimuth/changes/verification-evidence-bindings
Objective: freeze and apply the alpha 2 repository verification contract
Evidence: change-account validation, format review and current model parse

## Work package: declaration-kernel
Status: complete
Depends on: semantic-authority
Owns: tools/azimuth/src/plan.rs, tools/azimuth/src/verification.rs, tools/azimuth/src/judgment.rs, tools/azimuth/src/fingerprint.rs, tools/azimuth/src/model.rs, tools/azimuth/src/change.rs, tools/azimuth/tests/plans.rs, tools/azimuth/tests/verification.rs
Objective: implement strict declarations, cardinalities and canonical fingerprints
Evidence: strict parser, versioned fingerprint, selector, staleness and export-shape tests

## Work package: manifest-check-linkage
Status: complete
Depends on: declaration-kernel
Owns: tools/azimuth/src/manifest.rs, tools/azimuth/src/design.rs, tools/azimuth/tests/packages.rs, tools/azimuth/tests/designs.rs
Objective: accept Check implementations and reject every alpha 1 evidence key
Evidence: manifest allowlist, source identity and old-key rejection tests

## Work package: model-validation-projection
Status: complete
Depends on: declaration-kernel, manifest-check-linkage
Owns: tools/azimuth/src/validation.rs, tools/azimuth/src/assurance.rs, tools/azimuth/src/traceability.rs, tools/azimuth/tests/validation.rs, tools/azimuth/tests/assurance.rs, tools/azimuth/tests/traceability.rs
Objective: validate the Check graph, exact challenge resolution and the isolated service boundary
Evidence: Finding, traversal, zero-resolution, assurance and traceability tests

## Work package: loader-cli-cutover
Status: complete
Depends on: declaration-kernel, manifest-check-linkage, model-validation-projection
Owns: tools/azimuth/src/lib.rs, tools/azimuth/src/main.rs, tools/azimuth/src/federation.rs, tools/azimuth/src/spec.rs, tools/azimuth/Cargo.toml, tools/azimuth/Cargo.lock, tools/azimuth/tests/cli.rs, tools/azimuth/tests/federation.rs, tools/azimuth/tests/spec_parse.rs
Objective: integrate loading, assembly-owned identity, selection closure, export v2 and CLI deletion
Evidence: Rust, CLI, spoofing, merged-conflict, owner and populated-closure tests

## Work package: alpha1-importer-retirement
Status: complete
Depends on: semantic-authority
Owns: tools/extractors/typescript/package.json, tools/extractors/typescript/package-lock.json, tools/extractors/typescript/src/manual-cli.ts, tools/extractors/typescript/src/manual-results.ts, tools/extractors/typescript/src/manual-results.test.ts, tools/extractors/typescript/src/observation-cli.ts, tools/extractors/typescript/src/observations.ts, tools/extractors/typescript/src/observations.test.ts, tools/extractors/typescript/src/mutation-cli.ts, tools/extractors/typescript/src/mutation-results.ts, tools/extractors/typescript/src/mutation-results.test.ts, tools/extractors/typescript/src/pit-cli.ts, tools/extractors/typescript/src/pit-results.ts, tools/extractors/typescript/src/pit-results.test.ts, tools/extractors/typescript/src/pit-xml.ts, tools/extractors/typescript/src/sarif-cli.ts, tools/extractors/typescript/src/sarif-results.ts, tools/extractors/typescript/src/sarif-results.test.ts
Objective: delete alpha 1 result readers and binary entry points
Evidence: package build, unit tests and absence audits

## Work package: typescript-check-linkage
Status: complete
Depends on: semantic-authority, alpha1-importer-retirement
Owns: packages/typescript/src/index.ts, tools/extractors/typescript/src/cli.ts, tools/extractors/typescript/src/emitter.ts, tools/extractors/typescript/src/emitter.test.ts, tools/extractors/typescript/src/prometheus.ts, tools/extractors/typescript/src/prometheus.test.ts, tools/extractors/typescript/fixture/plain.test.ts, tools/extractors/typescript/fixture/traced.test.ts
Objective: replace TypeScript Covers markers and Prometheus keys with ImplementsCheck
Evidence: TypeScript build, retired-marker rejection, exact fingerprint and Prometheus tests

## Work package: dotnet-jvm-check-linkage
Status: complete
Depends on: semantic-authority
Owns: packages/dotnet/Azimuth.Annotations/Tags.cs, packages/dotnet/Azimuth.Annotations/Azimuth.Annotations.csproj, packages/jvm/src/main/java/dev/drim/azimuth/Azimuth.java, tools/extractors/dotnet/Azimuth.Emit/Collector.cs, tools/extractors/dotnet/Azimuth.Emit/Program.cs, tools/extractors/dotnet/Azimuth.Emit/SourceFiles.cs, tools/extractors/dotnet/Azimuth.Emit.Tests/CollectorTests.cs, tools/extractors/dotnet/fixture/Fixture.cs, tools/extractors/jvm/src/main/java/dev/drim/azimuth/emit/Main.java, tools/extractors/jvm/src/test/java/dev/drim/azimuth/emit/MainTest.java
Objective: replace .NET and JVM Covers markers with Check implementation linkage
Evidence: .NET and JVM package, extractor, strict fingerprint and fixture tests

## Work package: native-check-linkage
Status: complete
Depends on: semantic-authority
Owns: packages/go/azimuth/annotations.go, packages/python/azimuth_annotations/__init__.py, packages/rust/azimuth-annotations/src/lib.rs, packages/cpp/azimuth.hpp, tools/extractors/go/main.go, tools/extractors/go/main_test.go, tools/extractors/python/azimuth_emit.py, tools/extractors/python/test_azimuth_emit.py, tools/extractors/rust/src/main.rs, tools/extractors/cpp/azimuth_emit.py, tools/extractors/cpp/test_azimuth_emit.py
Objective: replace Go, Python, Rust and C++ Covers markers with Check implementation linkage
Evidence: all four package, extractor, retired-marker and fingerprint suites

## Work package: synthetic-cutover
Status: complete
Depends on: loader-cli-cutover, typescript-check-linkage, dotnet-jvm-check-linkage, native-check-linkage
Owns: experiments/polyglot, experiments/assurance-extensions, release/candidates.py, release/isolate_experiments.py, release/orchestrate.py, release/publication.py, release/qualify.py, release/test_isolate_experiments.py, release/test_orchestrate.py, release/test_publication.py, release/test_qualify.py, .azimuth/release/private-deployment-linkage.json
Objective: remove evidence enrollment from routine fixtures and cut release inputs to export version 2
Evidence: experiment gates and release qualification suites

## Work package: active-guidance
Status: complete
Depends on: loader-cli-cutover, typescript-check-linkage, dotnet-jvm-check-linkage, native-check-linkage, synthetic-cutover
Owns: AGENTS.md, README.md, docs/framework.md, docs/glossary.md, docs/assurance-extensions.md, docs/change-process.md, azimuth/README.md, azimuth/formats/spec.md, azimuth/formats/design.md, azimuth/formats/workspace.md, tools/azimuth/README.md, tools/extractors/README.md, release/README.md, services/assurance/README.md, .agents/skills/azimuth-apply/SKILL.md, .agents/skills/azimuth-cover/SKILL.md, .agents/skills/azimuth-verify/SKILL.md, .agents/skills/azimuth-archive/SKILL.md, .agents/skills/azimuth-propose/SKILL.md
Objective: replace Covers-centric public guidance and skills with the binding model
Evidence: terminology, service-boundary, citation, link, frontmatter and prohibited-name audits
