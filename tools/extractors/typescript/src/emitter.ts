/**
 * Static-scan emitter for TypeScript.
 *
 * Reads sources, finds linkage marker calls, resolves each
 * call's enclosing named symbol as the site, and writes the language-neutral manifest the core
 * reads. Each ecosystem emits the manifest natively; the core only ever reads manifests, which is
 * why adding a language is a day's work rather than a fork of the core.
 *
 * D17 constrains the core, not the extractors: AST work belongs here, where the compiler API is
 * already present and idiomatic.
 */

import * as fs from 'node:fs';
import * as path from 'node:path';
import { createHash } from 'node:crypto';
import * as ts from 'typescript';

const SCOPES = ['unit', 'component', 'e2e'] as const;
const QUANTIFICATIONS = ['example', 'universal'] as const;
const ORACLES = [
  'direct',
  'golden',
  'relational',
  'metamorphic',
  'model-based',
  'contract',
] as const;

const TEST_CALLS = new Set(['test', 'it']);

export interface Entry {
  spec: string;
  scenario: string;
  site: string;
  file: string;
  lang: string;
  source_fingerprint: string;
  area?: string;
  address_kind?: string;
  address?: string;
  mount?: string;
  evidence_kind?: string;
  evidence_outcome?: string;
  observed_at?: string;
  expires_at?: number;
  scope?: string;
  quantification?: string;
  oracle?: string;
}

export interface ClassMember {
  class: string;
  site: string;
  file: string;
  lang: string;
  area?: string;
  address_kind?: string;
  address?: string;
  mount?: string;
}

export interface Enumeration {
  class: string;
  kind: string;
  source: string;
  source_fingerprint: string;
  area?: string;
  address_kind?: string;
  address?: string;
  mount?: string;
}

export interface Artifact {
  id: string;
  kind: string;
  file: string;
}

export interface MechanismImplementation {
  spec: string;
  mechanism: string;
  binding: string;
  file: string;
  lang: string;
  source_fingerprint: string;
}

export interface MechanismCover {
  spec: string;
  mechanism: string;
  site: string;
  file: string;
  lang: string;
  source_fingerprint: string;
  scope: string;
  quantification: string;
  oracle?: string;
}

export interface ObservationSubject {
  relation: 'realization' | 'evidence' | 'mechanism';
  identity: string;
}

export interface ObservationBinding {
  role: 'evidence' | 'challenge';
  spec: string;
  scenario: string;
  assertion: string;
  outcome: 'satisfied' | 'violated' | 'clean' | 'findings' | 'inconclusive';
  subjects: ObservationSubject[];
  scope?: string;
  quantification?: string;
  oracle?: string;
}

export interface Observation {
  id: string;
  kind: string;
  tool: string;
  tool_version: string;
  report: string;
  inputs: string[];
  observed_at?: string;
  expires_at?: number;
  source_fingerprint: string;
  bindings: ObservationBinding[];
  payload: unknown;
}

export interface Manifest {
  realizes: Entry[];
  covers: Entry[];
  mechanism_implementations: MechanismImplementation[];
  mechanism_covers: MechanismCover[];
  class_members: ClassMember[];
  enumerations: Enumeration[];
  artifacts: Artifact[];
  observations?: Observation[];
}

export interface Warning {
  file: string;
  line: number;
  message: string;
}

export interface ScanResult {
  realizes: Entry[];
  covers: Entry[];
  mechanismImplementations: MechanismImplementation[];
  mechanismCovers: MechanismCover[];
  warnings: Warning[];
}

