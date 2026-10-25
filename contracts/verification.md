# Verification declarations

The verification facet is a module's `verification.md`, written in the Requirement-first account grammar of [account-verification.md](account-verification.md). It declares what each Check means, how its result contributes to a Scenario through an inline Evidence Binding, and which machine Verification Challenges search for weaknesses in the Checks and the implementation. It never records an execution result, and it authors no decision: independent Verification Review records ([assurance-reviews.md](assurance-reviews.md)) judge the verification's credibility, and deployment Requirement Status is computed by the Assurance Service from applicable evidence ([deployment-requirement-status.md](deployment-requirement-status.md)). Neither a binding nor a source marker constitutes independent acceptance.

Only a Requirement that declares `Verification: reviewed` ([spec.md](spec.md)) may receive Checks, Evidence Bindings or Verification Challenges, and such a Requirement owes an Evidence Binding for every Scenario. A Requirement without that declaration stops at intent. An ordinary native test without deliberate Check enrollment remains outside Azimuth.

Challenges are optional. The author declares a Verification Challenge where an objection search would strengthen the verification, scoped to selected Requirements, Mechanisms and Checks; no rule requires a Challenge form for any Requirement. The Verification Review judges whether the present or absent Challenges are adequate for the Requirement. Machine Verification Challenges select the `implementation` target kind and run as separate bounded machine work before the Verification Review. Challenge clean is a scoped negative search fact; findings require disposition against the proposition challenged, and inconclusive results remain gaps. No aggregate mutation score establishes review credibility or Requirement Status.

## File and identity

```markdown
# Verification: <owning-spec-id>
```

At most one `verification.md` sits beside an owning `spec.md`. The header declares mutable module membership; source location is not semantic identity.

Requirement, Scenario, Mechanism and Check IDs are stable project-wide lower-kebab names in separate typed namespaces. Module headers author membership; Scenario nesting authors the current parent relationship. Neither reconstructs identity. Verification Challenge IDs retain their project-global lower-kebab path grammar. An Evidence Binding is identified by its typed `(Check, Scenario)` pair. Contextual lookup requires project, kind and ID, optionally an exact revision. See [entity identity](entity-identity.md).

Declarations may appear in any order. Every id and referenced identity is validated over the complete project account before `--only` selection retains the closure for selected Requirements.

## Semantic challenge scope

Every planned Challenge selection carries:

```json
{
  "anchors": [
    {
      "kind": "implementation",
      "id": "admission-mutation",
      "fingerprint": "sha256:<64-lowercase-hex>"
    }
  ],
  "inputs": [
    {
      "kind": "check-implementation",
      "id": "payments|rust-item|recovery::replay-after-loss",
      "fingerprint": "sha256:<64-lowercase-hex>"
    }
  ],
  "fingerprint": "sha256:<scope-fingerprint>"
}
```

The closed scope kinds are `implementation | scenario | requirement | binding | check | check-implementation | realization | mechanism | mechanism-implementation | artifact | context | area | realization-obligation | surface | surface-member | enumeration`. Every item has exactly `kind`, non-empty stable `id` and fingerprint. Arrays sort and are unique by `(kind, id, fingerprint)` in that order; two fingerprints for one `(kind, id)` are a conflict rather than two items. An item may occur once in each array because selector provenance and target composition are distinct roles.

An anchor is the exact Challenge target selected at planning: for a compact machine Verification Challenge, the `implementation` target named by the Verification Challenge ID and its exact implementation fingerprint ([run-launch-plan.md](run-launch-plan.md)). Inputs are the complete semantic dependencies of that target: the selected Requirements and Scenarios, the production realization and mechanism-implementation sites its Mechanisms or Requirements select, and the selected Checks with their bindings and complete implementation sets. Overlapping selections union the arrays before fingerprinting.

Stable item ids and fingerprints are exact: Scenario and Requirement use their stable ids with their semantic digests; binding uses its `(Check, Scenario)` pair and contribution digest; Check uses its declared id and Check fingerprint. An Execution Context item (kind `context`) uses its owning Check id and the Execution Context fingerprint. Realization and Check implementation use SourceIdentity and source fingerprint. Locator paths never substitute for another kind's id.

