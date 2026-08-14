#!/usr/bin/env python3
"""Emit Azimuth linkage from compiler-parsed Python decorators."""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
from pathlib import Path
import sys

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


def strings(call: ast.Call, count: int, label: str, file: str) -> list[str]:
    values: list[str] = []
    for argument in call.args:
        if not isinstance(argument, ast.Constant) or not isinstance(argument.value, str):
            raise ValueError(f"{file}:{call.lineno}: {label} arguments must be string literals")
        values.append(argument.value)
    if len(values) < count:
        raise ValueError(f"{file}:{call.lineno}: {label} needs at least {count} arguments")
    return values


def marker(decorator: ast.expr) -> tuple[str, ast.Call] | None:
    if not isinstance(decorator, ast.Call):
        return None
    name = decorator.func.id if isinstance(decorator.func, ast.Name) else None
    if name in {"realizes", "covers", "implements_mechanism", "covers_mechanism"}:
        return name, decorator
    return None


def scan(path: Path, relative: str) -> dict[str, list[dict[str, object]]]:
    source = path.read_text(encoding="utf-8")
    tree = ast.parse(source, filename=relative)
    manifest = empty_manifest()

    class Visitor(ast.NodeVisitor):
        def __init__(self) -> None:
            self.parents: list[str] = []

        def visit_ClassDef(self, node: ast.ClassDef) -> None:
            self._visit_named(node)

        def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
            self._visit_named(node)

        def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> None:
            self._visit_named(node)

        def _visit_named(
            self, node: ast.ClassDef | ast.FunctionDef | ast.AsyncFunctionDef
        ) -> None:
            site = ".".join([*self.parents, node.name])
            segment = ast.get_source_segment(source, node) or source
            fingerprint = hashlib.sha256(segment.encode()).hexdigest()
            for decorator in node.decorator_list:
                found = marker(decorator)
                if found is None:
                    continue
                name, call = found
                if name == "realizes":
                    spec, scenario, *_ = strings(call, 2, name, relative)
                    manifest["realizes"].append(entry(spec, scenario, site, relative, fingerprint))
                elif name == "covers":
                    values = strings(call, 4, name, relative)
                    validate_form(values, call, relative)
                    item = entry(values[0], values[1], site, relative, fingerprint)
                    item.update(form(values))
                    manifest["covers"].append(item)
                elif name == "implements_mechanism":
                    spec, mechanism, *_ = strings(call, 2, name, relative)
                    binding = f"python-symbol:{relative}#{site}"
                    manifest["mechanism_implementations"].append(
                        {
                            "spec": spec,
                            "mechanism": mechanism,
                            "binding": binding,
                            "file": relative,
                            "lang": "python",
                            "source_fingerprint": fingerprint,
                        }
                    )
                    manifest["artifacts"].append(
                        {"id": binding, "kind": "python-symbol", "file": relative}
                    )
                else:
                    values = strings(call, 4, name, relative)
                    validate_form(values, call, relative)
                    item: dict[str, object] = {
                        "spec": values[0],
                        "mechanism": values[1],
                        "site": site,
                        "file": relative,
                        "lang": "python",
                        "source_fingerprint": fingerprint,
                    }
                    item.update(form(values))
                    manifest["mechanism_covers"].append(item)
            self.parents.append(node.name)
            self.generic_visit(node)
            self.parents.pop()

    Visitor().visit(tree)
    return manifest


def entry(spec: str, scenario: str, site: str, file: str, fingerprint: str) -> dict[str, object]:
    return {
        "spec": spec,
        "scenario": scenario,
        "site": site,
        "file": file,
        "lang": "python",
        "source_fingerprint": fingerprint,
    }


def form(values: list[str]) -> dict[str, str]:
    result = {"scope": values[2], "quantification": values[3]}
    if len(values) > 4:
        result["oracle"] = values[4]
    return result


def validate_form(values: list[str], call: ast.Call, file: str) -> None:
    if values[2] not in SCOPES:
        raise ValueError(f"{file}:{call.lineno}: unknown scope `{values[2]}`")
    if values[3] not in QUANTIFICATIONS:
        raise ValueError(f"{file}:{call.lineno}: unknown quantification `{values[3]}`")
    if len(values) > 4 and values[4] not in ORACLES:
        raise ValueError(f"{file}:{call.lineno}: unknown oracle `{values[4]}`")


def emit(inputs: list[Path], root: Path) -> dict[str, list[dict[str, object]]]:
    manifest = empty_manifest()
    files: list[Path] = []
    for item in inputs:
        files.extend(item.rglob("*.py") if item.is_dir() else [item])
    for path in sorted(set(files)):
        if any(part in {".git", ".venv", "__pycache__"} for part in path.parts):
            continue
        relative = path.resolve().relative_to(root.resolve()).as_posix()
        partial = scan(path, relative)
        for key, values in partial.items():
            manifest[key].extend(values)
    for values in manifest.values():
        values.sort(key=lambda item: json.dumps(item, sort_keys=True))
    return manifest


def main() -> int:
    parser = argparse.ArgumentParser(prog="azimuth-emit-python")
    parser.add_argument("inputs", nargs="+")
    parser.add_argument("--root", default=".")
    parser.add_argument("--output", "-o", required=True)
    args = parser.parse_args()
    try:
        manifest = emit([Path(value) for value in args.inputs], Path(args.root))
        output = Path(args.output)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    except (OSError, SyntaxError, ValueError) as error:
        print(f"azimuth-emit-python: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
