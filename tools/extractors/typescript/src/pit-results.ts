import { createHash } from 'node:crypto';
import * as path from 'node:path';
import { Entry, Manifest, ObservationBinding } from './emitter';
import { emptyManifest, subject } from './observations';
import { parseXml, XmlElement } from './pit-xml';

const selectionSchema = 'azimuth-pit-selection/1';
const statuses = [
  'KILLED',
  'SURVIVED',
  'NO_COVERAGE',
  'TIMED_OUT',
  'NON_VIABLE',
  'RUN_ERROR',
  'MEMORY_ERROR',
  'STARTED',
  'NOT_STARTED',
  'EQUIVALENT',
] as const;
type PitStatus = typeof statuses[number];

interface SelectedTest {
  site: string;
  pitNames: string[];
}

interface PitSelection {
  targetClasses: string[];
  selectedTests: SelectedTest[];
}

interface PitMutation {
  status: PitStatus;
  detected: boolean;
  testsRun: number;
  sourceFile: string;
  mutatedClass: string;
  mutatedMethod: string;
  methodDescription: string;
  line: number;
  mutator: string;
  description: string;
  killingTest: string;
}

export interface PitImport {
  reportPath: string;
  reportSource: string;
  configPath: string;
  configSource: string;
  selectionPath: string;
  selectionSource: string;
  linkage: Manifest;
  toolVersion: string;
}

export function importPitResults(input: PitImport): Manifest {
  const selection = readSelectionSource(input.selectionSource);
  const report = readReport(input.reportSource);
  const mutations = report.mutations;
  validateSelection(selection, mutations, input.linkage);
  const selectedCovers = selection.selectedTests.flatMap((test) =>
    input.linkage.covers.filter((site) => site.site === test.site && jvm(site)),
  );
  const bindings: ObservationBinding[] = [];
  const assessments: unknown[] = [];

  for (const claim of claims(selectedCovers)) {
    const claimCovers = selectedCovers.filter((site) => sameClaim(site, claim));
    const claimRealizes = input.linkage.realizes.filter(
      (site) => sameClaim(site, claim) && jvm(site),
    );
    const claimMutations = mutations.filter((mutation) =>
      claimRealizes.some((site) => mutationTargets(site, mutation))
        && testsFor(mutation, selection).some(
          (test) => claimCovers.some((site) => site.site === test),
        ),
    );
    if (claimMutations.length === 0) continue;

    const targetRealizes = claimRealizes.filter((site) =>
      claimMutations.some((mutation) => mutationTargets(site, mutation)),
    );
    const usedTestSites = new Set(
      claimMutations.flatMap((mutation) => testsFor(mutation, selection)),
    );
    const targetCovers = claimCovers.filter((site) => usedTestSites.has(site.site));
    const testSites = unique(targetCovers.map((site) => site.site)).sort();
    const mutationItems = claimMutations.map((mutation) => ({
      file: targetRealizes.find((site) => mutationTargets(site, mutation))?.file
        ?? mutation.sourceFile,
      line: mutation.line,
      status: mutation.status,
      detected: mutation.detected,
      mutated_class: mutation.mutatedClass,
      mutated_method: mutation.mutatedMethod,
      method_description: mutation.methodDescription,
      mutator: mutation.mutator,
      description: mutation.description,
      tests_run: mutation.testsRun,
      ...(mutation.killingTest ? { killing_test: mutation.killingTest } : {}),
    }));
    const reviewItems = mutationItems.filter((mutation) => mutation.status !== 'KILLED');
    assessments.push({
      spec: claim.spec,
      scenario: claim.scenario,
      target_files: unique(targetRealizes.map((site) => site.file)).sort(),
      target_classes: unique(claimMutations.map((mutation) => mutation.mutatedClass)).sort(),
      test_sites: testSites,
      killed: count(claimMutations, 'KILLED'),
      survived: count(claimMutations, 'SURVIVED'),
      no_coverage: count(claimMutations, 'NO_COVERAGE'),
      timed_out: count(claimMutations, 'TIMED_OUT'),
      non_viable: count(claimMutations, 'NON_VIABLE'),
      runtime_error: count(claimMutations, 'RUN_ERROR') + count(claimMutations, 'MEMORY_ERROR'),
      pending: count(claimMutations, 'STARTED') + count(claimMutations, 'NOT_STARTED'),
      equivalent: count(claimMutations, 'EQUIVALENT'),
      mutants: mutationItems,
      review_items: reviewItems,
    });
    bindings.push({
      role: 'challenge',
      spec: claim.spec,
      scenario: claim.scenario,
      assertion: 'selected evidence rejects generated changes at linked realization sites',
      outcome: reviewItems.length === 0 ? 'clean' : 'findings',
      subjects: [
        ...uniqueEntries(targetRealizes).map((site) => ({
          relation: 'realization' as const,
          identity: subject(site),
        })),
        ...uniqueEntries(targetCovers).map((site) => ({
          relation: 'evidence' as const,
          identity: subject(site),
        })),
      ],
    });
  }

  if (bindings.length === 0) {
    throw new Error('PIT selection and report intersect no linked claims');
  }
  bindings.sort((left, right) =>
    left.spec.localeCompare(right.spec) || left.scenario.localeCompare(right.scenario),
  );
  const fingerprint = createHash('sha256')
    .update(input.reportSource)
    .update('\0')
    .update(input.configSource)
    .update('\0')
    .update(input.selectionSource)
    .update('\0')
    .update(input.toolVersion)
    .digest('hex');

  return emptyManifest([{
    id: `pit-${createHash('sha256').update(input.reportPath).digest('hex').slice(0, 16)}`,
    kind: 'mutation-test',
    tool: 'PIT',
    tool_version: input.toolVersion,
    report: input.reportPath,
    inputs: [input.configPath, input.selectionPath],
    source_fingerprint: fingerprint,
    bindings,
    payload: {
      schema: 'azimuth-pit-summary/1',
      report_partial: report.partial,
      assessments,
    },
  }]);
}

