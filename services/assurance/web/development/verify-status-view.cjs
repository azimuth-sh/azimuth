const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const Module = require('node:module');
const ts = require('typescript');
const React = require('react');
const { renderToStaticMarkup } = require('react-dom/server');

const localTime = loadSource('../src/components/local-time.tsx', { '../lib/time': loadSource('../src/lib/time.ts', {}) });
const sharedStubs = {
  './local-time': localTime,
  './disclosure': loadSource('../src/components/disclosure.tsx', {}),
  './markdown': loadSource('../src/components/markdown.tsx', {}),
  './identifier': { Identifier: ({ value, label }) => React.createElement('code', { title: value }, label ?? value) },
};
const { RequirementVerification } = loadSource('../src/components/requirement-verification.tsx', sharedStubs);
const identifierLibrary = loadSource('../src/lib/identifier.ts', {});
const workHistoryComponents = loadSource('../src/components/work-history.tsx', { ...sharedStubs, '../lib/identifier': identifierLibrary });
const reviewDimensions = loadSource('../src/components/review-dimensions.tsx', { '@/lib/identifier': identifierLibrary });
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
  descriptions: [{ id: 'source', description: '**Caller** permissions `tests`\n\n- first\n- second' }, { id: 'monitor', description: 'Network isolation probe' }, { id: 'deployment', description: 'Rollout smoke Check' }],
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
    '@/components/disclosure': sharedStubs['./disclosure'],
    '@/components/local-time': localTime,
    '@/components/identifier': sharedStubs['./identifier'],
    '@/lib/identifier': loadSource('../src/lib/identifier.ts', {}),
    '@/components/review-dimensions': reviewDimensions,
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
      assert(!markup.includes('Assessment updating') && !markup.includes('Assessment overdue'));
      assert(markup.includes('All 7 applicable Requirements that declare verification are satisfied'));
      assert(markup.includes('Modules organize Requirements. Scenarios describe expected behavior.'));
      assert(markup.includes('VERIFICATION DECLARED · 7'), 'verification tiles count only Requirements declaring Verification');
      assert(/aria-label="Realization reviewed">.*?<strong>2<small> of 3 declared<\/small><\/strong>/.test(markup), 'accepted out of declared Realization Reviews');
      assert(/aria-label="Specified only">.*?<strong>1<\/strong>/.test(markup), 'Requirements declaring neither review');
      const drift = /<ul class="driftList"[^>]*>(.*?)<\/ul>/.exec(markup);
      assert(drift, 'drift list');
      assert(drift[1].includes('Reminders reflect current payments') && drift[1].includes('Reviewed at <code title="4c1d9e7a2b3f5061d8e9a7c4b2f1e0d3c5a6b7e8">4c1d9e7a</code>, code changed since'), 'drifted accepted Realization Review names its commit');
      assert(!drift[1].includes('Workers access only necessary payment data'), 'an undrifted review is not listed');
      assert(!markup.includes('Operators can pause reminders'), 'a specified-only Requirement never needs attention');
      assert(!/criticality|non-routine/i.test(markup), 'no criticality presentation remains');
    } else {
      assert(markup.includes(value === 'expired' ? 'Assessment overdue' : 'Assessment updating'));
      assert(!markup.includes('No Requirements need attention'));
      assert(!markup.includes('All 7 applicable Requirements that declare verification are satisfied'));
      assert(markup.includes('<td>2</td>'));
      assert(markup.includes('goodText">—'));
    }
  }
  console.log('Materialized overview passed: compact reads, verification-declared counts, Realization Review drift list, specified-only count, pending/expired suppression and preserved Requirement inventory.');
}
assert(html.includes('<strong>Caller</strong>') && html.includes('<code>tests</code>') && html.includes('<li>first</li>'), 'Check method renders as Markdown');
{
  const single = renderToStaticMarkup(React.createElement(RequirementVerification, { ...props, bindings: [{ check: 'source', scenario: 'source' }], scenarioIds: ['source'] }));
  assert(single.includes('<strong>Production monitoring</strong> · no Monitors declared'), 'empty stages collapse to one line');
  assert(single.includes('<strong>Deployment verification</strong> · no Checks declared'));
  assert(!single.includes('Scenarios without contributing Checks'));
  const grouped = renderToStaticMarkup(React.createElement(RequirementVerification, { ...props, bindings: [{ check: 'source', scenario: 'a' }, { check: 'source', scenario: 'b' }], scenarioIds: ['a', 'b'] }));
  assert.equal(grouped.split('<strong>source</strong>').length - 1, 1, 'one row per Check regardless of Scenario count');
  assert(grouped.includes('<code>a</code>') && grouped.includes('<code>b</code>'));
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
    '@/components/disclosure': sharedStubs['./disclosure'],
    '@/components/local-time': localTime,
    '@/components/markdown': sharedStubs['./markdown'],
    '@/components/work-history': workHistoryComponents,
    '@/components/review-dimensions': reviewDimensions,
    '@/lib/identifier': identifierLibrary,
  }).default;
  for (const value of ['ready', 'pending', 'expired', 'missing', 'elapsed']) {
    freshness = value;
    const markup = renderToStaticMarkup(await Page({ params: Promise.resolve({ projectId: 'payments-demo' }), searchParams: Promise.resolve({ context: `deployment:${deployment}`, requirement: 'reminders-arrive-on-time' }) }));
    assert.equal(markup.includes('Current assessment evidence sentinel'), value === 'ready', value);
    assert.equal(markup.includes('>Satisfied</span>'), value === 'ready', value);
    assert.equal(markup.includes('Verification review: <strong>Accepted</strong> · Realization review: <strong>Not declared</strong>'), value === 'ready' || value === 'elapsed', value);
  }
  assert.equal(snapshotReads, 0, 'Requirement details must not eagerly load the complete evidence snapshot');
  freshness = 'ready';
  await Page({ params: Promise.resolve({ projectId: 'payments-demo' }), searchParams: Promise.resolve({ context: `deployment:${deployment}`, requirement: 'reminders-arrive-on-time', history: '1' }) });
  assert.equal(snapshotReads, 1, 'History remains available on explicit request');
  console.log('Model assessment freshness passed: current-only conclusions, evidence and on-demand history.');
}
verifyModelAssessmentFreshness().catch(error => { console.error(error); process.exitCode = 1; });

