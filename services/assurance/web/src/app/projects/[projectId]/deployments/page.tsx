import Link from 'next/link';
import { cursor, readProject, type Page } from '@/lib/assurance';
import { requireViewer } from '@/lib/auth';
import { selectedEnvironment } from '@/lib/environment';
import { Identifier } from '@/components/identifier';
import { formatUtc } from '@/lib/time';

export const dynamic = 'force-dynamic';
type Deployment = {
  deployment: { id: string; environment: string; verification_target_fingerprint?: string };
  release_fingerprint: string;
  submitted_at: string;
};
export default async function DeploymentsPage({ params, searchParams }: {
  params: Promise<{ projectId: string }>;
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}) {
  const { projectId } = await params;
  await requireViewer(projectId);
  const query = await searchParams;
  const environment = await selectedEnvironment(projectId, query.environment);
  const [deployments, releases, current] = await Promise.all([
    readProject<Page<Deployment>>(projectId, `/deployments?limit=100&after=${encodeURIComponent(cursor(query.after))}`),
    readProject<Page<{ fingerprint: string; release: { label: string; model_revision: string } }>>(projectId, '/releases?limit=100'),
    readProject<Deployment>(projectId, `/environments/${encodeURIComponent(environment)}/deployment`),
  ]);
  const root = `/projects/${encodeURIComponent(projectId)}`;
  const release = (fingerprint: string) => releases?.items.find(item => item.fingerprint === fingerprint)?.release;
  const items = (deployments?.items ?? [])
    .filter(item => item.deployment.environment === environment)
    .sort((a, b) => Date.parse(b.submitted_at) - Date.parse(a.submitted_at));
  return <div className="dashboard">
    <header className="dashboardHeading"><div><p className="overline">DEPLOYMENTS</p><h1>Deployments</h1><p>Each time a release was observed running in {environment}, newest first.</p></div></header>
    <section className="dashboardPanel">
      <header className="panelHeading"><div><h2>{environment}</h2></div><span className="panelCount">{items.length}</span></header>
      {items.map(item => {
        const label = release(item.release_fingerprint);
        const target = item.deployment.verification_target_fingerprint;
        const isCurrent = current?.deployment.id === item.deployment.id;
        return <div className="releaseCompactRow" key={item.deployment.id}>
          <div>
            <strong>{label ? <Link className="textLink" href={`${root}/releases/${encodeURIComponent(item.release_fingerprint)}?environment=${encodeURIComponent(environment)}`}>{label.label}</Link> : 'Release unavailable'}</strong>
            <small><time dateTime={item.submitted_at}>{formatUtc(item.submitted_at)}</time> · <Identifier value={item.deployment.id} /></small>
          </div>
          {isCurrent && <span className="badge good">Current</span>}
          {label && target && <Link className="textLink" href={`${root}/model?environment=${encodeURIComponent(environment)}&revision=${encodeURIComponent(label.model_revision)}&verification-target=${encodeURIComponent(target)}`}>Requirements ↗</Link>}
        </div>;
      })}
      {!items.length && <p className="tableEmpty">No {environment} deployments recorded{deployments?.next ? ' on this page' : ''}.</p>}
    </section>
    {deployments?.next && <Link className="textLink" href={`${root}/deployments?environment=${encodeURIComponent(environment)}&after=${encodeURIComponent(deployments.next)}`}>Next deployments →</Link>}
  </div>;
}
