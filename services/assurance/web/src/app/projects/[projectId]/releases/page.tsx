import Link from 'next/link';
import { cursor, readProject, type Page } from '@/lib/assurance';
export const dynamic = 'force-dynamic';
export default async function ReleasesPage({ params, searchParams }: {
  params: Promise<{ projectId: string }>;
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}) {
  const { projectId } = await params;
  const root = `/projects/${encodeURIComponent(projectId)}`;
  const after = cursor((await searchParams).after);
  const releases = await readProject<Page<{ fingerprint: string; release: { label: string }; accepted: boolean }>>(projectId, `/releases?after=${encodeURIComponent(after)}`);
  return <div className="dashboard"><div className="dashboardHeading"><div><p className="overline">PROJECT RELEASES</p><h1>Releases</h1><p>Composed releases and their acceptance decisions.</p></div></div><section className="dashboardPanel">{releases?.items.map(r => <Link className="releaseListRow" key={r.fingerprint} href={`${root}/releases/${encodeURIComponent(r.fingerprint)}`}><strong>{r.release.label}</strong><span className={`badge ${r.accepted ? 'good' : 'neutral'}`}>{r.accepted ? 'Accepted' : 'Pending acceptance'}</span><span>↗</span></Link>)}{!releases?.items.length && <div className="panelEmpty"><h2>No releases published</h2></div>}</section>{releases?.next && <Link className="textLink" href={`${root}/releases?after=${encodeURIComponent(releases.next)}`}>Next releases →</Link>}</div>;
}
