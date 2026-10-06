# Proposal: private-assurance-service

Status: active

## Authority and scope

Dima authorized implementation on 2026-10-06. Update the Assurance State API and inspection-only Web UI, use GitHub OAuth with explicit stable-user-ID authorization, retain project-scoped machine writes, and package the current service. Terraform in drim-dev-infra deploys the workloads, database, Tailscale connectivity and HTTP-01 certificates for assurance.drim.dev. Public ingress serves ACME challenges only; the service is reachable through Tailscale. The original implementation authorization excluded deployment, Terraform apply, commit, publication and archival. Dima subsequently authorized committing the source and publishing the two Assurance images only, with builds and inspection and no tests. Full Azimuth package publication, hosted deployment, Terraform apply and archival remain outside this authorization. The local Compose profile is aligned to the current authenticated service; its accepted network-containment Case is updated accordingly while preserving its loopback boundary.

## Delivery boundary

Use explicit configuration for OAuth credentials, human project access, machine credentials, images and Tailscale identity. Keep credentials server-side. The browser cannot submit reviews, select authority or ingest Runs. Expose project-scoped read access to current Runs, independent reviews, selected authority and exact-Subject Assurance State without treating unresolved or stale evidence as supported. Preserve existing unrelated work and historical isolated service code; do not create compatibility aliases. Infrastructure provisioning inputs and live verification remain operator-supplied rollout prerequisites.
