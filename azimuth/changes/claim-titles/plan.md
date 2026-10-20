# Plan

1. Add parser, model, title preservation and catalog contracts.
2. Add authored titles to current framework and consumer Claims.
3. Add additive ASS schema revision and title UI.
4. Verify parser semantics, retained catalog round trips and database upgrade.
5. Commit and merge after ordinary CI; publish ASS images, apply reviewed Terraform, create a Git-driven consumer release and verify title data in production.

## Progress

Parser/catalog/model title implementation, current framework titles and ASS UI are complete. Native validation reports 46 Claims and zero Findings. Focused parser and PostgreSQL upgrade tests pass; UI typecheck/build pass. Normal CI39375001587 passed every lane. Canonical PR24 merged as f950254; ASS image publication39375672304 succeeded. The reviewed Terraform rollout changed exactly API/web images and their derived hashes (0 add, 2 update, 0 destroy). API/web are ready with zero restarts, schema v2 is active, and the existing immutable release response is unchanged. A private pre-upgrade backup is retained. Terraform convergence reports no changes. Consumer PR34 passed CI39375128838 attempt2 and merged54d8cbae. GitOps release PR10 completed trusted application release workflow39376349393, whose prepare/publish/bind jobs passed. Its generated-head configuration, publication and request gates passed; the PR merged as59e8db3f4d427adade391b592e4ad93377607e2c. ArgoCD synchronized that exact revision and reports Succeeded/Synced/Healthy; API/web are2/2 on their bound immutable images. Release acceptance is not performed.

## Production verification

- Release: `sha256:a0844577d78a05bd97cd84b2e21e90b0424cfe51034ebd5dd37fe57d79bb66ff`, labelled **drim.dev — concise Claim titles**.
- Model revision: `sha256:396375baa489a1b75204dfb246c02b61b2ea38f5c4e058907e21f6a0df8b8237`. Every normative Claim and Check fingerprint remains unchanged.
- Reconciler reports a fresh complete observation of deployment `observed-b8ec6f1b57812ab5a4d108d6f55bad558b2ad122846435144ab33a21b702f2bd`, Subject `sha256:3a2b70b11e72ecaac6dc295932c7ec9d88d2b323454ea87999b9709e919a6562`.
- Production release and catalog exactly match the CI publication; all14 Claims retain authored titles and full statements. The exact deployment status endpoint returns14 unresolved Claims. Fresh CI engineering gates are not presented as native Runs or Claim support; the snapshot retains8 historical native Runs unchanged. Evidence applicability remains separate follow-up work.
- Public seminar and ASS login return HTTP200. Authenticated visual inspection was not performed; production API data, web build and type checks verify the title wiring.
- Acceptance remains null and no Change was archived. Historical untitled catalogs remain immutable and display stable IDs; use the current environment context to inspect the titled release.
