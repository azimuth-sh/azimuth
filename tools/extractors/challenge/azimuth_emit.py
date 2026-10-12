#!/usr/bin/env python3
"""Extract explicit package implementation links from structural sources."""
import argparse
import ast
import hashlib
import json
from pathlib import Path
import re
import sys


def emit(root, paths):
    entries = []
    seen = set()
    for path in sorted(paths):
        path = path.resolve(strict=True)
        relative = path.relative_to(root.resolve(strict=True)).as_posix()
        source = path.read_text()
        declarations = []
        if path.suffix == '.py':
            modules = re.findall(r'^# azimuth.challenge: module ([A-Za-z_][A-Za-z0-9_.]*)\s*$', source, re.M)
            tree = ast.parse(source, filename=relative)
            imported = any(isinstance(statement, ast.ImportFrom) and statement.module == 'azimuth_challenge.annotations' and any(alias.name == 'implements_challenger' and alias.asname in (None, 'implements_challenger') for alias in statement.names) for statement in tree.body)
            for node in ast.walk(tree):
                if not isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    continue
                for decorator in node.decorator_list:
                    if not isinstance(decorator, ast.Call) or not isinstance(decorator.func, ast.Name) or decorator.func.id != 'implements_challenger':
                        continue
                    if not imported:
                        raise ValueError(f'{relative}:{node.lineno}: implements_challenger requires package import')
                    if len(modules) != 1 or len(decorator.args) != 1:
                        raise ValueError(f'{relative}: tagged callable requires one module declaration and one Challenger identity')
                    identity = ast.literal_eval(decorator.args[0])
                    options = {keyword.arg: ast.literal_eval(keyword.value) for keyword in decorator.keywords}
                    if set(options) - {'configuration_files'}:
                        raise ValueError(f'{relative}: unsupported Challenger decorator option')
                    declarations.append((identity, f'python:{modules[0]}.{node.name}', options.get('configuration_files', [])))
        markers = re.findall(r'^\s*# azimuth.challenge: implements-challenger ([a-z0-9][a-z0-9-]*)\s*$', source, re.M)
        if markers:
            sites = re.findall(r'^\s*# azimuth.challenge: site ([A-Za-z0-9_.:-]+)\s*$', source, re.M)
            inputs = re.findall(r'^\s*# azimuth.challenge: input ([^\s]+)\s*$', source, re.M)
            if len(sites) != 1:
                raise ValueError(f'{relative}: require one semantic # azimuth.challenge: site <identity>')
            declarations.extend((identity, sites[0], inputs) for identity in markers)
        for challenger, site, inputs in declarations:
            if not isinstance(challenger, str) or not re.fullmatch('[a-z0-9][a-z0-9-]*', challenger):
                raise ValueError(f'{relative}: invalid Challenger identity')
            declared = []
            for name in inputs:
                if Path(name).is_absolute() or '\\' in name or any(part in ('', '.', '..') for part in name.split('/')):
                    raise ValueError(f'{relative}: input must be normalized repository-relative path')
                current = root
                for segment in name.split('/'):
                    current = current / segment
                    if current.is_symlink():
                        raise ValueError(f'{relative}: symlink input is unsupported')
                input_path = root / name
                input_path.resolve(strict=True).relative_to(root.resolve(strict=True))
                if input_path.is_symlink():
                    raise ValueError(f'{relative}: symlink input is unsupported')
                declared.append({'file': name, 'fingerprint': 'sha256:' + hashlib.sha256(input_path.read_bytes()).hexdigest()})
            key = (challenger, site)
            if key in seen:
                raise ValueError(f'{relative}: duplicate semantic implementation {key}')
            seen.add(key)
            entries.append({'package':'azimuth.challenge','contract':'challenger-implementation','entity':challenger,
                            'site': site, 'file':relative,'lang':'python' if path.suffix == '.py' else 'structural',
                            'source_fingerprint':'sha256:' + hashlib.sha256(path.read_bytes()).hexdigest(),
                            'inputs': sorted(declared, key=lambda item:item['file'])})
    return {'extensions':entries}


def main():
    p=argparse.ArgumentParser()
    p.add_argument('--root',required=True)
    p.add_argument('--output',required=True)
    p.add_argument('inputs',nargs='+')
    a=p.parse_args()
    try:
        result=emit(Path(a.root),[Path(path) for path in a.inputs])
        Path(a.output).write_text(json.dumps(result,indent=2)+'\n')
    except (ValueError,OSError) as error:
        print(f'azimuth.challenge emitter: {error}',file=sys.stderr)
        return 2
    return 0
if __name__=='__main__':
    raise SystemExit(main())
