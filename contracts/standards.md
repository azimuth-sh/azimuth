# Decision Standards format

One project file declares the Decision Policies that non-routine decisions cite and the single Challenge Schedule that assigns every required Challenge form to one execution lane. It defines no Checks, no provider commands and no execution results.

The file defaults to `azimuth/standards/verification.md` and is overridden with `--standards`. A project account has exactly one such file; a federated project catalog names it as one `{"repository": …, "path": …}` pair and pins its raw SHA-256 file digest in the repository manifest. The file is optional: a project whose Claims are all routine may omit it. When it is absent and any Claim is standard or critical, loading warns that non-routine decisions cannot be resolved.

```markdown
# Decision policies and Challenge schedule

Intro prose is ignored by the parser.

## Decision Policy: credible-executable
Required challenge: implementation-perturbation
Required challenge: oracle-perturbation

Use for executable Check bindings whose credibility depends on the implementation and the oracle.

## Challenge Schedule: current
Gate challenge: implementation-perturbation
Scheduled challenge: oracle-perturbation

Implementation perturbation is accepted in the gate lane; oracle perturbation stays scheduled.
```

## File shape

The first line-level requirement is the exact title `# Decision policies and Challenge schedule`. It may appear at any position, must appear once, and its absence is a parse error reported against the file rather than a line.

Only three headings are recognized: the title, `## Decision Policy: <id>` and `## Challenge Schedule: current`. A heading whose text begins with `## Semantics` is accepted and opens no block, so the lines under it carry no labels and no rationale; they are still scanned, and any further `#` line among them is an error. Every line whose trimmed text begins with `#` and is none of the recognized headings is a parse error, reported as `unrecognized Decision Standards heading`. The rule is general rather than a list of rejected spellings: the retired `## Policy:` and `## Qualification Policy:` headings fail under it exactly as any other unrecognized heading does, and so does a `#` line inside a fenced block that sits outside any Decision Policy or Challenge Schedule block. Headings are matched after trimming, and the id is the trimmed remainder after the `:`.

Each recognized block is a label region followed by prose. Labels start directly under the heading and end at the first blank line or the next heading. A line in the label region that begins no known label continues the previous label's value, joined with a single space; a line that begins no known label with nothing to continue is rejected as an unrecognized line. Once at least one known label has appeared, a continuation line that looks like a label — an alphabetic, space and hyphen key before a `:` — is rejected as an unrecognized label-like line rather than silently joined.

Prose after the blank line, up to the next heading, is required review rationale. Fenced blocks and `>` quoted lines are excluded from it. Rationale never enters any digest.

## Decision Policy

```markdown
## Decision Policy: <policy-id>
Required challenge: <challenge-form-id>
```

A Decision Policy id and a Challenge form id are project-global lower kebab path ids: lowercase ASCII letters, digits and `-`, `/` permitted as a namespace separator, no empty segment and no leading or trailing `-` in a segment. Ids are declared, never derived from a path.

`Required challenge:` is the only recognized label and is repeatable. A policy needs at least one; repeating the same form is an error, and a `Required challenge:` line with an empty value is an invalid form id. Line order is not semantic — the parser sorts the forms. Two Decision Policy blocks with the same id are an error. Rationale prose is required.

An Evidence Binding's or Claim Judgment's `Policy:` must name a declared policy; a binding naming an unknown policy is the `binding-missing-policy` Finding, not a parse error.

## Challenge Schedule

```markdown
## Challenge Schedule: current
Gate challenge: <challenge-form-id>
Scheduled challenge: <challenge-form-id>
```

The only accepted id is `current`; any other id is an error. Exactly one schedule block exists — zero is an error reported against the file, and a second declaration is an error.

`Gate challenge:` and `Scheduled challenge:` are the only recognized labels and are both repeatable. Each lane accepts zero or more distinct forms, so a project may be gate-only or scheduled-only, but the union of the two lanes must be non-empty. Lines with an empty value are ignored. Repeating a form within one lane is an error, and a form present in both lanes is an error, so a listed form occupies exactly one lane. Line order is not semantic — the parser sorts each lane. Rationale prose is required.

## Coverage

Coverage is checked in two places.

At parse time, every form required by any Decision Policy in the file must occur in some lane; a required form with no lane is a parse error reported at the schedule's line.

Over the complete unselected project account, a scheduled form with no Decision Policy and no current Challenger is an error against the schedule, and a current Challenger whose form has no lane is an error against that Challenger. Both are load errors, not Findings. Together with lane exclusivity: every form required by any policy occurs exactly once, every form declared by a current Challenger occurs exactly once, and no additional form is valid.

## Digests

Both digests use the repository `canonical_sha256` serialization defined by the [verification format](verification.md) — object keys sorted recursively by their exact strings, two-space expanded serialization, LF, exactly one terminal LF hashed — not the RFC 8785 serialization used by the [Run-bundle format](run-bundle.md). Values are `sha256:` followed by 64 lowercase hex digits.

A Decision Policy digest is taken over exactly:

```json
{
  "format": "azimuth-decision-policy-digest",
  "version": 1,
  "id": <policy-id>,
  "required_challenges": <sorted-distinct-forms>
}
```

The Challenge Schedule digest is taken over exactly:

```json
{
  "format": "azimuth-challenge-schedule-digest",
  "version": 1,
  "id": "current",
  "gate_challenges": <sorted-distinct-gate-forms>,
  "scheduled_challenges": <sorted-distinct-scheduled-forms>
}
```

The literal `"current"` is written into the schedule preimage; the parsed id does not vary.

The Decision Policy digest participates in every Evidence Binding fingerprint as `decision_policy_digest`, and therefore in the Qualification fingerprint derived from it; it participates in the Claim Judgment preimage as `policy_digest`; and it appears as a `policy` scope component in Qualification and Claim Judgment semantic scope. Editing a policy's required forms stales only the decisions that cite that policy.

The Challenge Schedule digest participates in no Check, binding, context, Qualification, Challenger or Claim Judgment identity. It is published as the `digest` field of the exported `challenge_schedule` object and therefore participates in the complete-model digest that planning and finalization use. The lane itself is projected as `gate | scheduled` into every semantic Challenge selection: planning resolves the requested Challenger's form against the two lanes and fails when the form does not occupy exactly one.

## Parse errors

A parse failure names the file, the line and what was expected, and no partial standards are returned. The complete set of parse errors is:

- the title is missing, or declared twice;
- an unrecognized heading, which includes `## Policy:`, `## Qualification Policy:` and any other `#` line outside a recognized block;
- an invalid Decision Policy id, an invalid Challenge form id, or an empty form value on a `Required challenge:` line;
- a Decision Policy with no `Required challenge:` line, or with a repeated required form;
- a Decision Policy id declared twice;
- a Challenge Schedule id other than `current`, no schedule, or a second schedule;
- a schedule with no form in either lane, a form repeated within a lane, or a form in both lanes;
- a repeated non-repeatable label, an unrecognized line, or an unrecognized label-like line inside a block;
- a Decision Policy or Challenge Schedule with no rationale prose;
- a policy-required form with no scheduling lane.

## Out of scope

The lane controls selection expectation, not decision meaning: a clean Challenge Result is only a negative search fact and qualifies nothing by itself. This format defines no cache, cadence, TTL, historical applicability, cross-Subject reuse or time-based deferral, and no schedule construct runs a Challenger or names a provider command.
