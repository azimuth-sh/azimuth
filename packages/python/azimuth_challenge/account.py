"""Build a native import account from preserved tool reports, never an exit code alone."""
import argparse
import hashlib
import json
from pathlib import Path
import time
from .scope import read
from .native import normalize
from azimuth_run.protocol import fingerprint


def account(launch, reports):
    if launch.get('operation') != 'import' or launch['plan']['checks']:
        raise ValueError('native Challenge account requires a Challenge-only import plan')
    activities,artifacts,diagnostics,units,raw_reports=[],[],[],{},{}
    starts,finishes=[],[]
    for challenge in launch['plan']['challenges']:
        verification_challenge=challenge['verification_challenge']['id']
        units[challenge['id']]=[]
        raw_reports[challenge['id']]={}
        for unit in challenge['units']:
            key=(verification_challenge,unit['id'])
            if key not in reports:
                raise ValueError('every planned Challenge unit needs an explicit native report')
            path=reports.pop(key)
            raw=path.read_bytes()
            report=read(path)
            if report.get('format')!='azimuth-challenge-native-report' or report.get('version')!=2:
                raise ValueError('invalid native report')
            if fingerprint({key:value for key,value in report.items() if key!='fingerprint'})!=report.get('fingerprint'):
                raise ValueError('native report fingerprint mismatch')
            if (report['verification_target']!=launch['verification_target']
                    or report['verification_challenge']!=verification_challenge
                    or report['scope_fingerprint']!=unit['parameters']['scope_fingerprint']):
                raise ValueError('native report differs from planned Verification Target, Verification Challenge or scope')
            outcome,objections,gaps,_=normalize(report['tool'],json.loads(report['native_report_json']),execution_failed=report['timed_out'] or report['tool_exit']!=0)
            if (outcome,objections,gaps)!=(report['outcome'],report['objections'],report['inconclusive_reasons']):
                raise ValueError('native report interpretation mismatch')
            suffix=hashlib.sha256((challenge['id']+unit['id']).encode()).hexdigest()
            activity_id='challenge-'+suffix
            artifact_id='report-'+suffix
            artifacts.append({'id':artifact_id,'kind':'native-challenge-report','media_type':'application/json',
                'digest':'sha256:'+hashlib.sha256(raw).hexdigest(),'size_bytes':len(raw),
                'locator':{'kind':'unavailable','value':'sha256:'+hashlib.sha256(raw).hexdigest()}})
            refs=[]
            for ordinal,reason in enumerate(gaps):
                diagnostic_id='gap-'+suffix+'-'+str(ordinal)
                diagnostics.append({'id':diagnostic_id,'class':'normalization','severity':'warning','code':'challenge-search-incomplete',
                    'message':reason,'scope':{'kind':'verification-challenge-execution','verification_challenge_fingerprint':challenge['verification_challenge']['fingerprint'],'target_fingerprint':challenge['target']['fingerprint']},
                    'artifacts':[artifact_id],'details':{}})
                refs.append(diagnostic_id)
            activities.append({'id':activity_id,'status':'completed','started_at_ms':report['started_at_ms'],
                'finished_at_ms':report['finished_at_ms'],'artifacts':[artifact_id],'diagnostics':refs,
                'attributes':{'tool':report['tool'],'scope_fingerprint':report['scope_fingerprint'],'capture_fingerprint':report['capture_fingerprint']}})
            units[challenge['id']].append({'id':unit['id'],'attempts':[{'ordinal':0,'activity':activity_id,'outcome':outcome,'objections':objections}]})
            raw_reports[challenge['id']][unit['id']]=report
            starts.append(report['started_at_ms'])
            finishes.append(report['finished_at_ms'])
    if reports or not starts:
        raise ValueError('native report selection differs from finite planned units')
    execution=fingerprint(raw_reports)
    return {'format':'azimuth-challenge-native-account','version':2,'verification_target':launch['verification_target'],
            'source':{'system':'azimuth-challenge','execution':execution},'started_at_ms':min(starts),'finished_at_ms':max(finishes),
            'generated_at_ms':max(int(time.time()*1000),max(finishes)),'actual_execution_context':launch['plan']['required_execution_context'],
            'activities':activities,'units_by_challenge':units,'reports':raw_reports,'artifacts':artifacts,'diagnostics':diagnostics}


def main():
    p=argparse.ArgumentParser()
    p.add_argument('--launch',required=True)
    p.add_argument('--report',action='append',required=True)
    p.add_argument('--out',required=True)
    a=p.parse_args()
    try:
        reports={}
        for argument in a.report:
            selector,path=argument.split('=',1)
            verification_challenge,unit=selector.split(':',1)
            if (verification_challenge,unit) in reports:
                raise ValueError('duplicate native report selector')
            reports[verification_challenge,unit]=Path(path)
        result=account(read(a.launch),reports)
        Path(a.out).write_text(json.dumps(result,indent=2)+'\n')
    except (ValueError,KeyError,OSError) as error:
        p.exit(2,f'azimuth.challenge account: {error}\n')
    return 0
if __name__=='__main__':
    raise SystemExit(main())
