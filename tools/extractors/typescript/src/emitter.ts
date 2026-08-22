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
const COMPILER_GLOBAL_TYPES = new Set([
  'Array',
  'Date',
  'Error',
  'Map',
  'Promise',
  'ReadonlyArray',
  'ReadonlyMap',
  'ReadonlySet',
  'RegExp',
  'Set',
  'WeakMap',
  'WeakSet',
]);

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
  site: string;
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
  const semantic = semanticSource(text, path.resolve(file));
  return scanSource(file, semantic.source, semantic.checker);
}

interface ProjectIdentity {
  packageName: string;
  moduleSpecifier: string;
}

function scanSource(
  file: string,
  source: ts.SourceFile,
  checker: ts.TypeChecker,
  project?: ProjectIdentity,
): ScanResult {
  const lang = languageOf(file);
  const result: ScanResult = {
    realizes: [],
    checkImplementations: [],
    mechanismImplementations: [],
    warnings: [],
  };
  const retiredImports = retiredMarkerImports(source);
  const mechanismImports = mechanismImportAccount(source);
  const mechanismSites = new Set<string>();

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

    if (ts.isCallExpression(node)) {
      const intended = isIntendedMechanismCall(node, checker, mechanismImports);
      const resolved = isMechanismMarker(node, checker);
      if (!intended && resolved) {
        throw mechanismSiteError(
          node,
          source,
          'must use a direct, aliased or namespace import from @azimuth-sh/annotations',
        );
      }
      if (!intended) return;
      if (!resolved) {
        throw mechanismSiteError(
          node,
          source,
          'does not resolve to @azimuth-sh/annotations implementsMechanism',
        );
      }
      if (!project) {
        throw mechanismSiteError(node, source, 'requires a configured project account');
      }
      const args = stringArgs(node);
      if (args.length !== 2 || node.arguments.length !== 2) {
        throw mechanismSiteError(
          node,
          source,
          'needs exactly two string literal spec and mechanism arguments',
        );
      }
      const site = resolveMechanismSite(
        node,
        source,
        checker,
        project,
      );
      const siteKey = `${lang}\u0000${site.name}`;
      if (mechanismSites.has(siteKey)) {
        const location = warn(node, source, file, '');
        throw new Error(
          `${file}:${location.line}: mechanism site \`${site.name}\` has repeated marker calls`,
        );
      }
      mechanismSites.add(siteKey);
      result.mechanismImplementations.push({
        spec: args[0],
        mechanism: args[1],
        site: site.name,
        binding: symbolBinding(lang, site.name),
        file,
        lang,
        source_fingerprint: site.fingerprint,
      });
      return;
    }

  });

  return result;
}

interface MechanismImportAccount {
  bindings: Map<string, ts.Identifier>;
  namespaces: Map<string, ts.Identifier>;
}

function mechanismImportAccount(source: ts.SourceFile): MechanismImportAccount {
  const bindings = new Map<string, ts.Identifier>();
  const namespaces = new Map<string, ts.Identifier>();
  for (const statement of source.statements) {
    if (!ts.isImportDeclaration(statement)
      || !ts.isStringLiteral(statement.moduleSpecifier)
      || statement.moduleSpecifier.text !== '@azimuth-sh/annotations') {
      continue;
    }
    const named = statement.importClause?.namedBindings;
    if (named && ts.isNamespaceImport(named)) {
      namespaces.set(named.name.text, named.name);
      continue;
    }
    if (!named || !ts.isNamedImports(named)) continue;
    for (const element of named.elements) {
      const imported = element.propertyName?.text ?? element.name.text;
      if (imported === 'implementsMechanism') {
        bindings.set(element.name.text, element.name);
      }
    }
  }
  return { bindings, namespaces };
}

