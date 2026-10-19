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
- CI, image publication, fresh production deployment and reseeding remain pending. Release acceptance remains pending.
