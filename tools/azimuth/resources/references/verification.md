# Verification reference

Routine Requirements owe no enrolled Checks or evidence review. Ordinary engineering tests remain outside the Azimuth evidence graph unless deliberately enrolled for an applicable non-routine Requirement.

`verification.md` owns Check methods, inline Check–Scenario Evidence Bindings, package declarations and machine-executed Verification Challenges. Independent Requirement Reviews belong in separate review artifacts. Deployment Requirement Status is computed in AAS from the observed release composition and applicable evidence; historical authored Requirement Status records retain their original meaning. Method Qualification, Applicability Decision, Requirement Judgment and Check Review are not mandatory stages of the current compact account. Historical declaration and review contracts retain their original meanings; do not convert those records into positive current assessments.

Requirement, Scenario, Mechanism, Check and Verification Challenge IDs are stable project-wide names in typed namespaces. Module membership and Scenario parent relationships are separately authored. Qualified spellings are not aliases.

## Checks and package declarations

A document-owned Check has a stable ID and substantive prose describing assertions, controls and limits. Inline `Evidence bindings` relate it to stable Scenarios through nonempty Contributions; a Scenario-nested Contribution may inherit the Scenario. The Check–Scenario pair identifies the binding. Source markers identify implementations only. Do not add binding IDs, Decision Policy or Method Qualification metadata to the compact form. Declare execution scope and deployment contribution in the Check’s `Deployment:` metadata; pin context contract revisions when effective settings affect applicability.

Enable verification packages explicitly. `azimuth.surface` supplies Surfaces and independent Expectation Sets; `azimuth.network` supplies Probes and origins. Checks reference them through `Surface`, `Expectations`, `Select`, `Probe` and `Origin`. Publish field names, not duplicate language types; ecosystem emitters derive types and supported values from producer schemas. Expectations must be independent of the implementation metadata being examined.

Package source markers identify enumerators, expectations and probe implementations in their package namespaces. Pin expectation inputs so oracle changes affect definition fingerprints. `azimuth surface verify` validates source-linked schemas, enumeration completeness and joins in both directions; it does not prove behavior or author a review. Failed discovery, omitted members and failed controls remain findings.

## Machine-executed Verification Challenges

Declare the search purpose, form, target assessment kind and selected Requirements or Mechanisms in `verification.md`. Two blocks use the same label with different meanings: `- Requirements:` under a `### Verification Challenge:` heading selects the Requirement IDs that the challenge searches, while `  - Requirements:` inside a Check `- Execution:` list names execution capabilities the runner must provide. Implement code or configuration with source linkage. Use ordinary Requirement realization or Mechanism implementation tags to define product code targets; no duplicate Verification Challenge target marker is required. Tool adapters resolve semantic sites into supported source filters and report ambiguity, exclusions and expansions. Mutation targets, build/analysis dependencies and exercised Checks are distinct scopes.

Planning pins an exact Requirement Review; `requirement-review` is the only Challenge target kind. Execution records its own exact Verification Target. Current Stryker integration pins batch source input and native mutation statuses/locations; per-mutant Verification Targets and mutant-specific product Check Results are unavailable. Results are `clean | findings | inconclusive`; a clean search never establishes positive Requirement Status. Findings require disposition against the assessment they attack. Scoped `challenge_bundles` in review packets supply native proof Runs; the compiled `challenge_basis` (an array, possibly empty) pins exact Run, bundle, result and target fingerprints. These are assessment-objection dependencies, not a product Check Result basis for definition review. Publish those Runs before dependent review records.

## Review and execution workflow

Validate intent and structure with `azimuth change validate --scope structure`; inspect acceptance obligations separately with `--scope acceptance`. Keep structural validity, evidence completeness and human acceptance distinct.

Prepare definition review packets with `azimuth assurance review-input`, author a Requirement Review and compile aggregate Markdown with `review-compile`. Definition review examines Scenarios, design, implementations, oracles, coverage, Verification Challenges and limitations without requiring an execution Verification Target. Execute Checks and Verification Challenges only through explicitly configured bounded adapters against exact source, artifact or deployment Verification Targets. Local and CI are provenance, not interchangeable evidence from different content.

Execute source/artifact Checks locally or in CI against their exact Verification Targets and retain typed Runs and effective-context captures. Deployment-only Checks and Monitors execute after rollout. AAS computes deployment Requirement Status using the current Requirement Review, release provenance, actual deployment and compatible context evidence. Missing measurements, broken controls and incomplete scope remain gaps. A successful Run or clean Challenge alone does not confirm a deployed Requirement. Requirement Status is never authored; workflow instances pinned to templates that author it are rejected.

## Change-local records and publication

Use aggregate files such as `reviews/requirement-reviews.md`. These filenames are conventions. Record exact reviewer, actual review time, input fingerprint, outcome and rationale; optional Reassesses preserves correction history. Agent and model provenance belongs to the change's Work Provenance ledger, and compilation rejects `Agent` and `Model` review fields. See `azimuth reference show assurance-reviews` for native and Markdown rules.

Generated review packets, compiled bundles, scope reports and Runs belong in ignored change `.azimuth/`. Local drafts and exploratory executions stay local by default. CI publishes the current delivery package and relevant adverse facts; hosted evidence follows authorized deployment. CI transports an unchanged local review only with the authorized reviewer credential matching its original reviewer. Producer permission cannot submit a review. No publication implicitly selects accepted authority or accepts the change.
