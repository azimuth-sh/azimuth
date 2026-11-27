#!/usr/bin/env node
'use strict';

const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawn, spawnSync, execFileSync } = require('node:child_process');
const { createRequire } = require('node:module');
const manifest = require('./package.json');
const { addressOf, answers } = require('./components');

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
  const bounds = /^>=(\d+)\.(\d+)\.(\d+) <(\d+)\.(\d+)\.(\d+)$/.exec(supported);
  const installed = /^(\d+)\.(\d+)\.(\d+)$/.exec(version);
  const compare = (left, right) => {
    for (let index = 0; index < 3; index++) {
      const difference = Number(left[index]) - Number(right[index]);
      if (difference) return difference;
    }
    return 0;
  };
  if (!bounds || !installed || compare(installed.slice(1), bounds.slice(1, 4)) < 0 || compare(installed.slice(1), bounds.slice(4)) >= 0) {
    throw new Error(`installed Playwright ${version} is outside the supported range ${supported}`);
  }
  return application;
}

const NO_ADDRESS = 'no address in the runner configuration';
const ANSWERS = 'something else answers there';
const NOT_MADE = { start: 'the start read was not made', stop: 'the stop read was not made' };

function evaluateConfig(configPath, folder, index) {
  const output = path.join(folder, `evaluation-${index}.json`);
  const status = spawnSync(process.execPath, [path.join(__dirname, 'evaluate-config.js'), output, configPath || ''], { stdio: ['ignore', 2, 2] });
  if (status.status !== 0) return undefined;
  return JSON.parse(fs.readFileSync(output, 'utf8'));
}

// What the config says about each component the request lists: one address, none, or a refusal.
function addressesIn(evaluation, components) {
  return components.map(component => {
    const entries = evaluation.webServers.filter(server => server.name === component.name);
    if (entries.length > 1) throw new Error(`the config has ${entries.length} webServer entries named "${component.name}", and a component's entry must be one`);
    return entries.length ? addressOf(entries[0]) : undefined;
  });
}

function readsOf(folder, phase) {
  const file = path.join(folder, `${phase}.json`);
  return fs.existsSync(file) ? new Map(JSON.parse(fs.readFileSync(file, 'utf8')).map(({ name, read }) => [name, read])) : new Map();
}

async function run(request, application) {
  const components = request.components || [];
  const args = process.argv.slice(2);
  const configIndex = args.findIndex(arg => arg === '--config' || arg === '-c');
  const configPath = configIndex >= 0 ? args[configIndex + 1] : args.find(arg => arg.startsWith('--config='))?.slice(9);
  temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-playwright-'));

  const evaluation = evaluateConfig(configPath, temporary, 1);
  if (!evaluation) return { build: 'failed', tests: [] };
  if (!evaluation.webServers.length) throw new Error('the resolved config has no webServer entry');
  if (evaluation.webServers.some(server => server.reuseExistingServer)) {
    throw new Error('the resolved config sets reuseExistingServer to true');
  }
  const addresses = addressesIn(evaluation, components);
  const second = evaluateConfig(configPath, temporary, 2);
  if (!second) throw new Error('the resolved config could not be evaluated a second time');
  const later = addressesIn(second, components);
  components.forEach((component, index) => {
    if (addresses[index] !== later[index]) {
      throw new Error(`the address of "${component.name}" is ${addresses[index] ?? 'missing'} at one evaluation of the config and ${later[index] ?? 'missing'} at the next`);
    }
  });

  const looked = await Promise.all(addresses.map(address => address && answers(address)));
  const occupied = looked.some(Boolean);

  const output = path.join(temporary, 'report.json');
  const cli = application.resolve('@playwright/test/cli');
  async function launch(extra, wrap) {
    fs.rmSync(output, { force: true });
    const loader = wrap ? ['--require', path.join(__dirname, 'wrap-config.js')] : [];
    const { status, signal } = await new Promise((resolve, reject) => {
      const child = spawn(process.execPath, [...loader, cli, 'test', ...args, ...extra, '--retries=0', '--reporter=line,json'], {
        stdio: ['ignore', 2, 2],
        env: { ...process.env, ...(wrap ? { AZIMUTH_PLAYWRIGHT_PLAN: wrap } : {}), PLAYWRIGHT_JSON_OUTPUT_FILE: output, PLAYWRIGHT_JSON_OUTPUT_NAME: output, PLAYWRIGHT_JSON_ONLY_FAILURES: '0' },
      });
      child.on('error', reject);
      child.on('exit', (code, signal) => resolve({ status: code, signal }));
    });
    const report = fs.existsSync(output) ? JSON.parse(fs.readFileSync(output, 'utf8')) : null;
    return { status, signal, report };
  }

  const listed = await launch(['--list']);
  if (listed.signal) throw new Error(`the runner ended with ${listed.signal}`);
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
  let started = false;
  const readable = components.filter((component, index) => addresses[index]);
  if (selected.length && !occupied) {
    const filter = `(?:^|\\s)@${escapeRegex(request.marker_key)}:(?:${selected.map(escapeRegex).join('|')})(?=\\s|$)`;
    let plan;
    if (readable.length) {
      const file = path.join(temporary, 'plan.json');
      fs.writeFileSync(file, JSON.stringify({
        config: fs.realpathSync(evaluation.configFile),
        folder: temporary,
        components: readable.map(component => ({ name: component.name, address: addresses[components.indexOf(component)], report: component.report })),
      }));
      plan = file;
    }
    const executed = await launch(['--grep', filter], plan);
    started = true;
    // A runner that is killed writes no report, and nothing it reports is then a failure of the suite.
    if (!executed.signal) {
      ran = specs(executed.report);
      if (executed.report.errors.length || (executed.status !== 0 && ![...ran.values()].some(spec => outcome(spec.tests, spec.tests.length) === 'failed'))) {
        throw new Error('the runner failed outside a test');
      }
    }
    if (plan && !executed.signal && !fs.existsSync(path.join(temporary, 'wrapped'))) {
      throw new Error('the global setup of this config could not be wrapped; a config that the runner loads as an ES module is not supported');
    }
  }
  const starts = readsOf(temporary, 'start');
  const stops = readsOf(temporary, 'stop');
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
    components: components.map((component, index) => {
      if (!addresses[index]) return { name: component.name, error: NO_ADDRESS, reads: [] };
      if (looked[index]) return { name: component.name, error: ANSWERS, reads: [] };
      if (!started) return { name: component.name, reads: [] };
      return {
        name: component.name,
        reads: ['start', 'stop'].map(phase => (phase === 'start' ? starts : stops).get(component.name) || { phase, error: NOT_MADE[phase] }),
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