Auxiliary component digests use this file's canonical serialization over these exact envelopes:

```json
{"format":"azimuth-execution-context-fingerprint","version":2,"required_execution_context":<execution-context-object>}
{"format":"azimuth-mechanism-record-digest","version":1,"mechanism":<mechanism-record>}
{"format":"azimuth-artifact-property-digest","version":1,"artifact":<artifact-account>}
{"format":"azimuth-area-digest","version":1,"id":<area-id>}
{
  "format": "azimuth-realization-obligation-digest",
  "version": 2,
  "requirement": <parent-requirement-id>,
  "areas": <sorted-distinct-area-ids>
}
{"format":"azimuth-surface-account-digest","version":1,"surface":<surface-account>}
{
  "format": "azimuth-surface-member-digest",
  "version": 1,
  "surface": <surface-id>,
  "kind": "enumerated",
  "file": <site-member-file-identity>
}
```

The surface account has exactly `id`, `contributions` and `members`. Each contribution has `area`, `mount`, `enumerator` and `witness`; the witness has enumeration `kind`, stable `identity` and `source_fingerprint`. Contributions sort by `(area, mount, enumerator, witness.identity)`. A member is either `{"kind":"tagged","identity":<SourceIdentity>,"source_fingerprint":<fingerprint>}` or `{"kind":"enumerated","file":<site-member-file-identity>}`. Members sort first by kind in `tagged | enumerated` order and then by identity or file. Every set-like array is unique on its sort key; a collision with different content makes the dependent scope unavailable.

A mechanism record describes one mechanism of a Requirement, which may declare exact Scenario relevance. It is exactly:

```json
{
  "id": <mechanism-id>,
  "attachment": {
    "target_kind": "requirement",
    "target_id": <parent-requirement-id>,
    "scenarios": <sorted-scenario-ids-or-empty-for-complete-requirement>
  },
  "enforcement": <enforcement-kind>,
  "expect": {
    "unique": <boolean-or-null>,
    "columns": <ordered-columns>,
    "predicate": <string-or-null>
  },
  "artifacts": [{
    "id": <artifact-id>,
    "kind": <artifact-kind>,
    "identity": <SourceIdentity>,
    "unique": <boolean-or-null>,
    "columns": <ordered-columns>,
    "predicate": <string-or-null>
  }],
  "implementations": [{
    "identity": <SourceIdentity>,
    "source_fingerprint": <fingerprint>,
    "artifact": <artifact-id>
  }]
}
```

An explicit realization `Binding:` yields one Artifact and an empty `implementations` array. A marker-derived route yields one or more implementations with exact SourceIdentity and source fingerprint; each owns one companion Artifact. Both arrays sort by SourceIdentity. Zero marker relations, a marker without stable source identity or fingerprint, or a missing or duplicate companion makes the record unavailable. Adding, removing or changing any participating site changes the record. Changing `attachment.scenarios` changes it even when the Requirement is unchanged.

The two routes are exclusive. An explicit realization binding that names a marker companion's raw id or derived assembled key is a structural error and never produces an explicit-binding mechanism record. Ordinary non-companion Artifacts may still be shared by several explicit realization bindings.

For marker-derived records, each manifest `site` is already represented by its assembled SourceIdentity address and its area-qualified `binding` by its rewritten resolved Artifact id. The raw typed bindings never enter the preimage. Neither `site`, assembled `binding`, `lang` nor `file` is added as another record field. Language determines the semantic address kind. File is an accountable locator only.

A surface account is exactly its `id`, sorted contribution `(area, mount, enumerator)` objects, one sorted witness per contribution and sorted member records. A contribution owns `area`, `mount` and `enumerator`; its nested witness has exactly enumeration `kind`, stable `identity` and `source_fingerprint`. A tagged member records its stable SourceIdentity and source fingerprint; an enumerated member records its file identity. Their tagged/enumerated variants cannot collapse. The mount id is an authored contribution identity; its path is excluded.