export function scanText(text: string, file: string): ScanResult {
  const lang = languageOf(file);
  const result: ScanResult = {
    realizes: [],
    covers: [],
    mechanismImplementations: [],
    mechanismCovers: [],
    warnings: [],
  };
  const source = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true, scriptKind(file));

  visit(source, (node) => {
    if (isMarkerCall(node, 'realizes')) {
      const args = stringArgs(node);
      if (args.length < 2) {
        result.warnings.push(warn(node, source, file, 'realizes needs a spec and a scenario id'));
        return;
      }
      const site = resolveSite(node, source);
      result.realizes.push({
        spec: args[0],
        scenario: args[1],
        site: site.name,
        file,
        lang,
        source_fingerprint: site.fingerprint,
      });
      return;
    }

    if (isMarkerCall(node, 'covers')) {
      const args = stringArgs(node);
      if (args.length < 4) {
        result.warnings.push(
          warn(node, source, file, 'covers needs a spec, a scenario id, a scope and a quantification'),
        );
        return;
      }
      const [spec, scenario, scope, quantification, oracle] = args;
      if (!member(scope, SCOPES)) {
        result.warnings.push(warn(node, source, file, `unknown scope \`${scope}\``));
        return;
      }
      if (!member(quantification, QUANTIFICATIONS)) {
        result.warnings.push(
          warn(node, source, file, `unknown quantification \`${quantification}\``),
        );
        return;
      }
      if (oracle !== undefined && !member(oracle, ORACLES)) {
        result.warnings.push(warn(node, source, file, `unknown oracle \`${oracle}\``));
        return;
      }
      const site = resolveSite(node, source);
      result.covers.push({
        spec,
        scenario,
        site: site.name,
        file,
        lang,
        source_fingerprint: site.fingerprint,
        scope,
        quantification,
        ...(oracle === undefined ? {} : { oracle }),
      });
      return;
    }

    if (isMarkerCall(node, 'implementsMechanism')) {
      const args = stringArgs(node);
      if (args.length < 2) {
        result.warnings.push(
          warn(node, source, file, 'implementsMechanism needs a spec and a mechanism id'),
        );
        return;
      }
      const site = resolveSite(node, source);
      result.mechanismImplementations.push({
        spec: args[0],
        mechanism: args[1],
        binding: symbolBinding(lang, file, site.name),
        file,
        lang,
        source_fingerprint: site.fingerprint,
      });
      return;
    }

    if (isMarkerCall(node, 'coversMechanism')) {
      const args = stringArgs(node);
      if (args.length < 4) {
        result.warnings.push(
          warn(
            node,
            source,
            file,
            'coversMechanism needs a spec, a mechanism id, a scope and a quantification',
          ),
        );
        return;
      }
      const [spec, mechanism, scope, quantification, oracle] = args;
      if (!member(scope, SCOPES)) {
        result.warnings.push(warn(node, source, file, `unknown scope \`${scope}\``));
        return;
      }
      if (!member(quantification, QUANTIFICATIONS)) {
        result.warnings.push(
          warn(node, source, file, `unknown quantification \`${quantification}\``),
        );
        return;
      }
      if (oracle !== undefined && !member(oracle, ORACLES)) {
        result.warnings.push(warn(node, source, file, `unknown oracle \`${oracle}\``));
        return;
      }
      const site = resolveSite(node, source);
      result.mechanismCovers.push({
        spec,
        mechanism,
        site: site.name,
        file,
        lang,
        source_fingerprint: site.fingerprint,
        scope,
        quantification,
        ...(oracle === undefined ? {} : { oracle }),
      });
    }
  });

  return result;
}

function symbolBinding(lang: string, file: string, site: string): string {
  return `${lang}-symbol:${file}#${site}`;
}

function languageOf(file: string): 'javascript' | 'typescript' {
  return /\.(jsx?|mjs|cjs)$/.test(file) ? 'javascript' : 'typescript';
}

function visit(node: ts.Node, fn: (node: ts.Node) => void): void {
  fn(node);
  ts.forEachChild(node, (child) => visit(child, fn));
}

function isMarkerCall(node: ts.Node, name: string): node is ts.CallExpression {
  return (
    ts.isCallExpression(node) &&
    ts.isIdentifier(node.expression) &&
    node.expression.text === name
  );
}

function isTestCall(node: ts.Node): node is ts.CallExpression {
  return (
    ts.isCallExpression(node) &&
    ts.isIdentifier(node.expression) &&
    TEST_CALLS.has(node.expression.text) &&
    node.arguments.length > 0 &&
    ts.isStringLiteralLike(node.arguments[0])
  );
}

