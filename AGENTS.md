# Azimuth

Canonical development and release repository for Azimuth. Framework source, model packages, tooling, skills, documentation and release workflows evolve here; version tags identify immutable published states.

## Orientation

| Path | Holds |
|---|---|
| `docs/framework.md` | derived account of the framework; start here |
| `docs/glossary.md` | bounded terminology |
| `docs/change-process.md` | change delivery, evidence and rollout guidance |
| `docs/assurance-extensions.md` | role and authority boundaries extension work must preserve |
| `contracts/` | parser contracts |
| `azimuth/model/` | the framework's own accepted intent and mechanisms |
| `azimuth/changes/` | active changes; one identifier has one authority |
| `azimuth/changes/archive/` | the decision record: one archived change per accepted transition |
| `azimuth/explorations/` | non-normative pre-commitment research; never authority |
| `tools/azimuth/` | Rust CLI and core |
| `tools/extractors/` | language and structural extractors |
| `services/assurance/` | current Run ledger plus an isolated earlier service boundary |
| `experiments/` | synthetic, self-contained conformance evidence |

## Authority order

When two artifacts disagree, the earlier item wins:

1. `tools/azimuth/src/` and `tools/azimuth/tests/` — behaviour, and the tests that pin it;
2. `contracts/` — parser contracts, pinned by frozen-vector tests;
3. `azimuth/model/` — the framework's own accepted intent and mechanisms;
4. `azimuth/changes/archive/<id>/` — why one accepted transition happened;
5. `docs/` — derived prose, which holds no authority of its own.

Read the format contract for the area you are changing before structural work. Derived prose is never the reason to keep a behaviour. There is no monolithic decision log: an accepted change's `proposal.md`, `design.md` and `outcome.md` are its record, and archived changes are immutable.

## Working rules

