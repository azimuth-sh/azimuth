# Verification: private-assurance-service

## Engineering verification

Inspect human/session authorization, project confinement and the absence of browser or viewer write privileges. Check current API and UI integration against exact-Subject Assurance State and preserved immutable histories. Compile the service and typecheck/build the web image sources. Inspect Terraform formatting and validate initialized modules when provider access permits. Dima explicitly selected builds and inspection only for this work. Do not write or run service tests; record unverified runtime behavior honestly.

## Deployment verification

Before declaring this deployment established, verify the real HTTP-01 issuance and renewal path, private DNS resolution, certificate hostname, Tailscale identity/grants and GitHub Actions federation on the configured infrastructure. An unauthenticated public client must not reach the application or its API; only the ACME challenge is publicly routed. Exercise GitHub sign-in for allowed and denied users, project confinement, inspector write rejection, machine submissions and data survival. Source configuration and builds do not establish these live facts.

## Baseline

Canonical validate reports structurally valid accepted intent with nine unresolved design-binding evidence gaps when the existing release/deployment manifests are not supplied. These are pre-existing missing-input gaps; do not claim them resolved by service implementation. The infrastructure worktree contained an unrelated untracked .serena directory before implementation; preserve it.
