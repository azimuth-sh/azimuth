# Design: Verification Evidence Bindings

## Identity and authority

One strict `verification.md` may sit beside an owning spec. Its header identifies repository
authority, not an implicit namespace. Check, binding, Challenger and Challenge Plan ids are
project-global, lower kebab path ids. Claim identities remain `<spec>#<case>` and mechanism
identities remain `<spec>#<mechanism>`.

Loading discovers all verification authorities before applying `--only`. Selection retains the
closure of selected Claims, their bindings, Checks, Qualifications and applicable Challenge Plans.
Moving a file or spec package cannot change a semantic identity.

## Check and binding

A Check states repeatable method lines and exactly one atomic terminal proposition. It must have at
least one binding. A binding names exactly one Check and one case-level Claim, and each
`(Check, Claim)` pair is unique. One Check may bind to several Claims and one Claim may receive
several Checks without copying the Check declaration.

Each binding states the proposition connecting that terminal result to the Claim, its actual
`scope × quantification × oracle` form, an exact required-context string map, a closed challenge
domain and one qualification policy. The form belongs to the binding because the same Check may
bear differently on different Claims. Strength is absent: executable Checks demonstrate sampled
behavior; proof remains a property of design mechanisms and contributes to later Claim Judgment.

Routine Claims stop at intent. A verification declaration targeting one is visible as an
inapplicable-verification Finding rather than silently becoming optional evidence.

## Qualification

The Qualification id is the binding id. Exactly one Qualification block belongs to every binding;
duplicate or missing blocks fail validation. Its verdict is `qualified` or `rejected` and records
the expected fingerprint, date, qualifier and review rationale.

Fingerprint inputs are versioned canonical JSON and SHA-256:

- Check: id, ordered methods, terminal proposition, and the sorted semantic identities and source
  fingerprints of all implementations;
- Binding: id, Check id, semantic Claim digest, proposition, form, sorted challenge domain and
  resolved qualification-policy digest;
- Context: the canonical sorted exact string map; and
- Qualification: Check fingerprint, binding fingerprint and context fingerprint.

Paths, lines, mounts, criticality and explanatory prose are excluded. Source edits, Claim semantic
edits, binding semantic edits, policy edits and required-context edits therefore stale the exact
Qualification for the reason they should. Criticality changes obligation without rewriting
credibility.

## Challenge planning

A Challenger states an open form id and the objection it searches for. Challengers are ordinary
tools and are not recursively qualified in alpha 2.

A Challenge Plan names one Challenger and repeatable selectors. Selectors may target Qualifications
from a binding, Check, realization or mechanism, or Claim Judgments from a Claim, realization or
mechanism. The latter selectors are syntactically valid now but cannot resolve until the later
Claim Judgment format exists.

Selection traverses the model, unions matches, sorts and deduplicates exact fingerprints. A
realization or mechanism selector reaches case Claims and their bindings only when the binding's
challenge domain authorizes that relation. Zero resolution is a Finding and never falls back to a
whole suite. Paths, file globs and line numbers are invalid semantic selectors.

An authored plan remains stable when a Qualification is renewed. A later Run plan freezes the
resolved fingerprint so a Challenge Result can target exactly one reviewed decision.

## Source linkage

Language annotations expose only `ImplementsCheck(<check-id>)`. Extractors emit
`check_implementations` with the Check id, compiler-resolved site, file locator, language and exact
enclosing-site source fingerprint. Workspace and federated assembly derive area, mount,
address-kind and address exactly as for realizations.

Multiple source records may implement one Check and compose as a sorted set. An ordinary test with
no marker emits nothing. An implementation marker names no Claim, form or Qualification.

## Breaking transition

The manifest allowlist accepts `check_implementations` and rejects `covers`, `mechanism_covers` and
`observations`. The parser accepts no alpha 1 verification-plan or judgment heading. The export
increments to version 2 and contains no dual fields.

Old result importers are deleted rather than made to emit repository evidence. Mutation, PIT and
SARIF re-enter through Challenger adapters after Run and capability protocols exist. Manual and
operational results re-enter as provider-neutral Runs, not as special Covers records.

The canonical model has only routine Claims and therefore no verification files. Synthetic parser
and extractor fixtures exercise the new contract without creating Azimuth evidence for routine
framework requirements.
