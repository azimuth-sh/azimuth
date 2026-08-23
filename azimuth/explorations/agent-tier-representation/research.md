# Research: agent tier — prior art for a declared acceptor requirement

Non-normative. External literature gathered 2026-08-22 to ground F18–F26. Nothing here is a
repository dependency; the sources are cited so the findings can be checked or overturned.

## Criticality-indexed independence (F18)

- IEC 61508 Table 5 — assessor independence by SIL: independent person / preferably independent
  department / preferably independent organisation / independent organisation.
  <https://share.ansi.org/Shared%20Documents/News%20and%20Publications/Other%20Documents/IEC%2061508%20Commented%20Version.pdf>
- DO-178C — objectives satisfied "with independence"; count scales with DAL. Secondary sources
  give roughly 16 of 31 review objectives at Level A and 7 at Level B; they disagree, so verify
  against the Annex A tables before quoting a number.
  <https://blog.adacore.com/a-fresh-take-on-do-178c-software-reviews>
- ISO 26262 — ASIL-indexed confirmation measures (confirmation review, functional safety audit,
  functional safety assessment). <https://www.jamasoftware.com/requirements-management-guide/automotive-engineering/asil/>

## Declared functionaries (F19, F25)

- in-toto specification — layout declares steps and, per step, authorized functionaries by key
  with a signature threshold; a functionary is "an individual or automated script".
  <https://github.com/in-toto/docs/blob/master/in-toto-spec.md>
- SLSA supplies graduated build-integrity levels over in-toto attestations; it declares platform
  properties, not approvers. <https://slsa.dev/blog/2023/05/in-toto-and-slsa>
- Gerrit label permissions — per project, ref and group, which label values a user may set.
  <https://gerrit-review.googlesource.com/Documentation/config-labels.html>
- GitHub CODEOWNERS and the required-reviewer ruleset — versioned, path-scoped, humans only.
  <https://github.blog/changelog/2026-02-17-required-reviewer-rule-is-now-generally-available/>

None of these places the declaration inside the reviewed model that states the claim.

## Shipped confirmer independence (F20)

- Copilot coding agent cannot approve or merge its own pull requests, and approvals from users
  who collaborated with it do not satisfy review requirements.
  <https://www.developersdigest.tech/blog/agent-pr-governance-github-copilot-review>

## Measurement (F21)

- Sber, *AI-DISRUPT PDLC* v2.0, June 2026 — human override rate target 10–25%; below 5% read as
  formal "rubber" approval and a hidden governance risk; approval-to-override optimal 75:25.
  Anti-pattern signature: median review under two minutes, agreement above 95%.
- Azevedo, *Minimal Oversight: Uncertainty-Aware Governance for Delegated AI Systems*, arXiv
  2606.15563 — masking ratio of corrected to uncorrected success rate. Single-author preprint.
  <https://arxiv.org/html/2606.15563v1>
- Anthropic, *Measuring AI agent autonomy in practice* — ~93% of permission prompts approved;
  auto-approve rates rise with experience. <https://www.anthropic.com/research/measuring-agent-autonomy>

## The promotion loop (F22)

- ITIL standard change versus normal change; promotion through a reviewed standard change
  proposal held in a versioned catalogue. ServiceNow adds risk-driven auto-approval routing,
  described in vendor community material rather than in ITIL — do not overstate it.
  <https://www.servicenow.com/community/itsm-blog/best-practice-make-the-most-of-standard-changes/ba-p/2267543>

## Autonomy as a parameter, and trust (F23, F26)

- Scerri, Pynadath & Tambe, *Towards Adjustable Autonomy for the Real World*, JAIR 2002 —
  transfer-of-control strategies chosen by expected decision quality against delay and
  coordination cost. <https://arxiv.org/abs/1106.4573>
- Horvitz, *Principles of Mixed-Initiative User Interfaces*, CHI 1999 — expected-utility test for
  acting versus asking. <https://courses.ischool.berkeley.edu/i296a-4/f99/papers/horvitz-chi99.pdf>
- Variable-autonomy terminology: adaptive (agent varies its own), adjustable (operator varies),
  mixed-initiative (negotiated). <https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11576532/>
- Lee & See, *Trust in Automation: Designing for Appropriate Reliance*, Human Factors 2004.
  <https://journals.sagepub.com/doi/10.1518/hfes.46.1.50_30392>
- STAR Levels — autonomous capability and trust as orthogonal axes, arXiv 2210.09059.
  <https://arxiv.org/pdf/2210.09059>
- Feng, McDonald & Zhang, *Levels of Autonomy for AI Agents*, arXiv 2506.12469 — levels named by
  the human's role (Operator, Collaborator, Consultant, Approver, Observer); autonomy case and
  autonomy certificate. <https://knightcolumbia.org/content/levels-of-autonomy-for-ai-agents-1>
- SAE J3016 for the six-rung convention and fallback ownership. <https://users.ece.cmu.edu/~koopman/j3016/>

"Earned autonomy" was expected to be a term of art in aerospace certification; that could not be
substantiated. STAR Levels and incremental flight-test-to-clearance work cover the concept.

## Regulatory position (F24)

- EU AI Act Article 14 — oversight by natural persons with competence, training and authority;
  separate verification by two natural persons for some systems; no performance-based reduction.
  <https://artificialintelligenceact.eu/article/14/>
- NIST AI RMF — continuous monitoring, non-prescriptive.
  <https://airc.nist.gov/airmf-resources/playbook/measure/>
- ISO/IEC 42001 — documented oversight within an AI management system; no relaxation clause.

## Assurance-case literature

GSN and SACM annotate what supports a claim. Nothing found in either annotates who is permitted
to confirm one; independence lives in the process standard, never inside the argument structure.
<https://en.wikipedia.org/wiki/Goal_structuring_notation>

## Negative result

No system — research or product — was found in which the permitted confirmer of a decision is
declared inside a reviewed, versioned model, each proposal and acceptance is recorded with
identity and outcome, and that record is the formal justification for amending the declaration.
Three partial cases each lack a leg: ITIL promotion has the record and the artifact but the
declaration is workflow configuration; Azevedo has continuous measurement but no declaration;
vendor progressive-permissioning describes promotion on metrics as intent, not mechanism.

## Uncertainties

Three of the most on-point sources are recent single-author preprints with no visible peer
review. The DO-178C objective counts vary between secondary sources. Russian-language search
around the Sber whitepaper was shallow.
