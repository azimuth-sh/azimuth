"""Register immutable collector observation import adapters for native planning."""
import argparse
import importlib.util
from pathlib import Path
import json
import types


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--adapter', required=True, choices=['probe-results', 'prometheus-rules'])
    parser.add_argument('--python', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    python = args.python.resolve(strict=True)
    if ' ' in str(python):
        raise ValueError('Interpreter path must not contain spaces')
    directory = Path(__file__).resolve().parent
    protocol = directory.parents[1] / 'packages/python/azimuth_run/protocol.py'
    helper = types.ModuleType('protocol')
    exec(compile(protocol.read_bytes(), str(protocol), 'exec'), helper.__dict__)
    spec = importlib.util.spec_from_file_location('adapter', directory/'adapter.py')
    adapter = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(adapter)
    executable = ('#!' + str(python) + '\n').encode() + (directory/'adapter.py').read_bytes()
    resources = [{'id': 'collector', 'digest': adapter.digest((directory/'collector.py').read_bytes())},
                 {'id': 'run-protocol', 'digest': adapter.digest(protocol.read_bytes())}]
    description = adapter.description(args.adapter, helper, executable, resources)
    entry = {key: description[key] for key in ('protocol_version', 'id', 'provider_family', 'adapter_version', 'build',
        'adapter_fingerprint', 'descriptor_fingerprint', 'capabilities')}
    entry.update(semantic_settings={'adapter': args.adapter}, environment={'literals': {}}, limits=adapter.LIMITS)
    entry['configuration_fingerprint'] = helper.fingerprint(dict(format='azimuth-adapter-configuration-fingerprint', version=1,
        adapter_fingerprint=entry['adapter_fingerprint'], descriptor_fingerprint=entry['descriptor_fingerprint'],
        semantic_settings=entry['semantic_settings'], environment=entry['environment'], limits=entry['limits'], capabilities=entry['capabilities']))
    output = args.output.resolve()
    if output.exists():
        raise ValueError('Choose a fresh registration output')
    registration = output.parent / (args.adapter + '-' + entry['adapter_fingerprint'][7:])
    registration.mkdir(parents=True, exist_ok=False)
    for name, content in [('adapter', executable), ('collector.py', (directory/'collector.py').read_bytes()), ('protocol.py', protocol.read_bytes())]:
        target = registration/name
        target.write_bytes(content)
        target.chmod(0o500 if name == 'adapter' else 0o400)
    entry['content'] = {'executable': {'locator': (registration/'adapter').relative_to(output.parent).as_posix(), 'digest': adapter.digest(executable)},
        'resources': [dict(resources[0], locator=(registration/'collector.py').relative_to(output.parent).as_posix()),
                      dict(resources[1], locator=(registration/'protocol.py').relative_to(output.parent).as_posix())]}
    output.write_text(json.dumps({'format': 'azimuth-adapter-configuration', 'version': 1, 'adapters': [entry]}, indent=2)+'\n')
    output.chmod(0o400)
    print(output)


if __name__ == '__main__':
    main()
