# Migration: alpha-claim-case to alpha-requirement-scenario

Migration line: alpha-requirement-scenario
Account transformation: automatic authored-syntax rewrite

Claims become Requirements, specification Cases become Scenarios, assurance Subjects become Verification Targets, Check Observations become Check Results, Verification Context becomes Execution Context, Challengers become Verification Challenges, and Claim Review and Claim Health become Requirement Review and Requirement Status. Obligations, stable identifiers, evidence semantics and assessment rules do not change.

`azimuth migrate plan` rewrites only syntax positions in active Markdown under `azimuth/model`, `azimuth/standards` and active changes, never inside `archive` or `history` directories or fenced blocks:

- headings `## Claim:`, `### Case:`, `Claim design`, `Case design`, `Claim verification`, `Case verification`, `Claim review`, `Claim health`, `Claim judgment`, `Challenger` and the intent-delta operations `Add claim`, `Add case`, `Replace claim`, `Remove claim`, `Remove case`;
- metadata labels `Claim`, `Claims`, `Case`, `Cases`, `Challenger`, `Subjects` and `Remove cases`;
- the closed vocabularies in `Select`, `Required scope` and `Targets` values (`claim`, `case`, `claim-review`, `claim-health`, `claim-judgment`);
- the `Context:` label of `Evidence Binding` and `Method Qualification` blocks, which becomes `Execution Context:`; `Context:` elsewhere is ordinary text and stays.

It also rewrites `azimuth/workspace.json` from version 1 to version 2, renaming each realization obligation `claim` to `requirement`. Identifiers that contain retired words, such as `verification-context-is-observed-and-bounded`, are preserved; renaming one is a remove-and-add decision. Prose is not rewritten.

An installation already on `alpha-requirement-scenario` receives the same rewrite for any retired syntax that remains, without a migration-line change. The plan records each file's current and rewritten SHA-256. `azimuth migrate apply` recomputes the complete edit set, refuses any drift, writes every file and the installation's migration line in one transaction, and leaves managed resources for `azimuth update`. Re-extract source manifests and Execution Context reports with renamed packages; regenerate fingerprints, plans, Runs and reviews. Historical Runs, reviews, manifests and archives remain immutable and are not re-fingerprinted. Every Requirement, Scenario, binding, judgment, review and authority fingerprint changes, so prior reviews and baselines do not carry over.
