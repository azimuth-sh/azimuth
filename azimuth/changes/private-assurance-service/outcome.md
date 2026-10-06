# Outcome: private-assurance-service

Status: implemented; rollout pending

## Implemented

The current API includes a distinct project-scoped viewer role and bounded inspection reads for Runs, revision history, exact Subjects and independently assessed reviews. The inspection-only Web UI uses GitHub OAuth, stable-user-ID per-project allowlisting, secure sessions and server-confined viewer credentials. Container packaging now builds and starts the current Run ledger; earlier isolated server source is retained. The local Compose profile requires explicit runtime secrets and retains loopback bindings.

The infrastructure repository contains Terraform modules for the private application, dedicated HTTPS proxy, Tailscale operator and exact-host private DNS, an isolated CNPG database/role, scoped ingress policies, HTTP-01 certificate resources, scoped tailnet policy fragments and a GitHub Actions federation example. Deployment remains disabled until explicitly configured; published API/Web image digests are required. Existing unrelated source changes and the infrastructure .serena directory were preserved.

## Engineering observations

- Final API host compilation and actual Docker build passed without warnings. Final build logs: /private/tmp/assurance-api-host-build.log and /private/tmp/assurance-api-image-build-final.log.
- Web typecheck, production build and actual Docker build passed. Final image build log: /private/tmp/assurance-web-image-build-final.log. Dependency audit reported zero vulnerabilities.
- Production Terraform init with backend disabled and terraform validate passed. Formatting checks pass for modified Terraform files/modules. A broader repository-wide format check identifies eleven pre-existing unformatted files outside the change; they were left untouched.
- Independent source review found and resolved a history-page memory issue. SQL now caps transferred payloads at 32 MiB before application materialization, excludes lookahead payloads and reports oversized pages explicitly. No separate racy preflight or truncated evidence is used.
- Final source diff whitespace checks pass.
- Prospective Azimuth change validation reports structural validity with no structural errors. It retains nine baseline missing design-binding input gaps and acceptance blockers for unapplied intent and the active proposal. The intent preview is /private/tmp/private-assurance-service-intent-preview.json; no accepted intent or archival action is fabricated.

## Unperformed and residual work

Dima selected builds and inspection only. No tests, application startup, database behavioral checks or live OAuth/network checks were run. The existing private-deployment qualifier covers the earlier unauthenticated profile and is not evidence for this implementation; qualification alignment and behavioral verification remain outstanding under that constraint.

Deployment requires publishing the rebuilt images, provisioning GitHub OAuth and runtime Secrets, enrolling the Tailscale operator, merging reviewed grants/federated identity restrictions, configuring public DNS and the exact-host private nameserver, and a reviewed Terraform plan/apply. Certificate renewal, private isolation, database permissions and persistence need live verification afterward. Tailnet and DNS administrative settings were documented, not silently changed. No Terraform apply, live account mutation, publication, commit or deployment occurred.

## Departures

No implementation departure from the agreed private-network, GitHub-login and inspection-only scope. Behavioral and deployment qualification were not performed because Dima selected builds and inspection only.

## Residual decisions

The operator must supply real identity/Secret/image/DNS inputs and approve the deployment plan. The active intent delta remains prospective until the acceptance boundary. Tailnet policy ownership remains external; existing broad grants must be reviewed rather than overwritten.

## Terraform credential refinement

Dima requested Terraform-managed Secret creation and credential generation from tfvars inputs. The refinement is implemented and supersedes the earlier external-runtime-Secret provisioning instructions. Terraform now creates the GitHub/Web, API and Tailscale operator Secrets, generates stable distinct 64-character project/role/identity credentials and a session secret, and rolls API, Web and operator consumers when their Secret resource versions change. Explicit numeric GitHub user/project access and role identity declarations are validated. The sensitive assurance_producer_credentials output exposes producer tokens only; Dima selected manual GitHub Actions environment Secret delivery, documented as a direct pipe without displaying the token. No GitHub provider was added.

Production terraform validate, targeted formatting and source diff checks passed after the refinement. State and saved plans retain sensitive values and require protected access. No tests, deployment or account mutations were performed. The previous source and image build observations remain unchanged.

## Image-only publication preparation

Dima authorized publishing only the two Assurance images without tests. The dispatch-only publish-assurance.yml workflow pins source to the main commit, builds API and Web on native Linux AMD64 and ARM64 runners, assembles and inspects registry indexes, and retains exact source/run/digest records and provenance. Its labels report build-and-inspection-only verification. It invokes no tests or full package release workflow. The source commit uses skip ci to honor this constraint. Independent read-only workflow review found no blocking issue; attempt-qualified artifact names prevent mixing retained rerun artifacts. Publication is pending; no successful registry result is claimed yet.
