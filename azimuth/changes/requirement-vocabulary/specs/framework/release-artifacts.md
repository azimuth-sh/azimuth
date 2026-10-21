# Intent delta: framework/release-artifacts

## Replace requirement: first-alpha-contract
From: sha256:eec58bbf3a489b26368ca0b854b347588ee33a70ef5ebcbc0276f9e76ab0cb49
Because: The release contract names the current source version and migration line; the obligation is unchanged and only the pinned values move to 0.1.0-alpha.7 and alpha-requirement-scenario.
Title: First alpha contract
Criticality: routine

The first public alpha artifacts SHALL have one explicit and internally consistent release
contract before publication.

### Scenario: one-source-version
- Event: a selected first-alpha artifact exposes a version or source tag
- Required outcome: it matches version `0.1.0-alpha.7` and tag `v0.1.0-alpha.7`
- Additional condition or outcome: the release catalog and bundled resource account name migration line `alpha-requirement-scenario`

### Scenario: registry-identities-match-contract
- Event: a selected artifact is packed for its public registry
- Required outcome: its native metadata names the identity declared by the release contract
- Additional condition or outcome: its source metadata names `https://github.com/azimuth-sh/azimuth`
- Additional condition or outcome: its homepage metadata names `https://azimuth.sh`
- Additional condition or outcome: package identities include `azimuth`, `Azimuth.Annotations` and `Azimuth.Emit`
- Additional condition or outcome: package identities also include `@azimuth-sh/annotations` and `@azimuth-sh/emit`
- Additional condition or outcome: image identities include `ghcr.io/azimuth-sh/azimuth-assurance-api`
- Additional condition or outcome: image identities also include `ghcr.io/azimuth-sh/azimuth-assurance-web`

### Scenario: packed-contents-are-bounded-and-licensed
- Event: a selected package candidate is built
- Required outcome: its complete contents are allowlisted and its metadata declares Apache-2.0
- Additional condition or outcome: the Rust CLI candidate contains the complete declared consumer resource cohort

### Scenario: protocol-compatibility-is-explicit
- Event: the release cohort is qualified
- Required outcome: the catalog names every produced or accepted protocol and schema by independent version
- Additional condition or outcome: protocol versions are not inferred from the package release version

### Scenario: support-and-platforms-are-explicit
- Event: the first-alpha support account is inspected
- Required outcome: native binaries name Linux x64, macOS ARM64 and Windows x64
- Additional condition or outcome: assurance images name Linux AMD64 and ARM64
- Additional condition or outcome: supported framework surfaces are stated without inference

### Scenario: experimental-source-is-not-published
- Event: experimental polyglot or generic experiment source is classified
- Required outcome: it has no first-alpha public artifact identity or support promise
