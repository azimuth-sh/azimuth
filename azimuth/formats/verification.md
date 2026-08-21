# Verification declarations

The repository-owned evidence facet for alpha 2 (D43, D45). This file declares what a Check means,
how its terminal result bears on a case-level Claim, and whether that exact edge is credible in its
required context. It never records an execution result.

Only standard and critical Claims may receive verification declarations. Routine Claims stop at
intent. An ordinary native test without deliberate Check enrollment remains outside Azimuth.

## File and identity

```markdown
# Verification: <owning-spec-id>
```

At most one `verification.md` sits beside an owning `spec.md`. The header identifies repository
authority and location; it is not a namespace.

Check, Evidence Binding, Challenger and Challenge Plan ids are project-global, path-independent
lower kebab path ids. Claim ids are `<spec-id>#<case-id>`. Mechanism ids are
`<spec-id>#<mechanism-id>`. Moving this file or its package changes no identity.

Declarations may appear in any order. Every id and referenced identity is validated over the
complete project account before `--only` selection retains the closure for selected Claims.

## Check

```markdown
## Check: payments/recovery-under-broker-loss
Method: inject broker loss after the accepted write
Method: observe replay after broker recovery
Terminal: the accepted write is replayed exactly once after recovery

The two method lines describe one atomic observable result.
```

A Check has one project-global id, one or more `Method:` lines and exactly one `Terminal:`
proposition. The terminal proposition must describe one satisfied-or-violated result. If outcomes
can vary independently, declare separate Checks.

Prose after the label block is required review rationale and does not enter the fingerprint. Every
Check must have at least one Evidence Binding. A Check may have several source implementations;
their semantic identities compose one implementation set.

## Evidence Binding

```markdown
## Evidence Binding: payments/recovery-replay-edge
Check: payments/recovery-under-broker-loss
Claim: payments/recovery#accepted-write-is-replayed
Proposition: replay after injected broker loss directly exercises the recovery predicate
Scope: component
Quantification: example
Oracle: relational
Context: {"platform":"linux-x86_64","storage":"postgres-17"}
Challenge domain: ["realization","mechanism","check-implementation","oracle","context"]
Qualification policy: credible-executable

This edge is narrower than the whole recovery suite and can be challenged independently.
```

A binding names exactly one Check and one case-level Claim. One Check may bind to several Claims,
and one Claim may receive several Checks. The pair `(Check, Claim)` is unique.

`Proposition:` explains why the Check's one terminal result bears on this Claim. The form is the
actual evidence form for this edge:

- `Scope:` is `unit | component | e2e`;
- `Quantification:` is `example | universal`; and
- `Oracle:` is `direct | golden | relational | metamorphic | model-based | contract`.

There is no Strength field. Executable Checks demonstrate sampled behavior. Structural proof
remains in design mechanisms and contributes to Claim Judgment rather than becoming a fictitious
Observation.

`Context:` is a required JSON object from string keys to uninterpreted string values. `{}` is
explicit and valid. Equality is exact in alpha 2: values have no range, wildcard or
provider-expression semantics.

`Challenge domain:` is a non-empty JSON array drawn from the closed set
`realization | mechanism | check-implementation | oracle | context`. Values are sorted and
deduplicated semantically. It constrains relation-based challenge traversal; it is not a list of
tools.

`Qualification policy:` names one project policy from
`azimuth/standards/verification.md`. Prose after the labels is required rationale and is excluded
from fingerprints.

The policy's required forms enter this binding's fingerprint. Capability coverage and actual
challenge execution are validated by the dependent adapter and Run-planning contracts, not inferred
from the presence of repository prose.

## Qualification

```markdown
## Qualification: payments/recovery-replay-edge
Verdict: qualified
Fingerprint: sha256:<64-lowercase-hex>
Qualified: 2026-08-21
Qualifier: evidence-owner@example

The Check implementation, oracle and exact platform context make this edge credible.
```

