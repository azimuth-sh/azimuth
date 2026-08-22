# Decision policies and Challenge schedule

Project Decision Policies name the open Challenge forms required before an Evidence Binding or
Claim Judgment may be accepted. The separate schedule assigns every required form to one execution
lane. Neither construct runs Challengers or defines provider commands.

The grammar, coverage rules and digest preimages of this file are the
[Decision Standards format](../../contracts/standards.md). This file holds only what this project
declares.

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

The canonical preimage of this schedule is:

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

All current framework Claims are routine and therefore use no policy. This file establishes the
project vocabulary for synthetic format and tool tests without creating current evidence.
