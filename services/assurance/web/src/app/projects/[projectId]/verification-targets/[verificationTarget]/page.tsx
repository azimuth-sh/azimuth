import Link from 'next/link';
import { notFound } from 'next/navigation';
import { requireViewer } from '@/lib/auth';
import {
  readProject,
  routeFingerprint,
  type VerificationTargetState,
} from '@/lib/assurance';
import { requirementStatus } from '@/lib/production';
import { Disclosure } from '@/components/disclosure';
import { Identifier } from '@/components/identifier';
import { CodeDrift, ReviewChips } from '@/components/review-dimensions';
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
  const tones = { supported: 'good', violated: 'bad', unresolved: 'warn', 'not-applicable': 'neutral' };
  return (
    <div className="dashboard">
      <Link className="backLink" href={`/projects/${encodeURIComponent(projectId)}`}>← Overview</Link>
      <header className="dashboardHeading"><div><p className="overline">VERIFICATION TARGET</p><h1>Evidence inspection</h1><p>Exact Verification Target records. The environment overview shows computed deployment Requirement Status.</p></div><Identifier value={verificationTarget} /></header>
      {!state ? (
        <section className="dashboardPanel"><p className="tableEmpty">No main-model publication; conclusion unresolved.</p></section>
      ) : (
        <section className="dashboardPanel">
          <header className="panelHeading"><div><h2>Requirements</h2><p>Authority <Identifier value={state.authority_fingerprint} /></p></div><span className="panelCount">{state.requirements.length}</span></header>
          {state.requirements.map((requirement) => {
            const status = requirementStatus(requirement.status);
            return (
              <Disclosure
                key={requirement.requirement}
                summary={requirement.requirement}
                meta={<><ReviewChips verification={requirement.verification_review} realization={requirement.realization_review} /><span className={`badge ${tones[status]}`}>{statusLabels[status]}</span></>}
              >
                <CodeDrift drift={requirement.realization_code_drift} commit={requirement.explanation?.realization_review?.reviewed_commit} />
                {requirement.diagnostics.length > 0 && <ul className="verificationReasons">{requirement.diagnostics.map((d, i) => <li key={i}>{d}</li>)}</ul>}
                <Disclosure summary="Bindings and gaps">
                  <pre className="rawRecord">{JSON.stringify({ bindings: requirement.bindings, gaps: requirement.gaps, challenges: requirement.challenges }, null, 2)}</pre>
                </Disclosure>
              </Disclosure>
            );
          })}
          <Disclosure summary="Complete assessment, dependencies and challenge results">
            <pre className="rawRecord">{JSON.stringify(state, null, 2)}</pre>
          </Disclosure>
        </section>
      )}
    </div>
  );
}
