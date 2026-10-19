# Plan

1. Inventory strict contracts and establish explicit schemas for both APIs.
2. Implement typed persistence, fresh initialization and transactional write/read integration.
3. Preserve native round trips, correction histories, isolation and authorization in real-DB tests.
4. Fix heavy read paths, bounded blocking assessment work and compact UI summary requests.
5. Review, run normal CI, publish images and back up production.
6. Create separate fresh production database; review data-preserving Terraform plan and deploy.
7. Restore approved baseline and current publication, observe actual deployment, verify UI and data shape. Leave acceptance pending.

## Progress

- Steps 1–4 completed. Explicit schemas cover both API generations; no stored JSON columns remain.
- Real PostgreSQL tests pass for fingerprint-preserving historical/current Reviews, publications, Subjects, nulls/order, immutable conflicts and rollback. The original schema is rejected without mutation.
- Operational rehearsal restored the approved baseline and exact drim.dev publication into an isolated fresh database; full publication equality passed. Release list 9 ms; full publication and revision snapshot approximately 620 ms locally.
- Independent review found no remaining blocking correctness findings. Production backup retained privately before rollout.
- Steps 5–7 completed on 2026-10-19. Required CI run 39299836386 passed; image publication 39300924480 succeeded from c9144017ad8afd4062fcf60afe52860f2e470322.
- Production now uses the separate assurance_state_relational database. The original assurance_state database and pre-rollout backup remain retained. Terraform created one database and performed bounded updates with zero deletions; admitted startup defaults were declared explicitly after provider consistency errors.
- Approved baseline and the exact current release publication were restored, with full publication equality verified. Reconciler reports a fresh complete observation for the current release. Production has zero JSON/document columns and zero accepted releases; API/Web pods are ready with zero restarts.
- Web health and login return HTTP 200; full web build and typecheck pass. Browser automation was unavailable, so authenticated screen rendering was not visually inspected during this rollout.
- Release acceptance and archival remain pending by design.
