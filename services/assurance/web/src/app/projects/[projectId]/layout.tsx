import Link from 'next/link';
import { Suspense } from 'react';
import { readProject, type Page } from '@/lib/assurance';
import { selectedEnvironment } from '@/lib/environment';
import { EnvironmentSelector } from '@/components/environment-selector';
import { requireViewer } from '@/lib/auth';
import { ObservationRefresh } from '@/components/observation-refresh';
import { ProjectNavigation } from '@/components/project-navigation';

export default async function ProjectLayout({ children, params }: {
  children: React.ReactNode; params: Promise<{ projectId: string }>;
}) {
  const { projectId } = await params;
  await requireViewer(projectId);
  const [initial, selections, deployments] = await Promise.all([
    selectedEnvironment(projectId),
    readProject<Page<{ environment: string }>>(projectId, '/environments?limit=100'),
    readProject<Page<{ deployment: { environment: string } }>>(projectId, '/deployments?limit=100'),
  ]);
  const environments = [...new Set([...(selections?.items.map(item => item.environment) ?? []), ...(deployments?.items.map(item => item.deployment.environment) ?? [])])].sort();
  const root = `/projects/${encodeURIComponent(projectId)}`;
  return <div className="workspace">
    <aside className="sidebar">
      <Link href="/" className="workspaceLabel">WORKSPACE <span>↗</span></Link>
      <div className="projectSelector"><span className="projectAvatar">{projectId[0].toUpperCase()}</span><strong>{projectId}</strong></div>
      <Suspense fallback={null}><EnvironmentSelector project={projectId} initial={initial} environments={environments} /></Suspense>
      <p className="navHeading">ASSURANCE</p>
      <Suspense fallback={null}><ProjectNavigation root={root} environment={initial} /></Suspense>
      <div className="sidebarNote"><span className="liveDot" /> Inspection workspace<p>Environment evidence.<br />Explicit decisions.</p></div>
    </aside>
    <div className="workspaceContent"><ObservationRefresh />{children}</div>
  </div>;
}
