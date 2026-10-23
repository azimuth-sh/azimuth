#!/usr/bin/env python3
"""Publication preserves original assessments and never changes the default model."""
import argparse
import json
from pathlib import Path
import stat
import subprocess
import tempfile
import urllib.error
import urllib.parse
import urllib.request


class RejectRedirects(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, response, code, message, headers, url):
        return None


def credential(path):
    path = Path(path)
    if path.is_symlink() or not path.is_file() or stat.S_IMODE(path.stat().st_mode) & 0o077:
        raise ValueError('credential must be an owner-only regular file')
    value = path.read_text().strip()
    if len(value) < 32 or any(character.isspace() for character in value):
        raise ValueError('invalid credential file')
    return value


def parse_arguments(argv=None):
    parser = argparse.ArgumentParser()
    parser.add_argument('--azimuth', type=Path)
    parser.add_argument('--snapshot', type=Path)
    parser.add_argument('--project', required=True)
    parser.add_argument('--server', required=True)
    parser.add_argument('--owner-token-file', type=Path)
    parser.add_argument('--producer-token-file', type=Path)
    parser.add_argument('--reviewer-token-file', action='append', default=[])
    parser.add_argument('--publisher-token-file', type=Path)
    parser.add_argument('--provenance-ledger', action='append', default=[], type=Path)
    parser.add_argument('--failure-attributions', type=Path)
    parser.add_argument('--receipt', required=True, type=Path)
    args = parser.parse_args(argv)
    assurance_inputs = (args.azimuth, args.owner_token_file, args.producer_token_file)
    provenance = bool(args.provenance_ledger) or args.failure_attributions is not None
    if args.snapshot is not None:
        if any(value is None for value in assurance_inputs):
            parser.error('--snapshot requires --azimuth, --owner-token-file and --producer-token-file')
    elif any(value is not None for value in assurance_inputs) or args.reviewer_token_file:
        parser.error('--azimuth and owner, producer or reviewer credentials require --snapshot')
    elif not provenance:
        parser.error('nothing to publish: give --snapshot, --provenance-ledger or --failure-attributions')
    if provenance and args.publisher_token_file is None:
        parser.error('--provenance-ledger and --failure-attributions require --publisher-token-file')
    if args.publisher_token_file is not None and not provenance:
        parser.error('--publisher-token-file requires --provenance-ledger or --failure-attributions')
    return args


def quote(value):
    return urllib.parse.quote(value, safe='')


def project_base(server, project):
    origin = urllib.parse.urlsplit(server)
    if origin.scheme != 'https' or not origin.hostname or origin.username or origin.password or origin.path not in ('', '/') or origin.query or origin.fragment:
        raise ValueError('server must be an HTTPS origin without credentials, path or query')
    return server.rstrip('/') + '/api/v1/projects/' + quote(project)


def is_version_one(value):
    # bool is an int subclass, so a JSON `true` would otherwise pass as version 1.
    return type(value) is int and value == 1


def json_file(path, description):
    try:
        return json.loads(Path(path).read_bytes())
    except json.JSONDecodeError as error:
        raise ValueError(f'{path}:{error.lineno}: invalid {description} JSON: {error.msg}') from None


def provenance_ledger(path, project):
    ledger = json_file(path, 'provenance ledger')
    if not isinstance(ledger, dict) or ledger.get('format') != 'azimuth-work-provenance-ledger' or not is_version_one(ledger.get('version')):
        raise ValueError(f'{path}: expected format azimuth-work-provenance-ledger version 1')
    if ledger.get('project') != project:
        raise ValueError(f'{path}: provenance ledger does not belong to the requested project')
    if not isinstance(ledger.get('change'), str) or not ledger['change']:
        raise ValueError(f'{path}: provenance ledger names no change')
    return ledger


def failure_attributions(path):
    collection = json_file(path, 'failure attributions')
    if not isinstance(collection, dict) or collection.get('format') != 'azimuth-failure-attributions' or not is_version_one(collection.get('version')) or not isinstance(collection.get('attributions'), list):
        raise ValueError(f'{path}: expected format azimuth-failure-attributions version 1 with an attributions list')
    identities = set()
    for attribution in collection['attributions']:
        if not isinstance(attribution, dict) or attribution.get('format') != 'azimuth-failure-attribution' or not is_version_one(attribution.get('version')):
            raise ValueError(f'{path}: every entry must have format azimuth-failure-attribution version 1')
        identity = attribution.get('id')
        if not isinstance(identity, str) or not identity or not isinstance(attribution.get('fingerprint'), str):
            raise ValueError(f'{path}: every failure attribution requires an id and fingerprint')
        if identity in identities:
            raise ValueError(f'{path}: duplicate failure attribution id {identity}')
        identities.add(identity)
    return collection['attributions']


def provenance_requests(base, token, ledgers, attributions):
    changes = set()
    for ledger in ledgers:
        if ledger['change'] in changes:
            raise ValueError(f'more than one provenance ledger for change {ledger["change"]}')
        changes.add(ledger['change'])
    # Ledgers precede attributions because an attribution names attempts the ledgers introduce.
    requests = [(base + '/provenance/ledgers', ledger, token, {'kind': 'provenance-ledger', 'change': ledger['change']})
                for ledger in ledgers]
    requests.extend((base + '/provenance/attributions', attribution, token,
                     {'kind': 'failure-attribution', 'id': attribution['id'], 'fingerprint': attribution['fingerprint']})
                    for attribution in attributions)
    return requests


