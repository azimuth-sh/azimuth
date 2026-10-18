"""Durable collection of explicitly bound machine observations."""
import hashlib
import json
import logging
import os
from pathlib import Path
import sqlite3
import ssl
import subprocess
import tempfile
import time
from urllib.error import HTTPError
from urllib.parse import urlencode, urlsplit, quote
from urllib.request import Request, build_opener, HTTPRedirectHandler, HTTPSHandler
from azimuth_run import build_check_bundle, canonical_json

MAX_BYTES = 4 * 1024 * 1024

class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, *args):
        return None

def fetch(url, token=None, method='GET', body=None, ca=None):
    parsed = urlsplit(url)
    if parsed.scheme not in ('http', 'https') or parsed.username or parsed.password or parsed.fragment:
        raise ValueError('Unsupported endpoint')
    headers = {'Accept': 'application/json'}
    if token:
        headers['Authorization'] = 'Bearer ' + token
    if body:
        headers['Content-Type'] = 'application/json'
    opener = build_opener(NoRedirect(), HTTPSHandler(context=ssl.create_default_context(cafile=ca)))
    with opener.open(Request(url, data=body, headers=headers, method=method), timeout=15) as response:
        raw = response.read(MAX_BYTES + 1)
    if len(raw) > MAX_BYTES:
        raise ValueError('Response exceeds limit')
    return json.loads(raw)

def native_bundle(binding, record, import_inputs=None):
    launch = binding['launch']
    if record['subject_fingerprint'] != launch['subject_fingerprint'] or record['binding'] != binding['id']:
        raise ValueError('Observation deployment binding differs')
    if not binding['valid_from_ms'] <= record['started_at_ms'] <= record['finished_at_ms'] <= binding['valid_until_ms']:
        raise ValueError('Observation outside explicit deployment binding interval')
    checks = launch['plan']['checks']
    if len(checks) != 1 or checks[0]['id'] != record['check'] or len(checks[0]['units']) != 1:
        raise ValueError('Adapter requires exactly one Check and one whole observation unit')
    if record['outcome'] not in binding['allowed_outcomes'] or (binding['adapter'] == 'probe-results' and record['outcome'] == 'satisfied'):
        raise ValueError('Outcome not permitted for source')
    adapter = launch['adapters'][0]
    activity = {'id': 'measurement', 'status': 'completed', 'started_at_ms': record['started_at_ms'],
                'finished_at_ms': record['finished_at_ms'], 'artifacts': [], 'diagnostics': [],
                'attributes': {'native-report': json.dumps(record, separators=(',', ':'), sort_keys=True)}}
    raw = json.dumps(record, separators=(',', ':'), sort_keys=True).encode()
    return build_check_bundle(launch, source={'system': binding['source_system'], 'execution': record['execution_id']},
        normalizer={'id': 'adapter/' + adapter['id'], 'version': adapter['adapter_version'], 'build_fingerprint': adapter['adapter_fingerprint']},
        generated_at_ms=record['finished_at_ms'], started_at_ms=record['started_at_ms'], finished_at_ms=record['finished_at_ms'],
        actual_context=launch['plan']['required_context'], activities=[activity], artifacts=[], diagnostics=[],
        units_by_check={record['check']: [{'id': checks[0]['units'][0]['id'], 'attempts': [{'ordinal': 0, 'activity': 'measurement',
            'outcomes': {case: record['outcome'] for case in checks[0]['cases']}}]}]},
        import_inputs=import_inputs if import_inputs is not None else [{'id': 'native-report', 'digest': 'sha256:' + hashlib.sha256(raw).hexdigest(), 'size_bytes': len(raw)}])

class ProbeResults:
    def collect(self, binding, cursor):
        base = binding['url']
        page = fetch(base + '?' + urlencode({'cursor': cursor or '', 'limit': 50}))
        if page.get('format') != 'azimuth-check-results' or page.get('version') != 1 or len(page['items']) > 100:
            raise ValueError('Unsupported results page')
        self.history_gap = bool(page['history_gap'])
        if self.history_gap:
            logging.error('history-gap source=%s cursor=%s', binding['id'], cursor)
        records = page['items']
        for record in records:
            if record['outcome'] not in ('violated', 'inconclusive'):
                raise ValueError('Uncontrolled exposure probe cannot establish satisfaction')
        return records, page['next_cursor']

