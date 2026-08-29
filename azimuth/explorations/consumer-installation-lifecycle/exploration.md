# Exploration: Consumer installation lifecycle

Id: consumer-installation-lifecycle
Status: approved
Created: 2026-08-30

## Objective

Make every initialized Azimuth repository self-contained for agent-guided work and keep its CLI, managed workflows, adopted integration components, accepted artifact formats and release compatibility account synchronized without requiring access to the canonical Azimuth checkout or online documentation.

## Boundaries

This exploration covers a canonical consumer-resource bundle, rewritten consumer skills, parser-backed authoring references, explicit Codex and Claude Code installation, a tracked repository installation account, explicit component registration, offline managed-resource updates, semantic account migrations and release-cohort qualification.

It includes updates to affected current model artifacts and derived framework, change-process, CLI, release and adopter-facing documentation. Documentation work belongs to each behavioral transition rather than a later cleanup-only change.

It excludes copying canonical `contracts/`, extended framework documentation or the reference library into consumer repositories; consulting the canonical source or website during normal consumer-agent work; network update checks; CLI self-upgrade; automatic ecosystem-manifest editing; silent agent or component discovery; judgment-bearing automatic intent rewrites; obsolete syntax in normal validation; agent integrations beyond Codex and Claude Code in the first release; indefinite cross-line migration compatibility; and general legacy-adoption machinery for the two existing initialized repositories.

## Existing context

The current `azimuth init` creates the account directories, `standards/verification.md` and `workspace.json`, but does not install skills, `azimuth/README.md`, an installation manifest, authoring references or migration resources.

The repository's `.agents/skills/` files govern development of Azimuth itself. They contain contributor-specific authority and source-layout context and are not consumer release sources. Some current skill links assume the canonical checkout's top-level `contracts/`, which is absent from an ordinary initialized repository.

The current Cargo package does not include a future resource directory. The release catalog synchronizes publishable packages and images but does not account for bundled skills, references, templates, migration resources or a complete protocol compatibility matrix.

The current `azimuth change instructions` operation produces a contextual work-package handoff rather than a versioned authoring contract. No current `azimuth update`, `azimuth reference`, managed installation account, agent integration manager, component registry or account migration command exists.

See `research.md` for repository observations and external primary-source findings.

## Findings

Other SDD frameworks and coding agents converge on repository-local workflow discovery backed by packaged tooling. Spec Kit and OpenSpec install and refresh managed agent files while protecting user-owned specifications. OpenSpec also detects and guides a legacy workflow transition but leaves meaning-bearing project context for manual review. Kiro, Claude Code, Cursor and GitHub Copilot discover repository-local steering, rules or skills.

Those tools primarily migrate managed workflow infrastructure rather than semantically rewriting authoritative specifications. Azimuth needs a stronger boundary because its accepted account is strict, parser-backed, long-lived, identity-bearing and shared with emitters and protocols.

Consumer stage choreography belongs in installed skills. Templates provide initial file shape. Parser-sensitive grammar and supported operations belong behind an offline, version-matched `azimuth reference` surface. The project README is human orientation only, and project-specific policy remains in the target repository's instruction hierarchy.

Canonical consumer resources must therefore have their own release-owned source tree under `tools/azimuth/resources/`. The bundled skills must be rewritten from the consumer perspective rather than copied from or synchronized with `.agents/skills/`. They may rely only on the target repository, its Azimuth account, its applicable repository instructions and the installed CLI.

## Decisions

### D1: consumer-resource-authority

`tools/azimuth/resources/` is the canonical source for distributable consumer skills, references, templates and migrations. `.agents/skills/` remains independent contributor tooling for this repository.

### D2: guidance-layer-ownership

Skills own stage sequence, gates, commands, stopping conditions and prohibited actions. Templates own initial artifact shape. `azimuth reference` owns version-matched authoring contracts. `azimuth/README.md` owns concise human orientation. Consumer repository instructions own project-specific policy. Extended docs own explanation and rationale.

### D3: rewritten-consumer-skill-set

The resource bundle contains consumer rewrites of `azimuth-explore`, `azimuth-propose`, `azimuth-apply`, `azimuth-archive`, `azimuth-coordinate` and a new `azimuth-maintain` skill. No bundled skill directs an agent to canonical Azimuth source, contributor documentation, top-level canonical contracts or archived examples for current syntax.

