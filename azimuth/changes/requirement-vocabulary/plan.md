# Plan: Rename Claim-first vocabulary to Requirement, Scenario and Verification Target

- [x] Approve the proposal, design and work packages (Dima).
- [ ] Core first: rename in `tools/azimuth`, regenerate frozen vectors, add the migration edge and successor migration line, replace the retired-vocabulary rejection.
- [ ] Migrate the framework's own model and active changes with `azimuth migrate`; rewrite contracts, glossary and skills.
- [ ] Rename packages, extractors and the Python protocol mirror against the new core.
- [ ] Delete the legacy gate API; rename ASS domain, server, relational schema (version 6, fresh initialization only), routes, web and development data; rename Collector and Reconciler.
- [ ] Update release tooling and workflows; publish tooling, packages and images from CI.
- [ ] Update drim.dev, GitOps and infrastructure repositories and tooling pins; migrate authored artifacts.
- [ ] Stop producers, delete all ASS data, initialize the fresh schema, redeploy ASS, Collector, Reconciler, observer and application, restart producers.
- [ ] Execute Checks, publish fresh Runs, publish and deploy a new drim.dev release, verify production.
- [ ] Final terminology audit and report. Release acceptance and archival remain separate decisions.
