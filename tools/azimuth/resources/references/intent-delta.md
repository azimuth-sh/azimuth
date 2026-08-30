# Intent-delta reference

Each file under a change's `specs/` directory begins with `# Intent delta: <spec-id>`. The current delta parser accepts whole Claim additions and criticality changes. It does not accept remove, rename, arbitrary modification or adding a Case to an existing Claim.

Use `## Add claim: <lower-kebab-id>`, followed by `Criticality: routine`, one non-empty free-form normative Markdown statement and one or more `### Add case: <lower-kebab-id>` blocks. Each Case contains non-empty free-form normative Markdown in any human language. Core preserves and fingerprints that content without interpreting natural-language keywords, translations, tables, diagrams or code fences. Do not use retired Requirement or Scenario headings.
