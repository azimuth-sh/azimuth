import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import { emit, nextRoutes, scanText } from './emitter';

// Synthetic sources only (D2). A silently wrong emitter produces a green matrix, which is the exact
// failure the framework exists to prevent — so these assert on the shape of what is emitted, not
// merely that something was.

test('resolves a site to its enclosing function', () => {
  const result = scanText(
    `import { realizes } from '@azimuth-sh/annotations';
     export function handler() { realizes('alpha', 'route-thing'); }`,
    'a.ts',
  );
  assert.equal(result.realizes.length, 1);
  assert.deepEqual(
    {
      spec: result.realizes[0].spec,
      scenario: result.realizes[0].scenario,
      site: result.realizes[0].site,
    },
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

test('a Check implementation carries only its enclosing source facts', () => {
  const result = scanText(
    `test('the route answers', () => { implementsCheck('alpha/route-answer'); });`,
    'a.test.ts',
  );
  assert.deepEqual(result.checkImplementations[0], {
    check: 'alpha/route-answer',
    site: 'the route answers',
    file: 'a.test.ts',
    lang: 'typescript',
    source_fingerprint: result.checkImplementations[0].source_fingerprint,
  });
  assert.match(result.checkImplementations[0].source_fingerprint, /^sha256:[0-9a-f]{64}$/);
});

test('a site fingerprint changes only when that site changes', () => {
  const before = scanText(
    `test('first', () => { implementsCheck('alpha/shared'); assert(1); });
     test('second', () => { implementsCheck('alpha/shared'); assert(2); });`,
    'a.test.ts',
  );
  const after = scanText(
    `test('first', () => { implementsCheck('alpha/shared'); assert(1); });
     test('second', () => { implementsCheck('alpha/shared'); assert(3); });`,
    'a.test.ts',
  );

  assert.equal(
    before.checkImplementations[0].source_fingerprint,
    after.checkImplementations[0].source_fingerprint,
  );
  assert.notEqual(
    before.checkImplementations[1].source_fingerprint,
    after.checkImplementations[1].source_fingerprint,
  );
});

test('several source sites may implement one Check', () => {
  const result = scanText(
    `function first() { implementsCheck('alpha/shared'); }
     function second() { implementsCheck('alpha/shared'); }`,
    'a.test.ts',
  );
  assert.deepEqual(
    result.checkImplementations.map((entry) => [entry.check, entry.site]),
    [['alpha/shared', 'first'], ['alpha/shared', 'second']],
  );
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
  assert.match(
    result.mechanismImplementations[0].source_fingerprint,
    /^sha256:[0-9a-f]{64}$/,
  );
});

test('a complete manifest uses the exact fingerprint lexical contract', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-emitter-'));
  fs.writeFileSync(
    path.join(dir, 'fixture.ts'),
    `export function behavior() { realizes('alpha', 'behavior'); }
     export function mechanism() { implementsMechanism('alpha', 'guard'); }
     test('check', () => { implementsCheck('alpha/check'); });`,
  );

  const parsed = JSON.parse(JSON.stringify(emit([dir], dir).manifest)) as Record<string, unknown>;
  for (const key of ['realizes', 'check_implementations', 'mechanism_implementations']) {
    const entries = parsed[key] as Array<{ source_fingerprint: string }>;
    assert.ok(entries.length > 0, `${key} is empty`);
    assert.ok(
      entries.every((entry) => /^sha256:[0-9a-f]{64}$/.test(entry.source_fingerprint)),
      `${key} contains a non-canonical source fingerprint`,
    );
  }
  assert.equal('covers' in parsed, false);
  assert.equal('mechanism_covers' in parsed, false);
  assert.equal('observations' in parsed, false);
});

test('retired evidence markers fail instead of disappearing', () => {
  for (const marker of ['covers', 'coversMechanism']) {
    assert.throws(
      () => scanText(`${marker}('alpha', 'branch-selection');`, 'src/branch.test.ts'),
      new RegExp('retired alpha 1 marker `' + marker + '` is not supported'),
    );
  }

  assert.throws(
    () => scanText(
      `import { covers as oldCover } from '@azimuth-sh/annotations';
       oldCover('alpha', 'branch-selection');`,
      'src/branch.test.ts',
    ),
    /retired alpha 1 marker `covers` is not supported/,
  );
});

test('unrelated object methods named covers remain ordinary source', () => {
  const result = scanText(
    `const assertion = { covers() { return true; } };
     assertion.covers();
     function covers(value: string) { return value; }
     covers('ordinary');`,
    'src/assertion.ts',
  );
  assert.deepEqual(result.checkImplementations, []);
  assert.deepEqual(result.warnings, []);
});

// Form is how a test checks, not a property of code — so realizes never carries one, and the
// emitter has no way to attach one.
test('realizes carries no form', () => {
  const result = scanText(`function f() { realizes('a', 's'); }`, 'a.ts');
  assert.equal('scope' in result.realizes[0], false);
  assert.equal('quantification' in result.realizes[0], false);
});

test('a Check implementation needs exactly one literal id', () => {
  const result = scanText(
    `test('missing', () => { implementsCheck(); });
     test('dynamic', () => { implementsCheck(checkId); });
     test('extra', () => { implementsCheck('alpha/check', 'extra'); });`,
    'a.test.ts',
  );
  assert.equal(result.checkImplementations.length, 0);
  assert.equal(result.warnings.length, 3);
  assert.ok(
    result.warnings.every((warning) => /exactly one string Check id/.test(warning.message)),
  );
});

test('warnings carry a line number', () => {
  const result = scanText(
    `\n\ntest('t', () => { implementsCheck(); });`,
    'a.test.ts',
  );
  assert.equal(result.warnings[0].line, 3);
  assert.equal(result.warnings[0].file, 'a.test.ts');
});

test('an unmarked test is outside the Check model', () => {
  const result = scanText(
    `test('enrolled', () => { implementsCheck('alpha/enrolled'); });
     test('bare', () => { const x = 1; });`,
    'a.test.ts',
  );
  assert.deepEqual(Object.keys(result).sort(), [
    'checkImplementations',
    'mechanismImplementations',
    'realizes',
    'warnings',
  ]);
  assert.equal(result.checkImplementations.length, 1);
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
