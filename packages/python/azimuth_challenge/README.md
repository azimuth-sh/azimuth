# Machine Challenger package

Enable `azimuth.challenge` explicitly. Declarations in `verification.md` select Claim Review or Claim Health conclusions; `Realizes` and `ImplementsMechanism` select production targets. `ImplementsChallenger` links implementation code, never target code.

## Source links

.NET: `Azimuth.Challenge.Annotations.ImplementsChallengerAttribute("id")`, optional repository-relative `ConfigurationFile`.

TypeScript: `implementsChallenger("id")` imported from `@azimuth-sh/challenge` inside a semantically named function.

Python: `@implements_challenger("id", configuration_files=["relative/config.json"])` imported from `azimuth_challenge.annotations`, with a module declaration `# azimuth.challenge: module package.module`. The AST emitter extracts callable identity without executing sources:

```sh
python3 tools/extractors/challenge/azimuth_emit.py --root REPOSITORY --output manifest.json implementation.py
```

## Scope preparation

```sh
PYTHONPATH=packages/python python3 -m azimuth_challenge scope \
  --manifest repository=qualified-manifest.json --root repository=/checkout \
  --request scope-request.json --out source-scope.json
```

`--manifest` and `--root` repeat. Repository identities remain portable; bindings locate checkouts. A plain manifest path belongs to the request's repository. Prefer explicit manifest repository bindings in federated accounts.

Strict request: `format: azimuth-challenge-scope-request`, `version: 1`, `challenger`, `repository`, `tool: stryker-net|semgrep`, `claims`, `mechanisms`, `checks`, optional `areas`, `project`, `test_project`, `test_filter`, `rules`. Claims select assessed Claims. Nonempty Mechanisms narrow production targeting to their implementations; otherwise Claim realizations are targets. Areas narrow the selected source identities. Excluded repositories/Areas are reported. Missing mappings leave the report incomplete; never fall back to all files.

Current emitters locate sites but do not supply exact mutation regions. Scope preparation therefore explicitly reports whole-file expansion. Stryker's `mutate` entries are exact file patterns relative to the target project directory. An empty `test_filter` derives a disjunction of exact `FullyQualifiedName` predicates from selected .NET Check method sites. An authored filter must equal that resolved selection; otherwise the report is incomplete because omissions and expansions are unproven. The execution runner must support this filter (Microsoft Testing Platform runners such as xUnit v3 do not support the VSTest filter contract).

Build project inputs, analysis files and Check sites remain separate. The scope report fingerprints every selected file and the emitted site; execution rejects changed files. Independent Claim Review must examine completeness and the scope/configuration inputs. Selecting tags never proves all relevant helpers have been included.

## Execution

Capture tracked and nonignored untracked source, including changed content and deleted-file state:

```sh
PYTHONPATH=packages/python python3 -m azimuth_challenge.capture \
  --root /checkout --repository repository --git-executable /absolute/git --out capture.json
```

Run only at an explicitly authorized expensive-execution boundary:

```sh
PYTHONPATH=packages/python python3 -m azimuth_challenge.driver \
  --scope source-scope.json --root /checkout --capture capture.json --git-executable /absolute/git \
  --tool-executable /absolute/dotnet-or-semgrep --environment environment.json \
  --timeout 900 --out native-report.json
```

Internal relative source links retain their literal target and resolved identity in the capture; escaping links are rejected; dangling internal links retain their unavailable-target state. Selected source sites cannot use ambiguous symlink locators. The driver recaptures the complete current Git inventory before execution and rejects omitted, changed or newly present files, then recreates internal links in the isolated tree.

The driver copies digest-verified captured files into a disposable source directory, invokes a fixed argument vector without a shell or inherited environment, caps output and duration, and terminates its process group. This is isolation of mutation artifacts, not a hostile-code sandbox or guaranteed containment of escaping descendants. Build-tool configuration and runtime dependencies may require explicitly supplied environment values. No credentials are implied.

Stryker reports retain native mutation identities, statuses and locations. Surviving and uncovered mutations are objections requiring relevance/equivalence assessment. Compile errors, timeouts, discovery failure and uncovered execution remain visible; ignored mutants do not establish a clean search. The batch execution Subject is the exact captured source used by the Challenger. Stryker's report does not expose exact materialized source snapshots for individual mutants; this integration does not manufacture mutant Subjects or product Observations for them.

Semgrep uses only explicit local custom rules and target file arguments, with metrics and version checking disabled. Native rule findings become objections. Errors, skipped files and empty discovery are inconclusive. Community local rules provide intrafile analysis; clean output does not establish interfile or cross-language credential confinement.

Tool-specific docs: [Stryker.NET configuration](https://stryker-mutator.io/docs/stryker-net/configuration/), [Semgrep CLI](https://semgrep.dev/docs/cli-reference).

## Native Run import

The package includes a digest-pinned `adapter.py` (`azimuth-challenge/native`, `challenge.import`, forms `mutation` and `source-analysis`) and native normalization resource `native.py`. Configure both the normalizer and `azimuth_run/protocol.py` as exact resources. Adapter environment is empty; limits are 30 seconds, 16 MiB stdout and 1 MiB stderr. It imports an `azimuth-challenge-native-account` carrying raw reports and measured activities, independently normalizes tool statuses and validates all planned scope fingerprints. It never promotes an exit code or aggregate mutation score into Claim Health. Exact assessment pinning belongs to native Run planning.

Build the import account after planning and actual tool execution:

```sh
PYTHONPATH=packages/python python3 -m azimuth_challenge.account \
  --launch launch-plan.json --report challenger-id:unit-id=native-report.json --out native-account.json
azimuth run import --plan launch-plan.json --input native-account=native-account.json --out run.json
```

Reports must match the planned source Subject and reviewed scope exactly. The native import adapter is `packages/python/azimuth_challenge/adapter.py`; configured resources are `native-normalizer` (`native.py`) and `run-protocol` (`azimuth_run/protocol.py`), ordered by ID. Its fixed configuration prevents ambient normalization settings. Corrections are currently new executions; existing immutable Runs remain inspectable.
