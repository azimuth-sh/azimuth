# Azimuth

Azimuth keeps intent, implementation linkage, evidence and agent judgment in one inspectable
project account. The command-line tool checks local model packages and can assemble revision-bound
accounts across repositories without making a hosted service authoritative.

This repository is the canonical source for the framework. The first public alpha is being
qualified; no published artifact should be inferred from the versions currently present in source.

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
