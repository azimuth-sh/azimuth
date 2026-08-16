# Exploration: Drim dogfooding findings

Id: drim-dogfood
Status: exploring
Created: 2026-08-16

## Objective

Record observations made while adopting and using published Azimuth versions in Drim. The record
keeps consumer evidence available for later analysis without treating an observation, inference or
possible remedy as accepted framework truth.

## Boundaries

This exploration covers onboarding, initialization, agent guidance, migration friction, modeling
gaps and tool behavior observed in `drim-dev/drim-dev`. Each finding identifies the Azimuth version
and context that produced it. It also records follow-up observations from using Azimuth to act on
the dogfood findings in the canonical framework repository.

Registry reservation, artifact publication and other release-production findings remain outside
this exploration because the canonical alpha-release exploration already owns that activity. This
record is not a roadmap, issue tracker or source of current framework behavior.

## Existing context

The consumer exercised the public `0.1.0-alpha.1` release. Adoption has not yet carried the planned
referral feature through the complete Azimuth change lifecycle, so the entries below are adoption
findings rather than evidence about that lifecycle as a whole.

## Findings

### DF-001 — Consumer installation is not self-contained

Version: `0.1.0-alpha.1`
Context: initial installation in `drim-dev/drim-dev`
Status: open

**Observation.** The crate README documents installation from a source checkout. The public
repository directs readers to repository-local framework documents, while the consumer start path
does not explain the native release archives or distinguish binary installation from compilation
by `cargo install`. Dogfooding required separate explanations of the command, crate archive contents
and precompiled archives.

**Inference.** A new consumer cannot reliably discover every supported installation path from the
published package documentation alone.

**Consequence.** Installation can succeed while the consumer remains uncertain about which artifact
was installed, whether compilation is expected and when a qualified native archive applies.

**Falsifier.** A first-time consumer installs either supported form and accurately explains its
provenance and platform qualification using only the published consumer entry point.

**Disposition.** Retain for later analysis; no remedy is selected.

### DF-002 — Initialization does not bootstrap an agent workflow

Version: `0.1.0-alpha.1`
Context: `azimuth init` in `drim-dev/drim-dev`
Status: open

**Observation.** Initialization creates the model, change, exploration and standards directories,
`standards/verification.md`, and `workspace.json`. It does not install Azimuth skills, format
contracts, operating documentation or agent instructions. The initialized Drim repository contains
only the two generated files under `azimuth/`.

**Inference.** A coding agent reading only the consumer repository cannot discover how to propose,
apply, verify and archive an Azimuth change.

**Consequence.** Successful initialization does not make the released workflow operable by a cold
coding agent without additional, manually supplied context.

**Falsifier.** A cold agent correctly follows the released change workflow using only the
initialized consumer repository.

**Disposition.** Retain for later analysis; no remedy is selected.

### DF-003 — Generated configuration is not self-explanatory at its point of use

Version: `0.1.0-alpha.1`
Context: review of files produced by `azimuth init`
Status: open

**Observation.** Understanding the generated verification standard and empty workspace required
separate explanations. The generated workspace does not state when areas, surfaces or realization
obligations should be introduced.

**Inference.** Initialization establishes valid syntax but does not supply enough local context for
a first-time adopter to make sound modeling decisions.

**Consequence.** A consumer can retain valid defaults without understanding what they require or
when the empty architectural declarations must change.

**Falsifier.** A first-time adopter accurately explains the two generated files and makes a sound
decision about leaving or changing each one using only repository-local initialized material.

**Disposition.** Retain for later analysis; no remedy is selected.

### DF-004 — Replacement of pre-alpha state is manual

Version: `0.1.0-alpha.1`
Context: replacement of earlier Azimuth and OpenSpec integration in `drim-dev/drim-dev`
Status: accepted limitation

**Observation.** Drim contained pre-alpha Azimuth linkage, build integration and OpenSpec workflow
artifacts. They required a repository audit and selective removal while educational content had to
remain. `azimuth init` is additive and performs no migration or stale-state detection.

**Inference.** Adoption from an earlier experiment requires an explicit audit of the consumer's
previous state.

**Consequence.** Stale integration can survive initialization or broad cleanup can remove content
that merely discusses the superseded tools.

**Falsifier.** A consumer with structurally similar pre-alpha state can initialize the public alpha
without a manual audit and without retaining technical residue or deleting unrelated content.

**Disposition.** Retain as an observed consequence of the alpha's deliberate no-compatibility
boundary; it does not by itself establish a framework defect.

