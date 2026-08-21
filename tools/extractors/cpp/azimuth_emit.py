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
def empty_manifest() -> dict[str, list[dict[str, object]]]:
    return {
        "realizes": [],
        "check_implementations": [],
        "mechanism_implementations": [],
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
    source = path.read_text(encoding="utf-8")
    manifest = empty_manifest()
    for site, parts in compiler_annotations(path, compiler, includes):
        if len(parts) < 3 or parts[0] != "azimuth":
            raise ValueError(f"{relative}: malformed Azimuth annotation")
        kind = parts[1]
        fingerprint = "sha256:" + hashlib.sha256(function_source(source, site).encode()).hexdigest()
        if kind == "implements-check":
            if len(parts) != 3:
                raise ValueError(f"{relative}: implements-check needs exactly one argument")
            manifest["check_implementations"].append(
                {
                    "check": parts[2],
                    "site": site,
                    "file": relative,
                    "lang": "cpp",
                    "source_fingerprint": fingerprint,
                }
            )
            continue
        if len(parts) != 4:
            raise ValueError(f"{relative}: {kind} needs exactly two arguments")
        common = {
            "spec": parts[2],
            "site": site,
            "file": relative,
            "lang": "cpp",
            "source_fingerprint": fingerprint,
        }
        if kind == "realizes":
            manifest["realizes"].append({**common, "scenario": parts[3]})
        elif kind == "implements-mechanism":
            binding = f"cpp-symbol:{relative}#{site}"
            manifest["mechanism_implementations"].append(
                {**common, "mechanism": parts[3], "binding": binding}
            )
            manifest["artifacts"].append(
                {"id": binding, "kind": "cpp-symbol", "file": relative}
            )
        else:
            raise ValueError(f"{relative}: unknown annotation kind `{kind}`")
    return manifest


def function_source(source: str, site: str) -> str:
    signature = re.compile(rf"\b{re.escape(site)}\s*\([^;{{}}]*\)[^;{{}}]*\{{")
    found = signature.search(source)
    if found is None:
        raise ValueError(f"cannot resolve source for C++ function `{site}`")
    start = source.rfind("\n", 0, found.start()) + 1
    opening = source.find("{", found.start(), found.end())
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[start : index + 1]
    raise ValueError(f"cannot resolve end of C++ function `{site}`")


def emit(inputs: list[Path], root: Path, compiler: str, includes: list[Path]) -> dict[str, list[dict[str, object]]]:
    manifest = empty_manifest()
    files: list[Path] = []
    for item in inputs:
        files.extend(item.rglob("*.cpp") if item.is_dir() else [item])
    for path in sorted(set(files)):
        partial = scan(path, root, compiler, includes)
        for key, values in partial.items():
            manifest[key].extend(values)
    for values in manifest.values():
        values.sort(key=lambda item: json.dumps(item, sort_keys=True))
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
