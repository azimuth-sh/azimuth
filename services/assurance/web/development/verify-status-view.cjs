const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const Module = require('node:module');
const ts = require('typescript');
const React = require('react');
const { renderToStaticMarkup } = require('react-dom/server');

const filename = path.resolve(__dirname, '../src/components/claim-verification.tsx');
const source = ts.transpileModule(fs.readFileSync(filename, 'utf8'), {
  compilerOptions: { module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.ReactJSX, target: ts.ScriptTarget.ES2022 },
}).outputText;
const componentModule = new Module(filename, module);
componentModule.filename = filename;
componentModule.paths = Module._nodeModulePaths(path.dirname(filename));
componentModule._compile(source, filename);
const { ClaimVerification } = componentModule.exports;
const at = 1792400000000;
const context = `sha256:${'c'.repeat(64)}`;
const sourcePolicy = { subject: 'source', components: ['notifications'], contexts: [context], contribution: 'both', monitor: null };
const monitor = { id: 'delivery-monitor', interval_seconds: 60, freshness_seconds: 300 };
const observation = (check, outcome, applicable, time) => ({
  check, case: check, run: check, bundle_fingerprint: check, observation: check,
  outcome, observed_at_ms: time, selected_in_basis: applicable, accepted_basis: applicable, applicable, diagnostics: [],
});
const props = {
  bindings: [{ check: 'source', case: 'source' }, { check: 'monitor', case: 'monitor' }, { check: 'deployment', case: 'deployment' }],
  checks: [
    { id: 'source', fingerprint: 'source', deployment: sourcePolicy },
    { id: 'monitor', fingerprint: 'monitor', deployment: { ...sourcePolicy, subject: 'deployment', contexts: [], monitor } },
    { id: 'deployment', fingerprint: 'deployment', deployment: { ...sourcePolicy, subject: 'deployment', contexts: [] } },
  ],
  descriptions: [{ id: 'source', description: 'Caller permissions tests' }, { id: 'monitor', description: 'Network isolation probe' }, { id: 'deployment', description: 'Rollout smoke Check' }],
  cases: [], caseIds: ['source','monitor','deployment','uncovered'], evaluatedAt: at, hasDeployment: true, projectRoot: '/projects/example',
  state: { bindings: [
    { check: 'source', case: 'source', status: 'unresolved', reasons: ['Effective context differs on instance process-b.'] },
    { check: 'monitor', case: 'monitor', status: 'unresolved', monitor, reasons: ['Monitor observation is overdue.'] },
    { check: 'deployment', case: 'deployment', status: 'unresolved', reasons: [] },
  ], explanation: { coverage: [
    { check: 'source', case: 'source', status: 'unresolved', observations: [observation('source', 'satisfied', false, at - 1000)] },
    { check: 'monitor', case: 'monitor', status: 'unresolved', observations: [observation('monitor', 'satisfied', false, at - 600000)] },
    { check: 'deployment', case: 'deployment', status: 'unresolved', observations: [observation('deployment', 'violated', true, at - 1000)] },
  ] } },
};
const html = renderToStaticMarkup(React.createElement(ClaimVerification, props));
assert(html.indexOf('Production monitoring') < html.indexOf('Deployment verification'));
assert(html.indexOf('Deployment verification') < html.indexOf('Implementation verification'));
assert(html.includes('Overdue'));
assert(html.includes('Cases without contributing Checks'));
assert(html.includes('uncovered'));
assert(html.includes('Effective context differs on instance process-b.'));
assert(html.includes('Required context match not established'));
assert(html.includes('Retained fact · not applicable to this deployment'));
assert(html.includes('Adverse observation'));
assert(html.includes('Exact deployment verified'));
assert(html.includes('/projects/example/runs/deployment'));
const definitions = renderToStaticMarkup(React.createElement(ClaimVerification, { ...props, state: undefined, hasDeployment: false }));
assert(definitions.includes('Select a deployment'));
assert(!definitions.includes('Exact deployment verified'));
assert(!definitions.includes('Verified through the deployed composition'));
console.log('Claim verification rendering passed: monitoring order, overdue evidence, context mismatch, adverse facts and definitions-only state.');
