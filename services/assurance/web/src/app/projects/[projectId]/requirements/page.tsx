import { selectedEnvironment } from "@/lib/environment";
import Link from 'next/link';
import { notFound } from 'next/navigation';
import { requirementStatus, requirementsView } from '@/lib/production';
import { Identifier } from '@/components/identifier';
import { RequirementList, type RequirementRow } from '@/components/requirement-list';
import { shortIdentifier } from '@/lib/identifier';

export const dynamic = 'force-dynamic';
const filterIds = ['all', 'supported', 'violated', 'unresolved', 'not-applicable'];

export default async function RequirementsPage({ params, searchParams }: {
  params: Promise<{ projectId: string }>;
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}) {
  const { projectId } = await params;
  const query = await searchParams;
  const environment = await selectedEnvironment(projectId, query.environment);
  const status = query.status ?? 'all';
  if (typeof status !== 'string' || !filterIds.includes(status) || Array.isArray(query.deployment)) notFound();
  const root = `/projects/${encodeURIComponent(projectId)}`;
  const selected = typeof query.deployment === 'string' ? query.deployment : null;
  const { environmentDeployments, deployment, release, revision, verificationTarget, state, freshness, hasMore, catalog } = await requirementsView(projectId, selected, environment);
  const deploymentId = deployment?.deployment.id ?? selected;
  function listLink(filter: string, deploymentChoice = deploymentId) {
    const link = new URLSearchParams({ status: filter, environment });
    if (deploymentChoice) link.set('deployment', deploymentChoice);
    return `${root}/requirements?${link}`;
  }
  const rows: RequirementRow[] = (state?.requirements ?? []).map(requirement => {
    const module = catalog?.modules.find(m => m.requirements.includes(requirement.requirement));
    return {
      id: requirement.requirement,
      title: catalog?.requirements.find(c => c.id === requirement.requirement)?.title ?? requirement.requirement,
      summary: requirement.summary,
      status: requirementStatus(requirement.status),
      module: module?.id ?? 'Unassigned Requirements',
      details: `${root}/model?${new URLSearchParams({ environment, revision: revision!, 'verification-target': verificationTarget!, requirement: requirement.requirement })}`,
    };
  });
  const overview = deployment ? `${root}?environment=${encodeURIComponent(environment)}&deployment=${encodeURIComponent(deployment.deployment.id)}` : `${root}?environment=${encodeURIComponent(environment)}`;
  return <div className="dashboard requirementsPage">
    <Link className="backLink" href={overview}>← Environment overview</Link>
    <section className="deploymentBar" aria-label="Selected environment deployment"><div><span className="fieldLabel">DEPLOYED RELEASE</span><strong>{release && deployment ? <Link href={`${root}/releases/${encodeURIComponent(deployment.release_fingerprint)}?environment=${encodeURIComponent(environment)}`}>{release.publication.release.label} ↗</Link> : 'No deployment selected'}</strong><small>{deployment ? <Identifier value={deployment.deployment.id} /> : 'Choose an environment deployment to inspect its Requirements'}</small></div></section>
    {freshness && freshness.state !== 'ready' && <section className="dashboardPanel" role="status"><h2>{freshness.state === 'expired' ? 'Assessment overdue' : 'Assessment updating'}</h2><p>Evidence is being reassessed. Previous conclusions are not shown as current.</p></section>}
    {!deployment && (environmentDeployments.length > 0 || hasMore) && <section className="dashboardPanel deploymentPicker"><h2>Select a environment deployment</h2>{selected && <p>The requested deployment is not in the available environment records.</p>}{environmentDeployments.map(d => <Link key={d.deployment.id} href={listLink(status, d.deployment.id)}>{shortIdentifier(d.deployment.id)} →</Link>)}{hasMore && <p>Only the first 100 deployment records are listed. Use <Link href={`${root}/deployments`}>Deployments</Link> to browse further.</p>}</section>}
    {!state ? <section className="dashboardPanel panelEmpty"><h2>Deployment Requirement Status is not available</h2><p>{deployment ? 'No assessment is available for this deployment. Missing data does not satisfy any Requirement.' : 'Select a recorded environment deployment to see its Requirement Status.'}</p></section> : <RequirementList key={status} rows={rows} environment={environment} initialStatus={status} filterHrefs={Object.fromEntries(filterIds.map(id => [id, listLink(id)]))} />}
  </div>;
}
