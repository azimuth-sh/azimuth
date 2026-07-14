//! Static-scan emitter: read TypeScript sources, find `realizes(...)` / `covers(...)` marker calls,
//! resolve each call's enclosing named symbol as the `site`, and produce a language-neutral manifest
//! (`schema/manifest.schema.json`) that `rtm` ingests. This is the polyglot path — the alternative
//! to `rtm` scanning comments directly, and the only path once tags are real language annotations.

import * as fs from 'node:fs';
import * as path from 'node:path';
import * as ts from 'typescript';
import type { Oracle, Quantification, Scope } from './markers';

const LANG = 'typescript';

const SCOPES: readonly Scope[] = ['unit', 'component', 'e2e'];
const QUANTIFICATIONS: readonly Quantification[] = ['example', 'invariant'];
const ORACLES: readonly Oracle[] = ['direct', 'golden', 'metamorphic', 'model-based', 'contract'];

export interface RealizesEntry {
  spec: string;
  req: string;
  scenario: string;
  site: string;
  file: string;
  lang: string;
}

export interface CoversEntry extends RealizesEntry {
  scope: Scope;
  quantification: Quantification;
  oracle?: Oracle;
}

/** A test in a tracing file that declares no scenario and is not opted out — the dual of uncovered. */
export interface UntracedTestEntry {
  site: string;
  file: string;
}

export interface Manifest {
  realizes: RealizesEntry[];
  covers: CoversEntry[];
  untraced_tests: UntracedTestEntry[];
}

/** A marker call the scanner could not turn into a manifest entry, with why — surfaced by the CLI. */
export interface ScanWarning {
  file: string;
  line: number;
  message: string;
}

export interface ScanResult {
  realizes: RealizesEntry[];
  covers: CoversEntry[];
  untraced: UntracedTestEntry[];
  warnings: ScanWarning[];
}

/** The test-launcher calls that name a single test case (a `describe` groups, it is not a case). */
const TEST_CASES: readonly string[] = ['test', 'it'];

/**
 * Scan one source file's text for marker calls. `file` is the path recorded in the manifest (already
 * made relative to the codebase root by the caller); `sourcePath` is only used to key the AST.
 */
export function scanText(text: string, file: string): ScanResult {
  const source = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true, scriptKindOf(file));
  const result: ScanResult = { realizes: [], covers: [], untraced: [], warnings: [] };

  const visit = (node: ts.Node): void => {
    if (ts.isCallExpression(node) && ts.isIdentifier(node.expression)) {
      const name = node.expression.text;
      if (name === 'realizes') {
        collectRealizes(node, source, file, result);
      } else if (name === 'covers') {
        collectCovers(node, source, file, result);
      }
    }
    ts.forEachChild(node, visit);
  };
  visit(source);

  // Scope rule: only a file that participates in tracing (≥1 covers) is held to the untraced-test
  // check — the TS analog of a C# class that opts into tracing.
  if (result.covers.length > 0) {
    collectUntraced(source, file, result);
  }

  return result;
}

/**
 * Flag each test case in a tracing file whose body carries neither a `covers` (it traces a scenario)
 * nor an `untraced` (a deliberate opt-out) marker — the dual of an uncovered scenario. A test case is
 * a `test(...)`/`it(...)` call named by a string literal; a `describe` groups cases and is not one.
 */
function collectUntraced(source: ts.SourceFile, file: string, result: ScanResult): void {
  const visit = (node: ts.Node): void => {
    if (isTestCase(node)) {
      const name = (node.arguments[0] as ts.StringLiteralLike).text;
      if (!subtreeHasMarker(node, ['covers', 'untraced'])) {
        result.untraced.push({ site: name, file });
      }
    }
    ts.forEachChild(node, visit);
  };
  visit(source);
}

/** A `test('name', …)` / `it('name', …)` call — a single named test case. */
function isTestCase(node: ts.Node): node is ts.CallExpression {
  if (!ts.isCallExpression(node) || !ts.isIdentifier(node.expression)) {
    return false;
  }
  if (!TEST_CASES.includes(node.expression.text)) {
    return false;
  }
  const first = node.arguments[0];
  return first !== undefined && ts.isStringLiteralLike(first);
}

/** Whether the node's subtree contains a call to any of the named markers. */
function subtreeHasMarker(node: ts.Node, markers: readonly string[]): boolean {
  let found = false;
  const visit = (current: ts.Node): void => {
    if (found) {
      return;
    }
    if (
      ts.isCallExpression(current) &&
      ts.isIdentifier(current.expression) &&
      markers.includes(current.expression.text)
    ) {
      found = true;
      return;
    }
    ts.forEachChild(current, visit);
  };
  ts.forEachChild(node, visit);
  return found;
}

function collectRealizes(
  call: ts.CallExpression,
  source: ts.SourceFile,
  file: string,
  result: ScanResult,
): void {
  const args = stringArgs(call);
  const [spec, req, scenario] = args ?? [];
  if (spec === undefined || req === undefined || scenario === undefined) {
    result.warnings.push(warn(call, source, file, 'realizes expects three string-literal arguments (spec, req, scenario)'));
    return;
  }
  result.realizes.push({ spec, req, scenario, site: resolveSite(call), file, lang: LANG });
}

