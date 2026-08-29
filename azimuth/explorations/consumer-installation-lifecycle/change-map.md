# Change map: Consumer installation lifecycle

Exploration: consumer-installation-lifecycle

## Dependency graph

```text
self-contained-consumer-installation
├── installation-cohort-maintenance
└── account-format-migrations
    installation-cohort-maintenance + account-format-migrations
        └── synchronized-release-cohort
```

## A1: self-contained-consumer-installation

Depends on: none

Outcome: A newly initialized repository receives a complete version-matched, offline-capable Azimuth workflow and can discover every supported stage without access to the canonical source checkout.

Carries decisions: D1, D2, D3, D4, D5, D6, D7, D11, D12, D17, D18

Candidate scope:

- Establish `tools/azimuth/resources/` and its release-owned resource account.
- Rewrite all six consumer skills from the target-repository perspective.
- Add parser-backed reference descriptors, bundled prose and `azimuth reference list|show` with machine-readable output.
- Install the consumer README and tracked installation account.
- Support explicit Codex, Claude Code and `none` selection.
- Install exact managed copies by default and explicitly adopt only safe repository-internal aliases.
- Add atomic `azimuth agent add|remove`.
- Rename `azimuth change instructions` to `azimuth change brief` without an alpha compatibility alias.
- Include resources in Cargo packaging and make the minimum release allowlist changes needed for distributable candidates.
- Document the reviewed cleanup and fresh-initialization path for the two known legacy repositories without introducing general adoption machinery.
- Update affected current model and derived installation, framework, CLI and adopter documentation.

Excludes component adoption, cross-release resource update and semantic account migration.

Completion boundary: A clean repository can initialize, inspect installed reference material, run a supported stage through Codex or Claude Code and account for every installed managed path without canonical-source or network access.

## A2: installation-cohort-maintenance

Depends on: self-contained-consumer-installation

Outcome: An initialized repository can explicitly register adopted Azimuth components and synchronize one coherent managed-resource cohort to the running CLI release.

Carries decisions: D8, D9, D10, D18

Candidate scope:

- Declare supported component identities and explicit manifest adapters.
- Add component registration and removal without ecosystem-manifest edits.
- Add offline update checking, preview and atomic application.
- Reject modified managed files, alias drift, component-pin drift and any partial cohort.
- Retire only manifest-owned, unchanged resources explicitly retired or replaced by the new bundle.
- Expand the maintenance skill for component and update workflows.
- Update affected current model and derived framework, change-process, CLI and adopter documentation.

Excludes network release discovery, self-upgrade, package-manager invocation and semantic account migration.

Completion boundary: A repository can prove that its managed skills, installed orientation resources and explicitly adopted components match the currently running CLI release, or receive a precise no-write conflict account.

## A3: account-format-migrations

Depends on: self-contained-consumer-installation

Outcome: An incompatible CLI release can inspect and apply explicitly supported meaning-preserving account migrations without making historical syntax valid in normal operation.

Carries decisions: D13, D14, D15, D18

Candidate scope:

- Add explicit release `migrationLine` identity and bundled cumulative migration edges.
- Add dedicated versioned historical readers reachable only through migration commands.
- Expose migration references through `azimuth reference`.
- Add content-addressed migration planning with deterministic, review-required and unsupported classifications.
- Add exact-plan, input-stable, atomic migration application.
- Refuse all writes when review-required work is necessary for a valid target account.
- Expand the maintenance skill for upgrade and migration ordering.
- Update affected current model and derived framework, change-process, CLI, migration and adopter documentation.

Excludes normal-parser backward compatibility, invented semantic rewrites, placeholders and mixed-format intermediate accounts.

Completion boundary: A supported old account produces a complete reviewable migration disposition, and apply either establishes one valid target account from the exact reviewed plan or writes nothing.

## A4: synchronized-release-cohort

Depends on: installation-cohort-maintenance, account-format-migrations

Outcome: One release account and publishing process qualify the CLI, supported libraries and emitters, bundled consumer resources and accepted protocol/schema compatibility as one cohort.

Carries decisions: D15, D16, D18

Candidate scope:

- Extend the release catalog with bundled resources, protocol/schema compatibility and migration-line identity.
- Require exact release alignment across publishable components and resources while preserving independent protocol versions.
- Qualify resource population, reference coverage and component compatibility.
- Require every incompatible release to declare a migration path or explicit no-migration boundary.
- Update release candidate allowlists and retained release accounts.
- Document the operator sequence for CLI upgrade, ecosystem pin alignment, resource update, account migration and final validation.
- Complete a cross-document terminology and guidance audit.

Excludes adding new agent integrations or creating compatibility promises outside the declared migration line.

Completion boundary: Publication cannot complete unless every selected public component, bundled resource and declared protocol/schema compatibility matches the tagged catalog account.
