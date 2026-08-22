# Exploration: Drim dogfooding findings

Id: drim-dogfood Status: exploring Created: 2026-08-16

## Objective

Record observations made while adopting and using published Azimuth versions in Drim. The record keeps consumer evidence available for later analysis without treating an observation, inference or possible remedy as accepted framework truth.

## Boundaries

This exploration covers onboarding, initialization, agent guidance, migration friction, modeling gaps and tool behavior observed in `drim-dev/drim-dev`. Each finding identifies the Azimuth version and context that produced it. It also records follow-up observations from using Azimuth to act on the dogfood findings in the canonical framework repository.

Registry reservation, artifact publication and other release-production findings remain outside this exploration because the canonical alpha-release exploration already owns that activity. This record is not a roadmap, issue tracker or source of current framework behavior.

## Existing context

The consumer exercised the public `0.1.0-alpha.1` release. Adoption has not yet carried the planned referral feature through the complete Azimuth change lifecycle, so the entries below are adoption findings rather than evidence about that lifecycle as a whole.

## Findings

### DF-001 — Consumer installation is not self-contained

Version: `0.1.0-alpha.1` Context: initial installation in `drim-dev/drim-dev` Status: open

**Observation.** The crate README documents installation from a source checkout. The public repository directs readers to repository-local framework documents, while the consumer start path does not explain the native release archives or distinguish binary installation from compilation by `cargo install`. Dogfooding required separate explanations of the command, crate archive contents and precompiled archives.

**Inference.** A new consumer cannot reliably discover every supported installation path from the published package documentation alone.

**Consequence.** Installation can succeed while the consumer remains uncertain about which artifact was installed, whether compilation is expected and when a qualified native archive applies.

**Falsifier.** A first-time consumer installs either supported form and accurately explains its provenance and platform qualification using only the published consumer entry point.

**Disposition.** Retain for later analysis; no remedy is selected.

### DF-002 — Initialization does not bootstrap an agent workflow

Version: `0.1.0-alpha.1` Context: `azimuth init` in `drim-dev/drim-dev` Status: open

**Observation.** Initialization creates the model, change, exploration and standards directories, `standards/verification.md`, and `workspace.json`. It does not install Azimuth skills, format contracts, operating documentation or agent instructions. The initialized Drim repository contains only the two generated files under `azimuth/`.

**Inference.** A coding agent reading only the consumer repository cannot discover how to propose, apply, verify and archive an Azimuth change.

**Consequence.** Successful initialization does not make the released workflow operable by a cold coding agent without additional, manually supplied context.

**Falsifier.** A cold agent correctly follows the released change workflow using only the initialized consumer repository.

**Disposition.** Retain for later analysis; no remedy is selected.

### DF-003 — Generated configuration is not self-explanatory at its point of use

Version: `0.1.0-alpha.1` Context: review of files produced by `azimuth init` Status: open

**Observation.** Understanding the generated verification standard and empty workspace required separate explanations. The generated workspace does not state when areas, surfaces or realization obligations should be introduced.

**Inference.** Initialization establishes valid syntax but does not supply enough local context for a first-time adopter to make sound modeling decisions.

**Consequence.** A consumer can retain valid defaults without understanding what they require or when the empty architectural declarations must change.

**Falsifier.** A first-time adopter accurately explains the two generated files and makes a sound decision about leaving or changing each one using only repository-local initialized material.

**Disposition.** Retain for later analysis; no remedy is selected.

### DF-004 — Replacement of pre-alpha state is manual

Version: `0.1.0-alpha.1` Context: replacement of earlier Azimuth and OpenSpec integration in `drim-dev/drim-dev` Status: accepted limitation

**Observation.** Drim contained pre-alpha Azimuth linkage, build integration and OpenSpec workflow artifacts. They required a repository audit and selective removal while educational content had to remain. `azimuth init` is additive and performs no migration or stale-state detection.

**Inference.** Adoption from an earlier experiment requires an explicit audit of the consumer's previous state.

