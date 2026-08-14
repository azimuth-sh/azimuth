/**
 * Imports a Stryker report as judgment input without promoting it to claim evidence.
 *
 * Claim selection is derived from the ordinary manifest: a selected Stryker test must be a Covers
 * site, and the same claim must have a Realizes site in a mutated file. This avoids a second manual
 * claim-to-test map whose mistakes would look authoritative.
 */
import { createHash } from 'node:crypto';
import * as path from 'node:path';
import { Entry, Manifest, ObservationBinding } from './emitter';

interface Mutant {
  status: MutationStatus;
  mutatorName: string;
  replacement: string;
  location: { start: { line: number } };
}

const mutationStatuses = [
  'Killed',
  'Survived',
  'NoCoverage',
  'Timeout',
  'CompileError',
  'RuntimeError',
  'Pending',
  'Ignored',
] as const;
type MutationStatus = typeof mutationStatuses[number];

interface StrykerFile {
  mutants: Mutant[];
}

interface StrykerTest {
  id: string;
  name: string;
}

interface StrykerTestFile {
  tests: StrykerTest[];
}

interface StrykerReport {
  schemaVersion: string;
  files: Record<string, StrykerFile>;
  testFiles: Record<string, StrykerTestFile>;
}

export interface MutationImport {
  report: unknown;
  reportPath: string;
  reportSource: string;
  configPath: string;
  configSource: string;
  linkage: Manifest;
  root: string;
  toolVersion: string;
}

export function importMutationResults(input: MutationImport): Manifest {
  const report = readReport(input.report);
  const selectedTests = new Set(
    Object.values(report.testFiles).flatMap((file) => file.tests.map((test) => test.name)),
  );
  const mutatedFiles = new Map<string, Mutant[]>();
  for (const [file, value] of Object.entries(report.files)) {
    if (value.mutants.length === 0) continue;
    mutatedFiles.set(relative(input.root, file), value.mutants);
  }

  const selectedCovers = input.linkage.covers.filter((site) => selectedTests.has(site.site));
  const claims = uniqueClaims(selectedCovers);
  const bindings: ObservationBinding[] = [];
  const assessments: unknown[] = [];
  for (const claim of claims) {
    const realizationFiles = input.linkage.realizes
      .filter((site) => site.spec === claim.spec && site.scenario === claim.scenario)
      .map((site) => site.file);
    const targetFiles = [...new Set(realizationFiles.filter((file) => mutatedFiles.has(file)))].sort();
    if (targetFiles.length === 0) continue;
    const mutants = targetFiles.flatMap((file) => mutatedFiles.get(file) ?? []);
    const executed = ['Killed', 'Survived', 'NoCoverage', 'Timeout', 'RuntimeError']
      .reduce((total, status) => total + count(mutants, status as MutationStatus), 0);
    if (executed === 0) {
      throw new Error(
        `Stryker report has no executed mutants for ${claim.spec}#${claim.scenario}; check mutate filters`,
      );
    }
    const reviewItems = targetFiles.flatMap((file) =>
      (mutatedFiles.get(file) ?? [])
        .filter((mutant) => reviewStatus(mutant.status))
        .map((mutant) => ({
          file,
          line: mutant.location.start.line,
          status: mutant.status,
          mutator: mutant.mutatorName,
          replacement: mutant.replacement,
        })),
    );
    assessments.push({
      spec: claim.spec,
      scenario: claim.scenario,
      target_files: targetFiles,
      test_sites: selectedCovers
        .filter((site) => site.spec === claim.spec && site.scenario === claim.scenario)
        .map((site) => site.site)
        .sort(),
      killed: count(mutants, 'Killed'),
      survived: count(mutants, 'Survived'),
      no_coverage: count(mutants, 'NoCoverage'),
      timeout: count(mutants, 'Timeout'),
      compile_error: count(mutants, 'CompileError'),
      runtime_error: count(mutants, 'RuntimeError'),
      pending: count(mutants, 'Pending'),
      ignored: count(mutants, 'Ignored'),
      review_items: reviewItems,
    });
    const claimCovers = selectedCovers.filter(
      (site) => site.spec === claim.spec && site.scenario === claim.scenario,
    );
    const claimRealizes = input.linkage.realizes.filter(
      (site) => site.spec === claim.spec
        && site.scenario === claim.scenario
        && targetFiles.includes(site.file),
    );
    bindings.push({
      role: 'challenge',
      spec: claim.spec,
      scenario: claim.scenario,
      assertion: 'selected evidence rejects generated changes at linked realization sites',
      outcome: reviewItems.length === 0 ? 'clean' : 'findings',
      subjects: [
        ...claimRealizes.map((site) => ({ relation: 'realization' as const, identity: subject(site) })),
        ...claimCovers.map((site) => ({ relation: 'evidence' as const, identity: subject(site) })),
      ],
    });
  }
  bindings.sort((left, right) =>
    left.spec.localeCompare(right.spec) || left.scenario.localeCompare(right.scenario),
  );

  const sourceFingerprint = createHash('sha256')
    .update(input.reportSource)
    .update('\0')
    .update(input.configSource)
    .update('\0')
    .update(input.toolVersion)
    .digest('hex');

  return {
    realizes: [],
    covers: [],
    mechanism_implementations: [],
    mechanism_covers: [],
    class_members: [],
    enumerations: [],
    artifacts: [],
    observations: bindings.length === 0 ? [] : [{
      id: `mutation-${createHash('sha256').update(input.reportPath).digest('hex').slice(0, 16)}`,
      kind: 'mutation-test',
      tool: 'Stryker.NET',
      tool_version: input.toolVersion,
      report: input.reportPath,
      inputs: [input.configPath],
      source_fingerprint: sourceFingerprint,
      bindings,
      payload: { schema: 'azimuth-mutation-summary/1', assessments },
    }],
  };
}

