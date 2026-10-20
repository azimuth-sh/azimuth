import { selectedEnvironment } from "@/lib/environment";
import Link from 'next/link';
import { cookies } from 'next/headers';
import { notFound } from 'next/navigation';
import { requireViewer } from '@/lib/auth';
import { readProject, routeFingerprint, type Page, type ClaimReviewState, type SubjectState, type DeploymentCheckPolicy } from '@/lib/assurance';
import { claimStatus } from '@/lib/production';
import { ModelNavigation } from '@/components/model-navigation';
import { ModelWorkspace } from '@/components/model-workspace';
import { CopyValue } from '@/components/copy-value';
import { ClaimVerification } from '@/components/claim-verification';

export const dynamic = 'force-dynamic';
type Catalog = {
  modules: { id: string; title: string; parent?: string; claims: string[] }[];
  claims: { id: string; title?: string; statement: string }[];
  cases: { id: string; description: string }[];
  checks: { id: string; description: string }[];
};
type Authority = {
  fingerprint: string;
  claims: { id: string; cases: string[]; bindings: { check: string; case: string }[]; gaps: unknown }[];
  checks: { id: string; fingerprint: string; deployment?: DeploymentCheckPolicy | null }[];
};
type Revision = { revision: string; selected: boolean };
type ReleaseSummary = { fingerprint: string; release: { label: string; model_revision: string }; accepted: boolean };
type Deployment = { deployment: { id: string; environment: string; subject_fingerprint?: string }; release_fingerprint: string };
type Snapshot = { reviews: Record<string, unknown>[] };
const labels = { supported: 'Confirmed', violated: 'Violated', unresolved: 'Unresolved' };
const tones = { supported: 'good', violated: 'bad', unresolved: 'warn' };
function date(value: unknown) {
  return typeof value === 'number' && Number.isFinite(value)
    ? new Date(value).toLocaleString('en-GB', { timeZone: 'UTC', dateStyle: 'medium', timeStyle: 'short' }) + ' UTC'
    : 'Date not recorded';
}

