"""Resolve semantic implementation identities without repository-wide fallback."""
import argparse
import hashlib
import json
import re
from pathlib import Path
import sys
from azimuth_run.protocol import fingerprint


def read(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError(f"duplicate JSON field {key}")
            result[key] = value
        return result
    return json.loads(Path(path).read_text(), object_pairs_hook=unique, parse_constant=lambda value: (_ for _ in ()).throw(ValueError(f"invalid JSON numeric value {value}")))


def located(root, relative):
    if not isinstance(relative, str) or not relative or '\\' in relative or any(part in ('', '.', '..') for part in relative.split('/')):
        raise ValueError(f"non-normal source locator: {relative}")
    path = root / relative
    if path.is_symlink() or any(parent.is_symlink() for parent in path.parents if parent != root.parent):
        raise ValueError(f"symlink source locator: {relative}")
    path.resolve(strict=True).relative_to(root.resolve(strict=True))
    if not path.is_file():
        raise ValueError(f"missing source file: {relative}")
    return path


def resolve(request, manifests, roots):
    allowed = {'format', 'version', 'verification_challenge', 'requirements', 'mechanisms', 'checks', 'repository', 'project', 'test_project', 'test_filter', 'tool', 'rules', 'areas'}
    if set(request) - allowed or request.get('format') != 'azimuth-challenge-scope-request' or request.get('version') != 2:
        raise ValueError('expected strict azimuth-challenge-scope-request version 2')
    for field in ('requirements', 'mechanisms', 'checks', 'rules', 'areas'):
        values = request.get(field, [])
        if not isinstance(values, list) or any(not isinstance(value, str) or not value for value in values) or len(values) != len(set(values)):
            raise ValueError(f'{field} requires unique nonempty string values')
    for field in ('verification_challenge', 'repository'):
        if not isinstance(request.get(field), str) or not request[field]:
            raise ValueError(f'{field} requires a nonempty identity')
    if request.get('tool') not in ('stryker-net', 'semgrep'):
        raise ValueError('tool must be stryker-net or semgrep')
    repository = request['repository']
    if repository not in roots:
        raise ValueError('selected repository has no local binding')
    root = roots[repository]
    selected = []
    exclusions = []
    unresolved = []
    seen = {}
    for relation, key, ids in [('realizes', 'requirement', [] if request.get('mechanisms') else request.get('requirements', [])), ('mechanism_implementations', 'mechanism', request.get('mechanisms', [])), ('check_implementations', 'check', request.get('checks', []))]:
        for identity in ids:
            matches = [item for manifest in manifests for item in manifest.get(relation, []) if item.get(key) == identity]
            if not matches:
                unresolved.append({'kind': key, 'id': identity, 'reason': 'no emitter source mapping'})
            for item in matches:
                source = item.get('source') or (item if item.get('area') else None)
                item_repository = item.get('_repository', repository)
                if item_repository != repository:
                    exclusions.append({'kind': key, 'entity': identity, 'site': item['site'], 'file': item['file'], 'reason': 'outside declared repository scope'})
                    continue
                if request.get('areas'):
                    if not source or not source.get('area'):
                        unresolved.append({'kind': key, 'id': identity, 'site': item['site'], 'reason': 'Area selector requires qualified source identity'})
                        continue
                    if source['area'] not in request['areas']:
                        exclusions.append({'kind': key, 'entity': identity, 'site': item['site'], 'file': item['file'], 'reason': 'outside declared Area scope'})
                        continue
                path = located(root, item['file'])
                semantic = item['site']
                item_key = (relation, identity, semantic)
                if item_key in seen and seen[item_key] != item['file']:
                    raise ValueError(f'ambiguous source mapping for {semantic}')
                seen[item_key] = item['file']
                if not item.get('source_fingerprint', '').startswith('sha256:'):
                    raise ValueError(f'missing emitted site fingerprint: {semantic}')
                selected.append({'relationship': key, 'entity': identity, 'site': semantic,
                    'file': item['file'], 'source_fingerprint': item['source_fingerprint'],
                    'file_fingerprint': 'sha256:' + hashlib.sha256(path.read_bytes()).hexdigest(),
                    'role': 'check' if key == 'check' else 'target', 'repository': repository,
                    'reason': f'{key} implementation selected by declaration', 'precision': 'file'})
    targets = sorted({item['file'] for item in selected if item['role'] == 'target'})
    if not targets:
        unresolved.append({'reason': 'empty target scope'})
    checks = sorted({item['site'] for item in selected if item['role'] == 'check'})
    build = []
    for key in ('project', 'test_project'):
        if request.get(key):
            located(root, request[key])
            build.append(request[key])
    rules = request.get('rules', [])
    for rule in rules:
        located(root, rule)
    config = {}
    check_filter = None
    check_filter_limitations = []
    expansion = [{'file': file, 'reason': 'emitter exposes source site but no exact source region; entire containing file is targeted'} for file in targets]
    if request['tool'] == 'stryker-net':
        if not request.get('project') or not request.get('test_project') or not checks:
            raise ValueError('mutation requires explicit project, test project and selected Check implementation sites')
        check_sites = [item for item in selected if item['role'] == 'check']
        if any(not item['file'].endswith('.cs') or not re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*(?:[.+][A-Za-z_][A-Za-z0-9_]*)+', item['site']) for item in check_sites):
            unresolved.append({'reason': 'exact .NET test names cannot be derived from selected Check source sites'})
        else:
            exact_filter = '|'.join('FullyQualifiedName=' + site for site in checks)
            check_filter = request.get('test_filter') or exact_filter
            if check_filter != exact_filter:
                unresolved.append({'reason': 'provided Check filter differs from resolved exact selection; omissions and expansions cannot be established safely'})
                check_filter_limitations.append('The supplied filter may omit selected Checks or execute additional Checks; filter equality is not established.')
        project_dir = (root / request['project']).parent
        mutate = []
        for file in targets:
            relative = (root / file).relative_to(project_dir).as_posix()
            if any(char in relative for char in '*?[]{}!'):
                raise ValueError('tool glob metacharacters in exact source locator are unsupported')
            mutate.append(relative)
        config = {'stryker-config': {'project': Path(request['project']).name, 'mutate': mutate,
                   'test-case-filter': check_filter, 'test-runner': 'vstest', 'concurrency': 1, 'reporters': ['json'], 'thresholds': {'high': 100, 'low': 0, 'break': 0}}}
    else:
        if not rules:
            raise ValueError('source analysis requires explicit local custom rules')
        config = {'targets': targets, 'rules': rules, 'arguments': ['scan', '--json', '--metrics=off', '--disable-version-check']}
    report = {'format': 'azimuth-challenge-source-scope', 'version': 2, 'verification_challenge': request['verification_challenge'],
              'repository': repository, 'request': request, 'status': 'incomplete' if unresolved else 'complete',
              'sites': sorted(selected, key=lambda item: (item['role'], item['file'], item['site'], item['entity'])),
              'target_scope': targets, 'build_scope': build, 'analysis_scope': targets if request['tool'] == 'semgrep' else [],
              'check_scope': checks, 'check_filter': check_filter, 'check_filter_limitations': check_filter_limitations, 'exclusions': exclusions, 'unresolved': unresolved, 'expansions': expansion,
              'target_selection': 'mechanisms' if request.get('mechanisms') else 'requirements',
              'limitations': ['Target file completeness requires independent review of relevant call paths.',
                              'File targeting may include unrelated sites within selected files.',
                              'Build tool project references may expand compilation dependencies; these are not mutation targets.'] + (['Semgrep local rules provide intrafile analysis only; no cross-language or interfile assurance is inferred.'] if request['tool'] == 'semgrep' else ['Exact method filters require a test runner supporting the dotnet test FullyQualifiedName filter; unsupported runner filtering cannot establish selected Check execution.']),
              'tool_configuration': config}
    report['fingerprint'] = fingerprint(report)
    return report


def main():
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest='command', required=True)
    scope = sub.add_parser('scope')
    scope.add_argument('--manifest', action='append', required=True)
    scope.add_argument('--root', action='append', required=True)
    scope.add_argument('--request', required=True)
    scope.add_argument('--out', required=True)
    args = parser.parse_args()
    try:
        roots = {}
        for value in args.root:
            identity, path = value.split('=', 1)
            if identity in roots:
                raise ValueError('duplicate repository root')
            roots[identity] = Path(path).resolve(strict=True)
        request = read(args.request)
        manifests = []
        for value in args.manifest:
            if '=' in value:
                repository, path = value.split('=', 1)
            else:
                repository, path = request['repository'], value
            manifest = read(path)
            for relation in ('realizes', 'mechanism_implementations', 'check_implementations'):
                for item in manifest.get(relation, []):
                    item['_repository'] = repository
            manifests.append(manifest)
        report = resolve(request, manifests, roots)
        output = Path(args.out)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(json.dumps(report, indent=2) + '\n')
        return 1 if report['status'] != 'complete' else 0
    except (ValueError, OSError, KeyError) as error:
        print(f'azimuth.challenge: {error}', file=sys.stderr)
        return 2
