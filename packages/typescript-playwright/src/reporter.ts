import { randomUUID } from 'node:crypto';
import { mkdir, readFile, rename, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import type { FullConfig, FullResult, Reporter, Suite, TestCase, TestResult } from '@playwright/test/reporter';
import { attachmentName, type TestFacts, type TestReport } from './protocol';

export interface ReporterOptions {
  outputFile?: string;
  execution?: string;
}
export default class AzimuthReporter implements Reporter {
  private started = Date.now();
  private readonly tests: TestFacts[] = [];
  private readonly output: string;
  private readonly execution: string;
  constructor(options: ReporterOptions = {}) {
    this.output = resolve(options.outputFile ?? 'test-results/azimuth-report.json');
    this.execution = options.execution ?? randomUUID();
  }
  onBegin(_config: FullConfig, _suite: Suite): void { this.started = Date.now(); }
  async onTestEnd(test: TestCase, result: TestResult): Promise<void> {
    const captures = result.attachments.filter(item => item.name === attachmentName);
    let facts: Pick<TestFacts, 'id' | 'contexts' | 'diagnostics'> = { id: test.id, contexts: [], diagnostics: [] };
    if (captures.length === 1) {
      const attachment = captures[0];
      try {
        const bytes = attachment.body ?? (attachment.path ? await readFile(attachment.path) : undefined);
        if (!bytes) throw new Error('Missing attachment body');
        facts = JSON.parse(bytes.toString());
      } catch { facts.diagnostics.push('Azimuth capture attachment could not be read'); }
    } else if (result.status !== 'skipped') {
      facts.diagnostics.push('Azimuth capture fixture did not produce exactly one attachment');
    }
    const started = result.startTime.getTime();
    this.tests.push({ ...facts, attempt: result.retry,
      outcome: result.status === 'timedOut' ? 'timed-out' : result.status,
      started_at_ms: started, finished_at_ms: started + Math.max(0, Math.ceil(result.duration)) });
  }
  async onEnd(_result: FullResult): Promise<void> {
    const report: TestReport = { format: 'azimuth-test-report', version: 1, framework: 'playwright',
      execution: this.execution, started_at_ms: this.started, finished_at_ms: Date.now(),
      tests: this.tests.sort((a, b) => a.id.localeCompare(b.id) || a.attempt - b.attempt) };
    await mkdir(dirname(this.output), { recursive: true });
    const temporary = `${this.output}.${process.pid}.tmp`;
    await writeFile(temporary, `${JSON.stringify(report, null, 2)}\n`, { flag: 'wx' });
    await rename(temporary, this.output);
  }
}
