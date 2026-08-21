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

const TEST_CALLS = new Set(['test', 'it']);
const RETIRED_MARKERS = new Set(['covers', 'coversMechanism']);

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
}

export interface CheckImplementation {
  check: string;
  site: string;
  file: string;
  lang: string;
  source_fingerprint: string;
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

export interface Manifest {
  realizes: Entry[];
  check_implementations: CheckImplementation[];
  mechanism_implementations: MechanismImplementation[];
  class_members: ClassMember[];
  enumerations: Enumeration[];
  artifacts: Artifact[];
}

export interface Warning {
  file: string;
  line: number;
  message: string;
}

export interface ScanResult {
  realizes: Entry[];
  checkImplementations: CheckImplementation[];
  mechanismImplementations: MechanismImplementation[];
  warnings: Warning[];
}

export function scanText(text: string, file: string): ScanResult {
  const lang = languageOf(file);
  const result: ScanResult = {
    realizes: [],
    checkImplementations: [],
    mechanismImplementations: [],
    warnings: [],
  };
  const source = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true, scriptKind(file));
  const retiredImports = retiredMarkerImports(source);

  visit(source, (node) => {
    const retired = retiredMarkerCall(node, retiredImports);
    if (retired) {
      const location = warn(node, source, file, '');
      throw new Error(
        `${file}:${location.line}: retired alpha 1 marker \`${retired}\` is not supported`,
      );
    }

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

    if (isMarkerCall(node, 'implementsCheck')) {
      const args = stringArgs(node);
      if (args.length !== 1 || node.arguments.length !== 1) {
        result.warnings.push(
          warn(node, source, file, 'implementsCheck needs exactly one string Check id'),
        );
        return;
      }
      const site = resolveSite(node, source);
      result.checkImplementations.push({
        check: args[0],
        site: site.name,
        file,
        lang,
        source_fingerprint: site.fingerprint,
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

  });

  return result;
}

interface RetiredMarkerImports {
  calls: Map<string, string>;
  namespaces: Set<string>;
}

function retiredMarkerImports(source: ts.SourceFile): RetiredMarkerImports {
  // Bare calls are recognizable because the alpha 1 extractor treated these exact names as its
  // public marker surface. Imports add aliases and namespace calls without claiming unrelated
  // object methods such as `assertion.covers()`.
  const calls = new Map<string, string>([
    ['covers', 'covers'],
    ['coversMechanism', 'coversMechanism'],
  ]);
  const namespaces = new Set<string>();
  visit(source, (node) => {
    const name = declaredIdentifier(node);
    if (name && RETIRED_MARKERS.has(name)) calls.delete(name);
  });
  for (const statement of source.statements) {
    if (!ts.isImportDeclaration(statement)
      || !ts.isStringLiteral(statement.moduleSpecifier)
      || statement.moduleSpecifier.text !== '@azimuth-sh/annotations') {
      continue;
    }
    const bindings = statement.importClause?.namedBindings;
    if (bindings && ts.isNamespaceImport(bindings)) {
      namespaces.add(bindings.name.text);
    } else if (bindings && ts.isNamedImports(bindings)) {
      for (const element of bindings.elements) {
        const imported = element.propertyName?.text ?? element.name.text;
        if (RETIRED_MARKERS.has(imported)) calls.set(element.name.text, imported);
      }
    }
  }
  return { calls, namespaces };
}

function declaredIdentifier(node: ts.Node): string | undefined {
  if ((ts.isFunctionDeclaration(node)
      || ts.isClassDeclaration(node)
      || ts.isVariableDeclaration(node)
      || ts.isParameter(node))
    && node.name
    && ts.isIdentifier(node.name)) {
    return node.name.text;
  }
  if (ts.isImportSpecifier(node)) return node.name.text;
  return undefined;
}

function retiredMarkerCall(
  node: ts.Node,
  imports: RetiredMarkerImports,
): string | undefined {
  if (!ts.isCallExpression(node)) return undefined;
  if (ts.isIdentifier(node.expression)) return imports.calls.get(node.expression.text);
  if (!ts.isPropertyAccessExpression(node.expression)
    || !ts.isIdentifier(node.expression.expression)
    || !imports.namespaces.has(node.expression.expression.text)
    || !RETIRED_MARKERS.has(node.expression.name.text)) {
    return undefined;
  }
  return node.expression.name.text;
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
 * function or method, then a named binding an arrow was assigned to. An `implementsCheck` inside
 * `test('…', () => …)` therefore names the test, while a `realizes` in `export function GET`
 * names the handler.
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
    fingerprint: sha256Fingerprint(node.getText(source)),
  };
}

function sha256Fingerprint(input: string | Buffer): string {
  return `sha256:${createHash('sha256').update(input).digest('hex')}`;
}

function testName(call: ts.CallExpression): string {
  const first = call.arguments[0];
  return ts.isStringLiteralLike(first) ? first.text : '<test>';
}

function warn(node: ts.Node, source: ts.SourceFile, file: string, message: string): Warning {
  const { line } = source.getLineAndCharacterOfPosition(node.getStart(source));
  return { file, line: line + 1, message };
}

function scriptKind(file: string): ts.ScriptKind {
  if (file.endsWith('.tsx')) return ts.ScriptKind.TSX;
  if (file.endsWith('.jsx')) return ts.ScriptKind.JSX;
  if (file.endsWith('.js') || file.endsWith('.mjs') || file.endsWith('.cjs')) {
    return ts.ScriptKind.JS;
  }
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

export function emit(
  roots: string[],
  repoRoot: string,
): { manifest: Manifest; warnings: Warning[] } {
  const manifest: Manifest = {
    realizes: [],
    check_implementations: [],
    mechanism_implementations: [],
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
    manifest.check_implementations.push(...result.checkImplementations);
    manifest.mechanism_implementations.push(...result.mechanismImplementations);
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
  manifest.check_implementations.sort(compareCheckImplementation);
  manifest.mechanism_implementations.sort(compareMechanism);
  manifest.artifacts.sort((a, b) => a.id.localeCompare(b.id));
  manifest.artifacts = manifest.artifacts.filter(
    (artifact, index, all) => index === 0 || artifact.id !== all[index - 1].id,
  );
  return { manifest, warnings };
}

function compareCheckImplementation(a: CheckImplementation, b: CheckImplementation): number {
  return (
    a.check.localeCompare(b.check) ||
    a.file.localeCompare(b.file) ||
    a.site.localeCompare(b.site)
  );
}

function compare(a: Entry, b: Entry): number {
  return (
    a.spec.localeCompare(b.spec) ||
    a.scenario.localeCompare(b.scenario) ||
    a.site.localeCompare(b.site)
  );
}

function compareMechanism(a: MechanismImplementation, b: MechanismImplementation): number {
  return (
    a.spec.localeCompare(b.spec) ||
    a.mechanism.localeCompare(b.mechanism) ||
    a.binding.localeCompare(b.binding)
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
 * A member is identified by its file, because that is the unit the router names.
 * Framework-generated pages (`/_not-found`, `/_global-error`) are excluded: they are not sites
 * the project wrote.
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
      source_fingerprint: sha256Fingerprint(fs.readFileSync(manifestPath)),
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
