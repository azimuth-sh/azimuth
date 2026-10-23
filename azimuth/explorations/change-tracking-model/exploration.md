# Exploration: Change tracking model

Id: change-tracking-model
Created: 2026-10-23
Status: exploring

## Objective

Decide what Azimuth should track and hash to answer one question: is an earlier review or earlier evidence still valid for what exists now. Today's answer mixes three granularities (source site, test unit, exact Verification Target) and several scopes that the owner suspects are wider than what a review actually vouches for. The exploration also covers how a change of hashing scheme can be rolled out without a re-review campaign, and what the same tracking data must provide to a future agent-context feature that helps an agent repair a failed Requirement.

## Boundaries

- This record is non-normative. It authorizes no implementation, proposal or contract change. Every slice named in `change-map.md` needs its own proposal.
- Consumer observations (Drim) are evidence that motivates questions. They are not executable dependencies, and tooling fixtures stay synthetic.
- Heavy code analysis stays in extractors. Core reads manifests only. Any normalization or syntax-aware hashing discussed here is an extractor concern.
- The alpha design phase carries no backward-compatibility obligation unless an accepted change states one. The Azimuth Assurance Service nevertheless holds production state for Drim, so migration cost is a real operating concern; see open question 6.
- Out of scope: Requirement discovery, Decision Policies, Challenge scheduling and the Run protocol shapes except where they pin fingerprints.

## Existing context

Facts below were read from the repository on 2026-10-23. `research.md` holds the sourced detail and the two comparison tables.

- A Requirement review pins `requirement_fingerprint` and an exact dependency packet. That packet includes realization sites and their fingerprints, mechanisms, Check implementations, Verification Challenge declarations and implementation or configuration support (`contracts/assurance-reviews.md`). Review currentness checks those dependencies.
- Source-site fingerprints are produced per ecosystem by extractors and are bare `sha256:<hex>` values. The scheme that produced a value is not recorded with it.
- Granularity differs by language and by marker kind. A Python Check implementation hashes the whole module's bytes. A Python realization hashes the enclosing function or class text. Rego hashes the whole file. TypeScript hashes the declaration's raw text. .NET hashes the raw lines of a method for a site, and hashes every source file declaring the type, nested types included, for Check support. Helm hashes the raw template file, the chart's default `values.yaml`, any supplied values files, the rendered resource and the Helm version.
- Evidence applicability is separate from review currentness. Runs pin the exact Verification Target (source revision with a producer-asserted content fingerprint, artifact digests, or deployment fingerprint) and the Check fingerprint. Hermetic units can reuse earlier results when a unit input fingerprint is equal (`contracts/unit-inputs.md`).
- The repository already states that requalification follows definition drift and not re-execution.

## Findings

Fact, inference and owner direction are kept apart.

**Facts (verified by running the emitters).** Between two Drim deployment-repository commits (`f8b148a` and `ca1ed5f`), the Helm site fingerprint of an unchanged NetworkPolicy stayed equal. The fingerprint of a Service changed although its rendered document was byte-identical, because the raw template file, which also holds a Deployment, changed. The fingerprint of a Rego Check changed because 64 lines of policy were added. Both changed fingerprints belong to the same Requirement, so its review basis changed for one legitimate reason and one false positive. Details are in `research.md`.

**Facts (from code and contracts).**
- A Check fingerprint covers id, method text, terminal and its implementations' identity and source fingerprint.
- Preimages for Check, Requirement digest and Requirement judgment fingerprints carry a format and a version. Extractor source fingerprints do not.
- Only seven Mechanisms are tagged in Drim application code, all authentication or admission controls.
- A new Drim NetworkPolicy and gateway template that affect the isolation property carry no `Realizes` marker, so the review basis does not include them.

**Inferences.**
1. A review vouches for two things: that the verification approach is adequate (definitions) and that the verification implementation is faithful (Check code, fixtures, oracles). It does not vouch that production code is correct; Runs establish that against an exact subject. Tracking production-side sites as review inputs therefore forces human re-review for changes that re-execution already covers.
2. A scheme change makes every pinned review and Run stale because history is immutable and pins old values. Forward recomputation is cheap; carrying existing reviews across is not, unless an explicit migration exists.
3. Static call-graph dependency tracking through a framework application is unreliable (dependency injection, mediator dispatch, middleware, reflection). Declared edges (Mechanisms) and measured edges (executed sites) are more dependable than inferred ones.
4. Tags as a navigation map and fingerprints as an invalidation trigger are separate jobs. Agent context needs the map to be complete and fresh; it does not need hashes.

**Owner direction so far.** Do not invest heavily now; stay on the current approach and iterate. An explicit migration step is desirable in principle: validate every review under the current scheme, roll out the new scheme, recompute the hashes of the still-valid reviews. The exploration is recorded to be resumed later.

## Decisions

None are approved. The following are candidates for the owner to confirm or reject when the exploration resumes.

1. **Pin what was executed; track only what a review vouches for.** Reviews depend on definitions and Check implementations. Production-side sites, rendered infrastructure resources and unrelated files do not stale a review; a production change triggers re-execution against the new subject.
2. **Tags remain a navigation index** for Realizes and Mechanism sites even when their fingerprints stop gating reviews.
3. **Fingerprints carry a scheme version** so that an extractor change is an explicit migration, not a silent mass invalidation.
4. **Carry-forward migration.** A scheme change ships with an offline migration that recomputes under both schemes at pinned commits, appends immutable carry-forward attestations for reviews and Check-pinned Runs that were current under the old scheme, and applies only when the new scheme covers the same content or less. Widened coverage needs review of the added part.
5. **Helm** sites hash the canonical rendered resource only, and markers live in the manifest as annotations.
6. **Python Check implementations** hash the Check's function, not the module. .NET Check support hashes the method and declared helpers, not every file of the type.

## Open questions

1. Is the review's "design credibly explains the implementation" judgment important enough that a production rewrite should require an explicit design-changed decision, and how is that decision recorded?
2. Should a Mechanism's site changes be an advisory signal to reviewers, a gate, or ignored?
3. What belongs in a Check's declared test-side dependencies (fixtures, helpers, data), and who declares them?
4. How does the carry-forward record fit the review contract without counting as an independent Requirement Review? It must be authenticated as a publisher or CI act.
5. How is "same or narrower coverage" verified mechanically by the migration tool?
6. Given the alpha no-compatibility rule, is a carry-forward worth building before external users, or is one re-review per scheme change acceptable until then?
7. Must the reviewed basis be retained as text or a Git reference to support a reviewed-versus-current diff? Past reviews cannot be backfilled.
8. Should Runs record the production sites they executed, for agent context and test selection, and at what recording cost?
9. How is map completeness checked (untagged relevant files and rendered resources), and is that a lint, a Verification Challenge or a review obligation?
10. What would an agent context brief for a failed Requirement contain, and which parts need new recorded data (diff between the last satisfied and the failing release, executed sites, Run diagnostics, a completeness caveat)?
11. When do normalized token or syntax-tree hashes pay for themselves, and what trigger (for example repeated re-reviews forced by non-semantic edits) should prompt that work?

## Result

Pending. The candidate change graph is in `change-map.md`.
