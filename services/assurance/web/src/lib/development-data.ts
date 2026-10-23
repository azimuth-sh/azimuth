import 'server-only';
import { createHash } from 'node:crypto';
import data from '../../development/payments.json';
import { developmentScenario } from './development';

// These identities link illustrative UI records; they are not native protocol fingerprints.
function identity(label: string) {
  return `sha256:${createHash('sha256').update(`assurance-ui-fixture:${label}`).digest('hex')}`;
}

export function developmentRead(project: string, path: string): unknown {
  const scenario = developmentScenario();
  if (!scenario) throw new Error('Synthetic data is not enabled');
  if (project !== data.project) return null;
  const url = new URL(path, 'http://127.0.0.1');
  const route = decodeURIComponent(url.pathname);
  const page = (items: unknown[]) => ({ items, next: null });
  const revision = identity('model');
  const priorMain = identity('prior-main');
  const releaseId = identity('release');
  const verificationTarget = identity('production');
  const at = (minutes: number) => new Date(Date.now() - minutes * 60000).toISOString();
  const notApplicableHere = (id: string) => scenario !== 'healthy' && id === 'delivery-recovers-after-outage';
  const requirements = scenario === 'empty' ? [] : data.requirements.map((requirement) => ({
    ...requirement,
    status: scenario === 'healthy' ? 'supported' : notApplicableHere(requirement.id) ? 'not-applicable' : scenario === 'incident' && requirement.id === 'reminders-arrive-on-time' ? 'violated' : requirement.status,
    reason: scenario === 'healthy' ? 'Illustrative current review and applicable Check Results satisfy this Requirement.' : notApplicableHere(requirement.id) ? 'Outage recovery applies only where the queue-backed delivery context holds; this deployment runs without it.' : scenario === 'incident' && requirement.id === 'reminders-arrive-on-time' ? 'The production delivery probe exceeded the five-minute deadline.' : requirement.reason,
    gap: scenario === 'healthy' || notApplicableHere(requirement.id) ? undefined : requirement.gap,
  }));
  const authority = {
    fingerprint: revision, model_fingerprint: identity('source-model'),
    requirements: requirements.map(c => ({ id: c.id, scenarios: [c.scenario], bindings: [{ check: c.check, scenario: c.scenario }], gaps: [] })),
    checks: requirements.map(c => ({ id: c.check, fingerprint: identity(c.check), deployment: {
      verification_target: c.verification_target, components: ['notifications'], contexts: c.context_required ? [{ component: 'notifications', leaves: ['delivery.channel'] }] : [], contribution: 'both', monitor: c.monitor,
    } })),
  };
  const reviews = requirements.map(c => ({
    record: { kind: 'requirement-review', fingerprint: identity(`review:${c.id}`), requirement: c.id,
      reviewer: 'development-reviewer', reviewed_at_ms: Date.now() - 120 * 60000, decision: c.gap === 'rejected-review' ? 'rejected' : 'accepted',
      rationale: c.reason, provenance: { agent: 'synthetic-author', model: 'illustrative' } },
    assessment: c.gap === 'stale-review' ? 'stale' : c.gap === 'rejected-review' ? 'rejected' : 'accepted',
    diagnostics: c.gap?.endsWith('review') ? [c.reason] : [], historical: false,
    submitted_by: 'development-publisher', submitted_at: at(120), provenance_basis: 'synthetic-ui-data',
  }));
  const reviewState = { assessment: { state: 'ready', generation: 1, published_generation: 1, evaluated_at_ms: Date.now(), valid_until_ms: null }, authority_fingerprint: revision, requirements: requirements.map((c, i) => ({
    requirement: c.id, record: reviews[i].record, assessment: reviews[i].assessment, diagnostics: reviews[i].diagnostics,
  })) };
  const freshness = { state: scenario === 'pending' ? 'pending' : scenario === 'expired' ? 'expired' : 'ready', generation: 2, published_generation: scenario === 'pending' ? 1 : 2, evaluated_at_ms: Date.now() - 60000, valid_until_ms: Date.now() + (scenario === 'expired' ? -1000 : 240000) };
  const state = { assessment: freshness, format: 'azimuth-deployment-status', version: 2, deployment: 'payments-2026-10-18', verification_target_fingerprint: verificationTarget,
    evaluated_at_ms: Date.now(), authority_fingerprint: revision, model_fingerprint: authority.model_fingerprint,
    requirements: requirements.map((c, i) => {
      const computedStatus = 'computed';
      const hasCheckResult = c.gap !== 'missing-execution';
      const acceptedBasis = !c.gap;
      const checkResult = {
        check: c.check, scenario: c.scenario, run: identity(`run:${c.id}`),
        bundle_fingerprint: identity(`run-bundle:${c.id}`), check_result: identity(`check-result:${c.id}`),
        outcome: c.status === 'violated' ? 'violated' : 'satisfied',
        observed_at_ms: Date.now() - (c.gap === 'lost-visibility' ? 180 : 2) * 60000,
        selected_in_basis: acceptedBasis, accepted_basis: acceptedBasis, applicable: acceptedBasis, verification_target_fingerprint: c.verification_target === 'source' ? identity('tested-source') : verificationTarget, diagnostics: [c.reason],
      };
      const checkResults = hasCheckResult ? [checkResult] : [];
      const reasons = [
        ...(reviews[i].assessment !== 'accepted' ? [{ code: `requirement-review-${reviews[i].assessment}`, category: 'requirement-review', message: c.reason }] : []),
        ...(c.gap === 'context-mismatch' ? [{ code: 'context-mismatch', category: 'deployment-evidence', check: c.check, scenario: c.scenario, message: c.reason }] : []),
        ...(c.gap === 'lost-visibility' ? [{ code: 'monitor-overdue', category: 'deployment-evidence', check: c.check, scenario: c.scenario, message: c.reason }] : []),
        ...(!hasCheckResult ? [{ code: 'current-check-result-missing', category: 'evidence', check: c.check, scenario: c.scenario, message: c.reason }] : []),
      ];
      return { requirement: c.id, status: c.status, review: reviews[i].assessment, evaluated_at_ms: Date.now(),
        diagnostics: [c.reason], gaps: c.gap ? [{ kind: c.gap, description: c.reason }] : [],
        bindings: [{ scenario: c.scenario, check: c.check, status: acceptedBasis ? 'satisfied' : 'unresolved', monitor: c.monitor,
          reasons: c.gap ? [c.gap === 'lost-visibility' ? 'Monitor Check Result is overdue.' : c.reason] : [] }], challenges: [],
        explanation: {
          summary: c.reason, status: c.status, reasons,
          review: { status: reviews[i].assessment, fingerprint: reviews[i].record.fingerprint, reviewer: 'development-reviewer', reviewed_at_ms: Date.now() - 120 * 60000, rationale: c.reason },
          requirement_status: { status: computedStatus, fingerprint: null, reviewer: null, reviewed_at_ms: null, rationale: null },
          dependencies: [{ kind: 'release', fingerprint: releaseId }, ...(acceptedBasis && c.context_required ? [{ kind: 'context-capture', fingerprint: identity(`capture:${c.id}`) }] : [])],
          coverage: [{ check: c.check, scenario: c.scenario, status: !hasCheckResult ? 'missing' : acceptedBasis ? checkResult.outcome : 'unassessed', check_results: checkResults }],
          evidence: {
            supporting: acceptedBasis && c.status !== 'violated' ? checkResults : [],
            violations: acceptedBasis && c.status === 'violated' ? checkResults : [],
            inconclusive: [], unassessed: acceptedBasis ? [] : checkResults,
          }, residual_risks: [],
        },
      };
    }),
  };
  const executed = requirements.filter(c => c.gap !== 'missing-execution');
  const runs = executed.map(c => ({ run_id: identity(`run:${c.id}`), revision: 0,
    fingerprint: identity(`run-bundle:${c.id}`), verification_target_fingerprint: c.verification_target === 'source' ? identity('tested-source') : verificationTarget,
    model_fingerprint: authority.model_fingerprint, status: 'complete',
    verification_target: c.verification_target === 'source' ? { kind: 'source', repositories: [{ id: 'payments', revision: 'synthetic-source', content_fingerprint: identity('source-content') }] } : { kind: 'deployment', environment: 'production', id: 'payments-2026-10-18' },
  }));
  const bundles = executed.map((c, i) => ({ ...runs[i],
    check_executions: [{ check: { id: c.check, fingerprint: identity(c.check) }, results: [{
      fingerprint: identity(`check-result:${c.id}`), scenario: c.scenario, outcome: c.status === 'violated' ? 'violated' : 'satisfied',
      observed_at_ms: Date.now() - (c.gap === 'lost-visibility' ? 180 : 2) * 60000,
      diagnostics: [c.reason],
    }] }],
  }));
  const catalog = {
    modules: [...new Set(requirements.map(c => c.module.split('/')[0]))].map(id => ({ id, title: id, requirements: requirements.filter(c => c.module === id).map(c => c.id) })).concat(
      [...new Set(requirements.map(c => c.module).filter(id => id.includes('/')))].map(id => ({ id, title: id.split('/')[1], parent: id.split('/')[0], requirements: requirements.filter(c => c.module === id).map(c => c.id) }))),
    requirements: requirements.map(c => ({ id: c.id, title: c.title, statement: c.statement })),
    scenarios: requirements.map(c => ({ id: c.scenario, description: c.scenario.replaceAll('-', ' ') })),
    checks: requirements.map(c => ({ id: c.check, description: c.method })),
  };
  const priorMainRequirements = requirements.filter(c => c.id !== 'delivery-recovers-after-outage');
  const priorMainIds = new Set(priorMainRequirements.map(c => c.id));
  const priorMainAuthority = {
    ...authority,
    fingerprint: priorMain,
    model_fingerprint: identity('prior-main-source-model'),
    requirements: authority.requirements.filter(c => priorMainIds.has(c.id)),
    checks: authority.checks.filter(c => priorMainRequirements.some(requirement => requirement.check === c.id)),
  };
  const priorMainCatalog = {
    ...catalog,
    modules: catalog.modules.map(module => ({ ...module, requirements: module.requirements.filter(id => priorMainIds.has(id)) })),
    requirements: catalog.requirements.filter(c => priorMainIds.has(c.id)),
    scenarios: catalog.scenarios.filter(c => priorMainRequirements.some(requirement => requirement.scenario === c.id)),
    checks: catalog.checks.filter(c => priorMainRequirements.some(requirement => requirement.check === c.id)),
  };
  const deployment = { id: 'payments-2026-10-18', environment: 'production', verification_target_fingerprint: verificationTarget, deployed_at: at(60) };
  const stagingTarget = identity('staging');
  const stagingDeployment = { id: 'payments-staging-2026-10-19', environment: 'staging', verification_target_fingerprint: stagingTarget, deployed_at: at(20) };
  const onMain = scenario === 'healthy';
  const release = { fingerprint: releaseId, label: 'Payments 1.8', model_revision: revision, changes: [{ id: 'payment-reminders', label: 'Payment reminders' }],
    sources: [{ repository: 'example/payments', commit: 'synthetic-source' }],
    builds: [{ id: 'synthetic-build' }], artifacts: [{ label: 'payments:1.8 (illustrative)' }], deployments: [deployment, stagingDeployment] };
  if (scenario === 'empty') {
    if (route === '/authority') return null;
    if (route === '/requirement-reviews') return { authority_fingerprint: null, requirements: [] };
    if (route === '/reviews') return { records: [], next: null };
    if (['/runs', '/verification-targets', '/model-revisions', '/releases', '/changes', '/deployments', '/environments'].includes(route)) return page([]);
    return null;
  }
  const hour = 3600000;
  const olderDeployment = { deployment_id: 'payments-2026-09-12', release_fingerprint: identity('release:1.7'), label: 'Payments 1.7',
    first_observed_ms: Date.now() - 40 * 24 * hour, last_observed_ms: Date.now() - 10 * 24 * hour, rollout_id: 'rollout-1.7' };
  const previousDeployment = { deployment_id: 'payments-2026-10-10', release_fingerprint: identity('release:1.8-rc'), label: 'Payments 1.8 RC',
    first_observed_ms: Date.now() - 10 * 24 * hour, last_observed_ms: Date.now() - 2 * 24 * hour, rollout_id: 'rollout-1.8-rc' };
  const history = (environment: string) => [
    ...(environment === 'production' ? [{ deployment_id: deployment.id, release_fingerprint: releaseId, label: release.label, first_observed_ms: Date.now() - 2 * 24 * hour, last_observed_ms: Date.now() - 60000, rollout_id: 'rollout-1.8', current: true }] : [{ deployment_id: stagingDeployment.id, release_fingerprint: releaseId, label: release.label, first_observed_ms: Date.now() - 20 * 60000, last_observed_ms: Date.now() - 60000, rollout_id: 'rollout-1.8-staging', current: true }]),
    ...(environment === 'production' ? [{ ...previousDeployment, current: false }, { ...olderDeployment, current: false }] : []),
  ];
  const deliveryGapFor = (environment: string) => {
    const current = environment === 'production' ? deployment : stagingDeployment;
    return { environment, deployment_id: current.id, release_fingerprint: releaseId, release_model_revision: revision, main_model_revision: identity('main-model'),
      main_commit: '9fb783f0c2d14e6a8b7d5c3e1a09f4b2d6c8e7a1', main_sequence: 42,
      requirements: [
        { id: 'delivery-recovers-after-outage', difference: 'added', criticality: 'critical', release_status: null },
        { id: 'reminders-follow-current-payment', difference: 'changed', criticality: 'important', release_status: scenario === 'healthy' ? 'supported' : 'violated' },
        { id: 'notification-content-is-complete', difference: 'changed', criticality: 'routine', release_status: 'supported' },
      ].filter(r => scenario !== 'healthy' || r.difference !== 'changed' || r.id !== 'notification-content-is-complete') };
  };
  const environmentRoute = /^\/environments\/([^/]+)\/(delivery-gap|rollback-candidates|deployment-history)$/.exec(route);
  if (environmentRoute && ['production', 'staging'].includes(environmentRoute[1])) {
    const [, environment, kind] = environmentRoute;
    if (kind === 'delivery-gap') return deliveryGapFor(environment);
    if (kind === 'deployment-history') return { project, environment, deployments: history(environment), next_page_token: null };
    return { project, environment, candidates: environment === 'production' ? [
      { ...previousDeployment, last_status_at_ms: Date.now() - 2 * 24 * hour, non_routine_requirements: 4 },
      { ...olderDeployment, last_status_at_ms: Date.now() - 38 * 24 * hour, non_routine_requirements: 3 },
    ] : [] };
  }
  if (route === '/environments/staging/deployment') return {
    project, environment: 'staging', revision: 1, deployment: stagingDeployment, release_fingerprint: releaseId,
    reason: 'Explicit synthetic staging selection', selected_by: 'development-publisher', selected_at: at(20),
  };
  if (route === '/environments/staging/observations') return {
    revision: 1, fingerprint: identity('staging-deployment-observation'), deployment_id: stagingDeployment.id,
    submitted_by: 'development-reconciler', received_at: at(1), fresh: true,
    observation: { state: 'complete', observed_at_ms: Date.now() - 60000, valid_for_seconds: 300, release_fingerprint: releaseId },
  };
  if (route === `/deployments/${stagingDeployment.id}`) return { deployment: stagingDeployment, release_fingerprint: releaseId };
  if (route === `/deployments/${stagingDeployment.id}/status/summary`) return { ...state, deployment: stagingDeployment.id, verification_target_fingerprint: stagingTarget, requirements: state.requirements.map(({ bindings, explanation, ...requirement }) => ({ ...requirement, status: 'unresolved', explanation: { summary: 'Required Check Results are missing for this staging deployment.' } })) };
  if (route === `/deployments/${stagingDeployment.id}/status` || route === `/model-revisions/${revision}/verification-targets/${stagingTarget}/state`) return {
    ...state, deployment: stagingDeployment.id, verification_target_fingerprint: stagingTarget,
    requirements: state.requirements.map(requirement => ({ ...requirement, status: 'unresolved',
      diagnostics: ['Required Check Results are missing for this staging deployment.'],
      bindings: requirement.bindings.map(binding => ({ ...binding, status: 'unresolved', reasons: ['No applicable Check Result for this staging deployment.'] })),
      explanation: { ...requirement.explanation, status: 'unresolved', summary: 'Required Check Results are missing for this staging deployment.',
        requirement_status: { status: 'computed', fingerprint: null, reviewer: null, reviewed_at_ms: null, rationale: null },
        reasons: [{ code: 'evidence-missing', category: 'deployment-evidence', message: 'Staging has no applicable Check Results.' }],
        coverage: requirement.explanation.coverage.map(binding => ({ ...binding, status: 'missing', check_results: [] })),
        evidence: { supporting: [], violations: [], inconclusive: [], unassessed: [] },
      },
    })),
  };
  if (route === `/model-revisions/${revision}/catalog`) return { project, model_revision: revision, catalog, provenance_basis: 'synthetic-ui-data' };
  if (route === `/model-revisions/${priorMain}/catalog`) return { project, model_revision: priorMain, catalog: priorMainCatalog, provenance_basis: 'synthetic-ui-data' };
  if (route === '/environments/production/observations') return {
    revision: 1,
    fingerprint: identity('deployment-observation'),
    deployment_id: deployment.id,
    submitted_by: 'development-reconciler',
    received_at: at(1),
    fresh: true,
    observation: {
      state: 'complete',
      observed_at_ms: Date.now() - 60000,
      valid_for_seconds: 300,
      release_fingerprint: releaseId,
    },
  };
  if (route === '/environments/production/deployment') return {
    project, environment: 'production', revision: 1, deployment, release_fingerprint: releaseId,
    reason: 'Explicit synthetic development selection', selected_by: 'development-publisher', selected_at: at(60),
  };
  if (route === `/deployments/${deployment.id}`) return { deployment, release_fingerprint: releaseId };
  if (route === '/environments') return page([{ environment: 'production', revision: 1, deployment_id: deployment.id }, { environment: 'staging', revision: 1, deployment_id: stagingDeployment.id }]);
  if (route === '/authority') return onMain ? authority : priorMainAuthority;
  if (route === '/model-revisions') return page([{ revision, selected: onMain }, ...(!onMain ? [{ revision: priorMain, selected: true }] : [])]);
  if (route === '/verification-targets') return page([{ fingerprint: verificationTarget, verification_target: deployment }]);
  if (route === '/requirement-reviews') return onMain ? reviewState : { assessment: { state: 'ready', generation: 1, published_generation: 1, evaluated_at_ms: Date.now(), valid_until_ms: null }, authority_fingerprint: priorMain, requirements: [] };
  if (route === `/model-revisions/${revision}/requirement-reviews`) return reviewState;
  if (route === '/reviews') return { records: onMain ? reviews : [], next: null };
  if (route === '/runs') return page(runs);
  const run = runs.find(r => route === `/runs/${r.run_id}`);
  if (run) return { items: [bundles.find(b => b.run_id === run.run_id)], next: null };
  if (route === `/model-revisions/${revision}`) return { authority, selected: onMain };
  if (route === `/model-revisions/${revision}/snapshot`) return { authority, reviews: [...reviews.map(r => r.record), { kind: 'requirement-status', requirement: requirements[0]?.id, verification_target_fingerprint: verificationTarget, fingerprint: identity('historical-assessment'), conclusion: 'supported', reviewed_at_ms: Date.now()-86400000, rationale: 'Historical authored conclusion retained for inspection.' }], bundles };
  if (route === `/verification-targets/${verificationTarget}/state`) return onMain ? state : { ...state, authority_fingerprint: priorMain, requirements: [] };
  if (route === `/deployments/${deployment.id}/status/summary`) return { ...state, requirements: state.requirements.map(({ bindings, explanation, ...requirement }) => ({ ...requirement, status: freshness.state === 'ready' ? requirement.status : 'unresolved', explanation: { summary: freshness.state === 'ready' ? explanation.summary : 'Evidence is being reassessed.' } })) };
  if (route === `/deployments/${deployment.id}/status` || route === `/model-revisions/${revision}/verification-targets/${verificationTarget}/state`) return state;
  if (route === `/model-revisions/${priorMain}`) return { authority: priorMainAuthority, selected: !onMain };
  if (route === `/model-revisions/${priorMain}/requirement-reviews`) return { assessment: { state: 'ready', generation: 1, published_generation: 1, evaluated_at_ms: Date.now(), valid_until_ms: null }, authority_fingerprint: priorMain, requirements: [] };
  if (route === `/model-revisions/${priorMain}/snapshot`) return { authority: priorMainAuthority, reviews: [], bundles: [] };
  if (route === `/model-revisions/${priorMain}/verification-targets/${verificationTarget}/state`) return { authority_fingerprint: priorMain, model_fingerprint: priorMainAuthority.model_fingerprint, requirements: [] };
  if (route === '/releases') return page([{ fingerprint: releaseId, release }]);
  if (route === `/releases/${releaseId}` || route === `/releases/${releaseId}/summary`) return { publication: { release, catalog, evidence: [] }, submitted_by: 'development-publisher', submitted_at: at(90), deployments: [deployment, stagingDeployment] };
  if (route === '/changes') return page([{ id: 'payment-reminders', revisions: [{ status: onMain ? 'accepted' : 'under-assessment' }] }]);
  if (route === '/deployments') return page([{ deployment, release_fingerprint: releaseId }, { deployment: stagingDeployment, release_fingerprint: releaseId }]);
  if (/^\/environments\/[^/]+\/(deployment|observations)$/.test(route)) return null;
  throw new Error(`Synthetic Assurance UI data does not implement ${route}`);
}
