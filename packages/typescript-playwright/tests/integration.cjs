const assert = require('node:assert/strict');
const http = require('node:http');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const { spawn } = require('node:child_process');

(async () => {
  let changing = 0;
  const server = http.createServer((req, res) => {
    res.setHeader('Content-Type', 'application/json');
    if (req.url === '/unavailable') { res.writeHead(503); res.end('{}'); return; }
    const contexts = req.url === '/missing' ? {} : { settings: {
      contract_revision: `sha256:${'a'.repeat(64)}`,
      values: { enabled: true, revision: req.url === '/changing' ? changing++ : 1 }
    } };
    res.end(JSON.stringify({ format: 'azimuth-execution-context', version: 2, contexts }));
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'azimuth-playwright-'));
  const url = `http://127.0.0.1:${server.address().port}`;
  const root = path.resolve(__dirname, '..');
  const reportPath = path.join(directory, 'report.json');
  try {
    await fs.writeFile(path.join(directory, 'playwright.config.cjs'), `module.exports = {
      testDir: '.', workers: 2, retries: 1, timeout: 1000,
      outputDir: ${JSON.stringify(path.join(directory, 'artifacts'))},
      reporter: [[${JSON.stringify(path.join(root, 'dist/reporter.js'))}, {outputFile: ${JSON.stringify(reportPath)}, execution: 'synthetic-http'}]],
      use: {azimuthContexts: [{name: 'settings', url: ${JSON.stringify(url + '/stable')}, instance: 'synthetic-app'}]}
    };`);
    await fs.writeFile(path.join(directory, 'capture.spec.cjs'), `
      const { test, expect } = require(${JSON.stringify(path.join(root, 'dist/index.js'))});
      test('passes', async () => { expect(2 + 2).toBe(4); });
      test('fails', async () => { expect(2 + 2).toBe(5); });
      test('times out', async () => { await new Promise(() => {}); });
      test.skip('skipped', async () => {});
      test('retry succeeds', async ({}, info) => { expect(info.retry).toBe(1); });
      test.describe('changing', () => {
        test.use({azimuthContexts: [{name: 'settings', url: ${JSON.stringify(url + '/changing')}, instance: 'synthetic-app'}]});
        test('context changes', async () => {});
      });
      test.describe('missing', () => {
        test.use({azimuthContexts: [{name: 'settings', url: ${JSON.stringify(url + '/missing')}, instance: 'synthetic-app'}]});
        test('context absent', async () => {});
      });
      test.describe('unavailable', () => {
        test.use({azimuthContexts: [{name: 'settings', url: ${JSON.stringify(url + '/unavailable')}, instance: 'synthetic-app'}]});
        test('endpoint unavailable', async () => {});
      });
      test.describe('none', () => {
        test.use({azimuthContexts: [], azimuthTestId: 'explicit-unit-id'});
        test('no context', async () => {});
      });
    `);
    const code = await new Promise((resolve, reject) => {
      const child = spawn(process.execPath, [require.resolve('@playwright/test/cli'), 'test', '--config', path.join(directory, 'playwright.config.cjs')], { cwd: root, stdio: 'pipe' });
      let output = '';
      child.stdout.on('data', chunk => output += chunk);
      child.stderr.on('data', chunk => output += chunk);
      child.on('error', reject);
      child.on('exit', code => { if (code !== 1) reject(new Error(`Expected deliberate assertion failure, got ${code}: ${output}`)); else resolve(code); });
    });
    assert.equal(code, 1);
    const report = JSON.parse(await fs.readFile(reportPath));
    assert.equal(report.format, 'azimuth-test-report');
    assert.equal(report.tests.length, 12);
    assert.equal(report.tests.filter(t => t.outcome === 'timed-out').length, 2);
    assert.equal(report.tests.filter(t => t.outcome === 'failed').length, 3);
    assert.equal(report.tests.filter(t => t.outcome === 'skipped').length, 1);
    assert(report.tests.some(t => t.attempt === 1 && t.outcome === 'passed'));
    const changed = report.tests.find(t => t.diagnostics.some(d => d.includes('changed during')));
    assert(changed);
    assert.notDeepEqual(changed.contexts[0].before.report, changed.contexts[0].after.report);
    assert.equal(report.tests.filter(t => t.diagnostics.some(d => d.includes('before capture unavailable'))).length, 2);
    assert(report.tests.some(t => t.id === 'explicit-unit-id' && t.contexts.length === 0 && t.diagnostics.length === 0));
    for (const test of report.tests) {
      assert(test.finished_at_ms >= test.started_at_ms);
      for (const context of test.contexts) {
        assert.equal(context.before.instance, context.after.instance);
        assert.equal(context.before.instance, 'synthetic-app');
      }
    }
    assert.deepEqual(report.tests.map(t => [t.id, t.attempt]), [...report.tests].sort((a,b) => a.id.localeCompare(b.id) || a.attempt-b.attempt).map(t => [t.id,t.attempt]));
    console.log('Real Playwright/HTTP integration passed: parallel workers, retries, failures, skips, context change and unavailable context retained.');
  } finally {
    server.closeAllConnections();
    await new Promise(resolve => server.close(resolve));
    await fs.rm(directory, { recursive: true, force: true });
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