Scope identity is SHA-256 over RFC 8785 canonical UTF-8 for this exact object:

```json
{
  "format": "azimuth-challenge-scope-fingerprint",
  "version": 2,
  "anchors": <complete-anchors>,
  "inputs": <complete-inputs>
}
```

### Canonical scope vector

This preimage is already in RFC 8785 form:

```json
{"anchors":[{"fingerprint":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","id":"demo|rust-item|demo::subject","kind":"realization"}],"format":"azimuth-challenge-scope-fingerprint","inputs":[{"fingerprint":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","id":"demo|rust-item|demo::check","kind":"check-implementation"}],"version":2}
```

Its SHA-256 value is `sha256:19eb92d4097f865323d98e4ad33b579cf538d81a94c63dfb93995aa6f1d03dc5`.

## Canonical fingerprints

Repository fingerprints use canonical JSON: object keys sort recursively by their exact strings, set-like arrays use their declared order, strings preserve code points, expanded serialization uses two spaces and LF, and exactly one terminal LF is hashed. This is the model `canonical_sha256` contract, not the RFC 8785 serialization used by the Run-bundle format.

- Check fingerprint: format version, Check id, ordered methods, terminal proposition, and sorted implementation semantic identities plus source fingerprints.
- Execution Context fingerprint: the `azimuth-execution-context-fingerprint` envelope above over the context object.
- Verification Challenge fingerprint: the exact preimage below.

Paths, lines, mounts and explanatory prose are excluded. A source, Scenario, binding or Execution Context change stales only the fingerprints whose inputs include it.

Verification Challenge uses this exact fingerprint preimage:

```json
{
  "format": "azimuth-verification-challenge-fingerprint",
  "version": 2,
  "id": <verification-challenge-id>,
  "form": <open-form>,
  "searches_for": <objection-proposition>,
  "targets": <authored-target-kinds>,
  "requirements": <authored-requirement-ids>,
  "mechanisms": <authored-mechanism-ids>,
  "checks": <authored-check-ids>,
  "required_scope": <sorted-distinct-scope-kinds>
}
```

`targets` carries the authored target kinds and `requirements`, `mechanisms` and `checks` carry the compact account's selections for a `### Verification Challenge:` declaration.

## Source boundary

### Mechanism implementation linkage

Source uses the one-argument stable entity marker `ImplementsMechanism(<mechanism-id>)` in the idiomatic ecosystem spelling. The annotation neither accepts nor owns `site`, `binding` or an Artifact id. A language extractor derives this strict pair:

```json
{
  "mechanism_implementations": [
    {
      "mechanism": "completion-guard",
      "site": "cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
      "binding": "rust-symbol:cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
      "file": "src/capture.rs",
      "lang": "rust",
      "source_fingerprint": "sha256:<64-lowercase-hex>"
    }
  ],
  "artifacts": [
    {
      "id": "rust-symbol:cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
      "kind": "rust-symbol",
      "file": "src/capture.rs"
    }
  ]
}
```

These are the only fields on a raw MechanismImplementation. `mechanism` is a stable project-wide lower-kebab ID; no module field is emitted. `file` is one normalized workspace-relative locator, `lang` is one supported extractor language and `source_fingerprint` is exact `sha256:<64-lowercase-hex>`.

`site` is a non-empty qualified identity under the closed ecosystem profiles below. Except for the narrow C++ alpha profile, it contains a module, package or compilation-target identity and a declaring type or receiver. Where supported, it also contains the overload signature or generic arity needed to distinguish declarations. For example, a .NET extractor may emit `Payments.CaptureService.CompleteAsync(Payments.CompletionId,System.Threading.CancellationToken)`. A short method such as `CompleteAsync`, a path-plus-symbol such as `src/Capture.cs#CompleteAsync`, and a source path used only to distinguish overloads are invalid. The accountable emitter establishes those semantic facts. Core treats `site` as opaque: it can require a non-empty trimmed string without control characters or `|`, but it cannot prove from bytes that a module, receiver or overload is genuinely compiler-qualified.