### DF-005 — One ordered strength cannot require complementary evidence

Version: `0.1.0-alpha.1`
Context: modeling settlement reconciliation with both tests and operational detection
Status: open

**Observation.** Evidence strength is one ordered value:
`detection < demonstration < proof`. A demonstration therefore satisfies a detection floor. Azimuth
checks the detector and detector-test bindings when a detection item is declared, but deleting that
entire item creates no hole when demonstration evidence still satisfies the claim's required
strength.

**Inference.** The verification policy cannot express an obligation for both demonstration and
detection evidence on one claim.

**Consequence.** If monitoring is required only as complementary assurance, its declaration can
disappear without making the model incomplete. When notification is durable system behavior, a
separate behavioral claim can preserve that obligation, but that does not express a general
requirement for complementary assurance forms.

**Falsifier.** Show that the released model can require demonstration and detection independently
for one claim, such that removing either evidence form creates a machine finding while the other
remains.

**Disposition.** Retain for later analysis. Evidence-before-notation prevents generalizing one
observation into new syntax before a second structurally different concern demands it.

### DF-006 — The ordinary-work boundary is not discoverable

Version: `0.1.0-alpha.1`
Context: applying the canonical development-authority revision in `azimuth-sh/azimuth`
Status: open

**Observation.** Current guidance says that routine work with clear intent may begin with a
lightweight change and that a framework mechanism or operating document with no intent delta uses
an explicit framework-only change. It does not state when an edit is ordinary repository work that
needs no Azimuth artifact. During dogfooding, an agent created a proposal, plan and outcome and ran
the complete release and Docker qualification suite for an instruction and decision revision. The
authority revision was structurally significant, but the selected evidence included checks with no
discriminating relationship to the edited documents.

**Inference.** The workflow biases agents toward treating every repository edit as an Azimuth
semantic change and toward selecting evidence by repository scope rather than by what could
falsify the edit.

**Consequence.** Small or nonsemantic work can acquire change directories, lifecycle bookkeeping
and expensive checks that cannot detect a plausible error in the edit. That cost weakens the
framework's ceremony falsifier because unnecessary accounting is attributed to ordinary use.

**Falsifier.** A cold agent correctly distinguishes ordinary repository work, a routine semantic
change and a standard or critical semantic change, creates no Azimuth artifact for the ordinary
case, and selects checks that can reject plausible errors in each remaining case using only the
published workflow guidance.

**Disposition.** Retain for later analysis. No classification rule, CLI behavior or change-lifecycle
revision is selected by this finding.

### DF-007 — Published library consumption is not a development lane

Version: `0.1.0-alpha.1`
Context: Rust dependency and release-surface review in `azimuth-sh/azimuth`
Status: open

**Observation.** The published `azimuth` crate contains both the CLI binary and a Rust library, and
the release catalog names that artifact `rust-cli-core`. Normal repository development does not
consume it through crates.io: `azimuth-assurance-domain` uses a path dependency on
`tools/azimuth`. The Rust annotations and emitter remain experimental path-based source and are not
published. Release qualification validates retained package contents, but there is no mandatory
registry-only development lane that consumes exact published Rust, NuGet and npm library versions
without local path, project or workspace overrides.

**Inference.** Azimuth qualifies what it publishes but does not continuously exercise the released
libraries as a normal consumer while developing the next version. Self-dogfooding therefore covers
candidate integration without covering registry consumption between releases.

**Consequence.** Published API, dependency, packaging or installation defects can remain invisible
to ordinary development builds. The repository can also evolve around local source relationships
that consumers cannot reproduce from the selected public artifacts.

**Falsifier.** A clean mandatory development lane resolves exact supported library versions from
their public registries with no local override, exercises their consumer contracts, and fails when
a published artifact is absent or incompatible. A separate candidate lane continues to exercise
the source versions being changed.

**Disposition.** Retain for later analysis. The finding does not decide whether to split the core
crate, publish the Rust annotations or emitter, expose assurance-domain APIs, or define the two
development lanes.

## Decisions

Findings use stable `DF-NNN` identifiers and retain observation, inference, consequence, falsifier,
status and disposition. Corrections will be marked rather than silently rewritten. Recording a
finding does not authorize a framework change.

## Open questions

Analysis is deliberately deferred. No remedy, experiment or change decomposition is selected.

## Result

Seven adoption findings are recorded. The exploration remains active while Drim dogfooding
continues.
