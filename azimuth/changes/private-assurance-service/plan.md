# Plan: private-assurance-service

- [x] Implement inspection-only human access, current API reads, UI and image packaging.
- [x] Implement Terraform workloads, database and private networking configuration in the infrastructure repository.
- [x] Inspect integration, exercise permitted builds and product commands, and document configuration and rollout prerequisites.

- [x] Create Terraform-managed Secrets from explicit OAuth and allowlist inputs, generate stable distinct project/role credentials and document producer delivery.

- [ ] Publish the API and Web images only, retaining exact source revision and immutable registry digests; no tests or full release publication.
