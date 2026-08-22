# Exploration: Repository layout

Id: repository-layout
Status: active

## Objective

Establish which directories in this repository hold Azimuth-the-product and which hold this
project's own instance of it, and whether the paths tell a reader the truth. Record the public
contract inventory that the separation exposed.

## Boundaries

- Placement and naming only. What each contract says is the concern of the contracts themselves.
- Derive from artifacts and from what the tool scaffolds, not from prose that describes intent.
- Do not rename anything whose name is consumed by a third party without recording the cost.
- Do not write or run tests.

## Existing context

- `tools/azimuth/src/workflow.rs:58-85` defines the instance shape: `azimuth init` creates
  `model/`, `changes/archive/`, `explorations/archive/` and `standards/`, then writes
  `standards/verification.md` and `workspace.json`. Nothing else.
- `release/acceptance.py:35` names `repository-contracts` as one of five supported surfaces, and
  `README.md` states that formats, standards, skills and documentation are supported as repository
  artifacts at a Git tag.
- `release/artifacts.json` lists `experiments` and five packages and extractors under
  `experimentalSource`; the contracts appear in no catalog path.

## Findings

### F1 — `azimuth/` conflated product with instance

Everything under `azimuth/` was per-project instance data except the format contracts, which are
the universal specification every consumer's instance must satisfy. A consumer adopting Azimuth
authors `model/`, `changes/`, `standards/` and `workspace.json`; they never author a contract.

The tool settles it: `azimuth init` scaffolds four directories and two files, and the contracts are
not among them. They have been moved to a top-level `contracts/`.

### F2 — The contracts are a published surface, not internal documentation

They are named in `supportedSurfaces`, consumed by pinning a Git tag, and implemented against by
third parties who never see this repository's model. That argues against folding them into `docs/`,
which was the first placement considered: a specification an outside implementer works from should
not be filed with the derived prose that describes it.

### F3 — Seven documents covered about eleven of roughly twenty-one contracts

Files and contracts are not one to one. `adapter.md` alone specifies four wire contracts and
`run-launch-plan.md` two. Against the forty-seven `azimuth-*` identifiers in core, ten public
contracts had no document at all: the repository manifest, five federation formats, the run
inspection output, the export schema, the standards grammar, and the Finding registry. All ten
have since been written, and the two absent ecosystem site profiles — .NET and JVM — were added.

The gap fell entirely on the two oldest third-party surfaces, extraction and federation. The
adapter protocol, designed most recently, was the only one fully specified.

### F4 — Three different things are named `verification.md`

| Path | Holds |
|---|---|
| `contracts/verification.md` | the format contract for the evidence facet |
| `azimuth/model/<spec>/verification.md` | one spec package's evidence facet; none currently exist |
| `azimuth/standards/verification.md` | this project's Decision Policies and Challenge Schedule |

The third is the misnomer: its content is policies and a schedule, neither of which is
verification. The name presumably contracts "standards for the verification facet". Renaming it
would change what `azimuth init` writes, so it is a contract change for every consumer rather than
a rename.

### F5 — A live instance artifact was specifying its own format

`azimuth/standards/verification.md` carried a `## Semantics` section describing the grammar that
governs it, with nothing checking the description against the parser. One claim in it was wrong:
the retired `## Policy:` and `## Qualification Policy:` headings were described as "rejected rather
than aliased", implying a named check, where `tools/azimuth/src/verification.rs:1006` emits a single
generic `unrecognized Decision Standards heading` diagnostic for any unknown heading.

The specification now lives in `contracts/standards.md` and the instance file holds only its
declarations. The parser's `## Semantics` exception, which is what allowed an unchecked region to
exist inside a parsed artifact, has no remaining user.

### F6 — One published identity still carries the consumer's namespace

Writing `contracts/markers.md` forced the seven annotation package identities to be stated side by
side for the first time, and six are neutral while one is not:

| Ecosystem | Identity |
|---|---|
| TypeScript | `@azimuth-sh/annotations` |
| Go | `azimuth-sh/azimuth-go/azimuth` |
| Rust | `azimuth-annotations` |
| .NET | `Azimuth.Annotations` |
| C++ | `azimuth.hpp` |
| Python | `azimuth_annotations` |
| JVM | `dev.drim.azimuth` → `sh.azimuth` (renamed) |

The JVM annotations declared the consumer's reverse domain as the published Java namespace.
`AGENTS.md` requires that tooling never depend on consumer vocabulary, and a package namespace is
the most public identity an ecosystem has.

The canonical product domain is `azimuth.sh` (`README.md:12`), so the conventional target is
`sh.azimuth`, with the extractor at `sh.azimuth.emit`. The blast radius is eleven files and three
directory trees: the annotations, the extractor and its test, two polyglot fixture services, three
experiment scripts, and this repository's own contract and exploration prose.

Nothing was published — `packages/jvm` and `tools/extractors/jvm` are both under
`experimentalSource` — so no consumer had pinned the name.

