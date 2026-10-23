import type {
  CheckRunUnit,
  DeploymentRequirementChecks,
  ReleaseCheckRun,
  ReleaseRequirementChecks,
  RunSubject,
} from "@/lib/checks-run";
import { Identifier } from "./identifier";
import { LocalTime } from "./local-time";

function shortCommit(subject: RunSubject | null) {
  if (subject?.commit) return subject.commit.slice(0, 8);
  if (subject?.artifacts?.length) return subject.artifacts.map(a => a.id).join(", ");
  return "unknown subject";
}
function RunLine({ run, label }: { run: { run_id: string; executed_at_ms: number }; label: string }) {
  return <>{label} · Run <Identifier value={run.run_id} /> · <LocalTime value={run.executed_at_ms} /></>;
}

export function ReleaseCheckRuns({ requirements }: { requirements: ReleaseRequirementChecks[] }) {
  const withChecks = requirements.filter(requirement => requirement.checks.length);
  return <section className="dashboardPanel" aria-label="Checks run for this release">
    <header className="panelHeading"><div><h2>Checks run</h2><p>The Runs behind each Requirement. Applicability is confirmed per environment once deployed.</p></div></header>
    {withChecks.map(requirement => {
      const skipped = requirement.checks.filter(check => check.state === "skipped" || check.state === "inconclusive" || check.state === "missing").length;
      return <details className="releaseCompactRow" key={requirement.requirement} style={{ display: "block" }}>
        <summary><strong>{requirement.requirement}</strong> <span className="modelDetailMuted">{requirement.checks.length} {requirement.checks.length === 1 ? "Check" : "Checks"}</span>{skipped > 0 && <> <span className="badge neutral">{skipped} not conclusive</span></>}</summary>
        {requirement.checks.map(check => <div key={check.check_id}>
          <p><strong>{check.check_id}</strong> {check.state === "skipped" && <span className="badge neutral">Skipped tests — not evidence</span>}{check.state === "inconclusive" && <span className="badge neutral">Inconclusive</span>}{check.state === "missing" && <span className="badge neutral">No Run</span>}</p>
          {check.executed_for_release.map((run: ReleaseCheckRun) => <small key={run.run_fingerprint}><RunLine run={run} label="Executed for this release" /></small>)}
          {check.would_apply.map((run: ReleaseCheckRun) => <small key={run.run_fingerprint}><RunLine run={run} label={`Would apply (executed for ${shortCommit(run.subject)})`} /></small>)}
        </div>)}
      </details>;
    })}
    {!withChecks.length && <p className="releaseEmpty">No Checks are bound to this release's Requirements.</p>}
  </section>;
}

export function DeploymentCheckRuns({ environment, deploymentId, requirements }: {
  environment: string;
  deploymentId: string;
  requirements: DeploymentRequirementChecks[];
}) {
  const withChecks = requirements.filter(requirement => requirement.checks?.length);
  return <section className="dashboardPanel" aria-label={`Checks relied on in ${environment}`}>
    <header className="panelHeading"><div><h2>Checks behind Status in {environment}</h2><p>Runs executed for this release and Runs relied on from elsewhere.</p></div></header>
    {withChecks.map(requirement => <details className="releaseCompactRow" key={requirement.requirement} style={{ display: "block" }}>
      <summary><strong>{requirement.requirement}</strong> <span className="modelDetailMuted">{requirement.status}</span></summary>
      {requirement.checks.map(check => <div key={check.check_id}>
        <p><strong>{check.check_id}</strong> {check.outcome === "skipped" && <span className="badge neutral">Skipped</span>}{check.outcome === "inconclusive" && <span className="badge neutral">Inconclusive</span>}{check.outcome === "missing" && <span className="badge neutral">No applicable Run</span>}{check.outcome === "counterexample" && <span className="badge neutral">Counterexample</span>}</p>
        {check.runs.map(run => <small key={run.run_fingerprint}>
          {run.executed_for_release
            ? <RunLine run={run} label="Executed for this release" />
            : <RunLine run={run} label={`Relied on (reused from ${shortCommit(run.subject)})`} />}
        </small>)}
        {!check.runs.length && <small>No Run in the evidence basis.</small>}
      </div>)}
    </details>)}
    {!withChecks.length && <p className="releaseEmpty">No Check evidence recorded for this deployment.</p>}
  </section>;
}
