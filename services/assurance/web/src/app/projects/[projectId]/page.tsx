import Link from 'next/link';
import { requireViewer } from '@/lib/auth';
import { cursor, projectView } from '@/lib/assurance';
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
    subjects: cursor(query.subjects),
    reviews: cursor(query.reviews),
  });
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
        <h2>Exact Subjects</h2>
        {view.subjects?.items.length ? (
          view.subjects.items.map((item) => (
            <article key={item.fingerprint}>
              <Link
                href={`${root}/subjects/${encodeURIComponent(item.fingerprint)}`}
              >
                {item.fingerprint}
              </Link>
              <details>
                <summary>Subject identity</summary>
                <pre>{JSON.stringify(item.subject, null, 2)}</pre>
              </details>
            </article>
          ))
        ) : (
          <p>No Subjects recorded.</p>
        )}
        {view.subjects?.next && (
          <Link
            href={`${root}?subjects=${encodeURIComponent(view.subjects.next)}`}
          >
            Next Subjects →
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
                Exact Subject: <code>{run.subject_fingerprint}</code>
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
          <Link href={`${root}?runs=${encodeURIComponent(view.runs.next)}`}>
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
              </summary>
              <p>{review.diagnostics.join('; ')}</p>
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
            href={`${root}?reviews=${encodeURIComponent(view.reviews.next)}`}
          >
            Next reviews →
          </Link>
        )}
      </section>
    </div>
  );
}
