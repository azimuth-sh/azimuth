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
    const installation = path.join(current, 'node_modules', '@azimuth-sh', 'playwright');
    if (fs.existsSync(installation) && ownFolder === fs.realpathSync(installation)) return;
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
      if (entry.isDirectory() && entry.name !== '.git' && entry.name !== 'node_modules') {
        visit(path.join(current, entry.name));
      }
    }
  }
  visit(folder);
  projects.sort((a, b) => a.path.localeCompare(b.path));
  return { projects };
}

function specs(document) {
  if (!document || !Array.isArray(document.suites) || !Array.isArray(document.errors)) {
    throw new Error('the runner wrote a report without suites and errors');
  }
  const entries = new Map();
  function visit(suite, parents) {
    if (typeof suite.title !== 'string' || !Array.isArray(suite.specs)) throw new Error('the runner wrote an invalid suite');
    const titles = [...parents, suite.title];
    for (const spec of suite.specs) {
      if (typeof spec.title !== 'string' || typeof spec.file !== 'string' || !Number.isInteger(spec.line) || !Number.isInteger(spec.column) || !Array.isArray(spec.tags)
          || !spec.tags.every(tag => typeof tag === 'string') || !Array.isArray(spec.tests) || !spec.tests.length) {
        throw new Error('the runner wrote an invalid test listing');
      }
      for (const test of spec.tests) {
        if (!Array.isArray(test.results) || typeof test.expectedStatus !== 'string') throw new Error('the runner wrote an invalid test result');
        for (const result of test.results) {
          if (!['passed', 'failed', 'timedOut', 'skipped', 'interrupted'].includes(result.status)
              || !Number.isInteger(result.duration) || result.duration < 0) throw new Error('the runner wrote an invalid attempt');
        }
      }
      const name = [...titles, spec.title].join(' › ');
      const identity = JSON.stringify([spec.file, spec.line, spec.column, name]);
      entries.set(identity, { ...spec, name });
    }
    for (const child of suite.suites || []) visit(child, titles);
  }
  for (const suite of document.suites) visit(suite, []);
  return entries;
}

function outcome(rows, expectedCount) {
  const verdicts = rows.map(row => {
    const result = row.results[0];
    if (!result || result.status === 'interrupted') return 'not_run';
    if (result.status === 'skipped') return 'skipped';
    return result.status === row.expectedStatus ? 'passed' : 'failed';
  });
  if (verdicts.includes('failed')) return 'failed';
  if (rows.length !== expectedCount || verdicts.includes('not_run')) return 'not_run';
  return verdicts.every(verdict => verdict === 'passed') ? 'passed' : 'skipped';
}

function escapeRegex(text) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function applicationRunner() {
  const application = createRequire(path.join(process.cwd(), 'package.json'));
  const version = application('@playwright/test/package.json').version;
  const supported = manifest.peerDependencies['@playwright/test'];
  if (version !== supported) throw new Error(`installed Playwright ${version} is outside the supported range ${supported}`);
  return application;
}

async function run(request, application) {
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
        env: { ...process.env, PLAYWRIGHT_JSON_OUTPUT_FILE: output, PLAYWRIGHT_JSON_OUTPUT_NAME: output, PLAYWRIGHT_JSON_ONLY_FAILURES: '0' },
      });
      child.on('error', reject);
      child.on('exit', (code, signal) => signal ? reject(new Error(`the runner ended with ${signal}`)) : resolve(code));
    });
    const report = fs.existsSync(output) ? JSON.parse(fs.readFileSync(output, 'utf8')) : null;
    return { status, report };
  }

  const listed = await launch(['--list']);
  const listing = specs(listed.report);
  if (listed.status !== 0) return { build: 'failed', tests: [] };
  const marked = [...listing].map(([identity, spec]) => {
    const prefix = `${request.marker_key}:`;
    const checks = [...new Set(spec.tags.filter(tag => tag.startsWith(prefix) && /^[A-Za-z0-9-]+$/.test(tag.slice(prefix.length))).map(tag => tag.slice(prefix.length)))];
    return { identity, spec, checks };
  }).filter(test => test.checks.length);
  const asked = new Set(request.checks);
  const selected = [...new Set(marked.flatMap(test => test.checks).filter(check => asked.has(check)))];
  let ran = new Map();
  if (selected.length) {
    const filter = `(?:^|\\s)@${escapeRegex(request.marker_key)}:(?:${selected.map(escapeRegex).join('|')})(?=\\s|$)`;
    const executed = await launch(['--grep', filter]);
    ran = specs(executed.report);
    if (executed.report.errors.length || (executed.status !== 0 && ![...ran.values()].some(spec => outcome(spec.tests, spec.tests.length) === 'failed'))) {
      throw new Error('the runner failed outside a test');
    }
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
  process.stdin.setEncoding('utf8');
  for await (const chunk of process.stdin) input += chunk;
  const request = JSON.parse(input);
  if (request.format !== 'adapter-request-1') throw new Error('the request is not an adapter-request-1');
  const root = execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
  const folder = path.resolve(root, request.verb === 'discover' ? request.folder : request.suite);
  if (fs.realpathSync(folder) !== fs.realpathSync(process.cwd())) throw new Error('the request folder is not the folder where the adapter was started');
  installedIn(folder);
  const application = applicationRunner();
  let report;
  if (request.verb === 'discover') report = discover(folder, root);
  else if (request.verb === 'run') report = await run(request, application);
  else throw new Error(`the verb ${request.verb} is not known; this adapter knows discover and run`);
  writeReport(JSON.stringify({ format: 'adapter-report-1', adapter: { name: manifest.name, version: manifest.version }, ...report }));
}

main().catch(error => {
  console.error(`azimuth-playwright: ${error.message}`);
  process.exitCode = 1;
});
