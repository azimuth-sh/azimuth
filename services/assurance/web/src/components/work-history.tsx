import type { AgentDescriptor, AttemptUsage, FailureAttribution, ProvenanceAttempt, ProvenanceBasis, RequirementWorkHistory, TokenCounts } from '../lib/assurance';
import { shortIdentifier } from '../lib/identifier';
import { Disclosure } from './disclosure';
import { Markdown } from './markdown';
import { LocalTime } from './local-time';

const basisLabels: Record<ProvenanceBasis, string> = { harness: 'harness-attested', declared: 'declared', unknown: 'unknown' };
const positiveOutcomes = ['completed', 'accepted', 'approved', 'supported'];
const adverseOutcomes = ['rejected', 'failed', 'violated', 'denied'];
const roleLabels = { caused: 'Caused', 'allowed-through': 'Allowed through' };

function attemptAnchor(id: string) {
  return `attempt-${id.replace(/^attempt-/, '')}`;
}
function outcomeTone(outcome: string) {
  return positiveOutcomes.includes(outcome) ? 'good' : adverseOutcomes.includes(outcome) ? 'bad' : 'neutral';
}
function elapsed(from: number | null, to: number | null) {
  if (from === null || to === null || to < from) return null;
  const seconds = Math.round((to - from) / 1000);
  if (seconds < 60) return `${seconds} s`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return seconds % 60 ? `${minutes} min ${seconds % 60} s` : `${minutes} min`;
  return minutes % 60 ? `${Math.floor(minutes / 60)} h ${minutes % 60} min` : `${minutes / 60} h`;
}

// Unknown is rendered as a value of its own so an absent descriptor field is never mistaken for an empty one.
function DescriptorValue({ value, basis }: { value: string | null; basis: ProvenanceBasis }) {
  if (basis === 'unknown' || value === null) return <span className="basisMarker basis-unknown" title="Not recorded; never inferred">unknown</span>;
  return <><span>{value}</span> <span className={`basisMarker basis-${basis}`}>{basisLabels[basis]}</span></>;
}
function harnessValue(harness: AgentDescriptor['harness']) {
  if (harness.basis === 'unknown' || harness.name === null) return null;
  return `${harness.name} ${harness.version ?? '(version unknown)'}`;
}
export function DescriptorFacts({ agent }: { agent: AgentDescriptor }) {
  return <>
    <div><dt>Agent kind</dt><dd><DescriptorValue value={agent.kind.value} basis={agent.kind.basis} /></dd></div>
    <div><dt>Model</dt><dd><DescriptorValue value={agent.model.value} basis={agent.model.basis} /></dd></div>
    <div><dt>Effort</dt><dd><DescriptorValue value={agent.effort.value} basis={agent.effort.basis} /></dd></div>
    <div><dt>Agent type</dt><dd><DescriptorValue value={agent.agent_type.value} basis={agent.agent_type.basis} /></dd></div>
    <div><dt>Harness</dt><dd><DescriptorValue value={harnessValue(agent.harness)} basis={agent.harness.basis} /></dd></div>
  </>;
}

function tokens(count: number) {
  if (count < 1000) return `${count}`;
  const [value, unit] = count < 1_000_000 ? [count / 1000, 'k'] : [count / 1_000_000, 'M'];
  return `${value < 100 ? Number(value.toFixed(1)) : Math.round(value)}${unit}`;
}
function plural(count: number, noun: string) {
  return `${count} ${noun}${count === 1 ? '' : 's'}`;
}
// Cached input is shown beside, never added to, input: providers disagree on whether input includes it.
function tokenSummary(counts: TokenCounts) {
  const cached = counts.cache_read_input_tokens ? ` (+${tokens(counts.cache_read_input_tokens)} cached)` : '';
  return `${tokens(counts.input_tokens)} in${cached} / ${tokens(counts.output_tokens)} out`;
}
function tokenDetail(counts: TokenCounts) {
  const known = (count: number | null) => count === null ? 'not reported' : count.toLocaleString('en-US');
  return `input ${known(counts.input_tokens)}, cache read ${known(counts.cache_read_input_tokens)}, cache write ${known(counts.cache_write_input_tokens)}, output ${known(counts.output_tokens)}, reasoning ${known(counts.reasoning_output_tokens)}`;
}
function UsageValue({ usage }: { usage: AttemptUsage | null }) {
  if (usage === null) return <span className="basisMarker basis-unknown" title="Not recorded; never inferred">unknown</span>;
  const delegated = usage.delegated;
  return <>
    <span title={`${plural(usage.tool_calls, 'tool call')}; ${tokenDetail(usage)}`}>{plural(usage.model_calls, 'call')} · {tokenSummary(usage)}</span>
    {delegated.agents > 0 && <span title={`${plural(delegated.model_calls, 'call')}; ${tokenDetail(delegated)}`}> · {plural(delegated.agents, 'delegated agent')}: {tokenSummary(delegated)}</span>}
    {' '}<span className={`basisMarker basis-${usage.basis}`}>{basisLabels[usage.basis]}</span>
    {usage.overlapping && <> <span className="basisMarker usageOverlapping" title="Another workflow result was recorded inside this window, so these tokens are shared with it and cannot be split">overlapping</span></>}
  </>;
}

