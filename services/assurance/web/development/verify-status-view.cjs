const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const Module = require('node:module');
const ts = require('typescript');
const React = require('react');
const { renderToStaticMarkup } = require('react-dom/server');

const filename = path.resolve(__dirname, '../src/components/requirement-verification.tsx');
const source = ts.transpileModule(fs.readFileSync(filename, 'utf8'), {
  compilerOptions: { module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.ReactJSX, target: ts.ScriptTarget.ES2022, esModuleInterop: true },
}).outputText;
const componentModule = new Module(filename, module);
componentModule.filename = filename;
componentModule.paths = Module._nodeModulePaths(path.dirname(filename));
componentModule._compile(source, filename);
const { RequirementVerification } = componentModule.exports;
const at = 1792400000000;
const context = { component: 'notifications', leaves: ['delivery.channel', 'delivery.retry'] };
const sourcePolicy = { verification_target: 'source', components: ['notifications'], contexts: [context], contribution: 'both', monitor: null };
const monitor = { id: 'delivery-monitor', interval_seconds: 60, freshness_seconds: 300 };
const checkResult = (check, outcome, applicable, time) => ({
  check, scenario: check, run: check, bundle_fingerprint: check, check_result: check,
  outcome, observed_at_ms: time, selected_in_basis: applicable, accepted_basis: applicable, applicable, diagnostics: [],
});
const props = {
  bindings: [{ check: 'source', scenario: 'source' }, { check: 'monitor', scenario: 'monitor' }, { check: 'deployment', scenario: 'deployment' }],
  checks: [
    { id: 'source', fingerprint: 'source', deployment: sourcePolicy },
    { id: 'monitor', fingerprint: 'monitor', deployment: { ...sourcePolicy, verification_target: 'deployment', contexts: [], monitor } },
    { id: 'deployment', fingerprint: 'deployment', deployment: { ...sourcePolicy, verification_target: 'deployment', contexts: [] } },
  ],
  descriptions: [{ id: 'source', description: 'Caller permissions tests' }, { id: 'monitor', description: 'Network isolation probe' }, { id: 'deployment', description: 'Rollout smoke Check' }],
  scenarios: [], scenarioIds: ['source','monitor','deployment','uncovered'], evaluatedAt: at, hasDeployment: true, projectRoot: '/projects/example',
  state: { bindings: [
    { check: 'source', scenario: 'source', status: 'unresolved', reasons: ['Effective context differs on instance process-b.'] },
    { check: 'monitor', scenario: 'monitor', status: 'unresolved', monitor, reasons: ['Monitor Check Result is overdue.'] },
    { check: 'deployment', scenario: 'deployment', status: 'unresolved', reasons: [] },
  ], explanation: { coverage: [
    { check: 'source', scenario: 'source', status: 'unresolved', check_results: [checkResult('source', 'satisfied', false, at - 1000)] },
    { check: 'monitor', scenario: 'monitor', status: 'unresolved', check_results: [checkResult('monitor', 'satisfied', false, at - 600000)] },
    { check: 'deployment', scenario: 'deployment', status: 'unresolved', check_results: [checkResult('deployment', 'violated', true, at - 1000)] },
  ] } },
};
const html = renderToStaticMarkup(React.createElement(RequirementVerification, props));
assert(html.indexOf('Production monitoring') < html.indexOf('Deployment verification'));
assert(html.indexOf('Deployment verification') < html.indexOf('Implementation verification'));
assert(html.includes('Overdue'));
assert(html.includes('Scenarios without contributing Checks'));
assert(html.includes('uncovered'));
assert(html.includes('Effective context differs on instance process-b.'));
assert(html.includes('<code>notifications</code> · delivery.channel, delivery.retry'));
assert(html.includes('Required Execution Context match not established'));
assert(html.includes('Retained fact · not applicable to this deployment'));
assert(html.includes('Failed Check Result'));
assert(html.includes('>Failed</span>'));
assert(html.includes('>Passed</span>'));
assert(!html.includes('>Satisfied</span>'));
assert(html.includes('Exact deployment verified'));
assert(html.includes('/projects/example/runs/deployment'));
const definitions = renderToStaticMarkup(React.createElement(RequirementVerification, { ...props, state: undefined, hasDeployment: false }));
assert(definitions.includes('Select a deployment'));
assert(!definitions.includes('Exact deployment verified'));
assert(!definitions.includes('Verified through the deployed composition'));
console.log('Requirement verification rendering passed: monitoring order, overdue evidence, Execution Context mismatch, failed Check Results, Check Result labels and definitions-only state.');

