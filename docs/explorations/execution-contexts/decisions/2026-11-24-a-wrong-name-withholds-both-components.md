# Decision: a wrong name withholds the results of both components

Date: 2026-11-24. Decided by: the maintainer.

Asked whether an instance whose report gives a name other than its mark should withhold the
results of the marked component, of both components, or of the whole deployment, the maintainer
said: "Either component"

Claude's words, which the maintainer agreed to: when an instance in a deployment reports a name
other than the component its mark gives, no result that depends on a key of the marked component
or of the named component applies there, until a later observation finds every name agreeing
with its mark. The instance stays in the observation with both names, as an instance that cannot
be read, and the server shows its finding.

This replaces what the note `an-instance-that-cannot-be-read-counts-as-a-disagreement` says about
withholding the results of "that component" when the cause is a name other than its mark. For
the other causes, an instance that does not answer in time, answers with no report or gives a
report that does not follow its format, it withholds the results of its own component only.

Rejected: withholding the marked component's results only, because the named component then has
an instance serving users that was read under another name. Withholding every result of the
deployment, because results with no key of either component stay shown.

The cost: one wrong mark takes away the results of two components for as long as it stays.

Example: `api` maps `payouts.instant` and `payouts.limit`, and `website` maps `payouts.region`.
The workload of `website` is marked `api` by mistake. The reconciler reads the Pod
`website-5c8d-q7mn` under `api`, and its report says `"component": "website"`. The server shows
"Pod website-5c8d-q7mn: marked api, reports website", and the results that depend on any of the
three keys do not apply until the mark is fixed.
