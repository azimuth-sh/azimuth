#!/usr/bin/env python3

import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path, PurePosixPath

if __package__:
    from .qualify import (
        catalog_at,
        combined_digest,
        validate_approved_contract,
        validate_catalog,
    )
else:
    from qualify import catalog_at, combined_digest, validate_approved_contract, validate_catalog


SPEC = "framework/release-artifacts"
SCENARIOS = (
    "all-experimental-source-is-gated",
    "experiment-gates-need-no-domain-checkout",
    "external-domain-evidence-is-citation-only",
)
ROOT_GATE = PurePosixPath("scripts/check.sh")
POLYGLOT_GATE = PurePosixPath("experiments/polyglot/check.sh")
WORKFLOW = PurePosixPath(".github/workflows/ci.yml")
REFERENCE_FIXTURES = {PurePosixPath("release/test_isolate_experiments.py")}
EVIDENCE_REPOSITORIES = {"drim-dev/azimuth", "azimuth-sh/azimuth"}
PINNED_DOMAIN_URL = re.compile(
    r"https://github\.com/drim-dev/azimuth-demo/(?:blob|tree)/"
    r"(?P<revision>[0-9a-f]{40})(?:/[^\s)\]}>]*)?"
)
DOMAIN_URL = re.compile(
    r"https://github\.com/drim-dev/azimuth-demo/[^\s)\]}>]*"
)
RELATIVE_PATH = re.compile(r"(?P<path>(?:\.\./)+[A-Za-z0-9_.\-/]+)")
LOCAL_DOMAIN_LOCATORS = (
    re.compile(r"(?:^|[\s'\"=:])(?:\.\.?/)+(?:azimuth-demo|drim-dev)(?:/|\b)"),
    re.compile(r"(?:^|[\s'\"=:])~/drim/(?:azimuth-demo|drim-dev)(?:/|\b)"),
    re.compile(r"/mnt/(?:[^\s'\"]*/)*(?:drim|azimuth-demo)(?:/|\b)"),
    re.compile(r"/Users/[^/\s]+/drim/(?:azimuth-demo|drim-dev)(?:/|\b)"),
    re.compile(r"(?:^|[\s'\"`(=])(?:\.\.?/)*experiments/multirepo(?:/|\b)"),
)
EXECUTION_PROGRAMS = (
    "cargo",
    "clang++",
    "go",
    "gradle",
    "java",
    "javac",
    "node",
    "python3",
)
MANIFEST_NAMES = {"Cargo.toml", "build.gradle", "go.mod", "package.json", "settings.gradle"}
EXECUTABLE_SUFFIXES = {
    ".cpp",
    ".cs",
    ".csproj",
    ".go",
    ".gradle",
    ".hpp",
    ".java",
    ".js",
    ".json",
    ".kts",
    ".kt",
    ".py",
    ".rs",
    ".sh",
    ".toml",
    ".ts",
    ".yaml",
    ".yml",
}


class IsolationError(Exception):
    pass


def require(condition, message):
    if not condition:
        raise IsolationError(message)


def tracked_files(root):
    result = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=root,
        check=True,
        capture_output=True,
    )
    return sorted(
        PurePosixPath(path.decode())
        for path in result.stdout.split(b"\0")
        if path
    )


def git_revision(root):
    result = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def git_revision_is_ancestor(root, revision):
    result = subprocess.run(
        ["git", "merge-base", "--is-ancestor", revision, "HEAD"],
        cwd=root,
        capture_output=True,
    )
    return result.returncode == 0


def readable_tracked_files(root, paths):
    sources = {}
    for path in paths:
        try:
            sources[path] = (root / path).read_text()
        except (UnicodeDecodeError, OSError):
            continue
    return sources


def reference_sources(root, tracked):
    return readable_tracked_files(
        root,
        [path for path in tracked if path not in REFERENCE_FIXTURES],
    )


def command_lines(source):
    commands = []
    current = []
    for line in source.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        continued = stripped.endswith("\\")
        current.append(stripped.removesuffix("\\").strip())
        if not continued:
            commands.append(" ".join(current))
            current = []
    require(not current, "shell gate ends with an incomplete continued command")
    return commands


def paths_under(parent, paths):
    return [path for path in paths if path.is_relative_to(parent)]


def command_executes_path(line, path):
    if str(path) not in line:
        return False
    return any(
        re.search(rf"(?:^|[\s;&(]){re.escape(program)}(?:\s|$)", line)
        for program in EXECUTION_PROGRAMS
    )


