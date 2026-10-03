# Intent-delta reference

The canonical format contract is `contracts/intent-delta.md` in the Azimuth project.

Each file under a change's `specs/` directory begins with `# Intent delta: <spec-id>`. Supported operations are whole Claim addition, whole Claim replacement, explicit Claim or Case removal, and criticality change. Renaming is a remove-and-add decision with explicit reference repair.

Use `## Add claim: <lower-kebab-id>`, followed by `Criticality: routine`, one non-empty free-form normative Markdown statement and one or more `### Add case: <lower-kebab-id>` blocks. Each Case contains non-empty free-form normative Markdown in any human language. Core preserves and fingerprints that content without interpreting natural-language keywords, translations, tables, diagrams or code fences. Do not use retired Requirement or Scenario headings.

To replace a whole accepted Claim:

```markdown
## Replace claim: <claim-id>
From: sha256:<accepted-Claim-block-digest>
Because: <reason for the changed proposition>
Criticality: routine
Remove cases: <case-id>, <case-id>

<complete target Claim statement>

### Case: <case-id>
<complete target Case statement>
```

Omit `Remove cases:` when none is removed. It must name exactly the accepted Cases absent from the target block; replacement cannot silently drop them. The replacement may change a Case body or add a Case to the same Claim.

To remove a Claim or one Case, use `## Remove claim: <claim-id>` or `## Remove case: <case-id>`, followed by `From: sha256:<accepted-block-digest>` and a non-empty `Because:`. An optional `Replaced-by:` records the successor identity. A Case removal must leave at least one Case. Removal refuses current realization, design or verification references to the removed identity. Removing the last Claim retires the spec file.

`From:` fingerprints the exact accepted Markdown block, excluding trailing whitespace. Run `azimuth change intent-capture <id>` to insert accepted fingerprints for replacement or removal. It refuses to overwrite a different existing `From:`. Capture does not apply the delta.

Run `azimuth change intent-preview <id> --out <reviewed-preview-file>` during proposal review. It displays the target spec and full before/after diff. After implementation and acceptance, run `azimuth change intent-apply <id> --preview <reviewed-preview-file>`. Apply recomputes the projection and refuses a stale preview or stale `From:`. It never runs automatically from finalize or archive. Run `azimuth change check` and normal `azimuth validate` afterward. Do not treat proposed intent as accepted before implemented behavior is ready for acceptance.
