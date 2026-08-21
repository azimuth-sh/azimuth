import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import { prometheusArtifacts, prometheusLinkage } from './prometheus';

test('enumerates alert rules and their detector-test cases as separate artifacts', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-prometheus-'));
  const rules = path.join(root, 'rules.yml');
  const tests = path.join(root, 'tests.yml');
  fs.writeFileSync(rules, 'rules:\n  - alert: WorkerSilent\n  # - alert: NotARule\n  - alert: WorkOverdue\n');
  fs.writeFileSync(tests, 'alert_rule_test:\n  - alertname: WorkerSilent\n  - alertname: WorkOverdue\n');

  const artifacts = prometheusArtifacts(rules, tests, root);

  assert.deepEqual(
    artifacts.map((artifact) => artifact.id),
    [
      'prometheus-alert:WorkerSilent',
      'prometheus-alert:WorkOverdue',
      'prometheus-rule-test:WorkerSilent',
      'prometheus-rule-test:WorkOverdue',
    ],
  );
  assert.ok(artifacts.every((artifact) => artifact.file === 'rules.yml' || artifact.file === 'tests.yml'));
});

test('fails closed when no executable alert or test can be enumerated', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-prometheus-'));
  const rules = path.join(root, 'rules.yml');
  const tests = path.join(root, 'tests.yml');
  fs.writeFileSync(rules, '# - alert: CommentOnly\n');
  fs.writeFileSync(tests, '# no tests\n');

  assert.throws(() => prometheusArtifacts(rules, tests, root), /contains no alert rules/);
});

test('emits operational realization and Check implementation from validated rules', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-prometheus-linkage-'));
  const rules = path.join(root, 'alerts.yml');
  const tests = path.join(root, 'alerts.test.yml');
  fs.writeFileSync(rules, `# azimuth-realizes: operations/delivery backlog-alert\n- alert: Backlog\n`);
  fs.writeFileSync(
    tests,
    `# azimuth-implements-check: operations/backlog-alert\nalertname: Backlog\n`,
  );

  const linkage = prometheusLinkage(rules, tests, root);

  assert.equal(linkage.realizes[0].site, 'Backlog');
  assert.match(linkage.realizes[0].source_fingerprint, /^sha256:[0-9a-f]{64}$/);
  assert.deepEqual(linkage.check_implementations[0], {
    check: 'operations/backlog-alert', site: 'Backlog',
    file: 'alerts.test.yml', lang: 'prometheus',
    source_fingerprint: linkage.check_implementations[0].source_fingerprint,
  });
  assert.match(
    linkage.check_implementations[0].source_fingerprint,
    /^sha256:[0-9a-f]{64}$/,
  );

  fs.writeFileSync(rules, `# azimuth-realizes: operations/delivery backlog-alert\n- alert: Backlog\n  expr: backlog > 2\n`);
  const changed = prometheusLinkage(rules, tests, root);
  assert.notEqual(changed.realizes[0].source_fingerprint, linkage.realizes[0].source_fingerprint);
  assert.equal(
    changed.check_implementations[0].source_fingerprint,
    linkage.check_implementations[0].source_fingerprint,
  );
});

test('Check fingerprints are confined to their marker-delimited rule-test site', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-prometheus-fingerprint-'));
  const rules = path.join(root, 'alerts.yml');
  const tests = path.join(root, 'alerts.test.yml');
  fs.writeFileSync(rules, '- alert: First\n- alert: Second\n');
  fs.writeFileSync(
    tests,
    `# azimuth-implements-check: operations/shared\nalertname: First\nvalue: 1\n` +
      `# azimuth-implements-check: operations/shared\nalertname: Second\nvalue: 2\n`,
  );
  const before = prometheusLinkage(rules, tests, root);

  fs.writeFileSync(
    tests,
    `# azimuth-implements-check: operations/shared\nalertname: First\nvalue: 1\n` +
      `# azimuth-implements-check: operations/shared\nalertname: Second\nvalue: 3\n`,
  );
  const after = prometheusLinkage(rules, tests, root);

  assert.equal(before.check_implementations.length, 2);
  assert.equal(
    before.check_implementations[0].source_fingerprint,
    after.check_implementations[0].source_fingerprint,
  );
  assert.notEqual(
    before.check_implementations[1].source_fingerprint,
    after.check_implementations[1].source_fingerprint,
  );
});

test('retired Prometheus evidence keys fail explicitly', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-prometheus-removed-'));
  const rules = path.join(root, 'alerts.yml');
  const tests = path.join(root, 'alerts.test.yml');
  fs.writeFileSync(rules, '- alert: Backlog\n');
  fs.writeFileSync(
    tests,
    '# azimuth-covers: operations/delivery backlog-alert unit example direct\n' +
      'alertname: Backlog\n',
  );

  assert.throws(
    () => prometheusLinkage(rules, tests, root),
    /retired alpha 1 marker `azimuth-covers` is not supported/,
  );
});

test('ordinary comments containing covers remain unrelated', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-prometheus-unrelated-'));
  const rules = path.join(root, 'alerts.yml');
  const tests = path.join(root, 'alerts.test.yml');
  fs.writeFileSync(rules, '- alert: Backlog\n');
  fs.writeFileSync(tests, '# this test covers backlog behavior\nalertname: Backlog\n');

  assert.deepEqual(prometheusLinkage(rules, tests, root).check_implementations, []);
});
