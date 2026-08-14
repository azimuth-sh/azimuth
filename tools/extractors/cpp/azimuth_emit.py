#!/usr/bin/env python3
"""Emit Azimuth linkage from Clang-resolved C++ annotations."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

FUNCTION = re.compile(r"\bFunctionDecl\b.*?\b([A-Za-z_][A-Za-z0-9_]*)\s+'")
ANNOTATION = re.compile(r'AnnotateAttr.*"(azimuth\|[^"\\]+)"')
SCOPES = {"unit", "component", "e2e"}
QUANTIFICATIONS = {"example", "universal"}
ORACLES = {"direct", "golden", "relational", "metamorphic", "model-based", "contract"}


def empty_manifest() -> dict[str, list[dict[str, object]]]:
    return {
        "realizes": [],
        "covers": [],
        "mechanism_implementations": [],
        "mechanism_covers": [],
        "class_members": [],
        "enumerations": [],
        "artifacts": [],
    }


def compiler_annotations(path: Path, compiler: str, includes: list[Path]) -> list[tuple[str, list[str]]]:
    command = [compiler, "-std=c++20", "-fsyntax-only", "-Xclang", "-ast-dump"]
    command.extend(f"-I{include}" for include in includes)
    command.append(str(path))
    completed = subprocess.run(command, capture_output=True, text=True, check=False)
    if completed.returncode != 0:
        raise ValueError(completed.stderr.strip() or f"Clang rejected {path}")
    current: str | None = None
    annotations: list[tuple[str, list[str]]] = []
    for line in completed.stdout.splitlines():
        function = FUNCTION.search(line)
        if function:
            current = function.group(1)
            continue
        annotation = ANNOTATION.search(line)
        if annotation and current:
            annotations.append((current, annotation.group(1).split("|")))
    return annotations


def scan(path: Path, root: Path, compiler: str, includes: list[Path]) -> dict[str, list[dict[str, object]]]:
    relative = path.resolve().relative_to(root.resolve()).as_posix()
    fingerprint = hashlib.sha256(path.read_bytes()).hexdigest()
    manifest = empty_manifest()
    for site, parts in compiler_annotations(path, compiler, includes):
        if len(parts) < 4 or parts[0] != "azimuth":
            raise ValueError(f"{relative}: malformed Azimuth annotation")
        kind = parts[1]
        common = {
            "spec": parts[2],
            "site": site,
            "file": relative,
            "lang": "cpp",
            "source_fingerprint": fingerprint,
        }
        if kind == "realizes":
            manifest["realizes"].append({**common, "scenario": parts[3]})
        elif kind == "covers":
            validate_form(parts, relative)
            manifest["covers"].append(
                {
                    **common,
                    "scenario": parts[3],
                    "scope": parts[4],
                    "quantification": parts[5],
                    "oracle": parts[6],
                }
            )
        elif kind == "implements-mechanism":
            binding = f"cpp-symbol:{relative}#{site}"
            manifest["mechanism_implementations"].append(
                {**common, "mechanism": parts[3], "binding": binding}
            )
            manifest["artifacts"].append(
                {"id": binding, "kind": "cpp-symbol", "file": relative}
            )
        elif kind == "covers-mechanism":
            validate_form(parts, relative)
            manifest["mechanism_covers"].append(
                {
                    **common,
                    "mechanism": parts[3],
                    "scope": parts[4],
                    "quantification": parts[5],
                    "oracle": parts[6],
                }
            )
        else:
            raise ValueError(f"{relative}: unknown annotation kind `{kind}`")
    return manifest


def validate_form(parts: list[str], file: str) -> None:
    if len(parts) != 7:
        raise ValueError(f"{file}: evidence annotation needs scope, quantification and oracle")
    if parts[4] not in SCOPES:
        raise ValueError(f"{file}: unknown scope `{parts[4]}`")
    if parts[5] not in QUANTIFICATIONS:
        raise ValueError(f"{file}: unknown quantification `{parts[5]}`")
    if parts[6] not in ORACLES:
        raise ValueError(f"{file}: unknown oracle `{parts[6]}`")


def emit(inputs: list[Path], root: Path, compiler: str, includes: list[Path]) -> dict[str, list[dict[str, object]]]:
    manifest = empty_manifest()
    files: list[Path] = []
    for item in inputs:
        files.extend(item.rglob("*.cpp") if item.is_dir() else [item])
    for path in sorted(set(files)):
        partial = scan(path, root, compiler, includes)
        for key, values in partial.items():
            manifest[key].extend(values)
    return manifest


def main() -> int:
    parser = argparse.ArgumentParser(prog="azimuth-emit-cpp")
    parser.add_argument("inputs", nargs="+")
    parser.add_argument("--root", default=".")
    parser.add_argument("--output", "-o", required=True)
    parser.add_argument("--compiler", default="clang++")
    parser.add_argument("--include", action="append", default=[])
    args = parser.parse_args()
    try:
        manifest = emit(
            [Path(value) for value in args.inputs],
            Path(args.root),
            args.compiler,
            [Path(value) for value in args.include],
        )
        output = Path(args.output)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    except (OSError, ValueError) as error:
        print(f"azimuth-emit-cpp: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
