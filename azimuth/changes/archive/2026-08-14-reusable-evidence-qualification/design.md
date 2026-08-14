# Design: reusable-evidence-qualification

## Stable and changing records

An evidence definition owns the proposition, verification form, oracle, configuration inputs and
the execution-context constraints under which a successful result would be meaningful. Its
fingerprint changes only when those semantics change.

A qualification records an agent verdict over exactly that definition fingerprint. `qualified`
means a future applicable successful execution may establish the configured gate; it does not mean
the product claim has already been observed.

An observation is an immutable execution fact. It names the definition, its fingerprint, the exact
source or artifact subject, lifecycle stage, context, time, optional expiry and outcome. Replacing
an observation never rewrites history.

## Derived gate

A gate is open only when:

1. the definition is qualified at its current fingerprint;
2. an observation references the same fingerprint;
3. its subject and context match the requested lifecycle target;
4. it is observed and not expired;
5. every claim-specific outcome is satisfied;
6. no current challenge finding targets the definition or its evidence subject.

The evaluator reports the first complete set of deterministic reasons rather than a single green
score. A changed definition invalidates qualification. A new successful observation under an
unchanged definition renews execution state without renewing semantic judgment.

## Prototype boundary

The prototype is a pure library plus executable fixtures. It deliberately has no persistence or
network protocol. If the record split fails, no database schema or public API needs migration. If
it succeeds, the same records become the domain boundary of the reference service.