function readSelectionSource(source: string): PitSelection {
  try {
    return readSelection(JSON.parse(source) as unknown);
  } catch (error) {
    if (error instanceof SyntaxError) throw new Error('PIT selection must be valid JSON');
    throw error;
  }
}

function readSelection(value: unknown): PitSelection {
  if (!record(value) || value.schema !== selectionSchema) {
    throw new Error(`PIT selection schema must be ${selectionSchema}`);
  }
  knownKeys(value, ['schema', 'target_classes', 'selected_tests'], 'PIT selection');
  const targetClasses = stringList(value.target_classes, 'PIT selection target_classes');
  if (!Array.isArray(value.selected_tests) || value.selected_tests.length === 0) {
    throw new Error('PIT selection selected_tests must be a non-empty array');
  }
  const selectedTests = value.selected_tests.map((item, index) => {
    if (typeof item === 'string' && item.length > 0) return { site: item, pitNames: [item] };
    if (!record(item)) throw new Error(`PIT selection selected_tests[${index}] must name a site`);
    knownKeys(item, ['site', 'pit_names'], `PIT selection selected_tests[${index}]`);
    if (typeof item.site !== 'string' || item.site.length === 0) {
      throw new Error(`PIT selection selected_tests[${index}] must name a site`);
    }
    const pitNames = item.pit_names === undefined
      ? [item.site]
      : stringList(item.pit_names, `PIT selection selected_tests[${index}].pit_names`);
    return { site: item.site, pitNames: unique([item.site, ...pitNames]) };
  });
  if (unique(targetClasses).length !== targetClasses.length) {
    throw new Error('PIT selection target_classes contains duplicates');
  }
  if (unique(selectedTests.map((test) => test.site)).length !== selectedTests.length) {
    throw new Error('PIT selection selected_tests contains duplicate sites');
  }
  const aliases = selectedTests.flatMap(
    (test) => test.pitNames.map((name) => [name, test.site] as const),
  );
  for (const [name, site] of aliases) {
    if (aliases.some(([otherName, otherSite]) => otherName === name && otherSite !== site)) {
      throw new Error(`PIT test name ${name} resolves to more than one selected site`);
    }
  }
  return { targetClasses, selectedTests };
}

