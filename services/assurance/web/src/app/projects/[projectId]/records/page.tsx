import Link from 'next/link';
import { requireViewer } from '@/lib/auth';
import { readProject, type Page, cursor, projectView } from '@/lib/assurance';
export const dynamic = 'force-dynamic';
export default async function ProjectPage({
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
  const releases = await readProject<Page<{ fingerprint: string; release: { label: string; model_revision: string }; accepted: boolean }>>(projectId, `/releases?after=${encodeURIComponent(cursor(query.releases))}`);
  const changes = await readProject<Page<{ id: string; revisions: unknown[] }>>(projectId, `/changes?after=${encodeURIComponent(cursor(query.changes))}`);
  const deployments = await readProject<Page<{ deployment: { id: string; environment: string }; release_fingerprint: string }>>(projectId, `/deployments?after=${encodeURIComponent(cursor(query.deployments))}`);
  const root = `/projects/${encodeURIComponent(projectId)}`;
  return (
    <div className="shell">
      <Link className="backLink" href="/">
        ← Projects
      </Link>
      <h1>{projectId}</h1>
      <p>
        Inspection only. Pages list at most 25 records; follow each list
        independently. No missing or stale review is treated as supported.
      </p>
      <p><Link href={`${root}/model`}>Browse Modules → Requirements → Scenarios and Checks</Link></p>
      <section className="section"><h2>Composed releases</h2>{releases?.items.map(r => <p key={r.fingerprint}><Link href={`${root}/releases/${encodeURIComponent(r.fingerprint)}`}>{r.release.label}</Link> · {r.accepted ? 'accepted' : 'candidate'}</p>)}{releases?.next && <Link href={`${root}/records?releases=${encodeURIComponent(releases.next)}`}>Next releases →</Link>}</section>
      <section className="section"><h2>Changes</h2>{changes?.items.map(c => <details key={c.id}><summary>{c.id}</summary><pre>{JSON.stringify(c.revisions,null,2)}</pre></details>)}{changes?.next && <Link href={`${root}/records?changes=${encodeURIComponent(changes.next)}`}>Next Changes →</Link>}</section>
      <section className="section"><h2>Deployments</h2>{deployments?.items.map(d => <p key={d.deployment.id}>{d.deployment.environment} · {d.deployment.id} · <Link href={`${root}/releases/${encodeURIComponent(d.release_fingerprint)}`}>Reference release</Link></p>)}{deployments?.next && <Link href={`${root}/records?deployments=${encodeURIComponent(deployments.next)}`}>Next deployments →</Link>}</section>
      <section className="section">
        <h2>Selected model authority</h2>
        {view.authority ? (
          <details>
            <summary>{String(view.authority.fingerprint)}</summary>
            <pre>{JSON.stringify(view.authority, null, 2)}</pre>
          </details>
        ) : (
          <p className="notice">
            No authority selected. Assurance State remains unresolved.
          </p>
        )}
      </section>
      <section className="section">
        <h2>Requirement Reviews</h2>
        <p>
          Definition assessments are independent of Verification Targets. An
          accepted Requirement Review does not establish deployed Requirement Status.
        </p>
        {view.requirementReviews?.requirements.map((requirement) => (
          <article key={requirement.requirement}>
            <h3>{requirement.requirement}</h3>
            <p>
              Review: <strong>{requirement.assessment}</strong>
            </p>
            <p>{requirement.diagnostics.join('; ')}</p>
            {requirement.record && (
              <details>
                <summary>Reviewed dependencies and reviewer provenance</summary>
                <pre>{JSON.stringify(requirement.record, null, 2)}</pre>
              </details>
            )}
          </article>
        )) ?? <p>No selected authority; definition assessments unresolved.</p>}
      </section>
      <section className="section">
        <h2>Verification Targets</h2>
        {view.verificationTargets?.items.length ? (
          view.verificationTargets.items.map((item) => (
            <article key={item.fingerprint}>
              <Link
                href={`${root}/verification-targets/${encodeURIComponent(item.fingerprint)}`}
              >
                {item.fingerprint}
              </Link>
              <details>
                <summary>Verification Target identity</summary>
                <pre>{JSON.stringify(item.verification_target, null, 2)}</pre>
              </details>
            </article>
          ))
        ) : (
          <p>No Verification Targets recorded.</p>
        )}
        {view.verificationTargets?.next && (
          <Link
            href={`${root}/records?verification-targets=${encodeURIComponent(view.verificationTargets.next)}`}
          >
            Next Verification Targets →
          </Link>
        )}
      </section>
      <section className="section">
        <h2>Latest Run revisions</h2>
        {view.runs?.items.length ? (
          view.runs.items.map((run) => (
            <article key={run.run_id}>
              <Link href={`${root}/runs/${encodeURIComponent(run.run_id)}`}>
                {run.run_id}
              </Link>
              <p>
                Status: {run.status} · revision {run.revision}
              </p>
              <p>
                Verification Target: <code>{run.verification_target_fingerprint}</code>
              </p>
              <p>
                Executed model: <code>{run.model_fingerprint}</code>
              </p>
            </article>
          ))
        ) : (
          <p>No Runs recorded.</p>
        )}
        {view.runs?.next && (
          <Link href={`${root}/records?runs=${encodeURIComponent(view.runs.next)}`}>
            Next Runs →
          </Link>
        )}
      </section>
      <section className="section">
        <h2>Independent review history</h2>
        {view.reviews?.records.length ? (
          view.reviews.records.map((review) => (
            <details key={String(review.record.fingerprint)}>
              <summary>
                {String(review.record.kind)} · {review.assessment}
                {review.historical ? ' · historical record' : ''}
              </summary>
              <p>{review.diagnostics.join('; ')}</p>
              <p>
                Asserted reviewer: {String(review.record.reviewer)} · authenticated
                submitter: {review.submitted_by} · received:{' '}
                {review.submitted_at} · provenance: {review.provenance_basis ?? 'authenticated-reviewer'}
              </p>
              <pre>{JSON.stringify(review.record, null, 2)}</pre>
            </details>
          ))
        ) : (
          <p>
            No reviews available. A selected authority is required to assess
            review freshness.
          </p>
        )}
        {view.reviews?.next && (
          <Link
            href={`${root}/records?reviews=${encodeURIComponent(view.reviews.next)}`}
          >
            Next reviews →
          </Link>
        )}
      </section>
    </div>
  );
}