- State claims as falsifiable propositions and distinguish decided, proposed and open work.
- A Requirement owes only the reviews it declares. `Verification: reviewed` owes Checks bound to every Scenario and an independent Verification Review; `Realization: reviewed` owes `Realizes` sites (and `ImplementsMechanism` sites where a `realization.md` exists) and an independent Realization Review. A Requirement that declares neither is specified and owes no review or evidence. `verification.md` owns Checks, inline Check–Scenario Evidence Bindings, optional Challenge declarations and package declarations. The latest accepted review of each kind is a sealed head in the module's `verification-review.md` or `realization-review.md`, written only at finalization; the Azimuth Assurance Service retains review history and computes deployment Requirement Status from applicable evidence. No source marker or execution creates an accepted review.
- Use `azimuth` for the tool and reserve Check for a deliberately enrolled verification method. Commands for the current model are `azimuth validate`, `azimuth report traceability` and `azimuth export`.
- Configure short-lived provider adapters explicitly in strict `azimuth/adapters.json`; core never discovers executables through `PATH`, invokes a shell or inherits the ambient environment. Adapters are not daemons, webhook hosts or long-running supervisors.
- Use `azimuth adapter verify [--config <file>]` for the configured description handshake. Use `azimuth run plan --request <file>` to resolve Check-only, Challenge-only or mixed requests from the complete unselected model, then `azimuth run execute --plan <file>` or `azimuth run import --plan <file> --input <id>=<file>` for one bounded provider exchange. Planning has no partial-model or `--only` mode.
- Adapter content and import inputs are staged and hashed from the same opened streams. Every invocation requires supported fresh process-group isolation before spawn, one bounded core exchange whose deadline covers request writing, concurrent capped-stream draining and core's wait, and complete response validation before atomic output.
- Core signals the process group on every terminal path and cleans members and inherited pipes while they remain in the group. Authorized descendants may escape with `setsid`, `setpgid` or equivalent; core does not guarantee their termination. This is not non-escapable descendant containment, a filesystem or network sandbox, daemon supervision or hostile-code isolation.
- Use `azimuth run verify --bundle <file>...` for standalone Run-protocol consistency and `azimuth run inspect --bundle <file>...` for a deterministic local account. These commands do not establish current model authority or Assurance State.
- Challenge planning resolves the Challenges declared in `verification.md`, their semantic scope and accountable launch inputs. Each request names an explicit configured capability, finite units and target cap; core never auto-selects a capability, form, provider selector or broader fallback.
- Execute and import preserve protocol-valid adverse facts. A clean Challenge Result is only a negative search fact. `deferred` is not a result.
- The current `run-ledger` service provides project-scoped ingestion, append-only correction and review histories, main-model publication (the default model is the highest `(main_sequence, publication_revision)` publication; ratification is recorded in git) and exact-Verification-Target Assurance State. `azimuth assurance authority` exports the full current model projection. `azimuth run ingest` is not a command. There is no automatic expiry, cache-validity, cross-target reuse or historical applicability inference.
- A marker-derived mechanism uses the project-global one-argument `ImplementsMechanism(<mechanism-id>)` annotation and an extractor-derived, ecosystem-semantic qualified `site`, exact path-free typed binding and companion Artifact. Extractors fail closed on ambiguity, unsupported semantic identity or non-normal/outside-root locators; a source path never disambiguates the site.
- Evidence precedes notation: no mechanism enters the model until two structurally different concerns demand it in prose.
- Framework development, pull requests and version history are authoritative in this repository. Do not extract or synchronize generic source from a consumer fixture.
- Tooling and its tests use synthetic fixtures. They never depend on consumer vocabulary, paths or checkouts.
- Consumer-domain intent and real-domain fixtures remain in their owning repositories. Their dogfood findings may motivate changes here but do not become executable repository dependencies.
- Specs are organized by domain area, identifiers are declared rather than path-derived, and derivable artifacts are not maintained by hand.
- A federated local project account is incomplete when required workset inputs are missing. Never finalize a project account from `--local` output.
- Model authority follows intent. Change authority is singular in a complete project account.
- Exploration precedes commitment for uncertain multi-change work.
- Validate `work-packages.md` before delegation. Workers edit only their declared non-overlapping paths and never finalize or archive.
- Do not write, run or offer tests unless directly asked. This covers new tests, test infrastructure and executing existing suites such as `cargo test` or `./scripts/check.sh`. The framework is in a design phase, and test ceremony written now encodes decisions that are still open. On finding a coverage gap, record it and continue; do not close it. Verify work with the product's own commands and by inspection instead.
- There is no backward-compatibility obligation during the alpha design phase unless an accepted change states one explicitly.
- Enable verification packages explicitly in the project or workspace. Package source tags use their package namespaces; ecosystem emitters derive schemas and exact producer sites. `azimuth surface verify` validates complete enumeration and expectation reports against those descriptors. Compact method prose is preserved, never translated into a fabricated Terminal.
- Heavy code analysis belongs to an extractor, never to core. AST, call-graph and schema access happen in the ecosystem whose compiler API is already present; core reads manifests only. That division is what keeps the zero-dependency rule affordable rather than heroic.
- Every parse failure names the file, the line and what was expected. Strict formats are only tolerable when their errors are precise, and there is no parser library to supply that for free.
- Review declarations need counter-pressure before the first reviewed Requirement lands. Whoever declares a review does not pay for it, so without a cap or an explicit check at the change boundary every Requirement drifts to declaring both and the declarations stop carrying information. No mechanism enforces this yet.
- Keep each mechanism usable alone. Adding one must enrich existing validation without re-authoring existing artifacts, mixed adoption levels must coexist with no coordinating centre, and adopting Azimuth on an existing codebase must be possible by baselining current Findings and forbidding new ones.
- A Verification Challenge has no aggregate score. Findings are reviewed against the specific predicate they attack; a project-wide percentage or threshold rewards irrelevant findings and punishes deliberately untested infrastructure. A clean Challenge Result is likewise only a negative search fact.
- Re-review follows definition drift, not re-execution. Re-running unchanged Checks needs no new Verification Review and no commit; only a change to a blocking dependency of the review does.

## Adapter command boundary

```text
azimuth adapter verify [--config <file>]
azimuth run plan --request <file> [--model <dir>] \
  [--workspace <file>] [--manifest <file>...] [--config <file>] [--out <file>]
azimuth run execute --plan <file> [--predecessor <bundle>...] \
  [--config <file>] [--out <file>]
azimuth run import --plan <file> --input <id>=<file>... \
  [--predecessor <bundle>...] [--config <file>] [--out <file>]
```

`--manifest`, `--predecessor` and `--input` are repeatable where shown. Execute accepts only an execute launch; import accepts only an import launch and at least one exact input. A successful plan or provider exchange writes only after complete validation and atomic replacement. Exit one reports semantic, identity, content, transport or bundle mismatch; exit two reports command or schema failure. Neither nonzero class leaves the requested output file.

## Writing and commits

Comments explain why; names describe purpose. Preserve revisions rather than silently rewriting prior reasoning.

Commit subjects are imperative and scoped (`docs:`, `tools:`, `assurance:`). Bodies explain what changed, why it changed and any inconvenient findings surfaced by the work.
