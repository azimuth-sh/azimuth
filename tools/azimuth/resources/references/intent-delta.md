# Intent-delta reference

The canonical format contract is `contracts/intent-delta.md` in the Azimuth project.

Intent files live under a change's `specs/` tree or `deltas/<module-id>/spec.md`, never both. Each begins with `# Intent delta: <spec-id>`; the account-delta layout requires the declared module id to match its path. Supported operations are module term addition, whole Requirement addition, whole Requirement replacement, explicit Requirement or Scenario removal, and criticality change. Renaming is a remove-and-add decision with explicit reference repair.

Use `## Add term: <term-id>` followed by a non-empty free-form vocabulary definition for a module term. The projector produces `## Term:` in the accepted spec. Identical existing content is already applied; conflicting content for the same term id is rejected. This operation does not replace or remove terms.

Use `## Add requirement: <lower-kebab-id>`, followed by `Criticality: routine`, one non-empty free-form normative Markdown statement and one or more `### Add scenario: <lower-kebab-id>` blocks. Each Scenario contains non-empty free-form normative Markdown in any human language. Core preserves and fingerprints that content without interpreting natural-language keywords, translations, tables, diagrams or code fences. Do not use retired Claim or Case headings.

To replace a whole accepted Requirement:

```markdown
## Replace requirement: <requirement-id>
From: sha256:<accepted-Requirement-block-digest>
Because: <reason for the changed proposition>
Criticality: routine
Remove scenarios: <scenario-id>, <scenario-id>

<complete target Requirement statement>

### Scenario: <scenario-id>
<complete target Scenario statement>
```

Omit `Remove scenarios:` when none is removed. It must name exactly the accepted Scenarios absent from the target block; replacement cannot silently drop them. The replacement may change a Scenario body or add a Scenario to the same Requirement.

To remove a Requirement or one Scenario, use `## Remove requirement: <requirement-id>` or `## Remove scenario: <scenario-id>`, followed by `From: sha256:<accepted-block-digest>` and a non-empty `Because:`. An optional `Replaced-by:` records the successor identity. A Scenario removal must leave at least one Scenario. Removal refuses current realization, design or verification references to the removed identity. Removing the last Requirement retires the spec file.

`From:` fingerprints the exact accepted Markdown block, excluding trailing whitespace. Run `azimuth change intent-capture <id>` to insert accepted fingerprints for replacement or removal. It refuses to overwrite a different existing `From:`. Capture does not apply the delta.

Run `azimuth change intent-preview <id> --out <reviewed-preview-file>` during proposal review. It displays the target spec and full before/after diff. After implementation and acceptance, run `azimuth change intent-apply <id> --preview <reviewed-preview-file>`. Apply recomputes the projection and refuses a stale preview or stale `From:`. It never runs automatically from finalize or archive. Run `azimuth change validate` and normal `azimuth validate` afterward. Do not treat proposed intent as accepted before implemented behavior is ready for acceptance.
