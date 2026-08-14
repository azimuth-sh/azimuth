import assert from 'node:assert/strict';
import { test } from 'node:test';
import { importObservation } from './observations';

test('one load execution retains independent claim assertions', () => {
  const manifest = importObservation({
    export: {
      id: 'load-checkout-42', kind: 'load-test', tool: 'k6', tool_version: '1.0',
      observed_at: '2026-08-11T12:00:00Z', expires_at: 1789000000,
      bindings: [
        evidence('checkout/performance', 'latency', 'p95 latency < 300 ms'),
        evidence('checkout/performance', 'errors', 'error rate < 0.5 percent'),
      ],
      payload: { p95_ms: 241, error_rate: 0.001 },
    },
    reportPath: 'reports/load.json', reportSource: '{"run":42}',
    inputs: [{ path: 'tests/load.js', source: 'export default function() {}' }],
  });

  assert.equal(manifest.observations?.length, 1);
  assert.equal(manifest.observations?.[0].bindings.length, 2);
  assert.notEqual(
    manifest.observations?.[0].bindings[0].assertion,
    manifest.observations?.[0].bindings[1].assertion,
  );
});

test('one chaos execution may cover recovery and alerting without a blanket outcome', () => {
  const manifest = importObservation({
    export: {
      id: 'broker-loss-7', kind: 'chaos-experiment', tool: 'Chaos Mesh', tool_version: '2.7',
      observed_at: '2026-08-11T13:00:00Z', expires_at: 1789000000,
      bindings: [
        evidence('delivery/resilience', 'recovers', 'queue drains within 120 seconds'),
        evidence('delivery/operations', 'alerts', 'backlog alert arrives within 90 seconds'),
      ],
    },
    reportPath: 'reports/chaos.json', reportSource: '{"experiment":"broker-loss"}',
    inputs: [{ path: 'chaos/broker-loss.yml', source: 'kind: NetworkChaos' }],
  });

  assert.deepEqual(
    manifest.observations?.[0].bindings.map((binding) => binding.scenario),
    ['recovers', 'alerts'],
  );
});

test('rejects a broad passed bit without claim-specific evidence forms', () => {
  assert.throws(() => importObservation({
    export: {
      id: 'blanket', kind: 'chaos-experiment', tool: 'chaos', tool_version: '1',
      observed_at: '2026-08-11T13:00:00Z', expires_at: 1789000000,
      bindings: [{ role: 'evidence', spec: 'delivery', scenario: 'all', assertion: 'passed',
        outcome: 'satisfied', subjects: [] }],
    },
    reportPath: 'report.json', reportSource: '{}', inputs: [],
  }), /complete form/);
});

function evidence(spec: string, scenario: string, assertion: string) {
  return {
    role: 'evidence', spec, scenario, assertion, outcome: 'satisfied', subjects: [],
    scope: 'e2e', quantification: 'example', oracle: 'direct',
  };
}
