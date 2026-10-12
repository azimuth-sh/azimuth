import Link from 'next/link';
import { notFound } from 'next/navigation';
import { requireViewer } from '@/lib/auth';
import {
  readProject,
  routeFingerprint,
  type SubjectState,
} from '@/lib/assurance';
export const dynamic = 'force-dynamic';
export default async function SubjectPage({
  params,
}: {
  params: Promise<{ projectId: string; subject: string }>;
}) {
  const { projectId, subject: segment } = await params;
  const subject = routeFingerprint(segment);
  await requireViewer(projectId);
  if (!subject) notFound();
  const state = await readProject<SubjectState>(
    projectId,
    `/subjects/${encodeURIComponent(subject)}/state`,
  );
  return (
    <div className="shell">
      <Link href={`/projects/${encodeURIComponent(projectId)}`}>← Project</Link>
      <h1>Exact-Subject Claim Health</h1>
      <code>{subject}</code>
      {!state ? (
        <p>No selected authority; conclusion unresolved.</p>
      ) : (
        <>
          <p>
            Authority: <code>{state.authority_fingerprint}</code>
          </p>
          {state.claims.map((claim) => (
            <article className="section" key={claim.claim}>
              <h2>{claim.claim}</h2>
              <p>
                Conclusion: <strong>{claim.status}</strong> · Review:{' '}
                <strong>{claim.review ?? 'missing'}</strong> · Health
                assessment: <strong>{claim.health ?? 'missing'}</strong>
              </p>
              <p>{claim.diagnostics.join('; ')}</p>
              <details>
                <summary>Bindings and gaps</summary>
                <pre>
                  {JSON.stringify(
                    {
                      bindings: claim.bindings,
                      gaps: claim.gaps,
                      challenges: claim.challenges,
                    },
                    null,
                    2,
                  )}
                </pre>
              </details>
            </article>
          ))}
          <details>
            <summary>
              Complete health, dependencies and challenge results
            </summary>
            <pre>{JSON.stringify(state, null, 2)}</pre>
          </details>
        </>
      )}
    </div>
  );
}
