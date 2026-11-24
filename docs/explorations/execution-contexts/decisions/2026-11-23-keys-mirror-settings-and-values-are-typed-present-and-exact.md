# Decision: keys mirror settings, and values are typed, present and exact

Date: 2026-11-23. Decided by: the maintainer.

Claude proposed that a component's declaration names, for each key, the setting it controls.
The maintainer said: "Shouldn't we just mechanically mirror settings structure? With the
possibility to override". Claude found that a component may read a setting under another path
than the one declared, or may lack a setting and use a default in its code. Asked whether to
catch this with explicit presence and a run that flips each key, the maintainer asked: "How
would a flip control work technically?", "What about text or numeric values?", "Shouldn't we
just detect this during reviews?", and about an exact number in production: "Wouldn't this be
covered on the server that would mark the Check as inconclusive or something like that?".
Asked whether these answers should be one note, the maintainer said: "Yes"

Claude's words, which the maintainer agreed to: the key of a component's execution context is
the path of the setting the component reads, in its stack's own notation, for example
`Payouts:Instant` for an ASP.NET component, or `FEATURE_PAYOUTS` for a component that reads its
environment directly. A declaration may override how one key reaches the component, where the
stack's convention does not. Each key has a type: boolean, one of a declared list, or number.
Free text is not a key. The core compares normalized values of that type, and a Check states
exact values only. Every declared key is set explicitly: in a test run by the stated value or
the suite's default, and in a deployment by its configuration. A running component reports
each declared key as its configuration holds it, and reports a key it does not hold as absent.
A result that states an absent key does not apply. When a deployment's value differs from the
stated one, the server reports the result as "does not apply", and the fix is a new stated
value, a second Check, or a change to the deployment. The Verification Review compares each
key that a Check states with the path that the code reads. No run flips a key to prove that
the code reads it.

Rejected: verbatim strings, because `1000` and `1000.0` are one value to the component.
Conditions such as "at least 500", because they make the core interpret comparisons; they may
come later if such mismatches keep appearing. A run that flips each key, because it needs a run
per key and Challenges; it may come with them. A report of what the code does with its
settings, because it needs code in every component.

The cost: a key that the code reads under another path is caught only by a reading. A Check
that states `1000` stops applying at `1001`. Every deployment sets every declared key.

Example: the component `api` declares `Payouts:Instant`, a boolean, and `Payouts:Limit`, a
number. The Check `payout-is-sent-at-once` states `Payouts:Instant=true`. Production holds no
`Payouts:Instant`, so its report says absent and the result does not apply until production
sets it. Production then sets it to `True`. As booleans the two values are equal, and the
result applies.
