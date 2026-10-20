"""Durable target transport retains facts across retries and checkpoint failures."""
import http.server
import json
import threading
from datetime import datetime, timezone
import sqlite3
import time
import unittest
from services.collector.collector import initialize, stage_targets, target_observation, validate_binding, ProbeResults, PrometheusRules


class TargetTransportTests(unittest.TestCase):
    def setUp(self):
        self.db = sqlite3.connect(':memory:')
        initialize(self.db)
        self.source = {'id': 'exposure', 'project': 'example', 'environment': 'production', 'adapter': 'probe-results',
                       'allowed_outcomes': ['violated', 'inconclusive'], 'namespace': 'application', 'service': 'observer', 'port': 8080, 'path': '/results',
                       'definition': {'plan': {'checks': [{'id': 'exposure', 'units': [{'id': 'whole'}]}]},
                                      'adapters': [{'id': 'probe-results'}], 'routes': []}}
        now = int(time.time()*1000)
        self.record = {'execution_id': 'run-1', 'check': 'exposure', 'started_at_ms': now, 'finished_at_ms': now,
                       'outcome': 'inconclusive', 'targets': [{'component': 'api', 'image': 'example/api@sha256:' + 'a'*64, 'instance': 'containerd://process-1'}]}

    def tearDown(self):
        self.db.close()

    def test_subject_free_measurement_and_atomic_replay(self):
        validate_binding(self.source)
        observation = target_observation(self.source, self.record)
        self.assertNotIn('subject', observation)
        stage_targets(self.db, 'example', [observation], 'exposure', 'next')
        original = self.db.execute('SELECT * FROM target_outbox').fetchall()
        stage_targets(self.db, 'example', [observation], 'exposure', 'next')
        self.assertEqual(self.db.execute('SELECT * FROM target_outbox').fetchall(), original)
        self.assertEqual(self.db.execute('SELECT cursor FROM checkpoints').fetchone()[0], 'next')

    def test_capacity_failure_preserves_checkpoint_and_pending_bytes(self):
        observation = target_observation(self.source, self.record)
        stage_targets(self.db, 'example', [observation], 'exposure', 'first')
        with self.assertRaises(ValueError):
            stage_targets(self.db, 'example', [observation], 'exposure', 'lost', max_pending=1)
        self.assertEqual(self.db.execute('SELECT cursor FROM checkpoints').fetchone()[0], 'first')
        self.assertEqual(self.db.execute('SELECT COUNT(*) FROM target_outbox').fetchone()[0], 1)

    def test_identity_time_positive_and_release_bindings_fail_closed(self):
        for record in (dict(self.record, execution_id='x'*257), dict(self.record, targets=[dict(self.record['targets'][0], component='x'*257)]), dict(self.record, outcome='satisfied'), dict(self.record, finished_at_ms=int(time.time()*1000)+60000),
                       dict(self.record, targets=[dict(self.record['targets'][0], image='example/api:latest')]),
                       dict(self.record, targets=self.record['targets']*2)):
            with self.subTest(record=record), self.assertRaises(ValueError):
                target_observation(self.source, record)
        with self.assertRaises(ValueError): validate_binding(dict(self.source, launch={}))

    def test_offline_preparation_requires_exact_reviewed_identity(self):
        from services.collector.configure import prepare
        from copy import deepcopy
        source = deepcopy(self.source)
        source['launch'] = deepcopy(source.pop('definition'))
        source['launch']['plan']['fingerprint'] = 'sha256:' + 'b'*64
        source['launch']['subject'] = {'kind': 'deployment'}
        source['valid_from_ms'] = 0
        source['valid_until_ms'] = 1
        current = {'assurance_url': 'http://assurance', 'namespaces': ['application'], 'projects': {'example': {'token_file': '/credentials/checks'}}, 'sources': [source]}
        reviewed = {'sources': {'exposure': {'id': 'stable-exposure', 'environment': 'production', 'definition': self.source['definition']}}, 'contracts': []}
        result = prepare(current, reviewed)
        self.assertEqual(result['version'], 2)
        self.assertEqual(result['sources'][0]['id'], 'stable-exposure')
        self.assertNotIn('launch', result['sources'][0])
        self.assertNotIn('valid_until_ms', result['sources'][0])
        changed = deepcopy(reviewed)
        changed['sources']['exposure']['definition']['plan']['checks'][0]['id'] = 'different'
        with self.assertRaises(ValueError): prepare(current, changed)
        with self.assertRaises(ValueError): prepare(current, {'sources': {}, 'contracts': []})

    def test_unmatched_inconclusive_facts_remain_publishable(self):
        observation = target_observation(self.source, dict(self.record, targets=[]))
        stage_targets(self.db, 'example', [observation])
        self.assertEqual(self.db.execute('SELECT COUNT(*) FROM target_outbox').fetchone()[0], 1)


