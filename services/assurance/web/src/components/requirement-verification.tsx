import Link from 'next/link';
import type { CheckResultReference, DeploymentCheckPolicy, VerificationTargetState } from '../lib/assurance';
import { Disclosure } from './disclosure';
import { Markdown } from './markdown';
import { LocalTime } from './local-time';

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
function duration(seconds: number) {
  return seconds % 60 === 0 ? `${seconds / 60} min` : `${seconds} sec`;
}
function outcome(result: CheckResultReference) {
  return result.outcome === 'satisfied' ? 'Passed' : result.outcome === 'violated' ? 'Failed' : 'Inconclusive';
}
const labelPriority = ['Failed Check Result', 'Overdue', 'No Check Result', 'Not applicable', 'Definition', 'Applicable evidence'];

export function RequirementVerification(props: RequirementVerificationProps) {
  const bindings = props.bindings.map(binding => {
    const check = props.checks.find(check => check.id === binding.check);
    const assessed = props.state?.bindings?.find(item => item.check === binding.check && item.scenario === binding.scenario);
    const monitor = assessed?.monitor ?? check?.deployment?.monitor;
    const coverage = props.state?.explanation.coverage.find(item => item.check === binding.check && item.scenario === binding.scenario);
    const results = [...(coverage?.check_results ?? [])].sort((a, b) => b.observed_at_ms - a.observed_at_ms);
    const applicable = results.some(item => item.applicable ?? item.accepted_basis);
    const latest = results[0];
    const evaluated = props.evaluatedAt;
    const expired = !!monitor && !!latest && evaluated !== undefined && (latest.observed_at_ms > evaluated || evaluated - latest.observed_at_ms > monitor.freshness_seconds * 1000);
    const reasons = assessed?.reasons ?? [];
    const overdue = !!monitor && (expired || reasons.some(reason => /monitor.*(?:overdue|stale|expired)/i.test(reason)));
    const label = !props.hasDeployment ? 'Definition' : overdue ? 'Overdue' : !latest ? 'No Check Result' : applicable ? latest.outcome === 'violated' ? 'Failed Check Result' : 'Applicable evidence' : 'Not applicable';
    return { ...binding, checkDefinition: check, monitor, results, applicable, latest, reasons, label };
  });
  const checks = [...new Set(bindings.map(binding => binding.check))].map(id => {
    const entries = bindings.filter(binding => binding.check === id);
    const policy = entries[0].checkDefinition?.deployment;
    const monitor = entries.find(entry => entry.monitor)?.monitor;
    const applicable = entries.every(entry => entry.applicable);
    const label = labelPriority.find(candidate => entries.some(entry => entry.label === candidate)) ?? 'Applicable evidence';
    const latest = entries.map(entry => entry.latest).filter((item): item is CheckResultReference => !!item).sort((a, b) => b.observed_at_ms - a.observed_at_ms)[0];
    const facts = {
      'Verification Target': policy?.verification_target ?? 'Not declared',
      Components: policy?.components.join(', ') || 'Not declared',
      [policy?.verification_target === 'deployment' ? 'Deployment binding' : 'Release lineage']: !props.hasDeployment ? 'Select a deployment' : applicable ? policy?.verification_target === 'deployment' ? 'Exact deployment verified' : 'Verified through the deployed composition' : 'Not established for this evidence',
      'Execution Context': policy ? policy.contexts.length ? applicable ? `${policy.contexts.length} required ${policy.contexts.length === 1 ? 'Context Contract matched' : 'Context Contracts matched'}` : 'Required Execution Context match not established' : 'No Execution Context dependency declared' : 'Policy not declared',
    } as Record<string, string>;
    return { id, entries, policy, monitor, label, latest, facts };
  });
  const sections = [
    { id: 'requirement-monitors', title: 'Production monitoring', summary: 'Recurring deployment Checks. A missed or overdue execution leaves required evidence unresolved.', matches: (check: typeof checks[number]) => !!check.monitor },
    { id: 'requirement-deployment', title: 'Deployment verification', summary: 'Checks executed against the observed deployment.', matches: (check: typeof checks[number]) => !check.monitor && check.policy?.verification_target === 'deployment' },
    { id: 'requirement-implementation', title: 'Implementation verification', summary: 'Source and artifact evidence applied through the release composition.', matches: (check: typeof checks[number]) => !check.monitor && check.policy?.verification_target !== 'deployment' },
  ];
  const uncoveredScenarios = (props.scenarioIds ?? []).filter(id => !props.bindings.some(binding => binding.scenario === id));
  const scenarioText = (id: string) => props.scenarios.find(item => item.id === id)?.description;
  return <>{sections.map(section => {
    const entries = checks.filter(section.matches);
    if (!entries.length) return <section id={section.id} className="requirementDetailSection verificationSection compact" key={section.id} aria-label={section.title}>
      <p className="stageEmpty"><strong>{section.title}</strong> · {section.id === 'requirement-monitors' ? 'no Monitors declared' : 'no Checks declared'}</p>
    </section>;
    const shared = entries.length > 1 ? Object.keys(entries[0].facts).filter(key => entries.every(entry => entry.facts[key] === entries[0].facts[key])) : [];
    return <section id={section.id} className="requirementDetailSection verificationSection" key={section.id} aria-label={section.title}>
      <div className="requirementSectionHeading"><h3>{section.title}</h3><span className="panelCount">{entries.length}</span></div>
      <p className="modelDetailMuted">{section.summary}</p>
      {shared.length > 0 && <p className="stageFacts">{shared.map(key => entries[0].facts[key]).join(' · ')}</p>}
      {entries.map(check => {
        const { policy, monitor } = check;
        const own = Object.keys(check.facts).filter(key => !shared.includes(key));
        const reasons = check.entries.flatMap(entry => entry.reasons);
        const resultNotes = check.entries.flatMap(entry => entry.results.flatMap(result => result.diagnostics ?? []));
        const overdue = check.label === 'Overdue';
        const badge = !props.hasDeployment ? 'neutral' : check.label === 'Failed Check Result' ? 'bad' : check.label === 'Applicable evidence' ? 'good' : 'warn';
        return <Disclosure
          key={check.id}
          className="checkRow"
          summary={<><strong>{check.id}</strong><span className="scenarioChips">{check.entries.map(entry => <code key={entry.scenario}>{entry.scenario}</code>)}</span></>}
          meta={<>{check.latest && <LocalTime value={check.latest.observed_at_ms} />}<span className={`badge ${badge}`}>{check.label}</span></>}
        >
          {props.descriptions.find(item => item.id === check.id)?.description && <Markdown source={props.descriptions.find(item => item.id === check.id)!.description} />}
          <dl className="factGrid">
            {own.map(key => <div key={key}><dt>{key}</dt><dd>{check.facts[key]}</dd></div>)}
            {monitor && <><div><dt>Expected interval</dt><dd>{duration(monitor.interval_seconds)}</dd></div><div><dt>Freshness window</dt><dd>{duration(monitor.freshness_seconds)}</dd></div></>}
            {overdue && check.latest && <div><dt>Latest Check Result</dt><dd><LocalTime value={check.latest.observed_at_ms} /></dd></div>}
          </dl>
          {check.entries.map(entry => <Disclosure key={entry.scenario} summary={<code>{entry.scenario}</code>} meta={<span className={`badge ${!props.hasDeployment ? 'neutral' : entry.label === 'Failed Check Result' ? 'bad' : entry.label === 'Applicable evidence' ? 'good' : 'warn'}`}>{entry.label}</span>}>
            {scenarioText(entry.scenario) ? <Markdown source={scenarioText(entry.scenario)!} /> : <p className="modelDetailMuted">No Scenario description available.</p>}
            {!props.hasDeployment ? <p className="modelDetailMuted">Open a deployment context to inspect applicable evidence.</p> : !entry.results.length ? <p className="modelDetailMuted">No current Check Result is available for this Check and Scenario.</p> : entry.results.map((result, index) => <div className="recordedResult" key={`${result.run}:${result.check_result}:${index}`}>
              <span className={`badge ${result.outcome === 'satisfied' ? 'good' : result.outcome === 'violated' ? 'bad' : 'warn'}`}>{outcome(result)}</span>
              <small className="modelDetailMuted">{(result.applicable ?? result.accepted_basis) ? 'Applicable to this deployment' : 'Retained fact · not applicable to this deployment'}</small>
              <LocalTime className="modelDetailMuted" value={result.observed_at_ms} />
              <Link className="textLink" href={`${props.projectRoot}/runs/${encodeURIComponent(result.run)}`}>View Run ↗</Link>
            </div>)}
          </Disclosure>)}
          {(reasons.length > 0 || resultNotes.length > 0 || policy?.configuration_sources?.length || policy?.units?.length || policy?.contexts.length) ? <Disclosure summary="Diagnostics" meta={reasons.length + resultNotes.length || undefined}>
            {reasons.length > 0 && <ul className="verificationReasons">{reasons.map((reason, index) => <li key={index}>{reason}</li>)}</ul>}
            {resultNotes.length > 0 && <ul className="verificationReasons">{resultNotes.map((note, index) => <li key={index}>{note}</li>)}</ul>}
            {policy?.configuration_sources?.length ? <p className="modelDetailMuted">Configuration sources: {policy.configuration_sources.join(', ')}</p> : null}
            {policy?.units?.length ? <Disclosure summary={`Execution coverage · ${policy.units.length} required units${policy.controls?.length ? ` · ${policy.controls.length} control profiles` : ''}`}><ul>{policy.units.map(unit => <li key={unit}><code>{unit}</code>{policy.controls?.some(control => control.unit === unit) ? ' · sensitivity control' : ''}</li>)}</ul></Disclosure> : null}
            {policy?.contexts.length ? <Disclosure summary="Required Context Contracts"><ul>{policy.contexts.map(dependency => <li key={dependency.component}><code>{dependency.component}</code> · {dependency.leaves.join(', ')}</li>)}</ul></Disclosure> : null}
          </Disclosure> : null}
        </Disclosure>;
      })}
    </section>;
  })}{uncoveredScenarios.length > 0 && <section className="requirementDetailSection" aria-label="Scenarios without contributing Checks"><Disclosure summary="Scenarios without contributing Checks" meta={uncoveredScenarios.length}><p className="modelDetailMuted">These Scenarios have no declared Check binding; no supporting evidence is inferred.</p><ul>{uncoveredScenarios.map(id => <li key={id}><code>{id}</code></li>)}</ul></Disclosure></section>}</>;
}
