# Work packages: evidence-control-plane-model

## Work package: semantic-authority
Status: complete
Depends on: none
Owns: docs/decisions.md
Objective: record the authoritative evidence-control-plane decision and explicit alpha 2 revisions
Evidence: prose review, line-length check, and `azimuth change check evidence-control-plane-model`

## Work package: derived-public-account
Status: complete
Depends on: semantic-authority
Owns: docs/framework.md, docs/glossary.md, docs/assurance-extensions.md, azimuth/README.md
Objective: derive current terminology and framework guidance from the frozen semantic decision
Evidence: terminology and contradiction searches plus repository documentation checks

## Work package: current-routine-intent
Status: complete
Depends on: semantic-authority
Owns: azimuth/model/framework/evidence-control-plane
Objective: apply the six routine Claims without adding evidence, design or judgment facets
Evidence: model parse and `azimuth check rtm` with current manifests

## Work package: routine-assurance-deployment
Status: complete
Depends on: semantic-authority
Owns: azimuth/model/framework/assurance-deployment
Objective: lower assurance-deployment requirements and retire obsolete assurance facets
Evidence: criticality and facet inventory plus manifest-backed model validation

## Work package: routine-release-artifacts
Status: complete
Depends on: semantic-authority
Owns: azimuth/model/framework/release-artifacts
Objective: lower release-artifacts requirements and retire obsolete assurance facets
Evidence: criticality and facet inventory plus manifest-backed model validation

## Work package: routine-release-orchestration
Status: complete
Depends on: semantic-authority
Owns: azimuth/model/framework/release-orchestration
Objective: lower release-orchestration requirements and retire obsolete assurance facets
Evidence: criticality and facet inventory plus manifest-backed model validation
