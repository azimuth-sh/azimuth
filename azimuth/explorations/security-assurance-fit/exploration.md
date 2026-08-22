# Exploration: Security assurance fit

Id: security-assurance-fit Status: complete

> **Provenance warning.** This exploration was written against `docs/decisions.md`, a 3,013-line narrative that was later found to have been generated in one unsupervised commit and was deleted. Every `D<n>` citation below is unresolvable, and findings resting on them are unverified. Three corrections are known: the corpus claimed six Claim domains where the parser has two (`behaviour | sites`); every measurement it reported is unreproducible in this repository; and an accepted Claim Judgment is a required positive decision, not a negative-only one. Read this as recorded reasoning, not as a source. Re-derive anything load-bearing from `azimuth/formats/`, `tools/azimuth/tests/` and `azimuth/changes/archive/`.


## Objective

Determine whether the alpha 2 model can represent the verification activities of professional security practice, and whether extending it to cover them in full would benefit the product.

## Boundaries

- Assess against the current corpus only: D1–D48, the format contracts, project standards and the existing explorations. No adopting security team was consulted, so every finding below is derived from the model rather than from a consumer.
- Perform no landscape review. Unlike `continuous-assurance-service`, this exploration has no primary-source survey of application-security or vulnerability-management products. Comparative positioning claims are correspondingly weaker and are marked as such.
- Propose no notation. Evidence precedes notation; this exploration may identify pressure and name candidate work, and may not design a mechanism.
- Treat security as one lens over the general model. A finding is only interesting here if it is either genuinely security-specific or a general defect that security exposes unusually clearly.

## Existing context

- D1 chose the ride-hailing fixture partly for "authorization on every trip-scoped read". A security control was one of the four founding cross-cutting concerns, so control enforcement is native to the design's origin. Vulnerability management never was.
- `docs/assurance-extensions.md` already classifies the two hard cases: "A penetration or exploratory session is a Challenger by default because negative search does not imply product satisfaction", and "Broad static analysis is normally a Challenger because a clean search does not establish product behavior."
- E8 of `evidence-control-plane-alpha-2` lists "dependency and vulnerability analysis, DAST, penetration testing, red-team exercises" among candidate Challengers. F2 and F4 of `composable-assurance-extensions` name security testing among techniques requiring no new core semantics, and state that a broad run with no findings establishes no product predicate. Both are non-normative and neither produced a decision, so the question is unresolved rather than unasked.
- D45 removed Strength. D23's expiring external receipts and D4.2's freshness cadence were removed with the alpha 1 evidence model. `azimuth/standards/verification.md` states the current position: "Alpha 2 defines no cache, cadence, TTL, historical applicability, cross-Subject reuse or time-based deferral."
- D27 and open question 7 already record the cross-cutting-application gap in a non-security context.

## Findings

### F1 — The Check/Challenger split already encodes security epistemics correctly

The proposition test, not the executable brand, decides the role, and a `clean` Challenge Result cannot become positive product evidence. Practitioners know that absence of findings is not absence of vulnerabilities; almost no tool enforces it structurally. Azimuth does, in three independent places: the Challenge Result definition, the Decision Policy semantics and the extension acceptance boundary ("broad analysis creates no implicit product evidence").

This is the strongest single result of the assessment and it required no new model.

### F2 — Control-coverage claims are the best fit and are currently unbuildable

"Every route enforces authorization", "every mutating endpoint is idempotent" and "every dependency is inventoried" are site-domain Claims over a build-derived surface, which is precisely what D13.1 and D41 were designed for. D13.1's rationale is the security inventory problem stated exactly: "A hand-listed surface is worse than no rule at all."

The limit is implementation, not semantics. One enumerator exists (`next-routes`). No route-table, dependency-graph or infrastructure enumerator exists, so the capability the framework was designed around cannot currently be demonstrated on the surface that would show it best.

### F3 — Adversarial input domains cannot satisfy D13.1, and that is correct

Enumerable security domains — routes, sinks, buckets, dependencies — work. The domain that carries most security risk, the set of inputs an adversary can construct, is not derivable from the source the system is built from, so no enumerator can witness it and every Claim over it stays `example`-quantified permanently.

The consequence is a property, not a defect: Azimuth can express "these declared controls are enforced at these enumerated sites" and can never express "this system is secure." That boundary should be stated in the product's own words rather than discovered by an adopter.

### F4 — A Challenger can attack the evidence account but never the product

All seven Challenge Plan selectors resolve to a Qualification or a Claim Judgment, and all seventeen semantic scope kinds are model objects. A practitioner who finds a defect in a code path governed by no Claim has no target, and the candidate dispositions describe broken links in a declared graph rather than discovered behavior outside it.

This is the deepest mismatch, and it is not security-specific. Any Challenger, exploratory session or dogfood run can surface behavior no Claim covers, and today that observation leaves no trace in the model. In a closed-world account the correct reading is a gap in intent completeness.

### F5 — Repository decisions decay only with repository content

Qualification staleness is fingerprint-driven. A vulnerability disclosed against an unchanged pinned dependency changes no Check, binding, Claim, policy or context fingerprint, so every decision remains current while the world has moved underneath it.

The framework previously modelled this and named the failure mode precisely — D4.2: "A monitor that can no longer fire is worse than no monitor, because it is carried on the books as evidence" — and D23 required an expiry on every imported external result. D45 removed the mechanism without replacing the concern. Security is the sharpest lens on the loss, not the reason to repair it.

### F6 — No vulnerability entity exists, and adding one would reintroduce what D45 removed