**Consequence.** Stale integration can survive initialization or broad cleanup can remove content that merely discusses the superseded tools.

**Falsifier.** A consumer with structurally similar pre-alpha state can initialize the public alpha without a manual audit and without retaining technical residue or deleting unrelated content.

**Disposition.** Retain as an observed consequence of the alpha's deliberate no-compatibility boundary; it does not by itself establish a framework defect.

### DF-005 — One ordered strength cannot require complementary evidence

Version: `0.1.0-alpha.1` Context: modeling settlement reconciliation with both tests and operational detection Status: open

**Observation.** Evidence strength is one ordered value: `detection < demonstration < proof`. A demonstration therefore satisfies a detection floor. Azimuth checks the detector and detector-test bindings when a detection item is declared, but deleting that entire item creates no hole when demonstration evidence still satisfies the claim's required strength.

**Inference.** The verification policy cannot express an obligation for both demonstration and detection evidence on one claim.

**Consequence.** If monitoring is required only as complementary assurance, its declaration can disappear without making the model incomplete. When notification is durable system behavior, a separate behavioral claim can preserve that obligation, but that does not express a general requirement for complementary assurance forms.

**Falsifier.** Show that the released model can require demonstration and detection independently for one claim, such that removing either evidence form creates a machine finding while the other remains.

**Disposition.** Retain for later analysis. Evidence-before-notation prevents generalizing one observation into new syntax before a second structurally different concern demands it.

### DF-006 — The ordinary-work boundary is not discoverable

Version: `0.1.0-alpha.1` Context: applying the canonical development-authority revision in `azimuth-sh/azimuth` Status: open

**Observation.** Current guidance says that routine work with clear intent may begin with a lightweight change and that a framework mechanism or operating document with no intent delta uses an explicit framework-only change. It does not state when an edit is ordinary repository work that needs no Azimuth artifact. During dogfooding, an agent created a proposal, plan and outcome and ran the complete release and Docker qualification suite for an instruction and decision revision. The authority revision was structurally significant, but the selected evidence included checks with no discriminating relationship to the edited documents.

**Inference.** The workflow biases agents toward treating every repository edit as an Azimuth semantic change and toward selecting evidence by repository scope rather than by what could falsify the edit.

**Consequence.** Small or nonsemantic work can acquire change directories, lifecycle bookkeeping and expensive checks that cannot detect a plausible error in the edit. That cost weakens the framework's ceremony falsifier because unnecessary accounting is attributed to ordinary use.

**Falsifier.** A cold agent correctly distinguishes ordinary repository work, a routine semantic change and a standard or critical semantic change, creates no Azimuth artifact for the ordinary case, and selects checks that can reject plausible errors in each remaining case using only the published workflow guidance.

**Disposition.** Retain for later analysis. No classification rule, CLI behavior or change-lifecycle revision is selected by this finding.

### DF-007 — Published library consumption is not a development lane

Version: `0.1.0-alpha.1` Context: Rust dependency and release-surface review in `azimuth-sh/azimuth` Status: open

**Observation.** The published `azimuth` crate contains both the CLI binary and a Rust library, and the release catalog names that artifact `rust-cli-core`. Normal repository development does not consume it through crates.io: `azimuth-assurance-domain` uses a path dependency on `tools/azimuth`. The Rust annotations and emitter remain experimental path-based source and are not published. Release qualification validates retained package contents, but there is no mandatory registry-only development lane that consumes exact published Rust, NuGet and npm library versions without local path, project or workspace overrides.

**Inference.** Azimuth qualifies what it publishes but does not continuously exercise the released libraries as a normal consumer while developing the next version. Self-dogfooding therefore covers candidate integration without covering registry consumption between releases.

**Consequence.** Published API, dependency, packaging or installation defects can remain invisible to ordinary development builds. The repository can also evolve around local source relationships that consumers cannot reproduce from the selected public artifacts.

