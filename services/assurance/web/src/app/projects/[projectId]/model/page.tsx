import Link from 'next/link';
import type { ReactNode } from 'react';
import { requireViewer } from '@/lib/auth';
import { readProject, routeFingerprint, type Page, type ClaimReviewState, type SubjectState, type SubjectSummary } from '@/lib/assurance';
export const dynamic = 'force-dynamic';
type Catalog = { modules: { id: string; title: string; parent?: string; claims: string[] }[]; claims: { id: string; statement: string }[]; cases: { id: string; description: string }[]; checks: { id: string; description: string }[] };
type Authority = { fingerprint: string; claims: { id: string; cases: string[]; bindings: { check: string; case: string }[]; gaps: unknown }[]; checks: { id: string; fingerprint: string }[] };
type Revision = { revision: string; selected: boolean };
type ReleaseSummary = { fingerprint: string; release: { label: string; model_revision: string }; accepted: boolean };
export default async function ModelPage({ params, searchParams }: { params: Promise<{ projectId: string }>; searchParams: Promise<Record<string, string | string[] | undefined>> }) {
  const { projectId } = await params;
  await requireViewer(projectId);
  const query = await searchParams;
  const root = `/projects/${encodeURIComponent(projectId)}`;
  const revisions = await readProject<Page<Revision>>(projectId, '/model-revisions?limit=100');
  const selected = typeof query.revision === 'string' ? routeFingerprint(query.revision) : null;
  const revision = selected ?? revisions?.items.find(r => r.selected)?.revision;
  const subjects = await readProject<Page<SubjectSummary>>(projectId, '/subjects?limit=100');
  const subject = typeof query.subject === 'string' ? routeFingerprint(query.subject) : null;
  const releases = await readProject<Page<ReleaseSummary>>(projectId, '/releases?limit=100');
  const release = releases?.items.find(r => r.release.model_revision === revision);
  const publication = release ? await readProject<{ publication: { catalog?: Catalog } }>(projectId, `/releases/${encodeURIComponent(release.fingerprint)}`) : null;
  const catalog = publication?.publication.catalog;
  const model = revision ? await readProject<{ authority: Authority; selected: boolean }>(projectId, `/model-revisions/${encodeURIComponent(revision)}`) : null;
  const reviewState = revision ? await readProject<ClaimReviewState>(projectId, `/model-revisions/${encodeURIComponent(revision)}/claim-reviews`) : null;
  const health = revision && subject ? await readProject<SubjectState>(projectId, `/model-revisions/${encodeURIComponent(revision)}/subjects/${encodeURIComponent(subject)}/state`) : null;
  const snapshot = revision ? await readProject<{ reviews: Record<string, unknown>[]; bundles: Record<string, unknown>[] }>(projectId, `/model-revisions/${encodeURIComponent(revision)}/snapshot`) : null;
  const claimId = typeof query.claim === 'string' ? query.claim : null;
  const claim = model?.authority.claims.find(c => c.id === claimId);
  const modelLink = (extra = '') => `${root}/model?revision=${encodeURIComponent(revision ?? '')}${subject ? `&subject=${encodeURIComponent(subject)}` : ''}${extra}`;
  const claimBody = (id: string) => {
    const definition = model?.authority.claims.find(c => c.id === id);
    const review = reviewState?.claims.find(c => c.claim === id);
    const state = health?.claims.find(c => c.claim === id);
    return <article className="section" key={id}>
      <h2>{catalog?.claims.find(c => c.id === id)?.statement ?? id}</h2><code>{id}</code>
      <h3>Claim Review</h3><p><strong>{review?.assessment ?? 'missing'}</strong></p><p>{review?.diagnostics.join('; ')}</p>
      {review?.record && <details><summary>Decision, reasoning, recorded agent/model and exact dependencies</summary><pre>{JSON.stringify(review.record, null, 2)}</pre><p>Reviewer provenance may be a publisher-transported assertion. Authentication of the publisher does not authenticate the original author.</p></details>}
      <h3>Claim Health</h3><p><strong>{state?.status ?? 'unresolved'}</strong></p><p>{state?.diagnostics.join('; ') ?? (subject ? 'No assessment available for this exact selection.' : 'Select an exact source, artifact or deployment to view Health.')}</p>
      {state && <details><summary>Results, missing evidence, limitations and bindings</summary><pre>{JSON.stringify(state, null, 2)}</pre></details>}
      <details><summary>Review and Health assessment history</summary>{snapshot?.reviews.filter(r => r.claim === id && (r.kind !== 'claim-health' || r.subject_fingerprint === subject)).map(r => <section key={String(r.fingerprint)}><h4>{String(r.kind)} · {String(r.decision ?? r.conclusion)}</h4><pre>{JSON.stringify(r,null,2)}</pre></section>)}</details>
      <details><summary>Runs for the exact selected system version</summary><pre>{JSON.stringify(snapshot?.bundles.filter(b => b.subject_fingerprint === subject) ?? [],null,2)}</pre></details>
      <h3>Cases and contributing Checks</h3>{definition?.cases.map(id => <section key={id}><h4>{catalog?.cases.find(c => c.id === id)?.description ?? id}</h4><ul>{definition.bindings.filter(b => b.case === id).map(b => <li key={`${b.check}:${id}`}><strong>{b.check}</strong> {catalog?.checks.find(c => c.id === b.check)?.description}</li>)}</ul></section>)}
      <details><summary>Definition gaps</summary><pre>{JSON.stringify(definition?.gaps ?? [], null, 2)}</pre></details>
    </article>;
  };
  const moduleTree = (parent?: string, trail: string[] = []): ReactNode => <ul>{catalog?.modules.filter(m => m.parent === parent).map(m => <li key={m.id}><strong>{m.title || m.id}</strong> <code>{m.id}</code><ul>{m.claims.map(id => <li key={id}><Link href={modelLink(`&claim=${encodeURIComponent(id)}`)}>{catalog?.claims.find(c => c.id === id)?.statement ?? id}</Link></li>)}</ul>{!trail.includes(m.id) && moduleTree(m.id, [...trail, m.id])}</li>)}</ul>;
  return <div className="shell"><Link href={root}>← Project</Link><h1>Model, Claims and Checks</h1>
    <form method="get"><label>Model revision <select name="revision" defaultValue={revision}>{revisions?.items.map(r => <option key={r.revision} value={r.revision}>{r.selected ? 'Accepted / selected' : 'Candidate'} · {r.revision.slice(0, 19)}</option>)}</select></label>{' '}<label>Exact system version <select name="subject" defaultValue={subject ?? ''}><option value="">Select to view Claim Health</option>{subjects?.items.map(s => <option key={s.fingerprint} value={s.fingerprint}>{JSON.stringify(s.subject).slice(0, 100)} · {s.fingerprint.slice(0, 19)}</option>)}</select></label>{' '}<button type="submit">View</button></form>
    <p>{model?.selected ? 'Accepted / explicitly selected model' : 'Candidate model'} · <code>{revision ?? 'No model selected'}</code></p>
    <p>Review acceptance and observed Health are separate. Results are never transferred between source, build artifacts and deployments merely because they share a model.</p>
    {catalog ? moduleTree() : <p>Module catalog unavailable for this revision; Claim identities remain available below.</p>}
    {claim ? claimBody(claim.id) : model?.authority.claims.map(c => <p key={c.id}><Link href={modelLink(`&claim=${encodeURIComponent(c.id)}`)}>{catalog?.claims.find(d => d.id === c.id)?.statement ?? c.id}</Link></p>)}
    {(revisions?.next || subjects?.next || releases?.next) && <p className="notice">This selector shows the first 100 records. Use an exact revision/Subject query or project pagination for older records.</p>}
  </div>;
}
