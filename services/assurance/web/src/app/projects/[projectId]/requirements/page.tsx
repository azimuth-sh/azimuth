import { selectedEnvironment } from "@/lib/environment";
import Link from 'next/link';
import { notFound } from 'next/navigation';
import { environmentView, requirementStatus } from '@/lib/production';
import type { DeploymentStatusSummary } from '@/lib/assurance';

export const dynamic = 'force-dynamic';
const filters = [
  { id: 'all', label: 'All Requirements' },
  { id: 'supported', label: 'Satisfied' },
  { id: 'violated', label: 'Not satisfied' },
  { id: 'unresolved', label: 'Unresolved' },
] as const;
const statusLabels = { supported: 'Satisfied', violated: 'Not satisfied', unresolved: 'Unresolved' };
const tones = { supported: 'good', violated: 'bad', unresolved: 'warn' };

export default async function RequirementsPage({ params, searchParams }: {
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
  const { environmentDeployments, deployment, release, revision, verificationTarget, state, freshness, hasMore, catalog } = await environmentView(projectId, selected, environment);
  const requirements = state?.requirements ?? [];
  const matching = requirements.filter(c => status === 'all' || requirementStatus(c.status) === status);
  const groups = new Map<string, { title: string; requirements: DeploymentStatusSummary['requirements'] }>();
  for (const requirement of matching) {
    const module = catalog?.modules.find(m => m.requirements.includes(requirement.requirement));
    const key = module?.id ?? '';
    const group = groups.get(key) ?? { title: module?.id ?? 'Unassigned Requirements', requirements: [] };
    group.requirements.push(requirement);
    groups.set(key, group);
  }
  function listLink(status: string, deploymentId = deployment?.deployment.id ?? selected) {
    const query = new URLSearchParams({ status, environment });
    if (deploymentId) query.set('deployment', deploymentId);
    return `${root}/requirements?${query}`;
  }
  const overview = deployment ? `${root}?environment=${encodeURIComponent(environment)}&deployment=${encodeURIComponent(deployment.deployment.id)}` : `${root}?environment=${encodeURIComponent(environment)}`;
  return <div className="dashboard requirementsPage">
    <Link className="backLink" href={overview}>← Environment overview</Link>
    <header className="dashboardHeading"><div><p className="overline">ENVIRONMENT REQUIREMENTS</p><h1>{filter.id === 'all' ? 'All Requirements' : `${filter.label} Requirements`}</h1><p>Grouped by module · {state ? `${matching.length} matching ${matching.length === 1 ? 'Requirement' : 'Requirements'}` : 'Assessment unavailable'}</p></div><span className="environmentBadge">{environment}</span></header>
    <section className="deploymentBar" aria-label="Selected environment deployment"><div><span className="fieldLabel">DEPLOYED RELEASE</span><strong>{release && deployment ? <Link href={`${root}/releases/${encodeURIComponent(deployment.release_fingerprint)}?environment=${encodeURIComponent(environment)}`}>{release.publication.release.label} ↗</Link> : 'No deployment selected'}</strong><small>{deployment?.deployment.id ?? 'Choose a environment deployment to inspect its Requirements'}</small></div><div><span className="fieldLabel">GROUPING</span><strong>By module</strong></div></section>
    {freshness && freshness.state !== 'ready' && <section className="dashboardPanel" role="status"><h2>{freshness.state === 'expired' ? 'Assessment overdue' : 'Assessment updating'}</h2><p>Evidence is being reassessed. Previous conclusions are not shown as current.</p></section>}
    <nav className="requirementFilters" aria-label="Filter Requirements by Requirement Status">{filters.map(item => <Link key={item.id} href={listLink(item.id)} aria-current={item.id === filter.id ? 'page' : undefined}>{item.label}<span>{state ? item.id === 'all' ? requirements.length : requirements.filter(c => requirementStatus(c.status) === item.id).length : '—'}</span></Link>)}</nav>
    {!deployment && (environmentDeployments.length > 0 || hasMore) && <section className="dashboardPanel deploymentPicker"><h2>Select a environment deployment</h2>{selected && <p>The requested deployment is not in the available environment records.</p>}{environmentDeployments.map(d => <Link key={d.deployment.id} href={listLink(filter.id,d.deployment.id)}>{d.deployment.id} →</Link>)}{hasMore && <p>Only the first 100 deployment records are listed. Use <Link href={`${root}/records`}>Records</Link> to browse further.</p>}</section>}
    {!state ? <section className="dashboardPanel panelEmpty"><h2>Deployment Requirement Status is not available</h2><p>{deployment ? 'No assessment is available for this deployment. Missing data does not satisfy any Requirement.' : 'Select a recorded environment deployment to see its Requirement Status.'}</p></section> : !matching.length ? <section className="dashboardPanel panelEmpty"><h2>No {filter.id === 'all' ? '' : `${filter.label.toLowerCase()} `}Requirements</h2><p>{requirements.length ? 'No Requirements match this status in the selected deployment. Choose another status above.' : 'No Requirements have been assessed for this deployment.'}</p></section> : <div className="requirementGroups">{[...groups].map(([id,group]) => <section className="dashboardPanel requirementGroup" key={id}><div className="panelHeading"><h2><span className="moduleIcon" aria-hidden="true">◇</span>{group.title}</h2><span className="panelCount">{group.requirements.length}</span></div>{group.requirements.map(requirement => {
      const status = requirementStatus(requirement.status);
      const details = `${root}/model?${new URLSearchParams({ environment, revision: revision!, 'verification-target': verificationTarget!, requirement: requirement.requirement })}`;
      return <Link className="attentionRow" key={requirement.requirement} href={details}><span className={`attentionIcon ${tones[status]}`} aria-hidden="true">{status === 'supported' ? '✓' : status === 'violated' ? '!' : '?'}</span><div><div className="rowMeta"><span className={`badge ${tones[status]}`}>{statusLabels[status]}</span></div><h3>{catalog?.requirements.find(c => c.id === requirement.requirement)?.title ?? requirement.requirement}</h3><p>{requirement.explanation.summary}</p><span className="rowAction">View Requirement, review & evidence ↗</span></div></Link>;
    })}</section>)}</div>}
  </div>;
}