**Falsifier.** A clean mandatory development lane resolves exact supported library versions from their public registries with no local override, exercises their consumer contracts, and fails when a published artifact is absent or incompatible. A separate candidate lane continues to exercise the source versions being changed.

**Disposition.** Retain for later analysis. The finding does not decide whether to split the core crate, publish the Rust annotations or emitter, expose assurance-domain APIs, or define the two development lanes.

### DF-008 — Skill orientation depends on documents no project is guaranteed to have

Version: `0.1.0-alpha.1` Context: consumer adoption guidance for `drim-dev/drim-dev`; skill review in `azimuth-sh/azimuth` Status: open

**Observation.** Step 1 of `azimuth-propose` directs the agent to read `AGENTS.md` and `azimuth/changes/README.md`. The second file exists in no commit of `azimuth-sh/azimuth` on any branch and is not produced by `azimuth init`, so no project has ever had it. Two framework documents nevertheless cite it as authoritative: `docs/framework.md` attributes the parsed change protocol and lifecycle commands to it under D21.4 and D35, and `docs/change-process.md` names it as prevailing over that guide. The absence is already recorded as a departure in `azimuth/changes/canonical-development-authority/outcome.md`, where the change proceeded on the CLI contract and an archived precedent instead. The first file is a runtime-specific filename: the skills are held under `.agents/skills/` for portability, but agent runtimes differ in which project instruction file they load, and some load one before the skill runs at all. The remaining six skills name no repository document and read change artifacts, model packages and CLI output.

**Inference.** Skill orientation is coupled to hand-maintained documentation whose presence the tool does not establish, in one case to a document that has never existed. The coupling is not uniform across the skill set, which suggests it is incidental to how `azimuth-propose` was written rather than a decided contract for what a skill may require.

**Consequence.** A consumer agent follows an instruction that cannot be satisfied and continues without the orientation the step intended, silently and with no signal that anything was missed. Because two framework documents delegate the change protocol to the same absent file, a reader directed there from `docs/framework.md` reaches nothing, and a project-local copy would resolve one citation to a different document per project.

**Falsifier.** Every document a skill directs an agent to read is either produced by `azimuth init`, carried by the release, or absent by an explicit and stated allowance; and a cold agent in an initialized consumer project completes the propose step with no unresolved reference.

**Disposition.** Retain for later analysis; no remedy is selected. The finding does not decide whether to write the missing document in this repository, whether skills should name project instruction files at all, or whether a skill's readable inputs should be restricted to tool-guaranteed artifacts and CLI output. It does not authorize any change to `azimuth init`; DF-002 and DF-003 record adjacent initialization observations and are not merged into this one.

### DF-009 — Two framework documents delegate authority to an absent document

Version: `0.1.0-alpha.1` Context: documentation review in `azimuth-sh/azimuth` prompted by DF-008 Status: open

**Observation.** `docs/framework.md:126` states that the parsed change protocol and its lifecycle commands are in `azimuth/changes/README.md`, citing D21.4 and D35. `docs/change-process.md:5` names the same file, together with the three format contracts and `docs/decisions.md`, as authoritative wherever that guide disagrees with it. The file exists in no commit on any branch. Both documents are released repository artifacts at the same version tag. This is the same absent file as DF-008; that finding concerns what a skill instructs an agent to read, while this one concerns published documentation delegating its own authority.

**Inference.** The two citations were written against an intended document rather than an existing one, and no check compares a repository-relative reference in released documentation against the tree that ships with it.

**Consequence.** A reader following `docs/framework.md` to the change protocol reaches nothing, and the guidance that `docs/change-process.md` declares subordinate is the only guidance that exists. The authoritative account of a lifecycle the framework considers central is therefore unwritten while two documents present it as written elsewhere.

**Falsifier.** Every repository-relative reference in released documentation resolves in the tree at the same version tag, and the change protocol named by D21.4 and D35 is readable at the location its citations give.

