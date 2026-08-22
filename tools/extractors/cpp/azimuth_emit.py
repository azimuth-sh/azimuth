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

ANNOTATION = re.compile(r'AnnotateAttr.*"(azimuth\|[^"\\]+)"')
FUNCTION_KINDS = {
    "FunctionDecl",
    "CXXMethodDecl",
    "CXXConstructorDecl",
    "CXXDestructorDecl",
    "CXXConversionDecl",
}
AST_NODE = re.compile(
    r"\b(FunctionDecl|CXXMethodDecl|CXXConstructorDecl|CXXDestructorDecl|"
    r"CXXConversionDecl|AnnotateAttr)\b"
)


def empty_manifest() -> dict[str, list[dict[str, object]]]:
    return {
        "realizes": [],
        "check_implementations": [],
        "mechanism_implementations": [],
        "class_members": [],
        "enumerations": [],
        "artifacts": [],
    }


def compiler_output(
    path: Path, compiler: str, includes: list[Path], dump: str
) -> str:
    command = [compiler, "-std=c++20", "-fsyntax-only", "-Xclang", dump]
    command.extend(f"-I{include}" for include in includes)
    command.append(str(path))
    completed = subprocess.run(command, capture_output=True, text=True, check=False)
    if completed.returncode != 0:
        raise ValueError(completed.stderr.strip() or f"Clang rejected {path}")
    return completed.stdout


def location_value(location: dict[str, object], key: str) -> object | None:
    expansion = location.get("expansionLoc")
    if isinstance(expansion, dict) and key in expansion:
        return expansion[key]
    spelling = location.get("spellingLoc")
    if isinstance(spelling, dict) and key in spelling:
        return spelling[key]
    return location.get(key)


