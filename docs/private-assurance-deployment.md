# Private Assurance State Service deployment

## Ownership and access

The service source and image definitions live in canonical Azimuth. Terraform deployment is owned by drim-dev-infra. The intended origin is https://assurance.drim.dev. The first Web UI is inspection-only: GitHub sign-in identifies a human and an explicit stable-user-ID/project mapping authorizes reads. A server-side viewer token carries no producer, reviewer or owner authority and is never sent to the browser. Machine submissions continue to use separate project-scoped credentials.

Tailscale connectivity and application authorization are independent. Humans and authorized GitHub Actions jobs can reach the private HTTPS endpoint; joining the tailnet grants neither project access nor permission to append evidence. GitHub Actions federation must constrain the trusted repository and workflow identity. Untrusted jobs must not obtain network or ingestion credentials.

## One hostname, two network paths

Public DNS resolves assurance.drim.dev to the public load balancer. The only public route owned by this deployment is cert-manager's temporary /.well-known/acme-challenge/ route. There must be no public Assurance application route. Tailscale-connected devices resolve the exact hostname to the private endpoint through the configured DNS resolver. Other drim.dev names must retain their normal resolution.

A dedicated private proxy terminates HTTPS with the cert-manager certificate. Pages and /api/auth/ go to the web process; /api/v1/ and /api/health go to the current ledger API with /api stripped. Unknown API routes return 404. cert-manager's HTTP-01 self-check must use public resolution rather than the private client answer. GitHub redirects the user's browser to the callback; the callback is part of the private web origin.

## Required operator inputs

Supply GitHub OAuth credentials registered for the configured web origin and /api/auth/callback/github, Tailscale operator OAuth credentials, stable numeric GitHub user IDs with explicit project access, project producer/reviewer/owner identities, and published API/web image digests through the infrastructure inputs. Terraform generates distinct role credentials, a Web viewer credential for each project and a session secret, then creates the Kubernetes Secrets. Generated credentials persist across ordinary applies. OAuth secrets and generated tokens remain sensitive; Terraform state and saved plans still contain their values and require protected access. Deliver the selected producer token manually to its GitHub Actions environment Secret using the infrastructure documentation; Tailscale federation supplies connectivity independently. Configure public DNS at its authoritative provider and private resolution in the tailnet. Infrastructure documentation describes resource-specific variables and staged provisioning where custom-resource discovery requires it.

## Verification and rollout

Terraform configuration is preparation, not a deployment result. Before operator apply, inspect the plan, image selection, database ownership, private routes and tailnet grants. After deployment, verify sign-in allowed/denied paths, project confinement, rejected viewer writes, authorized machine submissions and current exact-Subject state inspection. Separately verify public unreachability, private resolution, certificate hostname and renewal, CI federation and database persistence/recovery. Preserve unresolved or failed checks as such; an inspection UI must not imply that missing evidence establishes a Claim.

The earlier isolated service API is not a source for current Assurance State. The current service owns immutable Run and review histories, explicit selected model authority and exact-Subject evaluation. This deployment does not automatically accept model authority, author reviews, schedule challenges or finalize a change.

## Image-only publication

The manually dispatched publish-assurance.yml workflow builds and publishes only the API and Web images from main. Its assurance-images GitHub environment permits main branch deployments only and uses the workflow token for registry writes. It uses native Linux AMD64 and ARM64 builders and produces a multi-platform registry digest for each image. Its tags include the exact source commit and workflow execution identity. This lane performs builds and manifest inspection only; it does not run tests, deployment qualification, full Azimuth release publication or live deployment. Choose immutable registry digests for Terraform. A successful build or provenance record does not establish runtime authorization or network isolation.