function verifyWorkHistory() {
  const { developmentRead } = loadSource('../src/lib/development-data.ts', { 'server-only': {}, './development': { developmentScenario: () => 'mixed' } });
  const { WorkHistory } = workHistoryComponents;
  const history = developmentRead('payments-demo', '/provenance/requirements/reminders-follow-current-payment');
  const markup = renderToStaticMarkup(React.createElement(WorkHistory, { history }));
  assert(markup.includes('Work provenance — not used for Requirement Status'), 'non-evidence caption');
  assert(markup.includes('never affects Verification or Realization Review currentness, Requirement Status or acceptance'));
  assert(/<details class="disclosure"><summary><span class="disclosureTitle">Work history<\/span>/.test(markup), 'work history is collapsed by default');
  assert(markup.includes('>intent</span>') && markup.includes('>dependency</span>'), 'relation badges');
  assert(markup.indexOf('Payments 1.8') < markup.indexOf('Payments 1.7'), 'newest release first');
  assert(markup.includes('basis-harness">harness-attested</span>'), 'harness basis marker');
  assert(markup.includes('<span>example-model-medium</span> <span class="basisMarker basis-declared">declared</span>'), 'declared basis marker');
  assert(markup.includes('basis-unknown" title="Not recorded; never inferred">unknown</span>'), 'unknown renders as unknown');
  assert(markup.includes('example-orchestrator (version unknown)'), 'declared harness with unknown version');
  assert(!/<dd>\s*<\/dd>/.test(markup), 'no descriptor or time value is blank');
  assert(markup.includes('<dt>Agent type</dt><dd><span>general-purpose</span> <span class="basisMarker basis-harness">harness-attested</span></dd>'), 'harness agent type');
  assert(markup.includes('<span>implementer</span> <span class="basisMarker basis-declared">declared</span>'), 'declared agent type');
  assert(markup.includes('>4 calls · 120k in (+98k cached) / 2.4k out</span>'), 'compact own token usage');
  assert(markup.includes('> · 2 delegated agents: 40k in / 900 out</span>'), 'delegated token usage when agents were launched');
  assert(markup.includes('>6 calls · 1.8k in (+912k cached) / 18.4k out</span>') && !markup.includes('0 delegated agents'), 'no delegated usage without delegated agents');
  assert.equal((markup.match(/class="basisMarker usageOverlapping"/g) ?? []).length, 1, 'only overlapping usage is marked');
  assert(/<dt>Token usage<\/dt><dd><span class="basisMarker basis-unknown" title="Not recorded; never inferred">unknown<\/span><\/dd>/.test(markup), 'unrecorded usage is unknown, never zero');
  assert(markup.includes('>rejected</span>') && markup.includes('>accepted</span>'), 'rejected review followed by a corrected one');
  assert(markup.includes('14 min'), 'derived duration from handoff to record');
  assert(markup.includes('>not recorded</time>'), 'missing handoff time is explicit');
  assert(markup.includes('No stored ledger for this change.'));
  assert(markup.includes('<summary><span class="disclosureTitle">Report</span>'), 'report is collapsed');
  assert(markup.includes('stale-snapshot-amount-1') && markup.includes('medium confidence'), 'failure attribution');
  const caused = history.attributions[0].attempts[0].attempt;
  assert(markup.includes(`id="${caused}"`) && markup.includes(`href="#${caused}"`), 'attribution links to the attempt');
  assert(markup.includes('>Caused</span>') && markup.includes('>Allowed through</span>'));
  assert(markup.includes('· superseded</p>') && markup.includes('· completed</p>'), 'attempt instance disposition');
  const intentAttempts = history.releases.find(release => release.relation === 'intent').changes[0].attempts;
  assert(markup.indexOf(`id="${intentAttempts[2].id}"`) < markup.indexOf(`id="${intentAttempts[3].id}"`), 'ledger order across instances is preserved');
  const { reviewAttempt } = workHistoryComponents;
  const accepted = intentAttempts[4];
  assert.equal(reviewAttempt(history, 'verification-review', accepted.generated[0].record_fingerprint), accepted, 'reviewing attempt matches the native review fingerprint');
  assert.equal(reviewAttempt(history, 'verification-review', accepted.generated[0].fingerprint), undefined, 'the review bundle file fingerprint never identifies a review');
  assert.equal(reviewAttempt(history, 'realization-review', accepted.generated[0].record_fingerprint), undefined, 'a review of another kind never identifies the attempt');
  const dependencyAttempts = history.releases.find(release => release.relation === 'dependency').changes[0].attempts;
  const realizationAttempt = dependencyAttempts.find(attempt => attempt.action === 'review-requirement-realization');
  assert.equal(reviewAttempt(history, 'realization-review', realizationAttempt.generated[0].record_fingerprint), realizationAttempt, 'a Realization Review attempt is matched by its native review');
  assert(markup.includes('<strong>review-requirement-realization</strong>') && markup.includes('<strong>review-requirement-verification</strong>'), 'work history shows both review tasks');
  const historical = structuredClone(history);
  historical.releases.flatMap(release => release.changes.flatMap(change => change.attempts)).forEach(attempt => {
    if (attempt.action === 'review-requirement-verification') attempt.action = attempt.task = 'review-requirement';
  });
  assert.equal(reviewAttempt(historical, 'verification-review', accepted.generated[0].record_fingerprint).id, accepted.id, 'a ledger recorded under an earlier action name still identifies its review');
  const unavailable = renderToStaticMarkup(React.createElement(WorkHistory, { history: undefined }));
  assert(unavailable.includes('Work history is unavailable right now') && unavailable.includes('not used for Requirement Status'));
  const empty = renderToStaticMarkup(React.createElement(WorkHistory, { history: null }));
  assert(empty.includes('No work history recorded for this Requirement.'));
  console.log('Work history rendering passed: non-evidence caption, collapsed section, relation badges, harness/declared/unknown descriptors and agent types, token usage, derived durations, collapsed reports and Failure Attribution links.');
}
verifyWorkHistory();

