import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import { spawnSync } from 'node:child_process';
import { emit, nextRoutes, scanText } from './emitter';

// Synthetic sources only. A silently wrong emitter produces a green matrix, which is the exact
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

test('a mechanism implementation emits the exact raw record and companion', () => {
  const dir = sourcePackage('@fixture/control', {
    'src/branch.ts':
      `export function selectBranch() { implementsMechanism('alpha', 'branch-selection'); }`,
  });
  const manifest = emit([path.join(dir, 'src')], dir).manifest;
  const implementation = manifest.mechanism_implementations[0];

  assert.deepEqual(implementation, {
    spec: 'alpha',
    mechanism: 'branch-selection',
    site: '@fixture/control::./src/branch::none::selectBranch():void',
    binding: 'typescript-symbol:@fixture/control::./src/branch::none::selectBranch():void',
    file: 'src/branch.ts',
    lang: 'typescript',
    source_fingerprint: implementation.source_fingerprint,
  });
  assert.deepEqual(Object.keys(implementation), [
    'spec',
    'mechanism',
    'site',
    'binding',
    'file',
    'lang',
    'source_fingerprint',
  ]);
  assert.match(implementation.source_fingerprint, /^sha256:[0-9a-f]{64}$/);
  assert.deepEqual(manifest.artifacts, [{
    id: implementation.binding,
    kind: 'typescript-symbol',
    file: implementation.file,
  }]);
});