The address-kind mapping for marker-derived mechanisms is closed in alpha 5:

- `csharp` maps to `dotnet-symbol`; and
- `cpp | go | java | javascript | kotlin | python | rust | typescript` maps to `<lang>-symbol`.

In raw extractor output, `binding` is split at its first `:` as `<address-kind>:<site>`. The prefix equals the mapping for `lang`; the complete remaining suffix equals `site` byte-for-byte, including receiver, type and overload syntax. Untyped bindings, a different prefix or suffix, a prequalified `<area>|<address-kind>|<site>` key and the retired `<kind>:<file>#<site>` form are rejected.

The raw companion Artifact requires `id`, `kind` and `file` and permits only the existing optional typed properties `unique`, `columns` and `predicate`. Its `id` equals the complete raw binding, `kind` equals the binding prefix and `file` equals the implementation locator. The raw pairing key is `(id, kind, file)`. Exactly one Artifact matches each implementation and exactly one MechanismImplementation target owns that companion. Repeating or ambiguously matching the triple, sharing it with another marker target, or mixing it with an ordinary Artifact is invalid. The same raw id in different files is retained until area assembly only when each collision is an exact, separately owned marker companion.

A paired raw companion is marker-only. Before the atomic rewrite, core derives its assembled key and rejects an explicit realization `Binding:` equal to either that key or the raw companion id. The MechanismImplementation resolves only the mechanism named by its exact `spec` and `mechanism`; the artifact id does not infer or fan out another target. Only an ordinary non-companion Artifact may be reused by several explicit realization bindings.

For the example above, both `Binding: rust-symbol:cargo:lib:pay:pay::Capture::complete fn(&self)->bool` and `Binding: payments|rust-symbol|cargo:lib:pay:pay::Capture::complete fn(&self)->bool` are invalid. By contrast, two explicit mechanisms may both declare `Binding: postgres-index:payments.capture_key` when that id resolves one ordinary, non-companion Artifact.

Project assembly resolves `file` to exactly one area and mount. It derives `<SourceIdentity> = <area>|<address-kind>|<site>` and, before any model identity, resolution or fingerprinting, atomically rewrites the implementation `binding` and companion Artifact `id` to that exact key. It also attaches `area`, `address_kind`, `address = site` and `mount` to both. For the raw example above in area `payments` and mount `code`, the exact assembled pair is:

```json
{
  "mechanism_implementations": [
    {
      "mechanism": "completion-guard",
      "site": "cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
      "binding": "payments|rust-symbol|cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
      "file": "src/capture.rs",
      "lang": "rust",
      "source_fingerprint": "sha256:<64-lowercase-hex>",
      "area": "payments",
      "address_kind": "rust-symbol",
      "address": "cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
      "mount": "code"
    }
  ],
  "artifacts": [
    {
      "id": "payments|rust-symbol|cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
      "kind": "rust-symbol",
      "file": "src/capture.rs",
      "area": "payments",
      "address_kind": "rust-symbol",
      "address": "cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
      "mount": "code"
    }
  ]
}
```

These are the exact assembled fields for the raw example, which omitted every optional Artifact property. Assembly preserves any emitted `unique`, `columns` and `predicate` value and rewrites only the companion id before attaching source identity. The canonical Artifact account always represents an absent property as `"unique": null`, `"columns": []` or `"predicate": null`. The assembled companion id is already its SourceIdentity key; it is not expanded again as `<area>|<kind>|<id>`.

Here, path-free means that the id is not derived from or extended with the workspace-relative `file`. A semantic package or module identity may retain its language-native separators.

Normally an Artifact's semantic address is its authored id. The marker companion is the one exception: assembly replaces its raw id with the area-qualified SourceIdentity key and uses that key directly as its identity. An explicit realization Artifact and every unrelated Artifact retain their authored kind/id address and are never reinterpreted or rewritten as companions.

