# Verification: experimental-source-isolation

## Experimental root account

Require component scope, universal quantification and a direct oracle for
`all-experimental-source-is-gated`. Evidence ranges over every root derived from the release
catalog and resolves each relation to an executable repository gate.

Mutate the account by adding one root and by removing one gate relation. Both cases must fail with
the missing root named. Reordering set-like entries must retain the same account.

## Checkout isolation

Require component scope, universal quantification and a direct oracle for
`experiment-gates-need-no-domain-checkout`. Execute the root check from a clean temporary clone
whose parent contains no Drim or demo checkout. Add synthetic local and mounted-domain locators and
show that the isolation gate rejects them before executing a lane.

## Citation boundary

Require unit scope, universal quantification and a direct oracle for
`external-domain-evidence-is-citation-only`. Range over every tracked canonical reference to the
retained domain repository. Accept commit-pinned HTTP citations; reject local paths and branch,
tag or unpinned repository URLs.

## Hosted workflow receipt

The first successful GitHub Actions run is rollout-dependent component evidence. Record the
repository, workflow identity, exact commit SHA and conclusion. A successful run at another
revision is stale and cannot satisfy acceptance.