test('a complete manifest uses the exact fingerprint lexical contract', () => {
  const dir = sourcePackage('@fixture/account', {
    'fixture.ts':
    `declare function realizes(spec: string, scenario: string): void;
     declare function implementsCheck(check: string): void;
     export function behavior() { realizes('alpha', 'behavior'); }
     export function mechanism() { implementsMechanism('alpha', 'guard'); }
     declare function test(name: string, body: () => void): void;
     test('check', () => { implementsCheck('alpha/check'); });`,
  });

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

test('a package, nested receiver and overload set form compiler-semantic identity', () => {
  const dir = sourcePackage('@fixture/routing', {
    'src/selector.ts': `
      namespace Route {
        export class Selector {
          choose(value: string): number;
          choose(value: number): number;
          choose(value: string | number): number {
            implementsMechanism('routing', 'branch-selection');
            return typeof value === 'string' ? value.length : value;
          }
        }
      }`,
  });

  const implementation = emit([path.join(dir, 'src')], dir)
    .manifest.mechanism_implementations[0];
  assert.equal(
    implementation.site,
    '@fixture/routing::./src/selector::instance::' +
      'Route.Selector.choose{(number):number;(string):number}',
  );
  assert.equal(implementation.binding, `typescript-symbol:${implementation.site}`);
});

test('a generic callable identity includes compiler-resolved generic arity and signature', () => {
  const dir = sourcePackage('@fixture/generic', {
    'src/identity.ts': `export function retain<Value>(value: Value): Value {
      implementsMechanism('routing', 'generic-guard');
      return value;
    }`,
  });

  const implementation = emit([path.join(dir, 'src')], dir)
    .manifest.mechanism_implementations[0];
  assert.equal(
    implementation.site,
    '@fixture/generic::./src/identity::none::retain<$0>($0):$0',
  );
});

test('generic renaming preserves an unrelated property key with the old generic spelling', () => {
  const before = sourcePackage('@fixture/generic-property', {
    'src/guard.ts': `export function guard<T>(value: { T: T }): T {
      implementsMechanism('routing', 'generic-property');
      return value.T;
    }`,
  });
  const after = sourcePackage('@fixture/generic-property', {
    'src/guard.ts': `export function guard<U>(value: { T: U }): U {
      implementsMechanism('routing', 'generic-property');
      return value.T;
    }`,
  });
  const beforeSite = emit([path.join(before, 'src')], before)
    .manifest.mechanism_implementations[0].site;
  const afterSite = emit([path.join(after, 'src')], after)
    .manifest.mechanism_implementations[0].site;

  assert.equal(beforeSite, afterSite);
  assert.match(beforeSite, /\{ T: \$0; \}/);
});

test('a property key matching generic spelling remains semantic identity', () => {
  const before = sourcePackage('@fixture/generic-key', {
    'src/guard.ts': `export function guard<T>(value: { T: T }): T {
      implementsMechanism('routing', 'generic-key');
      return value.T;
    }`,
  });
  const after = sourcePackage('@fixture/generic-key', {
    'src/guard.ts': `export function guard<U>(value: { U: U }): U {
      implementsMechanism('routing', 'generic-key');
      return value.U;
    }`,
  });
  const beforeSite = emit([path.join(before, 'src')], before)
    .manifest.mechanism_implementations[0].site;
  const afterSite = emit([path.join(after, 'src')], after)
    .manifest.mechanism_implementations[0].site;

  assert.notEqual(beforeSite, afterSite);
  assert.match(beforeSite, /\{ T: \$0; \}/);
  assert.match(afterSite, /\{ U: \$0; \}/);
});

test('mapped indexed and conditional type binders do not capture callable generics', () => {
  const before = sourcePackage('@fixture/nested-generic', {
    'src/guard.ts': `export function guard<T>(value: {
      [Key in keyof T]: T[Key] extends infer Item ? { T: Item } : never
    }): T {
      implementsMechanism('routing', 'nested-generic');
      return value as T;
    }`,
  });
  const after = sourcePackage('@fixture/nested-generic', {
    'src/guard.ts': `export function guard<U>(value: {
      [Key in keyof U]: U[Key] extends infer Item ? { T: Item } : never
    }): U {
      implementsMechanism('routing', 'nested-generic');
      return value as U;
    }`,
  });
  const beforeSite = emit([path.join(before, 'src')], before)
    .manifest.mechanism_implementations[0].site;
  const afterSite = emit([path.join(after, 'src')], after)
    .manifest.mechanism_implementations[0].site;

  assert.equal(beforeSite, afterSite);
  assert.match(beforeSite, /\[Key in keyof \$0\]/);
  assert.match(beforeSite, /\{ T: Item; \}/);
});

test('an explicit this parameter is canonical signature identity', () => {
  const alpha = sourcePackage('@fixture/explicit-this', {
    'src/guard.ts': `type Receiver = { mode: 'alpha' };
      export function guard(this: Receiver): void {
        implementsMechanism('routing', 'explicit-this');
      }`,
  });
  const beta = sourcePackage('@fixture/explicit-this', {
    'src/guard.ts': `type Receiver = { mode: 'beta' };
      export function guard(this: Receiver): void {
        implementsMechanism('routing', 'explicit-this');
      }`,
  });
  const alphaSite = emit([path.join(alpha, 'src')], alpha)
    .manifest.mechanism_implementations[0].site;
  const betaSite = emit([path.join(beta, 'src')], beta)
    .manifest.mechanism_implementations[0].site;

  assert.match(alphaSite, /::none::guard\(this:\{ mode: "alpha"; \}\):void$/);
  assert.match(betaSite, /::none::guard\(this:\{ mode: "beta"; \}\):void$/);
  assert.notEqual(alphaSite, betaSite);
});

test('a receiver generic outside the callable account fails closed', () => {
  const dir = sourcePackage('@fixture/generic-receiver', {
    'src/guard.ts': `export class Guard<Value extends { id: string }> {
      guard(value: Value): Value {
        implementsMechanism('routing', 'generic-receiver');
        return value;
      }
    }`,
  });

  assert.throws(
    () => emit([path.join(dir, 'src')], dir),
    /compiler type parameter is outside the callable generic account/,
  );
});

test('mechanism identity survives whole-project root relocation', () => {
  const source = `export function stable(value: string): boolean {
    implementsMechanism('routing', 'stable-guard');
    return value.length > 0;
  }`;
  const beforeDir = sourcePackage('@fixture/relocation', { 'src/stable.ts': source });
  const afterDir = sourcePackage('@fixture/relocation', { 'src/stable.ts': source });
  const before = emit([path.join(beforeDir, 'src')], beforeDir)
    .manifest.mechanism_implementations[0];
  const after = emit([path.join(afterDir, 'src')], afterDir)
    .manifest.mechanism_implementations[0];

  assert.equal(before.site, after.site);
  assert.equal(before.binding, after.binding);
  assert.equal(before.source_fingerprint, after.source_fingerprint);
  assert.equal(before.file, after.file);
  assert.notEqual(beforeDir, afterDir);
});

test('JavaScript maps its semantic site to javascript-symbol without a path', () => {
  const dir = sourcePackage('@fixture/javascript', {
    'src/guard.js': `/** @returns {boolean} */
      export function guard() {
        implementsMechanism('routing', 'javascript-guard');
        return true;
      }`,
  });
  const implementation = emit([path.join(dir, 'src')], dir)
    .manifest.mechanism_implementations[0];

  assert.equal(implementation.lang, 'javascript');
  assert.equal(
    implementation.site,
    '@fixture/javascript::./src/guard::none::guard():boolean',
  );
  assert.equal(implementation.binding, `javascript-symbol:${implementation.site}`);
  assert.ok(!implementation.site.includes('src/guard.js'));
  assert.ok(!implementation.binding.includes('src/guard.js'));
  assert.ok(!implementation.site.includes(dir));
  assert.ok(!implementation.binding.includes(dir));
});

test('mechanism extraction fails without semantic package identity', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-no-package-'));
  writeProjectScaffold(dir);
  const source = path.join(dir, 'guard.ts');
  fs.writeFileSync(source, mechanismSource(
    `function guard() { implementsMechanism('routing', 'guard'); }`,
  ));

  assert.throws(
    () => emit([source], dir),
    /needs a package\.json semantic module name/,
  );
});

test('anonymous mechanism sites fail closed', () => {
  const dir = sourcePackage('@fixture/ambiguous', {
    'src/guard.ts': `declare function register(value: () => void): void;
      register(() => { implementsMechanism('routing', 'guard'); });`,
  });

  assert.throws(
    () => emit([path.join(dir, 'src')], dir),
    /not enclosed by a named callable declaration/,
  );
});

test('compiler-ambiguous declarations fail instead of using a file fallback', () => {
  const dir = sourcePackage('@fixture/ambiguous', {
    'src/guard.ts': `function guard() {
      implementsMechanism('routing', 'guard');
    }
    function guard() { return true; }`,
  });

  assert.throws(
    () => emit([path.join(dir, 'src')], dir),
    /compiler diagnostic: Duplicate function implementation/,
  );
});

test('repeated marker calls at one semantic site fail closed', () => {
  const dir = sourcePackage('@fixture/repeated', {
    'src/guard.ts': `function guard() {
      implementsMechanism('routing', 'guard');
      implementsMechanism('routing', 'guard');
    }`,
  });

  assert.throws(
    () => emit([path.join(dir, 'src')], dir),
    /has repeated marker calls/,
  );
});

test('moving a module changes the mechanism identity', () => {
  const dir = sourcePackage('@fixture/federated', {
    'area-a/guard.ts': `export function guard() {
      implementsMechanism('alpha', 'guard');
    }`,
    'area-b/guard.ts': `export function guard() {
      implementsMechanism('beta', 'guard');
    }`,
  });
  const manifest = emit([path.join(dir, 'area-a'), path.join(dir, 'area-b')], dir).manifest;

  assert.equal(manifest.mechanism_implementations.length, 2);
  assert.equal(manifest.artifacts.length, 2);
  assert.notEqual(manifest.artifacts[0].id, manifest.artifacts[1].id);
  assert.match(manifest.mechanism_implementations[0].site, /\.\/area-a\/guard/);
  assert.match(manifest.mechanism_implementations[1].site, /\.\/area-b\/guard/);
});

test('invalid mechanism arguments are fatal rather than warnings', () => {
  for (const call of [
    'implementsMechanism()',
    "implementsMechanism('alpha')",
    "implementsMechanism('alpha', mechanism)",
    "implementsMechanism('alpha', 'guard', 'extra')",
  ]) {
    const dir = sourcePackage('@fixture/invalid-marker', {
      'src/guard.ts': `declare const mechanism: string;
        export function guard() { ${call}; }`,
    });
    assert.throws(
      () => emit([path.join(dir, 'src')], dir),
      /needs exactly two string literal spec and mechanism arguments/,
    );
  }
});

test('import aliases and namespace imports resolve the annotation symbol', () => {
  const dir = sourcePackage('@fixture/aliases', {
    'src/direct.ts': `import { implementsMechanism as mark } from '@azimuth-sh/annotations';
      export function direct() { mark('alpha', 'direct'); }`,
    'src/namespace.ts': `import * as azimuth from '@azimuth-sh/annotations';
      export function namespaced() { azimuth.implementsMechanism('alpha', 'namespace'); }`,
  });
  const implementations = emit([path.join(dir, 'src')], dir)
    .manifest.mechanism_implementations;

  assert.equal(implementations.length, 2);
  assert.deepEqual(
    implementations.map((entry) => entry.mechanism),
    ['direct', 'namespace'],
  );
});

test('an unprovable annotation import alias fails through compiler diagnostics', () => {
  const dir = sourcePackage('@fixture/unresolved-alias', {
    'src/guard.ts': `import { implementsMechanism as mark } from '@azimuth-sh/annotations';
      export function guard() { mark('alpha', 'guard'); }`,
  });
  fs.writeFileSync(
    path.join(dir, 'node_modules', '@azimuth-sh', 'annotations', 'index.d.ts'),
    'export declare function unrelated(): void;\n',
  );
  const output = path.join(dir, 'manifest.json');
  const completed = spawnSync(
    process.execPath,
    [path.join(__dirname, 'cli.js'), '--output', output, '--root', dir, path.join(dir, 'src')],
    { encoding: 'utf8' },
  );

  assert.equal(completed.status, 2);
  assert.match(completed.stderr, /^azimuth-emit: .*compiler diagnostic:/m);
  assert.equal(fs.existsSync(output), false);
});

test('a local homonym is ordinary source', () => {
  const dir = sourcePackage('@fixture/homonym', {
    'src/local.ts': `function implementsMechanism(spec: string, mechanism: string): void {
      void spec; void mechanism;
    }
    export function local() { implementsMechanism('alpha', 'ordinary'); }`,
  });

  assert.deepEqual(
    emit([path.join(dir, 'src')], dir).manifest.mechanism_implementations,
    [],
  );
});

test('an arbitrary object property homonym is ordinary source', () => {
  const dir = sourcePackage('@fixture/property-homonym', {
    'src/local.ts': `import { implementsMechanism as marker }
        from '@azimuth-sh/annotations';
      declare const local: any;
      void marker;
      local.implementsMechanism('alpha', 'ordinary');`,
  });

  assert.deepEqual(
    emit([path.join(dir, 'src')], dir).manifest.mechanism_implementations,
    [],
  );
});

test('overlapping selectors are canonicalized before marker extraction', () => {
  const dir = sourcePackage('@fixture/dedup', {
    'src/guard.ts': `export function guard() {
      implementsMechanism('alpha', 'guard');
    }`,
  });
  const file = path.join(dir, 'src', 'guard.ts');
  const manifest = emit([dir, path.join(dir, 'src'), file, file], dir).manifest;

  assert.equal(manifest.mechanism_implementations.length, 1);
  assert.equal(manifest.artifacts.length, 1);
});

test('all eight configured source extensions are discovered', () => {
  const extensions = ['ts', 'tsx', 'mts', 'cts', 'js', 'jsx', 'mjs', 'cjs'];
  const sources = Object.fromEntries(extensions.map((extension, index) => [
    `src/guard-${index}.${extension}`,
    `export function guard${index}() {
      implementsMechanism('alpha', 'guard-${index}');
    }`,
  ]));
  const dir = sourcePackage('@fixture/extensions', sources);
  const implementations = emit([path.join(dir, 'src')], dir)
    .manifest.mechanism_implementations;

  assert.equal(implementations.length, extensions.length);
  assert.deepEqual(
    new Set(implementations.map((entry) => entry.lang)),
    new Set(['javascript', 'typescript']),
  );
});

test('a whole configured Program fails on an unselected source diagnostic', () => {
  const dir = sourcePackage('@fixture/diagnostics', {
    'src/guard.ts': `export function guard() {
      implementsMechanism('alpha', 'guard');
    }`,
    'src/unselected.ts': 'const invalid: string = 42;',
  });

  assert.throws(
    () => emit([path.join(dir, 'src', 'guard.ts')], dir),
    /compiler diagnostic: Type 'number' is not assignable to type 'string'/,
  );
});

test('nearest config and owning package define the project identity', () => {
  const dir = sourcePackage('@fixture/outer', {});
  const nested = path.join(dir, 'packages', 'inner');
  fs.mkdirSync(nested, { recursive: true });
  fs.writeFileSync(path.join(nested, 'package.json'), JSON.stringify({ name: '@fixture/inner' }));
  writeProjectScaffold(nested);
  const source = path.join(nested, 'src', 'guard.ts');
  fs.mkdirSync(path.dirname(source), { recursive: true });
  fs.writeFileSync(source, mechanismSource(`export function guard() {
    implementsMechanism('alpha', 'guard');
  }`));

  const implementation = emit([source], dir).manifest.mechanism_implementations[0];
  assert.match(implementation.site, /^@fixture\/inner::\.\/src\/guard::/);
});

test('a package-relative module specifier must resolve to the selected source', () => {
  const dir = sourcePackage('@fixture/module-resolution', {
    'src/guard.native.ts': 'export const unrelated = true;',
    'src/guard.ts': `export function guard() {
      implementsMechanism('alpha', 'guard');
    }`,
  });
  const configPath = path.join(dir, 'tsconfig.json');
  const config = JSON.parse(fs.readFileSync(configPath, 'utf8')) as {
    compilerOptions: { moduleSuffixes?: string[] };
  };
  config.compilerOptions.moduleSuffixes = ['.native', ''];
  fs.writeFileSync(configPath, JSON.stringify(config));

  assert.throws(
    () => emit([path.join(dir, 'src', 'guard.ts')], dir),
    /module specifier .* does not uniquely resolve to the selected source/,
  );
});

test('ambiguous nearest configs and inputs spanning projects fail closed', () => {
  const ambiguous = sourcePackage('@fixture/ambiguous-config', {
    'src/guard.ts': `export function guard() {
      implementsMechanism('alpha', 'guard');
    }`,
  });
  fs.writeFileSync(path.join(ambiguous, 'jsconfig.json'), '{}');
  assert.throws(
    () => emit([path.join(ambiguous, 'src')], ambiguous),
    /both tsconfig\.json and jsconfig\.json define the nearest project/,
  );

  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-project-span-'));
  const first = path.join(root, 'first');
  const second = path.join(root, 'second');
  for (const [dir, name] of [[first, '@fixture/first'], [second, '@fixture/second']]) {
    fs.mkdirSync(dir, { recursive: true });
    fs.writeFileSync(path.join(dir, 'package.json'), JSON.stringify({ name }));
    writeProjectScaffold(dir);
    fs.mkdirSync(path.join(dir, 'src'), { recursive: true });
    fs.writeFileSync(
      path.join(dir, 'src', 'guard.ts'),
      mechanismSource("export function guard() { implementsMechanism('alpha', 'guard'); }"),
    );
  }
  assert.throws(
    () => emit([path.join(first, 'src'), path.join(second, 'src')], root),
    /span more than one configured TypeScript\/JavaScript project/,
  );
});

test('canonical type identity expands aliases and retains generic constraints', () => {
  const alias = sourcePackage('@fixture/types', {
    'src/guard.ts': `type Input = { id: string };
      export function guard<Value extends Input>(value: Value): Input {
        implementsMechanism('alpha', 'guard');
        return value;
      }`,
  });
  const structural = sourcePackage('@fixture/types', {
    'src/guard.ts': `export function guard<Value extends { id: string }>(value: Value): {
        id: string
      } {
        implementsMechanism('alpha', 'guard');
        return value;
      }`,
  });
  const aliasSite = emit([path.join(alias, 'src')], alias)
    .manifest.mechanism_implementations[0].site;
  const structuralSite = emit([path.join(structural, 'src')], structural)
    .manifest.mechanism_implementations[0].site;

  assert.equal(aliasSite, structuralSite);
  assert.match(aliasSite, /<\$0 extends \{ id: string; \}>/);
});

test('a named type without a path-free compiler qualification fails closed', () => {
  const dir = sourcePackage('@fixture/named-type', {
    'src/guard.ts': `interface Input { id: string }
      export function guard(value: Input): boolean {
        implementsMechanism('alpha', 'guard');
        return value.id.length > 0;
      }`,
  });

  assert.throws(
    () => emit([path.join(dir, 'src')], dir),
    /cannot qualify named type Input without a source locator/,
  );
});

test('static and instance receiver kinds are explicit', () => {
  const dir = sourcePackage('@fixture/receivers', {
    'src/guard.ts': `export class StaticGuard {
      static guard() { implementsMechanism('alpha', 'static'); }
    }
    export class InstanceGuard {
      guard() { implementsMechanism('alpha', 'instance'); }
    }`,
  });
  const sites = emit([path.join(dir, 'src')], dir)
    .manifest.mechanism_implementations.map((entry) => entry.site);

  assert.ok(sites.some((site) => site.includes('::static::StaticGuard.guard')));
  assert.ok(sites.some((site) => site.includes('::instance::InstanceGuard.guard')));
  assert.ok(sites.every((site) => !site.includes('(this:')));
});

test('the CLI reports invalid marker input without publishing output', () => {
  const dir = sourcePackage('@fixture/cli-failure', {
    'src/guard.ts': `export function guard() {
      implementsMechanism('alpha', 42);
    }`,
  });
  const output = path.join(dir, 'manifest.json');
  const completed = spawnSync(
    process.execPath,
    [path.join(__dirname, 'cli.js'), '--output', output, '--root', dir, path.join(dir, 'src')],
    { encoding: 'utf8' },
  );

  assert.equal(completed.status, 2);
  assert.match(completed.stderr, /^azimuth-emit: .*needs exactly two string literal/m);
  assert.equal(fs.existsSync(output), false);
});

test('the CLI publishes the strict mechanism pair after complete validation', () => {
  const dir = sourcePackage('@fixture/cli-success', {
    'src/guard.ts': `export function guard() {
      implementsMechanism('alpha', 'guard');
    }`,
  });
  const output = path.join(dir, 'manifest.json');
  const completed = spawnSync(
    process.execPath,
    [path.join(__dirname, 'cli.js'), '--output', output, '--root', dir, path.join(dir, 'src')],
    { encoding: 'utf8' },
  );

  assert.equal(completed.status, 0, completed.stderr);
  const manifest = JSON.parse(fs.readFileSync(output, 'utf8')) as {
    mechanism_implementations: Array<{ binding: string }>;
    artifacts: Array<Record<string, unknown>>;
  };
  assert.deepEqual(Object.keys(manifest.mechanism_implementations[0]), [
    'spec',
    'mechanism',
    'site',
    'binding',
    'file',
    'lang',
    'source_fingerprint',
  ]);
  assert.deepEqual(manifest.artifacts, [{
    id: manifest.mechanism_implementations[0].binding,
    kind: 'typescript-symbol',
    file: 'src/guard.ts',
  }]);
});

test('the CLI reports publication failures without an unhandled stack', () => {
  const dir = sourcePackage('@fixture/cli-publication', {
    'src/guard.ts': `export function guard() {
      implementsMechanism('alpha', 'guard');
    }`,
  });
  const output = path.join(dir, 'occupied');
  fs.mkdirSync(output);
  const completed = spawnSync(
    process.execPath,
    [path.join(__dirname, 'cli.js'), '--output', output, '--root', dir, path.join(dir, 'src')],
    { encoding: 'utf8' },
  );

  assert.equal(completed.status, 2);
  assert.match(completed.stderr, /^azimuth-emit: .*EISDIR/m);
  assert.doesNotMatch(completed.stderr, /\n\s+at /);
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

function sourcePackage(name: string, sources: Record<string, string>): string {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-source-package-'));
  fs.writeFileSync(path.join(dir, 'package.json'), JSON.stringify({ name }));
  writeProjectScaffold(dir);
  for (const [relative, text] of Object.entries(sources)) {
    const file = path.join(dir, relative);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    const hasAnnotationImport = text.includes("from '@azimuth-sh/annotations'");
    const declaresHomonym = /function\s+implementsMechanism\b/.test(text);
    const emitted = text.includes('implementsMechanism') && !hasAnnotationImport && !declaresHomonym
      ? mechanismSource(text)
      : text;
    fs.writeFileSync(file, emitted);
  }
  return dir;
}

function writeProjectScaffold(dir: string): void {
  fs.writeFileSync(
    path.join(dir, 'tsconfig.json'),
    JSON.stringify({
      compilerOptions: {
        allowJs: true,
        checkJs: true,
        module: 'commonjs',
        moduleResolution: 'node',
        noEmit: true,
        jsx: 'preserve',
        skipLibCheck: true,
        target: 'ES2022',
      },
      include: ['**/*'],
    }),
  );
  const annotations = path.join(dir, 'node_modules', '@azimuth-sh', 'annotations');
  fs.mkdirSync(annotations, { recursive: true });
  fs.writeFileSync(
    path.join(annotations, 'package.json'),
    JSON.stringify({ name: '@azimuth-sh/annotations', types: 'index.d.ts' }),
  );
  fs.writeFileSync(
    path.join(annotations, 'index.d.ts'),
    'export declare function implementsMechanism(spec: string, mechanism: string): void;\n',
  );
}

function mechanismSource(source: string): string {
  return "import { implementsMechanism } from '@azimuth-sh/annotations';\n" + source;
}

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
