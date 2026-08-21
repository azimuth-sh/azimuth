# Change: verification-evidence-bindings

Status: accepted and complete

Exploration: evidence-control-plane-alpha-2
Carries decisions: E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E12

## Problem

Alpha 1 makes test annotations the evidence relation. A `Covers` marker combines a native test,
its asserted evidence form and its Claim edge, while `verification.md` separately declares floors,
non-test evidence and residuals. Imported observations are then projected back into the same
relation. The result has no first-class Check identity, no independently reviewable binding and no
place for one Qualification per Check-to-Claim edge.

The old plan, judgment and observation formats also assign evidentiary meaning before a provider
has proved what it actually selected. Mutation, broad analysis and fault injection can appear as
product evidence even when their proposition is a challenge to that meaning.

## Outcome

`verification.md` owns project-global Checks, sparse Evidence Bindings, one Qualification per
binding, Challengers and traceability-based Challenge Plans. Check implementation markers emit
only normalized `check_implementations`; annotations never declare Claim coverage or evidence
form.

The loader derives exact Check, binding and Qualification fingerprints from semantic source
identities and source digests. Validation reports missing, stale, rejected, inapplicable and
unresolved verification declarations. Traceability derives Check relationships from the graph.

All alpha 1 Covers, verification-plan, judgment and imported-observation readers are deleted. Old
headings, manifest keys, annotations and CLI behavior fail rather than translate. The optional
Assurance Service remains a later change boundary; it receives no compatibility adapter here.

## Scope

In scope:

- add the alpha 2 verification repository contract and qualification-policy standard;
- establish project-global Check and Challenger identities and stable binding identities;
- define exact Evidence Binding form, context and challenge-domain fields;
- define one Qualification identity per binding and versioned SHA-256 fingerprints;
- define traceability selectors for Qualification and Claim-Judgment challenge targets;
- replace Covers annotations with `ImplementsCheck` in every supported ecosystem;
- normalize Check implementation source identity through the workspace/federation model;
- export format version 2 with Checks, bindings, Qualifications and Challenge Plans;
- enrich the traceability report with derived Check relationships;
- remove the obsolete `azimuth judge` surface until total Claim Judgment has an accepted format;
- delete alpha 1 Covers, plan, judgment and imported-observation parsing and related Findings;
- retire alpha 1 manual, observation, mutation, PIT and SARIF import CLIs; and
- cut synthetic experiments, release workflows, skills and active guidance atomically.

Out of scope:

- Run bundle syntax, provider execution, actual-selection validation or normalized outcomes;
- adapter capability discovery and invocation;
- a new Claim Judgment authoring format or challenge-result application;
- Assurance Service storage and API replacement;
- raising any current requirement above routine;
- treating ordinary native tests as enrolled Checks; and
- any alpha 1 compatibility reader, alias, migration or dual export.

## Affected claims

Add four routine requirements under `framework/verification-evidence-bindings`:

- Checks have atomic terminal propositions and sparse explicit Claim bindings;
- every Evidence Binding has one exact, context-bound Qualification;
- Challenge Plans resolve exact decision fingerprints through traceability; and
- Check implementation linkage is provider-neutral and old evidence inputs fail closed.

The requirements contain twelve case-level Claims. They owe no Azimuth evidence, Qualification or
Claim Judgment while routine. Ordinary parser, extractor, CLI and ecosystem tests remain required.

## Completion conditions

- The strict format parses every new declaration and rejects unknown fields, duplicate identities,
  missing cardinalities, old headings and old semantics.
- Check, Binding, Context and Qualification fingerprints use canonical versioned JSON and SHA-256.
- Check ids are project-global and path-independent; Claim ids remain `<spec>#<case>`.
- A Check has at least one Evidence Binding; a binding names one Check and one case-level Claim;
  `(Check, Claim)` pairs and Qualification-per-binding identities are unique.
- Required context is an exact string map and challenge domains use the closed alpha 2 set.
- Challenge selectors union, sort and deduplicate exact fingerprints; zero resolution is a Finding
  and source paths or globs are rejected.
- `ImplementsCheck` emits only implementation identity and source fingerprint; multiple
  implementation sites compose one Check deterministically.
- Explicit verification on a routine Claim is an inapplicable-verification Finding.
- The complete export is version 2 and has none of the removed plan, Covers, observation or old
  judgment collections.
- `azimuth judge` is unknown and no replacement Claim-Judgment command is invented.
- All seven language integrations and synthetic fixtures use the new linkage or no enrollment.
- Alpha 1 manifest keys and annotations fail closed, and retired importer binaries no longer exist.
- Current repository validation is clean with every active requirement at routine.
- Complete Rust, extractor, annotation, experiment and release tests pass.
