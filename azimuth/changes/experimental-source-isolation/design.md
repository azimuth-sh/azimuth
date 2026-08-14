# Design: experimental-source-isolation

## One source account

`release/artifacts.json` remains authoritative for experimental roots. The isolation gate derives
its population from `experimentalSource` and binds each root to an executable check; it does not
copy the eleven paths into CI configuration or another inventory. The gate fails if a root does not
exist, has no tracked source or has no executable account.

The relation may name a root check that derives finer membership from tracked manifests and source
directories. It may not count a comment, documentation mention or release classification as
execution coverage.

## Local and hosted execution

`scripts/check.sh` remains the executable repository account. GitHub Actions checks out the
canonical repository and invokes that script, so local and hosted gates cannot drift through two
command lists. Workflow setup may install language toolchains and contact ordinary dependency
registries; those are execution prerequisites, not source or evidence authorities.

The first hosted run is rollout-dependent evidence. Implementation may be committed while that
condition is pending, but acceptance and archive wait for an exact-revision successful run.

## Domain boundary

Isolation validation distinguishes a source dependency from a citation. An executable input must
resolve inside the canonical checkout or a task-specific build/cache directory. A citation to
domain-owned provenance is permitted only when its URL names an immutable commit. The gate never
fetches citation targets.

This boundary does not ban domain vocabulary from historical decisions or prior-art discussion.
It rejects locators that make a canonical build, test, release or acceptance depend on another
domain checkout, plus mutable links presented as provenance.

## Failure boundary

A missing toolchain, failed experiment, incomplete root account, external locator or inconclusive
hosted run fails the isolation claim. The change does not weaken a lane merely because a hosted
runner lacks its prerequisites; workflow setup must make the selected corpus executable.
