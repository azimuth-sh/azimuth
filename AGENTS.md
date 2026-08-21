# Azimuth

Canonical development and release repository for Azimuth. Framework source, model packages,
tooling, skills, documentation and release workflows evolve here; version tags identify immutable
published states. The repository must build, test and publish without a checkout of Drim, the
ride-hailing fixture or any other consumer domain. Immutable citations may preserve provenance;
executable or acceptance dependencies may not.

## Orientation

| Path | Holds |
|---|---|
| `docs/framework.md` | derived account of the framework; start here |
| `docs/decisions.md` | authoritative design decisions and their revision history |
| `docs/glossary.md` | bounded terminology |
| `docs/change-process.md` | change delivery, evidence and rollout guidance |
| `azimuth/formats/` | parser contracts |
| `azimuth/standards/` | Qualification policies for non-routine Claims |
| `azimuth/changes/` | active changes; one identifier has one authority |
| `tools/azimuth/` | Rust CLI and core |
| `tools/extractors/` | language and structural extractors |
| `services/assurance/` | isolated D42 service pending the Run-ledger change |
| `experiments/` | synthetic, self-contained conformance evidence |

Read `docs/decisions.md` before structural work. The framework document is derived and never
overrides a decision.

## Working rules

- State claims as falsifiable propositions and distinguish decided, proposed and open work.
- Use `azimuth` for the tool and reserve Check for a deliberately enrolled verification method.
  Commands for the current model are `azimuth validate`, `azimuth report traceability` and
  `azimuth export`.
- Evidence precedes notation: no mechanism enters the model until two structurally different
  concerns demand it in prose.
- Framework development, pull requests and version history are authoritative in this repository.
  Do not extract or synchronize generic source from a consumer fixture.
- Tooling and its tests use synthetic fixtures. They never depend on consumer vocabulary, paths or
  checkouts.
- Consumer-domain intent and real-domain fixtures remain in their owning repositories. Their
  dogfood findings may motivate changes here but do not become executable repository dependencies.
- Specs are organized by domain area, identifiers are declared rather than path-derived, and
  derivable artifacts are not maintained by hand.
- A federated local project account is incomplete when required workset inputs are missing. Never
  finalize a project account from `--local` output.
- Model authority follows intent. Change authority is singular in a complete project account.
- Exploration precedes commitment for uncertain multi-change work.
- Validate `work-packages.md` before delegation. Workers edit only their declared non-overlapping
  paths and never finalize or archive.
- There is no backward-compatibility obligation during the alpha design phase unless an accepted
  change states one explicitly.

## Writing and commits

Wrap prose at 100 columns. Comments explain why; names describe purpose. Preserve revisions rather
than silently rewriting prior reasoning.

Commit subjects are imperative and scoped (`docs:`, `tools:`, `assurance:`). Bodies explain what
changed, why it changed and any inconvenient findings surfaced by the work.
