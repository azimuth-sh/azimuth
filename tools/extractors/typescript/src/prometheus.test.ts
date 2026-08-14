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

test('emits explicit operational realization and evidence from validated rules', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-prometheus-linkage-'));
  const rules = path.join(root, 'alerts.yml');
  const tests = path.join(root, 'alerts.test.yml');
  fs.writeFileSync(rules, `# azimuth-realizes: operations/delivery backlog-alert\n- alert: Backlog\n`);
  fs.writeFileSync(tests, `# azimuth-covers: operations/delivery backlog-alert unit example direct\nalertname: Backlog\n`);

  const linkage = prometheusLinkage(rules, tests, root);

  assert.equal(linkage.realizes[0].site, 'Backlog');
  assert.deepEqual(linkage.covers[0], {
    spec: 'operations/delivery', scenario: 'backlog-alert', site: 'Backlog',
    file: 'alerts.test.yml', lang: 'prometheus',
    source_fingerprint: linkage.covers[0].source_fingerprint,
    scope: 'unit', quantification: 'example', oracle: 'direct',
  });

  fs.writeFileSync(rules, `# azimuth-realizes: operations/delivery backlog-alert\n- alert: Backlog\n  expr: backlog > 2\n`);
  const changed = prometheusLinkage(rules, tests, root);
  assert.notEqual(changed.realizes[0].source_fingerprint, linkage.realizes[0].source_fingerprint);
});
