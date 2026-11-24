# Decision: an instance that cannot be read counts as a disagreement

Date: 2026-11-24. Decided by: the maintainer.

Asked whether a ready replica whose report cannot be read should count like a disagreement, with a
finding naming that instance and no result that depends on its component's keys applying until it
is read again, the maintainer said: "Yes"

Claude's words, which the maintainer agreed to: a ready instance whose report cannot be read,
because it does not answer in time, answers with no report, gives a report that does not follow
its format, or gives a name other than its mark, stays in the observation with the cause. The
server shows a finding that names the instance, its component and the cause, and no result that
depends on a key of that component applies in the deployment until a later observation reads the
instance. A reconciler tries every ready instance. An observation is not sent only when the set of
ready instances changed across its reads.

This replaces what the note `a-reconciler-per-environment-reads-every-instance` says about an
observation that cannot prove it read every ready instance: an instance that was tried and could
not be read is part of the observation.

Rejected: leaving out an instance that cannot be read, because it still serves users, and one
replica with an unknown value would then leave the status saying that the results apply.

The cost: one replica that is slow to answer takes away every result that depends on a key of its
component, for as long as it stays slow.

Example: `api` has three ready Pods. Two give their reports, and `api-7d9f-xk2p` does not answer
within the reconciler's time. The observation holds the two reports and that Pod with the cause
"no answer". The server shows the finding "Pod api-7d9f-xk2p of api: no answer", and the results
that depend on `payouts.instant` or `payouts.region` do not apply until the Pod is read again.