export default async function ModelPage({ params, searchParams }: {
  params: Promise<{ projectId: string }>;
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}) {
  const { projectId } = await params;
  await requireViewer(projectId);
  const query = await searchParams;
  const environment = await selectedEnvironment(projectId, query.environment);
  const pinned = (await cookies()).get('assurance-modules-pinned')?.value === '1';
  if (['revision', 'subject', 'module', 'claim', 'context'].some(key => Array.isArray(query[key]))) notFound();
  const requestedRevision = typeof query.revision === 'string' ? routeFingerprint(query.revision) : null;
  let subject = typeof query.subject === 'string' ? routeFingerprint(query.subject) : null;
  if (query.revision && !requestedRevision || query.subject && !subject) notFound();
  const root = `/projects/${encodeURIComponent(projectId)}`;
  const [revisions, deployments, releases, currentEnvironment] = await Promise.all([
    readProject<Page<Revision>>(projectId, '/model-revisions?limit=100'),
    readProject<Page<Deployment>>(projectId, '/deployments?limit=100'),
    readProject<Page<ReleaseSummary>>(projectId, '/releases?limit=100'),
    readProject<Deployment>(projectId, `/environments/${encodeURIComponent(environment)}/deployment`),
  ]);
  function environmentDeployment(fingerprint: string) {
    if (currentEnvironment?.release_fingerprint === fingerprint) return currentEnvironment;
    const recorded = deployments?.items.filter(d => d.release_fingerprint === fingerprint && d.deployment.environment === environment) ?? [];
    return !deployments?.next && recorded.length === 1 ? recorded[0] : null;
  }
  const context = typeof query.context === 'string' ? query.context : undefined;
  let revision = requestedRevision ?? revisions?.items.find(r => r.selected)?.revision;
  let deployment: Deployment | null = null;
  let release: ReleaseSummary | undefined;
  if (context?.startsWith('deployment:')) {
    deployment = await readProject<Deployment>(projectId, `/deployments/${encodeURIComponent(context.slice(11))}`);
    if (!deployment) notFound();
  } else if (!context && subject) {
    deployment = deployments?.items.find(d => d.deployment.subject_fingerprint === subject) ?? null;
    if (!deployment) notFound();
  }
  if (deployment && deployment.deployment.environment !== environment) notFound();
  const releaseFingerprint = deployment?.release_fingerprint ?? (context?.startsWith('release:') ? routeFingerprint(context.slice(8)) : null);
  if (releaseFingerprint) {
    const record = await readProject<{ publication: { release: ReleaseSummary['release'] }; acceptance: unknown }>(projectId, `/releases/${encodeURIComponent(releaseFingerprint)}/summary`);
    if (!record) notFound();
    release = { fingerprint: releaseFingerprint, release: record.publication.release, accepted: !!record.acceptance };
    revision = release.release.model_revision;
    deployment ??= environmentDeployment(releaseFingerprint);
    subject = deployment?.deployment.subject_fingerprint ?? null;
  } else if (context === 'accepted') {
    revision = revisions?.items.find(r => r.selected)?.revision;
    subject = null;
  } else if (context) {
    notFound();
  } else {
    const matching = releases?.items.filter(r => r.release.model_revision === revision) ?? [];
    release = matching.length === 1 ? matching[0] : undefined;
    if (release) {
      deployment = environmentDeployment(release.fingerprint);
      subject = deployment?.deployment.subject_fingerprint ?? null;
    }
  }
  const contextValue = release ? `release:${release.fingerprint}` : 'accepted';
  const contexts = [
    { value: 'accepted', label: 'Accepted model · definitions' },
    ...(releases?.items ?? []).map(r => ({
      value: `release:${r.fingerprint}`,
      label: `${r.release.label} · ${environmentDeployment(r.fingerprint) ? environment : `no ${environment} deployment selected`}`,
    })),
  ];
  const [catalogRecord, model, reviewState, deploymentStatus] = revision ? await Promise.all([
    readProject<{ catalog: Catalog }>(projectId, `/model-revisions/${encodeURIComponent(revision)}/catalog`),
    readProject<{ authority: Authority; selected: boolean }>(projectId, `/model-revisions/${encodeURIComponent(revision)}`),
    readProject<ClaimReviewState>(projectId, `/model-revisions/${encodeURIComponent(revision)}/claim-reviews`),
    subject ? readProject<SubjectState>(projectId, `/model-revisions/${encodeURIComponent(revision)}/subjects/${encodeURIComponent(subject)}/state`) : Promise.resolve(null),
  ]) : [null, null, null, null];
  const catalog = catalogRecord?.catalog;
  const definitions = model?.authority.claims ?? [];
  const claimId = typeof query.claim === 'string' ? query.claim : undefined;
  const claim = definitions.find(c => c.id === claimId);
  const catalogModules = catalog?.modules ?? [];
  const unassigned = definitions.filter(c => !catalogModules.some(m => m.claims.includes(c.id)));
  const modules = [...catalogModules, ...(unassigned.length ? [{ id: '__unassigned', title: 'Unassigned Claims', claims: unassigned.map(c => c.id) }] : [])];
  const moduleId = claim ? modules.find(m => m.claims.includes(claim.id))?.id : typeof query.module === 'string' ? query.module : undefined;
  const module = modules.find(m => m.id === moduleId);
  const title = (id: string) => catalog?.claims.find(c => c.id === id)?.title ?? id;
  const values = new URLSearchParams({ environment });
  if (release) values.set("release", release.fingerprint);
  if (context || deployment || release) values.set('context', deployment ? `deployment:${deployment.deployment.id}` : contextValue);
  else if (revision) values.set('revision', revision);
  const base = `${root}/model?${values}`;
  const snapshot = claim && revision ? await readProject<Snapshot>(projectId, `/model-revisions/${encodeURIComponent(revision)}/snapshot`) : null;
  const state = deploymentStatus?.claims.find(c => c.claim === claim?.id);
  const review = reviewState?.claims.find(c => c.claim === claim?.id);
  const status = claimStatus(state?.status ?? 'unresolved');
  const systemName = deployment ? `${deployment.deployment.environment} · ${deployment.deployment.id}` : release ? `No ${environment} deployment selected · definitions only` : 'Definitions only';
  const modelName = release?.release.label ?? (model?.selected ? 'Accepted baseline' : 'Selected model');
  function descendants(id: string, trail: string[] = []): string[] {
    if (trail.includes(id)) return [];
    return [id, ...modules.filter(m => m.parent === id).flatMap(m => descendants(m.id,[...trail,id]))];
  }
  const visibleModules = module ? descendants(module.id) : modules.map(m => m.id);
  const visibleClaims = definitions.filter(c => !module || modules.some(m => visibleModules.includes(m.id) && m.claims.includes(c.id)));
  const history = snapshot?.reviews.filter(r => r.claim === claim?.id && (r.kind !== 'claim-health' || r.subject_fingerprint === subject)) ?? [];
  const claimRow = (id: string) => {
    const assessment = deploymentStatus?.claims.find(c => c.claim === id);
    const status = claimStatus(assessment?.status ?? 'unresolved');
    const review = reviewState?.claims.find(c => c.claim === id);
    return <Link className="modelClaimRow" href={`${base}&claim=${encodeURIComponent(id)}`} key={id} scroll={false}><span className={`modelHealthDot ${subject ? tones[status] : 'neutral'}`} /><div><h3>{title(id)}</h3><p>{assessment?.explanation.summary || (subject ? 'No assessment available for this system version.' : 'Claim definition · open a deployment context to inspect evidence.')}</p></div><div className="modelRowStatuses"><span className={`badge ${subject ? tones[status] : 'neutral'}`}>{subject ? labels[status] : 'Definition'}</span><small>Review: {review?.assessment ?? 'missing'}</small></div><span aria-hidden="true">›</span></Link>;
  };
  return <div className="modelPage">
    <header className="modelPageHeading"><div><p className="overline">MODEL WORKSPACE</p><h1>Model</h1></div><Link className="textLink" href={`${root}?environment=${encodeURIComponent(environment)}`}>← Environment overview</Link></header>
    <div className="modelContext"><div><strong>{modelName}</strong><span className="badge neutral">{release ? 'Release model' : model?.selected ? 'Accepted model' : 'Model definitions'}</span><span className="contextSystem">{systemName}</span></div><details className="contextChange"><summary>Change context</summary><form method="get"><input type="hidden" name="environment" value={environment} /><label>Browse context<select name="context" defaultValue={contextValue}>{contexts.map(option => <option key={option.value} value={option.value}>{option.label}</option>)}{!contexts.some(option => option.value === contextValue) && <option value={contextValue}>{modelName} · {systemName}</option>}</select></label><button type="submit">Apply</button></form></details></div>
    {(deployments?.next || releases?.next) && <p className="modelContextNotice">Showing the first 100 release and deployment contexts. Exact links remain available for older records.</p>}
    <ModelWorkspace
      initialPinned={pinned} selection={`${revision}:${moduleId}:${claimId}`}
      navigation={<ModelNavigation key={revision ?? 'empty'} modules={modules} selectedModule={module?.id} base={base} />}
      breadcrumb={<>{module ? <Link href={`${base}&module=${encodeURIComponent(module.id)}`}>{module.id === '__unassigned' ? module.title : module.id}</Link> : <Link href={base}>All modules</Link>}{claim && <><span>/</span><span>Claim</span></>}</>}
    >
      <section className="modelDetail" aria-label="Selected module or Claim">
        {claimId && !claim ? <div className="panelEmpty"><h2>Claim not found in this model</h2><Link className="textLink" href={base}>Browse modules →</Link></div> : claim ? <>
          <h2 className="claimStatement">{title(claim.id)}</h2>
          <section className="claimHealthSummary" aria-label="Claim Status"><div><span className={`badge ${subject ? tones[status] : 'neutral'}`}>{subject ? labels[status] : 'Definition'}</span><span className="modelDetailMuted">{systemName}</span></div><p>{state?.explanation.summary || (subject ? 'No Claim Status is available for this deployment.' : 'Browsing definitions. Open a deployment context to inspect Claim status and evidence.')}</p><span className="reviewSummary">{state?.evaluated_at_ms ? <>Evaluated {date(state.evaluated_at_ms)} · </> : null}Claim Review: <strong>{review?.assessment ?? 'missing'}</strong></span></section>
          <nav className="claimSections" aria-label="Claim sections"><a href="#claim-monitors">Monitoring</a><a href="#claim-deployment">Deployment</a><a href="#claim-implementation">Implementation</a><a href="#claim-statement">Statement</a><a href="#claim-review">Review</a><a href="#claim-history">History</a></nav>
          {state && state.explanation.reasons.length > 0 && <details className="statusReasons"><summary>Why this Claim is {labels[status].toLowerCase()}</summary><ul>{state.explanation.reasons.map((reason,i) => <li key={`${reason.code}:${i}`}><span>{reason.message}</span>{reason.check && <small>Check: {reason.check}</small>}</li>)}</ul></details>}
          <ClaimVerification bindings={claim.bindings} checks={model?.authority.checks ?? []} descriptions={catalog?.checks ?? []} cases={catalog?.cases ?? []} caseIds={claim.cases} state={state} evaluatedAt={state?.evaluated_at_ms ?? deploymentStatus?.evaluated_at_ms} hasDeployment={!!subject} projectRoot={root} />
          <section id="claim-statement" className="claimDetailSection"><h3>Statement</h3><p>{catalog?.claims.find(c => c.id === claim.id)?.statement ?? 'No statement available for this Claim.'}</p></section>
          <section id="claim-review" className="claimDetailSection"><div className="claimSectionHeading"><h3>Claim Review</h3><span className="badge neutral">{review?.assessment ?? 'missing'}</span></div><p>{typeof review?.record?.rationale === 'string' ? review.record.rationale : 'No review rationale is available.'}</p>{review?.diagnostics.map((d,i) => <p className="reviewDiagnostic" key={i}>{d}</p>)}<dl className="reviewFacts"><div><dt>Reviewer</dt><dd>{typeof review?.record?.reviewer === 'string' ? review.record.reviewer : 'Not recorded'}</dd></div><div><dt>Reviewed</dt><dd>{date(review?.record?.reviewed_at_ms)}</dd></div></dl><p className="modelDetailMuted">Review assesses the Claim, implementation and verification. Claim Status is computed from applicable evidence for the selected deployment.</p>{review?.record && <details><summary>Review record & provenance</summary><pre>{JSON.stringify(review.record,null,2)}</pre><p className="modelDetailMuted">Author provenance may be a publisher-transported assertion.</p></details>}</section>
          <section id="claim-history" className="claimDetailSection"><h3>Review & authored assessment history</h3><p className="modelDetailMuted">Historical authored assessments are retained records. They do not override computed deployment Claim Status.</p>{history.length ? history.map((r,i) => <details className="historyRecord" key={String(r.fingerprint ?? i)}><summary><span>{r.kind === 'claim-review' ? 'Claim Review' : 'Historical authored assessment (Claim Health)'}</span><span>{String(r.decision ?? r.conclusion ?? 'Recorded')}</span><time>{date(r.reviewed_at_ms)}</time></summary><pre>{JSON.stringify(r,null,2)}</pre></details>) : <p className="modelDetailMuted">No assessment history recorded for this selection.</p>}</section>
        </> : <>
          <p className="overline">{module ? 'MODULE' : 'MODEL STRUCTURE'}</p><h2 className="moduleDetailTitle">{module ? module.title : 'Modules & Claims'}</h2><p className="modelDetailMuted">{module ? 'Claims owned by this module and its nested modules.' : 'Select a module to explore its Claims, reviews and evidence.'}</p>
          {!module && <div className="moduleOverview">{modules.filter(m => !m.parent || !modules.some(parent => parent.id === m.parent)).map(m => <Link href={`${base}&module=${encodeURIComponent(m.id)}`} key={m.id}><span className="moduleIcon">◇</span><strong>{m.title}</strong><span>{definitions.filter(c => modules.some(n => descendants(m.id).includes(n.id) && n.claims.includes(c.id))).length} Claims</span><span aria-hidden="true">›</span></Link>)}</div>}
          {module ? <div className="moduleClaimList">{visibleClaims.map(c => claimRow(c.id))}{!visibleClaims.length && <p className="panelEmpty">No Claims in this module.</p>}</div> : !definitions.length && <div className="panelEmpty"><h3>{model ? 'No Claims in this model' : 'No model available'}</h3><p>Use Change context to select a published model.</p></div>}
        </>}
        <details className="modelTechnical"><summary>Technical details</summary>{revision && <CopyValue label="Model revision" value={revision} />}{subject && <CopyValue label="System fingerprint" value={subject} />}{claim && <CopyValue label="Claim ID" value={claim.id} />}{module && <CopyValue label="Module ID" value={module.id} />}{state && <details><summary>Complete Claim Status</summary><pre>{JSON.stringify(state,null,2)}</pre></details>}{claim && <details><summary>Definition gaps</summary><pre>{JSON.stringify(claim.gaps ?? [],null,2)}</pre></details>}</details>
      </section>
    </ModelWorkspace>
  </div>;
}