**Disposition.** Retain for later analysis; no remedy is selected. The finding does not decide whether the missing document is written, whether the two citations are redirected to existing material, or whether reference resolution becomes a repository check.

### DF-010 — Initialization does not reveal the language toolchain a model needs

Version: `0.1.0-alpha.1` Context: `azimuth init` followed by a first spec in a synthetic project Status: open

**Observation.** After `azimuth init`, the initialized tree contains `standards/verification.md` and `workspace.json` and nothing else. Adding one critical requirement and running `azimuth check` reports `unrealized` and `uncovered` with the explanations "no production code realizes this claim" and "no evidence covers this claim", exit 1. Nothing in the initialized project, the command output or the errors states that linkage is established by annotation packages and an extractor, names `@azimuth-sh/annotations`, `@azimuth-sh/emit`, `Azimuth.Annotations` or `Azimuth.Emit`, says that their versions must match the CLI, says that a manifest must be produced and passed with `--manifest`, or says that only .NET and TypeScript are supported while five further extractors exist as experimental source. The CLI usage lists `--manifest <file>  a linkage manifest` and names no producer for one. DF-002 records that initialization installs no workflow, skills or operating documentation; this finding concerns the language-side toolchain that linkage requires and that no skill supplies either.

**Inference.** The initialized project makes the model artifacts discoverable and leaves the mechanism that connects them to source entirely undiscoverable from within the project. The check correctly reports the holes but describes them in terms of code and evidence rather than of the linkage step that would resolve them.

**Consequence.** An agent that reaches these two errors without external context can conclude that conforming production code and tests already exist and that the tool is wrong, or write code and tests and observe the identical errors afterward, because the missing element is a tag and an emitted manifest rather than an implementation. Recovery depends on knowledge no artifact in the project carries.

**Falsifier.** A cold agent in an initialized consumer project, given a spec and conforming source, identifies the required annotation package and extractor for its ecosystem, emits a manifest and clears both holes using only repository-local and released material.

**Disposition.** Retain for later analysis; no remedy is selected. The finding does not decide whether initialization emits ecosystem guidance, whether hole explanations name the linkage step, whether the CLI names manifest producers, or whether the supported-ecosystem boundary is stated at initialization. It authorizes no change to `azimuth init`.

### DF-011 — The obligation to re-establish linkage in CI is stated without a consumer path

Version: `0.1.0-alpha.1` Context: consumer CI design for `drim-dev/drim-dev`; reference-implementation review in `azimuth-sh/azimuth` Status: open

**Observation.** `docs/change-process.md:141` states that CI extracts linkage and runs the machine tier, and line 207 states that test evidence is re-established by CI. The same document declares at line 5 that it is not a Git workflow or a deployment system. No released document shows a consumer how to satisfy the obligation. The skills carry the coupling: `azimuth-apply` directs the agent to emit every relevant language manifest and run `azimuth check` over their union, and `azimuth-archive` requires fresh manifests. Both address one agent working through one change rather than repository automation. This repository implements the pattern correctly in `release/check.sh`, which generates linkage manifests and runs `azimuth check` over them in one script, with `.gitignore` excluding `.azimuth/release/*` so manifests are build artifacts; `scripts/check.sh:67` invokes it and `.github/workflows/ci.yml` runs that. That script emits through this repository's Python release orchestration rather than the published extractors, and no released document identifies it as a pattern a consumer could follow.

A manifest is trusted input. `azimuth check --manifest` over a hand-written manifest naming `src/refund.ts` and `test/refund.test.ts`, in a tree containing neither file, reported one critical claim with `no holes` and exit 0. `tools/azimuth/README.md` states the machine tier's trust boundary, and `check.rs:1128` returns no `unjudged` holes when a project has recorded no judgments at all (D8.1), so a freshly initialized project has neither a manifest-to-source binding nor the agent-tier gate.

