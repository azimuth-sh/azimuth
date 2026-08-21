# Qualification policies

Project policies name the challenge forms required before an Evidence Binding may be accepted as
qualified. They do not run Challengers and do not define provider commands.

## Policy: credible-executable
Required challenge: implementation-perturbation
Required challenge: oracle-perturbation

Use for executable Check bindings whose credibility depends on both the subject implementation and
the discriminating oracle.

## Semantics

Policy ids and challenge-form ids are project-global lower kebab path ids. A policy contains one or
more distinct `Required challenge:` lines. Order is not semantic.

The resolved policy digest participates in every dependent binding fingerprint. Editing a policy
therefore stales Qualifications that relied on it. A clean Challenge Result later means only that
the selected Challenger found no objection in its declared search; it never qualifies an edge by
itself.

All current framework Claims are routine and therefore use no policy. This file establishes the
project vocabulary for synthetic format and tool tests without creating current evidence.
