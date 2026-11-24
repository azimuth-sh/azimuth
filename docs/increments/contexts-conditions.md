# Execution contexts, conditions in the model: behaviors

A row is of one of two kinds. A row of behavior is one observable fact, and a test shows it. A row of prose says what a text tells its reader; its section begins with the words "Rows of prose", and no test reads the words of the text for it. The second cell is copied exactly from `docs/design.md`. A plain test fails when a quoted sentence is no longer in that file. A behavior with no sentence to quote is not built. This list is built after `contexts-keys.md`, because a condition's type comes from its key's declaration.

Status of this list: read and accepted by a person on 2026-11-24.

## Reading `Context:`

| Behavior | Quote from the design |
| --- | --- |
| The reader accepts `Context:` under a Requirement in `specification.md` | under a Requirement or Scenario in `specification.md` |
| The reader accepts `Context:` under a Scenario in `specification.md` | under a Requirement or Scenario in `specification.md` |
| A `Context:` under a Check in `verification.md` is a reading error | **A declaration that stands where it has no meaning is a reading error.** |
| A `Context:` under a Requirement or Scenario in `verification.md` is a reading error | **A declaration that stands where it has no meaning is a reading error.** |
| A `Context:` above the first Requirement stays prose | Text above the first Requirement of a file stays prose, whatever it looks like. |
| A `Context:` line states one key and one exact value as `Context: <key>=<value>` | **One `Context:` line states one key and one exact value, as `Context: <key>=<value>`.** |
| Spaces and tabs around `=` change nothing | Spaces and tabs around `=` are ignored. |
| Several keys are read from several lines under the same heading | Several keys use several lines under the same heading in `specification.md`. |
| Spaces and `=` inside a one-of value's JSON string are part of its value | A one-of value is a JSON string, so spaces and `=` inside its quotes are values, not separators. |
| Boolean values are unquoted | Booleans and numbers are unquoted. |
| Number values are unquoted | Booleans and numbers are unquoted. |

## The effective condition

| Behavior | Quote from the design |
| --- | --- |
| A Scenario takes its Requirement's condition | **A Scenario takes its Requirement's condition and may add its own.** |
| A Scenario may add a key to its Requirement's condition | **A Scenario takes its Requirement's condition and may add its own.** |
| A Scenario with no `Context:` of its own keeps its Requirement's condition | A Scenario with no `Context:` of its own keeps its Requirement's condition. |
| Repeating an inherited value changes nothing | Repeating the same value changes nothing. |
| Repeating a key with the same normalized value on one heading changes nothing | A repeated key with the same normalized value changes nothing. |
| A Scenario with no condition of its own or its Requirement's has no condition | A Requirement with none, and a Scenario of it with none, are promised everywhere. |

## Typed values and normalization

| Behavior | Quote from the design |
| --- | --- |
| Boolean text is accepted without regard to ASCII case | Boolean text is `true` or `false`, without regard to ASCII case, and normalizes to lowercase. |
| Boolean text normalizes to lowercase | Boolean text is `true` or `false`, without regard to ASCII case, and normalizes to lowercase. |
| A one-of value is decoded from its JSON string | A one-of value is decoded from its JSON string, normalized to NFC and compared with the declared strings in NFC, with no case folding. |
| A one-of value compares equal to its declared string in another Unicode form after NFC normalization | A one-of value is decoded from its JSON string, normalized to NFC and compared with the declared strings in NFC, with no case folding. |
| A one-of value with another case is not folded to a declared string | A one-of value is decoded from its JSON string, normalized to NFC and compared with the declared strings in NFC, with no case folding. |
| A number is read in JSON decimal notation | A number uses the JSON decimal notation and is compared as an exact decimal, not a floating-point approximation. |
| Numbers compare as exact decimals | A number uses the JSON decimal notation and is compared as an exact decimal, not a floating-point approximation. |
| A normalized number has no exponent | Its normalized text has no exponent, no unnecessary leading or fractional trailing zeros, and no decimal point for a whole number; negative zero is `0`. |
| A normalized number has no unnecessary leading zeros | Its normalized text has no exponent, no unnecessary leading or fractional trailing zeros, and no decimal point for a whole number; negative zero is `0`. |
| A normalized number has no fractional trailing zeros | Its normalized text has no exponent, no unnecessary leading or fractional trailing zeros, and no decimal point for a whole number; negative zero is `0`. |
| A normalized whole number has no decimal point | Its normalized text has no exponent, no unnecessary leading or fractional trailing zeros, and no decimal point for a whole number; negative zero is `0`. |
| A range is not accepted as a condition value | No condition states a range. |
| Negative zero normalizes to `0` | Its normalized text has no exponent, no unnecessary leading or fractional trailing zeros, and no decimal point for a whole number; negative zero is `0`. |

## Reading errors in conditions