def annotated_declarations(
    path: Path, compiler: str, includes: list[Path]
) -> list[tuple[str, str, list[str], int, int]]:
    text = compiler_output(path, compiler, includes, "-ast-dump")
    text_records: list[dict[str, object]] = []
    function_stack: list[tuple[int, int]] = []
    for line in text.splitlines():
        node = AST_NODE.search(line)
        if node is None:
            continue
        depth = node.start()
        while function_stack and function_stack[-1][0] >= depth:
            function_stack.pop()
        kind = node.group(1)
        if kind in FUNCTION_KINDS:
            text_records.append({"saw_annotation": False, "azimuth": []})
            function_stack.append((depth, len(text_records) - 1))
            continue
        if not function_stack or depth != function_stack[-1][0] + 2:
            if ANNOTATION.search(line):
                raise ValueError(f"{path}: Azimuth annotation is not on a function declaration")
            continue
        record = text_records[function_stack[-1][1]]
        record["saw_annotation"] = True
        annotation = ANNOTATION.search(line)
        if annotation:
            values = record["azimuth"]
            if not isinstance(values, list):
                raise AssertionError("invalid internal Clang annotation account")
            values.append(annotation.group(1).split("|"))
    annotation_groups = [
        record["azimuth"] for record in text_records if record["saw_annotation"]
    ]
    tree = json.loads(compiler_output(path, compiler, includes, "-ast-dump=json"))
    current_source = path.resolve()
    declarations: list[tuple[str, str, int, int]] = []
    aliases: dict[str, str] = {}

    def collect_aliases(node: dict[str, object], context: tuple[str, ...]) -> None:
        kind = node.get("kind")
        name = node.get("name")
        child_context = context
        if kind in {"NamespaceDecl", "CXXRecordDecl", "RecordDecl"}:
            if isinstance(name, str) and name and not node.get("isImplicit"):
                child_context = (*context, name)
        if kind in {"TypedefDecl", "TypeAliasDecl"} and isinstance(name, str) and name:
            type_account = node.get("type")
            if isinstance(type_account, dict):
                canonical = type_account.get("desugaredQualType", type_account.get("qualType"))
                if isinstance(canonical, str) and canonical:
                    aliases["::".join((*context, name))] = canonical
        inner = node.get("inner", [])
        if isinstance(inner, list):
            for child in inner:
                if isinstance(child, dict):
                    collect_aliases(child, child_context)

    collect_aliases(tree, ())

    def canonical_signature(
        signature: str,
        node: dict[str, object],
        name: str,
        context: tuple[str, ...],
    ) -> str:
        canonical = signature
        for alias, target in sorted(aliases.items(), key=lambda item: -len(item[0])):
            canonical = re.sub(rf"(?<![A-Za-z0-9_]){re.escape(alias)}(?![A-Za-z0-9_])", target, canonical)
        contextual = sorted(
            (
                (alias.rsplit("::", 1)[-1], target, alias.count("::"))
                for alias, target in aliases.items()
                if alias.rpartition("::")[0]
                in {"::".join(context[:depth]) for depth in range(len(context) + 1)}
            ),
            key=lambda item: -item[2],
        )
        for alias, target, _ in contextual:
            canonical = re.sub(
                rf"(?<![A-Za-z0-9_:]){re.escape(alias)}(?![A-Za-z0-9_])",
                target,
                canonical,
            )
        inner = node.get("inner", [])
        if isinstance(inner, list):
            for child in inner:
                if not isinstance(child, dict) or child.get("kind") != "ParmVarDecl":
                    continue
                type_account = child.get("type")
                if not isinstance(type_account, dict):
                    continue
                parameter = type_account.get("desugaredQualType", type_account.get("qualType"))
                if isinstance(parameter, str) and re.search(
                    r"(?:lambda at |(?:^|[ (])(?:/|[A-Za-z]:[\\/])[^)]*:\d+:\d+)",
                    parameter,
                ):
                    raise ValueError(
                        f"{path}: canonical signature of `{name}` contains a source locator"
                    )
        return canonical

    def visit(
        node: dict[str, object],
        context: tuple[str, ...],
        ambiguous: bool,
        templated: bool,
        local: bool,
        module_attached: bool,
    ) -> None:
        kind = node.get("kind")
        name = node.get("name")
        child_context = context
        child_ambiguous = ambiguous
        child_templated = templated or (
            isinstance(kind, str) and "Template" in kind
        )
        child_local = local
        child_module = module_attached or kind in {"ModuleDecl", "ExportDecl"} or any(
            key in node for key in ("owningModule", "moduleOwnershipKind", "isModulePrivate")
        )
        if kind in {"NamespaceDecl", "CXXRecordDecl", "RecordDecl"}:
            if isinstance(name, str) and name and not node.get("isImplicit"):
                child_context = (*context, name)
            elif not node.get("isImplicit"):
                child_ambiguous = True
        inner = node.get("inner", [])
        if not isinstance(inner, list):
            inner = []
        marked = any(
            isinstance(child, dict) and child.get("kind") == "AnnotateAttr"
            for child in inner
        )
        if kind in FUNCTION_KINDS and marked:
            location = node.get("loc")
            declaration_file = location.get("file") if isinstance(location, dict) else None
            included_from = location.get("includedFrom") if isinstance(location, dict) else None
            if declaration_file is None and included_from is None:
                # Clang elides a repeated main-file locator. Included declarations retain
                # either their own file or an includedFrom account, so only the former may
                # inherit the compiler invocation's source identity.
                declared_source = current_source
            elif not isinstance(declaration_file, str):
                raise ValueError(
                    f"{path}: Clang did not identify the source file of annotated declaration"
                )
            else:
                try:
                    declared_source = Path(declaration_file).resolve(strict=True)
                except OSError as error:
                    raise ValueError(
                        f"{path}: cannot resolve annotated declaration source {declaration_file}: "
                        f"{error}"
                    ) from error
            if declared_source != current_source:
                raise ValueError(
                    f"{path}: annotated declaration is outside the current source file: "
                    f"{declaration_file}"
                )
            if child_ambiguous or not isinstance(name, str) or not name:
                raise ValueError(f"{path}: Clang reported an ambiguous annotated declaration")
            if local:
                raise ValueError(f"{path}: annotated declaration `{name}` is local")
            if templated:
                raise ValueError(f"{path}: annotated declaration `{name}` is templated or constrained")
            if child_module:
                raise ValueError(f"{path}: annotated declaration `{name}` is attached to a C++ module")
            if kind == "FunctionDecl" and node.get("storageClass") == "static":
                raise ValueError(f"{path}: annotated declaration `{name}` has internal linkage")
            if not isinstance(node.get("mangledName"), str):
                raise ValueError(
                    f"{path}: Clang did not prove external-linkage identity for `{name}`"
                )
            type_account = node.get("type")
            if not isinstance(type_account, dict):
                raise ValueError(f"{path}: Clang omitted the type of `{name}`")
            signature = type_account.get("desugaredQualType", type_account.get("qualType"))
            if not isinstance(signature, str) or not signature:
                raise ValueError(f"{path}: Clang omitted the canonical signature of `{name}`")
            signature = canonical_signature(signature, node, name, child_context)
            if re.search(r"(?:lambda at |(?:^|[ (])(?:/|[A-Za-z]:[\\/])[^)]*:\d+:\d+)", signature):
                raise ValueError(
                    f"{path}: canonical signature of `{name}` contains a source locator"
                )
            source_range = node.get("range")
            if not isinstance(source_range, dict):
                raise ValueError(f"{path}: Clang omitted the source range of `{name}`")
            begin = source_range.get("begin")
            end = source_range.get("end")
            if not isinstance(begin, dict) or not isinstance(end, dict):
                raise ValueError(f"{path}: Clang omitted the source range of `{name}`")
            start = location_value(begin, "offset")
            finish = location_value(end, "offset")
            token_length = location_value(end, "tokLen") or 1
            if not all(isinstance(value, int) for value in (start, finish, token_length)):
                raise ValueError(f"{path}: Clang omitted byte offsets for `{name}`")
            site = f"{'::'.join((*child_context, name))} {signature}"
            declarations.append((name, site, start, finish + token_length))
        if kind in FUNCTION_KINDS:
            child_local = True
        for child in inner:
            if isinstance(child, dict):
                visit(
                    child,
                    child_context,
                    child_ambiguous,
                    child_templated,
                    child_local,
                    child_module,
                )

    visit(tree, (), False, False, False, False)
    if len(annotation_groups) != len(declarations):
        raise ValueError(
            f"{path}: Clang annotation/declaration account is ambiguous "
            f"({len(annotation_groups)} annotated text declarations, "
            f"{len(declarations)} JSON declarations)"
        )
    result: list[tuple[str, str, list[str], int, int]] = []
    for declaration, groups in zip(declarations, annotation_groups, strict=True):
        if not isinstance(groups, list):
            raise AssertionError("invalid internal Clang annotation group")
        name, site, start, end = declaration
        result.extend((name, site, parts, start, end) for parts in groups)
    return result


