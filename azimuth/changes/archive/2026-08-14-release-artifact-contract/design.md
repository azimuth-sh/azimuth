# Design: release-artifact-contract

## Catalog authority

One machine-readable file under `release/` owns the synchronized source version, tag, artifact
identities, license, support classification and platform matrix. Ecosystem manifests retain the
fields their native tools require, but a deterministic checker compares those copies with the
catalog. Release workflows consume the catalog rather than redeclaring the matrix.

The catalog owns metadata, not publication state. Registry existence, credentials, checksums,
attestations and partial-publication recovery belong to later release orchestration.

## Selected artifacts

| Kind | Public identity | Qualified output |
|---|---|---|
| Rust crate | `azimuth` | library and CLI source package |
| NuGet package | `Azimuth.Annotations` | linkage attributes |
| NuGet package | `Azimuth.Emit` | .NET extractor tool |
| npm package | `@azimuth/annotations` | linkage tags and types |
| npm package | `@azimuth/emit` | TypeScript extractor and import CLIs |
| GHCR image | `ghcr.io/drim-dev/azimuth-assurance-api` | Linux AMD64 and ARM64 |
| GHCR image | `ghcr.io/drim-dev/azimuth-assurance-web` | Linux AMD64 and ARM64 |
| GitHub Release | `azimuth` CLI assets | Linux x64, macOS ARM64 and Windows x64 |

Repository formats, standards, skills and documentation are versioned by the Git tag rather than
published as another package. Polyglot implementations and generic experiments remain CI-covered
source and carry no public artifact identity.

## Packed-content boundary

Each native pack tool produces the candidate archive. The checker reads the candidate archive and
its native metadata rather than inferring publication contents from source directories. Per-
artifact allowlists state the required public entry points and permitted documentation or license
files. Common deny rules reject secrets, local configuration, build caches, fixtures and test-only
content.

Pack checks qualify contents and metadata. They do not establish that a registry accepted an
artifact or that a downloaded artifact matches it; those are release-orchestration obligations.

## Platform vocabulary

Native binary qualification uses Rust target triples:

- `x86_64-unknown-linux-gnu`;
- `aarch64-apple-darwin`; and
- `x86_64-pc-windows-msvc`.

Container qualification uses OCI platform names `linux/amd64` and `linux/arm64`. Keeping those
vocabularies explicit prevents one architecture label from being interpreted differently by Rust,
Docker and release workflows.
