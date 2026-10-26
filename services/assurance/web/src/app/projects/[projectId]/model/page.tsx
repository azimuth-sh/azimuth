import { selectedEnvironment } from "@/lib/environment";
import Link from 'next/link';
import { cookies } from 'next/headers';
import { notFound } from 'next/navigation';
import { requireViewer } from '@/lib/auth';
import { readProject, routeFingerprint, type Page, type RequirementReviewEntry, type RequirementReviewState, type VerificationTargetState, type DeploymentRequirementStatuses, type DeploymentCheckPolicy, type ModelRevision, type ReleaseSummary, type ReleaseDetail, type DeploymentReference as Deployment, type RequirementWorkHistory } from '@/lib/assurance';
import { requirementStatus } from '@/lib/production';
import { ModelNavigation } from '@/components/model-navigation';
import { ModelWorkspace } from '@/components/model-workspace';
import { CopyValue } from '@/components/copy-value';
import { RequirementVerification } from '@/components/requirement-verification';
import { Disclosure } from '@/components/disclosure';
import { Markdown } from '@/components/markdown';
import { LocalTime } from '@/components/local-time';
import { WorkHistory, ReviewAttempt, reviewAttempt } from '@/components/work-history';
import { shortIdentifier } from '@/lib/identifier';
import { CodeDrift, ReviewChips, reviewKindNames, reviewLabel, reviewTone, type ReviewKind } from '@/components/review-dimensions';

