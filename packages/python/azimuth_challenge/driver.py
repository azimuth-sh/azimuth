"""Execute configured tools in captured source copies and preserve native outcomes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
from .scope import read, located
from azimuth_run.protocol import fingerprint


from .native import normalize
from .capture import capture as capture_source


def execute(scope, root, executable, capture, environment, timeout, git_executable):
    if scope.get('status') != 'complete' or scope.get('format') != 'azimuth-challenge-source-scope':
        raise ValueError('execution requires complete prepared scope')
    supplied = scope.get('fingerprint')
    if fingerprint({key:value for key,value in scope.items() if key != 'fingerprint'}) != supplied:
        raise ValueError('scope fingerprint mismatch')
    if not Path(executable).is_absolute() or not Path(executable).is_file():
        raise ValueError('tool executable must be an explicit absolute file')
    if not isinstance(environment, dict) or any(not isinstance(key,str) or not isinstance(value,str) for key,value in environment.items()):
        raise ValueError('environment must be an explicit string map')
    if not 1 <= timeout <= 3600:
        raise ValueError('timeout must be between 1 and 3600 seconds')
    if capture.get('format') != 'azimuth-source-capture' or capture.get('version') != 1:
        raise ValueError('execution requires an accountable source capture')
    if fingerprint({key:value for key,value in capture.items() if key != 'fingerprint'}) != capture.get('fingerprint'):
        raise ValueError('source capture fingerprint mismatch')
    if capture.get('repository') != scope.get('repository'):
        raise ValueError('source capture does not match scoped repository')
    current_capture = capture_source(root, capture['repository'], git_executable)
    if current_capture != capture:
        raise ValueError('source inventory or captured content changed; recapture before execution')
    selected = {item['file']:item['file_fingerprint'] for item in scope['sites']}
    with tempfile.TemporaryDirectory(prefix='azimuth-challenge-') as temporary:
        isolated = (Path(temporary) / 'source').resolve()
        isolated.mkdir()
        for item in capture['files']:
            relative = item['file']
            path = located(root,relative)
            data = path.read_bytes()
            digest = 'sha256:' + hashlib.sha256(data).hexdigest()
            if digest != item['fingerprint']:
                raise ValueError(f'captured file changed: {relative}')
            if relative in selected and selected.pop(relative) != digest:
                raise ValueError(f'reviewed target file changed: {relative}')
            destination = isolated / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(data)
        if selected:
            raise ValueError('source capture omits selected source files')
        for link in capture['links']:
            relative=link['file']
            if relative in selected:
                raise ValueError('selected source must not be a symlink')
            destination=isolated / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.symlink_to(link['target'])
        for link in capture['links']:
            destination=isolated / link['file']
            try:
                resolved=destination.resolve(strict=False)
                resolved.relative_to(isolated)
                if resolved.exists() != link['target_exists']:
                    raise ValueError('captured source link target availability changed')
            except (ValueError,OSError,RuntimeError) as error:
                raise ValueError('captured source link target is not present in isolated inventory') from error
        request = scope['request']
        tool = request['tool']
        if tool == 'stryker-net':
            configuration = isolated / 'stryker-config.json'
            configuration.write_text(json.dumps(scope['tool_configuration']))
            working = (isolated / request['test_project']).parent
            arguments = [executable, 'stryker', '--config-file', str(configuration), '--skip-version-check', '--break-on-initial-test-failure', '--verbosity', 'debug']
        else:
            working = isolated
            arguments = [executable] + scope['tool_configuration']['arguments']
            for rule in scope['tool_configuration']['rules']:
                located(isolated,rule)
                arguments += ['--config',rule]
            arguments += scope['target_scope']
        started = int(time.time()*1000)
        stdout_path = Path(temporary) / 'stdout'
        stderr_path = Path(temporary) / 'stderr'
        timed_out = False
        with stdout_path.open('wb') as stdout, stderr_path.open('wb') as stderr:
            process = subprocess.Popen(arguments, cwd=working, env=environment, stdout=stdout, stderr=stderr, start_new_session=True)
            try:
                deadline = time.monotonic() + timeout
                while process.poll() is None:
                    remaining = deadline - time.monotonic()
                    if remaining <= 0:
                        raise subprocess.TimeoutExpired(arguments, timeout)
                    if stdout_path.stat().st_size > 16777216 or stderr_path.stat().st_size > 1048576:
                        raise ValueError('native tool output exceeded configured limits')
                    try:
                        process.wait(timeout=min(remaining, 0.2))
                    except subprocess.TimeoutExpired:
                        continue
            except subprocess.TimeoutExpired:
                timed_out = True
            finally:
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                process.wait()
        finished = int(time.time()*1000)
        output, errors = stdout_path.read_bytes(), stderr_path.read_bytes()
        if len(output) > 16777216 or len(errors) > 1048576:
            raise ValueError('native tool output exceeded configured limits')
        if tool == 'stryker-net':
            reports = list(working.glob('StrykerOutput/**/mutation-report.json'))
            if len(reports) != 1:
                native = {}
            else:
                native = read(reports[0])
        else:
            try:
                native = json.loads(output)
            except ValueError:
                native = {}
        outcome, findings, gaps, counts = normalize(tool,native,execution_failed=timed_out or process.returncode != 0)
        report = {'format':'azimuth-challenge-native-report','version':1,'challenger':scope['challenger'],
                  'scope_fingerprint':supplied,'capture_fingerprint':capture['fingerprint'],'subject':capture['subject'],'tool':tool,
                  'tool_executable_digest':'sha256:' + hashlib.sha256(Path(executable).read_bytes()).hexdigest(),
                  'tool_arguments':arguments,'environment_fingerprint':fingerprint(environment),
                  'started_at_ms':started,'finished_at_ms':finished,'outcome':outcome,
                  'execution_status':'failed' if timed_out or process.returncode != 0 else 'completed',
                  'assessment':'no-findings' if outcome == 'clean' else outcome,
                  'objections':findings,'inconclusive_reasons':gaps,'native_counts':counts,'native_report_json':json.dumps(native,sort_keys=True,allow_nan=False), 'limitations':scope['limitations'] + (['Stryker mutates compiled code internally; this report records the captured Challenger input Subject, not exact materialized source Subjects for each mutant. No mutant-specific product Observations are inferred.'] if tool == 'stryker-net' else []),
                  'tool_exit':process.returncode if process.returncode >= 0 else 'signal',
                  'timed_out':timed_out,'stdout':output.decode(errors='replace'),'stderr':errors.decode(errors='replace')}
        report['fingerprint'] = fingerprint(report)
        return report


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--scope',required=True)
    parser.add_argument('--root',required=True)
    parser.add_argument('--tool-executable',required=True)
    parser.add_argument('--capture',required=True)
    parser.add_argument('--git-executable',required=True)
    parser.add_argument('--environment',required=True)
    parser.add_argument('--timeout',type=int,default=900)
    parser.add_argument('--out',required=True)
    args=parser.parse_args()
    try:
        report=execute(read(args.scope),Path(args.root),args.tool_executable,read(args.capture),read(args.environment),args.timeout,args.git_executable)
        Path(args.out).write_text(json.dumps(report,indent=2)+'\n')
        return 0 if report['outcome']=='clean' else 1
    except (ValueError,OSError,KeyError) as error:
        parser.exit(2,f'azimuth.challenge: {error}\n')
if __name__=='__main__':
    raise SystemExit(main())
