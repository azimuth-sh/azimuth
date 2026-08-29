# Intent-delta reference

Each file under a change's `specs/` directory begins with `# Intent delta: <spec-id>`. The current delta parser accepts whole Claim additions and criticality changes. It does not accept remove, rename, arbitrary modification or adding a Case to an existing Claim.

Use `## Add claim: <lower-kebab-id>`, followed by `Criticality: routine`, one falsifiable SHALL proposition and one or more `### Add case: <lower-kebab-id>` blocks. Cases use GIVEN only for a real precondition, WHEN for the trigger and THEN or AND for observable outcomes. Do not use retired Requirement or Scenario headings.
