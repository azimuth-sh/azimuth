# Spec: framework/release-artifacts

## Claim: first-alpha-contract
Criticality: routine

The first public alpha artifacts SHALL have one explicit and internally consistent release
contract before publication.

### Case: one-source-version
- Event: a selected first-alpha artifact exposes a version or source tag
- Required outcome: it matches version `0.1.0-alpha.6` and tag `v0.1.0-alpha.6`
- Additional condition or outcome: the release catalog and bundled resource account name migration line `alpha-claim-case`

### Case: registry-identities-match-contract
- Event: a selected artifact is packed for its public registry
- Required outcome: its native metadata names the identity declared by the release contract
- Additional condition or outcome: its source metadata names `https://github.com/azimuth-sh/azimuth`
- Additional condition or outcome: its homepage metadata names `https://azimuth.sh`
- Additional condition or outcome: package identities include `azimuth`, `Azimuth.Annotations` and `Azimuth.Emit`
- Additional condition or outcome: package identities also include `@azimuth-sh/annotations` and `@azimuth-sh/emit`
- Additional condition or outcome: image identities include `ghcr.io/azimuth-sh/azimuth-assurance-api`
- Additional condition or outcome: image identities also include `ghcr.io/azimuth-sh/azimuth-assurance-web`

### Case: packed-contents-are-bounded-and-licensed
- Event: a selected package candidate is built
- Required outcome: its complete contents are allowlisted and its metadata declares Apache-2.0
- Additional condition or outcome: the Rust CLI candidate contains the complete declared consumer resource cohort

### Case: protocol-compatibility-is-explicit
- Event: the release cohort is qualified
- Required outcome: the catalog names every produced or accepted protocol and schema by independent version
- Additional condition or outcome: protocol versions are not inferred from the package release version

### Case: support-and-platforms-are-explicit
- Event: the first-alpha support account is inspected
- Required outcome: native binaries name Linux x64, macOS ARM64 and Windows x64
- Additional condition or outcome: assurance images name Linux AMD64 and ARM64
- Additional condition or outcome: supported framework surfaces are stated without inference

### Case: experimental-source-is-not-published
- Event: experimental polyglot or generic experiment source is classified
- Required outcome: it has no first-alpha public artifact identity or support promise

## Claim: experimental-source-isolation
Criticality: routine

The canonical repository's experimental source SHALL remain continuously verifiable without
reading a domain-owned checkout.

### Case: all-experimental-source-is-gated
- Event: the declared experimental-source account is evaluated
- Required outcome: every declared root is included in an executable repository gate
- Additional condition or outcome: the canonical continuous-integration workflow executes that gate

### Case: experiment-gates-need-no-domain-checkout
- Event: experimental gates run from a clean canonical checkout
- Required outcome: their source and evidence inputs resolve inside that checkout
- Additional condition or outcome: no Drim checkout, demo checkout or domain mount is required

### Case: external-domain-evidence-is-citation-only
- Event: canonical source refers to retained domain-owned evidence
- Required outcome: the reference is an immutable commit-pinned citation
- Additional condition or outcome: it is not opened as a build, test, release or acceptance input
