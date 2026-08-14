import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import { importManualResults } from './manual-results';

function run(status: 'passed' | 'failed' = 'passed'): unknown {
  return {
    provider: 'testrail',
    run_id: 'run-7',
    observed_at: '2026-08-08T01:00:00Z',
    expires_at: '2026-09-08T01:00:00Z',
    results: [{
      case_id: 'case-42',
      spec: 'payments/capture',
      scenario: 'receipt-explains-payment-state',
      status,
      scope: 'e2e',
      quantification: 'example',
      url: 'https://tracker.example/runs/7#42',
    }],
  };
}

test('imports an attributable manual pass as a covering evidence receipt', () => {
  const entry = importManualResults(run()).covers[0];
  assert.equal(entry.site, 'testrail:case-42');
  assert.equal(entry.evidence_kind, 'manual-test');
  assert.equal(entry.evidence_outcome, 'passed');
  assert.equal(entry.observed_at, '2026-08-08T01:00:00Z');
  assert.equal(entry.expires_at, 1788829200);
  assert.match(entry.source_fingerprint, /^[0-9a-f]{64}$/);
});

test('preserves a failed outcome instead of dropping it', () => {
  assert.equal(importManualResults(run('failed')).covers[0].evidence_outcome, 'failed');
});

test('rejects provider-specific status vocabulary until an adapter maps it', () => {
  const input = run() as { results: Array<{ status: string }> };
  input.results[0].status = 'blocked';
  assert.throws(() => importManualResults(input), /must be passed or failed/);
});