function subject(site: Entry): string {
  if (site.area && site.address_kind && site.address) {
    return `${site.area}|${site.address_kind}|${site.address}`;
  }
  return `${site.file}#${site.site}|${site.lang}`;
}

function readReport(value: unknown): StrykerReport {
  if (!record(value)) throw new Error('Stryker report must be an object');
  if (value.schemaVersion !== '2') throw new Error('Stryker report schemaVersion must be 2');
  if (!record(value.files)) throw new Error('Stryker report files must be an object');
  if (!record(value.testFiles)) throw new Error('Stryker report testFiles must be an object');
  const files: Record<string, StrykerFile> = {};
  for (const [file, item] of Object.entries(value.files)) {
    if (!record(item) || !Array.isArray(item.mutants)) {
      throw new Error(`Stryker file ${file} has no mutants array`);
    }
    const mutants = item.mutants.map((mutant, index) => {
      if (!record(mutant) || typeof mutant.status !== 'string') {
        throw new Error(`Stryker file ${file} mutant ${index} has no status`);
      }
      if (!isMutationStatus(mutant.status)) {
        throw new Error(`Stryker file ${file} mutant ${index} has unknown status ${mutant.status}`);
      }
      if (typeof mutant.mutatorName !== 'string' || typeof mutant.replacement !== 'string'
          || !record(mutant.location) || !record(mutant.location.start)
          || typeof mutant.location.start.line !== 'number') {
        throw new Error(`Stryker file ${file} mutant ${index} has incomplete review metadata`);
      }
      return {
        status: mutant.status,
        mutatorName: mutant.mutatorName,
        replacement: mutant.replacement,
        location: { start: { line: mutant.location.start.line } },
      };
    });
    files[file] = { mutants };
  }
  const testFiles: Record<string, StrykerTestFile> = {};
  for (const [file, item] of Object.entries(value.testFiles)) {
    if (!record(item) || !Array.isArray(item.tests)) {
      throw new Error(`Stryker test file ${file} has no tests array`);
    }
    const tests = item.tests.map((test, index) => {
      if (!record(test) || typeof test.id !== 'string' || typeof test.name !== 'string') {
        throw new Error(`Stryker test file ${file} test ${index} has no id or name`);
      }
      return { id: test.id, name: test.name };
    });
    testFiles[file] = { tests };
  }
  return { schemaVersion: '2', files, testFiles };
}

function uniqueClaims(sites: Entry[]): Array<{ spec: string; scenario: string }> {
  const keys = new Set<string>();
  const claims: Array<{ spec: string; scenario: string }> = [];
  for (const site of sites) {
    const key = `${site.spec}\0${site.scenario}`;
    if (keys.has(key)) continue;
    keys.add(key);
    claims.push({ spec: site.spec, scenario: site.scenario });
  }
  return claims;
}

function relative(root: string, file: string): string {
  return path.relative(path.resolve(root), path.resolve(file)).split(path.sep).join('/');
}

function count(mutants: Mutant[], status: MutationStatus): number {
  return mutants.filter((mutant) => mutant.status === status).length;
}

function isMutationStatus(value: string): value is MutationStatus {
  return (mutationStatuses as readonly string[]).includes(value);
}

function reviewStatus(value: MutationStatus): boolean {
  return ['Survived', 'NoCoverage', 'Timeout', 'RuntimeError', 'Pending'].includes(value);
}

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
