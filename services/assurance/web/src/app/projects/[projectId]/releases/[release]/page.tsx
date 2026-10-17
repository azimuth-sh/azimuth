import Link from 'next/link';
import { notFound } from 'next/navigation';
import { readProject, routeFingerprint } from '@/lib/assurance';
import { requireViewer } from '@/lib/auth';
export const dynamic = 'force-dynamic';
type Identity = Record<string, unknown>;
type Release = { fingerprint: string; label: string; baseline_model_revision: string; model_revision: string; changes: Identity[]; sources: Identity[]; builds: Identity[]; artifacts: Identity[]; deployments: Identity[] };
export default async function ReleasePage({params}: {params: Promise<{projectId:string;release:string}>}) {
  const { projectId, release: segment } = await params;
  await requireViewer(projectId);
  const fingerprint = routeFingerprint(segment);
  if (!fingerprint) notFound();
  const record = await readProject<{publication:{release:Release;evidence:Identity[]};submitted_by:string;submitted_at:string;acceptance:Identity|null;deployments:Identity[]}>(projectId, `/releases/${encodeURIComponent(fingerprint)}`);
  if (!record) notFound();
  const release = record.publication.release;
  const root = `/projects/${encodeURIComponent(projectId)}`;
  const section = (title: string, values: Identity[]) => <section className="section"><h2>{title}</h2>{values.length ? values.map((v,i) => <details key={i}><summary>{String(v.label ?? v.id ?? v.repository ?? v.digest ?? `Record ${i+1}`)}</summary><pre>{JSON.stringify(v,null,2)}</pre></details>) : <p>No records published.</p>}</section>;
  return <div className="shell"><Link href={root}>← Project</Link><h1>{release.label}</h1><p><strong>{record.acceptance ? 'Accepted composed release' : 'Candidate — not accepted'}</strong></p><code>{release.fingerprint}</code>
    <p>Authenticated publisher: {record.submitted_by} · received {record.submitted_at}</p>
    <p>Baseline <code>{release.baseline_model_revision}</code></p><p>Composed model <Link href={`${root}/model?revision=${encodeURIComponent(release.model_revision)}`}>{release.model_revision}</Link></p>
    <p>Publication does not authorize deployment or accept this model. Deployment records belong to the project/environment and can include several Changes.</p>
    {section('Included Change revisions',release.changes)}{section('Source snapshots',release.sources)}{section('Build provenance',release.builds)}{section('Immutable artifacts',release.artifacts)}{section('Recorded deployments',record.deployments)}
    {record.deployments.filter(d => typeof d.subject_fingerprint === 'string').map(d => <p key={String(d.id)}><Link href={`${root}/model?revision=${encodeURIComponent(release.model_revision)}&subject=${encodeURIComponent(String(d.subject_fingerprint))}`}>Claim Health for {String(d.environment)} · {String(d.id)}</Link></p>)}
    <section className="section"><h2>Evidence availability</h2>{record.publication.evidence.map((e,i) => <p key={i}><code>{String(e.digest)}</code> · {String(e.media_type)} · <strong>{String(e.availability)}</strong></p>)}<p>Unavailable reports limit inspection. Missing required records or measurements cannot establish supported Claim Health.</p></section>
    {record.acceptance && <details><summary>Owner acceptance decision</summary><pre>{JSON.stringify(record.acceptance,null,2)}</pre></details>}
  </div>;
}
