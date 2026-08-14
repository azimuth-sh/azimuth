import assert from 'node:assert/strict';
import { test } from 'node:test';
import { Manifest } from './emitter';
import { importMutationResults } from './mutation-results';

test('derives assessed claims from selected Covers tests and mutated Realizes files', () => {
  const linkage: Manifest = {
    realizes: [
      relation('payments/capture', 'amount', 'Capture.Handle', 'app/Capture.cs'),
      relation('payments/capture', 'unrelated', 'Capture.Handle', 'app/Capture.cs'),
    ],
    covers: [
      relation('payments/capture', 'amount', 'Tests.Amount', 'tests/CaptureTests.cs'),
      relation('payments/capture', 'unrelated', 'Tests.Other', 'tests/CaptureTests.cs'),
    ],
    mechanism_implementations: [],
    mechanism_covers: [],
    class_members: [],
    enumerations: [],
    artifacts: [],
  };
  const report = {
    schemaVersion: '2',
    files: {
      '/repo/app/Capture.cs': {
        mutants: [mutant('Killed', 10), mutant('Survived', 11), mutant('NoCoverage', 12)],
      },
    },
    testFiles: {
      '/repo/tests/CaptureTests.cs': { tests: [{ id: '1', name: 'Tests.Amount' }] },
    },
  };

  const manifest = importMutationResults({
    report,
    reportPath: 'report.json',
    reportSource: JSON.stringify(report),
    configPath: 'stryker-config.json',
    configSource: '{}',
    linkage,
    root: '/repo',
    toolVersion: '4.16.0',
  });

  assert.equal(manifest.observations?.length, 1);
  const observation = manifest.observations?.[0];
  assert.deepEqual(observation, {
    id: observation?.id,
    kind: 'mutation-test',
    tool: 'Stryker.NET',
    tool_version: '4.16.0',
    report: 'report.json',
    inputs: ['stryker-config.json'],
    source_fingerprint: observation?.source_fingerprint,
    bindings: [{
      role: 'challenge',
      spec: 'payments/capture',
      scenario: 'amount',
      assertion: 'selected evidence rejects generated changes at linked realization sites',
      outcome: 'findings',
      subjects: [
        { relation: 'realization', identity: 'app/Capture.cs#Capture.Handle|csharp' },
        { relation: 'evidence', identity: 'tests/CaptureTests.cs#Tests.Amount|csharp' },
      ],
    }],
    payload: {
      schema: 'azimuth-mutation-summary/1',
      assessments: [{
        spec: 'payments/capture', scenario: 'amount', target_files: ['app/Capture.cs'],
        test_sites: ['Tests.Amount'], killed: 1, survived: 1, no_coverage: 1, timeout: 0,
        compile_error: 0, runtime_error: 0, pending: 0, ignored: 0,
        review_items: [
          { file: 'app/Capture.cs', line: 11, status: 'Survived', mutator: 'test', replacement: 'wrong' },
          { file: 'app/Capture.cs', line: 12, status: 'NoCoverage', mutator: 'test', replacement: 'wrong' },
        ],
      }],
    },
  });
});

test('rejects a mutation status it cannot account for', () => {
  assert.throws(() => importMutationResults({
    report: {
      schemaVersion: '2',
      files: { '/repo/app/Capture.cs': { mutants: [{ ...mutant('Killed', 1), status: 'Mysterious' }] } },
      testFiles: {},
    },
    reportPath: 'report.json',
    reportSource: '{}',
    configPath: 'stryker-config.json',
    configSource: '{}',
    linkage: {
      realizes: [], covers: [], mechanism_implementations: [], mechanism_covers: [],
      class_members: [], enumerations: [], artifacts: [],
    },
    root: '/repo',
    toolVersion: '4.16.0',
  }), /unknown status Mysterious/);
});

test('rejects an unknown Stryker report schema', () => {
  assert.throws(() => importMutationResults({
    report: { schemaVersion: '3', files: {}, testFiles: {} },
    reportPath: 'report.json',
    reportSource: '{}',
    configPath: 'stryker-config.json',
    configSource: '{}',
    linkage: {
      realizes: [], covers: [], mechanism_implementations: [], mechanism_covers: [],
      class_members: [], enumerations: [], artifacts: [],
    },
    root: '/repo',
    toolVersion: '4.16.0',
  }), /schemaVersion must be 2/);
});

test('rejects an assessment for which filters left no executable mutants', () => {
  const linkage: Manifest = {
    realizes: [relation('payments/capture', 'amount', 'Capture.Handle', 'app/Capture.cs')],
    covers: [relation('payments/capture', 'amount', 'Tests.Amount', 'tests/CaptureTests.cs')],
    mechanism_implementations: [], mechanism_covers: [], class_members: [], enumerations: [],
    artifacts: [],
  };
  assert.throws(() => importMutationResults({
    report: {
      schemaVersion: '2',
      files: { '/repo/app/Capture.cs': { mutants: [mutant('Ignored', 1)] } },
      testFiles: {
        '/repo/tests/CaptureTests.cs': { tests: [{ id: '1', name: 'Tests.Amount' }] },
      },
    },
    reportPath: 'report.json', reportSource: '{}', configPath: 'stryker-config.json',
    configSource: '{}', linkage, root: '/repo', toolVersion: '4.16.0',
  }), /no executed mutants/);
});

function relation(spec: string, scenario: string, site: string, file: string) {
  return { spec, scenario, site, file, lang: 'csharp', source_fingerprint: 'abc' };
}

function mutant(status: string, line: number) {
  return {
    status,
    mutatorName: 'test',
    replacement: 'wrong',
    location: { start: { line, column: 1 }, end: { line, column: 2 } },
  };
}
