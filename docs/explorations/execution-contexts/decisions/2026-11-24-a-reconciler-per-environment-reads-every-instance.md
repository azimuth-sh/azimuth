# Decision: a reconciler for each environment reads every instance

Date: 2026-11-24. Decided by: the maintainer.

About finding the running instances of components, the maintainer said: "Finding components looks
like a classic service discovery problem", and then asked: "Do we need such an interface?
Shouldn't we just bake it into adapters and future reconcilers (one for each common
environment)?". About environments other than Kubernetes or Docker, the maintainer had asked:
"Should we have some declaration mechanism? Should it live in specific area?". Asked whether a
reconciler should exist for each common environment, finding instances by that environment's own
marks, with Azimuth fixing only the observation and what a reconciler must prove, the maintainer
said: "Yes"

Claude's words, which the maintainer agreed to: in a deployment, the reports of components are
read by a reconciler of that environment, which runs where the instances can be reached and only
sends what it observed. The server never calls into the environment. Each common environment has
its own reconciler, with its own way of finding instances, and Kubernetes comes first. Azimuth has
no interface for discovery. A reconciler learns which component a workload is from that
environment's own mark, such as a label on a Kubernetes workload. The mark is written in the
deployment's configuration and belongs to whichever area holds that configuration, and no area is
special for it. Azimuth fixes two things. The first is the observation that a reconciler sends:
the deployment, and for each ready instance the component its mark names, the environment's
identity of the instance, its image digest, and its report with the time read. The second is what
a reconciler must prove: it read every ready instance, each at its own address and not through an
address that spreads requests, and the set of ready instances stayed the same across its reads.
An observation that cannot prove this is not sent. A team whose environment has no reconciler
writes one against that observation.

This replaces what the notes say about the server reading the report of a component in a
deployment: the reconciler reads, and the server receives.

Rejected: an interface for discovery shared by adapters and reconcilers, because a reconciler
already belongs to one environment, and a launch is told its addresses by the suite. An Azimuth
file per environment that lists how to find each component, because it would repeat what the
deployment's configuration says and drift when instances are added. Instances that send their
reports themselves, because the library would then open a connection out of production code.

The cost: each new environment is a whole reconciler. An environment whose instances cannot be
reached, such as functions that start per request, cannot be observed this way.

Example: the reconciler for Kubernetes lists the Deployment labelled `azimuth.sh/component: api`
and its three ready Pods. It reads `/internal/azimuth` at each Pod's address, lists the Pods
again, finds the same three, and sends one observation with the three reports under `api`.