**Applied.** Renamed to `sh.azimuth` and `sh.azimuth.emit`, matching the canonical domain. The
extractor references the annotations through a typed `import sh.azimuth.Azimuth` rather than a
descriptor string, so an incomplete rename fails to compile rather than silently extracting
nothing. The change is unverified by execution: no build was run, because the repository rule
forbids it. Consistency was established by sweep — fixtures, experiment scripts and the contract
all moved together, and no `dev.drim` reference survives outside the immutable archive and the
`drim-dogfood` exploration.

### F7 — No contract is pinned to the code it specifies

No test reads any contract. The frozen-vector tests pin core against literals held in the test
files, so the published preimages and their digests exist as two independent copies that happen to
agree — the schedule digest appears in `azimuth/standards/verification.md` and again in
`tools/azimuth/tests/verification.rs:425`. Editing a contract's preimage changes no test outcome.

This is the same shape as the deleted decision narrative, one level down: prose asserting facts
about code with no mechanism holding them together.

### F8 — The contracts are already close to machine-readable

All eighty-eight fenced blocks across the original seven contracts carry a language tag — sixty-nine
`json`, thirteen `markdown`, six `text`. None is bare. What is missing is a role: nothing
distinguishes a valid example from a rejected one from a canonical preimage, and preimages are
bound to their digests by an adjacent English sentence.

One addition closes it. Everything after the language token in a fence info string is ignored by
every markdown renderer, so a role token is invisible in rendered output:

```text
```json azimuth:valid
```json azimuth:reject
```json azimuth:preimage sha256=58dc690f4b9e…
```text azimuth:illustrative
```

Carrying the digest in the info string makes the preimage-to-digest binding structural rather than
prose-adjacent. Some blocks are fragments never meant to parse alone, so `illustrative` must be the
default rather than an afterthought.

### F9 — Closed value sets are restated in five places and enforced in one

Twelve value vocabularies appear in both `docs/glossary.md` and the contracts, and several again in
`docs/framework.md`, `azimuth/README.md` and `docs/assurance-extensions.md`. Only the contract copy
is adjacent to the parser that enforces it, and none is pinned by a test.

The duplication demonstrably rots: both derived documents claimed six Claim domains where the
parser has two, and no contract contradicted them because no contract declared the field at all.

## Decisions

- **E1 — The contracts are product and live at the top level.** Applied: `azimuth/formats/` moved to
  `contracts/`, with references rewritten everywhere except the immutable change archive.
- **E2 — `azimuth/` holds only what `azimuth init` scaffolds.** That is the test for whether
  something belongs there.
- **E3 — Write the missing contracts rather than record them as gaps.** Applied: ten written, and
  the two absent site profiles added.
- **E4 — Each closed value set gets one normative home in a contract; the glossary keeps the
  definition and narrowing and drops the enumeration.** Applied: twelve glossary entries and two
  framework enumerations now point at the owning contract. Duplicate again later only if a reader is
  demonstrably worse off.
- **E5 — Adopt the fence role convention before any harness is written.** A harness built against
  an untagged format would force the convention afterwards, on eighty-eight blocks plus whatever
  the ten new contracts add.
- **E6 — Defer renaming `azimuth/standards/verification.md`.** It is what `azimuth init` writes, so
  the rename is a consumer-visible change and belongs with other scaffold changes, not here.

## Rejected alternatives

- **`docs/formats/`** — rejected by F2. It was the first choice and it buries a published surface.
- **Organising `contracts/` by audience now** — `protocol/`, `model/`, `extraction/`,
  `federation/`, `outputs/`. Attractive and probably right eventually, but it multiplies the
  reference churn of a move that had to happen anyway, and the audiences were only knowable after
  the missing ten existed.
- **Deleting the derived restatements outright.** A derived read is legitimate and `framework.md`
  declares itself derived. The defect is undetectable drift, not the existence of a second copy.

## Open questions

1. Should `contracts/` be subdivided by audience, and does the protocol subset deserve separation
   from the authoring subset given that only the former is implemented by outside parties?
2. Does the export deserve a `format` identifier? It is the only serialized artifact without one.
3. Should the `## Semantics` parser exception be removed now that nothing uses it?
4. Should `azimuth/standards/` be renamed when the next consumer-visible scaffold change lands?
5. Is a single normative home per value set sufficient, or does the glossary need generated
   inclusion to stay useful without restating?

## Result

No change is created by this exploration; the moves and the ten new contracts were applied as they
were established. What remains recorded is E4 pending application, E5 as a convention to adopt
before a harness exists, and the five questions above. It finishes when questions 1 and 5 are
dispositioned.

## What would falsify this

- **E2 is wrong** if something a consumer must author turns out not to be scaffolded by
  `azimuth init`, which would make the scaffold the incomplete artifact rather than the test.
- **F3 is wrong** if the forty-seven identifier count includes internal envelopes that no third
  party ever authors or receives, which would shrink the public surface below twenty-one.
- **E4 is wrong** if removing enumerations from the glossary makes it unusable as bounded
  terminology, measured by whether a reader must open a contract to understand a term.
- **F8 is wrong** if a role convention cannot express what a harness needs without a schema, which
  would mean the contracts must become machine-readable documents rather than annotated prose.
