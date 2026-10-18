"""Explicit native import boundary for collector observations."""
import hashlib
import json
from pathlib import Path
import sys
import types

LIMITS = {'timeout_ms': 30000, 'stdout_bytes': 16777216, 'stderr_bytes': 1048576}


def digest(content):
    return 'sha256:' + hashlib.sha256(content).hexdigest()


def description(adapter_id, helper, executable, resources):
    if adapter_id not in ('probe-results', 'prometheus-rules'):
        raise ValueError('Unknown collector adapter')
    identity = dict(protocol_version=1, id=adapter_id, provider_family='azimuth/collector',
        adapter_version='1', build='collector-observations',
        content={'executable_digest': digest(executable), 'resources': resources})
    fingerprint = helper.fingerprint(dict(format='azimuth-adapter-fingerprint', version=1, **identity))
    capability = dict(id='checks', classes=['check.import'], challenge_forms=[], semantic_settings={})
    capability['fingerprint'] = helper.fingerprint(dict(format='azimuth-adapter-capability-fingerprint',
        version=1, adapter_fingerprint=fingerprint, **capability))
    result = dict(format='azimuth-adapter-description', version=1, **identity,
        adapter_fingerprint=fingerprint, capabilities=[capability])
    result['descriptor_fingerprint'] = helper.fingerprint(dict(format='azimuth-adapter-descriptor-fingerprint',
        version=1, descriptor=result))
    return result


def load_resource(resource):
    content = Path(resource['locator']).read_bytes()
    if digest(content) != resource['digest']:
        raise ValueError('Resource content mismatch')
    return content


def main():
    request = json.load(sys.stdin)
    if request['format'] != 'azimuth-adapter-request' or request['version'] != 1:
        raise ValueError('Unsupported request')
    config = request['configuration']
    resources = config['resources']
    if [item['id'] for item in resources] != ['collector', 'run-protocol']:
        raise ValueError('Expected pinned collector and Run protocol resources')
    collector_bytes, protocol_bytes = map(load_resource, resources)
    helper = types.ModuleType('azimuth_run')
    exec(compile(protocol_bytes, resources[1]['locator'], 'exec'), helper.__dict__)
    sys.modules['azimuth_run'] = helper
    collector = types.ModuleType('collector')
    exec(compile(collector_bytes, resources[0]['locator'], 'exec'), collector.__dict__)
    adapter_id = config['semantic_settings']['adapter']
    descriptor = description(adapter_id, helper, Path(__file__).read_bytes(),
        [{key: item[key] for key in ('id', 'digest')} for item in resources])
    expected_config = helper.fingerprint(dict(format='azimuth-adapter-configuration-fingerprint', version=1,
        adapter_fingerprint=descriptor['adapter_fingerprint'], descriptor_fingerprint=descriptor['descriptor_fingerprint'],
        semantic_settings={'adapter': adapter_id}, environment={'literals': {}}, limits=LIMITS,
        capabilities=descriptor['capabilities']))
    if config['fingerprint'] != expected_config:
        raise ValueError('Adapter configuration mismatch')
    if config['capabilities'] != [{'address': adapter_id + '/checks',
            'fingerprint': descriptor['capabilities'][0]['fingerprint'], 'semantic_settings': {}}]:
        raise ValueError('Capability mismatch')
    operation = request['operation']
    response = dict(format='azimuth-adapter-response', version=1, request_id=request['request_id'],
        operation=operation, status='ok', description=descriptor)
    if operation == 'describe':
        if request['adapter'] != {'id': adapter_id, 'configuration_fingerprint': expected_config}:
            raise ValueError('Adapter identity mismatch')
        expected_request = helper.fingerprint(dict(format='azimuth-adapter-request-fingerprint', version=1,
            operation=operation, adapter=request['adapter']))
    elif operation == 'import':
        launch = request['launch_plan']
        if request['predecessors'] or request['terminal_predecessor'] is not None:
            raise ValueError('Each collector execution requires a separate initial Run')
        if len(launch['adapters']) != 1:
            raise ValueError('Expected one collector adapter')
        selected_adapter = launch['adapters'][0]
        if any(selected_adapter[key] != descriptor[key] for key in
                ('id', 'adapter_version', 'adapter_fingerprint', 'descriptor_fingerprint')):
            raise ValueError('Launch adapter mismatch')
        if selected_adapter['configuration_fingerprint'] != expected_config:
            raise ValueError('Launch configuration mismatch')
        if [item['id'] for item in request['inputs']] != ['native-report']:
            raise ValueError('Exactly one native-report input is required')
        item = request['inputs'][0]
        raw = load_resource(item)
        if len(raw) != item['size_bytes']:
            raise ValueError('Input size mismatch')
        account = json.loads(raw)
        binding = account['binding']
        if binding['launch'] != launch:
            raise ValueError('Observation launch mismatch')
        collector.validate_binding(binding, digest(collector_bytes), digest(protocol_bytes))
        bundle = collector.native_bundle(binding, account['record'],
            import_inputs=[{key: item[key] for key in ('id', 'digest', 'size_bytes')}])
        expected_request = helper.fingerprint(dict(format='azimuth-adapter-request-fingerprint', version=1,
            operation=operation, launch_fingerprint=launch['fingerprint'],
            inputs=[{key: item[key] for key in ('id', 'digest', 'size_bytes')}], predecessors=[]))
        response.update(launch_fingerprint=launch['fingerprint'], bundle=bundle)
    else:
        raise ValueError('Unsupported operation')
    if request['request_id'] != expected_request:
        raise ValueError('Request fingerprint mismatch')
    print(helper.canonical_json(response))


if __name__ == '__main__':
    try:
        main()
    except (KeyError, TypeError, ValueError, OSError) as error:
        print('Collector adapter rejected: ' + str(error), file=sys.stderr)
        sys.exit(1)
