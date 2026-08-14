# Spec: framework/release-artifacts

## Requirement: first-alpha-contract
Criticality: standard

The first public alpha artifacts SHALL have one explicit and internally consistent release
contract before publication.

### Scenario: one-source-version
WHEN a selected first-alpha artifact exposes a version or source tag
THEN it matches version `0.1.0-alpha.1` and tag `v0.1.0-alpha.1`

### Scenario: registry-identities-match-contract
WHEN a selected artifact is packed for its public registry
THEN its native metadata names the identity declared by the release contract
AND package identities include `azimuth`, `Azimuth.Annotations` and `Azimuth.Emit`
AND package identities also include `@azimuth/annotations` and `@azimuth/emit`
AND image identities include `ghcr.io/drim-dev/azimuth-assurance-api`
AND image identities also include `ghcr.io/drim-dev/azimuth-assurance-web`

### Scenario: packed-contents-are-bounded-and-licensed
WHEN a selected package candidate is built
THEN its complete contents are allowlisted and its metadata declares Apache-2.0

### Scenario: support-and-platforms-are-explicit
WHEN the first-alpha support account is inspected
THEN native binaries name Linux x64, macOS ARM64 and Windows x64
AND assurance images name Linux AMD64 and ARM64
AND supported framework surfaces are stated without inference

### Scenario: experimental-source-is-not-published
WHEN experimental polyglot or generic experiment source is classified
THEN it has no first-alpha public artifact identity or support promise
