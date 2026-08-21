# Decision policies and Challenge schedule

Project Decision Policies name the open Challenge forms required before an Evidence Binding or
Claim Judgment may be accepted. The separate schedule assigns every required form to one execution
lane. Neither construct runs Challengers or defines provider commands.

## Decision Policy: credible-executable
Required challenge: implementation-perturbation
Required challenge: oracle-perturbation

Use for executable Check bindings whose credibility depends on both the subject implementation and
the discriminating oracle.

## Challenge Schedule: current
Gate challenge: implementation-perturbation
Scheduled challenge: oracle-perturbation

The current project accepts implementation perturbation in the gate lane and permits the more
expensive oracle perturbation to remain visibly scheduled.

## Semantics

Policy ids and Challenge-form ids are project-global lower kebab path ids. A Decision Policy has
one or more distinct `Required challenge:` lines. Their order is not semantic. `## Policy:` and
`## Qualification Policy:` headings are rejected rather than aliased.

Exactly one `## Challenge Schedule: current` block exists. Each lane accepts zero or more distinct
lines, so a project may be gate-only or scheduled-only; the union must be non-empty. A form occurs
in exactly one lane, and every form required by any Decision Policy occurs exactly once. Every form
declared by a current Challenger must also occur exactly once; additional forms in either lane are
invalid.

The resolved Decision Policy digest participates in every dependent Evidence Binding and Claim
Judgment fingerprint. Editing required forms therefore stales only decisions using that policy.
The schedule has its own digest, participates in the complete-model fingerprint and is projected as
`gate | scheduled` into every semantic Challenge selection. It does not participate in Check,
binding, context, Qualification, Challenger or Claim Judgment identity.

The schedule digest uses the D45 repository `canonical_sha256` serialization defined by the
[verification format](../formats/verification.md), not D46 RFC 8785, over exactly:

```json
{
  "format": "azimuth-challenge-schedule-digest",
  "version": 1,
  "id": "current",
  "gate_challenges": <sorted-distinct-gate-forms>,
  "scheduled_challenges": <sorted-distinct-scheduled-forms>
}
```

The current canonical preimage is:

```json
{
  "format": "azimuth-challenge-schedule-digest",
  "gate_challenges": [
    "implementation-perturbation"
  ],
  "id": "current",
  "scheduled_challenges": [
    "oracle-perturbation"
  ],
  "version": 1
}
```

The serialized preimage has one terminal LF. Its SHA-256 value is
`sha256:ce320ac98fed500eff1ef1032817884ca0d7dba4c2160fa22641ed0c8b058ae1`.

A clean Challenge Result means only that the selected Challenger found no objection. It never
qualifies an edge or accepts a Claim Judgment by itself. The lane controls selection expectation,
not decision meaning. Alpha 2 defines no cache, cadence, TTL, historical applicability,
cross-Subject reuse or time-based deferral.

All current framework Claims are routine and therefore use no policy. This file establishes the
project vocabulary for synthetic format and tool tests without creating current evidence.