**Inference.** The framework states an automation obligation, declines the mechanics by an explicit boundary, and demonstrates the mechanics only in a form specific to its own release process. An adopter therefore designs the emission-to-check relationship from ordinary CI instinct.

**Consequence.** Ordinary instinct produces caching the manifest, committing it, or emitting it in a job separable from the check. Each yields a pipeline in which `azimuth check` passes against a manifest that no longer describes the tree and reports no holes, so the misconfiguration produces no signal at the moment it matters. This is the one stated obligation whose omission is silent rather than an error.

**Falsifier.** An adopter with no access to this repository's internals wires emission and checking for a supported ecosystem such that a stale, absent or non-corresponding manifest fails the pipeline, using only released material.

**Disposition.** Retain for later analysis; no remedy is selected. The finding does not decide whether consumer CI guidance is published, whether `azimuth check` validates manifest correspondence to the tree, or whether revision binding becomes available outside federation. It records that federation already binds manifests to an observed revision, digests and producer identity through `project observe` and rejects revision skew, and that this mechanism is unavailable to a single-repository project without a project catalog; it does not conclude that the mechanism should be relocated.

### DF-012 — Proof-strength evidence forces design-facet adoption asymmetrically

Version: `0.1.0-alpha.1` Context: modelling a critical at-most-once claim for the planned referrals module in `drim-dev/drim-dev` Status: open

**Observation.** `check.rs:823` breaks out of the `undeclared-mechanism` loop when `model.designs` is empty, so a project running `rtm` without the design artifact is not told that every critical requirement lacks a declared mechanism. The comment cites D8.1: each mechanism must be usable alone. The `unbacked-proof` loop at `check.rs:855` carries no equivalent gate. It iterates plan entries whose evidence strength is `Proof`, resolves the design for that spec, and reports an error when no mechanism at rung 1 or 2 backs the claim, whether or not the project has adopted the design facet at all.

**Inference.** Partial adoption holds until a project declares proof-strength evidence, at which point the design artifact becomes mandatory for that claim. The standard's `Quantification: universal` on critical claims makes a storage constraint the cheapest honest evidence for a uniqueness or exclusion claim, so the transition is reached by taking the framework's own strongest recommendation rather than by an unusual choice.

**Consequence.** A consumer using one artifact adds a second not because it decided the mechanism facet was worth adopting, but because it chose the evidence form the standard steers it toward. The requirement is defensible — proof without a declared mechanism is the `unbacked-proof` defect — but it arrives as an error at the moment of doing the right thing, with no statement that proof and the design facet are coupled.

**Falsifier.** A consumer that has adopted only the spec and plan artifacts can either declare proof-strength evidence without adopting designs, or is told before writing the plan entry that proof requires a mechanism declaration.

**Disposition.** Retain for later analysis; no remedy is selected. The finding does not decide whether `unbacked-proof` should be gated like `undeclared-mechanism`, whether the coupling should be stated in the standards or plan format, or whether the adoption sequence is correct as it stands.

### DF-013 — End-to-end evidence cannot satisfy a critical quantification floor

Version: `0.1.0-alpha.1` Context: mapping the documented drim-dev test pyramid onto the default verification standard Status: open

**Observation.** The default standard written by `azimuth init` requires `Quantification: universal` at critical and `example` at standard. An end-to-end test seeds one world and exercises one path; its honest quantification is `example`. The consumer's test strategy assigns integration confidence to end-to-end tests deliberately, on the reasoning that BFF routes and server components cannot be tested honestly in isolation without mocking the backend. Under that strategy the most expensive tier in the project can only ever cover standard claims, and every critical claim must route to proof-strength evidence, to universal evidence at a lower scope, or to an accepted residual.

**Inference.** The evidence model excludes a whole test tier from the claims a project cares about most. This follows correctly from what universal quantification means and is not a defect in the check, but it is a consequence a project's testing strategy does not anticipate.

