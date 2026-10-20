import { selectedEnvironment } from "@/lib/environment";
import Link from 'next/link';
import { notFound } from 'next/navigation';
import { environmentView, claimStatus } from '@/lib/production';
import type { SubjectState } from '@/lib/assurance';

export const dynamic = 'force-dynamic';
const filters = [
  { id: 'all', label: 'All Claims' },
  { id: 'supported', label: 'Confirmed' },
  { id: 'violated', label: 'Violated' },
  { id: 'unresolved', label: 'Unresolved' },
] as const;
const statusLabels = { supported: 'Confirmed', violated: 'Violated', unresolved: 'Unresolved' };
const tones = { supported: 'good', violated: 'bad', unresolved: 'warn' };

export default async function ClaimsPage({ params, searchParams }: {
  params: Promise<{ projectId: string }>;
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}) {
  const { projectId } = await params;
  const query = await searchParams;
  const environment = await selectedEnvironment(projectId, query.environment);
  const status = query.status ?? 'all';
  const filter = filters.find(item => item.id === status);
  if (!filter || Array.isArray(query.deployment)) notFound();
  const root = `/projects/${encodeURIComponent(projectId)}`;
  const selected = typeof query.deployment === 'string' ? query.deployment : null;
  const { environmentDeployments, deployment, release, revision, subject, state, hasMore, catalog } = await environmentView(projectId, selected, environment);
  const claims = state?.claims ?? [];
  const matching = claims.filter(c => status === 'all' || claimStatus(c.status) === status);
  const groups = new Map<string, { title: string; claims: SubjectState['claims'] }>();
  for (const claim of matching) {
    const module = catalog?.modules.find(m => m.claims.includes(claim.claim));
    const key = module?.id ?? '';
    const group = groups.get(key) ?? { title: module?.id ?? 'Unassigned Claims', claims: [] };
    group.claims.push(claim);
    groups.set(key, group);
  }
  function listLink(status: string, deploymentId = deployment?.deployment.id ?? selected) {
    const query = new URLSearchParams({ status, environment });
    if (deploymentId) query.set('deployment', deploymentId);
    return `${root}/claims?${query}`;
  }
  const overview = deployment ? `${root}?environment=${encodeURIComponent(environment)}&deployment=${encodeURIComponent(deployment.deployment.id)}` : `${root}?environment=${encodeURIComponent(environment)}`;
  return <div className="dashboard claimsPage">
    <Link className="backLink" href={overview}>← Environment overview</Link>
    <header className="dashboardHeading"><div><p className="overline">ENVIRONMENT CLAIMS</p><h1>{filter.id === 'all' ? 'All Claims' : `${filter.label} Claims`}</h1><p>Grouped by module · {state ? `${matching.length} matching ${matching.length === 1 ? 'Claim' : 'Claims'}` : 'Assessment unavailable'}</p></div><span className="environmentBadge">{environment}</span></header>
    <section className="deploymentBar" aria-label="Selected environment deployment"><div><span className="fieldLabel">DEPLOYED RELEASE</span><strong>{release && deployment ? <Link href={`${root}/releases/${encodeURIComponent(deployment.release_fingerprint)}?environment=${encodeURIComponent(environment)}`}>{release.publication.release.label} ↗</Link> : 'No deployment selected'}</strong><small>{deployment?.deployment.id ?? 'Choose a environment deployment to inspect its Claims'}</small></div><div><span className="fieldLabel">GROUPING</span><strong>By module</strong></div></section>
    <nav className="claimFilters" aria-label="Filter Claims by Status">{filters.map(item => <Link key={item.id} href={listLink(item.id)} aria-current={item.id === filter.id ? 'page' : undefined}>{item.label}<span>{state ? item.id === 'all' ? claims.length : claims.filter(c => claimStatus(c.status) === item.id).length : '—'}</span></Link>)}</nav>
    {!deployment && (environmentDeployments.length > 0 || hasMore) && <section className="dashboardPanel deploymentPicker"><h2>Select a environment deployment</h2>{selected && <p>The requested deployment is not in the available environment records.</p>}{environmentDeployments.map(d => <Link key={d.deployment.id} href={listLink(filter.id,d.deployment.id)}>{d.deployment.id} →</Link>)}{hasMore && <p>Only the first 100 deployment records are listed. Use <Link href={`${root}/records`}>Records</Link> to browse further.</p>}</section>}
    {!state ? <section className="dashboardPanel panelEmpty"><h2>Deployment Claim Status is not available</h2><p>{deployment ? 'No assessment is available for this deployment. Missing data does not confirm any Claim.' : 'Select a recorded environment deployment to see its Claim Status.'}</p></section> : !matching.length ? <section className="dashboardPanel panelEmpty"><h2>No {filter.id === 'all' ? '' : `${filter.label.toLowerCase()} `}Claims</h2><p>{claims.length ? 'No Claims match this status in the selected deployment. Choose another status above.' : 'No Claims have been assessed for this deployment.'}</p></section> : <div className="claimGroups">{[...groups].map(([id,group]) => <section className="dashboardPanel claimGroup" key={id}><div className="panelHeading"><h2><span className="moduleIcon" aria-hidden="true">◇</span>{group.title}</h2><span className="panelCount">{group.claims.length}</span></div>{group.claims.map(claim => {
      const status = claimStatus(claim.status);
      const details = `${root}/model?${new URLSearchParams({ environment, revision: revision!, subject: subject!, claim: claim.claim })}`;
      return <Link className="attentionRow" key={claim.claim} href={details}><span className={`attentionIcon ${tones[status]}`} aria-hidden="true">{status === 'supported' ? '✓' : status === 'violated' ? '!' : '?'}</span><div><div className="rowMeta"><span className={`badge ${tones[status]}`}>{statusLabels[status]}</span></div><h3>{catalog?.claims.find(c => c.id === claim.claim)?.title ?? claim.claim}</h3><p>{claim.explanation.summary}</p><span className="rowAction">View Claim, review & evidence ↗</span></div></Link>;
    })}</section>)}</div>}
  </div>;
}