function Attempt({ attempt }: { attempt: ProvenanceAttempt }) {
  const duration = elapsed(attempt.handoff_captured_at_ms, attempt.recorded_at_ms);
  return <article className="workAttempt" id={attemptAnchor(attempt.id)}>
    <header><span className="workAttemptSequence">#{attempt.sequence}</span><strong>{attempt.action}</strong><span className={`badge ${outcomeTone(attempt.outcome)}`}>{attempt.outcome}</span><code className="workAttemptActor">{attempt.actor}</code></header>
    <dl className="factGrid">
      <DescriptorFacts agent={attempt.agent} />
      <div><dt>Handoff captured</dt><dd><LocalTime value={attempt.handoff_captured_at_ms} fallback="not recorded" /></dd></div>
      <div><dt>Recorded</dt><dd><LocalTime value={attempt.recorded_at_ms} fallback="not recorded" />{duration && <> · {duration}</>}</dd></div>
      <div className="workUsage"><dt>Token usage</dt><dd><UsageValue usage={attempt.usage} /></dd></div>
    </dl>
    {attempt.report ? <Disclosure summary="Report"><Markdown source={attempt.report} /></Disclosure> : <p className="modelDetailMuted">No report recorded.</p>}
    {(attempt.used.length > 0 || attempt.generated.length > 0) && <Disclosure summary="Used and generated" meta={`${attempt.used.length} used · ${attempt.generated.length} generated`}>
      <ul className="workEntities">
        {attempt.used.map((entity, index) => <li key={`used:${index}`}><span>used</span> {entity.kind} <code title={entity.fingerprint ?? undefined}>{entity.ref}</code></li>)}
        {attempt.generated.map((entity, index) => <li key={`generated:${index}`}><span>generated</span> {entity.kind} <code title={entity.fingerprint ?? undefined}>{entity.record ?? entity.ref}</code></li>)}
      </ul>
    </Disclosure>}
    <p className="workAttemptId">Attempt <code title={attempt.id}>{shortIdentifier(attempt.id)}</code> · workflow instance <code title={attempt.instance}>{shortIdentifier(attempt.instance)}</code> · {attempt.disposition}</p>
  </article>;
}

function Attribution({ attribution, attempts }: { attribution: FailureAttribution; attempts: Map<string, ProvenanceAttempt> }) {
  return <Disclosure
    className="workAttribution"
    summary={<>Failure Attribution <code>{attribution.id}</code></>}
    meta={<><span className="badge neutral">{attribution.confidence} confidence</span><LocalTime value={attribution.recorded_at_ms} /></>}
  >
    <dl className="factGrid">
      <div><dt>Adverse fact</dt><dd>{attribution.adverse.kind} <code title={attribution.adverse.ref}>{shortIdentifier(attribution.adverse.ref)}</code></dd></div>
      <div><dt>Investigator</dt><dd>{attribution.investigator}</dd></div>
      {attribution.revises && <div><dt>Revises</dt><dd><code>{attribution.revises}</code></dd></div>}
    </dl>
    <ul className="workAttributionAttempts">{attribution.attempts.map(reference => {
      const attempt = attempts.get(reference.attempt);
      return <li key={`${reference.role}:${reference.attempt}`}><span className={`badge ${reference.role === 'caused' ? 'bad' : 'warn'}`}>{roleLabels[reference.role]}</span> {attempt ? <a href={`#${attemptAnchor(attempt.id)}`}>{attempt.action} · {attempt.actor}</a> : <span className="modelDetailMuted">attempt not in this history</span>} <code title={reference.attempt}>{shortIdentifier(reference.attempt)}</code></li>;
    })}</ul>
    <Markdown source={attribution.rationale} />
  </Disclosure>;
}