def resolved_manifest_paths(root, sources):
    resolved = []
    repository_root = root.resolve()
    for source_path, source in sources.items():
        if source_path.name not in MANIFEST_NAMES:
            continue
        for match in RELATIVE_PATH.finditer(source):
            candidate = (root / source_path.parent / match.group("path")).resolve()
            try:
                target = PurePosixPath(candidate.relative_to(repository_root).as_posix())
                resolved.append((source_path, target))
            except ValueError:
                continue
    return resolved


def derive_root_account(catalog, root, tracked, source_overrides=None):
    overrides = source_overrides or {}
    root_gate = overrides.get(ROOT_GATE, (root / ROOT_GATE).read_text())
    polyglot_gate = overrides.get(POLYGLOT_GATE, (root / POLYGLOT_GATE).read_text())
    root_commands = command_lines(root_gate)
    polyglot_commands = command_lines(polyglot_gate)
    experimental_sources = readable_tracked_files(
        root,
        paths_under(PurePosixPath("experiments/polyglot"), tracked),
    )
    experimental_sources.update(overrides)
    resolved_paths = resolved_manifest_paths(root, experimental_sources)
    account = []

    for value in catalog["experimentalSource"]:
        source_root = PurePosixPath(value)
        source_files = paths_under(source_root, tracked)
        require(source_files, f"{source_root}: experimental root has no tracked source")

        if source_root == PurePosixPath("experiments"):
            experiment_names = sorted(
                {
                    path.parts[1]
                    for path in source_files
                    if len(path.parts) > 1
                }
            )
            gates = []
            for name in experiment_names:
                gate = PurePosixPath("experiments") / name / "check.sh"
                require(gate in tracked, f"{source_root}: {name} has no tracked check.sh")
                gate_source = overrides.get(gate, (root / gate).read_text())
                require(
                    any(
                        command_executes_path(line, PurePosixPath("experiments") / name)
                        for line in command_lines(gate_source)
                    ),
                    f"{source_root}: {name} gate does not execute its source",
                )
                invocation = f"./{gate}"
                require(
                    any(invocation in line for line in root_commands),
                    f"{source_root}: root gate does not execute {gate}",
                )
                gates.append(str(gate))
        else:
            directly_referenced = any(
                command_executes_path(line, source_root) for line in polyglot_commands
            )
            manifest_referenced = any(
                (target == source_root or target.is_relative_to(source_root))
                and any(
                    command_executes_path(line, manifest.parent)
                    for line in polyglot_commands
                )
                for manifest, target in resolved_paths
            )
            require(
                directly_referenced or manifest_referenced,
                f"{source_root}: no executable polyglot gate relation was derived",
            )
            gates = [str(POLYGLOT_GATE)]

        account.append({"root": str(source_root), "gates": gates})

    return sorted(account, key=lambda item: item["root"])


def validate_workflow(source):
    checkout_steps = re.findall(
        r"^\s*-\s+uses:\s*actions/checkout@\S+\s*$",
        source,
        re.MULTILINE,
    )
    require(len(checkout_steps) == 1, "CI must contain exactly one canonical checkout step")
    require(
        len(re.findall(r"^\s*fetch-depth:\s*0\s*$", source, re.MULTILINE)) == 1,
        "CI checkout must retain history for archived receipt attribution",
    )
    require(
        not re.search(r"^\s*repository:\s*", source, re.MULTILINE),
        "CI must not check out another repository",
    )
    run_steps = re.findall(r"^\s*-\s+run:\s*(.+?)\s*$", source, re.MULTILINE)
    require(
        run_steps == ["./scripts/check.sh"],
        "CI must execute only the canonical ./scripts/check.sh command",
    )
    return {"file": str(WORKFLOW), "command": run_steps[0]}