function collectCovers(
  call: ts.CallExpression,
  source: ts.SourceFile,
  file: string,
  result: ScanResult,
): void {
  const args = stringArgs(call);
  const [spec, req, scenario, scopeRaw, quantRaw, oracleRaw] = args ?? [];
  if (
    spec === undefined ||
    req === undefined ||
    scenario === undefined ||
    scopeRaw === undefined ||
    quantRaw === undefined
  ) {
    result.warnings.push(warn(call, source, file, 'covers expects at least five string-literal arguments (spec, req, scenario, scope, quantification)'));
    return;
  }
  if (!isMember(scopeRaw, SCOPES)) {
    result.warnings.push(warn(call, source, file, `unknown scope "${scopeRaw}" (expected ${SCOPES.join(' | ')})`));
    return;
  }
  if (!isMember(quantRaw, QUANTIFICATIONS)) {
    result.warnings.push(warn(call, source, file, `unknown quantification "${quantRaw}" (expected ${QUANTIFICATIONS.join(' | ')})`));
    return;
  }
  const entry: CoversEntry = {
    spec,
    req,
    scenario,
    site: resolveSite(call),
    file,
    lang: LANG,
    scope: scopeRaw,
    quantification: quantRaw,
  };
  if (oracleRaw !== undefined) {
    if (!isMember(oracleRaw, ORACLES)) {
      result.warnings.push(warn(call, source, file, `unknown oracle "${oracleRaw}" (expected ${ORACLES.join(' | ')})`));
      return;
    }
    entry.oracle = oracleRaw;
  }
  result.covers.push(entry);
}

/** The literal values of a call's arguments, or `undefined` if any argument is not a string literal. */
function stringArgs(call: ts.CallExpression): string[] | undefined {
  const values: string[] = [];
  for (const arg of call.arguments) {
    if (ts.isStringLiteralLike(arg)) {
      values.push(arg.text);
    } else {
      return undefined;
    }
  }
  return values;
}

/**
 * The name of the symbol that encloses the marker call — the tagged site. Ascends the parent chain
 * and takes the nearest of: a named function/method declaration, a named `const`/`let` binding (an
 * arrow or function assigned to a name — Server Components, route exports), or a `test`/`it`/`describe`
 * call whose first argument is a string literal (how TS tests name themselves). Nearest wins, so a
 * `covers` inside `test('…', () => …)` names the test, while a `realizes` in `export function GET`
 * names the handler.
 */
function resolveSite(call: ts.CallExpression): string {
  let node: ts.Node | undefined = call.parent;
  while (node !== undefined) {
    if ((ts.isFunctionDeclaration(node) || ts.isMethodDeclaration(node)) && node.name !== undefined) {
      return node.name.getText();
    }
    if (ts.isVariableDeclaration(node) && ts.isIdentifier(node.name)) {
      return node.name.text;
    }
    if (ts.isCallExpression(node) && ts.isIdentifier(node.expression) && ['test', 'it', 'describe'].includes(node.expression.text)) {
      const first = node.arguments[0];
      if (first !== undefined && ts.isStringLiteralLike(first)) {
        return first.text;
      }
    }
    node = node.parent;
  }
  return 'unknown';
}

function warn(node: ts.Node, source: ts.SourceFile, file: string, message: string): ScanWarning {
  const { line } = source.getLineAndCharacterOfPosition(node.getStart(source));
  return { file, line: line + 1, message };
}

function isMember<T extends string>(value: string, members: readonly T[]): value is T {
  return (members as readonly string[]).includes(value);
}

function scriptKindOf(file: string): ts.ScriptKind {
  return file.endsWith('.tsx') ? ts.ScriptKind.TSX : ts.ScriptKind.TS;
}

export interface EmitOptions {
  root: string;
  include: string[];
}

export interface EmitOutput {
  manifest: Manifest;
  warnings: ScanWarning[];
  files: string[];
}

/** Resolve globs under `root`, scan every matched .ts/.tsx, and assemble a deterministic manifest. */
export function emit(options: EmitOptions): EmitOutput {
  const root = path.resolve(options.root);
  const include = options.include.length > 0 ? options.include : ['**/*.ts', '**/*.tsx'];
  const files = ts.sys
    .readDirectory(root, ['.ts', '.tsx'], undefined, include)
    .filter((file) => !file.endsWith('.d.ts'));

  const manifest: Manifest = { realizes: [], covers: [], untraced_tests: [] };
  const warnings: ScanWarning[] = [];
  const scanned: string[] = [];

  for (const absolute of files) {
    const relative = path.relative(root, absolute).split(path.sep).join('/');
    const text = fs.readFileSync(absolute, 'utf8');
    const result = scanText(text, relative);
    manifest.realizes.push(...result.realizes);
    manifest.covers.push(...result.covers);
    manifest.untraced_tests.push(...result.untraced);
    warnings.push(...result.warnings);
    scanned.push(relative);
  }

  manifest.realizes.sort(compareRealizes);
  manifest.covers.sort(compareRealizes);
  manifest.untraced_tests.sort(compareUntraced);

  return { manifest, warnings, files: scanned };
}

function compareUntraced(a: UntracedTestEntry, b: UntracedTestEntry): number {
  return a.file.localeCompare(b.file) || a.site.localeCompare(b.site);
}

function compareRealizes(a: RealizesEntry, b: RealizesEntry): number {
  return (
    a.file.localeCompare(b.file) ||
    a.site.localeCompare(b.site) ||
    a.spec.localeCompare(b.spec) ||
    a.req.localeCompare(b.req) ||
    a.scenario.localeCompare(b.scenario)
  );
}
