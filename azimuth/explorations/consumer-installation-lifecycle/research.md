# Research: Consumer installation lifecycle

Exploration: consumer-installation-lifecycle
Researched: 2026-08-29 to 2026-08-30

## Repository observations

### Current initialization

`tools/azimuth/src/workflow.rs::initialize` currently creates `model/`, `changes/archive/`, `explorations/archive/` and `standards/`, then creates default `standards/verification.md` and `workspace.json` when absent. It does not install agent skills, `azimuth/README.md`, references, an installation manifest or migration resources.

### Current contributor skills

The repository's `.agents/skills/` files combine general Azimuth stage behavior with canonical-repository development assumptions. For example, the current propose skill links to top-level canonical parser contracts. Such a relative link resolves only when the skill is located in the canonical repository layout.

Inference: consumer skills must be rewritten for a target repository rather than copied from or mechanically synchronized with contributor skills.

### Current package boundary

`tools/azimuth/Cargo.toml` currently includes Rust source, Cargo metadata, README and license in the crate package. A resource directory must become an explicit package input before `include_str!` or equivalent embedding can compile from the published crate.

### Current release account

`release/artifacts.json` declares release `0.1.0-alpha.3`, five published packages, two images, three native targets, supported surfaces and experimental-source exclusions. Release qualification checks package and image manifests against the catalog release. It does not currently declare a consumer resource bundle, migration line or the protocol/schema versions produced and accepted by the cohort.

### Current command vocabulary

`azimuth change instructions <change> --package <package>` resolves a work package and emits a contextual implementation handoff. It is not an artifact grammar reference. Renaming it to `azimuth change brief` leaves `reference` free for parser-compatible authoring knowledge.

### Existing initialized repositories

The owner identified only two repositories initialized before the proposed installation manifest. This makes fresh initialization after reviewed cleanup preferable to permanent legacy-adoption machinery.

## External primary-source findings

### GitHub Spec Kit

[Spec Kit core commands](https://github.com/github/spec-kit/blob/main/docs/reference/core.md/) state that initialization installs project structure, templates, scripts and AI-agent integration files. Managed shared files are tracked through a manifest, reinitialization preserves edits by default, and the CLI exposes machine-readable installed capabilities.

[Spec Kit's upgrade guide](https://github.github.com/spec-kit/upgrade.html) separates CLI upgrade from manifest-aware project integration upgrade. It refreshes unchanged managed agent files, scripts and templates while preserving user specifications, plans, tasks, source and project constitution.

Consequence for Azimuth: track managed repository resources, preserve user-owned authority and keep executable upgrade separate from repository synchronization.

### OpenSpec

[OpenSpec setup](https://openspec.dev/docs/setup) states that initialization installs project workflow skills or commands for explicitly selected AI tools and that rerunning initialization refreshes installed files.

[OpenSpec CLI reference](https://openspec.dev/docs/cli) exposes `openspec instructions` for artifact and stage guidance, `openspec schemas --json` for agent-readable capability discovery and `openspec update` for installed instruction files.

[OpenSpec schema documentation](https://openspec.dev/docs/customize-schemas) states that built-in schemas and templates ship inside the package. Project forks are snapshots and are not overwritten by update; users port later built-in improvements manually.

[OpenSpec's migration guide](https://github.com/Fission-AI/OpenSpec/blob/main/docs/migration-guide.md) describes init/update detection of legacy managed files, confirmed cleanup, replacement skill installation and preservation of project-owned content that requires manual judgment.

Consequence for Azimuth: bundle version-matched resources, refresh only managed files and distinguish deterministic layout migration from meaning-bearing account migration.

### Kiro

[Kiro steering documentation](https://kiro.dev/docs/steering/) describes repository-local `.kiro/steering/` and `AGENTS.md` discovery, scoped guidance and project-owned context.

[Kiro spec practices](https://kiro.dev/docs/specs/best-practices/) describe iterative refinement of requirements, design and tasks but do not establish a general tool-version migration surface for existing spec schemas.

Consequence for Azimuth: project-local guidance is normal, but Azimuth's stricter account requires its own explicit format-migration boundary.

### Claude Code

[Claude Code skills documentation](https://code.claude.com/docs/en/slash-commands) states that project skills live under `.claude/skills/<name>/SKILL.md`, follow the Agent Skills standard and load on demand. It does not explicitly guarantee project-skill discovery through symlinks.

[Claude Code memory documentation](https://code.claude.com/docs/en/memory) explicitly supports a symlink for `CLAUDE.md` while noting Windows privilege requirements. That guarantee is not stated for project skills.

Consequence for Azimuth: install exact Claude skill copies by default. Existing repository-internal aliases may be adopted only as a bounded accommodation with an explicit reopening condition.

### Cursor and GitHub Copilot

[Cursor rules documentation](https://docs.cursor.com/context/rules) describes version-controlled project rules under `.cursor/rules/`, scoped loading and remote rules imported into the project.

[GitHub Copilot customization documentation](https://docs.github.com/en/copilot/concepts/prompting/response-customization?tool=visualstudio) describes repository-wide, path-specific and agent instruction files discovered from the repository.

Consequence for Azimuth: normal agent operation should use staged repository-local instructions rather than require network access to framework documentation.

## Research conclusion

The common external pattern is local managed workflow discovery plus packaged tooling, with user-owned specifications protected from resource refresh. Azimuth should adopt that pattern for skills and templates and go further for semantic migration because its accepted account is strict, authoritative and versioned across CLI, emitters and protocols.
