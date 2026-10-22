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


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--azimuth', required=True, type=Path)
    parser.add_argument('--snapshot', required=True, type=Path)
    parser.add_argument('--project', required=True)
    parser.add_argument('--server', required=True)
    parser.add_argument('--owner-token-file', required=True, type=Path)
    parser.add_argument('--producer-token-file', required=True, type=Path)
    parser.add_argument('--reviewer-token-file', action='append', default=[])
    parser.add_argument('--receipt', required=True, type=Path)
    args = parser.parse_args()
    origin = urllib.parse.urlsplit(args.server)
    if origin.scheme != 'https' or not origin.hostname or origin.username or origin.password or origin.path not in ('', '/') or origin.query or origin.fragment:
        raise ValueError('server must be an HTTPS origin without credentials, path or query')
    if args.receipt.exists():
        raise ValueError('receipt already exists')
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
    quote = lambda value: urllib.parse.quote(value, safe='')
    base = args.server.rstrip('/') + '/api/v1/projects/' + quote(args.project)
    revision = snapshot['model_revision']
    requests = [(base + '/model-revisions', snapshot['authority'], owner, 'candidate-authority')]
    requests.extend((base + '/runs', bundle, producer, 'run') for bundle in sorted(snapshot['bundles'], key=lambda bundle: (bundle['run_id'], bundle['bundle_revision'])))
    requests.extend((base + '/model-revisions/' + quote(revision) + '/reviews',
                     review, reviewers[review['reviewer']], review['kind'])
                    for review in sorted(snapshot['reviews'], key=lambda review:
                        (review['kind'] != 'requirement-review', review['reviewed_at_ms'])))
    receipts = []
    opener = urllib.request.build_opener(RejectRedirects())
    args.receipt.parent.mkdir(parents=True, exist_ok=True)
    with args.receipt.open('x') as output:
        def persist(status):
            output.seek(0)
            json.dump({'format': 'azimuth-assurance-publication-receipt', 'version': 1,
                       'project': args.project, 'model_revision': revision,
                       'snapshot_fingerprint': validation['snapshot_fingerprint'],
                       'status': status,
                       'submissions': receipts}, output, indent=2)
            output.write('\n')
            output.truncate()
            output.flush()
        persist('incomplete')
        for url, payload, token, kind in requests:
            request = urllib.request.Request(url, data=json.dumps(payload).encode(), method='POST',
                headers={'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json'})
            try:
                with opener.open(request, timeout=30) as response:
                    body = response.read(1048577)
                    if response.status not in (200, 201) or len(body) > 1048576:
                        raise ValueError('invalid publication response status or size')
                    json.loads(body)
                    receipts.append({'kind': kind, 'fingerprint': payload.get('fingerprint', payload.get('bundle_fingerprint')),
                                     'http_status': response.status})
                    persist('incomplete')
            except urllib.error.HTTPError as error:
                raise ValueError(f'publication rejected: HTTP {error.code}; partial receipt retained') from None
            except urllib.error.URLError:
                raise ValueError('publication connectivity/TLS failure; partial receipt retained') from None
        persist('published')
    print(f'Published {len(receipts)} immutable records')


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error)) from None