def assurance_requests(args, base):
    owner = credential(args.owner_token_file)
    producer = credential(args.producer_token_file)
    reviewers = {}
    for item in args.reviewer_token_file:
        identity, separator, name = item.partition('=')
        if not separator or not identity or identity in reviewers:
            raise ValueError('reviewer credential requires a unique reviewer=path binding')
        reviewers[identity] = credential(name)
    content = args.snapshot.read_bytes()
    with tempfile.TemporaryDirectory(prefix='azimuth-publication-') as directory:
        captured = Path(directory) / 'snapshot.json'
        captured.write_bytes(content)
        captured.chmod(0o600)
        validation_path = Path(directory) / 'validation.json'
        subprocess.run([str(args.azimuth.resolve(strict=True)), 'assurance', 'verify-snapshot',
                        '--input', str(captured), '--out', str(validation_path)], check=True)
        validation = json.loads(validation_path.read_text())
    snapshot = json.loads(content)
    if snapshot['project'] != args.project:
        raise ValueError('snapshot does not belong to the requested project')
    if any(review['reviewer'] not in reviewers for review in snapshot['reviews']):
        raise ValueError('a reviewed record has no matching authorized reviewer credential binding')
    revision = snapshot['model_revision']
    requests = [(base + '/model-revisions', snapshot['authority'], owner, 'candidate-authority')]
    requests.extend((base + '/runs', bundle, producer, 'run') for bundle in sorted(snapshot['bundles'], key=lambda bundle: (bundle['run_id'], bundle['bundle_revision'])))
    requests.extend((base + '/model-revisions/' + quote(revision) + '/reviews',
                     review, reviewers[review['reviewer']], review['kind'])
                    for review in sorted(snapshot['reviews'], key=lambda review:
                        (review['kind'] != 'requirement-review', review['reviewed_at_ms'])))
    requests = [(url, payload, token, {'kind': kind, 'fingerprint': payload.get('fingerprint', payload.get('bundle_fingerprint'))})
                for url, payload, token, kind in requests]
    return requests, revision, validation['snapshot_fingerprint']


def submit(opener, url, payload, token):
    request = urllib.request.Request(url, data=json.dumps(payload).encode(), method='POST',
        headers={'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json'})
    with opener.open(request, timeout=30) as response:
        body = response.read(1048577)
        if response.status not in (200, 201) or len(body) > 1048576:
            raise ValueError('invalid publication response status or size')
        json.loads(body)
        return response.status


def write_receipt(output, receipt):
    output.seek(0)
    json.dump(receipt, output, indent=2)
    output.write('\n')
    output.truncate()
    output.flush()


def publish(opener, output, header, assurance, provenance):
    receipt = dict(header, status='incomplete', submissions=[])
    if provenance is not None:
        receipt['provenance_submissions'] = []
    write_receipt(output, receipt)
    for url, payload, token, entry in assurance:
        try:
            status = submit(opener, url, payload, token)
        except urllib.error.HTTPError as error:
            error.close()
            raise ValueError(f'publication rejected: HTTP {error.code}; partial receipt retained') from None
        except urllib.error.URLError:
            raise ValueError('publication connectivity/TLS failure; partial receipt retained') from None
        receipt['submissions'].append(dict(entry, http_status=status))
        write_receipt(output, receipt)
    for url, payload, token, entry in provenance or []:
        try:
            status = submit(opener, url, payload, token)
        except urllib.error.HTTPError as error:
            error.close()
            # A 409 means stored provenance differs from this submission: the service never overwrites it.
            outcome = 'conflict' if error.code == 409 else 'rejected'
            receipt['provenance_submissions'].append(dict(entry, outcome=outcome, http_status=error.code))
            write_receipt(output, receipt)
            raise ValueError(f'provenance publication rejected: HTTP {error.code}; partial receipt retained') from None
        except urllib.error.URLError:
            raise ValueError('publication connectivity/TLS failure; partial receipt retained') from None
        receipt['provenance_submissions'].append(dict(entry, outcome='accepted', http_status=status))
        write_receipt(output, receipt)
    receipt['status'] = 'published'
    write_receipt(output, receipt)
    return receipt


def main(argv=None):
    args = parse_arguments(argv)
    base = project_base(args.server, args.project)
    if args.receipt.exists():
        raise ValueError('receipt already exists')
    assurance, revision, snapshot_fingerprint = [], None, None
    if args.snapshot is not None:
        assurance, revision, snapshot_fingerprint = assurance_requests(args, base)
    provenance = None
    if args.publisher_token_file is not None:
        publisher = credential(args.publisher_token_file)
        ledgers = [provenance_ledger(path, args.project) for path in args.provenance_ledger]
        attributions = [] if args.failure_attributions is None else failure_attributions(args.failure_attributions)
        provenance = provenance_requests(base, publisher, ledgers, attributions)
    header = {'format': 'azimuth-assurance-publication-receipt', 'version': 1,
              'project': args.project, 'model_revision': revision,
              'snapshot_fingerprint': snapshot_fingerprint}
    opener = urllib.request.build_opener(RejectRedirects())
    args.receipt.parent.mkdir(parents=True, exist_ok=True)
    with args.receipt.open('x') as output:
        receipt = publish(opener, output, header, assurance, provenance)
    print(f'Published {len(receipt["submissions"])} immutable records'
          + (f' and {len(receipt["provenance_submissions"])} provenance records' if provenance is not None else ''))


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error)) from None