function loadSource(relative, stubs) {
  const file = path.resolve(__dirname, relative);
  const compiled = ts.transpileModule(fs.readFileSync(file, 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.ReactJSX, target: ts.ScriptTarget.ES2022, esModuleInterop: true },
  }).outputText;
  const loaded = new Module(file, module);
  loaded.filename = file;
  loaded.paths = Module._nodeModulePaths(path.dirname(file));
  const originalRequire = loaded.require.bind(loaded);
  loaded.require = name => Object.hasOwn(stubs, name) ? stubs[name] : originalRequire(name);
  loaded._compile(compiled, file);
  return loaded.exports;
}
async function verifyMaterializedOverview() {
  let scenario = 'healthy';
  const { developmentRead } = loadSource('../src/lib/development-data.ts', {
    'server-only': {}, './development': { developmentScenario: () => scenario },
  });
  const routes = [];
  const { environmentView } = loadSource('../src/lib/production.ts', {
    'server-only': {}, './delivery': { readDeliveryGap: async () => null, readRollbackCandidates: async () => [] }, './assurance': { readProject: async (project, route) => {
      routes.push(route);
      return developmentRead(project, route);
    } },
  });
  const Page = loadSource('../src/app/projects/[projectId]/page.tsx', {
    '@/lib/environment': { selectedEnvironment: async () => 'production' },
    '@/lib/production': { environmentView },
    'next/link': ({ children, prefetch, ...props }) => React.createElement('a', props, children),
  }).default;
  for (const value of ['healthy', 'pending', 'expired']) {
    scenario = value;
    routes.length = 0;
    const view = await environmentView('payments-demo', null);
    assert.equal(!!view.state, value === 'healthy');
    assert.equal(view.freshness.state, value === 'healthy' ? 'ready' : value);
    assert(routes.some(route => route.endsWith('/status/summary')));
    assert(!routes.some(route => route.endsWith('/status')));
    const markup = renderToStaticMarkup(await Page({ params: Promise.resolve({ projectId: 'payments-demo' }), searchParams: Promise.resolve({}) }));
    if (value === 'healthy') {
      assert(markup.includes('Assessment current'));
      assert(markup.includes('All 7 applicable Requirements are satisfied'));
      assert(markup.includes('Modules organize Requirements. Scenarios describe expected behavior.'));
    } else {
      assert(markup.includes(value === 'expired' ? 'Assessment overdue' : 'Assessment updating'));
      assert(!markup.includes('No Requirements need attention'));
      assert(!markup.includes('All 7 applicable Requirements are satisfied'));
      assert(markup.includes('<td>2</td>'));
      assert(markup.includes('goodText">—'));
    }
  }
  console.log('Materialized overview passed: compact reads, fresh counts, pending/expired suppression and preserved Requirement inventory.');
}
verifyMaterializedOverview().catch(error => { console.error(error); process.exitCode = 1; });

async function verifyModelAssessmentFreshness() {
  let freshness = 'ready';
  let snapshotReads = 0;
  const { developmentRead } = loadSource('../src/lib/development-data.ts', { 'server-only': {}, './development': { developmentScenario: () => 'healthy' } });
  const deployment = developmentRead('payments-demo', '/deployments?limit=100').items[0].deployment.id;
  const Page = loadSource('../src/app/projects/[projectId]/model/page.tsx', {
    '@/lib/environment': { selectedEnvironment: async () => 'production' },
    '@/lib/auth': { requireViewer: async () => {} },
    '@/lib/assurance': { routeFingerprint: value => value, readProject: async (project, route) => {
      if (route.endsWith('/snapshot')) snapshotReads += 1;
      const record = structuredClone(developmentRead(project, route));
      if (route.endsWith('/state') || route.endsWith('/requirement-reviews')) {
        if (freshness === 'missing') delete record.assessment;
        else { record.assessment.state = freshness === 'elapsed' ? 'ready' : freshness; if (freshness === 'elapsed' && route.endsWith('/state')) record.assessment.valid_until_ms = 0; }
        for (const requirement of record.requirements) if (requirement.explanation) requirement.explanation.summary = 'Current assessment evidence sentinel';
      }
      return record;
    } },
    '@/lib/production': { requirementStatus: value => ['supported', 'violated'].includes(value) ? value : 'unresolved' },
    'next/headers': { cookies: async () => ({ get: () => undefined }) },
    'next/navigation': { notFound: () => { throw Error('Unexpected missing fixture'); } },
    'next/link': ({ children, prefetch, ...props }) => React.createElement('a', props, children),
    '@/components/model-navigation': { ModelNavigation: () => null },
    '@/components/model-workspace': { ModelWorkspace: ({ children }) => React.createElement('div', null, children) },
    '@/components/copy-value': { CopyValue: () => null },
    '@/components/requirement-verification': { RequirementVerification },
  }).default;
  for (const value of ['ready', 'pending', 'expired', 'missing', 'elapsed']) {
    freshness = value;
    const markup = renderToStaticMarkup(await Page({ params: Promise.resolve({ projectId: 'payments-demo' }), searchParams: Promise.resolve({ context: `deployment:${deployment}`, requirement: 'reminders-arrive-on-time' }) }));
    assert.equal(markup.includes('Current assessment evidence sentinel'), value === 'ready', value);
    assert.equal(markup.includes('>Satisfied</span>'), value === 'ready', value);
    assert.equal(markup.includes('Requirement Review: <strong>accepted</strong>'), value === 'ready' || value === 'elapsed', value);
  }
  assert.equal(snapshotReads, 0, 'Requirement details must not eagerly load the complete evidence snapshot');
  freshness = 'ready';
  await Page({ params: Promise.resolve({ projectId: 'payments-demo' }), searchParams: Promise.resolve({ context: `deployment:${deployment}`, requirement: 'reminders-arrive-on-time', history: '1' }) });
  assert.equal(snapshotReads, 1, 'History remains available on explicit request');
  console.log('Model assessment freshness passed: current-only conclusions, evidence and on-demand history.');
}
verifyModelAssessmentFreshness().catch(error => { console.error(error); process.exitCode = 1; });
