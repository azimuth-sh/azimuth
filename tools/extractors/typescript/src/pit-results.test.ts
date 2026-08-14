import assert from 'node:assert/strict';
import { test } from 'node:test';
import { Manifest } from './emitter';
import { importPitResults, PitImport } from './pit-results';

test('imports PIT survivors and indeterminate outcomes as claim-scoped review items', () => {
  const reportSource = report([
    mutation('KILLED', 10,
      '<killingTest>rejectsIneligible(fixture.EligibilityTest)</killingTest>', 'true'),
    mutation('SURVIVED', 11),
    mutation('NO_COVERAGE', 12, '', 'false', 0),
    mutation('TIMED_OUT', 13),
    mutation('NON_VIABLE', 14),
    mutation('RUN_ERROR', 15),
    mutation('MEMORY_ERROR', 16),
    mutation('NOT_STARTED', 17),
    mutation('STARTED', 18),
    mutation('EQUIVALENT', 19),
  ]);
  const manifest = importPitResults(input(reportSource));
  const observation = manifest.observations?.[0];

  assert.equal(manifest.observations?.length, 1);
  assert.equal(observation?.tool, 'PIT');
  assert.equal(observation?.kind, 'mutation-test');
  assert.deepEqual(observation?.inputs, ['pom.xml', 'pit-selection.json']);
  assert.equal(observation?.bindings[0].role, 'challenge');
  assert.equal(observation?.bindings[0].outcome, 'findings');
  assert.deepEqual(observation?.bindings[0].subjects, [
    { relation: 'realization',
      identity: 'src/main/java/fixture/Eligibility.java#fixture.Eligibility.evaluate|java' },
    { relation: 'evidence',
      identity: 'src/test/java/fixture/EligibilityTest.java'
        + '#fixture.EligibilityTest.rejectsIneligible|java' },
  ]);
  assert.deepEqual((observation?.payload as any).assessments[0], {
    spec: 'rules/eligibility',
    scenario: 'rejects-ineligible',
    target_files: ['src/main/java/fixture/Eligibility.java'],
    target_classes: ['fixture.Eligibility'],
    test_sites: ['fixture.EligibilityTest.rejectsIneligible'],
    killed: 1,
    survived: 1,
    no_coverage: 1,
    timed_out: 1,
    non_viable: 1,
    runtime_error: 2,
    pending: 2,
    equivalent: 1,
    mutants: [
      review('KILLED', 10, 1, true, 'rejectsIneligible(fixture.EligibilityTest)'),
      review('SURVIVED', 11, 1),
      review('NO_COVERAGE', 12, 0),
      review('TIMED_OUT', 13, 1),
      review('NON_VIABLE', 14, 1),
      review('RUN_ERROR', 15, 1),
      review('MEMORY_ERROR', 16, 1),
      review('NOT_STARTED', 17, 1),
      review('STARTED', 18, 1),
      review('EQUIVALENT', 19, 1),
    ],
    review_items: [
      review('SURVIVED', 11, 1),
      review('NO_COVERAGE', 12, 0),
      review('TIMED_OUT', 13, 1),
      review('NON_VIABLE', 14, 1),
      review('RUN_ERROR', 15, 1),
      review('MEMORY_ERROR', 16, 1),
      review('NOT_STARTED', 17, 1),
      review('STARTED', 18, 1),
      review('EQUIVALENT', 19, 1),
    ],
  });
});

test('emits a clean challenge when every linked mutant is killed', () => {
  const manifest = importPitResults(input(report([
    mutation('KILLED', 10,
      '<killingTest>rejectsIneligible(fixture.EligibilityTest)</killingTest>', 'true'),
    mutation('KILLED', 11,
      '<killingTest>fixture.EligibilityTest.rejectsIneligible</killingTest>', 'true'),
  ])));

  assert.equal(manifest.observations?.[0].bindings[0].outcome, 'clean');
  const assessment = (manifest.observations?.[0].payload as any).assessments[0];
  assert.equal(assessment.killed, 2);
  assert.deepEqual(assessment.review_items, []);
});

test('uses a PIT-named killing test instead of every selected test for the same claim', () => {
  const value = input(report([
    mutation('KILLED', 10,
      '<killingTest>rejectsIneligible(fixture.EligibilityTest)</killingTest>', 'true'),
  ]));
  value.linkage.covers.push(relation(
    'rules/eligibility',
    'rejects-ineligible',
    'fixture.OtherEligibilityTest.rejectsIneligible',
    'src/test/java/fixture/OtherEligibilityTest.java',
  ));
  value.selectionSource = JSON.stringify({
    schema: 'azimuth-pit-selection/1',
    target_classes: ['fixture.Eligibility'],
    selected_tests: [
      { site: 'fixture.EligibilityTest.rejectsIneligible',
        pit_names: ['rejectsIneligible(fixture.EligibilityTest)'] },
      'fixture.OtherEligibilityTest.rejectsIneligible',
    ],
  });

  const binding = importPitResults(value).observations?.[0].bindings[0];
  assert.deepEqual(binding?.subjects.filter((item) => item.relation === 'evidence'), [{
    relation: 'evidence',
    identity: 'src/test/java/fixture/EligibilityTest.java'
      + '#fixture.EligibilityTest.rejectsIneligible|java',
  }]);
});

test('rejects an exact selected test that no longer resolves to Covers', () => {
  const value = input(report([mutation('SURVIVED', 10)]));
  value.selectionSource = JSON.stringify({
    schema: 'azimuth-pit-selection/1',
    target_classes: ['fixture.Eligibility'],
    selected_tests: ['fixture.EligibilityTest.renamed'],
  });

  assert.throws(() => importPitResults(value), /selected test .* resolves to no JVM Covers site/);
});