function isIntendedMechanismCall(
  call: ts.CallExpression,
  checker: ts.TypeChecker,
  imports: MechanismImportAccount,
): boolean {
  if (ts.isIdentifier(call.expression)) {
    const imported = imports.bindings.get(call.expression.text);
    if (imported) {
      return checker.getSymbolAtLocation(call.expression) === checker.getSymbolAtLocation(imported);
    }
    return call.expression.text === 'implementsMechanism'
      && !checker.getSymbolAtLocation(call.expression);
  }
  if (!ts.isPropertyAccessExpression(call.expression)
    || !ts.isIdentifier(call.expression.expression)
    || call.expression.name.text !== 'implementsMechanism') {
    return false;
  }
  const imported = imports.namespaces.get(call.expression.expression.text);
  return imported !== undefined
    && checker.getSymbolAtLocation(call.expression.expression)
      === checker.getSymbolAtLocation(imported);
}

function isMechanismMarker(call: ts.CallExpression, checker: ts.TypeChecker): boolean {
  let symbol = checker.getSymbolAtLocation(calledNameNode(call.expression));
  if (!symbol) return false;
  if ((symbol.flags & ts.SymbolFlags.Alias) !== 0) {
    symbol = checker.getAliasedSymbol(symbol);
  }
  if (symbol.getName() !== 'implementsMechanism') return false;
  const declarations = symbol.declarations ?? [];
  return declarations.length > 0 && declarations.every((declaration) =>
    owningPackageName(declaration.getSourceFile().fileName) === '@azimuth-sh/annotations');
}

