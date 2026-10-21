import { selectedEnvironment } from "@/lib/environment";
import Link from "next/link";
import { environmentView } from "@/lib/production";

export const dynamic = "force-dynamic";
const statusLabel = (status: string) =>
  status === "supported"
    ? "Satisfied"
    : status === "violated"
      ? "Not satisfied"
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
    verificationTarget,
    state,
    freshness,
    authority,
    hasMore,
    catalog,
    isCurrent,
    observation,
    observationFresh,
  } = await environmentView(projectId, selected, environment);
  const requirements = state?.requirements ?? [];
  const title = (id: string) =>
    catalog?.requirements.find((c) => c.id === id)?.title ?? id;
  const moduleName = (id: string) =>
    catalog?.modules.find((m) => m.requirements.includes(id))?.id ?? "Unassigned";
  const modelLink = (id?: string) =>
    `${root}/model?environment=${encodeURIComponent(environment)}&revision=${encodeURIComponent(revision ?? "")}&verification-target=${encodeURIComponent(verificationTarget ?? "")}${id ? `&requirement=${encodeURIComponent(id)}` : ""}`;
  const requirementListLink = (status: string) => {
    const query = new URLSearchParams({ status, environment });
    if (deployment) query.set("deployment", deployment.deployment.id);
    return `${root}/requirements?${query}`;
  };
  const counts = {
    supported: requirements.filter((c) => c.status === "supported").length,
    violated: requirements.filter((c) => c.status === "violated").length,
    unresolved: requirements.filter(
      (c) => !["supported", "violated"].includes(c.status),
    ).length,
  };
  const attention = requirements
    .filter((c) => c.status !== "supported")
    .sort(
      (a, b) =>
        Number(b.status === "violated") - Number(a.status === "violated"),
    );
  const modules =
    catalog?.modules.filter((m) => m.requirements.length) ??
    (requirements.length
      ? [
          {
            id: "unassigned",
            title: "Unassigned Requirements",
            requirements: requirements.map((c) => c.requirement),
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
            inspect its Requirement Status. This does not change what is designated current.
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
      {freshness && <section className="dashboardPanel" role="status" aria-label="Assessment freshness">
        <h2>{freshness.state === "ready" ? "Assessment current" : freshness.state === "expired" ? "Assessment overdue" : "Assessment updating"}</h2>
        <p>{freshness.state === "ready" ? "The displayed Requirement Status reflects the current evidence." : "Evidence is being reassessed. Previous conclusions are not shown as current."}
          {freshness.evaluated_at_ms ? ` Last evaluated ${new Date(freshness.evaluated_at_ms).toISOString().replace("T", " ").slice(0, 19)} UTC.` : ""}</p>
      </section>}
      <div className="metricGrid">
        {(
          [
            ["supported", "Satisfied", "Current evidence satisfies the Requirement"],
            ["violated", "Not satisfied", "Evidence contradicts the Requirement"],
            ["unresolved", "Unresolved", "More evidence or review is needed"],
          ] as const
        ).map(([status, label, description]) => (
          <Link
            key={status}
            href={requirementListLink(status)}
            aria-label={`View ${label.toLowerCase()} Requirements`}
            className={`statusMetric ${tone(status)}`}
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
              <h3>Deployment Requirement Status is not available</h3>
              <p>
                {deployment
                  ? "This deployment has no available model assessment. No Requirement is treated as satisfied."
                  : "Select a recorded deployment to see its Requirement Status."}
              </p>
            </div>
          ) : !attention.length ? (
            <div className="panelEmpty">
              <span className="emptySymbol good">✓</span>
              <h3>No Requirements need attention</h3>
              <p>
                All {requirements.length} assessed Requirements are satisfied for this
                deployment.
              </p>
            </div>
          ) : (
            <div className="attentionList">
              {attention.map((c) => (
                <Link
                  className="attentionRow"
                  key={c.requirement}
                  href={modelLink(c.requirement)}
                >
                  <span className={`attentionIcon ${tone(c.status)}`}>
                    {c.status === "violated" ? "!" : "?"}
                  </span>
                  <div>
                    <div className="rowMeta">
                      <span>{moduleName(c.requirement)}</span>
                      <span className={`badge ${tone(c.status)}`}>
                        {statusLabel(c.status)}
                      </span>
                    </div>
                    <h3>{title(c.requirement)}</h3>
                    <p>{c.explanation.summary}</p>
                    <span className="rowAction">
                      Inspect Requirement & evidence{" "}
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
            Modules organize Requirements. Scenarios describe expected behavior.
            Checks produce Results. ASS assesses the evidence and shows
            Requirement Status for a deployment.
          </p>
          <p>
            A passing Check or an accepted Requirement Review alone does not
            make a Requirement satisfied.
          </p>
          <dl>
            <div>
              <dt>Requirements assessed</dt>
              <dd>{state ? requirements.length : "—"}</dd>
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
            <p>Navigate from responsibilities to the Requirements they contain.</p>
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
                <th>Requirements</th>
                <th>Satisfied</th>
                <th>Not satisfied</th>
                <th>Unresolved</th>
                <th>
                  <span className="srOnly">Details</span>
                </th>
              </tr>
            </thead>
            <tbody>
              {modules.map((m) => {
                const rows = requirements.filter((c) => m.requirements.includes(c.requirement));
                return (
                  <tr key={m.id}>
                    <td>
                      <Link href={modelLink(m.requirements[0])}>
                        <span className="moduleIcon">◇</span>
                        {m.id}
                      </Link>
                    </td>
                    <td>{m.requirements.length}</td>
                    <td className="goodText">
                      {state ? rows.filter((c) => c.status === "supported").length : "—"}
                    </td>
                    <td className="badText">
                      {state ? rows.filter((c) => c.status === "violated").length : "—"}
                    </td>
                    <td className="warnText">
                      {
                        state ? rows.filter(
                          (c) => !["supported", "violated"].includes(c.status),
                        ).length : "—"
                      }
                    </td>
                    <td>
                      <Link
                        aria-label={`Inspect ${m.id}`}
                        href={modelLink(m.requirements[0])}
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