Within one `(area, address-kind)`, a qualified site denotes one compiler declaration. Records for different marker targets may not reuse it. Repeating one `(mechanism, SourceIdentity)`, naming another target at that SourceIdentity or supplying a conflicting source account is invalid. The same kind/site in two areas is legal and produces two distinct assembled binding and Artifact ids. One applicable mechanism may resolve several distinct qualified sites; each site and its companion Artifact enters the mechanism record. Structural linkage cannot prove that the set is complete or adequate.

An emitter fails before output when compiler or runtime metadata reports an ambiguous site in the compilation account for that record. Core does not reproduce that semantic proof. Local and federated assembly check syntax, raw binding equality, exact companion pairing, the atomic rewrite and uniqueness and consistency over each complete area. Both derive byte-identical assembled binding, Artifact id and SourceIdentity for the same area and raw record. Neither uses `file`, repository, revision or mount to repair an ambiguous semantic identity. Old records without `site`, path-bearing bindings and raw records carrying an assembled key are schema failures, not deprecated input. A marker companion referenced by an explicit realization binding is likewise rejected before rewrite, not reinterpreted as an ordinary Artifact.

The source locator of a mechanism-implementation Challenge scope item repeats this exact `site`; it does not substitute binding, Artifact id or file.

Moving unchanged source within the same area while preserving language, site and source fingerprint leaves the implementation, mechanism record and semantic Challenge scope identities unchanged. The new file remains visible in complete-model and launch locator accounts and changes their fingerprints. Moving across an area or changing language, site or source content is semantic.

#### Ecosystem semantic-site profiles

Every extractor resolves `--root` once and emits `file` as a non-empty normalized path below that root. The path uses `/`, contains no empty, `.` or `..` segment and contains no backslash. An input outside the root is invalid. Input files and directories select work; they never become semantic identity or alternate roots.

The C++ alpha profile accepts only a Clang-proven program-global declaration with external linkage. It rejects internal linkage, an anonymous namespace, a local declaration, a declaration attached to a C++ module, any function or enclosing class template, any constrained declaration and any canonical type containing a source locator. The exact site is:

```text
<program-global-qualified-name> <Clang canonical function type>
```

This is the one alpha profile without a package or module prefix. It is safe only because every accepted declaration has one program-global external identity. Supporting modules, templates, constraints or internal linkage requires a later build-target and module account; the extractor must reject them rather than append a translation-unit path.

For Python, `--root` is the declared semantic import root. The exact module is the selected `.py` file's normalized relative path with `/` changed to `.`, the `.py` suffix removed and a terminal `.__init__` removed. The site is `<module>.<__qualname__>`. Every module segment is a Python identifier. A file/package collision, two files resolving one module, an ambiguous namespace or an outside-root file is invalid. Positional input grouping does not affect the module.

TypeScript and JavaScript inputs belong to one whole configured project. Every selected file finds the same nearest unambiguous `tsconfig.json | jsconfig.json`; the nearest owning `package.json` at or above that config supplies one exact package name. Both configs at the same nearest level, no config, no package name, a file outside the configured project or inputs spanning projects are invalid. Input arguments only select files. Before Program creation, inputs are canonicalized by real path, deduplicated and sorted.

Discovery accepts exactly `.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`, `.mjs` and `.cjs` and rejects declaration files. One Program loads the complete config file set and compiler options. Every marked declaration has no relevant options, syntax or semantic diagnostic, including any diagnostic in a type declaration needed by its signature. A global diagnostic is relevant. If the emitter cannot prove another diagnostic irrelevant to marker or type resolution, it fails closed.

The exact TypeScript/JavaScript site account is:

```text
<package>::<module-specifier>::<receiver-kind>::<qualified-symbol><overload-set>
```

`receiver-kind` is exactly `static | instance | none`. `module-specifier` is the unique declared package export accepted by the configured resolver, or otherwise its unique package-relative compiler module specifier. The package-relative form keeps semantic module segments but contains neither an absolute path nor the workspace-relative `file`. A module move changes this value. Relocation evidence moves the complete project root while retaining package, config meaning and module specifier.

