import Link from 'next/link';
import { notFound } from 'next/navigation';
import { GateDecisionRecord, getProjectView } from '@/lib/assurance';

export default async function ProjectPage({
  params,
}: {
  params: Promise<{ projectId: string }>;
}) {
  const { projectId } = await params;
  let view;
  try {
    view = await getProjectView(projectId);
  } catch (error) {
    if (error instanceof Error && error.message === 'project not found') notFound();
    throw error;
  }

  const { snapshot, gates, workItems } = view;
  const open = gates.filter((gate) => gate.decision.status === 'open').length;
  const closed = gates.length - open;
  const latestImportedModel = snapshot.account.projectSnapshots.at(-1) ?? null;
  const activeContracts = snapshot.account.definitions.map((definition) => ({
    definition,
    contract: latestImportedModel?.claims.find(
      (contract) =>
        contract.spec === definition.claim.spec &&
        contract.claim === definition.claim.claim &&
        contract.contractFingerprint === definition.claim.contractFingerprint,
    ),
  }));

  return (
    <div className="shell projectShell">
      <Link href="/" className="backLink">
        ← All projects
      </Link>
      <section className="projectHero">
        <div>
          <p className="eyebrow">{snapshot.project.id}</p>
          <h1>{snapshot.project.name}</h1>
          <p className="lede">Current state is projected from immutable semantic and execution records.</p>
        </div>
        <div className="gateSummary" aria-label="Current gate summary">
          <Metric value={open} label="open" tone="good" />
          <Metric value={closed} label="closed" tone={closed > 0 ? 'bad' : 'neutral'} />
          <Metric value={workItems.length} label="work items" tone={workItems.length > 0 ? 'warn' : 'neutral'} />
        </div>
      </section>

      <section className="recordStrip" aria-label="Account record counts">
        <RecordCount label="Model snapshots" value={snapshot.account.projectSnapshots.length} />
        <RecordCount label="Definitions" value={snapshot.account.definitions.length} />
        <RecordCount label="Qualifications" value={snapshot.account.qualifications.length} />
        <RecordCount label="Observations" value={snapshot.account.observations.length} />
        <RecordCount label="Challenges" value={snapshot.account.challenges.length} />
        <RecordCount label="Decisions" value={snapshot.gateDecisions.length} />
      </section>

      <section className="section modelSection" aria-labelledby="model-heading">
        <div className="sectionHeading compact">
          <div>
            <p className="kicker">Repository authority</p>
            <h2 id="model-heading">Claim contracts</h2>
          </div>
          <span className="count">{activeContracts.length}</span>
        </div>
        {latestImportedModel ? (
          <>
            <dl className="modelIdentity">
              <div>
                <dt>Snapshot</dt>
                <dd>{shortFingerprint(latestImportedModel.id)}</dd>
              </div>
              <div>
                <dt>Accepted model</dt>
                <dd>{shortFingerprint(latestImportedModel.modelFingerprint)}</dd>
              </div>
            </dl>
            <div className="contractGrid">
              {activeContracts.map(({ definition, contract }) => (
                <article
                  className={`contractCard ${contract ? '' : 'contractMissing'}`}
                  key={definition.id}
                >
                  <p className="stage">{definition.stage} definition</p>
                  <h3>{definition.id}</h3>
                  <p className="claimIdentity">
                    {definition.claim.spec}#{definition.claim.claim}
                  </p>
                  {contract ? (
                    <dl className="contractFacts">
                      <div>
                        <dt>Surface</dt>
                        <dd>{contract.surface?.id ?? 'behavioural claim'}</dd>
                      </div>
                      <div>
                        <dt>Required areas</dt>
                        <dd>
                          {contract.obligatedAreas.map((area) => area.id).join(', ') || 'none'}
                        </dd>
                      </div>
                      <div>
                        <dt>Contract</dt>
                        <dd>{shortFingerprint(contract.contractFingerprint)}</dd>
                      </div>
                    </dl>
                  ) : (
                    <p className="reasonLine">
                      This definition is not applicable to the latest snapshot.
                    </p>
                  )}
                </article>
              ))}
            </div>
          </>
        ) : (
          <div className="notice noticeError">
            <strong>No repository snapshot registered</strong>
            <p>Definitions cannot acquire trustworthy claim provenance until one is imported.</p>
          </div>
        )}
      </section>

      <section className="section splitSection">
        <div>
          <div className="sectionHeading compact">
            <div>
              <p className="kicker">Current projection</p>
              <h2>Lifecycle gates</h2>
            </div>
            <span className="count">{gates.length}</span>
          </div>
          <div className="gateList">
            {gates.length === 0 ? <div className="empty">No gate has been evaluated.</div> : null}
            {gates.map((gate) => (
              <GateCard key={gate.id} gate={gate} />
            ))}
          </div>
        </div>

        <aside>
          <div className="sectionHeading compact">
            <div>
              <p className="kicker">Exceptional path</p>
              <h2>Focused work</h2>
            </div>
            <span className="count countWarm">{workItems.length}</span>
          </div>
          <div className="workList">
            {workItems.length === 0 ? (
              <div className="notice noticeGood">
                <strong>No current assurance work</strong>
                <p>Latest evaluated targets have no unresolved action.</p>
              </div>
            ) : (
              workItems.map((item) => (
                <article className="workCard" key={item.decisionId}>
                  <div className="cardMeta">
                    <span>{item.stage}</span>
                    <span>{shortRevision(item.subject.revision)}</span>
                  </div>
                  <h3>{item.definitionId}</h3>
                  <ul>
                    {item.work.map((work) => (
                      <li key={work}>{humanize(work)}</li>
                    ))}
                  </ul>
                  <p className="reasonLine">Because {item.reasons.map(humanize).join(', ')}</p>
                </article>
              ))
            )}
          </div>
        </aside>
      </section>
    </div>
  );
}
function GateCard({ gate }: { gate: GateDecisionRecord }) {
  const open = gate.decision.status === 'open';
  return (
    <article className={`gateCard ${open ? 'gateOpen' : 'gateClosed'}`}>
      <div className="gateCardTop">
        <div>
          <span className="stage">{gate.request.stage}</span>
          <h3>{gate.request.definitionId}</h3>
        </div>
        <span className="statusPill">{gate.decision.status}</span>
      </div>
      <dl className="subjectGrid">
        <div>
          <dt>Revision</dt>
          <dd>{shortRevision(gate.request.subject.revision)}</dd>
        </div>
        <div>
          <dt>Environment</dt>
          <dd>{gate.request.subject.environment ?? '—'}</dd>
        </div>
        {gate.request.subject.deploymentId ? (
          <div>
            <dt>Deployment</dt>
            <dd>{gate.request.subject.deploymentId}</dd>
          </div>
        ) : null}
        <div>
          <dt>Observation</dt>
          <dd>{gate.decision.observationId ?? 'none applicable'}</dd>
        </div>
      </dl>
      {!open ? (
        <div className="reasonTags">
          {gate.decision.reasons.map((reason) => (
            <span key={reason}>{humanize(reason)}</span>
          ))}
        </div>
      ) : (
        <p className="basis">Qualified by {gate.decision.qualificationId}; execution is applicable.</p>
      )}
    </article>
  );
}

function Metric({ value, label, tone }: { value: number; label: string; tone: string }) {
  return (
    <div className={`metric metric-${tone}`}>
      <strong>{value}</strong>
      <span>{label}</span>
    </div>
  );
}

function RecordCount({ label, value }: { label: string; value: number }) {
  return (
    <div>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function humanize(value: string) {
  return value.replaceAll('-', ' ');
}

function shortRevision(revision: string) {
  return revision.length > 18 ? `${revision.slice(0, 15)}…` : revision;
}

function shortFingerprint(fingerprint: string) {
  return fingerprint.length > 22 ? `${fingerprint.slice(0, 19)}…` : fingerprint;
}
