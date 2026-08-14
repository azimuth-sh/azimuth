# Azimuth artifacts

This directory contains the complete repository-owned Azimuth account. Physical colocation makes
one domain area readable without collapsing the logical separation between intent, mechanism,
evidence and judgment (D32).

```text
azimuth/
├── workspace.json        # local areas, derived surfaces and area obligations
├── model/<spec-id>/
│   ├── spec.md            # required
│   ├── design.md          # required for critical intent; otherwise optional
│   ├── verification.md    # deviations, non-test evidence and residuals only
│   └── judgments.md       # agent-tier verdicts when required or performed
├── standards/
│   ├── verification.md   # evidence form required by criticality
│   └── judgment.md       # agent-tier methods such as targeted mutation testing
├── changes/
│   ├── <active-change>/
│   └── archive/
├── explorations/
│   ├── <active-exploration>/
│   └── archive/
└── formats/
    ├── spec.md
    ├── design.md
    ├── verification.md
    └── workspace.md
```

The leaf directory is a **model package**, not a four-file template. `spec.md` is its anchor.
Sibling files are discovered only by the exact names above and remain absent when the corresponding
facet has nothing non-derivable to say. In particular, routine intent normally creates only
`spec.md`, and a standard claim following the project evidence standard needs no empty
`verification.md`.

Every file declares the spec id it belongs to. The id is authoritative; the package path is a
navigation convention. A mismatch or a facet outside its spec's package produces a warning rather
than changing identity. Moving a package therefore changes source locations without changing tags
or expiring judgments when the inspected content is identical.

The four sibling files do not imply four one-to-one records. Requirements, scenarios, mechanisms,
implementation sites and evidence retain their existing declared relations. A reusable control
gets a concern-oriented package such as `security/authentication` or
`resilience/dependency-failures`; its application to business domains must be established by
bindings and evidence rather than inferred from directory proximity.

Format contracts live in `formats/`. Proposed states and immutable history live in `changes/`.
Neither is scanned as the accepted current model.

`workspace.json` uses the same area-and-mount vocabulary as federation without adding a repository
field. It binds independently derived surfaces to enumerators and may require ordinary claims to
have realizations in named areas. Areas are inferred from source paths; tags do not repeat them.

The two standards have different authority. `verification.md` says what evidence must establish
the product claim. `judgment.md` says how the agent audits whether that evidence is discriminating.
Mutation and broad static-analysis runs are therefore configured as judgment challenges and emit
no `Covers` relation. Load or chaos executions may instead emit evidence bindings when each binding
declares a claim-specific assertion, outcome and form.

An exploration is project-level research and decision shaping above individual changes. Its
required anchor is `exploration.md`; optional `research.md` and `change-map.md` appear only when
the material warrants them. It can produce several changes, an experiment or no work. A downstream
proposal points to the exploration and decision ids it carries; the reverse map is derived.

## Federated placement

In a multi-repository project, each repository may own one or more model roots with the same package
contract. A project catalog calls each root a model source and assigns it one intent authority.
Spec identity remains declared globally: two model sources cannot own the same spec.

Source code and evidence are grouped into stable areas with named mounts. Areas are not inferred
from this package tree and do not replace domain-oriented spec ids. Complete assembly and exact
revision receipts are described by D33 and `tools/azimuth/README.md`. Each product checkout carries
a small `azimuth/project-reference.json`; `azimuth project locate` resolves the singular catalog
and reports that repository's exact areas and model sources.

Repository observations also enumerate the exact tracked active and archived change directories
(D34). Complete assembly rejects duplicate change ids across repositories. Project-aware acceptance
compares complete accepted-active and tested-archive worksets; a local archive cannot substitute for
that account.
