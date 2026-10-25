---
name: azimuth-author-change-outcome
description: Record what an implemented Azimuth change actually established, its departures, verification, remaining limits and operational conditions. Use for author-change-outcome.
---

# Author a change outcome

Load `azimuth reference show retained-inputs` before assembling or relying on the final durable evidence package.

Read the captured handoff, proposal, reviewed deltas and exact candidate, engineering progress, current reviews, required Check/Challenge records and dispositions. Load `azimuth reference show outcome`.

Author `outcome.md` with factual `Status: complete`; actual human acceptance is a separate later record, so do not edit this pinned outcome merely to mark approval. Record delivered behavior against each completion condition; explain accepted departures, actual engineering verification, unperformed work, residual risks and rollout-dependent conditions. Keep source-level completion distinct from build, deployment and observed production behavior. Preserve adverse records and do not infer success from a command exit or file presence.

Identify the exact retained basis of the outcome. Every relied-on review, Run, Challenge disposition and supporting input must be retrievable under the installed retention contract. Ignore-only local locators do not suffice for CI. Do not copy every scratch artifact merely to make a large package.

If writing the outcome reveals a material unresolved obligation or departure, return to its owning task and decision rather than disguising it as a limitation. There is no separate verify-change task to compensate for incomplete task contracts.

Retain the outcome and return for actual human acceptance. Do not author an acceptance decision on your own initiative, finalize, archive, publish or commit implicitly.