There is no severity, exploitability, reachability, remediation deadline or risk acceptance anywhere in the model. Run-bundle diagnostics carry a closed `class` and a `info | warning | error` severity, but "Diagnostics explain facts but do not determine outcomes", so they are transport-level and not a finding inventory.

A vulnerability queue wants severity, cadence, TTL and accepted-risk-with-expiry: Strength, freshness and expiry under new names. Granting them to one domain forces either a security-only fork of the decision model or a global reversal justified by a domain with no adopter.

### F7 — Cross-cutting control application is already a recorded unresolved gap

D27's own product validation states that the machine "does not derive every business path that must call it", and open question 7 generalizes it: a check over a set "can assert only that a discharge *exists*, not that it is right." "Is the auth verifier actually invoked on every path, and correctly?" is the same question. It is residue today.

### F8 — Deployed-versus-declared configuration is explicitly residual

D26: "Source bindings prove the requested topology exists in code. They do not prove a particular environment deployed it unchanged." Infrastructure-as-code review fits as a code-artifact or site-domain Claim; cloud configuration drift, which is the majority of real infrastructure findings, is a named deferral rather than an unbuilt feature.

### F9 — Two practical blockers stop any real security adapter today

`azimuth/formats/adapter.md`: "Version 1 supports no secret value, secret reference or interpolation syntax", and core clears the child environment. Scanner tokens, cloud credentials and test accounts have no path to an adapter.

Separately, launch inputs project scope items into source, Artifact, enumeration or surface-member locators only. A dynamic Challenger receives no endpoint; a `deployment` Subject carries an environment id, a deployment id, a fingerprint and artifact digests, but no address. A non-secret base URL could travel in `semantic_settings`; credentials cannot travel at all.

### F10 — The volume and ceremony models are mismatched

Sparseness is a defended property: enrollment exists "to prevent thousands of native test cases from becoming accidental assurance authority." The ceremony per node — Check, binding, Qualification, Claim Judgment, each fingerprinted and each carrying a named accountable human — is priced for tens of propositions. A single dependency scan emits thousands of findings that nobody will qualify.

## Decisions

- **E1 — Reject support for security activities in full.** Full coverage means a finding queue, severity, remediation SLAs, risk acceptance, feed ingestion and asset inventory. That is a posture-management product entering a crowded market from behind, and F6 and F10 show it cannot be grafted onto the current model without either forking the decision semantics or drowning the graph.
- **E2 — Adopt security as a design pressure test rather than a supported domain.** It stresses closed-world assumptions, negative-search epistemics, decay and enumerator soundness harder than functional correctness does, and it has already produced F4 and F5 at no product cost.
- **E3 — Treat time-based staleness as a general core repair.** Frame it as decisions that decay with the world rather than with repository content, with D4.2 and D23 as prior art. It is not a security feature and must not be introduced as one.
- **E4 — Give an unattributed finding a home.** A Challenger result, exploratory session or dogfood observation that maps to no Claim is a gap in intent completeness. Recording it is cheap and benefits every domain.
- **E5 — Build enumerators and one worked coverage surface.** This needs no model change and demonstrates the capability F2 identifies as the strongest existing fit. The founding fixture concern, authorization over trip-scoped reads, is the natural subject.
- **E6 — Position as security-control assurance, not vulnerability management, and say so.** Do not describe Azimuth as supporting security work until E3 ships. Rendering `clean` without modelling decay manufactures the exact false assurance D4.2 named, now with a compliance artifact attached.

## Rejected alternatives

- **Full vulnerability management inside the model.** Rejected by E1. The decisive objection is not effort but semantics: the constructs it requires are the ones D45 deliberately removed.
- **A security-only lane with relaxed ceremony.** Rejected because a second lane with different rules is the beginning of a second framework, and it would place the highest-volume, lowest-review data beside the model's most carefully reviewed decisions.
- **Reinstating D23 expiring receipts as a security feature.** The mechanism is close to right and the framing is wrong; E3 covers the same ground without scoping a core repair to one domain.
- **Marketing the current fit as security support.** Rejected by E6. Partial support here is worse than none.

## Open questions

1. Whether E3 and E4 survive contact with a non-security consumer. Both were derived through a security lens against the corpus, so evidence-precedes-notation is not yet satisfied for either. They remain proposed pressure, not accepted scope.
2. Which enumerator the first worked surface in E5 should use, and whether D27's cross-spec mechanism composition is a prerequisite for it or merely adjacent.
3. Whether F7's per-member discharge problem has any answer short of the agent tier, given that open question 7 has stood unresolved since phase 0.
4. Whether a landscape review would change E1. This exploration performed none, and the market claims supporting E1 are inference rather than surveyed fact.

## Result

No change is created from this exploration. Its disposition is that security is a lens, not a product direction, and the resulting candidate work is E3, E4 and E5, in that order of leverage. Each must be justified on general grounds before it is proposed; none inherits authority from this document.

## What would falsify this

- **E1 is wrong** if an adopting team's security obligations turn out to be expressible as declared controls over enumerated surfaces in practice and the only thing missing is volume handling — that would make full support an increment rather than a different product.
- **E3 is misframed** if no non-security concern ever demands time-based staleness, which would make it a domain feature that E1 has already rejected.
- **F3 is wrong** if a real adversarial concern is given a sound derived enumerator, which would mean the six-domain closure absorbs adversarial search after all.
- **F1 is decorative** if practitioners route security tooling through Checks anyway to obtain green results, which would show the role split is advisory rather than structural.
