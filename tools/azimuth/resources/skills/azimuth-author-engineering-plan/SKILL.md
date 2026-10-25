---
name: azimuth-author-engineering-plan
description: Write a dependency-ordered engineering plan from reviewed spec, design and verification deltas, with validated path-isolated packages when delegation is useful.
---

# Author an engineering plan

Read the original request, captured handoff and complete reviewed spec/design/verification candidate. Plan concrete engineering work without deciding missing product, authority, compatibility or completion semantics. Return material gaps to their owning delta and review.

Cover production implementation, required interfaces/configuration, Check/Challenge code, expectations, instrumentation, capture, retained records, rollout prerequisites and relevant documentation. Sequence dependencies so an implementer can proceed without guessing. Use repository-permitted verification commands; a plan does not override a repository restriction on test work.

When several workers can implement independent slices, author `work-packages.md` using `azimuth reference show work-packages`. Give each package an objective, dependencies, exact non-overlapping owned paths and inspectable evidence. Reserve shared contracts, integration files, proposal state, current model facets, outcome and archival for the coordinator. Validate packages before delegation and use tool-generated briefs. Do not create packages merely to mirror workflow tasks.

Keep a single authoritative plan and update it during correction; do not add a mandatory correction-planning stage or duplicate implementation report. A rejected review returns its readable issues directly to bounded correction.

Retain the plan, validated package declaration when present, and reviewed inputs. Return for the real implementation authorization gate. Local work packages never authorize CI to generate assignments or dispatch agents.
