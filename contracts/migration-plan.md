# Migration plan

`azimuth migrate plan` writes a strict content-addressed transition account. Version 1 is:

```json
{
  "format": "azimuth-migration-plan",
  "schemaVersion": 1,
  "migrationLine": "alpha-claim-case",
  "fromRelease": "0.1.0-alpha.3",
  "toRelease": "0.1.0-alpha.4",
  "installation": "azimuth/installation.json",
  "installationSha256": "<64 lowercase hexadecimal characters>",
  "disposition": "automatic",
  "edits": [],
  "findings": [],
  "fingerprint": "sha256:<64 lowercase hexadecimal characters>"
}
```

Disposition is exactly `automatic | review-required | unsupported`. Findings are deterministic human-readable accounts of semantic work that prevents automatic application. Edits are a release-edge-specific closed schema; an apply implementation rejects every edit form it does not understand. The alpha.3-to-alpha.4 edge has no semantic edits.

The fingerprint is canonical SHA-256 over every field except `fingerprint`, with recursively sorted object keys. Apply requires `automatic`, a current fingerprint, the exact installation digest and a wholly supported edit set before writing. Review-required and unsupported plans are read-only accounts and never authorize placeholder or partial output.