function stringArgs(call: ts.CallExpression): string[] {
  const out: string[] = [];
  for (const argument of call.arguments) {
    if (!ts.isStringLiteralLike(argument)) break;
    out.push(argument.text);
  }
  return out;
}

/**
 * Walks outward to the nearest thing a human would name: a test's own title, then a named
 * function or method, then a named binding an arrow was assigned to. A `covers` inside
 * `test('…', () => …)` therefore names the test, while a `realizes` in `export function GET` names
 * the handler.
 */
function resolveSite(call: ts.CallExpression, source: ts.SourceFile): {
  name: string;
  fingerprint: string;
} {
  let node: ts.Node | undefined = call.parent;
  while (node) {
    if (isTestCall(node)) {
      return namedSite(testName(node), node, source);
    }
    if ((ts.isFunctionDeclaration(node) || ts.isMethodDeclaration(node)) && node.name) {
      return namedSite(node.name.getText(), node, source);
    }
    if (ts.isVariableDeclaration(node) && ts.isIdentifier(node.name)) {
      return namedSite(node.name.text, node, source);
    }
    if (ts.isClassDeclaration(node) && node.name) {
      return namedSite(node.name.text, node, source);
    }
    node = node.parent;
  }
  return namedSite('<module>', source, source);
}

function namedSite(name: string, node: ts.Node, source: ts.SourceFile): {
  name: string;
  fingerprint: string;
} {
  return {
    name,
    fingerprint: createHash('sha256').update(node.getText(source)).digest('hex'),
  };
}

function testName(call: ts.CallExpression): string {
  const first = call.arguments[0];
  return ts.isStringLiteralLike(first) ? first.text : '<test>';
}

function warn(node: ts.Node, source: ts.SourceFile, file: string, message: string): Warning {
  const { line } = source.getLineAndCharacterOfPosition(node.getStart(source));
  return { file, line: line + 1, message };
}

function member<T extends string>(value: string | undefined, values: readonly T[]): value is T {
  return value !== undefined && (values as readonly string[]).includes(value);
}

function scriptKind(file: string): ts.ScriptKind {
  if (file.endsWith('.tsx')) return ts.ScriptKind.TSX;
  if (file.endsWith('.jsx')) return ts.ScriptKind.JSX;
  if (file.endsWith('.js') || file.endsWith('.mjs') || file.endsWith('.cjs')) return ts.ScriptKind.JS;
  return ts.ScriptKind.TS;
}

const SOURCE = /\.(ts|tsx|js|jsx|mjs|cjs)$/;
const SKIP = new Set(['node_modules', 'dist', 'build', '.git', 'target']);

export function walk(dir: string, out: string[] = []): string[] {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (SKIP.has(entry.name)) continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, out);
    else if (SOURCE.test(entry.name) && !entry.name.endsWith('.d.ts')) out.push(full);
  }
  return out;
}

export function emit(roots: string[], repoRoot: string): { manifest: Manifest; warnings: Warning[] } {
  const manifest: Manifest = {
    realizes: [],
    covers: [],
    mechanism_implementations: [],
    mechanism_covers: [],
    class_members: [],
    enumerations: [],
    artifacts: [],
  };
  const warnings: Warning[] = [];

  const files: string[] = [];
  for (const root of roots) {
    const stat = fs.statSync(root);
    if (stat.isDirectory()) walk(root, files);
    else files.push(root);
  }
  files.sort();

  for (const file of files) {
    const relative = path.relative(repoRoot, file).split(path.sep).join('/');
    const result = scanText(fs.readFileSync(file, 'utf8'), relative);
    manifest.realizes.push(...result.realizes);
    manifest.covers.push(...result.covers);
    manifest.mechanism_implementations.push(...result.mechanismImplementations);
    manifest.mechanism_covers.push(...result.mechanismCovers);
    manifest.artifacts.push(
      ...result.mechanismImplementations.map((entry) => ({
        id: entry.binding,
        kind: `${entry.lang}-symbol`,
        file: entry.file,
      })),
    );
    warnings.push(...result.warnings);
  }

  manifest.realizes.sort(compare);
  manifest.covers.sort(compare);
  manifest.mechanism_implementations.sort(compareMechanism);
  manifest.mechanism_covers.sort(compareMechanism);
  manifest.artifacts.sort((a, b) => a.id.localeCompare(b.id));
  manifest.artifacts = manifest.artifacts.filter(
    (artifact, index, all) => index === 0 || artifact.id !== all[index - 1].id,
  );
  return { manifest, warnings };
}

