"""Native tool result interpretation with explicit coverage limitations."""
import json

def normalize(tool, native, *, execution_failed=False):
    findings, inconclusive = [], []
    if execution_failed:
        inconclusive.append('tool execution failed, timed out or did not produce a valid complete native report')
    if tool == 'stryker-net':
        files = native.get('files')
        if not isinstance(files, dict) or not files:
            return 'inconclusive', [], inconclusive + ['mutation discovery missing or empty'], {}
        counts = {}
        for file, account in files.items():
            for mutant in account.get('mutants', []):
                status = mutant.get('status', 'Unknown')
                counts[status] = counts.get(status, 0) + 1
                identity = f"{file}:{mutant.get('id', '?')}"
                if status == 'Survived':
                    findings.append(f'{identity}: survived mutation; assess equivalence and relevance before rejecting review')
                elif status == 'NoCoverage':
                    findings.append(f'{identity}: no selected Check coverage')
                elif status not in ('Killed', 'Ignored'):
                    inconclusive.append(f'{identity}: {status}')
        if not counts or set(counts) <= {'Ignored'}:
            inconclusive.append('no executed assessable mutations')
    else:
        counts = {}
        for finding in native.get('results', []):
            findings.append(f"{finding.get('check_id', '?')}:{finding.get('path', '?')}:{finding.get('start', {}).get('line', '?')}: {finding.get('extra', {}).get('message', '')}")
        for error in native.get('errors', []):
            inconclusive.append(json.dumps(error, sort_keys=True))
        paths = native.get('paths', {})
        if not paths.get('scanned'):
            inconclusive.append('analysis scanned no source files')
        if paths.get('skipped'):
            inconclusive.append('analysis skipped requested source paths')
    outcome = 'findings' if findings else 'inconclusive' if inconclusive else 'clean'
    return outcome, findings, inconclusive, counts

