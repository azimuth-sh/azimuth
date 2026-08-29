#!/usr/bin/env python3
"""Emit Azimuth linkage from compiler-parsed Python decorators."""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
from pathlib import Path
import sys


def module_identity(relative: str) -> tuple[str, bool]:
    locator = Path(relative)
    if (
        locator.is_absolute()
        or locator.suffix != ".py"
        or not locator.parts
        or any(part in {"", ".", ".."} or "\\" in part for part in locator.parts)
    ):
        raise ValueError(f"{relative}: cannot derive an importable Python module")
    parts = list(locator.with_suffix("").parts)
    is_package = parts[-1] == "__init__"
    if is_package:
        parts.pop()
    if not parts or any(not part.isidentifier() for part in parts):
        raise ValueError(f"{relative}: cannot derive an importable Python module")
    return ".".join(parts), is_package


def empty_manifest() -> dict[str, list[dict[str, object]]]:
    return {
        "realizes": [],
        "check_implementations": [],
        "mechanism_implementations": [],
        "class_members": [],
        "enumerations": [],
        "artifacts": [],
    }


def strings(call: ast.Call, count: int, label: str, file: str) -> list[str]:
    if call.keywords:
        raise ValueError(f"{file}:{call.lineno}: {label} does not accept keyword arguments")
    values: list[str] = []
    for argument in call.args:
        if not isinstance(argument, ast.Constant) or not isinstance(argument.value, str):
            raise ValueError(f"{file}:{call.lineno}: {label} arguments must be string literals")
        values.append(argument.value)
    if len(values) != count:
        raise ValueError(f"{file}:{call.lineno}: {label} needs exactly {count} arguments")
    return values


def marker(decorator: ast.expr) -> tuple[str, ast.Call] | None:
    if not isinstance(decorator, ast.Call):
        return None
    name = decorator.func.id if isinstance(decorator.func, ast.Name) else None
    if name in {
        "realizes",
        "implements_check",
        "implements_mechanism",
        "covers",
        "covers_mechanism",
    }:
        return name, decorator
    return None


def scan(
    path: Path, relative: str, module_relative: str | None = None
) -> dict[str, list[dict[str, object]]]:
    source = path.read_text(encoding="utf-8")
    tree = ast.parse(source, filename=relative)
    manifest = empty_manifest()
    module, _ = module_identity(module_relative or relative)
    mechanism_sites: dict[str, tuple[str, str]] = {}
    ordinary_retired_names = {
        node.name
        for node in tree.body
        if isinstance(node, (ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef))
        and node.name in {"covers", "covers_mechanism"}
    }

    class Visitor(ast.NodeVisitor):
        def __init__(self) -> None:
            self.parents: list[tuple[str, str]] = []

        def visit_ClassDef(self, node: ast.ClassDef) -> None:
            self._visit_named(node)

        def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
            self._visit_named(node)

        def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> None:
            self._visit_named(node)

        def _visit_named(
            self, node: ast.ClassDef | ast.FunctionDef | ast.AsyncFunctionDef
        ) -> None:
            qualname: list[str] = []
            for kind, name in self.parents:
                qualname.append(name)
                if kind == "function":
                    qualname.append("<locals>")
            qualname.append(node.name)
            site = ".".join(qualname)
            segment = ast.get_source_segment(source, node) or source
            fingerprint = "sha256:" + hashlib.sha256(segment.encode()).hexdigest()
            for decorator in node.decorator_list:
                found = marker(decorator)
                if found is None:
                    continue
                name, call = found
                if name in ordinary_retired_names:
                    continue
                if name in {"covers", "covers_mechanism"}:
                    raise ValueError(
                        f"{relative}:{call.lineno}: retired alpha 1 marker {name} is not supported"
                    )
                if name == "realizes":
                    spec, claim, *_ = strings(call, 2, name, relative)
                    manifest["realizes"].append(entry(spec, claim, site, relative, fingerprint))
                elif name == "implements_check":
                    check, = strings(call, 1, name, relative)
                    manifest["check_implementations"].append(
                        {
                            "check": check,
                            "site": site,
                            "file": relative,
                            "lang": "python",
                            "source_fingerprint": fingerprint,
                        }
                    )
                elif name == "implements_mechanism":
                    spec, mechanism, *_ = strings(call, 2, name, relative)
                    semantic_site = f"{module}.{site}"
                    if semantic_site in mechanism_sites:
                        prior = mechanism_sites[semantic_site]
                        raise ValueError(
                            f"{relative}:{call.lineno}: ambiguous mechanism site "
                            f"`{semantic_site}` for {prior[0]}#{prior[1]} and "
                            f"{spec}#{mechanism}"
                        )
                    mechanism_sites[semantic_site] = (spec, mechanism)
                    binding = f"python-symbol:{semantic_site}"
                    manifest["mechanism_implementations"].append(
                        {
                            "spec": spec,
                            "mechanism": mechanism,
                            "site": semantic_site,
                            "binding": binding,
                            "file": relative,
                            "lang": "python",
                            "source_fingerprint": fingerprint,
                        }
                    )
                    manifest["artifacts"].append(
                        {"id": binding, "kind": "python-symbol", "file": relative}
                    )
            parent_kind = (
                "function"
                if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
                else "class"
            )
            self.parents.append((parent_kind, node.name))
            self.generic_visit(node)
            self.parents.pop()

    Visitor().visit(tree)
    return manifest


