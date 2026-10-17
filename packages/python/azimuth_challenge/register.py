"""Pin native import behavior independently from mutable development checkouts."""
import argparse
import json
from pathlib import Path
import sys
from azimuth_run.protocol import fingerprint
from .adapter import digest, registration_identity


def register(python, output):
    python = Path(python).resolve(strict=True)
    if not python.is_file() or ' ' in str(python):
        raise ValueError('select an exact Python executable without spaces')
    package = Path(__file__).parent
    adapter = (package / 'adapter.py').read_bytes()
    executable_data = ('#!' + str(python) + '\n').encode() + adapter.split(b'\n', 1)[1]
    resources = [('native-normalizer', (package / 'native.py').read_bytes()),
                 ('run-protocol', (package.parent / 'azimuth_run/protocol.py').read_bytes())]
    descriptions = [{'id': identity, 'digest': digest(data)} for identity, data in resources]
    identity, capability, descriptor, configuration = registration_identity(executable_data, descriptions, fingerprint)
    output = Path(output).resolve()
    if output.exists():
        raise ValueError('adapter configuration already exists; retain it and select a fresh output')
    output.parent.mkdir(parents=True, exist_ok=True)
    directory = output.parent / ('challenge-native-' + descriptor['adapter_fingerprint'][7:])
    directory.mkdir(exist_ok=False)
    executable = directory / 'adapter'
    executable.write_bytes(executable_data)
    executable.chmod(0o500)
    resource_entries = []
    for (name, data), description in zip(resources, descriptions):
        path = directory / (name + '.py')
        path.write_bytes(data)
        path.chmod(0o400)
        resource_entries.append(dict(description, locator=path.relative_to(output.parent).as_posix()))
    entry = dict(identity)
    entry['content'] = {'executable': {'locator': executable.relative_to(output.parent).as_posix(),
                                       'digest': digest(executable_data)}, 'resources': resource_entries}
    entry.update(semantic_settings={'native_contract': 'azimuth-challenge-native-report-v1'},
                 environment={'literals': {}},
                 limits={'timeout_ms': 30000, 'stdout_bytes': 16777216, 'stderr_bytes': 1048576},
                 capabilities=[capability], adapter_fingerprint=descriptor['adapter_fingerprint'],
                 descriptor_fingerprint=descriptor['descriptor_fingerprint'], configuration_fingerprint=configuration)
    with output.open('x') as target:
        json.dump({'format': 'azimuth-adapter-configuration', 'version': 1, 'adapters': [entry]}, target, indent=2)
        target.write('\n')
    output.chmod(0o400)
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--python', required=True)
    parser.add_argument('--out', required=True)
    args = parser.parse_args()
    try:
        print(register(args.python, args.out))
        return 0
    except (OSError, ValueError) as error:
        parser.exit(2, f'azimuth.challenge registration: {error}\n')


if __name__ == '__main__':
    raise SystemExit(main())