class PrometheusRules:
    def collect(self, binding, cursor):
        end = int(time.time()*1000)
        window = end // (binding.get('interval_seconds', 60) * 1000)
        identity = binding['id'] + '/' + str(window)
        if cursor == identity:
            return [], cursor
        rules = fetch(binding['url'].rstrip('/') + '/api/v1/rules?type=alert')
        targets = fetch(binding['url'].rstrip('/') + '/api/v1/targets?state=active')
        if rules.get('status') != 'success' or targets.get('status') != 'success':
            raise ValueError('Prometheus API did not succeed')
        matches = [rule for group in rules['data']['groups'] for rule in group['rules']
                   if rule.get('name') == binding['rule'] and all(rule.get('labels', {}).get(k) == v for k,v in binding['rule_labels'].items())]
        selected = [target for target in targets['data']['activeTargets']
                    if all(target.get('labels', {}).get(k) == v for k,v in binding['target_labels'].items())]
        fresh = end - binding.get('max_age_seconds', 120) * 1000
        def timestamp(value):
            from datetime import datetime
            return int(datetime.fromisoformat(value.replace('Z', '+00:00')).timestamp()*1000)
        healthy = (len(matches) == 1 and bool(selected) and all(target.get('health') == 'up' and not target.get('lastError')
                   and timestamp(target['lastScrape']) >= fresh for target in selected)
                   and matches[0].get('health') == 'ok' and not matches[0].get('lastError')
                   and timestamp(matches[0]['lastEvaluation']) >= fresh)
        outcome = 'inconclusive'
        if healthy:
            state = matches[0].get('state')
            if state == 'firing':
                outcome = 'violated'
            elif state == 'inactive' and binding.get('allow_healthy_clear', False):
                outcome = 'satisfied'
        return [{'execution_id': identity, 'check': binding['launch']['plan']['checks'][0]['id'],
                 'binding': binding['id'], 'subject_fingerprint': binding['launch']['subject_fingerprint'],
                 'started_at_ms': end, 'finished_at_ms': end, 'outcome': outcome,
                 'rule': matches, 'targets_healthy': healthy, 'selected_target_count': len(selected)}], identity

ADAPTERS = {'probe-results': ProbeResults, 'prometheus-rules': PrometheusRules}

def discover(config):
    approved = {(binding['namespace'], binding['service']): binding for binding in config['sources'] if binding['adapter'] == 'probe-results'}
    token = Path('/var/run/secrets/kubernetes.io/serviceaccount/token').read_text()
    ca = '/var/run/secrets/kubernetes.io/serviceaccount/ca.crt'
    base = 'https://' + os.environ['KUBERNETES_SERVICE_HOST'] + ':' + os.environ.get('KUBERNETES_SERVICE_PORT', '443')
    result = [dict(binding) for binding in config['sources'] if binding['adapter'] != 'probe-results']
    for namespace in config['namespaces']:
        url = base + '/api/v1/namespaces/' + quote(namespace, safe='') + '/services?' + urlencode({'labelSelector': 'azimuth.sh/collect=true'})
        page = fetch(url, token=token, ca=ca)
        if page.get('metadata', {}).get('continue'):
            raise ValueError('Service discovery pagination not supported; narrow namespace inventory')
        for service in page['items']:
            name = service['metadata']['name']
            binding = approved.get((namespace, name))
            if not binding:
                continue
            if service['spec'].get('type', 'ClusterIP') != 'ClusterIP' or service['spec'].get('externalName') or service['spec'].get('externalIPs'):
                raise ValueError('Discovery accepts internal ClusterIP Services only')
            annotations = service['metadata'].get('annotations', {})
            port = int(annotations.get('azimuth.sh/results-port', '8080'))
            path = annotations.get('azimuth.sh/results-path', '/results')
            if port != binding['port'] or path != binding['path'] or not path.startswith('/') or '?' in path or '#' in path:
                raise ValueError('Discovered scrape route differs from approved binding')
            item = dict(binding)
            item['url'] = 'http://' + name + '.' + namespace + '.svc:' + str(port) + path
            result.append(item)
    return result

def validate_binding(binding, code_fingerprint, protocol_fingerprint):
    if binding['adapter'] == 'probe-results' and set(binding['allowed_outcomes']) != {'violated', 'inconclusive'}:
        raise ValueError('Exposure probes permit only violated or inconclusive outcomes')
    launch = binding['launch']
    if len(launch['adapters']) != 1 or launch['adapters'][0]['id'] != binding['adapter']:
        raise ValueError('Launch must select the configured collector adapter')
    if binding['collector_fingerprint'] != code_fingerprint:
        raise ValueError('Binding must identify the executing collector code')
    if binding['protocol_fingerprint'] != protocol_fingerprint:
        raise ValueError('Binding must identify the executing Run protocol resource')


