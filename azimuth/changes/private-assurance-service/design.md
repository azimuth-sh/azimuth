# Design: private-assurance-service

## Human and machine access

GitHub authenticates humans through the web server. Stable GitHub user IDs are allowlisted for specific projects. The UI is inspection-only and never exposes service credentials to the browser. A distinct viewer credential lets the web server read the current API without authority to append Runs, reviews or authority selections. Existing producer, reviewer and owner credentials retain their separate machine roles. Sessions and all project reads fail closed on absent or invalid authorization.

## Private origin and certificate issuance

The public DNS answer for assurance.drim.dev points to the existing public load balancer solely for HTTP-01. The public controller receives only dynamically created ACME solver routes, never an Assurance route. Private DNS resolves the same exact hostname to a Tailscale endpoint. A dedicated private HTTPS proxy terminates the certificate and routes ordinary pages and /api/auth/ to the web server, and /api/v1/ plus /api/health to the current API after stripping /api. Unknown API paths return 404. It is not the shared public Traefik controller. Cert-manager issues and renews the hostname certificate using the public HTTP-01 solver. Its self-check must resolve against public DNS even though user clients use private DNS.

A Tailscale Kubernetes operator deploys the private endpoint in the existing cluster. Tailnet grants restrict human and CI identities to Assurance HTTPS and the necessary private DNS resolver. GitHub Actions uses workload identity federation with explicit repository and workflow constraints. Joining the tailnet does not confer application-level producer or reviewer authority. Existing unrelated tailnet policy must not be overwritten.

## Deployment ownership

Canonical Azimuth owns service implementation, image packaging and reusable operator guidance. drim-dev-infra owns Terraform integration, workload configuration, an isolated database/account, private proxy, operator, DNS resolver and certificate resources. Images and external identity/configuration inputs are explicit. Configuration must remain non-deploying until required values are supplied. Live OAuth registration, tailnet registration, public DNS and Terraform apply remain rollout tasks.

## Terraform-managed credentials

Dima authorized Terraform ownership of runtime Secrets on 2026-10-06. Operator OAuth and GitHub OAuth credentials are sensitive deployment inputs. Stable numeric GitHub user IDs declare explicit project access. Producer, reviewer and owner identities are declared independently of their generated credentials; distinct per-project role/identity tokens, Web viewer tokens and a session secret are generated and retained in Terraform state. Kubernetes Secrets are created after their namespaces and before workloads. Secret updates roll out consumers; no automatic scheduled rotation is implied. Terraform state and saved plans contain secret values and require protected access. Dima selected manual delivery of each producer token to its GitHub Actions environment Secret. Terraform does not manage GitHub account resources; network federation does not confer producer authority.
