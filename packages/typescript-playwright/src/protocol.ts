export type ContextValue = string | boolean | number;
export interface VerificationContext {
  contract_revision: string;
  values: Record<string, ContextValue>;
}
export interface VerificationContextReport {
  format: 'azimuth-verification-context';
  version: 1;
  contexts: Record<string, VerificationContext>;
}
export interface ContextSnapshot {
  captured_at_ms: number;
  instance: string;
  report: VerificationContextReport;
}
export interface ContextCapture {
  name: string;
  before: ContextSnapshot;
  after: ContextSnapshot;
}
export interface TestFacts {
  id: string;
  attempt: number;
  outcome: 'passed' | 'failed' | 'skipped' | 'timed-out' | 'interrupted';
  started_at_ms: number;
  finished_at_ms: number;
  contexts: ContextCapture[];
  diagnostics: string[];
}
export interface TestReport {
  format: 'azimuth-test-report';
  version: 1;
  framework: 'playwright';
  execution: string;
  started_at_ms: number;
  finished_at_ms: number;
  tests: TestFacts[];
}
export const attachmentName = 'azimuth-execution-facts-v1';

function object(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}
function keys(value: Record<string, unknown>, expected: string[]): boolean {
  return Object.keys(value).sort().join(',') === expected.sort().join(',');
}
export function validateContextReport(value: unknown): asserts value is VerificationContextReport {
  if (!object(value) || !keys(value, ['format', 'version', 'contexts']) ||
      value.format !== 'azimuth-verification-context' || value.version !== 1 || !object(value.contexts)) {
    throw new Error('Unsupported verification context envelope');
  }
  for (const [name, context] of Object.entries(value.contexts)) {
    if (!name || !object(context) || !keys(context, ['contract_revision', 'values']) ||
        typeof context.contract_revision !== 'string' || !/^sha256:[a-f0-9]{64}$/.test(context.contract_revision) ||
        !object(context.values)) {
      throw new Error('Invalid verification context declaration');
    }
    for (const field of Object.values(context.values)) {
      if (typeof field !== 'string' && typeof field !== 'boolean' &&
          !(typeof field === 'number' && Number.isSafeInteger(field) && field >= 0)) {
        throw new Error('Unsupported verification context value');
      }
    }
  }
}
export function contextIdentity(context: VerificationContext): string {
  return JSON.stringify([context.contract_revision,
    Object.entries(context.values).sort(([a], [b]) => a.localeCompare(b))]);
}
