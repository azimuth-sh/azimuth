# Decision: keys are declared in execution-context.toml

Date: 2026-11-23. Decided by: the maintainer.

Asked where the keys, the components and the components each suite starts should be declared,
the maintainer asked: "What about context.yaml(.yml?) like values.yaml in Helm?". Shown the same
nested form in TOML in a file of its own, the maintainer asked: "context.toml or
execution-context.toml?", and to the second with `Context:` and `AZIMUTH_CONTEXT` keeping their
short names, said: "Yes"

Claude's words, which the maintainer agreed to: the keys of the execution context are declared in
`azimuth/execution-context.toml`, a TOML file of its own beside the project file. Each key is a
table named by its dotted name. It holds `type`, which is `boolean`, `one of` with its `values`,
or `number`, and `components`, the components that must read it. No key is the beginning of
another key. The components, each with the path of its report, and under each suite the
components it starts, as `starts`, are declared in `azimuth/project.toml`, which holds the
topology. A component named in `execution-context.toml` that `project.toml` does not declare is a
lint finding. Every change to `execution-context.toml` is class D, like a change to the project
file. The line on a Requirement or a Scenario stays `Context:`, and the variable stays
`AZIMUTH_CONTEXT`.

This replaces what the note `a-key-belongs-to-the-product` says about the project file declaring
each key: the keys are declared in `execution-context.toml`.

Rejected: YAML, because, as the design says, it "changes the type of an unquoted value silently".
Under YAML 1.1 rules the value `no` becomes `false` and `1.10` becomes `1.1`, which breaks exact
values. Also, unlike `values.yaml` in Helm, this file holds declarations and no values. The keys
in `project.toml`, because many keys would crowd the topology. The keys in the model, because the
model's form is made for prose, and a key shared by two modules would have no single home. The
name `context.toml`, because the agents that work in a repository have a context of their own,
and the file could be taken for guidance meant for them.

The cost: one fact spans two files, held together only by lint. The nested form forbids a key
that begins another key.

Example:

```toml
[payouts.instant]
type = "boolean"
components = ["api", "website"]

[payouts.region]
type = "one of"
values = ["no", "se"]
components = ["api"]
```

`project.toml` declares `api` and `website`, and the suite `website-tests` declares
`starts = ["website"]`.