function latestRecorded(changes: RequirementWorkHistory['releases'][number]['changes']) {
  const times = changes.flatMap(change => change.attempts.map(attempt => attempt.recorded_at_ms)).filter((time): time is number => time !== null);
  return times.length ? Math.max(...times) : null;
}
export function reviewAttempt(history: RequirementWorkHistory | null | undefined, reviewFingerprint: string | undefined) {
  if (!history || !reviewFingerprint) return undefined;
  return history.releases.flatMap(release => release.changes.flatMap(change => change.attempts))
    .find(attempt => attempt.action === 'review-requirement' && attempt.generated.some(entity => entity.record_fingerprint === reviewFingerprint));
}
export function ReviewAttempt({ attempt }: { attempt: ProvenanceAttempt }) {
  return <div className="workReviewAttempt">
    <p className="modelDetailMuted">Reviewing attempt from Work Provenance: <a href={`#${attemptAnchor(attempt.id)}`}>{attempt.actor}</a></p>
    <dl className="factGrid"><DescriptorFacts agent={attempt.agent} /></dl>
  </div>;
}

export function WorkHistory({ history }: { history: RequirementWorkHistory | null | undefined }) {
  const attempts = new Map((history?.releases ?? []).flatMap(release => release.changes.flatMap(change => change.attempts)).map(attempt => [attempt.id, attempt]));
  const releases = [...(history?.releases ?? [])].sort((a, b) => b.integration_sequence - a.integration_sequence);
  const attributions = history?.attributions ?? [];
  return <section id="requirement-work-history" className="requirementDetailSection disclosureSection">
    <Disclosure summary="Work history" meta={<><span className="workCaption">Work provenance — not used for Requirement Status</span>{history && <span>{releases.length} {releases.length === 1 ? 'release' : 'releases'} · {attempts.size} {attempts.size === 1 ? 'attempt' : 'attempts'}</span>}</>}>
      <p className="modelDetailMuted">Which workflow attempts changed or reviewed this Requirement, as recorded by the workflow and transported by the publisher. Work history is not evidence: it never affects Verification or Realization Review currentness, Requirement Status or acceptance.</p>
      {history === undefined ? <p className="modelDetailMuted">Work history is unavailable right now; Requirement Status is unaffected.</p>
        : !releases.length ? <p className="modelDetailMuted">No work history recorded for this Requirement.</p>
        : releases.map(release => <div className="workRelease" key={release.release}>
          <div className="workReleaseHeading"><strong>{release.label}</strong><span className={`badge ${release.relation === 'intent' ? 'warn' : 'neutral'}`} title={release.relation === 'intent' ? 'The Requirement or its Scenarios changed' : 'Only a reviewed dependency changed'}>{release.relation}</span><span className="modelDetailMuted">Integration #{release.integration_sequence}</span>{latestRecorded(release.changes) !== null && <span className="modelDetailMuted">last work recorded <LocalTime value={latestRecorded(release.changes)} /></span>}</div>
          {release.changes.map(change => <div className="workChange" key={change.change}>
            <h4>Change <code>{change.change}</code></h4>
            {change.attempts.length ? change.attempts.map(attempt => <Attempt attempt={attempt} key={attempt.id} />) : <p className="modelDetailMuted">No stored ledger for this change.</p>}
          </div>)}
        </div>)}
      {attributions.length > 0 && <div className="workAttributions"><h4>Failure Attributions</h4><p className="modelDetailMuted">An investigator&apos;s judgment of which attempts caused an adverse fact or allowed it through.</p>{attributions.map(attribution => <Attribution attribution={attribution} attempts={attempts} key={attribution.fingerprint} />)}</div>}
    </Disclosure>
  </section>;
}