test('rejects a mutation target that no longer resolves to Realizes', () => {
  const value = input(report([mutation('SURVIVED', 10)]));
  value.linkage.realizes = [];

  assert.throws(() => importPitResults(value), /target class .* resolves to no JVM Realizes site/);
});

test('rejects unknown PIT statuses', () => {
  const reportSource = report([mutation('MYSTERIOUS', 10)]);
  assert.throws(() => importPitResults(input(reportSource)), /unknown status MYSTERIOUS/);
});

test('rejects unknown selection and XML schemas', () => {
  const selection = input(report([mutation('SURVIVED', 10)]));
  selection.selectionSource = JSON.stringify({
    schema: 'azimuth-pit-selection/2',
    target_classes: ['fixture.Eligibility'],
    selected_tests: ['fixture.EligibilityTest.rejectsIneligible'],
  });
  assert.throws(
    () => importPitResults(selection),
    /selection schema must be azimuth-pit-selection\/1/,
  );

  assert.throws(
    () => importPitResults(input('<mutationReport/>')),
    /schema must have a mutations root/,
  );
  assert.throws(
    () => importPitResults(input('<mutations future="true"/>')),
    /unknown field future/,
  );
  assert.throws(
    () => importPitResults(input('<mutations partial="sometimes"/>')),
    /partial must be true or false/,
  );
  assert.throws(
    () => importPitResults(input(report([
      mutation('SURVIVED', 10, '<futureField>x</futureField>'),
    ]))),
    /unknown element futureField/,
  );

  const malformedSelection = input(report([mutation('SURVIVED', 10)]));
  malformedSelection.selectionSource = '{';
  assert.throws(() => importPitResults(malformedSelection), /selection must be valid JSON/);
});

test('accepts and preserves the partial marker emitted by PIT 1.22', () => {
  const imported = importPitResults(input(
    report([mutation('SURVIVED', 10)]).replace('<mutations>', '<mutations partial="true">'),
  ));

  const payload = imported.observations?.[0].payload as { report_partial?: boolean };
  assert.equal(payload.report_partial, true);
});

test('rejects malformed PIT XML and document declarations', () => {
  assert.throws(
    () => importPitResults(input('<mutations><mutation></mutations>')),
    /malformed PIT XML: unexpected closing tag/,
  );
  assert.throws(
    () => importPitResults(input('<!DOCTYPE mutations><mutations/>')),
    /document declarations are not supported/,
  );
});

test('fingerprints the report, PIT config, selection and tool version', () => {
  const reportSource = report([mutation('SURVIVED', 10)]);
  const original = importPitResults(input(reportSource)).observations?.[0].source_fingerprint;
  const changedConfig = input(reportSource);
  changedConfig.configSource = '<configuration><threads>2</threads></configuration>';
  const next = importPitResults(changedConfig).observations?.[0].source_fingerprint;

  assert.notEqual(original, next);
});

function input(reportSource: string): PitImport {
  const linkage: Manifest = {
    realizes: [relation(
      'rules/eligibility',
      'rejects-ineligible',
      'fixture.Eligibility.evaluate',
      'src/main/java/fixture/Eligibility.java',
    )],
    covers: [relation(
      'rules/eligibility',
      'rejects-ineligible',
      'fixture.EligibilityTest.rejectsIneligible',
      'src/test/java/fixture/EligibilityTest.java',
    )],
    mechanism_implementations: [],
    mechanism_covers: [],
    class_members: [],
    enumerations: [],
    artifacts: [],
  };
  const selection = {
    schema: 'azimuth-pit-selection/1',
    target_classes: ['fixture.Eligibility'],
    selected_tests: [{
      site: 'fixture.EligibilityTest.rejectsIneligible',
      pit_names: ['rejectsIneligible(fixture.EligibilityTest)'],
    }],
  };
  return {
    reportPath: 'target/pit-reports/mutations.xml',
    reportSource,
    configPath: 'pom.xml',
    configSource: '<configuration/>',
    selectionPath: 'pit-selection.json',
    selectionSource: JSON.stringify(selection),
    linkage,
    toolVersion: '1.25.3',
  };
}

function relation(spec: string, scenario: string, site: string, file: string) {
  return { spec, scenario, site, file, lang: 'java', source_fingerprint: 'abc' };
}

function report(mutations: string[]): string {
  return `<?xml version="1.0" encoding="UTF-8"?><mutations>${mutations.join('')}</mutations>`;
}

function mutation(
  status: string,
  line: number,
  extra = '',
  detected = 'false',
  testsRun = 1,
): string {
  return `<mutation detected="${detected}" status="${status}" numberOfTestsRun="${testsRun}">
    <sourceFile>Eligibility.java</sourceFile>
    <mutatedClass>fixture.Eligibility</mutatedClass>
    <mutatedMethod>evaluate</mutatedMethod>
    <methodDescription>()I</methodDescription>
    <lineNumber>${line}</lineNumber>
    <mutator>org.pitest.mutationtest.engine.gregor.mutators.ReturnValsMutator</mutator>
    <indexes><index>1</index></indexes>
    <blocks><block>0</block></blocks>
    <description>replaced return value</description>
    ${extra}
  </mutation>`;
}

function review(
  status: string,
  line: number,
  testsRun: number,
  detected = false,
  killingTest = '',
) {
  return {
    file: 'src/main/java/fixture/Eligibility.java',
    line,
    status,
    detected,
    mutated_class: 'fixture.Eligibility',
    mutated_method: 'evaluate',
    method_description: '()I',
    mutator: 'org.pitest.mutationtest.engine.gregor.mutators.ReturnValsMutator',
    description: 'replaced return value',
    tests_run: testsRun,
    ...(killingTest ? { killing_test: killingTest } : {}),
  };
}
