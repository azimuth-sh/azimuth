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
from urllib.request import Request, build_opener, HTTPRedirectHandler, HTTPSHandler, ProxyHandler
from azimuth_run import build_check_bundle, canonical_json, fingerprint

MAX_BYTES = 4 * 1024 * 1024
TARGET_OBSERVATION_VERSION = 2
# Native plan without its Verification-Target-dependent fingerprint (contracts/target-observation.md).
PLAN_FIELDS = frozenset({'model_fingerprint', 'required_execution_context', 'checks', 'challenges'})

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
    opener = build_opener(NoRedirect(), ProxyHandler({}), HTTPSHandler(context=ssl.create_default_context(cafile=ca)))
    try:
        with opener.open(Request(url, data=body, headers=headers, method=method), timeout=15) as response:
            raw = response.read(MAX_BYTES + 1)
    except HTTPError as error:
        error.close()
        raise
    if len(raw) > MAX_BYTES:
        raise ValueError('Response exceeds limit')
    return json.loads(raw, object_pairs_hook=unique, parse_constant=lambda _: fail('Invalid JSON constant'))

def native_bundle(binding, record, import_inputs=None):
    launch = binding['launch']
    if record['verification_target_fingerprint'] != launch['verification_target_fingerprint'] or record['binding'] != binding['id']:
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
        actual_context=launch['plan']['required_execution_context'], activities=[activity], artifacts=[], diagnostics=[],
        units_by_check={record['check']: [{'id': checks[0]['units'][0]['id'], 'attempts': [{'ordinal': 0, 'activity': 'measurement',
            'outcomes': {scenario: record['outcome'] for scenario in checks[0]['scenarios']}}]}]},
        import_inputs=import_inputs if import_inputs is not None else [{'id': 'native-report', 'digest': 'sha256:' + hashlib.sha256(raw).hexdigest(), 'size_bytes': len(raw)}])

class ProbeResults:
    def collect(self, binding, cursor):
        base = binding['url']
        page = fetch(base + '?' + urlencode({'cursor': cursor or '', 'limit': 50}))
        if set(page) != {'format', 'version', 'items', 'next_cursor', 'history_gap', 'has_more'} or page.get('format') != 'azimuth-check-results' or type(page.get('version')) is not int or page['version'] != 2 or not isinstance(page['items'], list) or len(page['items']) > 100 or type(page['history_gap']) is not bool or type(page['has_more']) is not bool or (page['next_cursor'] is not None and (not isinstance(page['next_cursor'], str) or len(page['next_cursor']) > 512)):
            raise ValueError('Unsupported results page')
        self.history_gap = page['history_gap']
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
        rules = fetch(binding['url'].rstrip('/') + '/api/v1/rules?type=alert')
        targets = fetch(binding['url'].rstrip('/') + '/api/v1/targets?state=active')
        if rules.get('status') != 'success' or targets.get('status') != 'success':
            raise ValueError('Prometheus API did not succeed')
        matches = [rule for group in rules['data']['groups'] for rule in group['rules']
                   if rule.get('name') == binding['rule'] and all(rule.get('labels', {}).get(k) == v for k,v in binding['rule_labels'].items())]
        selected = [target for target in targets['data']['activeTargets']
                    if all(target.get('labels', {}).get(k) == v for k,v in binding['target_labels'].items())]
        fresh = end - binding.get('max_age_seconds', 120) * 1000
        names = binding['identity_labels']
        def identity(labels):
            return {key: labels.get(names[key], '') for key in ('component', 'image', 'instance')}
        def timestamp(value):
            from datetime import datetime
            return int(datetime.fromisoformat(value.replace('Z', '+00:00')).timestamp()*1000)
        actual, alert_targets = [], []
        evaluated, details = end, {'rule': binding['rule'], 'attribution': 'unresolved'}
        healthy = False
        if len(matches) == 1:
            rule = matches[0]
            try:
                evaluated = timestamp(rule['lastEvaluation'])
                actual = [identity(target.get('labels', {})) for target in selected]
                firing = [alert for alert in rule.get('alerts', []) if alert.get('state') == 'firing']
                alert_targets = [identity(alert.get('labels', {})) for alert in firing]
                validate_targets(actual)
                validate_targets(alert_targets)
                healthy = (bool(actual) and bool(alert_targets) and rule.get('state') == 'firing'
                           and rule.get('health') == 'ok' and not rule.get('lastError')
                           and fresh <= evaluated <= end + 5000
                           and all(target.get('health') == 'up' and not target.get('lastError')
                                   and fresh <= timestamp(target['lastScrape']) <= evaluated for target in selected)
                           and all(target in actual for target in alert_targets))
                details = {'rule': rule['name'], 'state': rule.get('state'), 'last_evaluation_ms': evaluated,
                           'alert_targets': alert_targets, 'selected_targets': actual,
                           'attribution': 'matched' if healthy else 'unresolved'}
            except (KeyError, TypeError, ValueError):
                evaluated, healthy = end, False
        outcome = 'violated' if healthy else 'inconclusive'
        measured = alert_targets if healthy else []
        details.update(outcome=outcome, targets=measured, measured_at_ms=evaluated)
        execution = binding['id'] + '/' + fingerprint(details)
        if cursor == execution:
            return [], cursor
        return [{'execution_id': execution, 'check': binding['definition']['plan']['checks'][0]['id'],
                 'started_at_ms': evaluated, 'finished_at_ms': evaluated, 'outcome': outcome,
                 'targets': measured, 'measurement': details}], execution

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

