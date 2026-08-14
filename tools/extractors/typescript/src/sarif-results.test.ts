import assert from 'node:assert/strict';
import { test } from 'node:test';
import { Manifest } from './emitter';
import { importSarif } from './sarif-results';

test('one SARIF run challenges every linked claim in its analyzed artifacts', () => {
  const linkage: Manifest = {
    realizes: [
      relation('payments/capture', 'amount', 'Capture.Handle', 'app/Capture.cs'),
      relation('security/payment', 'no-secret', 'Capture.Handle', 'app/Capture.cs'),
      relation('unrelated', 'other', 'Other.Handle', 'app/Other.cs'),
    ],
    covers: [], mechanism_implementations: [], mechanism_covers: [], class_members: [],
    enumerations: [], artifacts: [],
  };
  const report = {
    version: '2.1.0',
    runs: [{
      tool: { driver: { name: 'CodeQL', version: '2.23.0' } },
      artifacts: [{ location: { uri: 'app/Capture.cs' } }],
      results: [{ ruleId: 'cs/weak-crypto', level: 'warning', message: { text: 'weak' },
        locations: [{ physicalLocation: { artifactLocation: { uri: 'app/Capture.cs' },
          region: { startLine: 12 } } }] }],
    }],
  };

  const manifest = importSarif({ report, reportPath: 'scan.sarif',
    reportSource: JSON.stringify(report), linkage, root: '/repo', inputs: [] });

  assert.equal(manifest.observations?.length, 1);
  assert.deepEqual(
    manifest.observations?.[0].bindings.map((binding) => `${binding.spec}#${binding.scenario}`),
    ['payments/capture#amount', 'security/payment#no-secret'],
  );
  assert.ok(manifest.observations?.[0].bindings.every((binding) => binding.outcome === 'findings'));
});

test('rejects an unknown SARIF version', () => {
  assert.throws(() => importSarif({ report: { version: '3', runs: [] }, reportPath: 'scan.sarif',
    reportSource: '{}', linkage: empty(), root: '/repo', inputs: [] }), /version 2.1.0/);
});

function relation(spec: string, scenario: string, site: string, file: string) {
  return { spec, scenario, site, file, lang: 'csharp', source_fingerprint: 'abc' };
}

function empty(): Manifest {
  return { realizes: [], covers: [], mechanism_implementations: [], mechanism_covers: [],
    class_members: [], enumerations: [], artifacts: [] };
}
