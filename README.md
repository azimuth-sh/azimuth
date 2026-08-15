# Azimuth

Azimuth keeps intent, implementation linkage, evidence and agent judgment in one inspectable
project account. The command-line tool checks local model packages and can assemble revision-bound
accounts across repositories without making a hosted service authoritative.

This repository is the frozen distribution source for version `0.1.0-alpha.1`. The selected first
public alpha contract is not published until the release operation verifies every registry target.
The canonical product site is <https://azimuth.sh>.

## Repository layout

| Path | Holds |
|---|---|
| `tools/azimuth/` | dependency-free Rust CLI and core, including federation |
| `tools/extractors/` | language extractors and external-evidence importers |
| `packages/` | annotation packages; only .NET and TypeScript are selected for the first release |
| `azimuth/formats/` | parser contracts for model and project artifacts |
| `azimuth/standards/` | verification and judgment standards |
| `azimuth/changes/` | active framework changes transferred from development |
| `docs/` | framework definition, decisions, glossary and operating guidance |
| `services/assurance/` | optional execution ledger and diagnostic web application |
| `experiments/` | self-contained conformance and lifecycle experiments |
| `.agents/skills/` | agent workflows for exploration, change delivery and verification |

The implementations for Go, JVM, Python, Rust annotations and C++ remain experiments. Their
presence in source is not a publication or compatibility promise.

## First-alpha support contract

The supported alpha surface comprises the Rust CLI and core (including federation), the .NET and
TypeScript annotations and extractors, and the optional assurance API and diagnostic web images.
Formats, standards, skills and documentation are supported as repository artifacts at the same Git
tag. Alpha contracts may change incompatibly in a later prerelease.

Qualified native CLI targets are Linux x64, macOS ARM64 and Windows x64. Qualified assurance-image
platforms are Linux AMD64 and Linux ARM64. Other source may compile elsewhere, but that is not a
qualified binary or image claim.

The Go, JVM, Python, Rust-annotation and C++ integrations and every tree under `experiments/` are
experimental source. CI exercises them, but version `0.1.0-alpha.1` assigns them no public package
identity or support promise.

## Start here

Read `docs/framework.md`, then `docs/decisions.md` for the evidence and revisions behind the
current design. `tools/azimuth/README.md` documents the commands implemented by the CLI.

Run the domain-independent repository checks with:

```sh
./scripts/check.sh
```

Some assurance integration tests require Docker. The script reports that boundary rather than
silently treating an unavailable Docker daemon as passing evidence.

## License

Apache-2.0. See `LICENSE`.