async function verifyRequirementPageWorkHistory() {
  const { developmentRead } = loadSource('../src/lib/development-data.ts', { 'server-only': {}, './development': { developmentScenario: () => 'healthy' } });
  let failProvenance = false;
  const Page = loadSource('../src/app/projects/[projectId]/model/page.tsx', {
    '@/lib/environment': { selectedEnvironment: async () => 'production' },
    '@/lib/auth': { requireViewer: async () => {} },
    '@/lib/assurance': { routeFingerprint: value => value, readProject: async (project, route) => {
      if (failProvenance && route.startsWith('/provenance/')) throw new Error('Assurance API unavailable (503); no conclusion inferred');
      return structuredClone(developmentRead(project, route));
    } },
    '@/lib/production': { requirementStatus: value => ['supported', 'violated'].includes(value) ? value : 'unresolved' },
    'next/headers': { cookies: async () => ({ get: () => undefined }) },
    'next/navigation': { notFound: () => { throw Error('Unexpected missing fixture'); } },
    'next/link': ({ children, prefetch, ...props }) => React.createElement('a', props, children),
    '@/components/model-navigation': { ModelNavigation: () => null },
    '@/components/model-workspace': { ModelWorkspace: ({ children }) => React.createElement('div', null, children) },
    '@/components/copy-value': { CopyValue: () => null },
    '@/components/requirement-verification': { RequirementVerification },
    '@/components/disclosure': sharedStubs['./disclosure'],
    '@/components/local-time': localTime,
    '@/components/markdown': sharedStubs['./markdown'],
    '@/components/work-history': workHistoryComponents,
    '@/components/review-dimensions': reviewDimensions,
    '@/lib/identifier': identifierLibrary,
  }).default;
  const render = async requirement => renderToStaticMarkup(await Page({ params: Promise.resolve({ projectId: 'payments-demo' }), searchParams: Promise.resolve({ requirement }) }));
  const markup = await render('reminders-follow-current-payment');
  assert(markup.indexOf('id="verification-review"') < markup.indexOf('id="realization-review"'), 'Verification Review precedes Realization Review');
  assert(markup.indexOf('id="realization-review"') < markup.indexOf('id="requirement-work-history"'), 'work history follows the reviews');
  const realization = markup.slice(markup.indexOf('id="realization-review"'), markup.indexOf('id="requirement-work-history"'));
  assert(realization.includes('<span class="badge good">Accepted</span>') && realization.includes('<span class="codeDrift">Reviewed at <code'), 'drift is a muted note beside an accepted review');
  assert(realization.includes('<dt>Reviewed commit</dt>') && !realization.includes('<dt>Challenge basis</dt>'), 'Realization Review record fields');
  assert(markup.includes('Reviewing attempt from Work Provenance: <a href="#attempt-'), 'review shows its generating attempt');
  assert(realization.includes('Reviewing attempt from Work Provenance: <a href="#attempt-') && realization.includes('session-reviewer-4'), 'Realization Review shows its generating attempt');
  assert(markup.includes('<dt>Reviewed dependencies</dt>') && markup.includes('<dt>Challenge basis</dt><dd>none</dd>'), 'review record fields');
  assert(markup.includes('verification</code> model/reminders/planning/verification.md'), 'reviewed artifacts');
  assert(markup.includes('&quot;dependencies_fingerprint&quot;') && !markup.includes('&quot;provenance&quot;') && !markup.includes('Author provenance'), 'review record carries no authored provenance');
  const other = await render('reminders-arrive-on-time');
  assert(!other.includes('Reviewing attempt from Work Provenance'), 'no reviewer descriptor without a generating attempt');
  assert(other.includes('so no realization review is owed'), 'an undeclared Realization Review is not owed');
  const specified = await render('operators-can-pause-reminders');
  assert(specified.includes('Verification review: <strong>Not declared</strong> · Realization review: <strong>Not declared</strong>'), 'specified-only Requirement shows no review owed');
  assert(!specified.includes('codeDrift'), 'no drift without a Realization Review');
  failProvenance = true;
  const degraded = await render('reminders-follow-current-payment');
  assert(degraded.includes('>Satisfied</span>') && degraded.includes('Work history is unavailable right now'), 'unavailable provenance never hides Requirement Status');
  console.log('Requirement page work history passed: placement after review, reviewing-attempt descriptor, review record fields and degraded provenance reads.');
}
verifyRequirementPageWorkHistory().catch(error => { console.error(error); process.exitCode = 1; });
