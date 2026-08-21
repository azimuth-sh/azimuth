# Azimuth artifacts

This directory contains the repository-owned part of the Azimuth evidence control plane. Physical
colocation makes one domain area readable without collapsing intent, mechanism, implementation
linkage and reviewed evidentiary meaning (D32, D43, D45).

```text
azimuth/
├── workspace.json        # local areas, derived surfaces and area obligations
├── model/<spec-id>/
│   ├── spec.md            # required intent
│   ├── design.md          # current mechanisms when required
│   └── verification.md    # Checks, bindings, Qualifications and challenge declarations
├── standards/
│   └── verification.md   # Qualification policies and required challenge forms
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

The leaf directory is a model package, not a fixed-file template. `spec.md` is its anchor. Sibling
files are discovered only by the exact names above and remain absent when their facet has nothing
to say. Every current framework Claim is routine, so the canonical model has no current
`verification.md` files. Routine Claims cannot receive Checks, bindings or Qualifications.

Every file declares the spec id it belongs to. Declared identity is authoritative; package paths
are a navigation convention. Moving a package changes source locations without changing semantic
identities.

Requirement Claims state normative propositions and own criticality. Case-level Claims refine
observable conditions and retain stable `<spec>#<case>` identities. Mechanisms and implementation
sites may relate to several Claims; directory proximity never creates a semantic edge.

For a future non-routine Claim, `verification.md` may declare first-class Checks, sparse
Check-to-Claim Evidence Bindings, one Qualification per binding, Challengers and Challenge Plans.
Source uses `ImplementsCheck(<project-global-check-id>)`; extractors emit implementation identity
only. Unmarked native tests remain ordinary engineering tests.

Run envelopes, provider adapters, normalized outcomes and Assurance Service ingestion are not
current repository formats. D43 defines their semantic boundary, while dependent changes will
define their executable contracts. The existing service remains isolated on its D42 v1 wire until
the Run-ledger replacement is accepted.

`workspace.json` uses the same area-and-mount vocabulary as federation without adding a repository
field. It binds independently derived surfaces to enumerators and may require non-routine Claims to
have realizations in named areas. Areas derive from source paths; markers do not repeat them.

Format contracts live in `formats/`. Proposed states and immutable history live in `changes/`.
Neither is scanned as the accepted current model.

An exploration is project-level research and decision shaping above individual changes. Its
required anchor is `exploration.md`; optional research and change maps appear only when warranted.
A downstream proposal points to the exploration and decision ids it carries.

## Federated placement

In a multi-repository project, each repository may own one or more model roots with the same
package contract. A project catalog assigns every model source one intent authority. Two model
sources cannot own the same spec.

Source is grouped into stable areas with named mounts. Complete assembly and exact revision
receipts are described by D33 and `tools/azimuth/README.md`. Each product checkout carries a small
`azimuth/project-reference.json`; `azimuth project locate` resolves the singular catalog and
reports that repository's areas and model sources.

Repository accounts enumerate tracked active and archived change directories (D34). Complete
assembly rejects duplicate change ids. Project-aware acceptance compares complete accepted-active
and tested-archive worksets; a local archive cannot substitute for that account.