function compare(a: Entry, b: Entry): number {
  return (
    a.spec.localeCompare(b.spec) ||
    a.scenario.localeCompare(b.scenario) ||
    a.site.localeCompare(b.site)
  );
}

function compareMechanism(
  a: MechanismImplementation | MechanismCover,
  b: MechanismImplementation | MechanismCover,
): number {
  const aSite = 'binding' in a ? a.binding : a.site;
  const bSite = 'binding' in b ? b.binding : b.site;
  return (
    a.spec.localeCompare(b.spec) ||
    a.mechanism.localeCompare(b.mechanism) ||
    aSite.localeCompare(bSite)
  );
}

/**
 * Enumerates a Next.js app's routes as members of a class, from the build output rather than from
 * annotations.
 *
 * The point is the source: `app-path-routes-manifest.json` is written by the build, so a route that
 * exists is a member whether or not anyone remembered to tag it. Membership derived from tags can
 * only ever reach files somebody already annotated, which is the enumerator failure D13.1 names.
 *
 * A member is identified by its file, because that is the unit the router names. Framework-generated
 * pages (`/_not-found`, `/_global-error`) are excluded: they are not sites the project wrote.
 */
export function nextRoutes(
  classId: string,
  appDir: string,
  repoRoot: string,
  origin?: { area: string; mount: string },
): { members: ClassMember[]; enumeration?: Enumeration; warnings: Warning[] } {
  const manifestPath = path.join(appDir, '.next', 'app-path-routes-manifest.json');
  const warnings: Warning[] = [];

  if (!fs.existsSync(manifestPath)) {
    warnings.push({
      file: manifestPath,
      line: 0,
      message: 'route manifest not found — build the app before emitting, or the class will be ' +
        'narrower than the app and report green over the difference',
    });
    return { members: [], warnings };
  }

  const routes = JSON.parse(fs.readFileSync(manifestPath, 'utf8')) as Record<string, string>;
  const members: ClassMember[] = [];

  for (const [key, route] of Object.entries(routes)) {
    if (key.startsWith('/_')) continue;

    const base = path.join(appDir, 'src', 'app', key);
    const source = ['.ts', '.tsx'].map((ext) => base + ext).find((f) => fs.existsSync(f));
    if (!source) {
      warnings.push({
        file: manifestPath,
        line: 0,
        message: `route \`${route}\` has no source at ${base}.ts(x); it is left out of the class`,
      });
      continue;
    }

    members.push({
      class: classId,
      site: route,
      file: path.relative(repoRoot, source).split(path.sep).join('/'),
      lang: 'typescript',
      ...(origin ? {
        area: origin.area,
        address_kind: 'next-route',
        address: route,
        mount: origin.mount,
      } : {}),
    });
  }

  members.sort((a, b) => a.class.localeCompare(b.class) || a.file.localeCompare(b.file));
  if (warnings.length > 0) {
    return { members, warnings };
  }

  return {
    members,
    enumeration: {
      class: classId,
      kind: 'next-routes',
      source: path.relative(repoRoot, manifestPath).split(path.sep).join('/'),
      source_fingerprint: createHash('sha256').update(fs.readFileSync(manifestPath)).digest('hex'),
      ...(origin ? {
        area: origin.area,
        address_kind: 'next-route-manifest',
        address: classId,
        mount: origin.mount,
      } : {}),
    },
    warnings,
  };
}
