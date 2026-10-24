---
name: azimuth-maintain
description: Initialize and maintain a repository's Azimuth installation, agent integrations, adopted annotation or emitter components, managed resources and account migrations. Use for setup, adding or removing an agent, aligning a new Azimuth release, or resolving update and migration diagnostics.
---

# Maintain an Azimuth installation

Read the target repository's applicable instructions and inspect package-manager ownership before changing anything. Never consult a canonical Azimuth checkout for consumer guidance.

## Initialize

Select integrations explicitly and run `azimuth init --agents codex,claude` or `azimuth init --agents none`. Review created paths and commit `azimuth/installation.json` with the managed resources. Use `--adopt-alias` only for a pre-existing relative alias that resolves to the exact repository-internal managed skill location.

## Manage agents

Use `azimuth agent add <id>` and `azimuth agent remove <id>`. Let the CLI own installed Azimuth skill files. Put project-specific policy in repository instructions or separate project-owned skills. Resolve any managed-file conflict before retrying; never merge a mixed workflow cohort.

Each integration also installs a Work Provenance capture hook as a managed resource: `.claude/settings.json` for `claude` and `.codex/hooks.json` for `codex`. Keep project Claude Code settings in `.claude/settings.local.json` or user settings, never in the managed file. An existing unmanaged file at either path blocks installation; ask the owner how to move its content before retrying. The hooks run `azimuth` from `PATH`, so every contributor's CLI must be a release that ships `azimuth provenance capture`. Tell the owner that Codex runs the hook only after they trust the project and the hook itself through `/hooks`, and again after the hook changes.

## Manage components

Use the repository's native package manager or tool manifest to install and pin the exact Azimuth release first. Then run `azimuth component add <id> --manifest <path>`. Component removal unregisters the component but does not edit the ecosystem manifest. Do not infer adoption from filesystem scanning.

## Upgrade

1. Update the repository's CLI and adopted component pins through their owning package managers as one reviewed change.
2. Run `azimuth update --check`, then `azimuth update --dry-run`.
3. Resolve modified managed files, alias drift and component mismatches.
4. Run `azimuth update` to synchronize the repository to the running CLI.
5. If the CLI reports an account-format transition, run `azimuth migrate plan --out <file>`, resolve every review-required item, replan and apply only the exact content-addressed plan.
6. Run the repository's normal `azimuth validate` and inspect the complete diff.

`azimuth update` is offline and never upgrades the CLI, invokes a package manager or rewrites user-owned Azimuth artifacts. `azimuth migrate apply` refuses input drift and never writes a partial or placeholder-bearing account.