| Behavior | Quote from the design |
| --- | --- |
| A `Context:` without a `key=value` assignment is a reading error | `Context:` without exactly one `key=value` assignment, an invalid key spelling, an unfinished JSON string or malformed number is a reading error. |
| A `Context:` with more than one assignment is a reading error | `Context:` without exactly one `key=value` assignment, an invalid key spelling, an unfinished JSON string or malformed number is a reading error. |
| A `Context:` with an invalid key spelling is a reading error | `Context:` without exactly one `key=value` assignment, an invalid key spelling, an unfinished JSON string or malformed number is a reading error. |
| A `Context:` with an unfinished JSON string is a reading error | `Context:` without exactly one `key=value` assignment, an invalid key spelling, an unfinished JSON string or malformed number is a reading error. |
| A `Context:` with a malformed number is a reading error | `Context:` without exactly one `key=value` assignment, an invalid key spelling, an unfinished JSON string or malformed number is a reading error. |
| A `Context:` number that would need more than 1000 added zeros without an exponent is a reading error | A number that would need more than 1000 added zeros to be written without an exponent is a reading error. |

## What commands do on a reading error

| Behavior | Quote from the design |
| --- | --- |
| A condition reading error has the same words in every command | Every command reads the model through the same reader, so a reading error has the same words everywhere. |
| On a condition reading error, `azimuth run`, `azimuth check run` and the reading commands do nothing else | print the same message, do nothing else, end with failure |
| On a condition reading error, commands end with status 2 | a reading error, a wrong argument, `azimuth run` under a CLI version other than the pinned one, a missing common ancestor |
| A misplaced `Context:` message names the file | The message names the file and the place where the line belongs. |
| A misplaced `Context:` message names the place where the line belongs | The message names the file and the place where the line belongs. |

## Lint findings

| Behavior | Quote from the design |
| --- | --- |
| Lint reports a `Context:` that names an undeclared key | An unknown key, a well-formed value of the wrong declared type, a one-of value not listed, and contradicting assignments are findings. |
| Lint reports a well-formed value of the wrong declared type | An unknown key, a well-formed value of the wrong declared type, a one-of value not listed, and contradicting assignments are findings. |
| Lint reports a one-of value not in the declared list | An unknown key, a well-formed value of the wrong declared type, a one-of value not listed, and contradicting assignments are findings. |
| Lint reports contradicting `Context:` lines on one heading | Contradicting lines on one heading are a lint finding, and the affected Checks give no result, "context not applied", until the contradiction is gone. |
| Lint reports a Scenario that gives an inherited key another value | Giving an inherited key another value is a lint finding. |
| A condition finding fails lint | A finding fails only lint and the commit lane. |
| A condition finding fails the commit lane | A finding fails only lint and the commit lane. |
| A condition finding does not stop the reading commands | work. The affected Check is shown with the state it has |

## The claim digest

| Behavior | Quote from the design |
| --- | --- |
| The claim part is the `text-1` digest of the claim text followed by its effective condition | **The claim part is the `text-1` digest of the claim text followed by its effective condition.** |
| The claim text is normalized as model text before the condition is appended | Normalize the claim text as model text. |
| A nonempty condition is appended after LF as one `Context: <key>=<normalized value>` line per key, sorted by key, with no last LF | When the condition is nonempty, append LF and then one line `Context: <key>=<normalized value>` per key, sorted by key, with no last LF. |
| One-of values in the claim part use the JSON string spelling of RFC 8785 | One-of values use the JSON string spelling of RFC 8785. |
| Inherited and repeated lines contribute each key once to the claim part | Inherited and repeated lines contribute each key once. |
| A Check with no condition keeps the digest of its claim text alone | A Check with no condition keeps the digest of its claim text alone. |
| Changing a condition makes the Check owed with "claim changed" | A changed condition makes the Check owed with "claim changed", also when a provider has removed the specification file from its files. |
| The fingerprint gains no extra context part | No extra fingerprint part holds context; reported values of unstated keys are not fingerprint inputs of their own. |

## Checks whose condition cannot be applied

| Behavior | Quote from the design |
| --- | --- |
| `azimuth run` leaves a Check with a condition owed with "context not applied" when no started component can apply it | A condition that no started component can apply gives "context not applied" in both `azimuth run` and `azimuth check run`, also in a suite with Checks that have no condition. |
| `azimuth check run` leaves a Check with a condition owed with "context not applied" when no started component can apply it | A condition that no started component can apply gives "context not applied" in both `azimuth run` and `azimuth check run`, also in a suite with Checks that have no condition. |
| `azimuth run` leaves such a Check owed also in a suite with Checks that have no condition | A condition that no started component can apply gives "context not applied" in both `azimuth run` and `azimuth check run`, also in a suite with Checks that have no condition. |
| `azimuth check run` leaves such a Check owed also in a suite with Checks that have no condition | A condition that no started component can apply gives "context not applied" in both `azimuth run` and `azimuth check run`, also in a suite with Checks that have no condition. |
| `azimuth run` leaves Checks with a faulty readable condition owed with "context not applied" | Both running commands leave affected Checks owed with "context not applied" until their condition can be applied. |
| `azimuth check run` leaves Checks with a faulty readable condition owed with "context not applied" | Both running commands leave affected Checks owed with "context not applied" until their condition can be applied. |
| A Check whose condition was not applied has no result, whatever its tests did | If it did not, the Check has no result, with the reason "context not applied", whatever its tests did. |
| `azimuth run` can still run Checks without the faulty condition in the same suite | Checks without the faulty condition can still run. |
| `azimuth check run` can still run a named Check without the faulty condition in the same suite | Checks without the faulty condition can still run. |
| The local lane ends with status 1 for "context not applied" | The local lane ends with status 1, as for a crash. |
