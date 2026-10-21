#!/usr/bin/python3
"""Import native machine Verification Challenge reports through the bounded Azimuth adapter host."""
import hashlib
import json
from pathlib import Path
import sys
import types


def digest(data):
    return 'sha256:' + hashlib.sha256(data).hexdigest()


def unique(pairs):
    result={}
    for key,value in pairs:
        if key in result:
            raise ValueError(f'duplicate field {key}')
        result[key]=value
    return result


def registration_identity(executable_data, resources, fingerprint):
    identity={'protocol_version':1,'id':'azimuth-challenge','provider_family':'azimuth/challenge','adapter_version':'0.1.0-alpha.6',
              'build':'machine-challenge-import','content':{'executable_digest':digest(executable_data),
              'resources':[{'id':item['id'],'digest':item['digest']} for item in resources]}}
    adapter_fp=fingerprint(dict(format='azimuth-adapter-fingerprint',version=1,**identity))
    capability={'id':'native','classes':['challenge.import'],'challenge_forms':['mutation','source-analysis'],'semantic_settings':{}}
    capability['fingerprint']=fingerprint(dict(format='azimuth-adapter-capability-fingerprint',version=1,adapter_fingerprint=adapter_fp,**capability))
    descriptor=dict(format='azimuth-adapter-description',version=1,**identity,adapter_fingerprint=adapter_fp,capabilities=[capability])
    descriptor['descriptor_fingerprint']=fingerprint({'format':'azimuth-adapter-descriptor-fingerprint','version':1,'descriptor':descriptor})
    expected_configuration=fingerprint({'format':'azimuth-adapter-configuration-fingerprint','version':1,
        'adapter_fingerprint':adapter_fp,'descriptor_fingerprint':descriptor['descriptor_fingerprint'],
        'semantic_settings':{'native_contract':'azimuth-challenge-native-report-v1'},'environment':{'literals':{}},
        'limits':{'timeout_ms':30000,'stdout_bytes':16777216,'stderr_bytes':1048576},'capabilities':[capability]})
    return identity, capability, descriptor, expected_configuration


