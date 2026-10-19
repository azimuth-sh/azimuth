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
  const baseline = identity('baseline');
  const releaseId = identity('release');
  const subject = identity('production');
  const at = (minutes: number) => new Date(Date.now() - minutes * 60000).toISOString();
  const claims = scenario === 'empty' ? [] : data.claims.map((claim) => ({
    ...claim,
    status: scenario === 'healthy' ? 'supported' : scenario === 'incident' && claim.id === 'reminders-arrive-on-time' ? 'violated' : claim.status,
    reason: scenario === 'healthy' ? 'Illustrative current review and applicable observations support this Claim.' : scenario === 'incident' && claim.id === 'reminders-arrive-on-time' ? 'The production delivery probe exceeded the five-minute deadline.' : claim.reason,
    gap: scenario === 'healthy' ? undefined : claim.gap,
  }));
  const authority = {
    fingerprint: revision, model_fingerprint: identity('source-model'),
    claims: claims.map(c => ({ id: c.id, cases: [c.case], bindings: [{ check: c.check, case: c.case }], gaps: [] })),
    checks: claims.map(c => ({ id: c.check, fingerprint: identity(c.check) })),
  };
  const reviews = claims.map(c => ({
    record: { kind: 'claim-review', fingerprint: identity(`review:${c.id}`), claim: c.id,
      reviewer: 'development-reviewer', reviewed_at_ms: Date.now() - 120 * 60000, decision: c.gap === 'rejected-review' ? 'rejected' : 'accepted',
      rationale: c.reason, provenance: { agent: 'synthetic-author', model: 'illustrative' } },
    assessment: c.gap === 'stale-review' ? 'stale' : c.gap === 'rejected-review' ? 'rejected' : 'accepted',
    diagnostics: c.gap?.endsWith('review') ? [c.reason] : [], historical: false,
    submitted_by: 'development-publisher', submitted_at: at(120), provenance_basis: 'synthetic-ui-data',
  }));
  const reviewState = { authority_fingerprint: revision, claims: claims.map((c, i) => ({
    claim: c.id, record: reviews[i].record, assessment: reviews[i].assessment, diagnostics: reviews[i].diagnostics,
  })) };
  const state = { authority_fingerprint: revision, model_fingerprint: authority.model_fingerprint,
    claims: claims.map((c, i) => {
      const healthStatus = c.gap ? 'missing' : 'accepted';
      const hasObservation = c.gap !== 'missing-execution';
      const acceptedBasis = !c.gap;
      const observation = {
        check: c.check, case: c.case, run: identity(`run:${c.id}`),
        bundle_fingerprint: identity(`run-bundle:${c.id}`), observation: identity(`observation:${c.id}`),
        outcome: c.status === 'violated' ? 'violated' : 'satisfied',
        observed_at_ms: Date.now() - (c.gap === 'lost-visibility' ? 180 : 2) * 60000,
        selected_in_basis: acceptedBasis, accepted_basis: acceptedBasis, diagnostics: [c.reason],
      };
      const observations = hasObservation ? [observation] : [];
      const reasons = [
        ...(reviews[i].assessment !== 'accepted' ? [{ code: `claim-review-${reviews[i].assessment}`, category: 'claim-review', message: c.reason }] : []),
        ...(healthStatus !== 'accepted' ? [{ code: 'claim-health-missing', category: 'claim-health', message: 'Claim Health is missing for this assessment.' }] : []),
        ...(!hasObservation ? [{ code: 'current-observation-missing', category: 'evidence', check: c.check, case: c.case, message: c.reason }] : []),
      ];
      return { claim: c.id, status: c.status, review: reviews[i].assessment, health: healthStatus,
        diagnostics: [c.reason], gaps: c.gap ? [{ kind: c.gap, description: c.reason }] : [],
        bindings: [{ case: c.case, check: c.check }], challenges: [],
        explanation: {
          summary: c.reason, status: c.status, reasons,
          review: { status: reviews[i].assessment, fingerprint: reviews[i].record.fingerprint, reviewer: 'development-reviewer', reviewed_at_ms: Date.now() - 120 * 60000, rationale: c.reason },
          health: { status: healthStatus, fingerprint: acceptedBasis ? identity(`health:${c.id}`) : null, reviewer: acceptedBasis ? 'development-reviewer' : null, reviewed_at_ms: acceptedBasis ? Date.now() - 60000 : null, rationale: acceptedBasis ? c.reason : null },
          coverage: [{ check: c.check, case: c.case, status: !hasObservation ? 'missing' : acceptedBasis ? observation.outcome : 'unassessed', observations }],
          evidence: {
            supporting: acceptedBasis && c.status !== 'violated' ? observations : [],
            violations: acceptedBasis && c.status === 'violated' ? observations : [],
            inconclusive: [], unassessed: acceptedBasis ? [] : observations,
          }, residual_risks: [],
        },
      };
    }),
  };
  const executed = claims.filter(c => c.gap !== 'missing-execution');
  const runs = executed.map(c => ({ run_id: identity(`run:${c.id}`), revision: 0,
    fingerprint: identity(`run-bundle:${c.id}`), subject_fingerprint: subject,
    model_fingerprint: authority.model_fingerprint, status: 'complete',
    subject: { kind: 'deployment', environment: 'production', id: 'payments-2026-10-18' },
  }));
  const bundles = executed.map((c, i) => ({ ...runs[i],
    check_executions: [{ check: { id: c.check, fingerprint: identity(c.check) }, observations: [{
      fingerprint: identity(`observation:${c.id}`), case: c.case, outcome: c.status === 'violated' ? 'violated' : 'satisfied',
      observed_at_ms: Date.now() - (c.gap === 'lost-visibility' ? 180 : 2) * 60000,
      diagnostics: [c.reason],
    }] }],
  }));
  const catalog = {
    modules: [...new Set(claims.map(c => c.module.split('/')[0]))].map(id => ({ id, title: id, claims: claims.filter(c => c.module === id).map(c => c.id) })).concat(
      [...new Set(claims.map(c => c.module).filter(id => id.includes('/')))].map(id => ({ id, title: id.split('/')[1], parent: id.split('/')[0], claims: claims.filter(c => c.module === id).map(c => c.id) }))),
    claims: claims.map(c => ({ id: c.id, statement: c.statement })),
    cases: claims.map(c => ({ id: c.case, description: c.case.replaceAll('-', ' ') })),
    checks: claims.map(c => ({ id: c.check, description: c.method })),
  };
  const baselineClaims = claims.filter(c => c.id !== 'delivery-recovers-after-outage');
  const baselineIds = new Set(baselineClaims.map(c => c.id));
  const baselineAuthority = {
    ...authority,
    fingerprint: baseline,
    model_fingerprint: identity('baseline-source-model'),
    claims: authority.claims.filter(c => baselineIds.has(c.id)),
    checks: authority.checks.filter(c => baselineClaims.some(claim => claim.check === c.id)),
  };
  const baselineCatalog = {
    ...catalog,
    modules: catalog.modules.map(module => ({ ...module, claims: module.claims.filter(id => baselineIds.has(id)) })),
    claims: catalog.claims.filter(c => baselineIds.has(c.id)),
    cases: catalog.cases.filter(c => baselineClaims.some(claim => claim.case === c.id)),
    checks: catalog.checks.filter(c => baselineClaims.some(claim => claim.check === c.id)),
  };
  const deployment = { id: 'payments-2026-10-18', environment: 'production', subject_fingerprint: subject, deployed_at: at(60) };
  const stagingSubject = identity('staging');
  const stagingDeployment = { id: 'payments-staging-2026-10-19', environment: 'staging', subject_fingerprint: stagingSubject, deployed_at: at(20) };
  const accepted = scenario === 'healthy';
  const release = { fingerprint: releaseId, label: 'Payments 1.8', baseline_model_revision: baseline,
    model_revision: revision, changes: [{ id: 'payment-reminders', label: 'Payment reminders' }],
    sources: [{ repository: 'example/payments', commit: 'synthetic-source' }],
    builds: [{ id: 'synthetic-build' }], artifacts: [{ label: 'payments:1.8 (illustrative)' }], deployments: [deployment, stagingDeployment] };
  if (scenario === 'empty') {
    if (route === '/authority') return null;
    if (route === '/claim-reviews') return { authority_fingerprint: null, claims: [] };
    if (route === '/reviews') return { records: [], next: null };
    if (['/runs', '/subjects', '/model-revisions', '/releases', '/changes', '/deployments', '/environments'].includes(route)) return page([]);
    return null;
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
  if (route === `/model-revisions/${revision}/subjects/${stagingSubject}/state`) return {
    authority_fingerprint: revision, model_fingerprint: authority.model_fingerprint,
    claims: state.claims.map(claim => ({ ...claim, status: 'unresolved', health: 'missing',
      diagnostics: ['No assessment has been recorded for this staging deployment.'],
      explanation: { ...claim.explanation, status: 'unresolved', summary: 'No assessment has been recorded for this staging deployment.',
        health: { status: 'missing', fingerprint: null, reviewer: null, reviewed_at_ms: null, rationale: null },
        reasons: [{ code: 'claim-health-missing', category: 'claim-health', message: 'Staging evidence has not been assessed.' }],
        coverage: claim.explanation.coverage.map(binding => ({ ...binding, status: 'missing', observations: [] })),
        evidence: { supporting: [], violations: [], inconclusive: [], unassessed: [] },
      },
    })),
  };
  if (route === `/model-revisions/${revision}/catalog`) return { project, model_revision: revision, catalog, provenance_basis: 'synthetic-ui-data' };
  if (route === `/model-revisions/${baseline}/catalog`) return { project, model_revision: baseline, catalog: baselineCatalog, provenance_basis: 'synthetic-ui-data' };
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
  if (route === '/authority') return accepted ? authority : baselineAuthority;
  if (route === '/model-revisions') return page([{ revision, selected: accepted }, ...(!accepted ? [{ revision: baseline, selected: true }] : [])]);
  if (route === '/subjects') return page([{ fingerprint: subject, subject: deployment }]);
  if (route === '/claim-reviews') return accepted ? reviewState : { authority_fingerprint: baseline, claims: [] };
  if (route === `/model-revisions/${revision}/claim-reviews`) return reviewState;
  if (route === '/reviews') return { records: accepted ? reviews : [], next: null };
  if (route === '/runs') return page(runs);
  const run = runs.find(r => route === `/runs/${r.run_id}`);
  if (run) return { items: [bundles.find(b => b.run_id === run.run_id)], next: null };
  if (route === `/model-revisions/${revision}`) return { authority, selected: accepted };
  if (route === `/model-revisions/${revision}/snapshot`) return { authority, reviews: reviews.map(r => r.record), bundles };
  if (route === `/subjects/${subject}/state`) return accepted ? state : { ...state, authority_fingerprint: baseline, claims: [] };
  if (route === `/model-revisions/${revision}/subjects/${subject}/state`) return state;
  if (route === `/model-revisions/${baseline}`) return { authority: baselineAuthority, selected: !accepted };
  if (route === `/model-revisions/${baseline}/claim-reviews`) return { authority_fingerprint: baseline, claims: [] };
  if (route === `/model-revisions/${baseline}/snapshot`) return { authority: baselineAuthority, reviews: [], bundles: [] };
  if (route === `/model-revisions/${baseline}/subjects/${subject}/state`) return { authority_fingerprint: baseline, model_fingerprint: baselineAuthority.model_fingerprint, claims: [] };
  if (route === '/releases') return page([{ fingerprint: releaseId, release, accepted }]);
  if (route === `/releases/${releaseId}` || route === `/releases/${releaseId}/summary`) return { publication: { release, catalog, evidence: [] }, submitted_by: 'development-publisher', submitted_at: at(90), acceptance: accepted ? { owner: 'development-owner', rationale: 'Illustrative acceptance' } : null, deployments: [deployment, stagingDeployment] };
  if (route === '/changes') return page([{ id: 'payment-reminders', revisions: [{ status: accepted ? 'accepted' : 'under-assessment' }] }]);
  if (route === '/deployments') return page([{ deployment, release_fingerprint: releaseId }, { deployment: stagingDeployment, release_fingerprint: releaseId }]);
  if (/^\/environments\/[^/]+\/(deployment|observations)$/.test(route)) return null;
  throw new Error(`Synthetic Assurance UI data does not implement ${route}`);
}