The Qualification id is exactly the Evidence Binding id. Every applicable binding has exactly one
Qualification. Duplicate blocks are parse errors; missing, rejected or stale Qualifications are
Findings.

`Verdict:` is `qualified | rejected`. `Qualified:` is an ISO date and `Qualifier:` is a
non-empty accountable identity. Rationale is required and never changes the expected fingerprint.

## Challenger

```markdown
## Challenger: mutation/implementation-perturbation
Form: implementation-perturbation
Searches for: an implementation change that leaves the bound Check satisfied

Surviving changes are objections to credibility, not product outcomes.
```

`Form:` is an open lower kebab path id interpreted by qualification policy and later adapter
capabilities. `Searches for:` is the objection proposition. A Challenger never directly evaluates
a product Claim and is not recursively qualified in alpha 2.

## Challenge Plan

```markdown
## Challenge Plan: payments/recovery-credibility
Challenger: mutation/implementation-perturbation
Select: qualification from binding payments/recovery-replay-edge
Select: qualification from check payments/recovery-under-broker-loss
Select: qualification from realization payments|rust-item|recovery::replay
Select: qualification from mechanism payments/recovery#transactional-outbox
Select: claim-judgment from claim payments/recovery#accepted-write-is-replayed
Select: claim-judgment from realization payments|rust-item|recovery::replay
Select: claim-judgment from mechanism payments/recovery#transactional-outbox

The plan states semantic reach; a Run freezes the exact resolved fingerprints.
```

A plan names one Challenger and one or more repeatable `Select:` lines using only the seven forms
shown above. Resolution unions, sorts and deduplicates current decision fingerprints.

Qualification traversal from realizations or mechanisms reaches case-level Claims and their
Evidence Bindings only when each binding's Challenge domain authorizes that relation. Claim
Judgment selectors are reserved and resolve only after a current Claim Judgment format exists.
Zero resolution is a Finding. Source paths, line numbers, globs and whole-suite fallback are
invalid.

The selector retains the complete semantic source address after the relation token, including
spaces. An assembly-derived address such as `web|next-route|GET /payments/[id]` is semantic; a
source-file locator remains invalid even when placed in the address field.

## Canonical fingerprints

All fingerprints use SHA-256 over versioned canonical JSON with sorted object keys and set-like
collections. There is no legacy fingerprint reader.

- Check fingerprint: format version, Check id, ordered methods, terminal proposition, and sorted
  implementation semantic identities plus source fingerprints.
- Binding fingerprint: format version, binding id, Check id, semantic Claim digest, Proposition,
  form tuple, sorted challenge domain and qualification-policy digest.
- Context fingerprint: format and version plus the canonical required-context object.
- Qualification fingerprint: Check, Binding and Context fingerprints.

Paths, lines, mounts, criticality and explanatory prose are excluded. A source, Claim, binding,
policy or context change stales the decision it actually affects.

## Source boundary

Source uses `ImplementsCheck(<check-id>)`. A language extractor emits only:

```json
{
  "check_implementations": [
    {
      "check": "payments/recovery-under-broker-loss",
      "site": "recovery::replay_after_loss",
      "file": "src/recovery.rs",
      "lang": "rust",
      "source_fingerprint": "sha256:<64-lowercase-hex>"
    }
  ]
}
```

Workspace or project assembly attaches `area`, `mount`, `address_kind` and `address` from
declared repository structure. Files and mounts remain locators; the semantic source identity is
`<area>|<address-kind>|<address>`.

An implementation marker contains no Claim, form, context or Qualification. A native test without
the marker emits nothing.

## Rejected alpha 1 input

The parser rejects old `## Claim` and residual headings, evidence floors, non-test evidence,
Strength, detector fields and old judgment blocks. Manifests reject `covers`,
`mechanism_covers` and `observations`. Annotations reject Covers and CoversMechanism. Nothing is
translated, deprecated or exported twice.
