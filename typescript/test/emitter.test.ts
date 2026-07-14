import { strict as assert } from 'node:assert';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { test } from 'node:test';
import { emit, scanText } from '../src/emitter';

const fixturesDir = path.join(__dirname, 'fixtures');

test('emits realizes + covers from the sample fixture with resolved sites and form', () => {
  const { manifest, warnings } = emit({ root: fixturesDir, include: ['sample.ts'] });

  assert.deepEqual(warnings, []);

  assert.deepEqual(manifest.realizes, [
    {
      spec: 'public-certificates',
      req: 'detail',
      scenario: 'detail-valid',
      site: 'CertificatePage',
      file: 'sample.ts',
      lang: 'typescript',
    },
    {
      spec: 'public-certificates',
      req: 'detail',
      scenario: 'detail-valid',
      site: 'GET',
      file: 'sample.ts',
      lang: 'typescript',
    },
  ]);

  const revoked = manifest.covers.find((c) => c.scenario === 'detail-revoked-void');
  assert.ok(revoked);
  assert.equal(revoked.site, 'revoked certificate returns 404');
  assert.equal(revoked.scope, 'component');
  assert.equal(revoked.quantification, 'invariant');
  assert.equal(revoked.oracle, 'direct');
  assert.equal(revoked.lang, 'typescript');

  const unpublished = manifest.covers.find((c) => c.scenario === 'detail-unpublished');
  assert.ok(unpublished);
  assert.equal(unpublished.site, 'unpublished is an indistinguishable 404');
  assert.equal(unpublished.scope, 'component');
  assert.equal(unpublished.quantification, 'example');
  assert.equal(unpublished.oracle, undefined);
});

test('the emitted manifest validates against the shared schema shape', () => {
  const { manifest } = emit({ root: fixturesDir, include: ['sample.ts'] });
  for (const entry of manifest.realizes) {
    assert.deepEqual(Object.keys(entry).sort(), ['file', 'lang', 'req', 'scenario', 'site', 'spec']);
  }
  for (const entry of manifest.covers) {
    assert.ok(['unit', 'component', 'e2e'].includes(entry.scope));
    assert.ok(['example', 'invariant'].includes(entry.quantification));
  }
});

test('resolves the enclosing named function as the site', () => {
  const text = [
    'export function DoThing() {',
    "  realizes('demo', 'do', 'it-works');",
    '}',
  ].join('\n');
  const { realizes: entries } = scanText(text, 'a.ts');
  assert.equal(entries.length, 1);
  assert.equal(entries[0].site, 'DoThing');
});

test('a covers with an unknown scope is skipped with a warning', () => {
  const text = "test('t', () => { covers('d', 'r', 's', 'wide', 'invariant'); });";
  const { covers: entries, warnings } = scanText(text, 'a.ts');
  assert.equal(entries.length, 0);
  assert.equal(warnings.length, 1);
  assert.match(warnings[0].message, /unknown scope/);
});

test('a marker call with a non-literal argument is skipped with a warning', () => {
  const text = [
    'const spec = getSpec();',
    'export function GET() {',
    '  realizes(spec, "do", "it-works");',
    '}',
  ].join('\n');
  const { realizes: entries, warnings } = scanText(text, 'a.ts');
  assert.equal(entries.length, 0);
  assert.equal(warnings.length, 1);
});

test('a covers without an oracle omits the field entirely', () => {
  const text = "test('t', () => { covers('d', 'r', 's', 'unit', 'example'); });";
  const { covers: entries } = scanText(text, 'a.ts');
  assert.equal(entries.length, 1);
  assert.equal('oracle' in entries[0], false);
});

test('scans .tsx and finds markers inside JSX-bearing components', () => {
  const text = [
    'export const Page = () => {',
    "  realizes('demo', 'render', 'renders-ok');",
    '  return <div>ok</div>;',
    '};',
  ].join('\n');
  const { realizes: entries } = scanText(text, 'page.tsx');
  assert.equal(entries.length, 1);
  assert.equal(entries[0].site, 'Page');
});

test('a source with no markers yields empty arrays', () => {
  const { manifest } = emit({ root: fixturesDir, include: ['does-not-exist-*.ts'] });
  assert.deepEqual(manifest.realizes, []);
  assert.deepEqual(manifest.covers, []);
});

test('the fixture file is on disk (guards the emit path against silent empty scans)', () => {
  assert.ok(fs.existsSync(path.join(fixturesDir, 'sample.ts')));
});