**Consequence.** A consumer discovers the exclusion only when `wrong-form` fires, or later when an agent judges an end-to-end tag claiming `universal` to be dishonest. The available responses — storage constraints, property-style evidence at unit scope, or a recorded residual — are all reasonable, but the consumer meets them as a reported hole rather than as guidance.

**Falsifier.** A project adopting Azimuth with an end-to-end-centred integration strategy correctly predicts, before writing its first critical claim, which evidence forms can satisfy that claim and which cannot, using only released material.

**Disposition.** Retain for later analysis; no remedy is selected. The finding does not propose a quantification value for end-to-end evidence, a change to the default standard, or new guidance.

### DF-014 — Judgment freshness couples claims that share one test symbol

Version: `0.1.0-alpha.1` Context: review of the consumer's committed Playwright suite against extractor fingerprint semantics Status: open

**Observation.** The TypeScript extractor resolves each tag call's enclosing named symbol as the site and uses the same AST node as the source-fingerprint boundary. The consumer's end-to-end convention, recorded in its `e2e-testing` skill, models one user journey as a single `test()` containing several `test.step()` calls. Several `Covers` calls placed in one such journey therefore resolve to one site carrying one fingerprint.

**Inference.** Every claim covered by a journey shares that journey's fingerprint, so an edit to any step expires the judgments of all of them together. Granularity of judgment freshness follows test decomposition, which projects choose for readability and failure localization rather than for Azimuth.

**Consequence.** A project following ordinary end-to-end practice obtains coarser freshness than a project that writes one test per claim, without any signal connecting the two. Re-judging is triggered for claims whose evidence did not change.

**Falsifier.** Judgment freshness distinguishes the evidence a claim actually depends on within a shared test symbol, or the consequence of test decomposition on freshness is stated where evidence is tagged.

**Disposition.** Retain for later analysis; no remedy is selected. The finding does not decide whether finer boundaries than the enclosing symbol are derivable, nor recommend a test decomposition.

### DF-015 — Released skills are not discoverable by every supported runtime

Version: `0.1.0-alpha.1` Context: installing the released skills into `drim-dev/drim-dev` for Claude Code and Codex Status: open

**Observation.** The release carries its skills at `.agents/skills/`, which Codex scans natively from the working directory up to the repository root. Claude Code reads `.claude/skills/`, offers no configurable search path, and does not read `.agents/skills/`. This repository resolves the difference with committed symlinks from `.claude/skills/<name>` to `../../.agents/skills/<name>`, which Claude Code documents as supported. No released document describes that step, so a consumer that copies the skills as shipped has them working in one supported runtime and silently absent in the other. Separately, five of the seven skills carry `agents/openai.yaml` interface metadata while `azimuth-cover` and `azimuth-verify` do not.

**Inference.** The distribution layout encodes a runtime assumption that the release does not state. DF-002 records that initialization installs no skills; this concerns whether installing them by hand is sufficient.

**Consequence.** An agent in a consumer repository can be missing the change-lifecycle workflow entirely while the files are present in the checkout, with no error and nothing to notice.

**Falsifier.** A consumer installs the released skills for either supported coding agent and both discover them, using only released material.

**Disposition.** Retain for later analysis; no remedy is selected. The finding does not decide whether the release documents the symlink pattern, ships both layouts, moves the canonical location, or normalizes the per-skill interface metadata.

## Decisions

Findings use stable `DF-NNN` identifiers and retain observation, inference, consequence, falsifier, status and disposition. Corrections will be marked rather than silently rewritten. Recording a finding does not authorize a framework change.

## Open questions

Analysis is deliberately deferred. No remedy, experiment or change decomposition is selected.

One question is recorded without analysis because DF-011 raised it and it is decision-level rather than observational: whether binding a linkage manifest to an observed revision, digests and producer identity belongs to federation, where `project observe` implements it today, or to manifests generally, with federation as one consumer of it. Recording the question selects nothing and does not establish that a second structurally different concern has been demonstrated.

## Result

Fifteen adoption findings are recorded. The exploration remains active while Drim dogfooding continues.
