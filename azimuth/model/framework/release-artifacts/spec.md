# Spec: framework/release-artifacts

## Claim: first-alpha-contract
Criticality: routine

The first public alpha artifacts SHALL have one explicit and internally consistent release
contract before publication.

### Case: one-source-version
WHEN a selected first-alpha artifact exposes a version or source tag
THEN it matches version `0.1.0-alpha.4` and tag `v0.1.0-alpha.4`
AND the release catalog and bundled resource account name migration line `alpha-claim-case`

### Case: registry-identities-match-contract
WHEN a selected artifact is packed for its public registry
THEN its native metadata names the identity declared by the release contract
AND its source metadata names `https://github.com/azimuth-sh/azimuth`
AND its homepage metadata names `https://azimuth.sh`
AND package identities include `azimuth`, `Azimuth.Annotations` and `Azimuth.Emit`
AND package identities also include `@azimuth-sh/annotations` and `@azimuth-sh/emit`
AND image identities include `ghcr.io/azimuth-sh/azimuth-assurance-api`
AND image identities also include `ghcr.io/azimuth-sh/azimuth-assurance-web`

### Case: packed-contents-are-bounded-and-licensed
WHEN a selected package candidate is built
THEN its complete contents are allowlisted and its metadata declares Apache-2.0
AND the Rust CLI candidate contains the complete declared consumer resource cohort

### Case: protocol-compatibility-is-explicit
WHEN the release cohort is qualified
THEN the catalog names every produced or accepted protocol and schema by independent version
AND protocol versions are not inferred from the package release version

### Case: support-and-platforms-are-explicit
WHEN the first-alpha support account is inspected
THEN native binaries name Linux x64, macOS ARM64 and Windows x64
AND assurance images name Linux AMD64 and ARM64
AND supported framework surfaces are stated without inference

### Case: experimental-source-is-not-published
WHEN experimental polyglot or generic experiment source is classified
THEN it has no first-alpha public artifact identity or support promise

## Claim: experimental-source-isolation
Criticality: routine

The canonical repository's experimental source SHALL remain continuously verifiable without
reading a domain-owned checkout.

### Case: all-experimental-source-is-gated
WHEN the declared experimental-source account is evaluated
THEN every declared root is included in an executable repository gate
AND the canonical continuous-integration workflow executes that gate

### Case: experiment-gates-need-no-domain-checkout
WHEN experimental gates run from a clean canonical checkout
THEN their source and evidence inputs resolve inside that checkout
AND no Drim checkout, demo checkout or domain mount is required

### Case: external-domain-evidence-is-citation-only
WHEN canonical source refers to retained domain-owned evidence
THEN the reference is an immutable commit-pinned citation
AND it is not opened as a build, test, release or acceptance input
