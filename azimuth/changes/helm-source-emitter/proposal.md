# Change: helm-source-emitter

Status: proposed

Intent delta: none
Because: The existing manifest contract already accepts extractor-emitted Realizes records and derives source identity from language and site. This change supplies a Helm producer and a Helm-specific address kind without adding an accepted behavioral Claim.

## Problem

A consumer Helm chart can carry Realizes markers, but Azimuth has no reusable Helm emitter. A chart-local script currently mixes source-link extraction with application-specific Kubernetes access checks. Reusing it would copy domain policy into the framework.

## Outcome

An experimental, private TypeScript package renders a local Helm chart, identifies marked Kubernetes resources by chart, release, API version, kind, namespace and name, and emits a linkage manifest. The manifest contract derives `helm-resource` address kind for its sites. The consumer keeps its private API access checks in its own repository.

## Scope

In scope:

- a standalone Helm emitter under `tools/extractors/helm`;
- strict marker, source locator and resource identity handling;
- source fingerprints bound to source, selected values, rendered resource, Helm version and render parameters;
- a `helm-resource` address kind for Realizes and Check implementation records; and
- an experimental-source classification without publication.

Out of scope:

- Kubernetes policy analysis, deployment evidence or cluster access;
- public package publication or adoption registration;
- a Helm mechanism-implementation marker; and
- consumer-domain chart fixtures or tests in the canonical repository.

## Affected claims

No accepted Claim changes. This is an explicit framework-only change.

## Completion conditions

- A marked, rendered Helm resource yields one Realizes record with a repository-relative template locator and a stable semantic site.
- Malformed or ambiguous marker, missing source or resource identity fails without writing an output manifest.
- The package builds and the Azimuth CLI accepts the new address kind.
- The canonical package has no dependency on a consumer checkout and is not published.
- The consumer's chart-specific access checks remain in the consumer repository.