def main():
    request=json.loads(sys.stdin.read(),object_pairs_hook=unique)
    if request.get('format')!='azimuth-adapter-request' or request.get('version')!=1 or request.get('operation') not in ('describe','import'):
        raise ValueError('unsupported machine Challenge adapter request')
    configuration=request['configuration']
    resources=configuration['resources']
    if [item['id'] for item in resources] != ['native-normalizer', 'run-protocol']:
        raise ValueError('adapter requires exactly pinned run-protocol helper')
    data=Path(resources[1]['locator']).read_bytes()
    if digest(data)!=resources[1]['digest']:
        raise ValueError('helper digest mismatch')
    helper=types.ModuleType('challenge_protocol')
    exec(compile(data,resources[1]['locator'],'exec'),helper.__dict__)
    native=types.ModuleType('challenge_native')
    normalizer_data=Path(resources[0]['locator']).read_bytes()
    if digest(normalizer_data)!=resources[0]['digest']:
        raise ValueError('native normalizer digest mismatch')
    exec(compile(normalizer_data,resources[0]['locator'],'exec'),native.__dict__)
    identity, capability, descriptor, expected_configuration = registration_identity(
        Path(__file__).read_bytes(), resources, helper.fingerprint)
    adapter_fp = descriptor['adapter_fingerprint']
    if configuration['fingerprint']!=expected_configuration or configuration['semantic_settings']!={'native_contract':'azimuth-challenge-native-report-v1'}:
        raise ValueError('unexpected adapter configuration')
    response={'format':'azimuth-adapter-response','version':1,'request_id':request['request_id'],'operation':request['operation'],'status':'ok','description':descriptor}
    if request['operation']=='describe':
        if request['adapter'] != {'id':'azimuth-challenge','configuration_fingerprint':expected_configuration}:
            raise ValueError('descriptor requested for different adapter')
        expected=helper.fingerprint({'format':'azimuth-adapter-request-fingerprint','version':1,'operation':'describe','adapter':request['adapter']})
    else:
        if request['predecessors'] or request['terminal_predecessor']:
            raise ValueError('native corrections require a new execution; prior immutable Runs remain unchanged')
        launch=request['launch_plan']
        inputs=request['inputs']
        if len(inputs)!=1 or inputs[0]['id']!='native-account':
            raise ValueError('import requires exactly one native-account input')
        item=inputs[0]
        data=Path(item['locator']).read_bytes()
        if digest(data)!=item['digest'] or len(data)!=item['size_bytes']:
            raise ValueError('native input digest mismatch')
        account=json.loads(data,object_pairs_hook=unique)
        if (account.get('format')!='azimuth-challenge-native-account' or account.get('version')!=2
                or account['verification_target']!=launch['verification_target']):
            raise ValueError('native account or Verification Target mismatch')
        if (len(launch.get('adapters', [])) != 1
                or launch['adapters'][0]['id'] != 'azimuth-challenge'
                or launch['adapters'][0]['configuration_fingerprint'] != expected_configuration):
            raise ValueError('launch configuration mismatch')
        for challenge in launch['plan']['challenges']:
            for unit in challenge['units']:
                report=account['reports'][challenge['id']][unit['id']]
                if (report.get('format')!='azimuth-challenge-native-report' or report.get('version')!=2
                        or report.get('verification_challenge')!=challenge['verification_challenge']['id']):
                    raise ValueError('native report identity mismatch')
                if helper.fingerprint({key:value for key,value in report.items() if key!='fingerprint'})!=report.get('fingerprint'):
                    raise ValueError('native report fingerprint mismatch')
                if report.get('verification_target') != launch['verification_target']:
                    raise ValueError('native report Verification Target differs from plan')
                if report['scope_fingerprint']!=unit['parameters']['scope_fingerprint']:
                    raise ValueError('native report differs from planned reviewed scope')
                expected_tool='stryker-net' if challenge['verification_challenge']['id'] and next(route for route in launch['routes'] if route['selection']['id']==challenge['id'])['capability']['challenge_form']=='mutation' else 'semgrep'
                if report['tool']!=expected_tool:
                    raise ValueError('native tool differs from configured Challenge form')
                outcome,objections,gaps,_=native.normalize(report['tool'],json.loads(report['native_report_json']),execution_failed=report['timed_out'] or report['tool_exit']!=0)
                if (outcome,objections,gaps)!=(report['outcome'],report['objections'],report['inconclusive_reasons']):
                    raise ValueError('native report conclusion differs from independent normalization')
                attempts=account['units_by_challenge'][challenge['id']]
                measured=next(item for item in attempts if item['id']==unit['id'])
                if len(measured['attempts'])!=1 or measured['attempts'][0]['outcome']!=outcome or measured['attempts'][0].get('objections',[])!=objections:
                    raise ValueError('native attempt conclusion differs from tool report')
        expected=helper.fingerprint({'format':'azimuth-adapter-request-fingerprint','version':1,'operation':'import',
            'launch_fingerprint':launch['fingerprint'],'inputs':[{key:item[key] for key in ('id','digest','size_bytes')}],'predecessors':[]})
        bundle=helper.build_challenge_bundle(launch,source=account['source'],normalizer={'id':'adapter/azimuth-challenge','version':identity['adapter_version'],'build_fingerprint':adapter_fp},
            actual_context=account['actual_execution_context'],
            **{key:account[key] for key in ('started_at_ms','finished_at_ms','generated_at_ms','activities','units_by_challenge','artifacts','diagnostics')},
            import_inputs=[{key:item[key] for key in ('id','digest','size_bytes')}])
        response.update(launch_fingerprint=launch['fingerprint'],bundle=bundle)
    if request['request_id']!=expected:
        raise ValueError('request fingerprint mismatch')
    print(json.dumps(response))
    return 0
if __name__=='__main__':
    try:
        raise SystemExit(main())
    except (ValueError,KeyError,OSError) as error:
        print(f'azimuth.challenge adapter: {error}',file=sys.stderr)
        raise SystemExit(2)