def isolation_account_fingerprint(root_account, execution, workflow, inputs, citations):
    account = {
        "roots": root_account,
        "execution": execution,
        "workflow": workflow,
        "executableInputs": sorted(str(path) for path in inputs),
        "domainCitations": citations,
    }
    canonical = json.dumps(account, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(canonical.encode()).hexdigest()


def validate_workflow_receipt(
    receipt,
    revision,
    account_fingerprint,
    revision_is_ancestor=None,
):
    expected = {
        "format": "azimuth-github-workflow-receipt",
        "schemaVersion": 1,
        "workflow": str(WORKFLOW),
        "conclusion": "success",
        "accountFingerprint": account_fingerprint,
    }
    for field, value in expected.items():
        require(
            receipt.get(field) == value,
            f"workflow receipt {field} is {receipt.get(field)!r}, expected {value!r}",
        )
    repository = receipt.get("repository", "")
    require(
        repository in EVIDENCE_REPOSITORIES,
        "workflow receipt repository does not identify an accepted Actions repository",
    )
    receipt_revision = receipt.get("revision", "")
    require(
        re.fullmatch(r"[0-9a-f]{40}", receipt_revision) is not None,
        "workflow receipt revision is not a full Git commit identity",
    )
    is_ancestor = revision_is_ancestor or (lambda candidate: False)
    require(
        receipt_revision == revision or is_ancestor(receipt_revision),
        "workflow receipt revision is neither current nor an ancestor",
    )
    run_url = receipt.get("runUrl", "")
    require(
        re.fullmatch(rf"https://github\.com/{re.escape(repository)}/actions/runs/[0-9]+", run_url),
        "workflow receipt runUrl does not identify a canonical Actions run",
    )
    return {
        field: receipt[field]
        for field in (*expected, "repository", "revision", "runUrl")
    }


def archived_workflow_receipt(
    root,
    revision,
    account_fingerprint,
    revision_is_ancestor=None,
):
    archive = root / "azimuth/changes/archive"
    candidates = sorted(archive.glob("*/workflow-receipt.json"))
    matches = []
    for path in candidates:
        try:
            receipt = json.loads(path.read_text())
        except (json.JSONDecodeError, OSError) as error:
            raise IsolationError(f"cannot read archived workflow receipt {path}: {error}")
        if receipt.get("accountFingerprint") != account_fingerprint:
            continue
        matches.append(
            validate_workflow_receipt(
                receipt,
                revision,
                account_fingerprint,
                revision_is_ancestor,
            )
        )
    require(
        len(matches) <= 1,
        "more than one archived workflow receipt matches the current isolation account",
    )
    return matches[0] if matches else None


def validate_root_sequence(source, account):
    commands = command_lines(source)
    experiment_gates = sorted(
        {
            gate
            for item in account
            if item["root"] == "experiments"
            for gate in item["gates"]
        }
    )
    release_command = "./release/check.sh --experiments-executed"
    release_positions = [
        index for index, command in enumerate(commands) if release_command in command
    ]
    require(
        len(release_positions) == 1,
        f"root gate must invoke {release_command!r} exactly once",
    )
    release_position = release_positions[0]
    for gate in experiment_gates:
        invocation = f"./{gate}"
        positions = [
            index for index, command in enumerate(commands) if invocation in command
        ]
        require(
            positions and max(positions) < release_position,
            f"root gate must execute {gate} before release qualification",
        )
    return {
        "command": release_command,
        "experimentGates": experiment_gates,
        "outcome": "passed",
    }


def executable_inputs(catalog, root, tracked):
    selected = {ROOT_GATE, POLYGLOT_GATE, WORKFLOW, PurePosixPath("release/check.sh")}
    roots = [PurePosixPath(path) for path in catalog["experimentalSource"]]
    for path in tracked:
        if path.suffix.lower() not in EXECUTABLE_SUFFIXES:
            continue
        if any(path.is_relative_to(source_root) for source_root in roots):
            selected.add(path)
    return readable_tracked_files(root, sorted(selected))


def validate_domain_inputs(sources):
    for path, source in sources.items():
        for line_number, line in enumerate(source.splitlines(), start=1):
            for pattern in LOCAL_DOMAIN_LOCATORS:
                match = pattern.search(line)
                if match is not None:
                    raise IsolationError(
                        f"{path}:{line_number}: external domain locator {match.group(0)!r}"
                    )
            require(
                "github.com/drim-dev/azimuth-demo" not in line,
                f"{path}:{line_number}: domain citation is an executable input",
            )


def validate_domain_citations(sources):
    citations = []
    for path, source in sources.items():
        for match in DOMAIN_URL.finditer(source):
            url = match.group(0)
            pinned = PINNED_DOMAIN_URL.fullmatch(url)
            require(pinned is not None, f"{path}: mutable domain citation {url!r}")
            citations.append(
                {"file": str(path), "revision": pinned.group("revision"), "url": url}
            )
    require(citations, "canonical source contains no immutable domain provenance citation")
    return sorted(citations, key=lambda item: (item["file"], item["url"]))


def validate_domain_references(sources):
    citations = validate_domain_citations(sources)
    for path, source in sources.items():
        without_citations = PINNED_DOMAIN_URL.sub("", source)
        for line_number, line in enumerate(without_citations.splitlines(), start=1):
            for pattern in LOCAL_DOMAIN_LOCATORS:
                match = pattern.search(line)
                if match is not None:
                    raise IsolationError(
                        f"{path}:{line_number}: local domain evidence locator {match.group(0)!r}"
                    )
    return citations


def write_linkage(root, output_root):
    qualifier = root / "release/isolate_experiments.py"
    tests = root / "release/test_isolate_experiments.py"
    result = output_root / "experimental-isolation.json"
    qualification_fingerprint = combined_digest([result, qualifier])
    test_fingerprint = combined_digest([tests, qualifier])
    component_claims = set(SCENARIOS[:2])
    linkage = {
        "realizes": [
            {
                "spec": SPEC,
                "scenario": scenario,
                "site": "qualify_experimental_isolation",
                "file": "release/isolate_experiments.py",
                "lang": "python",
                "source_fingerprint": combined_digest([qualifier]),
            }
            for scenario in SCENARIOS
        ],
        "covers": [
            {
                "spec": SPEC,
                "scenario": scenario,
                "site": (
                    "qualify_experimental_isolation"
                    if scenario in component_claims
                    else "test_domain_citations_must_be_commit_pinned"
                ),
                "file": (
                    ".azimuth/release/experimental-isolation.json"
                    if scenario in component_claims
                    else "release/test_isolate_experiments.py"
                ),
                "lang": "experimental-isolation" if scenario in component_claims else "python",
                "source_fingerprint": (
                    qualification_fingerprint if scenario in component_claims else test_fingerprint
                ),
                "scope": "component" if scenario in component_claims else "unit",
                "quantification": "universal",
                "oracle": "direct",
            }
            for scenario in SCENARIOS
        ],
        "mechanism_implementations": [],
        "mechanism_covers": [],
        "class_members": [],
        "enumerations": [],
        "artifacts": [
            {
                "id": "experimental-isolation-gate",
                "kind": "repository-guard",
                "file": "release/isolate_experiments.py",
            }
        ],
        "observations": [],
    }
    (output_root / "experimental-isolation-linkage.json").write_text(
        json.dumps(linkage, indent=2) + "\n"
    )


def qualify_experimental_isolation(
    root,
    output_root,
    experiments_executed,
    workflow_receipt=None,
):
    require(
        experiments_executed,
        "covering evidence requires the canonical composed experiment execution",
    )
    catalog = catalog_at(root)
    validate_catalog(catalog, root)
    validate_approved_contract(catalog)
    tracked = tracked_files(root)
    root_account = derive_root_account(catalog, root, tracked)
    execution = validate_root_sequence((root / ROOT_GATE).read_text(), root_account)
    workflow = validate_workflow((root / WORKFLOW).read_text())
    inputs = executable_inputs(catalog, root, tracked)
    validate_domain_inputs(inputs)
    citations = validate_domain_references(reference_sources(root, tracked))
    account_fingerprint = isolation_account_fingerprint(
        root_account,
        execution,
        workflow,
        inputs,
        citations,
    )
    revision = git_revision(root)
    workflow_execution = (
        validate_workflow_receipt(
            workflow_receipt,
            revision,
            account_fingerprint,
            lambda candidate: git_revision_is_ancestor(root, candidate),
        )
        if workflow_receipt is not None
        else archived_workflow_receipt(
            root,
            revision,
            account_fingerprint,
            lambda candidate: git_revision_is_ancestor(root, candidate),
        )
    )
    qualification = {
        "format": "azimuth-experimental-isolation",
        "schemaVersion": 1,
        "accountFingerprint": account_fingerprint,
        "roots": root_account,
        "execution": execution,
        "workflow": workflow,
        "workflowExecution": workflow_execution or {"status": "pending"},
        "executableInputs": sorted(str(path) for path in inputs),
        "domainCitations": citations,
    }
    output_root.mkdir(parents=True, exist_ok=True)
    (output_root / "experimental-isolation.json").write_text(
        json.dumps(qualification, indent=2) + "\n"
    )
    write_linkage(root, output_root)
    return qualification


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--experiments-executed", action="store_true")
    parser.add_argument("--workflow-receipt", type=Path)
    arguments = parser.parse_args()
    root = arguments.root.resolve()
    output = arguments.out or root / ".azimuth/release"
    receipt = (
        json.loads(arguments.workflow_receipt.read_text())
        if arguments.workflow_receipt
        else None
    )
    result = qualify_experimental_isolation(
        root,
        output,
        arguments.experiments_executed,
        receipt,
    )
    print(
        f"qualified {len(result['roots'])} experimental root(s), "
        f"{len(result['executableInputs'])} executable input(s), and "
        f"{len(result['domainCitations'])} immutable citation(s)"
    )


if __name__ == "__main__":
    try:
        main()
    except IsolationError as error:
        raise SystemExit(f"experimental isolation failed: {error}")
