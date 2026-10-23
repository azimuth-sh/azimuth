import { selectedEnvironment } from "@/lib/environment";
import Link from "next/link";
import { notFound } from "next/navigation";
import { readProject, routeFingerprint } from "@/lib/assurance";
import { requireViewer } from "@/lib/auth";
import { environmentView, requirementStatus } from "@/lib/production";
import { CopyValue } from "@/components/copy-value";
import { Identifier } from "@/components/identifier";
import { Disclosure } from "@/components/disclosure";
import { shortIdentifier } from "@/lib/identifier";
import { DeploymentCheckRuns, ReleaseCheckRuns } from "@/components/check-runs";
import { readDeploymentChecks, readReleaseChecks } from "@/lib/checks-run";

export const dynamic = "force-dynamic";
type Identity = Record<string, unknown>;
type Ratification = { status: "ratified" | "unratified"; reason?: string; pull_request?: string | number; verified_at_ms: number };
type Release = {
  fingerprint: string;
  label: string;
  integration: { commit: string; main_sequence: number; ratification: Ratification | null } | null;
  model_revision: string;
  changes: Identity[];
  sources: Identity[];
  builds: Identity[];
  artifacts: Identity[];
  components?: { id: string; digest: string }[];
};
function title(record: Identity, fallback: string) {
  for (const key of ["label", "name", "id", "repository"]) {
    if (typeof record[key] === "string") return record[key];
  }
  return fallback;
}
function receivedAt(value: string) {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? "Time unavailable" : new Intl.DateTimeFormat("en-GB", {
    dateStyle: "medium", timeStyle: "short", timeZone: "UTC",
  }).format(date) + " UTC";
}
export default async function ReleasePage({ params, searchParams }: {
  params: Promise<{ projectId: string; release: string }>;
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}) {
  const { projectId, release: segment } = await params;
  const environment = await selectedEnvironment(projectId, (await searchParams).environment);
  await requireViewer(projectId);
  const fingerprint = routeFingerprint(segment);
  if (!fingerprint) notFound();
  const record = await readProject<{
    publication: { release: Release; evidence: Identity[] };
    submitted_by: string;
    submitted_at: string;
    deployments: Identity[];
  }>(projectId, `/releases/${encodeURIComponent(fingerprint)}`);
  if (!record) notFound();
  const release = record.publication.release;
  const ratification = release.integration?.ratification ?? null;
  const commit = release.integration?.commit.slice(0, 8) ?? "";
  const pullRequest = ratification?.pull_request === undefined ? null : String(ratification.pull_request);
  const root = `/projects/${encodeURIComponent(projectId)}`;
  const environmentState = await environmentView(projectId, null, environment);
  const selectedRelease = environmentState.deployment?.release_fingerprint === fingerprint;
  const confirmedDeployment = selectedRelease && environmentState.observationFresh;
  const requirements = selectedRelease ? environmentState.state?.requirements ?? [] : [];
  const counts = { supported: 0, violated: 0, unresolved: 0, "not-applicable": 0 };
  for (const requirement of requirements) counts[requirementStatus(requirement.status)]++;
  const components = release.components ?? [];
  const selectedDeploymentId = selectedRelease ? environmentState.deployment?.deployment.id : undefined;
  const [releaseChecks, deploymentChecks] = await Promise.all([
    readReleaseChecks(projectId, fingerprint),
    selectedDeploymentId ? readDeploymentChecks(projectId, selectedDeploymentId) : Promise.resolve(null),
  ]);
  const modelUrl = `${root}/model?environment=${encodeURIComponent(environment)}&revision=${encodeURIComponent(release.model_revision)}${selectedRelease && environmentState.verificationTarget ? `&verification-target=${encodeURIComponent(environmentState.verificationTarget)}` : ""}`;
  return <div className="dashboard releaseWorkspace">
    <nav className="pageBreadcrumb" aria-label="Breadcrumb"><Link href={`${root}/releases?environment=${encodeURIComponent(environment)}`}>Releases</Link><span>/</span><span>{release.label}</span></nav>
    <header className="dashboardHeading">
      <div><p className="overline">RELEASE</p><h1>{release.label}</h1><p>Published {receivedAt(record.submitted_at)}</p></div>
      <span className={`badge ${release.integration ? "good" : "neutral"}`}>{release.integration ? `Integrated at ${release.integration.commit.slice(0, 8)}` : "No integration commit"}</span>
    </header>
    <section className="deploymentBar" aria-label="Release summary">
      <div><span className="fieldLabel">{environment.toUpperCase()}</span><strong>{confirmedDeployment ? `Observed in ${environment}` : selectedRelease ? "Current state unconfirmed" : "Not confirmed as current"}</strong><small>{selectedRelease && environmentState.deployment ? <Identifier value={environmentState.deployment.deployment.id} /> : `No matching current ${environment} deployment`}</small></div>
      <div><span className="fieldLabel">SOURCES</span><strong>{release.sources.length} {release.sources.length === 1 ? "repository" : "repositories"}</strong></div>
      <Link className="outlineButton" href={modelUrl}>Inspect Requirements ↗</Link>
    </section>
    <div className="releaseColumns">
      <div className="releaseSections">
        {release.changes.length > 0 && <section className="dashboardPanel">
          <header className="panelHeading"><div><h2>Included changes</h2><p>Changes recorded with this release.</p></div><span className="badge neutral">{release.changes.length}</span></header>
          {release.changes.map((change, i) => <div className="releaseCompactRow" key={i}><strong>{title(change, `Change ${i + 1}`)}</strong>{typeof change.revision === "string" && <span className="modelDetailMuted">Pinned revision</span>}</div>)}
        </section>}
        <section className="dashboardPanel">
          <header className="panelHeading"><div><h2>Source repositories</h2><p>Exact source versions used for this release.</p></div></header>
          {release.sources.map((source, i) => <div className="releaseCompactRow" key={i}><strong>{title(source, `Repository ${i + 1}`)}</strong><span className="modelDetailMuted">{typeof source.commit === "string" && /^[0-9a-f]{40,64}$/.test(source.commit) ? source.commit.slice(0, 8) : "Source snapshot recorded"}</span></div>)}
          {!release.sources.length && <p className="releaseEmpty">No sources recorded.</p>}
        </section>
        <section className="dashboardPanel">
          <header className="panelHeading"><div><h2>Components</h2><p>{components.length ? "Runtime components bound to immutable build outputs." : `${release.artifacts.length} artifacts · ${release.builds.length} builds. Component bindings are not recorded.`}</p></div></header>
          {components.map(component => <div className="releaseCompactRow" key={component.id}><strong>{component.id}</strong><span className="badge neutral">Pinned image</span></div>)}
          {!components.length && release.artifacts.map((artifact, i) => <div className="releaseCompactRow" key={i}><strong>{title(artifact, `Artifact ${i + 1}`)}</strong><span className="modelDetailMuted">{typeof artifact.kind === "string" ? artifact.kind : "Build output"}</span></div>)}
          {!components.length && !release.artifacts.length && <p className="releaseEmpty">No build artifacts recorded.</p>}
        </section>
        <section className="dashboardPanel">
          <header className="panelHeading"><div><h2>Deployment history</h2><p>Recorded deployments of this release.</p></div></header>
          {record.deployments.map((deployment, i) => <div className="releaseCompactRow" key={i}><div><strong>{typeof deployment.environment === "string" ? deployment.environment : "Environment unavailable"}</strong><small>{shortIdentifier(title(deployment, `Deployment ${i + 1}`))}</small></div>{typeof deployment.verification_target_fingerprint === "string" && <Link className="textLink" href={`${root}/model?environment=${encodeURIComponent(String(deployment.environment))}&revision=${encodeURIComponent(release.model_revision)}&verification-target=${encodeURIComponent(deployment.verification_target_fingerprint)}`}>Inspect Requirements ↗</Link>}</div>)}
          {!record.deployments.length && <p className="releaseEmpty">No deployment recorded yet.</p>}
        </section>
      </div>
      <aside className="releaseSections">
        <section className="dashboardPanel releaseAssessment">
          <p className="overline">{environment.toUpperCase()} EVIDENCE</p><h2>{confirmedDeployment ? "Current Requirement Status" : "Current state unconfirmed"}</h2>
          {requirements.length ? <><p>{confirmedDeployment ? "Evidence for the observed deployment." : "Recorded evidence; current environment is not confirmed."}</p><dl className="releaseStatusCounts"><div><dt>Satisfied</dt><dd>{counts.supported}</dd></div><div><dt>Not satisfied</dt><dd>{counts.violated}</dd></div><div><dt>Unresolved</dt><dd>{counts.unresolved}</dd></div><div><dt>Not applicable here</dt><dd>{counts["not-applicable"]}</dd></div></dl><Link className="textLink" href={modelUrl}>Inspect evidence ↗</Link></> : <p>No Requirement Status available for a matching selected environment deployment.</p>}
        </section>
        <section className="dashboardPanel releaseAssessment">
          <p className="overline">INTEGRATION</p><h2>{release.integration ? `Integrated at commit ${release.integration.commit.slice(0, 8)}` : "No integration recorded"}</h2>
          {release.integration ? <>
            <p>Main sequence {release.integration.main_sequence}.</p>
            {ratification?.status === "ratified" && <p><span className="badge good">Ratified at {commit} — verified by CI</span></p>}
            {ratification?.status === "unratified" && <p><span className="badge neutral">Unratified</span>{ratification.reason ? ` ${ratification.reason}` : ""}{pullRequest && (/^https?:\/\//.test(pullRequest) ? <> · <a className="textLink" href={pullRequest} rel="noreferrer">Pull request ↗</a></> : ` · Pull request ${pullRequest}`)}</p>}
            {!ratification && <p className="modelDetailMuted">Not on main</p>}
          </> : <p>This release was published without an integration attestation.</p>}
        </section>
        {releaseChecks && <ReleaseCheckRuns requirements={releaseChecks} />}
        {selectedDeploymentId && deploymentChecks ? <DeploymentCheckRuns environment={environment} deploymentId={selectedDeploymentId} requirements={deploymentChecks} />
          : <section className="dashboardPanel"><header className="panelHeading"><div><h2>Runs relied on</h2><p>Appears per environment once this release is deployed there. Select an environment where it runs to see it.</p></div></header></section>}
        {/* C4: delivery-gap and rollback-candidate panels go here. */}
      </aside>
    </div>
    <details className="dashboardPanel releaseTechnical"><summary>Technical details</summary>
      <CopyValue label="Release fingerprint" value={release.fingerprint} />
      {release.integration && <CopyValue label="Integration commit" value={release.integration.commit} />}
      <CopyValue label="Release model" value={release.model_revision} />
      <p className="modelDetailMuted">Published by {record.submitted_by} · {receivedAt(record.submitted_at)}</p>
      <Disclosure summary="Publication and provenance records"><pre className="rawRecord">{JSON.stringify(record.publication, null, 2)}</pre></Disclosure>
      <Disclosure summary="Deployment records"><pre className="rawRecord">{JSON.stringify(record.deployments, null, 2)}</pre></Disclosure>
    </details>
  </div>;
}