The overload set is sorted and non-empty. Each overload contains generic arity, generic constraints, parameter optional/rest modifiers and canonical parameter and return types. Generic parameters are replaced by `$0`, `$1` and so on in declaration order. The checker recursively resolves aliases to canonical target types; alias spelling does not enter the site. Named compiler default-library types retain their compiler-resolved global names; the resolved symbol must include a declaration from the Program's default libraries, and dependency or project augmentations preserve that symbol identity. A same-named consumer symbol with no default-library declaration does not qualify. TypeScript union syntax is encoded as `union(<member>,...)`, with canonical member accounts sorted lexically; nested unions use the same notation after generic substitution. This encoding preserves the site prohibition on `|`; source type syntax remains unchanged. A path-bearing canonical type, duplicate canonical overload or type whose alias/canonical identity is unavailable is invalid.

A mechanism call emits only when its compiler symbol resolves to the `@azimuth-sh/annotations` package's `implementsMechanism` export through a direct, aliased or namespace import. A local homonym is ordinary source. A marker-shaped imported call with wrong arity or non-literal arguments, an anonymous or ambiguous enclosing declaration, an ambiguous symbol or any relevant diagnostic is a controlled extraction failure before output. JavaScript uses `javascript-symbol`; TypeScript uses `typescript-symbol`. The public marker takes one project-global Mechanism ID; CLI syntax is unchanged.

Go uses `<package-import-path>.<receiver><function><signature>` from `go/types`. Receiver and callable type parameters share one zero-based account, receiver parameters first. Every occurrence of a compiler type parameter in receiver, constraints, parameters and results is replaced by the literal token `$<index>`; its source name is excluded. Parameter names are excluded. Constraint, parameter and result order and variadic position remain exact. For example:

```text
example.test/service.Apply[any]($0)->($0)
```

Rust accepts only a compiler-accepted source reachable from exactly one conventional Cargo target through its conventional module graph. It rejects ambiguous multi-target reachability, custom target paths, `#[path]`, generated or included source and unreachable files. Target kind is one of `lib | proc-macro | bin | example | test | bench`. The compiler crate name is the Cargo target name with `-` changed to `_`. The exact site prefix is:

```text
cargo:<target-kind>:<Cargo-target-name>:<compiler-crate-name>::<module>::<declaration>
```

The root module omits `<module>`. Inline and conventional file modules use the compiler-proven module graph, never a path guessed from `src/`. The declaration is qualified by every enclosing module, type or trait implementation.

One ASCII space separates the Rust qualified declaration from its normalized declared signature. The signature retains callable qualifiers, receiver form, declared parameter types, return type, generic constraints and where constraints. It excludes the callable name already present in the qualified declaration and excludes every value-parameter pattern name. Generic parameters are replaced in declared order by `$0`, `$1` and so on. Outside strings and lifetimes, token normalization removes whitespace around punctuation and uses one space only between adjacent word tokens. Declared type-path spelling is identity: an alias and its resolved underlying type produce different sites. This is intentionally weaker than resolved-type identity and is the complete Rust alpha contract.

.NET reflects over built assemblies rather than source. Compiled metadata resolves repeatable attributes, inheritance and generics; the site is derived from metadata alone and a PDB path never disambiguates it. Compiler-generated types and special-name methods are excluded. The exact site is the declaring type's metadata name for a type-level marker, and for a method:

```text
<declaring-type>.<method-name>``<generic-arity>(<parameter-type>,...)
```

