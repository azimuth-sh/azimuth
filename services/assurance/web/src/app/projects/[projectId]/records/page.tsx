import Link from 'next/link';
import { requireViewer } from '@/lib/auth';
import { readProject, type Page, cursor, projectView } from '@/lib/assurance';
import { Disclosure } from '@/components/disclosure';
import { Identifier } from '@/components/identifier';
import { Markdown } from '@/components/markdown';
import { shortIdentifier } from '@/lib/identifier';
export const dynamic = 'force-dynamic';
export default async function RecordsPage({
  params,
  searchParams,
}: {
  params: Promise<{ projectId: string }>;
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}) {
  const { projectId } = await params;
  await requireViewer(projectId);
  const query = await searchParams;
  const view = await projectView(projectId, {
    runs: cursor(query.runs),
    verificationTargets: cursor(query['verification-targets']),
    reviews: cursor(query.reviews),
  });
  const releases = await readProject<Page<{ fingerprint: string; release: { label: string; model_revision: string; integration: { commit: string; main_sequence: number } | null } }>>(projectId, `/releases?after=${encodeURIComponent(cursor(query.releases))}`);
  const changes = await readProject<Page<{ id: string; revisions: unknown[] }>>(projectId, `/changes?after=${encodeURIComponent(cursor(query.changes))}`);
  const deployments = await readProject<Page<{ deployment: { id: string; environment: string }; release_fingerprint: string }>>(projectId, `/deployments?after=${encodeURIComponent(cursor(query.deployments))}`);
  const root = `/projects/${encodeURIComponent(projectId)}`;
  const reviewed = view.requirementReviews?.requirements ?? [];
  return (
    <div className="dashboard">
      <Link className="backLink" href="/">← Projects</Link>
      <header className="dashboardHeading"><div><p className="overline">INSPECTION</p><h1>Records</h1><p>Everything {projectId} has recorded. Lists show at most 25 records; raw records open on demand.</p></div></header>
      <section className="dashboardPanel"><header className="panelHeading"><div><h2>Releases</h2></div><span className="panelCount">{releases?.items.length ?? 0}</span></header>
        {releases?.items.map(r => <Link className="releaseListRow" key={r.fingerprint} href={`${root}/releases/${encodeURIComponent(r.fingerprint)}`}><strong>{r.release.label}</strong><span className={`badge ${r.release.integration ? 'good' : 'neutral'}`}>{r.release.integration ? `Integrated at ${r.release.integration.commit.slice(0, 8)}` : 'No integration commit'}</span></Link>)}
        {!releases?.items.length && <p className="tableEmpty">No releases recorded.</p>}
        {releases?.next && <Link className="textLink" href={`${root}/records?releases=${encodeURIComponent(releases.next)}`}>Next releases →</Link>}
      </section>
      <section className="dashboardPanel"><header className="panelHeading"><div><h2>Deployments</h2></div><span className="panelCount">{deployments?.items.length ?? 0}</span></header>
        {deployments?.items.map(d => <div className="releaseCompactRow" key={d.deployment.id}><div><strong>{d.deployment.environment}</strong><small><Identifier value={d.deployment.id} /></small></div><Link className="textLink" href={`${root}/releases/${encodeURIComponent(d.release_fingerprint)}`}>Release ↗</Link></div>)}
        {!deployments?.items.length && <p className="tableEmpty">No deployments recorded.</p>}
        {deployments?.next && <Link className="textLink" href={`${root}/records?deployments=${encodeURIComponent(deployments.next)}`}>Next deployments →</Link>}
      </section>
      <section className="dashboardPanel"><header className="panelHeading"><div><h2>Changes</h2></div><span className="panelCount">{changes?.items.length ?? 0}</span></header>
        {changes?.items.map(c => <Disclosure key={c.id} summary={c.id} meta={`${c.revisions.length} ${c.revisions.length === 1 ? 'revision' : 'revisions'}`}><pre className="rawRecord">{JSON.stringify(c.revisions, null, 2)}</pre></Disclosure>)}
        {!changes?.items.length && <p className="tableEmpty">No changes recorded.</p>}
        {changes?.next && <Link className="textLink" href={`${root}/records?changes=${encodeURIComponent(changes.next)}`}>Next Changes →</Link>}
      </section>
      <section className="dashboardPanel"><header className="panelHeading"><div><h2>Requirement Reviews</h2><p>Definition assessments of the selected model. An accepted review does not establish deployed Requirement Status.</p></div><span className="panelCount">{reviewed.length}</span></header>
        {view.requirementReviews ? reviewed.map(requirement => <Disclosure key={requirement.requirement} summary={requirement.requirement} meta={<span className={`badge ${requirement.assessment === 'accepted' ? 'good' : 'neutral'}`}>{requirement.assessment}</span>}>
          {requirement.diagnostics.length > 0 && <ul className="verificationReasons">{requirement.diagnostics.map((d, i) => <li key={i}>{d}</li>)}</ul>}
          {typeof requirement.record?.rationale === 'string' && <Markdown source={requirement.record.rationale} />}
          {requirement.record && <Disclosure summary="Reviewed dependencies and reviewer provenance"><pre className="rawRecord">{JSON.stringify(requirement.record, null, 2)}</pre></Disclosure>}
        </Disclosure>) : <p className="tableEmpty">No main-model publication; definition assessments unresolved.</p>}
      </section>
      <section className="dashboardPanel"><header className="panelHeading"><div><h2>Verification Targets</h2></div><span className="panelCount">{view.verificationTargets?.items.length ?? 0}</span></header>
        {view.verificationTargets?.items.length ? view.verificationTargets.items.map(item => <div className="releaseCompactRow" key={item.fingerprint}><Link className="textLink" href={`${root}/verification-targets/${encodeURIComponent(item.fingerprint)}`}><code title={item.fingerprint}>{shortIdentifier(item.fingerprint)}</code></Link></div>) : <p className="tableEmpty">No Verification Targets recorded.</p>}
        {view.verificationTargets?.next && <Link className="textLink" href={`${root}/records?verification-targets=${encodeURIComponent(view.verificationTargets.next)}`}>Next Verification Targets →</Link>}
      </section>
      <section className="dashboardPanel"><header className="panelHeading"><div><h2>Latest Run revisions</h2></div><span className="panelCount">{view.runs?.items.length ?? 0}</span></header>
        {view.runs?.items.length ? view.runs.items.map(run => <div className="releaseCompactRow" key={run.run_id}><div><Link className="textLink" href={`${root}/runs/${encodeURIComponent(run.run_id)}`}><code title={run.run_id}>{shortIdentifier(run.run_id)}</code></Link><small>Revision {run.revision}</small></div><span className={`badge ${run.status === 'passed' ? 'good' : 'neutral'}`}>{run.status}</span></div>) : <p className="tableEmpty">No Runs recorded.</p>}
        {view.runs?.next && <Link className="textLink" href={`${root}/records?runs=${encodeURIComponent(view.runs.next)}`}>Next Runs →</Link>}
      </section>
      <section className="dashboardPanel"><header className="panelHeading"><div><h2>Independent review history</h2></div><span className="panelCount">{view.reviews?.records.length ?? 0}</span></header>
        {view.reviews?.records.length ? view.reviews.records.map(review => <Disclosure key={String(review.record.fingerprint)} summary={`${String(review.record.kind)}${review.historical ? ' · historical' : ''}`} meta={<><span>{String(review.record.reviewer)}</span><span className={`badge ${review.assessment === 'accepted' ? 'good' : 'neutral'}`}>{review.assessment}</span></>}>
          {review.diagnostics.length > 0 && <ul className="verificationReasons">{review.diagnostics.map((d, i) => <li key={i}>{d}</li>)}</ul>}
          <p className="modelDetailMuted">Authenticated submitter {review.submitted_by} · received {review.submitted_at} · provenance {review.provenance_basis ?? 'authenticated-reviewer'}</p>
          <Disclosure summary="Raw record"><pre className="rawRecord">{JSON.stringify(review.record, null, 2)}</pre></Disclosure>
        </Disclosure>) : <p className="tableEmpty">No reviews available. A main-model publication is required to assess review freshness.</p>}
        {view.reviews?.next && <Link className="textLink" href={`${root}/records?reviews=${encodeURIComponent(view.reviews.next)}`}>Next reviews →</Link>}
      </section>
      <Disclosure summary="Selected model authority">
        {view.authority ? <><p className="modelDetailMuted">Fingerprint <Identifier value={String(view.authority.fingerprint)} /></p><pre className="rawRecord">{JSON.stringify(view.authority, null, 2)}</pre></> : <p className="notice">No main-model publication. Assurance State remains unresolved.</p>}
      </Disclosure>
    </div>
  );
}
