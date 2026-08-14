import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import { nextRoutes, scanText } from './emitter';

// Synthetic sources only (D2). A silently wrong emitter produces a green matrix, which is the exact
// failure the framework exists to prevent — so these assert on the shape of what is emitted, not
// merely that something was.

test('resolves a site to its enclosing function', () => {
  const result = scanText(
    `import { realizes } from '@azimuth/annotations';
     export function handler() { realizes('alpha', 'route-thing'); }`,
    'a.ts',
  );
  assert.equal(result.realizes.length, 1);
  assert.deepEqual(
    { spec: result.realizes[0].spec, scenario: result.realizes[0].scenario, site: result.realizes[0].site },
    { spec: 'alpha', scenario: 'route-thing', site: 'handler' },
  );
  assert.equal(result.realizes[0].lang, 'typescript');
});

test('resolves a site to a named binding an arrow was assigned to', () => {
  const result = scanText(
    `export const projection = () => { realizes('alpha', 'thing'); };`,
    'a.ts',
  );
  assert.equal(result.realizes[0].site, 'projection');
});

test('resolves a site to a class method', () => {
  const result = scanText(
    `class Trip { complete() { realizes('alpha', 'thing'); } }`,
    'a.ts',
  );
  assert.equal(result.realizes[0].site, 'complete');
});

test('a site may realize several claims', () => {
  const result = scanText(
    `function f() { realizes('alpha', 'first'); realizes('alpha', 'second'); }`,
    'a.ts',
  );
  assert.deepEqual(
    result.realizes.map((r) => r.scenario),
    ['first', 'second'],
  );
  assert.deepEqual(new Set(result.realizes.map((r) => r.site)), new Set(['f']));
});

// A covers inside test('…') names the test, which is what a human would look for.
test('a covers inside a test names the test', () => {
  const result = scanText(
    `test('the route answers', () => { covers('alpha', 'route-thing', 'component', 'universal'); });`,
    'a.test.ts',
  );
  assert.equal(result.covers[0].site, 'the route answers');
  assert.equal(result.covers[0].scope, 'component');
  assert.equal(result.covers[0].quantification, 'universal');
  assert.equal(result.covers[0].oracle, undefined);
  assert.match(result.covers[0].source_fingerprint, /^[0-9a-f]{64}$/);
});

test('a site fingerprint changes only when that site changes', () => {
  const before = scanText(
    `test('first', () => { covers('a', 'first', 'unit', 'example'); assert(1); });
     test('second', () => { covers('a', 'second', 'unit', 'example'); assert(2); });`,
    'a.test.ts',
  );
  const after = scanText(
    `test('first', () => { covers('a', 'first', 'unit', 'example'); assert(1); });
     test('second', () => { covers('a', 'second', 'unit', 'example'); assert(3); });`,
    'a.test.ts',
  );

  assert.equal(before.covers[0].source_fingerprint, after.covers[0].source_fingerprint);
  assert.notEqual(before.covers[1].source_fingerprint, after.covers[1].source_fingerprint);
});

test('a relational oracle is carried when given', () => {
  const result = scanText(
    `test('t', () => { covers('a', 's', 'e2e', 'example', 'relational'); });`,
    'a.test.ts',
  );
  assert.equal(result.covers[0].oracle, 'relational');
});

test('a mechanism implementation derives a symbol binding', () => {
  const result = scanText(
    `export function selectBranch() { implementsMechanism('alpha', 'branch-selection'); }`,
    'src/branch.ts',
  );
  assert.deepEqual(result.mechanismImplementations[0], {
    spec: 'alpha',
    mechanism: 'branch-selection',
    binding: 'typescript-symbol:src/branch.ts#selectBranch',
    file: 'src/branch.ts',
    lang: 'typescript',
    source_fingerprint: result.mechanismImplementations[0].source_fingerprint,
  });
});

test('mechanism evidence carries the checking form', () => {
  const result = scanText(
    `test('all branches', () => {
       coversMechanism('alpha', 'branch-selection', 'unit', 'universal', 'model-based');
     });`,
    'src/branch.test.ts',
  );
  assert.deepEqual(
    {
      spec: result.mechanismCovers[0].spec,
      mechanism: result.mechanismCovers[0].mechanism,
      site: result.mechanismCovers[0].site,
      scope: result.mechanismCovers[0].scope,
      quantification: result.mechanismCovers[0].quantification,
      oracle: result.mechanismCovers[0].oracle,
    },
    {
      spec: 'alpha',
      mechanism: 'branch-selection',
      site: 'all branches',
      scope: 'unit',
      quantification: 'universal',
      oracle: 'model-based',
    },
  );
});

