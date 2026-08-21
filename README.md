# Azimuth

Azimuth is an evidence control plane. It keeps durable intent, implementation linkage and reviewed
evidentiary meaning in one inspectable project account without making an execution provider or
hosted service authoritative. The command-line tool validates local model packages, derives
traceability, assembles revision-bound accounts across repositories and verifies provider-neutral
Run bundles without a hosted service.

This is the canonical development and release repository. Framework changes, pull requests and
version history land here, and publication derives artifacts from immutable version tags in the
same history. Version `0.1.0-alpha.1` remains fixed by its tag while `main` carries subsequent
development. The canonical product site is <https://azimuth.sh>.

## Repository layout

| Path | Holds |
|---|---|
| `tools/azimuth/` | dependency-free Rust CLI and core, including federation |
| `tools/extractors/` | language and structural extractors |
| `packages/` | annotation packages; only .NET and TypeScript are selected for the first release |
| `azimuth/formats/` | parser contracts for model and project artifacts |
| `azimuth/standards/` | Qualification policies for non-routine Claims |
| `azimuth/changes/` | active framework changes transferred from development |
| `docs/` | framework definition, decisions, glossary and operating guidance |
| `services/assurance/` | isolated D42 service pending the Run-ledger replacement |
| `experiments/` | self-contained conformance and lifecycle experiments |
| `.agents/skills/` | agent workflows for exploration and change delivery |

The implementations for Go, JVM, Python, Rust annotations and C++ remain experiments. Their
presence in source is not a publication or compatibility promise.

## Published alpha 1 support contract

The immutable alpha 1 tag supports the Rust CLI and core (including federation), the .NET and
TypeScript annotations and extractors, and the assurance API and diagnostic web images defined at
that tag. Main is making an incompatible alpha 2 transition and does not preserve old readers or
command aliases.
Formats, standards, skills and documentation are supported as repository artifacts at the same Git
tag. Alpha contracts may change incompatibly in a later prerelease.

Qualified native CLI targets are Linux x64, macOS ARM64 and Windows x64. Qualified assurance-image
platforms are Linux AMD64 and Linux ARM64. Other source may compile elsewhere, but that is not a
qualified binary or image claim.

The Go, JVM, Python, Rust-annotation and C++ integrations and every tree under `experiments/` are
experimental source. CI exercises them, but version `0.1.0-alpha.1` assigns them no public package
identity or support promise.

## Alpha 2 model

Main positions Azimuth as an evidence control plane. `azimuth validate`,
`azimuth report traceability` and `azimuth export` are the active model commands. The repository
defines requirement and case Claims, sparse Check-to-Claim Evidence Bindings, one Qualification
per binding and semantic challenge declarations. All current framework Claims are routine, so
ordinary engineering tests are not enrolled as Azimuth Checks.

D46 defines the strict [`azimuth-run-bundle`](azimuth/formats/run-bundle.md) version 1 exchange.
`azimuth run verify` checks its shape, identities, selection, reduction and correction history;
`azimuth run inspect` presents the same protocol account without claiming current model acceptance
or Assurance State.

Plan generation, provider execution and native report import remain adapter work. Resolving current
decision targets into a Run plan and applying accepted Runs to Subject-specific state remain
dependent changes. The existing Assurance Service stays isolated on its D42 v1 wire until the
Run-ledger replacement is accepted; there is no compatibility bridge or service export command.

## Development and dogfooding

Generic framework source, model packages, documentation, skills and release workflows are changed
in this repository. Consumer-domain intent remains in the repository that owns it. Real-domain
fixtures such as `azimuth-demo` consume candidate revisions or published versions and return
findings here; this repository never builds, tests or publishes by reading those checkouts.

That boundary keeps the public source independently usable while allowing federation and external
dogfooding to test it. A release is tagged and published from this repository rather than extracted
from a separate development tree.

## Start here

Read `docs/framework.md`, then `docs/decisions.md` for the evidence and revisions behind the
current design. `tools/azimuth/README.md` documents the commands implemented by the CLI.

Run the domain-independent repository checks with:

```sh
./scripts/check.sh
```

Some assurance integration tests require Docker. The script reports that boundary rather than
silently treating an unavailable Docker daemon as a passing engineering check.

## License

Apache-2.0. See `LICENSE`.
