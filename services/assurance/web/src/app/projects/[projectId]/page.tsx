import { selectedEnvironment } from "@/lib/environment";
import Link from "next/link";
import { environmentView } from "@/lib/production";

export const dynamic = "force-dynamic";
const statusLabel = (status: string) =>
  status === "supported"
    ? "Confirmed"
    : status === "violated"
      ? "Violated"
      : "Unresolved";
const tone = (status: string) =>
  status === "supported" ? "good" : status === "violated" ? "bad" : "warn";

export default async function ProjectPage({
  params,
  searchParams,
}: {
  params: Promise<{ projectId: string }>;
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}) {
  const { projectId } = await params;
  const query = await searchParams;
  const environment = await selectedEnvironment(projectId, query.environment);
  const root = `/projects/${encodeURIComponent(projectId)}`;
  const selected =
    typeof query.deployment === "string" ? query.deployment : null;
  const {
    environmentDeployments,
    deployment,
    release,
    revision,
    subject,
    state,
    authority,
    hasMore,
    catalog,
    isCurrent,
    observation,
    observationFresh,
  } = await environmentView(projectId, selected, environment);
  const claims = state?.claims ?? [];
  const title = (id: string) =>
    catalog?.claims.find((c) => c.id === id)?.title ?? id;
  const moduleName = (id: string) =>
    catalog?.modules.find((m) => m.claims.includes(id))?.id ?? "Unassigned";
  const modelLink = (id?: string) =>
    `${root}/model?environment=${encodeURIComponent(environment)}&revision=${encodeURIComponent(revision ?? "")}&subject=${encodeURIComponent(subject ?? "")}${id ? `&claim=${encodeURIComponent(id)}` : ""}`;
  const claimListLink = (status: string) => {
    const query = new URLSearchParams({ status, environment });
    if (deployment) query.set("deployment", deployment.deployment.id);
    return `${root}/claims?${query}`;
  };
  const counts = {
    supported: claims.filter((c) => c.status === "supported").length,
    violated: claims.filter((c) => c.status === "violated").length,
    unresolved: claims.filter(
      (c) => !["supported", "violated"].includes(c.status),
    ).length,
  };
  const attention = claims
    .filter((c) => c.status !== "supported")
    .sort(
      (a, b) =>
        Number(b.status === "violated") - Number(a.status === "violated"),
    );
  const modules =
    catalog?.modules.filter((m) => m.claims.length) ??
    (claims.length
      ? [
          {
            id: "unassigned",
            title: "Unassigned Claims",
            claims: claims.map((c) => c.claim),
          },
        ]
      : []);
  return (
    <div className="dashboard">
      <div className="pageBreadcrumb">
        Workspace <span>/</span> {projectId} <span>/</span> Overview
      </div>
      <header className="dashboardHeading">
        <div>
          <p className="overline">ENVIRONMENT ASSURANCE</p>
          <h1>Overview</h1>
          <p>What is running. What we know. What needs attention.</p>
        </div>
        <span className="environmentBadge">
          <span className={`statusDot ${observationFresh ? "good" : "warn"}`} />{" "}
          {environment}
        </span>
      </header>
      <section className="deploymentBar" aria-label="Environment context">
        <div>
          <span className="fieldLabel">LAST OBSERVED RELEASE</span>
          <strong>
            {release ? (
              <Link
                href={`${root}/releases/${encodeURIComponent(deployment!.release_fingerprint)}?environment=${encodeURIComponent(environment)}`}
              >
                {release.publication.release.label}{" "}
                <span aria-hidden="true">↗</span>
              </Link>
            ) : (
              "No deployment selected"
            )}
          </strong>
          <small>
            {deployment
              ? `${isCurrent && observationFresh ? "Freshly observed deployment" : "Recorded deployment — current state unconfirmed"} · ${deployment.deployment.id}`
              : "No current deployment selected"}
          </small>
        </div>
        <div>
          <span className="fieldLabel">ACCEPTANCE</span>
          <span className={`badge ${release?.acceptance ? "good" : "neutral"}`}>
            {release
              ? release.acceptance
                ? "Accepted"
                : "Pending acceptance"
              : "Not available"}
          </span>
        </div>
        <div>
          <span className="fieldLabel">ACCEPTED MODEL</span>
          <Link className="mono" href={`${root}/model?environment=${encodeURIComponent(environment)}`}>
            {authority ? authority.fingerprint.slice(7, 19) : "Not selected"}
            {authority ? " ↗" : ""}
          </Link>
          <small>
            {revision && authority?.fingerprint !== revision
              ? "Differs from deployed release"
              : "Selected model baseline"}
          </small>
        </div>
        <Link className="outlineButton" href={`${root}/records`}>
          View records ↗
        </Link>
      </section>
      {!selected && !observationFresh && (
        <section className="dashboardPanel">
          <h2>Current environment state is unconfirmed</h2>
          <p>
            {observation
              ? `Last observation: ${observation.observation.state}. ${observation.fresh ? "A complete matching rollout is not established." : "Observation is overdue."}`
              : "No deployment observation has been recorded."}{" "}
            Recorded release evidence below is historical context, not
            confirmation of the current environment.
          </p>
        </section>
      )}
      {!deployment && (environmentDeployments.length > 0 || hasMore || selected) && (
        <section className="dashboardPanel deploymentPicker">
          <h2>Select a environment deployment</h2>
          <p>
            No current deployment is selected. Choose a recorded deployment to
            inspect its Claim Status. This does not change what is designated current.
          </p>
          {environmentDeployments.map((d) => (
            <Link
              key={d.deployment.id}
              href={`${root}?environment=${encodeURIComponent(environment)}&deployment=${encodeURIComponent(d.deployment.id)}`}
            >
              {d.deployment.id} →
            </Link>
          ))}
          {hasMore && (
            <p>
              The first 100 records are shown; browse Records for the complete
              deployment list.
            </p>
          )}
        </section>
      )}
      <div className="metricGrid">
        {(
          [
            ["supported", "Confirmed", "Current evidence supports the Claim"],
            ["violated", "Violated", "Evidence contradicts the Claim"],
            ["unresolved", "Unresolved", "More evidence or review is needed"],
          ] as const
        ).map(([status, label, description]) => (
          <Link
            key={status}
            href={claimListLink(status)}
            aria-label={`View ${label.toLowerCase()} Claims`}
            className={`healthMetric ${tone(status)}`}
          >
            <span>
              {label}
              <span className={`statusDot ${tone(status)}`} />
            </span>
            <strong>{state ? counts[status] : "—"}</strong>
            <small>{description}</small>
          </Link>
        ))}
      </div>
      <div className="dashboardColumns">
        <section className="dashboardPanel" id="attention">
          <div className="panelHeading">
            <div>
              <h2>Needs attention</h2>
              <p>Violations first. Uncertainty stays visible.</p>
            </div>
            <span className="panelCount">{state ? attention.length : "—"}</span>
          </div>
          {!state ? (
            <div className="panelEmpty">
              <span className="emptySymbol">◇</span>
              <h3>Deployment Claim Status is not available</h3>
              <p>
                {deployment
                  ? "This deployment has no available model assessment. No confirmed status is inferred."
                  : "Select a recorded deployment to see its Claim Status."}
              </p>
            </div>
          ) : !attention.length ? (
            <div className="panelEmpty">
              <span className="emptySymbol good">✓</span>
              <h3>No Claims need attention</h3>
              <p>
                All {claims.length} assessed Claims are confirmed for this
                deployment.
              </p>
            </div>
          ) : (
            <div className="attentionList">
              {attention.map((c) => (
                <Link
                  className="attentionRow"
                  key={c.claim}
                  href={modelLink(c.claim)}
                >
                  <span className={`attentionIcon ${tone(c.status)}`}>
                    {c.status === "violated" ? "!" : "?"}
                  </span>
                  <div>
                    <div className="rowMeta">
                      <span>{moduleName(c.claim)}</span>
                      <span className={`badge ${tone(c.status)}`}>
                        {statusLabel(c.status)}
                      </span>
                    </div>
                    <h3>{title(c.claim)}</h3>
                    <p>{c.explanation.summary}</p>
                    <span className="rowAction">
                      Inspect Claim & evidence{" "}
                      <span aria-hidden="true">↗</span>
                    </span>
                  </div>
                </Link>
              ))}
            </div>
          )}
        </section>
        <aside className="dashboardPanel assessmentPanel">
          <span className="overline">ASSESSMENT CONTEXT</span>
          <h2>Evidence, with boundaries.</h2>
          <p>
            Claim Status describes the selected deployment. A passing Check or an
            accepted review alone does not confirm a Claim.
          </p>
          <dl>
            <div>
              <dt>Claims assessed</dt>
              <dd>{state ? claims.length : "—"}</dd>
            </div>
            <div>
              <dt>Model</dt>
              <dd>
                {revision ? (
                  <Link href={modelLink()}>Release model ↗</Link>
                ) : (
                  "Unavailable"
                )}
              </dd>
            </div>
            <div>
              <dt>Release acceptance</dt>
              <dd>
                {release
                  ? release.acceptance
                    ? "Accepted"
                    : "Pending"
                  : "Unavailable"}
              </dd>
            </div>
          </dl>
          <Link
            className="textLink"
            href={revision ? modelLink() : `${root}/model`}
          >
            Explore model →
          </Link>
        </aside>
      </div>
      <section className="dashboardPanel modulePanel" id="modules">
        <div className="panelHeading">
          <div>
            <h2>Status by module</h2>
            <p>Navigate from responsibilities to the Claims they contain.</p>
          </div>
          <Link
            className="textLink"
            href={revision ? modelLink() : `${root}/model`}
          >
            Browse model ↗
          </Link>
        </div>
        <div className="tableScroll">
          <table>
            <thead>
              <tr>
                <th>Module</th>
                <th>Claims</th>
                <th>Confirmed</th>
                <th>Violated</th>
                <th>Unresolved</th>
                <th>
                  <span className="srOnly">Details</span>
                </th>
              </tr>
            </thead>
            <tbody>
              {modules.map((m) => {
                const rows = claims.filter((c) => m.claims.includes(c.claim));
                return (
                  <tr key={m.id}>
                    <td>
                      <Link href={modelLink(m.claims[0])}>
                        <span className="moduleIcon">◇</span>
                        {m.id}
                      </Link>
                    </td>
                    <td>{rows.length}</td>
                    <td className="goodText">
                      {rows.filter((c) => c.status === "supported").length}
                    </td>
                    <td className="badText">
                      {rows.filter((c) => c.status === "violated").length}
                    </td>
                    <td className="warnText">
                      {
                        rows.filter(
                          (c) => !["supported", "violated"].includes(c.status),
                        ).length
                      }
                    </td>
                    <td>
                      <Link
                        aria-label={`Inspect ${m.id}`}
                        href={modelLink(m.claims[0])}
                      >
                        ↗
                      </Link>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
        {!modules.length && (
          <p className="tableEmpty">
            No module assessments available for this selection.
          </p>
        )}
      </section>
      <p className="dashboardFootnote">
        Read-only view · Assessments are retrieved on page load · Acceptance is
        an explicit decision
      </p>
    </div>
  );
}