### D4: explicit-agent-selection

Initialization requires an explicit agent selection. The first release supports Codex and Claude Code, while `azimuth init --agents none` deliberately creates an account without agent integration files. Later integrations are explicit additions, never filesystem inference.

### D5: portable-copies-and-safe-aliases

Azimuth installs exact managed copies for each selected agent by default and never creates symlinks. It may explicitly adopt a pre-existing relative alias only when normalization proves that it targets the expected managed skill location inside the same repository. The installation account records and revalidates the alias.

### D6: explicit-agent-lifecycle

`azimuth agent add <integration>` atomically installs a complete supported integration. `azimuth agent remove <integration>` atomically removes unchanged managed files and updates the installation account. A team-owned adopted alias remains on removal but no longer counts as an active Azimuth integration.

### D7: tracked-installation-account

`azimuth/installation.json` is committed and written by the CLI. It records the Azimuth release, selected agents, adopted components and manifest locations, managed resource identities, paths and hashes, and explicitly adopted aliases. Agents do not maintain it by hand.

### D8: managed-resource-integrity

Installed Azimuth workflows remain CLI-managed. Repository customization belongs in `AGENTS.md` or separate project-owned skills. Any modified managed file blocks an entire update before writes. A retired file may be removed only when the manifest proves ownership, its current content matches the recorded hash and the new cohort explicitly retires or replaces it.

### D9: explicit-component-registration

The repository owner or an authorized agent explicitly selects adopted annotation libraries and emitters and supplies their manifest paths. Ecosystem tooling installs or pins the dependency first. `azimuth component add` validates the existing exact-release component and records it; core does not edit package manifests or lockfiles.

### D10: offline-resource-update

`azimuth update` synchronizes managed repository resources to the currently running CLI release. It performs no network discovery, self-upgrade, package-manager invocation, component-pin rewrite or semantic account migration. Declared component drift blocks the complete update with precise diagnostics.

### D11: embedded-parser-backed-reference

Authoring references remain embedded in the CLI and are exposed through `azimuth reference list|show`, including machine-readable output. Parser-owned structured descriptors provide exact fields, operations, identities and versions; bundled prose provides rationale and examples. References are not installed as repository files.

### D12: command-vocabulary

`reference` means release-versioned authoring contract, `create` means artifact scaffolding, `brief` means contextual delegation and `validate` means conformance. `azimuth change instructions` is retired without an alpha compatibility alias and replaced by `azimuth change brief <change> --package <package>`.

### D13: separate-semantic-migration

Managed-resource synchronization and user-owned account migration are separate operations. Historical formats are read only by dedicated versioned migration readers reachable through `azimuth migrate`; normal validation remains strict to the current format.

### D14: reviewed-atomic-migration

`azimuth migrate plan` writes a content-addressed plan with exact input hashes, deterministic edits, review-required items and unsupported boundaries. `azimuth migrate apply --plan <file>` refuses input drift and applies the exact reviewed deterministic transition atomically. Any review-required work needed for a valid target account blocks all writes until it is resolved manually and replanned.

### D15: explicit-migration-line

The release catalog declares a `migrationLine` identity. Every release retains all supported migration edges in its current line so skipped upgrades can compose them. Changing the identity is an explicit compatibility boundary. Every incompatible release provides either a supported migration path or an explicit no-migration boundary.

### D16: synchronized-release-cohort

CLI, core library, supported annotations, emitters, native distributions and bundled resources use the catalog release version. Protocols and schemas keep independent versions, and the catalog declares which versions the cohort produces and accepts. Release instructions and machine qualification cover the complete cohort.

### D17: fresh-legacy-transition

Azimuth does not implement general adoption for installations without `installation.json`. The two known existing repositories review and remove old generated resources, preserve genuine project-owned policy and run fresh initialization.

### D18: documentation-in-each-transition

Every candidate change updates its affected accepted model and derived framework, change-process, CLI, release and adopter-facing documentation. The intent to rewrite bundled skills from the consumer perspective is explicit documentation scope, not an implementation detail.

## Rejected alternatives

