"""Prepare an explicit reviewed collector configuration without contacting ASS."""
import argparse
import json
import os
from pathlib import Path
import sys
import tempfile

sys.path.insert(0, str(Path(__file__).resolve().parent))
from collector import unique, validate_binding
from contexts import validate_sources


def prepare(current, reviewed):
    if set(reviewed) != {'sources', 'contracts'} or not isinstance(reviewed['sources'], dict):
        raise ValueError('Reviewed input requires exact sources map and full contracts list')
    contracts = {item['fingerprint']: item for item in reviewed['contracts']}
    if len(contracts) != len(reviewed['contracts']):
        raise ValueError('Duplicate reviewed contract')
    result = {key: current[key] for key in ('assurance_url', 'namespaces', 'projects')}
    result.update(format='azimuth-collector', version=2, interval_seconds=current.get('interval_seconds', 30), max_pending=current.get('max_pending', 10000), sources=[], context_sources=[])
    if set(reviewed['sources']) != {source['id'] for source in current['sources']}:
        raise ValueError('Explicit stable source mapping must cover exactly current sources')
    for source in current['sources']:
        selected = reviewed['sources'][source['id']]
        if set(selected) - {'id', 'environment', 'definition', 'identity_labels'} or not {'id', 'environment', 'definition'} <= set(selected):
            raise ValueError('Reviewed source requires explicit stable id, environment and Verification-Target-free definition')
        definition = selected['definition']
        original = source.get('definition', source.get('launch'))
        if original is None:
            raise ValueError('Current Check definition unavailable')
        expected = {key: original[key] for key in ('plan', 'adapters', 'routes')}
        expected['plan'] = {key: value for key, value in expected['plan'].items() if key != 'fingerprint'}
        if definition != expected:
            raise ValueError('Reviewed definition differs from current exact Check/provider identity')
        route = ('namespace', 'service', 'port', 'path') if source['adapter'] == 'probe-results' else ('url', 'rule', 'rule_labels', 'target_labels')
        item = {key: source[key] for key in ('project', 'adapter') + route}
        for key in ('interval_seconds', 'max_age_seconds'):
            if key in source:
                item[key] = source[key]
        item.update(selected)
        item['allowed_outcomes'] = ['violated', 'inconclusive']
        validate_binding(item)
        result['sources'].append(item)
    identities = [item['id'] for item in result['sources']]
    if len(identities) != len(set(identities)):
        raise ValueError('Stable source identities must be unique')
    for source in current.get('context_sources', []):
        item = dict(source)
        references = [value if isinstance(value, str) else value['fingerprint'] for value in source['contracts']]
        if any(reference not in contracts for reference in references):
            raise ValueError('Missing full reviewed context contract')
        item['contracts'] = [contracts[reference] for reference in references]
        result['context_sources'].append(item)
    validate_sources(result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--current', required=True, type=Path)
    parser.add_argument('--reviewed', required=True, type=Path)
    parser.add_argument('--out', required=True, type=Path)
    args = parser.parse_args()
    read = lambda path: json.loads(path.read_text(), object_pairs_hook=unique)
    output = prepare(read(args.current), read(args.reviewed))
    if args.out.exists():
        raise ValueError('Output must be a fresh path')
    with tempfile.NamedTemporaryFile(mode='w', dir=args.out.parent, delete=False) as temporary:
        try:
            json.dump(output, temporary, sort_keys=True, indent=2)
            temporary.write('\n'); temporary.flush(); os.fsync(temporary.fileno())
            os.link(temporary.name, args.out)
        finally:
            os.unlink(temporary.name)


if __name__ == '__main__':
    main()