class AdapterHttpTests(unittest.TestCase):
    def test_probe_version_and_prometheus_actual_identity(self):
        now = datetime.now(timezone.utc).isoformat()
        target = {'component': 'api', 'image': 'example/api@sha256:' + 'a'*64, 'instance': 'containerd://real-process'}
        state = {'version': 2, 'target': target, 'alert_target': dict(target), 'alerts': True}
        class Handler(http.server.BaseHTTPRequestHandler):
            def log_message(self, *_):
                pass
            def do_GET(self):
                if self.path.startswith('/results'):
                    value = {'format': 'azimuth-check-results', 'version': state['version'], 'items': [], 'next_cursor': None, 'history_gap': False, 'has_more': False}
                elif self.path.startswith('/api/v1/rules'):
                    value = {'status': 'success', 'data': {'groups': [{'rules': [{'name': 'Alert', 'labels': {'scope': 'api'}, 'health': 'ok', 'lastEvaluation': now, 'state': 'firing', 'alerts': [{'state': 'firing', 'labels': state['alert_target']}] if state['alerts'] else []}]}]}}
                else:
                    value = {'status': 'success', 'data': {'activeTargets': [{'labels': dict(state['target'], job='api'), 'health': 'up', 'lastScrape': now}]}}
                self.send_response(200); self.end_headers(); self.wfile.write(json.dumps(value).encode())
        server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        worker = threading.Thread(target=server.serve_forever); worker.start()
        try:
            origin = 'http://127.0.0.1:' + str(server.server_port)
            self.assertEqual(ProbeResults().collect({'url': origin + '/results', 'id': 'stable'}, None), ([], None))
            state['version'] = 1
            with self.assertRaises(ValueError): ProbeResults().collect({'url': origin + '/results', 'id': 'stable'}, None)
            source = {'url': origin, 'id': 'alerts', 'rule': 'Alert', 'rule_labels': {'scope': 'api'}, 'target_labels': {'job': 'api'},
                      'identity_labels': {key: key for key in target}, 'definition': {'plan': {'checks': [{'id': 'alert'}]}}}
            records, _ = PrometheusRules().collect(source, None)
            self.assertEqual(records[0]['targets'], [target])
            self.assertEqual(records[0]['outcome'], 'violated')
            self.assertNotIn('subject_fingerprint', records[0])
            self.assertEqual(records[0]['finished_at_ms'], int(datetime.fromisoformat(now).timestamp()*1000))
            self.assertEqual(PrometheusRules().collect(source, PrometheusRules().collect(source, None)[1])[0], [])
            state['target'] = dict(target, instance='containerd://replacement-process')
            records, _ = PrometheusRules().collect(source, None)
            self.assertEqual((records[0]['targets'], records[0]['outcome']), ([], 'inconclusive'))
            state['target'] = target
            state['alerts'] = False
            records, _ = PrometheusRules().collect(source, None)
            self.assertEqual((records[0]['targets'], records[0]['outcome']), ([], 'inconclusive'))
            state['alerts'] = True
            state['target'] = {'component': 'api'}
            records, _ = PrometheusRules().collect(source, None)
            self.assertEqual((records[0]['targets'], records[0]['outcome']), ([], 'inconclusive'))
        finally:
            server.shutdown(); server.server_close(); worker.join()


if __name__ == '__main__':
    unittest.main()
