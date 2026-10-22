import Link from 'next/link';
import type { CheckResultReference, DeploymentCheckPolicy, VerificationTargetState } from '../lib/assurance';

type RequirementState = VerificationTargetState['requirements'][number];
export interface VerificationCheck {
  id: string;
  fingerprint: string;
  deployment?: DeploymentCheckPolicy | null;
}
export interface RequirementVerificationProps {
  bindings: { check: string; scenario: string }[];
  checks: VerificationCheck[];
  descriptions: { id: string; description: string }[];
  scenarios: { id: string; description: string }[];
  scenarioIds?: string[];
  state?: RequirementState;
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
function outcome(result: CheckResultReference) {
  return result.outcome === 'satisfied' ? 'Passed' : result.outcome === 'violated' ? 'Failed' : 'Inconclusive';
}

export function RequirementVerification(props: RequirementVerificationProps) {
  const bindings = props.bindings.map(binding => {
    const check = props.checks.find(check => check.id === binding.check);
    const assessed = props.state?.bindings?.find(item => item.check === binding.check && item.scenario === binding.scenario);
    return { ...binding, checkDefinition: check, assessed, monitor: assessed?.monitor ?? check?.deployment?.monitor };
  });
  const sections = [
    { id: 'requirement-monitors', title: 'Production monitoring', summary: 'Recurring deployment Checks. A missed or overdue execution leaves required evidence unresolved.', matches: (binding: typeof bindings[number]) => !!binding.monitor },
    { id: 'requirement-deployment', title: 'Deployment verification', summary: 'Checks executed against the observed deployment.', matches: (binding: typeof bindings[number]) => !binding.monitor && binding.checkDefinition?.deployment?.verification_target === 'deployment' },
    { id: 'requirement-implementation', title: 'Implementation verification', summary: 'Source and artifact evidence applied through the release composition.', matches: (binding: typeof bindings[number]) => !binding.monitor && binding.checkDefinition?.deployment?.verification_target !== 'deployment' },
  ];
  const uncoveredScenarios = (props.scenarioIds ?? []).filter(id => !props.bindings.some(binding => binding.scenario === id));
  const requirementNotApplicable = props.state?.status === 'not-applicable';
  return <>{requirementNotApplicable && <section id="requirement-not-applicable" className="requirementDetailSection" aria-label="Not applicable here"><div className="requirementSectionHeading"><h3>Not applicable here</h3><span className="badge neutral">Not applicable</span></div><p className="modelDetailMuted">A required context condition does not hold for this deployment. The Requirement is neither satisfied nor failing here and is not counted as passing.</p></section>}{sections.map(section => {
    const entries = bindings.filter(section.matches);
    return <section id={section.id} className="requirementDetailSection verificationSection" key={section.id} aria-label={section.title}>
      <div className="requirementSectionHeading"><h3>{section.title}</h3><span className="panelCount">{entries.length}</span></div>
      <p className="modelDetailMuted">{section.summary}</p>
      {!entries.length && <p className="verificationEmpty">{section.id === 'requirement-monitors' ? 'No Monitors declared for this Requirement.' : 'No Checks declared in this stage.'}</p>}
      {entries.map(binding => {
        const policy = binding.checkDefinition?.deployment;
        const coverage = props.state?.explanation.coverage.find(item => item.check === binding.check && item.scenario === binding.scenario);
        const results = [...(coverage?.check_results ?? [])].sort((a,b) => b.observed_at_ms - a.observed_at_ms);
        const applicable = results.some(item => item.applicable ?? item.accepted_basis);
        const latest = results[0];
        const evaluated = props.evaluatedAt;
        const expired = !!binding.monitor && !!latest && evaluated !== undefined && (latest.observed_at_ms > evaluated || evaluated - latest.observed_at_ms > binding.monitor.freshness_seconds * 1000);
        const reasons = binding.assessed?.reasons ?? [];
        const overdue = !!binding.monitor && (expired || reasons.some(reason => /monitor.*(?:overdue|stale|expired)/i.test(reason)));
        const notApplicableHere = props.hasDeployment && (requirementNotApplicable || binding.assessed?.status === 'not-applicable');
        const label = !props.hasDeployment ? 'Definition' : notApplicableHere ? 'Not applicable here' : overdue ? 'Overdue' : !latest ? 'No Check Result' : applicable ? latest?.outcome === 'violated' ? 'Failed Check Result' : 'Applicable evidence' : 'Not applicable';
        return <article className="verificationCard" key={`${binding.check}:${binding.scenario}`}>
          <div className="requirementSectionHeading"><h4>{props.descriptions.find(item => item.id === binding.check)?.description ?? binding.check}</h4><span className={`badge ${!props.hasDeployment || notApplicableHere ? 'neutral' : applicable && !overdue ? latest?.outcome === 'violated' ? 'bad' : 'good' : 'warn'}`}>{label}</span></div>
          <p className="verificationScenario">Scenario: {props.scenarios.find(item => item.id === binding.scenario)?.description ?? binding.scenario}</p>
          <dl className="verificationFacts">
            <div><dt>Verification Target</dt><dd>{policy?.verification_target ?? 'Not declared'}</dd></div>
            <div><dt>Components</dt><dd>{policy?.components.join(', ') || 'Not declared'}</dd></div>
            {binding.monitor && <><div><dt>Expected interval</dt><dd>{duration(binding.monitor.interval_seconds)}</dd></div><div><dt>Freshness window</dt><dd>{duration(binding.monitor.freshness_seconds)}</dd></div></>}
            <div><dt>{policy?.verification_target === 'deployment' ? 'Deployment binding' : 'Release lineage'}</dt><dd>{!props.hasDeployment ? 'Select a deployment' : applicable ? policy?.verification_target === 'deployment' ? 'Exact deployment verified' : 'Verified through the deployed composition' : 'Not established for this evidence'}</dd></div>
            <div><dt>Execution Context</dt><dd>{policy ? policy.contexts.length ? applicable ? `${policy.contexts.length} required ${policy.contexts.length === 1 ? 'Context Contract matched' : 'Context Contracts matched'}` : 'Required Execution Context match not established' : 'No Execution Context dependency declared' : 'Policy not declared'}</dd></div>
            {latest && <div><dt>Latest Check Result</dt><dd><time dateTime={new Date(latest.observed_at_ms).toISOString()}>{time(latest.observed_at_ms)}</time></dd></div>}
          </dl>
          {reasons.length > 0 && <ul className="verificationReasons">{reasons.map((reason,index) => <li key={index}>{reason}</li>)}</ul>}
          {!props.hasDeployment ? <p className="modelDetailMuted">Open a deployment context to inspect applicable evidence.</p> : !results.length ? <p className="modelDetailMuted">No current Check Result is available for this Check and Scenario.</p> : results.map((result,index) => <div className="recordedResult" key={`${result.run}:${result.check_result}:${index}`}>
            <span className={`badge ${result.outcome === 'satisfied' ? 'good' : result.outcome === 'violated' ? 'bad' : 'warn'}`}>{outcome(result)}</span>
            <small className="modelDetailMuted">{(result.applicable ?? result.accepted_basis) ? 'Applicable to this deployment' : 'Retained fact · not applicable to this deployment'}</small>
            <Link className="textLink" href={`${props.projectRoot}/runs/${encodeURIComponent(result.run)}`}>View Run ↗</Link>
            {result.diagnostics?.length ? <p>{result.diagnostics.join(' ')}</p> : null}
          </div>)}
          {policy?.configuration_sources?.length ? <p className="modelDetailMuted">Configuration sources: {policy.configuration_sources.join(', ')}</p> : null}
          {policy?.units?.length ? <details className="verificationContracts"><summary>Execution coverage · {policy.units.length} required units{policy.controls?.length ? ` · ${policy.controls.length} control profiles` : ''}</summary><ul>{policy.units.map(unit => <li key={unit}><code>{unit}</code>{policy.controls?.some(control => control.unit === unit) ? ' · sensitivity control' : ''}</li>)}</ul></details> : null}
          {policy?.contexts.length ? <details className="verificationContracts"><summary>Required Context Contracts</summary><ul>{policy.contexts.map(revision => <li key={revision}><code>{revision}</code></li>)}</ul></details> : null}
        </article>;
      })}
    </section>;
  })}{uncoveredScenarios.length > 0 && <section className="requirementDetailSection" aria-label="Scenarios without contributing Checks"><h3>Scenarios without contributing Checks</h3><p className="modelDetailMuted">These Scenarios have no declared Check binding; no supporting evidence is inferred.</p><ul>{uncoveredScenarios.map(id => <li key={id}>{props.scenarios.find(item => item.id === id)?.description ?? id}</li>)}</ul></section>}</>;
}