function readReport(source: string): { mutations: PitMutation[]; partial: boolean } {
  const root = parseXml(source);
  if (root.name !== 'mutations' || root.text.trim()) {
    throw new Error('PIT XML schema must have a mutations root');
  }
  knownKeys(root.attributes, ['partial'], 'PIT mutations root');
  const partialValue = root.attributes.partial ?? 'false';
  if (partialValue !== 'true' && partialValue !== 'false') {
    throw new Error('PIT mutations root partial must be true or false');
  }
  if (root.children.some((child) => child.name !== 'mutation')) {
    throw new Error('PIT XML schema contains an unknown root element');
  }
  if (root.children.length === 0) throw new Error('PIT XML report contains no mutations');
  return {
    mutations: root.children.map((node, index) => mutation(node, index)),
    partial: partialValue === 'true',
  };
}

function mutation(node: XmlElement, index: number): PitMutation {
  knownKeys(node.attributes, ['detected', 'status', 'numberOfTestsRun'], `PIT mutation ${index}`);
  if (node.text.trim()) throw new Error(`PIT mutation ${index} contains mixed text`);
  const detected = node.attributes.detected;
  if (detected !== 'true' && detected !== 'false') {
    throw new Error(`PIT mutation ${index} detected must be true or false`);
  }
  const status = node.attributes.status;
  if (!isStatus(status)) {
    throw new Error(`PIT mutation ${index} has unknown status ${status ?? '(missing)'}`);
  }
  const testsRun = integer(
    node.attributes.numberOfTestsRun,
    `PIT mutation ${index} numberOfTestsRun`,
  );
  const allowed = [
    'sourceFile', 'mutatedClass', 'mutatedMethod', 'methodDescription', 'lineNumber', 'mutator',
    'index', 'indexes', 'block', 'blocks', 'killingTest', 'description',
  ];
  for (const child of node.children) {
    if (!allowed.includes(child.name)) {
      throw new Error(`PIT mutation ${index} contains unknown element ${child.name}`);
    }
  }
  for (const name of ['indexes', 'blocks']) {
    for (const container of node.children.filter((child) => child.name === name)) {
      const member = name === 'indexes' ? 'index' : 'block';
      if (container.text.trim() || container.children.some((child) => child.name !== member)) {
        throw new Error(`PIT mutation ${index} has malformed ${name}`);
      }
      for (const child of container.children) scalar(child, `${name}/${member}`);
    }
  }
  return {
    status,
    detected: detected === 'true',
    testsRun,
    sourceFile: requiredText(node, 'sourceFile', index),
    mutatedClass: requiredText(node, 'mutatedClass', index),
    mutatedMethod: requiredText(node, 'mutatedMethod', index),
    methodDescription: requiredText(node, 'methodDescription', index, true),
    line: integer(requiredText(node, 'lineNumber', index), `PIT mutation ${index} lineNumber`),
    mutator: requiredText(node, 'mutator', index),
    description: requiredText(node, 'description', index),
    killingTest: optionalText(node, 'killingTest', index),
  };
}

function validateSelection(
  selection: PitSelection,
  mutations: PitMutation[],
  linkage: Manifest,
): void {
  for (const target of selection.targetClasses) {
    if (!mutations.some((mutation) => selectedClass(target, mutation.mutatedClass))) {
      throw new Error(`PIT target class ${target} has no reported mutations`);
    }
    if (!linkage.realizes.some((site) => jvm(site) && classSite(site.site, target))) {
      throw new Error(`PIT target class ${target} resolves to no JVM Realizes site`);
    }
  }
  for (const mutation of mutations) {
    if (!selection.targetClasses.some((target) => selectedClass(target, mutation.mutatedClass))) {
      throw new Error(
        `PIT report mutation class ${mutation.mutatedClass} is outside exact selection`,
      );
    }
    if (!linkage.realizes.some((site) => jvm(site) && mutationTargets(site, mutation))) {
      throw new Error(
        `PIT mutation ${mutation.mutatedClass}:${mutation.line} resolves to no JVM Realizes site`,
      );
    }
  }
  for (const test of selection.selectedTests) {
    if (!linkage.covers.some((site) => jvm(site) && site.site === test.site)) {
      throw new Error(`PIT selected test ${test.site} resolves to no JVM Covers site`);
    }
  }
  for (const mutation of mutations.filter((item) => item.killingTest)) {
    if (testsFor(mutation, selection).length === 0) {
      throw new Error(`PIT killing test ${mutation.killingTest} resolves to no selected test`);
    }
  }
}