// Form is how a test checks, not a property of code — so realizes never carries one, and the
// emitter has no way to attach one.
test('realizes carries no form', () => {
  const result = scanText(`function f() { realizes('a', 's'); }`, 'a.ts');
  assert.equal('scope' in result.realizes[0], false);
  assert.equal('quantification' in result.realizes[0], false);
});

test('an unknown scope is a warning, not a silent entry', () => {
  const result = scanText(
    `test('t', () => { covers('a', 's', 'integration', 'example'); });`,
    'a.test.ts',
  );
  assert.equal(result.covers.length, 0);
  assert.equal(result.warnings.length, 1);
  assert.match(result.warnings[0].message, /unknown scope `integration`/);
});

test('an unknown quantification is a warning', () => {
  const result = scanText(
    `test('t', () => { covers('a', 's', 'unit', 'property'); });`,
    'a.test.ts',
  );
  assert.equal(result.covers.length, 0);
  assert.match(result.warnings[0].message, /unknown quantification/);
});

test('a covers missing its form is a warning', () => {
  const result = scanText(`test('t', () => { covers('a', 's'); });`, 'a.test.ts');
  assert.equal(result.covers.length, 0);
  assert.match(result.warnings[0].message, /needs a spec, a scenario id, a scope/);
});

test('warnings carry a line number', () => {
  const result = scanText(
    `\n\ntest('t', () => { covers('a', 's'); });`,
    'a.test.ts',
  );
  assert.equal(result.warnings[0].line, 3);
  assert.equal(result.warnings[0].file, 'a.test.ts');
});

test('an untagged test is outside the evidence model', () => {
  const result = scanText(
    `test('covered', () => { covers('a', 's', 'unit', 'example'); });
     test('bare', () => { const x = 1; });`,
    'a.test.ts',
  );
  assert.deepEqual(Object.keys(result).sort(), [
    'covers',
    'mechanismCovers',
    'mechanismImplementations',
    'realizes',
    'warnings',
  ]);
});

test('tsx parses', () => {
  const result = scanText(
    `export function View() { realizes('a', 's'); return <div className="x" />; }`,
    'a.tsx',
  );
  assert.equal(result.realizes.length, 1);
  assert.equal(result.realizes[0].site, 'View');
});

test('javascript uses the same compiler parser but keeps its language identity', () => {
  const result = scanText(
    `export function handler() { realizes('a', 's'); }`,
    'service.js',
  );

  assert.equal(result.realizes[0].lang, 'javascript');
  assert.equal(result.realizes[0].site, 'handler');
});

// Nothing outside a marker call is a tag. A string that merely mentions one is prose.
test('a mention of a marker in a string is not a tag', () => {
  const result = scanText(`const doc = "call realizes('a', 's') to tag a site";`, 'a.ts');
  assert.deepEqual(result.realizes, []);
});

function builtApp(routes: Record<string, string>, sources: string[]): string {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-app-'));
  fs.mkdirSync(path.join(dir, '.next'), { recursive: true });
  fs.writeFileSync(
    path.join(dir, '.next', 'app-path-routes-manifest.json'),
    JSON.stringify(routes),
  );
  for (const source of sources) {
    const full = path.join(dir, 'src', 'app', source);
    fs.mkdirSync(path.dirname(full), { recursive: true });
    fs.writeFileSync(full, '');
  }
  return dir;
}

test('enumerates class members from the build output, tagged or not', () => {
  const dir = builtApp(
    { '/page': '/', '/api/thing/route': '/api/thing', '/_not-found/page': '/_not-found' },
    ['page.tsx', 'api/thing/route.ts'],
  );

  const { members, warnings } = nextRoutes('beta', dir, dir);

  assert.equal(warnings.length, 0);
  assert.deepEqual(
    members.map((m) => m.site).sort(),
    ['/', '/api/thing'],
    'framework-generated pages are not sites the project wrote',
  );
  assert.ok(members.every((m) => m.class === 'beta'));
  assert.deepEqual(
    members.map((m) => m.file).sort(),
    ['src/app/api/thing/route.ts', 'src/app/page.tsx'],
  );
});

test('warns rather than silently narrowing the class when the app is not built', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-app-'));

  const { members, warnings } = nextRoutes('beta', dir, dir);

  assert.equal(members.length, 0);
  assert.equal(warnings.length, 1);
  assert.match(warnings[0].message, /report green over the difference/);
});

test('warns when a route has no source, rather than dropping it in silence', () => {
  const dir = builtApp({ '/ghost/page': '/ghost' }, []);

  const { members, warnings } = nextRoutes('beta', dir, dir);

  assert.equal(members.length, 0);
  assert.equal(warnings.length, 1);
  assert.match(warnings[0].message, /has no source/);
});
