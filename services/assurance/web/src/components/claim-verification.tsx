import Link from 'next/link';
import type { DeploymentCheckPolicy, StatusObservation, SubjectState } from '../lib/assurance';

type ClaimState = SubjectState['claims'][number];
export interface VerificationCheck {
  id: string;
  fingerprint: string;
  deployment?: DeploymentCheckPolicy | null;
}
export interface ClaimVerificationProps {
  bindings: { check: string; case: string }[];
  checks: VerificationCheck[];
  descriptions: { id: string; description: string }[];
  cases: { id: string; description: string }[];
  caseIds?: string[];
  state?: ClaimState;
  evaluatedAt?: number;
  hasDeployment: boolean;
  projectRoot: string;
}
function time(value: number) {
  return new Date(value).toLocaleString('en-GB', { timeZone: 'UTC', dateStyle: 'medium', timeStyle: 'short' }) + ' UTC';
}
function duration(seconds: number) {
  return seconds % 60 === 0 ? `${seconds / 60} min` : `${seconds} sec`;
}
function outcome(observation: StatusObservation) {
  return observation.outcome === 'satisfied' ? 'Satisfied' : observation.outcome === 'violated' ? 'Adverse observation' : 'Inconclusive';
}

export function ClaimVerification(props: ClaimVerificationProps) {
  const bindings = props.bindings.map(binding => {
    const check = props.checks.find(check => check.id === binding.check);
    const assessed = props.state?.bindings?.find(item => item.check === binding.check && item.case === binding.case);
    return { ...binding, checkDefinition: check, assessed, monitor: assessed?.monitor ?? check?.deployment?.monitor };
  });
  const sections = [
    { id: 'claim-monitors', title: 'Production monitoring', summary: 'Recurring deployment Checks. A missed or overdue execution leaves required evidence unresolved.', matches: (binding: typeof bindings[number]) => !!binding.monitor },
    { id: 'claim-deployment', title: 'Deployment verification', summary: 'Checks executed against the observed deployment.', matches: (binding: typeof bindings[number]) => !binding.monitor && binding.checkDefinition?.deployment?.subject === 'deployment' },
    { id: 'claim-implementation', title: 'Implementation verification', summary: 'Source and artifact evidence applied through the release composition.', matches: (binding: typeof bindings[number]) => !binding.monitor && binding.checkDefinition?.deployment?.subject !== 'deployment' },
  ];
  const uncoveredCases = (props.caseIds ?? []).filter(id => !props.bindings.some(binding => binding.case === id));
  return <>{sections.map(section => {
    const entries = bindings.filter(section.matches);
    return <section id={section.id} className="claimDetailSection verificationSection" key={section.id} aria-label={section.title}>
      <div className="claimSectionHeading"><h3>{section.title}</h3><span className="panelCount">{entries.length}</span></div>
      <p className="modelDetailMuted">{section.summary}</p>
      {!entries.length && <p className="verificationEmpty">{section.id === 'claim-monitors' ? 'No Monitors declared for this Claim.' : 'No Checks declared in this stage.'}</p>}
      {entries.map(binding => {
        const policy = binding.checkDefinition?.deployment;
        const coverage = props.state?.explanation.coverage.find(item => item.check === binding.check && item.case === binding.case);
        const observations = [...(coverage?.observations ?? [])].sort((a,b) => b.observed_at_ms - a.observed_at_ms);
        const applicable = observations.some(item => item.applicable ?? item.accepted_basis);
        const latest = observations[0];
        const evaluated = props.evaluatedAt;
        const expired = !!binding.monitor && !!latest && evaluated !== undefined && (latest.observed_at_ms > evaluated || evaluated - latest.observed_at_ms > binding.monitor.freshness_seconds * 1000);
        const reasons = binding.assessed?.reasons ?? [];
        const overdue = !!binding.monitor && (expired || reasons.some(reason => /monitor.*(?:overdue|stale|expired)/i.test(reason)));
        const label = !props.hasDeployment ? 'Definition' : overdue ? 'Overdue' : !latest ? 'No observation' : applicable ? latest?.outcome === 'violated' ? 'Adverse observation' : 'Applicable evidence' : 'Not applicable';
        return <article className="verificationCard" key={`${binding.check}:${binding.case}`}>
          <div className="claimSectionHeading"><h4>{props.descriptions.find(item => item.id === binding.check)?.description ?? binding.check}</h4><span className={`badge ${!props.hasDeployment ? 'neutral' : applicable && !overdue ? latest?.outcome === 'violated' ? 'bad' : 'good' : 'warn'}`}>{label}</span></div>
          <p className="verificationCase">Case: {props.cases.find(item => item.id === binding.case)?.description ?? binding.case}</p>
          <dl className="verificationFacts">
            <div><dt>Execution Subject</dt><dd>{policy?.subject ?? 'Not declared'}</dd></div>
            <div><dt>Components</dt><dd>{policy?.components.join(', ') || 'Not declared'}</dd></div>
            {binding.monitor && <><div><dt>Expected interval</dt><dd>{duration(binding.monitor.interval_seconds)}</dd></div><div><dt>Freshness window</dt><dd>{duration(binding.monitor.freshness_seconds)}</dd></div></>}
            <div><dt>{policy?.subject === 'deployment' ? 'Deployment binding' : 'Release lineage'}</dt><dd>{!props.hasDeployment ? 'Select a deployment' : applicable ? policy?.subject === 'deployment' ? 'Exact deployment verified' : 'Verified through the deployed composition' : 'Not established for this evidence'}</dd></div>
            <div><dt>Verification context</dt><dd>{policy ? policy.contexts.length ? applicable ? `${policy.contexts.length} required ${policy.contexts.length === 1 ? 'contract matched' : 'contracts matched'}` : 'Required context match not established' : 'No context dependency declared' : 'Policy not declared'}</dd></div>
            {latest && <div><dt>Latest observation</dt><dd><time dateTime={new Date(latest.observed_at_ms).toISOString()}>{time(latest.observed_at_ms)}</time></dd></div>}
          </dl>
          {reasons.length > 0 && <ul className="verificationReasons">{reasons.map((reason,index) => <li key={index}>{reason}</li>)}</ul>}
          {!props.hasDeployment ? <p className="modelDetailMuted">Open a deployment context to inspect applicable evidence.</p> : !observations.length ? <p className="modelDetailMuted">No current observation is available for this Check and Case.</p> : observations.map((observation,index) => <div className="recordedResult" key={`${observation.run}:${observation.observation}:${index}`}>
            <span className={`badge ${observation.outcome === 'satisfied' ? 'good' : observation.outcome === 'violated' ? 'bad' : 'warn'}`}>{outcome(observation)}</span>
            <small className="modelDetailMuted">{(observation.applicable ?? observation.accepted_basis) ? 'Applicable to this deployment' : 'Retained fact · not applicable to this deployment'}</small>
            <Link className="textLink" href={`${props.projectRoot}/runs/${encodeURIComponent(observation.run)}`}>View Run ↗</Link>
            {observation.diagnostics?.length ? <p>{observation.diagnostics.join(' ')}</p> : null}
          </div>)}
          {policy?.configuration_sources?.length ? <p className="modelDetailMuted">Configuration sources: {policy.configuration_sources.join(', ')}</p> : null}
          {policy?.units?.length ? <details className="verificationContracts"><summary>Execution coverage · {policy.units.length} required units{policy.controls?.length ? ` · ${policy.controls.length} control profiles` : ''}</summary><ul>{policy.units.map(unit => <li key={unit}><code>{unit}</code>{policy.controls?.some(control => control.unit === unit) ? ' · sensitivity control' : ''}</li>)}</ul></details> : null}
          {policy?.contexts.length ? <details className="verificationContracts"><summary>Required context contracts</summary><ul>{policy.contexts.map(revision => <li key={revision}><code>{revision}</code></li>)}</ul></details> : null}
        </article>;
      })}
    </section>;
  })}{uncoveredCases.length > 0 && <section className="claimDetailSection" aria-label="Cases without contributing Checks"><h3>Cases without contributing Checks</h3><p className="modelDetailMuted">These Cases have no declared Check binding; no supporting evidence is inferred.</p><ul>{uncoveredCases.map(id => <li key={id}>{props.cases.find(item => item.id === id)?.description ?? id}</li>)}</ul></section>}</>;
}