def entry(spec: str, claim: str, site: str, file: str, fingerprint: str) -> dict[str, object]:
    return {
        "spec": spec,
        "claim": claim,
        "site": site,
        "file": file,
        "lang": "python",
        "source_fingerprint": fingerprint,
    }


def emit(inputs: list[Path], root: Path) -> dict[str, list[dict[str, object]]]:
    manifest = empty_manifest()
    try:
        semantic_root = root.resolve(strict=True)
    except OSError as error:
        raise ValueError(f"{root}: cannot resolve --root: {error}") from error
    if not semantic_root.is_dir():
        raise ValueError(f"{root}: --root must be a directory")
    files: dict[Path, str] = {}
    for item in inputs:
        try:
            selected = item.resolve(strict=True)
        except OSError as error:
            raise ValueError(f"{item}: cannot resolve input: {error}") from error
        try:
            selected.relative_to(semantic_root)
        except ValueError as error:
            raise ValueError(f"{item}: input is outside --root") from error
        if selected.is_dir():
            candidates = selected.rglob("*.py")
        elif selected.suffix == ".py":
            candidates = [selected]
        else:
            raise ValueError(f"{item}: Python input must be a .py file or directory")
        for path in candidates:
            resolved = path.resolve(strict=True)
            try:
                relative = resolved.relative_to(semantic_root).as_posix()
            except ValueError as error:
                raise ValueError(f"{path}: input is outside --root") from error
            if any(part in {".git", ".venv", "__pycache__"} for part in Path(relative).parts):
                continue
            files[resolved] = relative

    modules: dict[str, tuple[Path, bool]] = {}
    for path, relative in sorted(files.items()):
        module, is_package = module_identity(relative)
        prior = modules.get(module)
        if prior is not None and prior[0] != path:
            raise ValueError(
                f"{relative}: Python module `{module}` collides with "
                f"{prior[0].relative_to(semantic_root).as_posix()}"
            )
        modules[module] = (path, is_package)
    for module, (path, _) in modules.items():
        parts = module.split(".")
        for index in range(1, len(parts)):
            prefix = ".".join(parts[:index])
            prior = modules.get(prefix)
            if prior is not None and not prior[1]:
                raise ValueError(
                    f"{path.relative_to(semantic_root).as_posix()}: namespace `{prefix}` "
                    f"collides with module {prior[0].relative_to(semantic_root).as_posix()}"
                )

    for path, relative in sorted(files.items()):
        if any(part in {".git", ".venv", "__pycache__"} for part in path.parts):
            continue
        partial = scan(path, relative, relative)
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