def fail(message):
    raise ValueError(message)


def unique(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            fail('Duplicate JSON field')
        result[key] = value
    return result


def validate_targets(targets):
    import re
    if not isinstance(targets, list) or len(targets) > 128:
        fail('Invalid target inventory')
    identities = set()
    for target in targets:
        if not isinstance(target, dict) or set(target) != {'component', 'image', 'instance'}:
            fail('Invalid measured target fields')
        if not all(isinstance(value, str) and 0 < len(value) <= (256 if key == 'component' else 512) for key, value in target.items()):
            fail('Invalid measured target identity')
        if not re.fullmatch(r'[^\s@]+@sha256:[0-9a-f]{64}', target['image']) or not re.fullmatch(r'[a-z0-9][a-z0-9+.-]*://[A-Za-z0-9_.:-]{1,480}', target['instance']):
            fail('Actual immutable image and runtime instance required')
        identity = tuple(target[key] for key in ('component', 'image', 'instance'))
        if identity in identities:
            fail('Duplicate measured target')
        identities.add(identity)


def validate_binding(binding, *unused):
    if any(key in binding for key in ('launch', 'verification_target', 'verification_target_fingerprint', 'valid_from_ms', 'valid_until_ms')):
        fail('Release bindings are not supported by collector configuration version 2')
    import re
    for key in ('id', 'project', 'environment'):
        if not isinstance(binding.get(key), str) or not re.fullmatch(r'[a-z0-9][a-z0-9_.:/-]{0,127}', binding[key]):
            fail('Invalid stable source identity')
    if binding.get('adapter') not in ADAPTERS:
        fail('Unknown configured adapter')
    common = {'id', 'project', 'environment', 'adapter', 'allowed_outcomes', 'definition'}
    fields = common | ({'namespace', 'service', 'port', 'path'} if binding['adapter'] == 'probe-results' else {'url', 'rule', 'rule_labels', 'target_labels', 'identity_labels', 'interval_seconds', 'max_age_seconds'})
    if set(binding) - fields or not common <= set(binding):
        fail('Unknown or missing source configuration fields')
    if binding['adapter'] == 'probe-results':
        if not {'namespace', 'service', 'port', 'path'} <= set(binding) or type(binding['port']) is not int or not 1 <= binding['port'] <= 65535 or not isinstance(binding['path'], str) or not binding['path'].startswith('/') or any(value in binding['path'] for value in ('?', '#', '..', '//')):
            fail('Invalid approved probe discovery route')
    for key, default in (('interval_seconds', 60), ('max_age_seconds', 120)):
        value = binding.get(key, default)
        if type(value) is not int or not 10 <= value <= 86400:
            fail('Invalid bounded source interval')
    if binding['adapter'] == 'prometheus-rules':
        if not binding.get('target_labels') or set(binding.get('identity_labels', {})) != {'component', 'image', 'instance'}:
            fail('Prometheus requires explicit actual target identity labels')
        parsed = urlsplit(binding['url'])
        if parsed.scheme not in ('http', 'https') or not parsed.hostname or parsed.username or parsed.password or parsed.query or parsed.fragment:
            fail('Invalid approved Prometheus endpoint')
    definition = binding['definition']
    if set(definition) != {'plan', 'adapters', 'routes'} or not isinstance(definition['plan'], dict) or set(definition['plan']) != PLAN_FIELDS:
        fail('Verification-Target-free native definition with plan fields ' + ', '.join(sorted(PLAN_FIELDS)) + ' required')
    checks = definition['plan']['checks']
    if not isinstance(checks, list) or len(checks) != 1 or not isinstance(checks[0], dict) or not isinstance(checks[0].get('scenarios'), list) or len(checks[0]['units']) != 1 or len(definition['adapters']) != 1 or definition['adapters'][0]['id'] != binding['adapter']:
        fail('One Check with selected Scenarios, one unit and configured adapter required')
    if set(binding['allowed_outcomes']) != {'violated', 'inconclusive'}:
        fail('Runtime collection permits only adverse or inconclusive facts')


def target_observation(source, record):
    validate_targets(record['targets'])
    started, finished = record['started_at_ms'], record['finished_at_ms']
    if type(started) is not int or type(finished) is not int or not 0 <= started <= finished <= int(time.time()*1000) + 5000:
        fail('Invalid measurement interval')
    if record['check'] != source['definition']['plan']['checks'][0]['id'] or record['outcome'] not in source['allowed_outcomes']:
        fail('Measurement differs from approved Check')
    if not isinstance(record['execution_id'], str) or not 0 < len(record['execution_id']) <= 256:
        fail('Invalid measurement execution')
    if not record['targets'] and record['outcome'] != 'inconclusive':
        fail('Unattributed collection facts must remain inconclusive')
    # `subject`/`subject_fingerprint` are earlier names of the Verification Target binding; a stale producer must not embed one in details.
    if any(key in record for key in ('binding', 'verification_target', 'verification_target_fingerprint', 'subject', 'subject_fingerprint', 'deployment', 'release')):
        fail('Runtime reports cannot carry configured release bindings')
    details = canonical_json(record)
    if len(details.encode()) > 65536:
        fail('Measurement details exceed limit')
    return {'format': 'azimuth-target-observation', 'version': TARGET_OBSERVATION_VERSION, 'kind': 'check',
            'source': source['id'], 'execution_id': record['execution_id'], 'environment': source['environment'],
            'started_at_ms': started, 'finished_at_ms': finished, 'targets': record['targets'],
            'check': {'definition': source['definition'], 'outcome': record['outcome'], 'details': details}, 'capture': None}


def initialize(db):
    db.executescript('CREATE TABLE IF NOT EXISTS checkpoints (source TEXT PRIMARY KEY, cursor TEXT NOT NULL); CREATE TABLE IF NOT EXISTS outbox (fingerprint TEXT PRIMARY KEY, project TEXT NOT NULL, payload TEXT NOT NULL); CREATE TABLE IF NOT EXISTS collection_gaps (source TEXT NOT NULL, cursor TEXT NOT NULL, observed_at_ms INTEGER NOT NULL); CREATE TABLE IF NOT EXISTS target_outbox (fingerprint TEXT PRIMARY KEY, project TEXT NOT NULL, token_file TEXT, payload TEXT NOT NULL);')
    db.commit()


def stage_targets(db, project, observations, source=None, cursor=None, history_gap=False, max_pending=10000, token_file=None):
    with db:
        pending = sum(db.execute('SELECT COUNT(*) FROM ' + table).fetchone()[0] for table in ('outbox', 'target_outbox'))
        if pending + len(observations) > max_pending:
            fail('Outbox full; checkpoint retained')
        for observation in observations:
            db.execute('INSERT OR IGNORE INTO target_outbox VALUES (?,?,?,?)', (fingerprint(observation), project, token_file, canonical_json({'observation': observation})))
        if source is not None:
            if history_gap:
                db.execute('INSERT INTO collection_gaps VALUES (?,?,?)', (source, cursor or '', int(time.time()*1000)))
            db.execute('INSERT OR REPLACE INTO checkpoints VALUES (?,?)', (source, cursor or ''))


def publish_targets(db, config):
    for identity, project, token_file, payload in db.execute('SELECT fingerprint,project,token_file,payload FROM target_outbox ORDER BY rowid LIMIT 100').fetchall():
        try:
            token = Path(token_file or config['projects'][project]['token_file']).read_text().strip()
            receipt = fetch(config['assurance_url'].rstrip('/') + '/v1/projects/' + quote(project, safe='') + '/target-observations', token=token, method='POST', body=payload.encode())
            if receipt.get('fingerprint') != identity:
                fail('Unexpected target observation receipt')
            with db:
                db.execute('DELETE FROM target_outbox WHERE fingerprint=?', (identity,))
        except Exception as failure:
            logging.error('target-publication-failed fingerprint=%s error=%s status=%s', identity, type(failure).__name__, getattr(failure, 'code', None))


def main():
    logging.basicConfig(level=logging.INFO)
    config = json.loads(Path(os.environ.get('COLLECTOR_CONFIG', '/config/collector.json')).read_text())
    import contexts
    context_sources = contexts.validate_sources(config)
    context_kube = contexts.KubernetesContexts() if context_sources else None
    if config.get('format') != 'azimuth-collector' or config.get('version') != 2:
        fail('Explicit collector configuration version 2 required')
    ids = set()
    for binding in config['sources']:
        if binding['id'] in ids or binding['project'] not in config['projects']:
            fail('Duplicate or unapproved source identity')
        ids.add(binding['id'])
        validate_binding(binding)
    state = Path(os.environ.get('COLLECTOR_STATE', '/state'))
    state.mkdir(parents=True, exist_ok=True)
    db = sqlite3.connect(state/'collector.db')
    initialize(db)
    while True:
        try:
            try:
                bindings = discover(config)
            except Exception as failure:
                logging.error('discovery-failed error=%s', type(failure).__name__)
                bindings = []
            for binding in bindings:
                try:
                    previous = db.execute('SELECT cursor FROM checkpoints WHERE source=?', (binding['id'],)).fetchone()
                    adapter = ADAPTERS[binding['adapter']]()
                    records, cursor = adapter.collect(binding, previous[0] if previous else None)
                    observations = [target_observation(binding, record) for record in records]
                    stage_targets(db, binding['project'], observations, binding['id'], cursor, getattr(adapter, 'history_gap', False), config.get('max_pending', 10000))
                except Exception as failure:
                    logging.error('collection-failed source=%s error=%s', binding['id'], type(failure).__name__)
            for source in context_sources:
                try:
                    captures = contexts.collect(None, source, context_kube)
                    stage_targets(db, source['project'], captures, max_pending=config.get('max_pending', 10000), token_file=source['token_file'])
                except Exception as failure:
                    logging.error('context-collection-failed source=%s error=%s', source['id'], type(failure).__name__)
            publish_targets(db, config)
            for fingerprint, project, payload in db.execute('SELECT fingerprint,project,payload FROM outbox LIMIT 100').fetchall():
                try:
                    token = Path(config['projects'][project]['token_file']).read_text().strip()
                    receipt = fetch(config['assurance_url'].rstrip('/') + '/v1/projects/' + quote(project, safe='') + '/runs', token=token, method='POST', body=payload.encode())
                    if receipt.get('bundle_fingerprint') != fingerprint:
                        raise ValueError('Unexpected AAS receipt')
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
