/**
 * Provider-neutral import boundary for external manual-test results.
 *
 * A TestRail, Qase, Zephyr or similar adapter only has to project its export into this small shape.
 * The result remains attributable to the external run; the importer does not claim that a charter
 * ran merely because a case exists.
 */
import { createHash } from 'node:crypto';
import { Manifest } from './emitter';

interface ManualResult {
  case_id: string;
  spec: string;
  scenario: string;
  status: 'passed' | 'failed';
  scope: 'unit' | 'component' | 'e2e';
  quantification: 'example' | 'universal';
  url: string;
}

interface ManualRun {
  provider: string;
  run_id: string;
  observed_at: string;
  expires_at: string;
  results: ManualResult[];
}

export function importManualResults(value: unknown): Manifest {
  const run = readRun(value);
  const expiresAt = Date.parse(run.expires_at);
  if (!Number.isFinite(expiresAt)) throw new Error('expires_at must be an ISO-8601 instant');
  if (!Number.isFinite(Date.parse(run.observed_at))) {
    throw new Error('observed_at must be an ISO-8601 instant');
  }

  return {
    realizes: [],
    covers: run.results.map((result) => ({
      spec: result.spec,
      scenario: result.scenario,
      site: `${run.provider}:${result.case_id}`,
      file: result.url,
      lang: 'external',
      source_fingerprint: createHash('sha256')
        .update(JSON.stringify({
          provider: run.provider,
          run_id: run.run_id,
          observed_at: run.observed_at,
          expires_at: run.expires_at,
          result,
        }))
        .digest('hex'),
      evidence_kind: 'manual-test',
      evidence_outcome: result.status,
      observed_at: run.observed_at,
      expires_at: Math.floor(expiresAt / 1000),
      scope: result.scope,
      quantification: result.quantification,
    })),
    mechanism_implementations: [],
    mechanism_covers: [],
    class_members: [],
    enumerations: [],
    artifacts: [],
  };
}

function readRun(value: unknown): ManualRun {
  if (!isRecord(value)) throw new Error('manual result export must be an object');
  const provider = requiredString(value, 'provider');
  const run_id = requiredString(value, 'run_id');
  const observed_at = requiredString(value, 'observed_at');
  const expires_at = requiredString(value, 'expires_at');
  if (!Array.isArray(value.results)) throw new Error('results must be an array');
  const results = value.results.map((result, index) => readResult(result, index));
  return { provider, run_id, observed_at, expires_at, results };
}

function readResult(value: unknown, index: number): ManualResult {
  if (!isRecord(value)) throw new Error(`results[${index}] must be an object`);
  const status = requiredString(value, 'status');
  if (status !== 'passed' && status !== 'failed') {
    throw new Error(`results[${index}].status must be passed or failed`);
  }
  const scope = requiredString(value, 'scope');
  if (scope !== 'unit' && scope !== 'component' && scope !== 'e2e') {
    throw new Error(`results[${index}].scope must be unit, component or e2e`);
  }
  const quantification = requiredString(value, 'quantification');
  if (quantification !== 'example' && quantification !== 'universal') {
    throw new Error(`results[${index}].quantification must be example or universal`);
  }
  return {
    case_id: requiredString(value, 'case_id'),
    spec: requiredString(value, 'spec'),
    scenario: requiredString(value, 'scenario'),
    status,
    scope,
    quantification,
    url: requiredString(value, 'url'),
  };
}

function requiredString(value: Record<string, unknown>, key: string): string {
  const field = value[key];
  if (typeof field !== 'string' || field.length === 0) throw new Error(`${key} must be a string`);
  return field;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
