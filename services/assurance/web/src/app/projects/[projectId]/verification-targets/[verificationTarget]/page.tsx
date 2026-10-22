import Link from 'next/link';
import { notFound } from 'next/navigation';
import { requireViewer } from '@/lib/auth';
import {
  readProject,
  routeFingerprint,
  type VerificationTargetState,
} from '@/lib/assurance';
import { requirementStatus } from '@/lib/production';
export const dynamic = 'force-dynamic';
const statusLabels = { supported: 'Satisfied', violated: 'Not satisfied', unresolved: 'Unresolved', 'not-applicable': 'Not applicable here' };
export default async function VerificationTargetPage({
  params,
}: {
  params: Promise<{ projectId: string; verificationTarget: string }>;
}) {
  const { projectId, verificationTarget: segment } = await params;
  const verificationTarget = routeFingerprint(segment);
  await requireViewer(projectId);
  if (!verificationTarget) notFound();
  const state = await readProject<VerificationTargetState>(
    projectId,
    `/verification-targets/${encodeURIComponent(verificationTarget)}/state`,
  );
  return (
    <div className="shell">
      <Link href={`/projects/${encodeURIComponent(projectId)}`}>← Project</Link>
      <h1>Verification Target evidence inspection</h1>
      <p>Inspect exact Verification Target records here. The environment overview shows computed deployment Requirement Status.</p><code>{verificationTarget}</code>
      {!state ? (
        <p>No main-model publication; conclusion unresolved.</p>
      ) : (
        <>
          <p>
            Authority: <code>{state.authority_fingerprint}</code>
          </p>
          {state.requirements.map((requirement) => (
            <article className="section" key={requirement.requirement}>
              <h2>{requirement.requirement}</h2>
              <p>
                Recorded conclusion: <strong>{statusLabels[requirementStatus(requirement.status)]}</strong> · Requirement Review:{' '}
                <strong>{requirement.review ?? 'missing'}</strong> · Requirement Status record: <strong>{requirement.requirement_status ?? 'computed'}</strong>
              </p>
              <p>{requirement.diagnostics.join('; ')}</p>
              <details>
                <summary>Bindings and gaps</summary>
                <pre>
                  {JSON.stringify(
                    {
                      bindings: requirement.bindings,
                      gaps: requirement.gaps,
                      challenges: requirement.challenges,
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
              Complete assessment, dependencies and challenge results
            </summary>
            <pre>{JSON.stringify(state, null, 2)}</pre>
          </details>
        </>
      )}
    </div>
  );
}
