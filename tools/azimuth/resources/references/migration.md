# Migration reference

Upgrade the CLI and ecosystem-owned Azimuth component pins first. Use `azimuth update --check` and `--dry-run` to reconcile managed repository resources. Semantic account migration is separate.

`azimuth migrate plan --out <file>` records exact source hashes and classifies the transition as `automatic`, `review-required` or `unsupported`. Resolve every review-required item manually and replan. `azimuth migrate apply --plan <file>` accepts only a complete automatic plan whose inputs remain unchanged. Normal validation never accepts historical syntax merely because a migration reader recognizes it.