The ``` ``<generic-arity> ``` segment is present only on a generic method definition and absent otherwise. A metadata type name is the CLR full name, so a namespace-qualified name with `+` between nested types and a `` ` ``-arity suffix on a generic definition. A by-reference type appends `&`, a pointer appends `*`, an array appends `[` with one comma fewer than its rank and `]`, a constructed generic is `<definition>[<argument>,...]`, and a generic parameter is `!<position>` when declared by the type and `!!<position>` when declared by the method. Positions are zero-based; parameter names are not identity. The address kind is `dotnet-symbol` and `lang` is `csharp`.

The file locator comes from the assembly's portable PDB, made relative to `--root`. An assembly built without a readable portable PDB yields no path and a warning rather than an invented one; a PDB document path that does not lie below `--root` is emitted as read. A marker fingerprint spans the sequence-point line range of the declaring method, extended upward to the line naming the method and over any immediately preceding attribute lines, and for an async or iterator method it is taken from the generated state machine's `MoveNext`. A type's fingerprint is the hash of its declared members' fingerprints, sorted and newline-joined. `ImplementsCheck` and `ImplementsMechanism` require a non-empty fingerprint and fail when metadata cannot produce one.

The JVM profile reads compiled classes from `--classes` and matches them to sources under `--source-root`, with `--root` fixing the emitted locator. Synthetic, anonymous and local classes, synthetic and bridge methods and `module-info` are excluded. `lang` is `kotlin` when the resolved source file ends `.kt` and `java` otherwise; the address kind follows as `kotlin-symbol` or `java-symbol`. The exact site is the binary class name for a type-level marker, and for a method:

```text
<binary-class-name>.<method-name>(<parameter-descriptor>...)<return-descriptor>
```

The descriptor is the erased JVM form: `V Z B C S I J F D` for `void`, `boolean`, `byte`, `char`, `short`, `int`, `long`, `float` and `double`, `L<binary-name-with-slashes>;` for a reference type and the JVM array spelling for an array. Generic parameters are erased and are not identity; overloads separate by their erased descriptor and by nothing else. Parameter names are excluded.

A class's source is resolved from its binary name with any `$` suffix removed and `.` changed to `/`, trying `.java`, then `.kt`, then the `Kt`-stripped `.kt` name for a Kotlin file facade. No unique source, or one source address reachable through two roots, is invalid. A `Realizes` or `ImplementsMechanism` fingerprint is the complete source file. An `ImplementsCheck` fingerprint is the exact enclosing method source: for Java the unique `javac`-parsed method with that name and parameter count, and for Kotlin the unique `fun <name>` declaration with its preceding annotation lines and its brace or expression body. A method that is not uniquely identified this way is invalid, and a method carrying both `ImplementsCheck` and `ImplementsMechanism` uses that same method span for both.

These profiles change no canonical fingerprint preimage. The preimages continue to carry the exact opaque `site` and exclude `file`. Re-emitting a site that violated a profile changes its dependent model and Plan fingerprints; the canonical serializer and published hash vectors do not change.

### Check implementation linkage

Source uses `ImplementsCheck(<check-id>)`. A language extractor emits only:

```json
{
  "check_implementations": [
    {
      "check": "recovery-under-broker-loss",
      "site": "recovery::replay_after_loss",
      "file": "src/recovery.rs",
      "lang": "rust",
      "source_fingerprint": "sha256:<64-lowercase-hex>"
    }
  ]
}
```

For Check records, workspace or project assembly attaches `area`, `mount`, `address_kind` and `address` from declared repository structure. Files and mounts remain locators; the semantic source identity is `<area>|<address-kind>|<address>`.

Configured TypeScript and JavaScript extraction qualifies realization and Check sites as
`<package>::<compiler-resolved-module-specifier>::<enclosing-site>`. The package and module
use the same compiler account as mechanism extraction. A file-level marker uses `<module>`
as its enclosing site; distinct resolved modules remain distinct semantic implementations.
The standalone `scanText` fragment utility has no project account and returns an unqualified
fragment label; its output is not a configured repository manifest.

An implementation marker contains no Scenario, contribution or Execution Context. A native test without the marker emits nothing.

## Rejected input

The parser rejects `## Claim`, `## Judgment` and residual headings, evidence floors, non-test evidence, Strength, detector fields, `Policy:`, `Qualification policy:` and a `judgments.md` delta facet; a model `judgments.md` is not read. Manifests reject `covers`, `mechanism_covers` and `observations`. Annotations reject Covers and CoversMechanism. Nothing is translated or exported twice.
