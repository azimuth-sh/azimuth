# Migration plan

`azimuth migrate plan` writes a strict content-addressed transition account. Schema version 2 is:

```json
{
  "format": "azimuth-migration-plan",
  "schemaVersion": 2,
  "fromMigrationLine": "alpha-claim-case",
  "migrationLine": "alpha-requirement-scenario",
  "fromRelease": "0.1.0-alpha.6",
  "toRelease": "0.1.0-alpha.7",
  "installation": "azimuth/installation.json",
  "installationSha256": "<64 lowercase hexadecimal characters>",
  "disposition": "automatic",
  "edits": [
    {
      "path": "azimuth/model/billing/invoices/spec.md",
      "sha256": "<64 lowercase hexadecimal characters>",
      "rewrittenSha256": "<64 lowercase hexadecimal characters>"
    }
  ],
  "findings": [],
  "fingerprint": "sha256:<64 lowercase hexadecimal characters>"
}
```

`fromMigrationLine` and `fromRelease` are the installation's current values; `migrationLine` and `toRelease` are the running CLI's. A plan is `unsupported` when no release edge or migration-line edge reaches the running CLI.

Disposition is exactly `automatic | review-required | unsupported`. Findings are deterministic human-readable accounts of semantic work that prevents automatic application; each names a file, a line and retired syntax that the rewrite cannot express. Each edit is exactly `{path, sha256, rewrittenSha256}`: a repository-relative path and the SHA-256 of the file's current and rewritten bytes. Edits sort by path.

The `alpha-claim-case` to `alpha-requirement-scenario` account migration rewrites authored syntax in active Markdown under `azimuth/model`, `azimuth/standards` and unarchived changes, never inside `archive` or `history` directories or fenced blocks: Claim and Case headings, labels and intent-delta operations become Requirement and Scenario forms, Challenger becomes Verification Challenge, Claim review and Claim health become Requirement review and Requirement status, and the `Context:` label of Evidence Binding and Method Qualification blocks becomes `Execution Context:`. It also rewrites `azimuth/workspace.json` from version 1 to version 2, renaming each realization obligation `claim` key to `requirement`. Stable identifiers and prose are not rewritten. An installation already on `alpha-requirement-scenario` receives the same rewrite for any remaining retired syntax without a migration-line change. The shipped `alpha-claim-case-to-alpha-requirement-scenario` and `0.1.0-alpha.6-to-0.1.0-alpha.7` migration references describe this edge.

The fingerprint is canonical SHA-256 over every field except `fingerprint`, with recursively sorted object keys. Apply requires `automatic`, the running migration line and release, a current fingerprint, the installation's exact path, source release, source migration line and digest, and an edit set equal to one recomputed from the current files before writing. It writes every edited file and, when the line changes, the installation's migration line in one transaction; managed resources are left for `azimuth update`. Review-required and unsupported plans are read-only accounts and never authorize placeholder or partial output.
