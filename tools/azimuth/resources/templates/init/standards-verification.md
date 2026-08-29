# Decision policies and Challenge schedule

Decision Policies name the open Challenge forms that must be searched before an Evidence Binding or Claim Judgment may be accepted. The separate schedule assigns every required form to one execution lane. Neither construct runs Challengers or defines provider commands.

Replace these starting declarations with the policies and lanes this project actually needs. Routine Claims use no policy, so a project that has not raised criticality may delete the Decision Policy block and keep one lane entry.

## Decision Policy: credible-executable
Required challenge: implementation-perturbation

Use for executable Check bindings whose credibility depends on the subject implementation.

## Challenge Schedule: current
Gate challenge: implementation-perturbation

This project searches implementation perturbation in the gate lane. Every form required by a Decision Policy, and every form declared by a current Challenger, occurs in exactly one lane.
