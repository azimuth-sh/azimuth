# Research: Continuous assurance service

Research date: 2026-08-12

This is a landscape review, not an exhaustive vendor survey, legal opinion or patent search. Facts
below come from the linked primary documentation and research papers; comparisons to Azimuth are
inferences from publicly described capabilities.

## Assurance-case lineage

The [NIST definition of an assurance
case](https://csrc.nist.gov/glossary/term/assurance_case) centers a reasoned, auditable set of claims,
argumentation, evidence and assumptions. The current [OMG SACM 2.3
specification](https://www.omg.org/spec/SACM/About-SACM) provides a machine-readable metamodel.
SACM's published evidence-model history includes provenance, custody, timing, support, challenge,
observations and resolutions that record judgments over evidence conflicts. This makes the
observation/challenge/judgment family established prior art rather than an Azimuth invention.

[Adelard ASCE](https://www.adelard.com/asce/) is a commercial Claims–Arguments–Evidence and GSN
environment with evidence-change tracking. [Assurance
Forge](https://assurance-forge.com/) is an open-source SACM-oriented safety-case tool with GSN
visualization and manual or AI-assisted review. These systems are conceptually close but target
explicit assurance cases rather than a lightweight routine software path.

## Continuous assurance research

The 2024 [Evidential Tool Bus study](https://arxiv.org/abs/2403.01918) describes distributed tool
execution, hash-addressed evidence, automatic claim construction and incremental maintenance of an
industrial automated-driving assurance case. [ACCESS](https://arxiv.org/abs/2403.15236) describes
assurance-case-centric engineering with links to heterogeneous artifacts and evaluation at both
development and runtime.

These works are the closest conceptual predecessors to an SDLC-wide Azimuth. Their safety-critical,
formal and model-based context leaves room for a smaller repository-native protocol, but Azimuth
should not claim to have originated continuous or runtime assurance cases.

## Structural traceability

The 2026 [ReqToCode paper](https://arxiv.org/abs/2603.13999) generates language-native requirement
objects referenced by implementation and test code, validates links during builds and gives
requirements a deprecation-to-failure lifecycle. It independently supports Azimuth's thesis that
traceability should be structural and created with the code, especially as agents produce more
code. It does not publicly describe Azimuth's evidence forms, mechanisms, judgment tier,
observations or progressive criticality.

Jama, IBM ELM, Polarion, Codebeamer, SAP Cloud ALM and Azure DevOps already connect requirements,
tests, executions and defects. For example, [Jama traceability](https://help.jamasoftware.com/ah/en/getting-to-know-jama-connect-features/traceability-from-requirements-to-test.html)
marks downstream relationships suspect after an upstream change, while [IBM
ELM](https://www.ibm.com/docs/en/engineering-lifecycle-management-suite/test-management/beta?topic=efforts-tracking-requirements-development-artifacts)
links requirements, development and test artifacts across lifecycle applications. Their existence
validates the lifecycle problem; their centralized artifact model and relationship-maintenance cost
are the adoption boundary Azimuth is trying to change.

## Execution ledgers and TestOps

[Kosli](https://docs.kosli.com/understand_kosli/how_kosli_works) models repeatable Flows, execution
Trails, hash-identified Artifacts, Attestations, evidence storage, runtime Environment Snapshots and
policies. Its [attestation documentation](https://docs.kosli.com/getting_started/attestations)
describes append-only results, artifact and Git binding, evidence files and compliance evaluation.
It is the closest commercial comparison for the proposed observation service.

[Allure TestOps launches](https://docs.qameta.io/use-testops/test-plans-and-launches/launches-overview/)
provide a shared execution context containing results, environment attributes, CI jobs, reruns and
triage. [ReportPortal](https://reportportal.io/docs/) stores real-time test results and history and
analyzes failures into product, automation and system issues. Both are natural observation
producers but do not ordinarily judge whether evidence semantically establishes a business claim.

## Agent decisions and production gates

[Pointbreak](https://withpointbreak.com/) preserves agent claims, observations, validations, open
questions and assessments against an exact captured revision. It is the closest public comparison
for Azimuth's change-level agent judgment, but it does not publicly describe an accepted current
specification model, generated RTM, criticality obligations or lifecycle observation federation.

[Harness Continuous
Verification](https://developer.harness.io/3k-docs/continuous-delivery/verify/verify-deployments-with-the-verify-step/)
queries APM and logging sources to evaluate canary, rolling, blue-green and load-test deployments
and can trigger rollback on anomalies. Its [Git-based monitored-service
configuration](https://developer.harness.io/docs/platform/git-experience/gitx-monitored-services/)
shows the useful split between versioned verification definitions and runtime execution. Harness is
deployment-health-centered rather than claim-centered.

## Evidence integrity and lifecycle interoperability

[in-toto](https://pkg.go.dev/github.com/in-toto/in-toto-golang/in_toto) verifies signed evidence
for declared supply-chain steps. [Sigstore Rekor](https://github.com/sigstore/rekor) provides a
tamper-resistant transparency log for signed software-supply-chain metadata. [SCITT, RFC
9943](https://www.rfc-editor.org/info/rfc9943/) defines a content-agnostic architecture for signed
statements and verifiable transparency receipts. These systems can protect Azimuth observation
identity and custody but do not determine semantic sufficiency.

[OSLC Core](https://docs.oasis-open-projects.org/oslc-op/core/v3.0/oslc-core.html) standardizes
linked lifecycle resources across requirements, quality, change, ALM and DevOps domains. It is a
candidate integration seam for enterprise systems rather than a replacement for Azimuth's
repository and area identity.

## Comparative result

| System family | Strongest overlap | Missing relative to the proposed composition |
|---|---|---|
| SACM, ASCE, Assurance Forge | Claims, evidence, challenge, observation and resolution | Lightweight source and routine path |
| ETB, ACCESS | Continuous distributed and runtime assurance | Ordinary product-team and agent workflow |
| ALM products | Requirements, tests, runs, baselines and impact | Source-derived linkage and low ceremony |
| ReqToCode | Structural implementation and test links | Judgment and lifecycle evidence |
| Kosli | Immutable attestations, artifacts, runtime state and gates | Scenario semantics and evidence qualification |
| Pointbreak | Agent evidence and assessment over exact work | Accepted model and generated completeness |
| Allure, ReportPortal | Durable test executions and triage | Product-claim semantics |
| Harness | Production observations and rollout gating | Cross-lifecycle claim graph |
| in-toto, Sigstore, SCITT | Provenance, signatures and receipts | Semantic truth and sufficiency |

The market is fragmented rather than empty. Azimuth's defensible contribution is the composition:
an OpenSpec-like routine path that progressively connects consequential claims to implementation,
mechanisms, qualified evidence, challenges, agent judgment and revision-bound lifecycle
observations.
