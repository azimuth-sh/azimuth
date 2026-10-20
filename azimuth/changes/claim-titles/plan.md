# Plan

1. Add parser, model, title preservation and catalog contracts.
2. Add authored titles to current framework and consumer Claims.
3. Add additive ASS schema revision and title UI.
4. Verify parser semantics, retained catalog round trips and database upgrade.
5. Commit and merge after ordinary CI; publish ASS images, apply reviewed Terraform, create a Git-driven consumer release and verify title data in production.

## Progress

Parser/catalog/model title implementation, current framework titles and ASS UI are complete. Native validation reports 46 Claims and zero Findings. Focused parser and PostgreSQL upgrade tests pass; UI typecheck/build pass. Normal CI and production rollout remain pending.
