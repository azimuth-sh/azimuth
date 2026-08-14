import { createHash } from 'node:crypto';
import * as path from 'node:path';
import { Entry, Manifest, ObservationBinding } from './emitter';
import { emptyManifest, subject } from './observations';

interface SarifResult {
  ruleId?: string;
  level?: string;
  message?: { text?: string };
  locations?: Array<{ physicalLocation?: {
    artifactLocation?: { uri?: string };
    region?: { startLine?: number };
  } }>;
}

interface SarifRun {
  tool: { driver: { name: string; version?: string } };
  artifacts?: Array<{ location?: { uri?: string } }>;
  results?: SarifResult[];
}

interface SarifLog {
  version: string;
  runs: SarifRun[];
}

export interface SarifImport {
  report: unknown;
  reportPath: string;
  reportSource: string;
  linkage: Manifest;
  root: string;
  inputs: Array<{ path: string; source: string }>;
}

export function importSarif(input: SarifImport): Manifest {
  const log = readLog(input.report);
  const targetedFiles = new Set<string>();
  const findings: Array<{ rule: string; level: string; message: string; file: string; line: number }> = [];
  for (const run of log.runs) {
    for (const artifact of run.artifacts ?? []) {
      if (artifact.location?.uri) targetedFiles.add(relative(input.root, artifact.location.uri));
    }
    for (const result of run.results ?? []) {
      for (const location of result.locations ?? []) {
        const uri = location.physicalLocation?.artifactLocation?.uri;
        if (!uri) continue;
        const file = relative(input.root, uri);
        targetedFiles.add(file);
        findings.push({
          rule: result.ruleId ?? 'unknown',
          level: result.level ?? 'warning',
          message: result.message?.text ?? '',
          file,
          line: location.physicalLocation?.region?.startLine ?? 0,
        });
      }
    }
  }
  if (targetedFiles.size === 0) {
    throw new Error('SARIF report names no analyzed artifacts');
  }

  const bindings: ObservationBinding[] = [];
  for (const claim of claims(input.linkage.realizes.filter((site) => targetedFiles.has(site.file)))) {
    const sites = input.linkage.realizes.filter(
      (site) => site.spec === claim.spec
        && site.scenario === claim.scenario
        && targetedFiles.has(site.file),
    );
    const claimFindings = findings.filter((finding) => sites.some((site) => site.file === finding.file));
    bindings.push({
      role: 'challenge',
      spec: claim.spec,
      scenario: claim.scenario,
      assertion: 'static analysis reports adverse results at linked realization sites',
      outcome: claimFindings.length === 0 ? 'clean' : 'findings',
      subjects: sites.map((site) => ({ relation: 'realization', identity: subject(site) })),
    });
  }
  if (bindings.length === 0) {
    throw new Error('SARIF targets intersect no Realizes sites');
  }
  bindings.sort((left, right) =>
    left.spec.localeCompare(right.spec) || left.scenario.localeCompare(right.scenario),
  );
  const driver = log.runs[0].tool.driver;
  const fingerprint = createHash('sha256').update(input.reportSource);
  for (const item of input.inputs) fingerprint.update('\0').update(item.source);
  return emptyManifest([{
    id: `sarif-${createHash('sha256').update(input.reportPath).digest('hex').slice(0, 16)}`,
    kind: 'static-analysis',
    tool: driver.name,
    tool_version: driver.version ?? 'unknown',
    report: input.reportPath,
    inputs: input.inputs.map((item) => item.path),
    source_fingerprint: fingerprint.digest('hex'),
    bindings,
    payload: { schema: 'azimuth-sarif-summary/1', findings },
  }]);
}

function readLog(value: unknown): SarifLog {
  if (!record(value) || value.version !== '2.1.0' || !Array.isArray(value.runs)) {
    throw new Error('SARIF report must use version 2.1.0 and contain runs');
  }
  for (const [index, run] of value.runs.entries()) {
    if (!record(run) || !record(run.tool) || !record(run.tool.driver)
        || typeof run.tool.driver.name !== 'string') {
      throw new Error(`SARIF run ${index} has no tool driver`);
    }
  }
  if (value.runs.length === 0) throw new Error('SARIF report contains no runs');
  return value as unknown as SarifLog;
}

function claims(sites: Entry[]): Array<{ spec: string; scenario: string }> {
  const found = new Map<string, { spec: string; scenario: string }>();
  for (const site of sites) found.set(`${site.spec}\0${site.scenario}`, site);
  return [...found.values()];
}

function relative(root: string, uri: string): string {
  const value = uri.startsWith('file://') ? new URL(uri).pathname : uri;
  return path.relative(path.resolve(root), path.resolve(root, value)).split(path.sep).join('/');
}

function record(value: unknown): value is Record<string, any> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
