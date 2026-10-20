# Plan

Implementation and rollout are complete. Release acceptance remains pending.

## Completed work

1. Add optional authored `Title:` metadata to Claim and Invariant headers, intent projection, complete model export and release catalogs.
2. Add titles to the 46 current framework Claims and 14 current drim.dev Claims. Keep normative Claim and Case fingerprints unchanged; publish a distinct complete model revision.
3. Show titles in ASS lists and headings, with stable-ID fallback for untitled historical catalogs. Keep full statements in Claim details.
4. Upgrade relational catalog storage from schema 1 to schema 2 transactionally, adding one nullable text column and preserving existing records.
5. Pass normal CI, publish ASS images, apply the reviewed image-only Terraform plan, publish a Git-driven drim.dev release and observe its rollout.

## Engineering verification

- Native framework validation: 46 Claims, zero Findings; change validation passes.
- All 36 parser tests pass, including unchanged Claim/Case fingerprints and distinct complete model revisions after title edits.
- ASS server unit and PostgreSQL integration tests pass, including schema-upgrade preservation/idempotence and registered deployments without Runs.
- ASS web type checking and production build pass.
- Independent read-only review identified multiline catalog titles; the validator now rejects them.
- Canonical CI [39375001587](https://github.com/azimuth-sh/azimuth/actions/runs/39375001587) passed every lane. Implementation PR #24 merged as `f950254ad80caa1d5e02b8453758899d07d198e0`.
- Consumer CI [39375128838](https://github.com/drim-dev/drim-dev/actions/runs/39375128838), attempt 2, passed after updating its exact canonical tooling pin. Consumer PR #34 merged as `54d8cbaeae8f1a1cd592e2d36a877f1a40e33989`.

## ASS rollout

- Image publication [39375672304](https://github.com/azimuth-sh/azimuth/actions/runs/39375672304) succeeded.
- API image digest: `sha256:185adbc16b792e46c9c97356a06766473529ba6cb2811519dcf329174c11b21e`.
- Web image digest: `sha256:4581c103a5e5d25c090019aa414aea6797b1b601ff52e3ffb92fbdcb7cd019ce`.
- Reviewed Terraform changes contained only API/web images and their derived configuration hashes: 0 added, 2 updated, 0 destroyed. Apply succeeded and convergence reports no changes.
- API/web rollouts are ready with zero restarts. Schema 2 is active; the existing immutable release response is exactly unchanged. Existing records and credentials are retained, with a private pre-upgrade backup.

## Consumer release and production verification

- Release workflow [39376349393](https://github.com/drim-dev/drim-dev/actions/runs/39376349393) passed prepare, publish and bind. GitOps PR #10 passed its generated-head configuration, publication and request gates, then merged as `59e8db3f4d427adade391b592e4ad93377607e2c`.
- ArgoCD synchronized that exact revision and reports Succeeded, Synced and Healthy. API/web have 2/2 ready replicas on the bound immutable images.
- Release: `sha256:a0844577d78a05bd97cd84b2e21e90b0424cfe51034ebd5dd37fe57d79bb66ff`, labelled **drim.dev — concise Claim titles**.
- Model revision: `sha256:396375baa489a1b75204dfb246c02b61b2ea38f5c4e058907e21f6a0df8b8237`. All normative Claim and Check fingerprints remain unchanged.
- Reconciler reports a fresh complete observation of deployment `observed-b8ec6f1b57812ab5a4d108d6f55bad558b2ad122846435144ab33a21b702f2bd`, Subject `sha256:3a2b70b11e72ecaac6dc295932c7ec9d88d2b323454ea87999b9709e919a6562`.
- Production release and catalog exactly match the CI publication. All 14 Claims retain authored titles and full statements. The exact deployment status endpoint returns 14 unresolved Claims.
- Fresh CI engineering gates are not native Runs or Claim support. The snapshot retains eight historical native Runs unchanged; evidence applicability remains separate follow-up work.
- Public seminar and ASS login return HTTP 200. Authenticated visual inspection was not performed; production API data, web build and type checks verify the title wiring.
- Acceptance remains null and no Change was archived. Historical untitled catalogs remain immutable and display stable IDs. Use the current environment context to inspect the titled release.