def main():
    logging.basicConfig(level=logging.INFO)
    config = json.loads(Path(os.environ.get('COLLECTOR_CONFIG', '/config/collector.json')).read_text())
    code_fingerprint = 'sha256:' + hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    protocol_fingerprint = 'sha256:' + hashlib.sha256(Path(build_check_bundle.__code__.co_filename).read_bytes()).hexdigest()
    ids = set()
    for binding in config['sources']:
        if binding['id'] in ids:
            raise ValueError('Duplicate source binding identity')
        ids.add(binding['id'])
        validate_binding(binding, code_fingerprint, protocol_fingerprint)
        if binding['launch']['subject']['kind'] != 'deployment':
            raise ValueError('Collector requires explicit deployment bindings')
        if binding['adapter'] == 'prometheus-rules':
            if not binding.get('target_labels') or binding['rule_labels'].get('azimuth_deployment_fingerprint') != binding['launch']['subject']['deployment_fingerprint']:
                raise ValueError('Prometheus mappings require target labels and exact deployment fingerprint label')
    state = Path(os.environ.get('COLLECTOR_STATE', '/state'))
    state.mkdir(parents=True, exist_ok=True)
    db = sqlite3.connect(state/'collector.db')
    db.executescript('CREATE TABLE IF NOT EXISTS checkpoints (source TEXT PRIMARY KEY, cursor TEXT NOT NULL); CREATE TABLE IF NOT EXISTS outbox (fingerprint TEXT PRIMARY KEY, project TEXT NOT NULL, payload TEXT NOT NULL); CREATE TABLE IF NOT EXISTS collection_gaps (source TEXT NOT NULL, cursor TEXT NOT NULL, observed_at_ms INTEGER NOT NULL);')
    while True:
        try:
            try:
                bindings = discover(config)
            except Exception as failure:
                logging.error('discovery-failed error=%s', type(failure).__name__)
                bindings = []
            for binding in bindings:
                try:
                    if db.execute('SELECT COUNT(*) FROM outbox').fetchone()[0] >= config.get('max_pending', 10000):
                        raise ValueError('Outbox full; collection paused')
                    previous = db.execute('SELECT cursor FROM checkpoints WHERE source=?', (binding['id'],)).fetchone()
                    adapter = ADAPTERS[binding['adapter']]()
                    records, cursor = adapter.collect(binding, previous[0] if previous else None)
                    bundles = [native_bundle(binding, record) for record in records]
                    for bundle in bundles:
                        with tempfile.NamedTemporaryFile(mode='w', dir=state, suffix='.json') as file:
                            file.write(canonical_json(bundle)); file.flush()
                            subprocess.run(['/usr/local/bin/azimuth', 'run', 'verify', '--bundle', file.name], check=True, timeout=30, capture_output=True)
                    with db:
                        if getattr(adapter, 'history_gap', False):
                            db.execute('INSERT INTO collection_gaps VALUES (?,?,?)', (binding['id'], previous[0] if previous else '', int(time.time()*1000)))
                        for bundle in bundles:
                            db.execute('INSERT OR IGNORE INTO outbox VALUES (?,?,?)', (bundle['bundle_fingerprint'], binding['project'], canonical_json(bundle)))
                        db.execute('INSERT OR REPLACE INTO checkpoints VALUES (?,?)', (binding['id'], cursor or ''))
                except Exception as failure:
                    logging.error('collection-failed source=%s error=%s', binding['id'], type(failure).__name__)
            for fingerprint, project, payload in db.execute('SELECT fingerprint,project,payload FROM outbox LIMIT 100').fetchall():
                try:
                    token = Path(config['projects'][project]['token_file']).read_text().strip()
                    receipt = fetch(config['assurance_url'].rstrip('/') + '/v1/projects/' + quote(project, safe='') + '/runs', token=token, method='POST', body=payload.encode())
                    if receipt.get('bundle_fingerprint') != fingerprint:
                        raise ValueError('Unexpected ASS receipt')
                    with db:
                        db.execute('DELETE FROM outbox WHERE fingerprint=?', (fingerprint,))
                except Exception as failure:
                    logging.error('publication-failed fingerprint=%s error=%s', fingerprint, type(failure).__name__)
        except Exception as failure:
            logging.error('discovery-failed error=%s', type(failure).__name__)
        (state/'heartbeat').write_text(str(int(time.time())))
        time.sleep(max(10, config.get('interval_seconds', 30)))

if __name__ == '__main__':
    main()
