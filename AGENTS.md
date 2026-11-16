# Azimuth

Azimuth is being rewritten from its design. `docs/design.md` describes the target, and the code is built toward it in increments.

## Sources

- `docs/design.md` is the only source of what Azimuth does. A behavior that no sentence of it states is not built. Propose a change to the design first.
- The earlier implementation is in the git history. It is not a source. Do not read it for guidance and do not copy from it.
- An example in the design shows a rule and is never a source.

## Azimuth is built without Azimuth

Azimuth is developed as plain development with tests. It comes under Azimuth after the fourth increment of the core and before the server is built. Until then this repository has no Azimuth model, no markers and no change folders for its own code.

Four habits replace what Azimuth would enforce.

1. **A list of behaviors for each increment.** Before an increment is built, its behaviors are listed in a short document. Each line quotes the sentence of `docs/design.md` it comes from. A person reads the list before the build starts.
2. **Tests and code come from different sessions.** One session writes the tests from the list and the design. Another session writes the code.
3. **A sample repository.** Azimuth's tests include a small sample repository, and the CLI runs on it end to end.
4. **Done means used.** An increment is done when the real work it serves is proven with it, and not when its own tests pass.

The design has no rule ids. A plain test fails when a sentence quoted by a behavior is no longer in `docs/design.md`.

## Shape of the code

- The core is written in Rust and ships as one file with no runtime.
- A test adapter is a separate command in the language of its stack.
- The language of the server is decided when the server is built.
- Versions start at `0.2.0-alpha.1`.
- There is no backward-compatibility obligation during the alpha phase unless the design states one.
- Azimuth's own tests use the sample repository in this repository. They never depend on another project's vocabulary, paths or checkout.

## Writing and commits

Comments explain why; names describe purpose.

Commit subjects are imperative and scoped, for example `docs:` or `core:`. Bodies explain what changed, why it changed and any inconvenient findings surfaced by the work.