- Using `.agents/skills/` as the release source was rejected because contributor instructions and consumer workflows have different authority and context.
- Copying canonical contracts, extended docs or references into consumer repositories was rejected because it duplicates release-owned material and creates drift.
- Putting stage workflows in templates or references was rejected because skills own lifecycle choreography.
- Automatic agent and component discovery was rejected because filesystem presence does not establish repository intent.
- Editable managed skills and three-way merges were rejected because they create ambiguous workflow authority and mixed-version installations.
- Default symlink creation was rejected because discovery, checkout, archive, watcher and Windows behavior are not portable enough.
- Rejecting every existing symlink was rejected because an exact repository-internal alias can preserve a deliberate team convention within a strict boundary.
- Network update discovery and self-upgrade were rejected because repository synchronization should remain deterministic and package-manager-independent.
- Ecosystem-manifest editing by core was rejected because native tooling owns dependency installation and lockfiles.
- Fully hand-authored references were rejected because parser-sensitive facts can drift; fully generated references were rejected because rationale and examples require deliberate prose.
- `instructions` was rejected for both authoring reference and work-package handoff because it obscures two different concepts.
- Historical formats in normal validation were rejected because migration compatibility must not make obsolete syntax current.
- Text-only migration rewrites were rejected because they cannot prove structure or meaning preservation.
- Partial migration application and generated placeholders were rejected because they knowingly create an intermediate invalid or invented account.
- Indefinite migration retention was rejected because it creates an unbounded cross-line compatibility obligation.
- General legacy-installation adoption was rejected because only two repositories need a one-time transition.
- A broad first agent matrix was rejected because integration variability would dominate the foundational design.

## Residual risks

- Safe alias adoption depends on undocumented third-party skill discovery behavior. Agent discovery or reload failure reopens D5 and requires managed copies.
- Exact installed copies duplicate bytes. Any undetected divergence reopens the manifest and update integrity design.
- Hybrid reference prose can mislead without contradicting a parser descriptor. Release review must inspect semantic examples as well as structural coverage.
- Offline update cannot announce a newer public CLI. Installation and release guidance must state the CLI-first upgrade order clearly.
- Explicit component registration can omit an undeclared dependency. Azimuth diagnoses declared cohort drift but does not claim complete adoption from scanning.
- All-or-nothing migration can make a large semantic transition expensive. It may be reconsidered only if a future migration defines independently valid checkpoints.
- Migration history grows within a line. A line reset must be declared before publication rather than inferred after maintenance becomes inconvenient.
- Fresh initialization can discard locally modified generated guidance if the two legacy repositories skip review. Genuine project policy must be preserved before cleanup.
- Exact component release alignment may eventually be too restrictive. It can be reopened only through an explicit machine-verifiable compatibility model.
- Derived documentation can lag earlier authority. Code, contracts, release accounts and accepted model remain controlling when prose drifts.
- `azimuth-maintain` covers several related operations. It may be split later if size impairs reliable discovery, without changing command ownership.

## Open questions

No material product or workflow decision remains open.

Bounded proposal design still owns the exact installation and migration JSON schemas, parser-descriptor representation, resource embedding and derived hashing mechanism, atomic replacement implementation, diagnostics and exit classifications, initial `migrationLine` spelling, first component identity registry and manifest adapters, and cleanup sequence for the two existing repositories. Those choices may not change the ownership, safety, compatibility or command boundaries above.

No standalone experiment is authorized. Safe alias adoption is the least-certain decision and must be exercised during implementation and release qualification against the supported Codex and Claude Code layouts.

## Result

The direction yields four candidate changes documented in `change-map.md`:

1. `self-contained-consumer-installation`
2. `installation-cohort-maintenance`
3. `account-format-migrations`
4. `synchronized-release-cohort`

The first establishes a usable consumer installation. Cohort maintenance and semantic migrations then proceed independently from that foundation. Release synchronization depends on both. The exploration does not approve or create any candidate change.

Safe alias adoption is the least-certain decision. Claude Code documents project skills under `.claude/skills/` but does not explicitly guarantee skill discovery through symlinks. If a supported agent fails to discover or reload an adopted alias, the repository must replace it with exact managed copies; Azimuth will not add agent-specific integrity exceptions.