def scan(path: Path, root: Path, compiler: str, includes: list[Path]) -> dict[str, list[dict[str, object]]]:
    relative = path.resolve().relative_to(root.resolve()).as_posix()
    source = path.read_bytes()
    manifest = empty_manifest()
    mechanism_sites: dict[str, tuple[str, str]] = {}
    for simple_site, semantic_site, parts, start, end in annotated_declarations(
        path, compiler, includes
    ):
        if len(parts) < 3 or parts[0] != "azimuth":
            raise ValueError(f"{relative}: malformed Azimuth annotation")
        kind = parts[1]
        fingerprint = "sha256:" + hashlib.sha256(source[start:end]).hexdigest()
        if kind == "implements-check":
            if len(parts) != 3:
                raise ValueError(f"{relative}: implements-check needs exactly one argument")
            manifest["check_implementations"].append(
                {
                    "check": parts[2],
                    "site": simple_site,
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
            "site": simple_site,
            "file": relative,
            "lang": "cpp",
            "source_fingerprint": fingerprint,
        }
        if kind == "realizes":
            manifest["realizes"].append({**common, "scenario": parts[3]})
        elif kind == "implements-mechanism":
            spec, mechanism = parts[2], parts[3]
            if semantic_site in mechanism_sites:
                prior = mechanism_sites[semantic_site]
                raise ValueError(
                    f"{relative}: ambiguous mechanism site `{semantic_site}` for "
                    f"{prior[0]}#{prior[1]} and {spec}#{mechanism}"
                )
            mechanism_sites[semantic_site] = (spec, mechanism)
            binding = f"cpp-symbol:{semantic_site}"
            manifest["mechanism_implementations"].append(
                {
                    **common,
                    "site": semantic_site,
                    "mechanism": mechanism,
                    "binding": binding,
                }
            )
            manifest["artifacts"].append(
                {"id": binding, "kind": "cpp-symbol", "file": relative}
            )
        else:
            raise ValueError(f"{relative}: unknown annotation kind `{kind}`")
    return manifest


def emit(inputs: list[Path], root: Path, compiler: str, includes: list[Path]) -> dict[str, list[dict[str, object]]]:
    manifest = empty_manifest()
    try:
        semantic_root = root.resolve(strict=True)
    except OSError as error:
        raise ValueError(f"{root}: cannot resolve --root: {error}") from error
    if not semantic_root.is_dir():
        raise ValueError(f"{root}: --root must be a directory")
    files: list[Path] = []
    for item in inputs:
        try:
            selected = item.resolve(strict=True)
            selected.relative_to(semantic_root)
        except OSError as error:
            raise ValueError(f"{item}: cannot resolve input: {error}") from error
        except ValueError as error:
            raise ValueError(f"{item}: input is outside --root") from error
        if selected.is_dir():
            files.extend(selected.rglob("*.cpp"))
        elif selected.suffix == ".cpp":
            files.append(selected)
        else:
            raise ValueError(f"{item}: C++ input must be a .cpp file or directory")
    canonical: set[Path] = set()
    for path in files:
        resolved = path.resolve(strict=True)
        try:
            resolved.relative_to(semantic_root)
        except ValueError as error:
            raise ValueError(f"{path}: input is outside --root") from error
        canonical.add(resolved)
    for path in sorted(canonical):
        partial = scan(path, semantic_root, compiler, includes)
        for key, values in partial.items():
            manifest[key].extend(values)
    mechanism_sites: dict[str, tuple[str, str, str]] = {}
    for implementation in manifest["mechanism_implementations"]:
        site = str(implementation["site"])
        target = (str(implementation["spec"]), str(implementation["mechanism"]))
        file = str(implementation["file"])
        prior = mechanism_sites.get(site)
        if prior is not None:
            raise ValueError(
                f"{file}: ambiguous mechanism site `{site}` for "
                f"{prior[0]}#{prior[1]} in {prior[2]} and {target[0]}#{target[1]}"
            )
        mechanism_sites[site] = (*target, file)
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
