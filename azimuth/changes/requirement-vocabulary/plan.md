# Plan: Rename Claim-first vocabulary to Requirement, Scenario and Verification Target

- [x] Approve the proposal, design and work packages (Dima).
- [x] Core first: rename in `tools/azimuth`, regenerate frozen vectors, add the migration edge and successor migration line, replace the retired-vocabulary rejection.
- [x] Migrate the framework's own model and active changes with `azimuth migrate`; rewrite contracts, glossary and skills.
- [x] Rename packages, extractors and the Python protocol mirror against the new core.
- [x] Delete the legacy gate API; rename ASS domain, server, relational schema (version 6, fresh initialization only), routes, web and development data; rename Collector and Reconciler.
- [x] Update release tooling and workflows; publish tooling, packages and images from CI (Azimuth `d7b3383`).
- [x] Update drim.dev, GitOps and infrastructure repositories and tooling pins; migrate authored artifacts.
- [x] Stop producers, delete all ASS data, initialize the fresh schema, redeploy ASS, Collector, Reconciler, observer and application, restart producers.
- [x] Publish and deploy a new drim.dev release (`sha256:a37b2ce9…`, drim.dev `db22fdd`, GitOps config `86dcc29`, merged `f5f1223`), fresh independent Requirement Reviews and CI Runs published.
- [ ] Production verification is partial: the deployment is observed and all 14 Requirements are present, but every Requirement is Unresolved (see Outcome notes).
- [x] Final terminology audit and report. Release acceptance and archival remain separate decisions.

## Outcome notes

- A reset ASS accepts target observations before any release exists. They cannot resolve (no reconciliation bracket) and the resolver retries all of them every cycle, which starves the ledger and stalls the Collector outbox. Register the release before restarting producers after a reset.
- Owner actions needed after a reset beyond the baseline authority: register the application's Context Contracts.
- `alpha.7` resolves `azimuth/standards` relative to the working directory; a different working directory changes `model_fingerprint`.
- `alpha.7` cannot load pre-rename Claim Review records, so successor Requirement Reviews carry no native `Reassesses` link.
