# Verification reference

Routine Claims reject Checks, Evidence Bindings, Qualifications and Claim Judgments. Ordinary engineering checks remain outside the Azimuth evidence graph.

For a non-routine Claim, the model package's `verification.md` owns Checks, sparse Check-to-Case Evidence Bindings, one Qualification per binding, total-composition Claim Judgments when applicable, Challengers and Challenge Plans. Every accepted Qualification or Claim Judgment names a current Decision Policy. Project `standards/verification.md` owns those policies and the single `Challenge Schedule: current` assigning required forms to exactly one `gate | scheduled` lane.

Use `azimuth validate`, `azimuth report traceability` and `azimuth export` for the current account. A clean Challenge Result is only a negative search fact and does not establish positive product evidence.

Claim, Case, Mechanism and Check IDs are stable project-wide lower-kebab names in separate typed namespaces. Module membership and Case parent relationships are separately authored; source markers use one stable target ID. Qualified spellings are not aliases. See the entity identity contract.

## Compact Claim-first verification

A document-owned Check has a stable ID and substantive prose describing assertions, controls and limits. Inline `Evidence bindings` relate it to stable Cases through nonempty Contributions; a Case-nested Contribution may inherit the Case. The Check–Case pair identifies the binding. Source markers identify implementations only. Do not repeat binding IDs, Context, Decision Policy or Method Qualification metadata in this compact authoring form. Independent review decisions remain separate.

Enable `azimuth.surface` and `azimuth.network` explicitly in the workspace/project when used. Shared `Surface` declarations publish field names and Area; `Expectation set` declarations name Surface and expected field names; `Probe` declarations name Area and Origins. Checks reference these through `Surface`, `Expectations`, `Select`, `Probe` and `Origin`. Selectors use `Member.<field>` or `Expected.<field>` and supported equality/`includes` values from extracted producer schemas. Publish field names only; let emitters derive language types. Expectations are independently authored, not derived from the metadata under examination.

SurfaceEnumerator, SurfaceExpectations and ImplementsProbe source markers belong to their package namespaces. Pin expectation input files so changing an oracle changes the Check fingerprint. Runtime enumeration records exact Subject, producer and input fingerprints, completeness and members. `azimuth surface verify --manifest <file> --enumeration <file> --expectations <file>` validates source-linked schemas and joins, not product behavior or a Claim conclusion. Failed discovery, omitted members and failed controls remain findings.
