# Intent delta: framework/verification-evidence-bindings

## Replace requirement: method-and-applicability-decisions-have-exact-boundaries
From: sha256:c850613854c98e08af9e07f1f2777ea20614e19cf7a6bc2bdf5f319b862cc8b8
Because: One definition review covers method credibility and Case contributions, while health assesses exact execution facts.
Remove scenarios: one-applicability-decision-per-binding

Azimuth SHALL separate assessment of a Claim definition from assessment of its health for an exact Subject. A Claim Review SHALL assess the complete relevant definition, implementation and declared Check-to-Case contributions. Claim Health SHALL assess the current review and exact execution basis. Neither assessment SHALL infer support from source markers, artifact existence or another Subject.

### Scenario: claim-review-assesses-declared-contributions

Given several Check-to-Case contributions under one Claim, when its Claim Review is authored, then the reviewer assesses their meaning and implementation together with the Claim's realization and mechanisms. No separate Check Review, Method Qualification or Applicability Decision is required.

### Scenario: semantic-change-expires-dependent-decisions

Given retained assessments, when a fingerprinted definition, realization, mechanism, Check implementation, independent input or relied-on objection changes, then dependent assessments become stale. An unrelated Claim's unchanged assessment retains its identity; a reassessment is a new record rather than a mutation of historical approval.

### Scenario: routine-claim-rejects-verification

Given a routine Claim in the historical declaration lane, when its historical verification declaration is inspected, then existing inapplicability diagnostics remain visible rather than being converted into approval. The current standard workflow does not require Claim Review or Claim Health for routine Claims; an explicitly requested native assessment does not create a mandatory review stage for all routine Claims.

### Scenario: health-preserves-evidence-gaps

Given Observations for an exact Subject, when Claim Health is assessed, then every applicable Case and declared contribution is accounted for. Missing, violated or inconclusive outcomes, unavailable current review and changed active Run heads cannot produce supported health. Another Subject's Observations do not fill the gap without an explicit supported composition rule.

## Replace requirement: challenge-plans-resolve-through-traceability
From: sha256:e97609564e4583ccd686379aac4e573c6a4a46377f7d09cbeb1a6e770c479532
Because: Current machine planning selects Claim Review or Claim Health and resolves tagged production and Check scope.

Azimuth SHALL resolve machine Challenge requests through stable model relations to exact current Claim Review or Claim Health fingerprints. Assessment identity and execution scope SHALL remain separate. Provider file paths and source regions SHALL be derived by adapters from the selected semantic scope, not substituted for assessment identity.

### Scenario: selectors-compose-deterministically

Given selectors reaching overlapping assessment targets through stable relations, when a Challenge request is resolved, then the semantic selection is a sorted deduplicated union of exact targets and its required scope remains accountable.

### Scenario: empty-selection-fails-closed

Given a request reaching no current eligible assessment, when execution is planned, then planning reports the unresolved target and emits no broad or whole-suite substitute.

### Scenario: paths-are-not-semantic-selectors

Given a source path, line or glob used as assessment identity, when the request is parsed, then it is rejected. Concrete source filters may occur in pinned adapter configuration after semantic target and scope resolution; they do not define a Claim or assessment target.
