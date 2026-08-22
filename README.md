# Azimuth

Azimuth is an evidence control plane. It keeps durable intent, implementation linkage and reviewed
evidentiary meaning in one inspectable project account without making an execution provider or
hosted service authoritative. The command-line tool validates local model packages, derives
traceability, assembles revision-bound accounts across repositories, plans Check Runs and invokes
explicitly configured short-lived adapters without requiring a hosted service.

This is the canonical development and release repository. Framework changes, pull requests and
version history land here, and publication derives artifacts from immutable version tags in the
same history. Each published version remains fixed by its tag while `main` carries subsequent
development. The canonical product site is <https://azimuth.sh>.

## Repository layout

| Path | Holds |
|---|---|
| `tools/azimuth/` | dependency-free Rust CLI and core, including federation |
| `tools/extractors/` | language and structural extractors |
| `packages/` | annotation packages; only .NET and TypeScript are selected for the first release |
| `azimuth/formats/` | parser contracts for model and project artifacts |
| `azimuth/standards/` | Decision Policies and the current Challenge Schedule |
| `azimuth/changes/` | active framework changes transferred from development |
| `docs/` | framework definition, decisions, glossary and operating guidance |
| `services/assurance/` | isolated D42 service pending the Run-ledger replacement |
| `experiments/` | self-contained conformance and lifecycle experiments |
| `.agents/skills/` | agent workflows for exploration and change delivery |

The implementations for Go, JVM, Python, Rust annotations and C++ remain experiments. Their
presence in source is not a publication or compatibility promise.

## Published alpha 2 support contract

The immutable alpha 2 tag supports the Rust CLI and core (including federation), the .NET and
TypeScript annotations and extractors, and the assurance API and diagnostic web images defined at
that tag. Alpha 2 is an incompatible transition from alpha 1: old readers, formats and command
aliases are removed rather than retained, so an alpha 1 model does not load.
Formats, standards, skills and documentation are supported as repository artifacts at the same Git
tag. Alpha contracts may change incompatibly in a later prerelease.

Qualified native CLI targets are Linux x64, macOS ARM64 and Windows x64. Qualified assurance-image
platforms are Linux AMD64 and Linux ARM64. Other source may compile elsewhere, but that is not a
qualified binary or image claim.

Durable Run ingestion, authorization, retention and Subject-specific Assurance State are not in
this release. The assurance images ship the isolated D42 service, which the Run-ledger change will
replace; alpha 2 claims no ledger, no hosted event gateway and no production provider adapters.

The Go, JVM, Python, Rust-annotation and C++ integrations and every tree under `experiments/` are
experimental source. CI exercises them, but version `0.1.0-alpha.2` assigns them no public package
identity or support promise.

## Alpha 2 model

Main positions Azimuth as an evidence control plane. `azimuth validate`,
`azimuth report traceability` and `azimuth export` are the active model commands. The repository
defines requirement and case Claims, sparse Check-to-Claim Evidence Bindings, one Qualification
per binding, one total-composition Claim Judgment per non-routine case Claim and semantic Challenge
declarations. Decision Policies require open Challenge forms; one separate schedule assigns each
required or declared form exactly once to `gate` or `scheduled`. All current framework Claims are
routine, so ordinary engineering tests are not enrolled as Azimuth Checks.

D46 defines the strict [`azimuth-run-bundle`](azimuth/formats/run-bundle.md) version 1 exchange.
`azimuth run verify` checks its shape, identities, selection, reduction and correction history;
`azimuth run inspect` presents the same protocol account without claiming current model acceptance
or Assurance State.

D47 adds strict [`adapter`](azimuth/formats/adapter.md) configuration and
[`azimuth-run-launch-plan`](azimuth/formats/run-launch-plan.md) version 1. Core loads the complete
model, derives a provider-neutral Check and Challenge Plan and binds it to exact configured
capability routes.
`azimuth adapter verify`, `azimuth run plan`, `azimuth run execute` and `azimuth run import` expose
the short-lived provider boundary. Adapter content and imports are staged from the same streams
core hashes. On supported hosts, every exchange uses a fresh process group, bounded output and one
deadline for core request, response, diagnostics and wait activity. Core signals remaining group
members on every terminal path. Authorized adapter code can deliberately use `setsid`, `setpgid` or
an equivalent to leave the group. It cannot extend core's wait beyond the deadline, but Azimuth does
not guarantee its termination. This is not non-escapable descendant containment, daemon
supervision, hostile-code isolation or a filesystem or network sandbox.

An adapter-returned protocol-valid `timed-out` Run fact exits zero only when its complete response
arrives within the host deadline. A host-enforced deadline is a transport timeout, exits one and
publishes no bundle.

Planning accepts strict Check-only, Challenge-only and mixed requests. It loads the complete
unselected model, resolves all seven Qualification and Claim Judgment selector forms, preserves
every candidate disposition and requires the fixed requested Plan union to cover each selected
decision's required forms. Every Challenge freezes its lane, semantic scope and accountable launch
inputs; callers name each capability explicitly and core never widens an unresolved selector.

Protocol-valid `clean`, `findings` and `inconclusive` Challenge Results are execution facts. Clean
is only a negative search fact and creates neither credibility nor product evidence. Every
Challenge omitted from a partial, cancelled or timed-out Run has one exact diagnostic and no
fabricated Result; scheduled omission is allowed deferral, while gate omission records execution
failure. The [Challenge-planning conformance](experiments/challenge-planning/README.md) exercises
these boundaries through public commands.

`model.extract` execution, durable Run ingestion, authorization, retention and Subject-specific
Assurance State remain deferred. Current planning defines no cache, cadence or cross-Subject reuse
semantics. Adapters remain bounded short-lived processes; there is no daemon, webhook or
long-running adapter boundary. The existing Assurance Service stays isolated on its D42 v1 wire
until the Run-ledger replacement is accepted; there is no compatibility bridge or service export
command.

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
