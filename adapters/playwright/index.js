#!/usr/bin/env node
'use strict';

const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawn, execFileSync } = require('node:child_process');
const { createRequire } = require('node:module');
const manifest = require('./package.json');

// Standard output belongs exclusively to the adapter report, including when a config logs.
const writeReport = process.stdout.write.bind(process.stdout);
process.stdout.write = process.stderr.write.bind(process.stderr);
let temporary;
process.on('exit', () => {
  if (temporary) fs.rmSync(temporary, { recursive: true, force: true });
});
// Every process in the launch group already receives the interrupt from the core.
process.on('SIGINT', () => process.exit(1));

function installedIn(folder) {
  const ownFolder = fs.realpathSync(__dirname);
  for (let current = fs.realpathSync(folder); ; current = path.dirname(current)) {
    if (ownFolder === path.join(current, 'node_modules', '@azimuth-sh', 'playwright')) return;
    if (path.dirname(current) === current) break;
  }
  throw new Error('the package is not installed in this application');
}

function discover(folder, root) {
  const projects = [];
  function visit(current) {
    const entries = fs.readdirSync(current, { withFileTypes: true });
    if (entries.some(entry => entry.isFile() && /^playwright\.config\.(?:[cm]?[jt]s)$/.test(entry.name))) {
      projects.push({ path: path.relative(root, current).split(path.sep).join('/') || '.' });
    }
    for (const entry of entries) {
      if (entry.isDirectory() && !entry.name.startsWith('.') && entry.name !== 'node_modules') {
        visit(path.join(current, entry.name));
      }
    }
  }
  visit(folder);
  projects.sort((a, b) => a.path.localeCompare(b.path));
  return { projects };
}

function specs(document) {
  const entries = new Map();
  function visit(suite, parents) {
    const titles = [...parents, suite.title];
    for (const spec of suite.specs || []) {
      const name = [...titles, spec.title].join(' › ');
      const identity = JSON.stringify([spec.file, spec.line, spec.column, name]);
      entries.set(identity, { ...spec, name });
    }
    for (const child of suite.suites || []) visit(child, titles);
  }
  for (const suite of document.suites || []) visit(suite, []);
  return entries;
}

function outcome(rows, expectedCount) {
  const results = rows.map(row => row.results[0]);
  if (results.some(result => result && ['failed', 'timedOut'].includes(result.status))) return 'failed';
  if (results.length !== expectedCount || results.some(result => !result || result.status === 'interrupted')) return 'not_run';
  return results.every(result => result.status === 'passed') ? 'passed' : 'skipped';
}

function escapeRegex(text) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

async function run(request) {
  const application = createRequire(path.join(process.cwd(), 'package.json'));
  const runner = createRequire(application.resolve('@playwright/test/package.json'));
  // The public FullConfig loses the individual entries when webServer is an array.
  const { configLoader } = runner('playwright/lib/common');
  const args = process.argv.slice(2);
  const configIndex = args.findIndex(arg => arg === '--config' || arg === '-c');
  const configPath = configIndex >= 0 ? args[configIndex + 1] : args.find(arg => arg.startsWith('--config='))?.slice(9);
  let config;
  try {
    config = await configLoader.loadConfigFromFile(configPath, { retries: 0 });
  } catch (error) {
    console.error(error);
    return { build: 'failed', tests: [] };
  }
  if (!config.webServers.length) throw new Error('the resolved config has no webServer entry');
  if (config.webServers.some(server => server.reuseExistingServer)) {
    throw new Error('the resolved config sets reuseExistingServer to true');
  }

  temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-playwright-'));
  const output = path.join(temporary, 'report.json');
  const cli = application.resolve('@playwright/test/cli');
  async function launch(extra) {
    fs.rmSync(output, { force: true });
    const status = await new Promise((resolve, reject) => {
      const child = spawn(process.execPath, [cli, 'test', ...args, ...extra, '--retries=0', '--reporter=line,json'], {
        stdio: ['ignore', 2, 2],
        env: { ...process.env, PLAYWRIGHT_JSON_OUTPUT_NAME: output },
      });
      child.on('error', reject);
      child.on('exit', (code, signal) => signal ? reject(new Error(`the runner ended with ${signal}`)) : resolve(code));
    });
    const report = fs.existsSync(output) ? JSON.parse(fs.readFileSync(output, 'utf8')) : null;
    return { status, report };
  }

  const listed = await launch(['--list']);
  if (listed.status !== 0 || !listed.report) return { build: 'failed', tests: [] };
  const marked = [...specs(listed.report)].map(([identity, spec]) => {
    const prefix = `${request.marker_key}:`;
    const checks = [...new Set(spec.tags.filter(tag => tag.startsWith(prefix)).map(tag => tag.slice(prefix.length)))];
    return { identity, spec, checks };
  }).filter(test => test.checks.length);
  const asked = new Set(request.checks);
  const selected = [...new Set(marked.flatMap(test => test.checks).filter(check => asked.has(check)))];
  let ran = new Map();
  if (selected.length) {
    const filter = `(?:^|\\s)@${escapeRegex(request.marker_key)}:(?:${selected.map(escapeRegex).join('|')})(?=\\s|$)`;
    const executed = await launch(['--grep', filter]);
    if (!executed.report) throw new Error('the runner wrote no report');
    ran = specs(executed.report);
  }
  return {
    build: 'ok',
    tests: marked.map(({ identity, spec, checks }) => {
      const rows = ran.get(identity)?.tests || [];
      return {
        name: spec.name,
        checks,
        outcome: rows.length ? outcome(rows, spec.tests.length) : 'not_run',
        duration_ms: rows.reduce((sum, row) => sum + (row.results[0]?.duration || 0), 0),
      };
    }),
  };
}

async function main() {
  let input = '';
  for await (const chunk of process.stdin) input += chunk;
  const request = JSON.parse(input);
  if (request.format !== 'adapter-request-1') throw new Error('the request is not an adapter-request-1');
  const root = execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
  const folder = path.resolve(root, request.verb === 'discover' ? request.folder : request.suite);
  installedIn(folder);
  let report;
  if (request.verb === 'discover') report = discover(folder, root);
  else if (request.verb === 'run') report = await run(request);
  else throw new Error(`the verb ${request.verb} is not known; this adapter knows discover and run`);
  writeReport(JSON.stringify({ format: 'adapter-report-1', adapter: { name: manifest.name, version: manifest.version }, ...report }));
}

main().catch(error => {
  console.error(`azimuth-playwright: ${error.message}`);
  process.exitCode = 1;
});