export const dynamic = 'force-dynamic';
type Catalog = {
  modules: { id: string; title: string; parent?: string; requirements: string[] }[];
  requirements: { id: string; title?: string; statement: string }[];
  scenarios: { id: string; description: string }[];
  checks: { id: string; description: string }[];
};
type Authority = {
  fingerprint: string;
  requirements: { id: string; scenarios: string[]; bindings: { check: string; scenario: string }[]; gaps: unknown }[];
  checks: { id: string; fingerprint: string; deployment?: DeploymentCheckPolicy | null }[];
};
type Version = { kind: 'environment' | 'release' | 'main' | 'revision'; label: string };
type Snapshot = { reviews: Record<string, unknown>[] };
const labels = { supported: 'Satisfied', violated: 'Not satisfied', unresolved: 'Unresolved', 'not-applicable': 'Not applicable here' };
const tones = { supported: 'good', violated: 'bad', unresolved: 'warn', 'not-applicable': 'neutral' };
function plural(count: number, noun: string) {
  return `${count} ${noun}${count === 1 ? '' : 's'}`;
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
  if (['revision', 'verification-target', 'module', 'requirement', 'version'].some(key => Array.isArray(query[key]))) notFound();
  const requestedRevision = typeof query.revision === 'string' ? routeFingerprint(query.revision) : null;
  const requestedTarget = query['verification-target'];
  let verificationTarget = typeof requestedTarget === 'string' ? routeFingerprint(requestedTarget) : null;
  if (query.revision && !requestedRevision || requestedTarget && !verificationTarget) notFound();
  const requestedVersion = typeof query.version === 'string' ? query.version : undefined;
  const root = `/projects/${encodeURIComponent(projectId)}`;
  const [revisions, deployments, releases, currentEnvironment] = await Promise.all([
    readProject<Page<ModelRevision>>(projectId, '/model-revisions?limit=100'),
    readProject<Page<Deployment>>(projectId, '/deployments?limit=100'),
    readProject<Page<ReleaseSummary>>(projectId, '/releases?limit=100'),
    readProject<Deployment>(projectId, `/environments/${encodeURIComponent(environment)}/deployment`),
  ]);
  function environmentDeployment(fingerprint: string) {
    if (currentEnvironment?.release_fingerprint === fingerprint) return currentEnvironment;
    const recorded = deployments?.items.filter(d => d.release_fingerprint === fingerprint && d.deployment.environment === environment) ?? [];
    return !deployments?.next && recorded.length === 1 ? recorded[0] : null;
  }
  const defaultRevision = revisions?.items.find(r => r.selected)?.revision;
  let revision: string | undefined;
  let deployment: Deployment | null = null;
  let release: ReleaseSummary | undefined;
  let version: Version;
  let versionParam: string | null = null;
  async function loadRelease(fingerprint: string) {
    const record = await readProject<ReleaseDetail>(projectId, `/releases/${encodeURIComponent(fingerprint)}/summary`);
    if (!record) notFound();
    release = { fingerprint, release: record.publication.release };
    revision = release.release.model_revision;
    deployment = environmentDeployment(fingerprint);
    verificationTarget = deployment?.deployment.verification_target_fingerprint ?? null;
  }
  if (requestedRevision) {
    revision = requestedRevision;
    version = { kind: 'revision', label: 'Selected model' };
    if (verificationTarget) {
      deployment = deployments?.items.find(d => d.deployment.verification_target_fingerprint === verificationTarget) ?? null;
      if (!deployment || deployment.deployment.environment !== environment) notFound();
      release = releases?.items.find(r => r.fingerprint === deployment?.release_fingerprint);
    } else {
      release = releases?.items.find(r => r.release.model_revision === requestedRevision && environmentDeployment(r.fingerprint));
      if (release) {
        deployment = environmentDeployment(release.fingerprint);
        verificationTarget = deployment?.deployment.verification_target_fingerprint ?? null;
      }
    }
  } else if (requestedVersion === 'main') {
    revision = defaultRevision;
    version = { kind: 'main', label: 'Main default model' };
    versionParam = 'main';
  } else if (requestedVersion?.startsWith('release:')) {
    const fingerprint = routeFingerprint(requestedVersion.slice(8));
    if (!fingerprint) notFound();
    await loadRelease(fingerprint);
    version = { kind: 'release', label: release!.release.label };
    versionParam = requestedVersion;
  } else if (requestedVersion) {
    notFound();
  } else if (currentEnvironment) {
    await loadRelease(currentEnvironment.release_fingerprint);
    version = { kind: 'environment', label: release!.release.label };
  } else {
    revision = defaultRevision;
    version = { kind: 'main', label: 'Main default model' };
  }
  const otherVersions = [
    ...(version.kind === 'main' ? [] : [{ href: `${root}/model?${new URLSearchParams({ environment, version: 'main' })}`, label: 'Main default model', detail: 'definitions only' }]),
    ...(releases?.items ?? []).filter(r => r.fingerprint !== release?.fingerprint).map(r => ({
      href: `${root}/model?${new URLSearchParams({ environment, version: `release:${r.fingerprint}` })}`,
      label: r.release.label,
      detail: environmentDeployment(r.fingerprint) ? `deployed in ${environment}` : `not deployed in ${environment}`,
    })),
  ];
  const [catalogRecord, model, reviewState, deploymentStatus] = revision ? await Promise.all([
    readProject<{ catalog: Catalog }>(projectId, `/model-revisions/${encodeURIComponent(revision)}/catalog`),
    readProject<{ authority: Authority; selected: boolean }>(projectId, `/model-revisions/${encodeURIComponent(revision)}`),
    readProject<RequirementReviewState>(projectId, `/model-revisions/${encodeURIComponent(revision)}/requirement-reviews`),
    deployment && verificationTarget ? readProject<DeploymentRequirementStatuses>(projectId, `/deployments/${encodeURIComponent(deployment.deployment.id)}/requirements`) : Promise.resolve(null),
  ]) : [null, null, null, null];
  const catalog = catalogRecord?.catalog;
  const definitions = model?.authority.requirements ?? [];
  const requirementId = typeof query.requirement === 'string' ? query.requirement : undefined;
  const requirement = definitions.find(c => c.id === requirementId);
  const catalogModules = catalog?.modules ?? [];
  const unassigned = definitions.filter(c => !catalogModules.some(m => m.requirements.includes(c.id)));
  const modules = [...catalogModules, ...(unassigned.length ? [{ id: '__unassigned', title: 'Unassigned Requirements', requirements: unassigned.map(c => c.id) }] : [])];
  const moduleId = requirement ? modules.find(m => m.requirements.includes(requirement.id))?.id : typeof query.module === 'string' ? query.module : undefined;
  const module = modules.find(m => m.id === moduleId);
  const title = (id: string) => catalog?.requirements.find(c => c.id === id)?.title ?? id;
  const values = new URLSearchParams({ environment });
  if (versionParam) values.set('version', versionParam);
  else if (requestedRevision) {
    values.set('revision', requestedRevision);
    if (verificationTarget) values.set('verification-target', verificationTarget);
  }
  const base = `${root}/model?${values}`;
  const historyRequested = query.history === '1';
  // Work history is advisory; an unavailable ledger must not hide Requirement Status, so read failures become `undefined`.
  const assessmentFresh = deploymentStatus?.assessment?.state === 'ready' && deploymentStatus.assessment.valid_until_ms !== null && deploymentStatus.assessment.valid_until_ms > Date.now();
  // Only the selected Requirement's full assessment is read; the other rows need just their status line.
  const [snapshot, workHistory, requirementDetail] = requirement ? await Promise.all([
    revision && historyRequested ? readProject<Snapshot>(projectId, `/model-revisions/${encodeURIComponent(revision)}/snapshot`) : Promise.resolve(null),
    readProject<RequirementWorkHistory>(projectId, `/provenance/requirements/${encodeURIComponent(requirement.id)}`).catch(() => undefined),
    assessmentFresh && deployment ? readProject<VerificationTargetState>(projectId, `/deployments/${encodeURIComponent(deployment.deployment.id)}/requirements/${encodeURIComponent(requirement.id)}`) : Promise.resolve(null),
  ]) : [null, null, null];
  const state = assessmentFresh ? requirementDetail?.requirements[0] : undefined;
  const reviewFresh = reviewState?.assessment?.state === 'ready';
  const reviewFallback = reviewFresh ? 'missing' : undefined;
  const reviewEntry = (id: string | undefined, kind: ReviewKind) => reviewFresh ? reviewState?.requirements.find(c => c.requirement === id && c.kind === kind) : undefined;
  const verificationReview = reviewEntry(requirement?.id, 'verification-review');
  const realizationReview = reviewEntry(requirement?.id, 'realization-review');
  const status = requirementStatus(state?.status ?? 'unresolved');
  const systemName = deployment ? deployment.deployment.environment : release ? `Not deployed in ${environment} · definitions only` : `No ${environment} deployment · definitions only`;
  const modelName = version.label;
  function descendants(id: string, trail: string[] = []): string[] {
    if (trail.includes(id)) return [];
    return [id, ...modules.filter(m => m.parent === id).flatMap(m => descendants(m.id,[...trail,id]))];
  }
  const visibleModules = module ? descendants(module.id) : modules.map(m => m.id);
  const visibleRequirements = definitions.filter(c => !module || modules.some(m => visibleModules.includes(m.id) && m.requirements.includes(c.id)));
  const history = snapshot?.reviews.filter(r => r.requirement === requirement?.id) ?? [];
  const reviewSection = (kind: ReviewKind, review: RequirementReviewEntry | undefined) => {
    const status = review?.assessment ?? reviewFallback;
    const reviewer = reviewAttempt(workHistory, kind, review?.record?.fingerprint);
    const record = review?.record;
    return <section id={kind} className="requirementDetailSection disclosureSection"><Disclosure summary={reviewKindNames[kind]} meta={<><span className={`badge ${reviewTone(status)}`}>{reviewLabel(status)}</span>{typeof record?.reviewer === 'string' && <span>{record.reviewer}</span>}{record?.reviewed_at_ms !== undefined && <LocalTime value={typeof record.reviewed_at_ms === 'number' ? record.reviewed_at_ms : undefined} fallback="Date not recorded" />}</>}>
      {status === 'not-declared' ? <p className="modelDetailMuted">This Requirement does not declare <code>{kind === 'verification-review' ? 'Verification:' : 'Realization:'}</code>, so no {reviewKindNames[kind].toLowerCase()} is owed.</p> : typeof record?.rationale === 'string' ? <Markdown source={record.rationale} /> : <p className="modelDetailMuted">No review rationale is available.</p>}
      {kind === 'realization-review' && <CodeDrift drift={review?.code_drift} commit={record?.reviewed_commit} />}
      {review?.diagnostics.map((d,i) => <p className="reviewDiagnostic" key={i}>{d}</p>)}
      {record && <dl className="factGrid">
        <div><dt>Decision</dt><dd>{record.decision ?? 'not recorded'}</dd></div>
        <div><dt>Reassesses</dt><dd>{record.reassesses ? <code title={record.reassesses}>{shortIdentifier(record.reassesses)}</code> : 'none'}</dd></div>
        <div><dt>Reviewed dependencies</dt><dd>{record.dependencies_fingerprint ? <code title={record.dependencies_fingerprint}>{shortIdentifier(record.dependencies_fingerprint)}</code> : 'not recorded'}</dd></div>
        {kind === 'verification-review' ? <div><dt>Challenge basis</dt><dd>{record.challenge_basis?.length ? `${record.challenge_basis.length} Verification Challenge ${record.challenge_basis.length === 1 ? 'Result' : 'Results'}` : 'none'}</dd></div>
          : <div><dt>Reviewed commit</dt><dd>{record.reviewed_commit ? <code title={record.reviewed_commit}>{shortIdentifier(record.reviewed_commit)}</code> : 'not recorded'}</dd></div>}
        <div><dt>Issues</dt><dd>{record.issues?.length ? <ul className="reviewArtifacts">{record.issues.map(issue => <li key={issue.id}><code>{issue.id}</code> {issue.blocking ? 'blocking · ' : ''}{issue.reason}</li>)}</ul> : 'none'}</dd></div>
        {record.dispositions?.length ? <div><dt>Dispositions</dt><dd><ul className="reviewArtifacts">{record.dispositions.map(disposition => <li key={disposition.issue}><code title={disposition.issue}>{shortIdentifier(disposition.issue)}</code> {disposition.outcome}</li>)}</ul></dd></div> : null}
        <div><dt>Reviewed artifacts</dt><dd>{record.artifacts?.length ? <ul className="reviewArtifacts">{record.artifacts.map(artifact => <li key={`${artifact.id}:${artifact.digest}`}><code title={artifact.digest}>{artifact.id}</code> {artifact.locator}</li>)}</ul> : 'none'}</dd></div>
      </dl>}
      {reviewer && <ReviewAttempt attempt={reviewer} />}
      <p className="modelDetailMuted">{kind === 'verification-review' ? 'A Verification Review assesses whether the Checks credibly verify the Scenarios. Requirement Status is computed from applicable evidence for the selected deployment and requires an accepted current Verification Review.' : 'A Realization Review assesses whether the code at the realization sites does what the Scenarios say. It is an opinion about a code snapshot: it never makes a Requirement satisfied, and code changed since the reviewed commit is information, not staleness.'}</p>
      {record && <Disclosure summary="Review record"><pre className="rawRecord">{JSON.stringify(record,null,2)}</pre></Disclosure>}
    </Disclosure></section>;
  };
  const requirementRow = (id: string) => {
    const assessment = assessmentFresh ? deploymentStatus?.requirements.find(c => c.requirement === id) : undefined;
    const status = requirementStatus(assessment?.status ?? 'unresolved');
    const verification = reviewEntry(id, 'verification-review')?.assessment ?? reviewFallback;
    const realization = reviewEntry(id, 'realization-review')?.assessment ?? reviewFallback;
    return <Link className="modelRequirementRow" href={`${base}&requirement=${encodeURIComponent(id)}`} key={id} scroll={false}><span className={`modelStatusDot ${verificationTarget ? tones[status] : 'neutral'}`} /><div><h3>{title(id)}</h3><p>{assessment?.summary || (verificationTarget ? 'No assessment available for this system version.' : 'Requirement definition · open a version running in an environment to inspect evidence.')}</p></div><div className="modelRowStatuses"><span className={`badge ${verificationTarget ? tones[status] : 'neutral'}`}>{verificationTarget ? labels[status] : 'Definition'}</span><ReviewChips verification={verification} realization={realization} /></div><span aria-hidden="true">›</span></Link>;
  };
  return <div className="modelPage">
    <header className="modelPageHeading"><div><p className="overline">MODEL WORKSPACE</p><h1>Model</h1></div><Link className="textLink" href={`${root}?environment=${encodeURIComponent(environment)}`}>← Environment overview</Link></header>
    <div className="modelContext"><div><strong>{modelName}</strong><span className="badge neutral">{version.kind === 'environment' ? 'Environment release model' : version.kind === 'release' ? 'Release model' : version.kind === 'main' ? 'Default model on main' : model?.selected ? 'Default model on main' : 'Model definitions'}</span><span className="contextSystem">{systemName}</span></div><details className="contextChange"><summary>Other versions</summary><ul>{otherVersions.map(option => <li key={option.href}><Link href={option.href}>{option.label}</Link> <small>· {option.detail}</small></li>)}{!otherVersions.length && <li>No other versions published.</li>}</ul></details></div>
    {deploymentStatus?.assessment && !assessmentFresh && <section className="dashboardPanel" role="status"><h2>{deploymentStatus.assessment.state === 'expired' ? 'Assessment overdue' : 'Assessment updating'}</h2><p>Evidence is being reassessed. Previous conclusions are not shown as current.</p></section>}
    {(deployments?.next || releases?.next) && <p className="modelContextNotice">Showing the first 100 release and deployment contexts. Exact links remain available for older records.</p>}
    <ModelWorkspace
      initialPinned={pinned} selection={`${revision}:${moduleId}:${requirementId}`}
      navigation={<ModelNavigation key={revision ?? 'empty'} modules={modules} selectedModule={module?.id} base={base} />}
      breadcrumb={<>{module ? <Link href={`${base}&module=${encodeURIComponent(module.id)}`}>{module.id === '__unassigned' ? module.title : module.id}</Link> : <Link href={base}>All modules</Link>}{requirement && <><span>/</span><span>Requirement</span></>}</>}
    >
      <section className="modelDetail" aria-label="Selected module or Requirement">
        {requirementId && !requirement ? <div className="panelEmpty"><h2>Requirement not found in this model</h2><Link className="textLink" href={base}>Browse modules →</Link></div> : requirement ? <>
          <h2 className="requirementStatement">{title(requirement.id)}</h2>
          <section className="requirementStatusSummary" aria-label="Requirement Status"><div><span className={`badge ${verificationTarget ? tones[status] : 'neutral'}`}>{verificationTarget ? labels[status] : 'Definition'}</span><span className="modelDetailMuted">{systemName}</span></div><p>{state?.explanation.summary || (verificationTarget ? 'No Requirement Status is available for this deployment.' : 'Browsing definitions. Open a version running in this environment to inspect Requirement Status and evidence.')}</p><span className="statusLine">{plural(new Set(requirement.bindings.map(b => b.check)).size, 'Check')} · {plural(requirement.scenarios.length, 'Scenario')}</span><span className="reviewSummary">{state?.evaluated_at_ms ? <>Evaluated <LocalTime value={state.evaluated_at_ms} /> · </> : null}Verification review: <strong>{reviewLabel(verificationReview?.assessment ?? reviewFallback)}</strong> · Realization review: <strong>{reviewLabel(realizationReview?.assessment ?? reviewFallback)}</strong></span></section>
          <nav className="requirementSections" aria-label="Requirement sections"><a href="#requirement-monitors">Monitoring</a><a href="#requirement-deployment">Deployment</a><a href="#requirement-implementation">Implementation</a><a href="#requirement-statement">Statement</a><a href="#verification-review">Reviews</a><a href="#requirement-work-history">Work history</a><Link prefetch={false} href={`${base}&requirement=${encodeURIComponent(requirement.id)}&history=1#requirement-history`}>History</Link></nav>
          {state && state.explanation.reasons.length > 0 && <details className="statusReasons"><summary>Why this Requirement is {labels[status].toLowerCase()}</summary><ul>{state.explanation.reasons.map((reason,i) => <li key={`${reason.code}:${i}`}><span>{reason.message}</span>{reason.check && <small>Check: {reason.check}</small>}</li>)}</ul></details>}
          <RequirementVerification bindings={requirement.bindings} checks={model?.authority.checks ?? []} descriptions={catalog?.checks ?? []} scenarios={catalog?.scenarios ?? []} scenarioIds={requirement.scenarios} state={state} evaluatedAt={state?.evaluated_at_ms ?? requirementDetail?.evaluated_at_ms} hasDeployment={!!verificationTarget} projectRoot={root} />
          <section id="requirement-statement" className="requirementDetailSection disclosureSection"><Disclosure open summary="Statement">{catalog?.requirements.find(c => c.id === requirement.id)?.statement ? <Markdown source={catalog.requirements.find(c => c.id === requirement.id)!.statement} /> : <p className="modelDetailMuted">No statement available for this Requirement.</p>}</Disclosure></section>
          {reviewSection('verification-review', verificationReview)}
          {reviewSection('realization-review', realizationReview)}
          <WorkHistory history={workHistory} />
          <section id="requirement-history" className="requirementDetailSection"><h3>Review history</h3><p className="modelDetailMuted">Retained Verification and Realization Reviews for this model. They do not override computed deployment Requirement Status.</p>{history.length ? history.map((r,i) => <Disclosure key={String(r.fingerprint ?? i)} summary={reviewKindNames[r.kind as ReviewKind] ?? 'Review'} meta={<><span>{String(r.decision ?? 'Recorded')}</span><LocalTime value={typeof r.reviewed_at_ms === 'number' ? r.reviewed_at_ms : undefined} fallback="Date not recorded" /></>}><pre className="rawRecord">{JSON.stringify(r,null,2)}</pre></Disclosure>) : historyRequested ? <p className="modelDetailMuted">No review history recorded for this selection.</p> : <Link className="textLink" prefetch={false} href={`${base}&requirement=${encodeURIComponent(requirement.id)}&history=1#requirement-history`}>Load review history →</Link>}</section>
        </> : <>
          <p className="overline">{module ? 'MODULE' : 'MODEL STRUCTURE'}</p><h2 className="moduleDetailTitle">{module ? module.title : 'Modules & Requirements'}</h2><p className="modelDetailMuted">{module ? 'Requirements owned by this module and its nested modules.' : 'Select a module to explore its Requirements, reviews and evidence.'}</p>
          {!module && <div className="moduleOverview">{modules.filter(m => !m.parent || !modules.some(parent => parent.id === m.parent)).map(m => <Link href={`${base}&module=${encodeURIComponent(m.id)}`} key={m.id}><span className="moduleIcon">◇</span><strong>{m.title}</strong><span>{definitions.filter(c => modules.some(n => descendants(m.id).includes(n.id) && n.requirements.includes(c.id))).length} Requirements</span><span aria-hidden="true">›</span></Link>)}</div>}
          {module ? <div className="moduleRequirementList">{visibleRequirements.map(c => requirementRow(c.id))}{!visibleRequirements.length && <p className="panelEmpty">No Requirements in this module.</p>}</div> : !definitions.length && <div className="panelEmpty"><h3>{model ? 'No Requirements in this model' : 'No model available'}</h3><p>Open Other versions to select a published model.</p></div>}
        </>}
        <details className="modelTechnical"><summary>Technical details</summary>{revision && <CopyValue label="Model revision" value={revision} />}{deployment && <CopyValue label="Deployment" value={deployment.deployment.id} />}{verificationTarget && <CopyValue label="Verification Target fingerprint" value={verificationTarget} />}{requirement && <CopyValue label="Requirement ID" value={requirement.id} />}{module && <CopyValue label="Module ID" value={module.id} />}{state && <details><summary>Complete Requirement Status</summary><pre>{JSON.stringify(state,null,2)}</pre></details>}{requirement && <details><summary>Definition gaps</summary><pre>{JSON.stringify(requirement.gaps ?? [],null,2)}</pre></details>}</details>
      </section>
    </ModelWorkspace>
  </div>;
}
