"""Provenance transport tests; the HTTPS-only CLI is exercised through its functions against a local HTTP receiver."""
import contextlib
import http.server
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import threading
import unittest
import urllib.request

SCRIPT = Path(__file__).resolve().parent / 'publish-assurance.py'
specification = importlib.util.spec_from_file_location('publish_assurance', SCRIPT)
publication = importlib.util.module_from_spec(specification)
specification.loader.exec_module(publication)

TOKEN = 'p' * 40
HEADER = {'format': 'azimuth-assurance-publication-receipt', 'version': 1, 'project': 'demo',
          'model_revision': None, 'snapshot_fingerprint': None}


def ledger(change, project='demo'):
    return {'format': 'azimuth-work-provenance-ledger', 'version': 1, 'project': project, 'change': change, 'instances': []}


def attribution(identity):
    return {'format': 'azimuth-failure-attribution', 'version': 1, 'id': identity, 'fingerprint': 'sha256:' + identity}


class Receiver:
    """Answers each POST path with a configured status and records what arrived."""

    def __init__(self, statuses):
        received = self.received = []

        class Handler(http.server.BaseHTTPRequestHandler):
            def do_POST(self):
                body = self.rfile.read(int(self.headers['Content-Length']))
                received.append((self.path, self.headers['Authorization'], json.loads(body)))
                status = statuses[self.path]
                content = b'{}'
                self.send_response(status)
                self.send_header('Content-Type', 'application/json')
                self.send_header('Content-Length', str(len(content)))
                self.end_headers()
                self.wfile.write(content)

            def log_message(self, format, *args):
                pass

        self.server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        self.thread = threading.Thread(target=self.server.serve_forever)

    def __enter__(self):
        self.thread.start()
        return 'http://127.0.0.1:%d/api/v1/projects/demo' % self.server.server_port

    def __exit__(self, *exception):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join()


class UsageTests(unittest.TestCase):
    def usage_error(self, argv):
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr), self.assertRaises(SystemExit) as raised:
            publication.parse_arguments(argv)
        self.assertEqual(raised.exception.code, 2)
        return stderr.getvalue()

    base = ['--project', 'demo', '--server', 'https://assurance.example', '--receipt', 'receipt.json']
    assurance = ['--snapshot', 's.json', '--azimuth', 'azimuth', '--owner-token-file', 'o', '--producer-token-file', 'p']

    def test_ledger_without_publisher_credential_is_a_usage_error(self):
        self.assertIn('require --publisher-token-file', self.usage_error(self.base + ['--provenance-ledger', 'l.json']))

    def test_attributions_without_publisher_credential_is_a_usage_error(self):
        self.assertIn('require --publisher-token-file', self.usage_error(self.base + self.assurance + ['--failure-attributions', 'a.json']))

    def test_publisher_credential_without_provenance_input_is_a_usage_error(self):
        self.assertIn('--publisher-token-file requires', self.usage_error(self.base + self.assurance + ['--publisher-token-file', 't']))

    def test_no_input_is_a_usage_error(self):
        self.assertIn('nothing to publish', self.usage_error(self.base))

    def test_snapshot_still_requires_its_credentials(self):
        self.assertIn('--snapshot requires', self.usage_error(self.base + ['--snapshot', 's.json', '--azimuth', 'azimuth', '--owner-token-file', 'o']))

    def test_assurance_credentials_without_snapshot_are_a_usage_error(self):
        message = self.usage_error(self.base + ['--owner-token-file', 'o', '--publisher-token-file', 't', '--provenance-ledger', 'l.json'])
        self.assertIn('require --snapshot', message)

    def test_provenance_only_publication_is_accepted(self):
        args = publication.parse_arguments(self.base + ['--publisher-token-file', 't', '--provenance-ledger', 'a.json', '--provenance-ledger', 'b.json'])
        self.assertIsNone(args.snapshot)
        self.assertEqual(args.provenance_ledger, [Path('a.json'), Path('b.json')])

    def test_server_must_be_https(self):
        with self.assertRaisesRegex(ValueError, 'HTTPS origin'):
            publication.project_base('http://assurance.example', 'demo')


class InputValidationTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.root = Path(self.directory.name)

    def tearDown(self):
        self.directory.cleanup()

    def file(self, name, content):
        path = self.root / name
        path.write_text(content if isinstance(content, str) else json.dumps(content))
        return path

    def test_ledger_of_another_project_is_rejected(self):
        path = self.file('ledger.json', ledger('ingress', project='other'))
        with self.assertRaisesRegex(ValueError, 'does not belong to the requested project'):
            publication.provenance_ledger(path, 'demo')

    def test_ledger_format_and_version_are_required(self):
        for content in (dict(ledger('ingress'), format='azimuth-failure-attributions'), dict(ledger('ingress'), version=2), dict(ledger('ingress'), version=True)):
            with self.subTest(content=content), self.assertRaisesRegex(ValueError, 'azimuth-work-provenance-ledger version 1'):
                publication.provenance_ledger(self.file('ledger.json', content), 'demo')

    def test_ledger_without_change_is_rejected(self):
        content = ledger('ingress')
        del content['change']
        with self.assertRaisesRegex(ValueError, 'names no change'):
            publication.provenance_ledger(self.file('ledger.json', content), 'demo')

    def test_malformed_ledger_names_file_and_line(self):
        path = self.file('ledger.json', '{\n  "format": \n}')
        with self.assertRaisesRegex(ValueError, r'ledger\.json:3: invalid provenance ledger JSON'):
            publication.provenance_ledger(path, 'demo')

    def test_attribution_collection_format_and_version_are_required(self):
        for content in ({'format': 'azimuth-failure-attribution', 'version': 1, 'attributions': []},
                        {'format': 'azimuth-failure-attributions', 'version': 2, 'attributions': []},
                        {'format': 'azimuth-failure-attributions', 'version': 1}):
            with self.subTest(content=content), self.assertRaisesRegex(ValueError, 'azimuth-failure-attributions version 1'):
                publication.failure_attributions(self.file('attributions.json', content))

    def test_attribution_entries_are_validated(self):
        collection = {'format': 'azimuth-failure-attributions', 'version': 1, 'attributions': [dict(attribution('gap-1'), version=2)]}
        with self.assertRaisesRegex(ValueError, 'azimuth-failure-attribution version 1'):
            publication.failure_attributions(self.file('attributions.json', collection))
        collection['attributions'] = [attribution('gap-1'), attribution('gap-1')]
        with self.assertRaisesRegex(ValueError, 'duplicate failure attribution id gap-1'):
            publication.failure_attributions(self.file('attributions.json', collection))

    def test_two_ledgers_for_one_change_are_rejected(self):
        with self.assertRaisesRegex(ValueError, 'more than one provenance ledger for change ingress'):
            publication.provenance_requests('https://a/api/v1/projects/demo', TOKEN, [ledger('ingress'), ledger('ingress')], [])

    def test_publisher_credential_must_be_owner_only(self):
        path = self.file('publisher', TOKEN)
        path.chmod(0o640)
        with self.assertRaisesRegex(ValueError, 'owner-only'):
            publication.credential(path)


class TransportTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.receipt_path = Path(self.directory.name) / 'receipt.json'
        self.opener = urllib.request.build_opener(publication.RejectRedirects())

    def tearDown(self):
        self.directory.cleanup()

    def publish(self, requests, assurance=()):
        with self.receipt_path.open('x') as output:
            return publication.publish(self.opener, output, HEADER, list(assurance), requests)

    def stored_receipt(self):
        return json.loads(self.receipt_path.read_text())

    def test_accepted_ledgers_and_attributions_are_receipted(self):
        statuses = {'/api/v1/projects/demo/provenance/ledgers': 201, '/api/v1/projects/demo/provenance/attributions': 200}
        receiver = Receiver(statuses)
        with receiver as base:
            requests = publication.provenance_requests(base, TOKEN, [ledger('ingress')], [attribution('gap-1')])
            self.publish(requests)
        self.assertEqual(receiver.received, [
            ('/api/v1/projects/demo/provenance/ledgers', 'Bearer ' + TOKEN, ledger('ingress')),
            ('/api/v1/projects/demo/provenance/attributions', 'Bearer ' + TOKEN, attribution('gap-1')),
        ])
        self.assertEqual(self.stored_receipt(), dict(HEADER, status='published', submissions=[], provenance_submissions=[
            {'kind': 'provenance-ledger', 'change': 'ingress', 'outcome': 'accepted', 'http_status': 201},
            {'kind': 'failure-attribution', 'id': 'gap-1', 'fingerprint': 'sha256:gap-1', 'outcome': 'accepted', 'http_status': 200},
        ]))

    def test_conflict_is_receipted_and_stops_publication(self):
        statuses = {'/api/v1/projects/demo/provenance/ledgers': 409, '/api/v1/projects/demo/provenance/attributions': 201}
        receiver = Receiver(statuses)
        with receiver as base:
            requests = publication.provenance_requests(base, TOKEN, [ledger('ingress')], [attribution('gap-1')])
            with self.assertRaisesRegex(ValueError, 'provenance publication rejected: HTTP 409; partial receipt retained'):
                self.publish(requests)
        self.assertEqual([path for path, _, _ in receiver.received], ['/api/v1/projects/demo/provenance/ledgers'])
        self.assertEqual(self.stored_receipt(), dict(HEADER, status='incomplete', submissions=[], provenance_submissions=[
            {'kind': 'provenance-ledger', 'change': 'ingress', 'outcome': 'conflict', 'http_status': 409},
        ]))

    def test_other_rejection_is_receipted(self):
        receiver = Receiver({'/api/v1/projects/demo/provenance/attributions': 422})
        with receiver as base:
            requests = publication.provenance_requests(base, TOKEN, [], [attribution('gap-1')])
            with self.assertRaisesRegex(ValueError, 'HTTP 422'):
                self.publish(requests)
        self.assertEqual(self.stored_receipt()['provenance_submissions'], [
            {'kind': 'failure-attribution', 'id': 'gap-1', 'fingerprint': 'sha256:gap-1', 'outcome': 'rejected', 'http_status': 422},
        ])

    def test_assurance_records_precede_provenance_and_keep_their_receipt_shape(self):
        statuses = {'/api/v1/projects/demo/runs': 201, '/api/v1/projects/demo/provenance/ledgers': 201}
        receiver = Receiver(statuses)
        with receiver as base:
            assurance = [(base + '/runs', {'bundle_fingerprint': 'sha256:run'}, 'r' * 40, {'kind': 'run', 'fingerprint': 'sha256:run'})]
            self.publish(publication.provenance_requests(base, TOKEN, [ledger('ingress')], []), assurance)
        self.assertEqual([path for path, _, _ in receiver.received], ['/api/v1/projects/demo/runs', '/api/v1/projects/demo/provenance/ledgers'])
        receipt = self.stored_receipt()
        self.assertEqual(receipt['submissions'], [{'kind': 'run', 'fingerprint': 'sha256:run', 'http_status': 201}])
        self.assertEqual(receipt['status'], 'published')

    def test_receipt_without_provenance_has_no_provenance_section(self):
        self.publish(None)
        self.assertEqual(self.stored_receipt(), dict(HEADER, status='published', submissions=[]))


if __name__ == '__main__':
    unittest.main()