function testsFor(mutation: PitMutation, selection: PitSelection): string[] {
  if (!mutation.killingTest) return selection.selectedTests.map((test) => test.site);
  const normalized = normalizeTestName(mutation.killingTest);
  return selection.selectedTests
    .filter((test) => test.pitNames.includes(mutation.killingTest)
      || test.pitNames.includes(normalized)
      || test.site === normalized)
    .map((test) => test.site);
}

function normalizeTestName(value: string): string {
  const methodAndClass = /^(.*)\(([^()]+)\)$/.exec(value);
  return methodAndClass ? `${methodAndClass[2]}.${methodAndClass[1]}` : value;
}

function mutationTargets(site: Entry, mutation: PitMutation): boolean {
  const sourceMatches = path.posix.basename(site.file) === path.posix.basename(mutation.sourceFile);
  return sourceMatches && classSite(site.site, mutation.mutatedClass.replace(/\$.*/, ''));
}

function classSite(site: string, className: string): boolean {
  const outer = className.replace(/\$.*/, '');
  return site === outer || site.startsWith(`${outer}.`);
}

function selectedClass(target: string, reported: string): boolean {
  return reported === target || reported.startsWith(`${target}$`);
}

function requiredText(node: XmlElement, name: string, index: number, allowEmpty = false): string {
  const matches = node.children.filter((child) => child.name === name);
  if (matches.length !== 1) throw new Error(`PIT mutation ${index} must contain one ${name}`);
  const value = scalar(matches[0], name);
  if (!allowEmpty && value.length === 0) throw new Error(`PIT mutation ${index} has empty ${name}`);
  return value;
}

function optionalText(node: XmlElement, name: string, index: number): string {
  const matches = node.children.filter((child) => child.name === name);
  if (matches.length > 1) throw new Error(`PIT mutation ${index} contains repeated ${name}`);
  return matches.length === 0 ? '' : scalar(matches[0], name);
}

function scalar(node: XmlElement, name: string): string {
  if (Object.keys(node.attributes).length > 0 || node.children.length > 0) {
    throw new Error(`PIT XML ${name} must contain text only`);
  }
  return node.text.trim();
}

function count(mutations: PitMutation[], status: PitStatus): number {
  return mutations.filter((mutation) => mutation.status === status).length;
}

function claims(sites: Entry[]): Array<{ spec: string; scenario: string }> {
  const found = new Map<string, { spec: string; scenario: string }>();
  for (const site of sites) found.set(`${site.spec}\0${site.scenario}`, site);
  return [...found.values()];
}

function sameClaim(site: Entry, claim: { spec: string; scenario: string }): boolean {
  return site.spec === claim.spec && site.scenario === claim.scenario;
}

function uniqueEntries(entries: Entry[]): Entry[] {
  const found = new Map<string, Entry>();
  for (const entry of entries) found.set(subject(entry), entry);
  return [...found.values()].sort((left, right) => subject(left).localeCompare(subject(right)));
}

function unique(values: string[]): string[] {
  return [...new Set(values)];
}

function stringList(value: unknown, name: string): string[] {
  if (!Array.isArray(value) || value.length === 0
      || value.some((item) => typeof item !== 'string' || item.length === 0)) {
    throw new Error(`${name} must be a non-empty string array`);
  }
  return value;
}

function integer(value: unknown, name: string): number {
  const parsed = typeof value === 'string' && /^\d+$/.test(value) ? Number(value) : value;
  if (typeof parsed !== 'number' || !Number.isSafeInteger(parsed) || parsed < 0) {
    throw new Error(`${name} must be a non-negative integer`);
  }
  return parsed;
}

function isStatus(value: string | undefined): value is PitStatus {
  return typeof value === 'string' && (statuses as readonly string[]).includes(value);
}

function jvm(site: Entry): boolean {
  return site.lang === 'java' || site.lang === 'kotlin';
}

function knownKeys(value: Record<string, unknown>, allowed: string[], name: string): void {
  const unknown = Object.keys(value).find((key) => !allowed.includes(key));
  if (unknown) throw new Error(`${name} contains unknown field ${unknown}`);
}

function record(value: unknown): value is Record<string, any> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
