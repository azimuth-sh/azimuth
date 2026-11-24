# Execution contexts, keys in the model: behaviors

A row is of one of two kinds. A row of behavior is one observable fact, and a test shows it. A row of prose says what a text tells its reader; its section begins with the words "Rows of prose", and no test reads the words of the text for it. The second cell is copied exactly from `docs/design.md`. A plain test fails when a quoted sentence is no longer in that file. A behavior with no sentence to quote is not built.

Status of this list: a draft, not yet read by a person.

## Key declarations in `azimuth/execution-context.toml`

| Behavior | Quote from the design |
| --- | --- |
| The reader reads key declarations from `azimuth/execution-context.toml`, beside the project file | `azimuth/execution-context.toml`, beside the project file |
| Each key is a table at its dotted name | Each key is a table at its dotted name. |
| A key name is lowercase segments joined by dots | Its name is lowercase segments joined by dots. |
| A segment starts with an ASCII lowercase letter and continues with ASCII lowercase letters, digits or hyphens | A segment starts with an ASCII lowercase letter and continues with ASCII lowercase letters, digits or hyphens. |
| The reader reads no component from a key's name | Azimuth reads no component in the name. |
| A key's `type` may be `boolean` | `boolean`, `one of`, or `number`. Free text is not a key |
| A key's `type` may be `one of` | `boolean`, `one of`, or `number`. Free text is not a key |
| A key's `type` may be `number` | `boolean`, `one of`, or `number`. Free text is not a key |
| A one-of key's `values` declares its allowed strings | for `one of` only, a nonempty list of distinct strings; a condition must name one of them |
| A key's `components` lists the components that must read it | a nonempty list of the components that must read this key |
| An absent `execution-context.toml` declares no keys | An absent file and an empty file both declare no keys. |
| An empty `execution-context.toml` declares no keys | An absent file and an empty file both declare no keys. |
| `azimuth init` writes no `execution-context.toml` | `azimuth init` writes no `execution-context.toml` and no components, and leaves `starts` omitted. |

## Reading errors in key declarations

| Behavior | Quote from the design |
| --- | --- |
| Malformed TOML in `execution-context.toml` is a reading error | malformed TOML, an unknown field, an invalid dotted key, a missing `type` or `components`, an unknown type, `values` missing for `one of` or present for another type, an empty one-of list or repeated value after NFC normalization, or a field of the wrong TOML type. |
| An unknown field in a key table is a reading error | malformed TOML, an unknown field, an invalid dotted key, a missing `type` or `components`, an unknown type, `values` missing for `one of` or present for another type, an empty one-of list or repeated value after NFC normalization, or a field of the wrong TOML type. |
| An invalid dotted key is a reading error | malformed TOML, an unknown field, an invalid dotted key, a missing `type` or `components`, an unknown type, `values` missing for `one of` or present for another type, an empty one-of list or repeated value after NFC normalization, or a field of the wrong TOML type. |
| A key table with no `type` is a reading error | malformed TOML, an unknown field, an invalid dotted key, a missing `type` or `components`, an unknown type, `values` missing for `one of` or present for another type, an empty one-of list or repeated value after NFC normalization, or a field of the wrong TOML type. |
| A key table with no `components` is a reading error | malformed TOML, an unknown field, an invalid dotted key, a missing `type` or `components`, an unknown type, `values` missing for `one of` or present for another type, an empty one-of list or repeated value after NFC normalization, or a field of the wrong TOML type. |
| An unknown `type` is a reading error | malformed TOML, an unknown field, an invalid dotted key, a missing `type` or `components`, an unknown type, `values` missing for `one of` or present for another type, an empty one-of list or repeated value after NFC normalization, or a field of the wrong TOML type. |
| A one-of key with no `values` is a reading error | malformed TOML, an unknown field, an invalid dotted key, a missing `type` or `components`, an unknown type, `values` missing for `one of` or present for another type, an empty one-of list or repeated value after NFC normalization, or a field of the wrong TOML type. |
| A boolean or number key with `values` is a reading error | malformed TOML, an unknown field, an invalid dotted key, a missing `type` or `components`, an unknown type, `values` missing for `one of` or present for another type, an empty one-of list or repeated value after NFC normalization, or a field of the wrong TOML type. |
| An empty one-of list is a reading error | malformed TOML, an unknown field, an invalid dotted key, a missing `type` or `components`, an unknown type, `values` missing for `one of` or present for another type, an empty one-of list or repeated value after NFC normalization, or a field of the wrong TOML type. |
| A one-of list with a repeated value after NFC normalization is a reading error | malformed TOML, an unknown field, an invalid dotted key, a missing `type` or `components`, an unknown type, `values` missing for `one of` or present for another type, an empty one-of list or repeated value after NFC normalization, or a field of the wrong TOML type. |
| A field of the wrong TOML type is a reading error | malformed TOML, an unknown field, an invalid dotted key, a missing `type` or `components`, an unknown type, `values` missing for `one of` or present for another type, an empty one-of list or repeated value after NFC normalization, or a field of the wrong TOML type. |
| An intermediate table holding anything but nested key tables is a reading error | Intermediate tables hold only nested key tables |

## What commands do on a reading error

| Behavior | Quote from the design |
| --- | --- |
| A key-declaration reading error has the same words in every command | Every command reads the model through the same reader, so a reading error has the same words everywhere. |
| On a key-declaration reading error, `azimuth run`, `azimuth check run` and the reading commands do nothing else | print the same message, do nothing else, end with failure |
| On a key-declaration reading error, commands end with status 2 | a reading error, a wrong argument, `azimuth run` under a CLI version other than the pinned one, a missing common ancestor |

## Lint findings

| Behavior | Quote from the design |
| --- | --- |
| Lint reports a key that begins another at a segment boundary | A key that begins another at a segment boundary, or a key listed for no component |
| Lint reports a key whose `components` list is empty | A key that begins another at a segment boundary, or a key listed for no component |
| A key-declaration finding fails lint | A finding fails only lint and the commit lane. |
| A key-declaration finding fails the commit lane | A finding fails only lint and the commit lane. |
| A key-declaration finding does not stop the reading commands | work. The affected Check is shown with the state it has |

## `azimuth change diff`

| Behavior | Quote from the design |
| --- | --- |
| A change to `azimuth/execution-context.toml` is class D | **Every change to `execution-context.toml` is class D, as is a change to `starts` in the project file.** |
| The reason names `azimuth/execution-context.toml` | `azimuth change diff` names the file as the reason. |
