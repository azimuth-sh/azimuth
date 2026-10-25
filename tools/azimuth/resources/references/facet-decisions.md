# Facet decisions reference

A workflow authoring task may return output kind `facet-unchanged` when the requested change requires no transition for its assigned spec, design or verification facet. Independent review remains required and examines the composed account. This decision is appropriate for an implementation correction that preserves the accepted obligation, architecture or verification method; it cannot substitute for a needed facet correction.

Write and durably retain a JSON file with this exact shape:

```json
{
  "format": "azimuth-facet-unchanged",
  "version": 1,
  "module": "module-id",
  "facet": "spec",
  "baseline": {
    "kind": "accepted-facet",
    "repository": "repository-id",
    "ref": "azimuth/model/module-id/spec.md",
    "fingerprint": "sha256:<actual-file-byte-digest>"
  },
  "rationale": "Why this change requires no transition in this facet."
}
```

`facet` is exactly `spec`, `design` or `verification`; `module` must match the task target. `baseline.repository` is optional and selects a declared repository when present; otherwise the reference resolves relative to the workflow instance. Use a normal portable relative `ref` without traversal or symlinks. Reference the actual accepted facet path, not a delta or preview. Hash the real file bytes with SHA-256. `ref` must end in the exact facet filename: `spec.md`, `design.md` or `verification.md`. Its parent must contain accepted `spec.md` with `# Spec: <assigned-module-id>`; an existing design or verification facet must declare that same module identity. The baseline source must be the accepted module reviewed in scope, not an arbitrary directory with matching labels. Core validates the co-located declarations; those declarations alone do not establish membership in a separately registered account.

`fingerprint: null` is permitted only for verified absence of `design.md` or `verification.md` beside that existing accepted Spec, without a symlink or path escape. An absent spec needs an actual spec-addition delta; it cannot be declared unchanged. Null does not mean unknown or unchecked. Custom facet filenames or layouts that cannot satisfy these co-location checks are unsupported by this decision form.

The workflow verifies the baseline and required target identity. The reviewer must independently decide whether unchanged content remains sufficient for the request and projected account. Record rejected decisions and corrections through the ordinary stable review issue/disposition protocol. If a facet transition is needed, author its guarded delta and repeat affected review instead of relaxing the decision.

Author this JSON directly from observed inputs. There is no separate generator command. Retain the decision and baseline basis alongside other review inputs; do not claim a missing observation or unavailable repository was verified.
