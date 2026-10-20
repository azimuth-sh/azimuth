import { test as base, expect } from '@playwright/test';
import { attachmentName, contextIdentity, validateContextReport, type ContextSnapshot, type ContextCapture } from './protocol';
export type * from './protocol';

export interface ContextEndpoint {
  name: string;
  url: string;
  instance: string;
  contractRevision?: string;
  timeoutMs?: number;
}
export interface AzimuthOptions {
  azimuthContexts: ContextEndpoint[];
  azimuthTestId: string | undefined;
}

async function capture(endpoint: ContextEndpoint): Promise<ContextSnapshot> {
  if (!endpoint.name || !endpoint.instance || !['http:', 'https:'].includes(new URL(endpoint.url).protocol)) {
    throw new Error('Invalid context endpoint configuration');
  }
  const response = await fetch(endpoint.url, { signal: AbortSignal.timeout(endpoint.timeoutMs ?? 5000), redirect: 'error' });
  if (!response.ok) throw new Error('Context endpoint returned an unsuccessful response');
  const report: unknown = await response.json();
  validateContextReport(report);
  const context = report.contexts[endpoint.name];
  if (!context) throw new Error('Required context is missing');
  if (endpoint.contractRevision && context.contract_revision !== endpoint.contractRevision) {
    throw new Error('Context contract revision differs');
  }
  // Retaining only the requested context avoids recording unrelated application settings.
  return { captured_at_ms: Date.now(), instance: endpoint.instance,
    report: { format: report.format, version: report.version, contexts: { [endpoint.name]: context } } };
}

export const test = base.extend<AzimuthOptions & { azimuthCapture: void }>({
  azimuthContexts: [[], { option: true }],
  azimuthTestId: [undefined, { option: true }],
  azimuthCapture: [async ({ azimuthContexts, azimuthTestId }, use, testInfo) => {
    const diagnostics: string[] = [];
    const before = new Map<string, ContextSnapshot>();
    const contexts: ContextCapture[] = [];
    const names = new Set<string>();
    for (const endpoint of azimuthContexts) {
      if (names.has(endpoint.name)) throw new Error('Duplicate verification context name');
      names.add(endpoint.name);
      try { before.set(endpoint.name, await capture(endpoint)); }
      catch { diagnostics.push(`Context ${endpoint.name}: before capture unavailable`); }
    }
    try { await use(); }
    finally {
      for (const endpoint of azimuthContexts) {
        try {
          const after = await capture(endpoint);
          const initial = before.get(endpoint.name);
          if (!initial) continue;
          contexts.push({ name: endpoint.name, before: initial, after });
          if (contextIdentity(initial.report.contexts[endpoint.name]) !== contextIdentity(after.report.contexts[endpoint.name])) {
            diagnostics.push(`Context ${endpoint.name}: changed during execution`);
          }
        } catch { diagnostics.push(`Context ${endpoint.name}: after capture unavailable`); }
      }
      await testInfo.attach(attachmentName, { contentType: 'application/json',
        body: Buffer.from(JSON.stringify({ id: azimuthTestId ?? testInfo.testId, contexts, diagnostics })) });
    }
  }, { auto: true, timeout: 15000 }],
});
export { expect };