function calledNameNode(expression: ts.LeftHandSideExpression): ts.Node {
  return ts.isPropertyAccessExpression(expression) ? expression.name : expression;
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

interface SemanticSource {
  source: ts.SourceFile;
  checker: ts.TypeChecker;
}

function semanticSource(text: string, sourcePath: string): SemanticSource {
  const absolute = path.resolve(sourcePath);
  const options: ts.CompilerOptions = {
    allowJs: true,
    checkJs: true,
    jsx: ts.JsxEmit.Preserve,
    module: ts.ModuleKind.CommonJS,
    moduleResolution: ts.ModuleResolutionKind.Node10,
    target: ts.ScriptTarget.ES2022,
  };
  const host = ts.createCompilerHost(options, true);
  const originalGetSourceFile = host.getSourceFile.bind(host);
  const sameFile = (candidate: string): boolean => path.resolve(candidate) === absolute;
  host.fileExists = (candidate) => sameFile(candidate) || fs.existsSync(candidate);
  host.readFile = (candidate) => sameFile(candidate) ? text : fs.readFileSync(candidate, 'utf8');
  host.getSourceFile = (candidate, languageVersion, onError, shouldCreateNewSourceFile) => {
    if (sameFile(candidate)) {
      return ts.createSourceFile(
        absolute,
        text,
        languageVersion,
        true,
        scriptKind(absolute),
      );
    }
    return originalGetSourceFile(candidate, languageVersion, onError, shouldCreateNewSourceFile);
  };
  const program = ts.createProgram([absolute], options, host);
  const source = program.getSourceFile(absolute);
  if (!source) throw new Error(`${sourcePath}: compiler did not load the source file`);
  return { source, checker: program.getTypeChecker() };
}

interface PackageAccount {
  name: string;
  root: string;
}

function packageAccount(start: string): PackageAccount {
  let current = path.resolve(start);
  while (true) {
    const packageFile = path.join(current, 'package.json');
    if (fs.existsSync(packageFile)) {
      let document: unknown;
      try {
        document = JSON.parse(fs.readFileSync(packageFile, 'utf8'));
      } catch (error) {
        throw new Error(`${packageFile}: invalid package metadata: ${(error as Error).message}`);
      }
      const name = (document as { name?: unknown }).name;
      if (typeof name !== 'string' || !validPackageIdentity(name)) {
        throw new Error(`${packageFile}: mechanism extraction needs a semantic package name`);
      }
      return { name, root: current };
    }
    const parent = path.dirname(current);
    if (parent === current) {
      throw new Error(
        `${start}: mechanism extraction needs a package.json semantic module name`,
      );
    }
    current = parent;
  }
}

function owningPackageName(sourcePath: string): string | undefined {
  let current = path.dirname(path.resolve(sourcePath));
  while (true) {
    const packageFile = path.join(current, 'package.json');
    if (fs.existsSync(packageFile)) {
      try {
        const document = JSON.parse(fs.readFileSync(packageFile, 'utf8')) as { name?: unknown };
        return typeof document.name === 'string' ? document.name : undefined;
      } catch {
        return undefined;
      }
    }
    const parent = path.dirname(current);
    if (parent === current) return undefined;
    current = parent;
  }
}

function validPackageIdentity(value: string): boolean {
  return value === value.trim()
    && !/[\u0000-\u001f\u007f|]/.test(value)
    && /^(?:@[a-z0-9][a-z0-9._-]*\/)?[a-z0-9][a-z0-9._-]*$/.test(value);
}

type MechanismDeclaration =
  | ts.FunctionDeclaration
  | ts.MethodDeclaration
  | ts.VariableDeclaration;

function resolveMechanismSite(
  call: ts.CallExpression,
  source: ts.SourceFile,
  checker: ts.TypeChecker,
  project: ProjectIdentity,
): { name: string; fingerprint: string } {
  const declaration = enclosingMechanismDeclaration(call);
  if (!declaration) {
    throw mechanismSiteError(call, source, 'is not enclosed by a named callable declaration');
  }
  const nameNode = declaration.name;
  if (!nameNode || !ts.isIdentifier(nameNode)) {
    throw mechanismSiteError(call, source, 'uses an unsupported computed or anonymous symbol');
  }
  const symbol = checker.getSymbolAtLocation(nameNode);
  if (!symbol) {
    throw mechanismSiteError(call, source, 'has no compiler-resolved symbol');
  }
  const qualified = qualifiedDeclarationName(declaration, checker, call, source);
  const signatures = callableSignatures(symbol, declaration, checker, call, source);
  const receiver = receiverKind(declaration);
  const name = `${project.packageName}::${project.moduleSpecifier}::${receiver}::${qualified}` +
    signatureAccount(signatures, checker, declaration, call, source);
  if (name !== name.trim() || /[\u0000-\u001f\u007f|]/.test(name)) {
    throw mechanismSiteError(call, source, 'resolved to an invalid qualified identity');
  }
  if (/import\(["'](?:\/|\.\.?\/|[A-Za-z]:[\\/])/.test(name)) {
    throw mechanismSiteError(call, source, 'resolved a signature through a file path');
  }
  return { name, fingerprint: sha256Fingerprint(declaration.getText(source)) };
}

function receiverKind(declaration: MechanismDeclaration): 'static' | 'instance' | 'none' {
  if (!ts.isMethodDeclaration(declaration)) return 'none';
  if (!ts.isClassLike(declaration.parent)) return 'none';
  return hasModifier(declaration, ts.SyntaxKind.StaticKeyword) ? 'static' : 'instance';
}

function hasModifier(node: ts.Node, kind: ts.SyntaxKind): boolean {
  return ts.canHaveModifiers(node)
    && (ts.getModifiers(node)?.some((modifier) => modifier.kind === kind) ?? false);
}

function enclosingMechanismDeclaration(call: ts.CallExpression): MechanismDeclaration | undefined {
  let node: ts.Node | undefined = call.parent;
  while (node) {
    if (ts.isFunctionDeclaration(node) || ts.isMethodDeclaration(node)) return node;
    if (ts.isVariableDeclaration(node)
      && ts.isIdentifier(node.name)
      && node.initializer
      && (ts.isArrowFunction(node.initializer) || ts.isFunctionExpression(node.initializer))) {
      return node;
    }
    node = node.parent;
  }
  return undefined;
}

function qualifiedDeclarationName(
  declaration: MechanismDeclaration,
  checker: ts.TypeChecker,
  call: ts.CallExpression,
  source: ts.SourceFile,
): string {
  const parts: string[] = [];
  let node: ts.Node | undefined = declaration;
  while (node && !ts.isSourceFile(node)) {
    const named = semanticDeclarationName(node, checker);
    if (named) parts.unshift(named);
    node = node.parent;
  }
  if (parts.length === 0) {
    throw mechanismSiteError(call, source, 'has no compiler-qualified symbol name');
  }
  return parts.join('.');
}

function semanticDeclarationName(node: ts.Node, checker: ts.TypeChecker): string | undefined {
  let name: ts.DeclarationName | undefined;
  if (ts.isFunctionDeclaration(node)
    || ts.isMethodDeclaration(node)
    || ts.isClassDeclaration(node)
    || ts.isClassExpression(node)
    || ts.isVariableDeclaration(node)) {
    name = node.name;
  } else if (ts.isModuleDeclaration(node)) {
    name = node.name;
  }
  if (!name || (!ts.isIdentifier(name) && !ts.isStringLiteral(name))) return undefined;
  const symbol = checker.getSymbolAtLocation(name);
  if (!symbol) return undefined;
  const semanticName = symbol.getName();
  return semanticName === 'default' ? undefined : semanticName;
}

function callableSignatures(
  symbol: ts.Symbol,
  declaration: MechanismDeclaration,
  checker: ts.TypeChecker,
  call: ts.CallExpression,
  source: ts.SourceFile,
): readonly ts.Signature[] {
  const implementations = symbol.declarations?.filter(hasCallableBody).length ?? 0;
  if (implementations !== 1) {
    throw mechanismSiteError(call, source, 'does not resolve to exactly one implementation');
  }
  const type = checker.getTypeOfSymbolAtLocation(symbol, declaration);
  const signatures = checker.getSignaturesOfType(type, ts.SignatureKind.Call);
  if (signatures.length === 0) {
    throw mechanismSiteError(call, source, 'has no compiler-resolved call signature');
  }
  return signatures;
}

function hasCallableBody(declaration: ts.Declaration): boolean {
  if (ts.isFunctionDeclaration(declaration) || ts.isMethodDeclaration(declaration)) {
    return declaration.body !== undefined;
  }
  return ts.isVariableDeclaration(declaration)
    && declaration.initializer !== undefined
    && (ts.isArrowFunction(declaration.initializer)
      || ts.isFunctionExpression(declaration.initializer));
}

function signatureAccount(
  signatures: readonly ts.Signature[],
  checker: ts.TypeChecker,
  siteDeclaration: MechanismDeclaration,
  call: ts.CallExpression,
  source: ts.SourceFile,
): string {
  const accounts = signatures.map((signature) => {
    const typeParameters = signature.typeParameters ?? [];
    const genericPositions = new Map<ts.Symbol, number>();
    const genericNames: string[] = [];
    for (const [index, parameter] of typeParameters.entries()) {
      const symbol = parameter.symbol;
      const name = symbol?.getName();
      if (!symbol || !name || genericPositions.has(symbol) || genericNames.includes(name)) {
        throw mechanismSiteError(call, source, 'has ambiguous generic parameter identity');
      }
      genericPositions.set(symbol, index);
      genericNames.push(name);
    }
    const genericsAccount = { positions: genericPositions, names: genericNames };
    const generics = typeParameters.map((parameter, index) => {
      const constraint = checker.getBaseConstraintOfType(parameter);
      const declaration = parameter.symbol?.declarations?.find(ts.isTypeParameterDeclaration);
      const defaultType = declaration?.default
        ? checker.getTypeFromTypeNode(declaration.default)
        : undefined;
      const constraintAccount = constraint
        ? ` extends ${canonicalType(constraint, checker, declaration, genericsAccount)}`
        : '';
      const defaultAccount = defaultType
        ? `=${canonicalType(defaultType, checker, declaration, genericsAccount)}`
        : '';
      return `$${index}` +
        constraintAccount +
        defaultAccount;
    });
    const thisParameter = signature.thisParameter
      ? [canonicalThisParameter(
        signature.thisParameter,
        checker,
        signature.declaration ?? siteDeclaration,
        genericsAccount,
      )]
      : [];
    const parameters = signature.getParameters().map((parameter) => {
      const declaration = parameter.valueDeclaration ?? parameter.declarations?.[0];
      const type = checker.getTypeOfSymbolAtLocation(
        parameter,
        declaration ?? signature.declaration ?? siteDeclaration,
      );
      const rendered = canonicalType(type, checker, declaration, genericsAccount);
      const rest = declaration && ts.isParameter(declaration) && declaration.dotDotDotToken
        ? '...'
        : '';
      const optional = (parameter.flags & ts.SymbolFlags.Optional) !== 0 ? '?' : '';
      return `${rest}${rendered}${optional}`;
    });
    const result = canonicalType(
      checker.getReturnTypeOfSignature(signature),
      checker,
      signature.declaration ?? siteDeclaration,
      genericsAccount,
    );
    return `${generics.length > 0 ? `<${generics.join(',')}>` : ''}` +
      `(${[...thisParameter, ...parameters].join(',')}):${result}`;
  });
  const unique = [...new Set(accounts)].sort();
  if (unique.length !== accounts.length) {
    throw mechanismSiteError(call, source, 'has ambiguous duplicate call signatures');
  }
  return unique.length === 1 ? unique[0] : `{${unique.join(';')}}`;
}

interface GenericAccount {
  positions: ReadonlyMap<ts.Symbol, number>;
  names: readonly string[];
}

function canonicalThisParameter(
  parameter: ts.Symbol,
  checker: ts.TypeChecker,
  fallback: ts.Node,
  generics: GenericAccount,
): string {
  const declaration = parameter.valueDeclaration ?? parameter.declarations?.[0];
  const type = checker.getTypeOfSymbolAtLocation(parameter, declaration ?? fallback);
  return `this:${canonicalType(type, checker, declaration, generics)}`;
}

function canonicalType(
  type: ts.Type,
  checker: ts.TypeChecker,
  location: ts.Node | undefined,
  generics: GenericAccount,
): string {
  if ((type.flags & ts.TypeFlags.TypeParameter) !== 0) {
    const index = type.symbol ? generics.positions.get(type.symbol) : undefined;
    if (index === undefined) {
      throw new Error('compiler type parameter is outside the callable generic account');
    }
    return `$${index}`;
  }
  const node = checker.typeToTypeNode(
    type,
    undefined,
    ts.NodeBuilderFlags.NoTruncation |
      ts.NodeBuilderFlags.InTypeAlias |
      ts.NodeBuilderFlags.UseStructuralFallback,
  );
  if (!node) throw new Error('compiler could not render a canonical type identity');
  const printer = ts.createPrinter({ removeComments: true });
  const rendered = printer.printNode(
    ts.EmitHint.Unspecified,
    node,
    location?.getSourceFile() ?? ts.createSourceFile('type.ts', '', ts.ScriptTarget.Latest),
  );
  const canonical = canonicalTypeSyntax(rendered, generics.names);
  if (containsLocatorType(canonical)) {
    throw new Error(`compiler type identity contains a source locator: ${canonical}`);
  }
  return canonical;
}

interface TokenReplacement {
  end: number;
  text: string;
}

function canonicalTypeSyntax(text: string, genericNames: readonly string[]): string {
  const parameters = genericNames.length > 0 ? `<${genericNames.join(',')}>` : '';
  const prefix = `type __AzimuthCanonical${parameters} = `;
  const semantic = semanticSource(
    `${prefix}${text};`,
    path.resolve('.azimuth-canonical-type.ts'),
  );
  const declaration = semantic.source.statements.find(ts.isTypeAliasDeclaration);
  if (!declaration || (declaration.typeParameters?.length ?? 0) !== genericNames.length) {
    throw new Error('compiler could not bind canonical type syntax');
  }
  const genericSymbols = new Map<ts.Symbol, number>();
  for (const [index, parameter] of (declaration.typeParameters ?? []).entries()) {
    const symbol = semantic.checker.getSymbolAtLocation(parameter.name);
    if (!symbol) throw new Error('compiler could not bind canonical generic syntax');
    genericSymbols.set(symbol, index);
  }
  const replacements = new Map<number, TokenReplacement>();
  const typeStart = declaration.type.getStart(semantic.source);
  visit(declaration.type, (part) => {
    if (ts.isImportTypeNode(part)) {
      throw new Error('compiler type identity requires a path-bearing import locator');
    }
    if (ts.isTypeQueryNode(part)) {
      throw new Error('compiler type identity requires an unqualified value declaration');
    }
    if (ts.isIdentifier(part)) {
      const symbol = semantic.checker.getSymbolAtLocation(part);
      const index = symbol ? genericSymbols.get(symbol) : undefined;
      if (index !== undefined) {
        replacements.set(part.getStart(semantic.source) - typeStart, {
          end: part.end - typeStart,
          text: `$${index}`,
        });
      }
    }
    if (ts.isTypeReferenceNode(part)) {
      const name = entityNameText(part.typeName);
      const symbol = semantic.checker.getSymbolAtLocation(part.typeName);
      if ((symbol && (genericSymbols.has(symbol)
          || (symbol.flags & ts.SymbolFlags.TypeParameter) !== 0))
        || COMPILER_GLOBAL_TYPES.has(name)) {
        return;
      }
      throw new Error(`compiler cannot qualify named type ${name} without a source locator`);
    }
  });
  return canonicalTokens(text, replacements);
}

function entityNameText(name: ts.EntityName): string {
  return ts.isIdentifier(name)
    ? name.text
    : `${entityNameText(name.left)}.${name.right.text}`;
}

function canonicalTokens(
  text: string,
  replacements: ReadonlyMap<number, TokenReplacement>,
): string {
  const scanner = ts.createScanner(ts.ScriptTarget.Latest, true, ts.LanguageVariant.Standard, text);
  const tokens: Array<{ kind: ts.SyntaxKind; text: string; separated: boolean }> = [];
  let previousEnd = 0;
  for (let kind = scanner.scan(); kind !== ts.SyntaxKind.EndOfFileToken; kind = scanner.scan()) {
    const token = scanner.getTokenText();
    const replacement = replacements.get(scanner.getTokenPos());
    tokens.push({
      kind,
      text: replacement?.end === scanner.getTextPos() ? replacement.text : token,
      separated: scanner.getTokenPos() > previousEnd,
    });
    previousEnd = scanner.getTextPos();
  }
  return tokens.map((token, index) => {
    if (index === 0 || (!token.separated
        && !needsTokenSpace(tokens[index - 1].kind, token.kind))) {
      return token.text;
    }
    return ` ${token.text}`;
  }).join('');
}

function needsTokenSpace(left: ts.SyntaxKind, right: ts.SyntaxKind): boolean {
  return isWordToken(left) && isWordToken(right);
}

function isWordToken(kind: ts.SyntaxKind): boolean {
  return kind === ts.SyntaxKind.Identifier
    || kind === ts.SyntaxKind.PrivateIdentifier
    || kind === ts.SyntaxKind.NumericLiteral
    || kind === ts.SyntaxKind.BigIntLiteral
    || kind === ts.SyntaxKind.StringLiteral
    || (kind >= ts.SyntaxKind.FirstKeyword && kind <= ts.SyntaxKind.LastKeyword);
}

function containsLocatorType(value: string): boolean {
  return /import\(["'](?:\/|\.\.?\/|[A-Za-z]:[\\/])/.test(value);
}

function mechanismSiteError(
  node: ts.Node,
  source: ts.SourceFile,
  message: string,
): Error {
  const { line } = source.getLineAndCharacterOfPosition(node.getStart(source));
  return new Error(`${source.fileName}:${line + 1}: mechanism marker ${message}`);
}

function symbolBinding(lang: string, site: string): string {
  return `${lang}-symbol:${site}`;
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

const SOURCE = /\.(ts|tsx|mts|cts|js|jsx|mjs|cjs)$/;
const SKIP = new Set(['node_modules', 'dist', 'build', '.git', 'target']);

function isSourceFile(file: string): boolean {
  return SOURCE.test(file) && !/\.d\.(ts|mts|cts)$/.test(file);
}

export function walk(dir: string, out: string[] = []): string[] {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (SKIP.has(entry.name)) continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, out);
    else if (isSourceFile(entry.name)) out.push(full);
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

  const canonicalRoot = fs.realpathSync(repoRoot);
  const files = selectedFiles(roots, canonicalRoot);
  const project = configuredProject(files, canonicalRoot);
  const checker = project.program.getTypeChecker();

  const hasMechanismMarker = validateProjectMarkers(project.program, files);
  const relevantDiagnostics = projectDiagnostics(project.program);
  if (relevantDiagnostics.length > 0 && hasMechanismMarker) {
    throw new Error(formatDiagnostic(relevantDiagnostics[0]));
  }

  for (const file of files) {
    const relative = normalizedFile(canonicalRoot, file);
    const source = sourceFileByRealPath(project.program, file);
    if (!source) {
      throw new Error(`${relative}: configured compiler Program omitted selected source`);
    }
    const identity: ProjectIdentity = {
      packageName: project.package.name,
      moduleSpecifier: moduleSpecifier(source.fileName, project.package, project.program),
    };
    const result = scanSource(relative, source, checker, identity);
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
  manifest.artifacts.sort((a, b) => a.id.localeCompare(b.id) || a.file.localeCompare(b.file));
  validateMechanismSites(manifest.mechanism_implementations);
  return { manifest, warnings };
}

interface ConfiguredProject {
  configPath: string;
  package: PackageAccount;
  program: ts.Program;
}

function selectedFiles(roots: string[], repoRoot: string): string[] {
  const files = new Set<string>();
  for (const rawRoot of roots) {
    const selected = fs.realpathSync(rawRoot);
    ensureBelowRoot(selected, repoRoot, rawRoot);
    const stat = fs.statSync(selected);
    const candidates = stat.isDirectory() ? walk(selected) : [selected];
    for (const candidate of candidates) {
      if (!isSourceFile(candidate)) {
        throw new Error(`${candidate}: unsupported TypeScript/JavaScript source extension`);
      }
      const canonical = fs.realpathSync(candidate);
      ensureBelowRoot(canonical, repoRoot, candidate);
      files.add(canonical);
    }
  }
  if (files.size === 0) throw new Error('no TypeScript or JavaScript source files selected');
  return [...files].sort((left, right) => left.localeCompare(right));
}

function ensureBelowRoot(candidate: string, root: string, shown: string): void {
  const relative = path.relative(root, candidate);
  if (relative === '' || (!relative.startsWith(`..${path.sep}`) && relative !== '..'
      && !path.isAbsolute(relative))) {
    return;
  }
  throw new Error(`${shown}: input is outside --root`);
}

function normalizedFile(root: string, file: string): string {
  const relative = path.relative(root, file).split(path.sep).join('/');
  if (relative.length === 0 || relative.startsWith('/') || relative.includes('\\')
      || relative.split('/').some((segment) => !segment || segment === '.' || segment === '..')) {
    throw new Error(`${file}: source locator is not a normalized path below --root`);
  }
  return relative;
}

function configuredProject(files: string[], repoRoot: string): ConfiguredProject {
  const configs = new Set(files.map((file) => nearestConfig(file, repoRoot)));
  if (configs.size !== 1) {
    throw new Error('selected inputs span more than one configured TypeScript/JavaScript project');
  }
  const configPath = [...configs][0];
  const loaded = ts.readConfigFile(configPath, ts.sys.readFile);
  if (loaded.error) throw new Error(formatDiagnostic(loaded.error));
  const parsed = ts.parseJsonConfigFileContent(
    loaded.config,
    ts.sys,
    path.dirname(configPath),
    undefined,
    configPath,
  );
  if (parsed.errors.length > 0) throw new Error(formatDiagnostic(parsed.errors[0]));
  const program = ts.createProgram({
    rootNames: parsed.fileNames,
    options: parsed.options,
    projectReferences: parsed.projectReferences,
  });
  for (const file of files) {
    if (!sourceFileByRealPath(program, file)) {
      throw new Error(`${file}: selected source is outside the configured project`);
    }
  }
  return {
    configPath,
    package: packageAccount(path.dirname(configPath)),
    program,
  };
}

function nearestConfig(file: string, repoRoot: string): string {
  let current = path.dirname(file);
  while (true) {
    const candidates = ['tsconfig.json', 'jsconfig.json']
      .map((name) => path.join(current, name))
      .filter((candidate) => fs.existsSync(candidate));
    if (candidates.length > 1) {
      throw new Error(
        `${current}: both tsconfig.json and jsconfig.json define the nearest project`,
      );
    }
    if (candidates.length === 1) return fs.realpathSync(candidates[0]);
    if (current === repoRoot) break;
    const parent = path.dirname(current);
    if (parent === current || path.relative(repoRoot, parent).startsWith('..')) break;
    current = parent;
  }
  throw new Error(`${file}: no tsconfig.json or jsconfig.json project below --root`);
}

function sourceFileByRealPath(program: ts.Program, file: string): ts.SourceFile | undefined {
  const canonical = fs.realpathSync(file);
  return program.getSourceFiles().find((source) => {
    try {
      return fs.realpathSync(source.fileName) === canonical;
    } catch {
      return false;
    }
  });
}

function projectDiagnostics(program: ts.Program): readonly ts.Diagnostic[] {
  return [
    ...program.getOptionsDiagnostics(),
    ...program.getGlobalDiagnostics(),
    ...program.getSyntacticDiagnostics(),
    ...program.getSemanticDiagnostics(),
  ];
}

function validateProjectMarkers(program: ts.Program, files: string[]): boolean {
  const checker = program.getTypeChecker();
  let found = false;
  for (const file of files) {
    const source = sourceFileByRealPath(program, file);
    if (source) {
      const imports = mechanismImportAccount(source);
      visit(source, (node) => {
        if (!ts.isCallExpression(node)) return;
        if (!isIntendedMechanismCall(node, checker, imports)) return;
        found = true;
        const args = stringArgs(node);
        if (args.length !== 2 || node.arguments.length !== 2) {
          throw mechanismSiteError(
            node,
            source,
            'needs exactly two string literal spec and mechanism arguments',
          );
        }
      });
    }
  }
  return found;
}

function formatDiagnostic(diagnostic: ts.Diagnostic): string {
  const message = ts.flattenDiagnosticMessageText(diagnostic.messageText, '\n');
  if (!diagnostic.file || diagnostic.start === undefined) return `compiler diagnostic: ${message}`;
  const { line, character } = diagnostic.file.getLineAndCharacterOfPosition(diagnostic.start);
  return `${diagnostic.file.fileName}:${line + 1}:${character + 1}: ` +
    `compiler diagnostic: ${message}`;
}

function moduleSpecifier(
  sourceFile: string,
  account: PackageAccount,
  program: ts.Program,
): string {
  const canonical = fs.realpathSync(sourceFile);
  ensureBelowRoot(canonical, account.root, sourceFile);
  const relative = path.relative(account.root, canonical).split(path.sep).join('/');
  const modulePath = relative
    .replace(/\.mts$/, '.mjs')
    .replace(/\.cts$/, '.cjs')
    .replace(/\.(?:ts|tsx|js|jsx)$/, '');
  const specifier = modulePath === 'index' ? '.' : `./${modulePath}`;
  const anchor = path.join(account.root, '__azimuth_module_account__.ts');
  const resolved = ts.resolveModuleName(
    specifier,
    anchor,
    program.getCompilerOptions(),
    ts.sys,
  ).resolvedModule;
  let resolvedFile: string | undefined;
  if (resolved) {
    try {
      resolvedFile = fs.realpathSync(resolved.resolvedFileName);
    } catch {
      resolvedFile = undefined;
    }
  }
  if (resolvedFile !== canonical) {
    throw new Error(
      `${sourceFile}: package-relative module specifier \`${specifier}\` does not uniquely ` +
        'resolve to the selected source in the configured project',
    );
  }
  return specifier;
}

function validateMechanismSites(implementations: MechanismImplementation[]): void {
  const sites = new Map<string, MechanismImplementation>();
  for (const implementation of implementations) {
    const key = `${implementation.lang}\u0000${implementation.site}`;
    const prior = sites.get(key);
    if (prior) {
      throw new Error(
        `${implementation.file}: mechanism site \`${implementation.site}\` is already owned by ` +
          `${prior.spec}/${prior.mechanism} in ${prior.file}`,
      );
    }
    sites.set(key, implementation);
  }
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
