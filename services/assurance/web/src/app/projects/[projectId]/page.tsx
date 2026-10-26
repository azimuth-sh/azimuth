import { selectedEnvironment } from "@/lib/environment";
import Link from "next/link";
import { environmentView } from "@/lib/production";
import { Disclosure } from "@/components/disclosure";
import { Identifier } from "@/components/identifier";
import { LocalTime } from "@/components/local-time";
import { shortIdentifier } from "@/lib/identifier";
import { CodeDrift, declares, specifiedOnly } from "@/components/review-dimensions";

export const dynamic = "force-dynamic";
const statusLabel = (status: string) =>
  status === "supported"
    ? "Satisfied"
    : status === "violated"
      ? "Not satisfied"
      : status === "not-applicable"
        ? "Not applicable here"
        : "Unresolved";
const tone = (status: string) =>
  status === "supported"
    ? "good"
    : status === "violated"
      ? "bad"
      : status === "not-applicable"
        ? "neutral"
        : "warn";
const STALE_STATUS_MS = 30 * 24 * 60 * 60 * 1000;
const differenceLabel = { added: "Added", changed: "Changed", removed: "Removed" } as const;
const declaredReviews = (r: { realization: string | null; verification: string | null }) =>
  [r.verification === "reviewed" && "Verification", r.realization === "reviewed" && "Realization"]
    .filter(Boolean)
    .join(" · ") || "Specified only";

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
    defaultRevision,
    deliveryGap,
    rollbackCandidates,
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
  // Status tiles count only Requirements that declare `Verification:`: a Requirement without it owns
  // no Checks, so its Status is unresolved by construction and says nothing about the deployment.
  const verified = requirements.filter((c) => declares(c.verification_review));
  const counts = {
    supported: verified.filter((c) => c.status === "supported").length,
    violated: verified.filter((c) => c.status === "violated").length,
    unresolved: verified.filter(
      (c) => !["supported", "violated", "not-applicable"].includes(c.status),
    ).length,
  };
  const realizationDeclared = requirements.filter((c) => declares(c.realization_review));
  const realizationAccepted = realizationDeclared.filter((c) => c.realization_review === "accepted");
  const drifted = realizationAccepted.filter((c) => c.realization_code_drift === true);
  const specified = requirements.filter(specifiedOnly);
  const notApplicable = requirements.filter((c) => c.status === "not-applicable");
  const attention = verified
    .filter((c) => c.status !== "supported" && c.status !== "not-applicable")
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
              ? isCurrent && observationFresh ? "Freshly observed" : "Recorded deployment · current state unconfirmed"
              : "No current deployment selected"}
            {deployment && <> · <Identifier value={deployment.deployment.id} /></>}
          </small>
        </div>
        <div>
          <span className="fieldLabel">MODEL</span>
          {revision ? (
            <Link href={modelLink()}>Release model ↗</Link>
          ) : (
            <span>Not available</span>
          )}
          <small>
            {deliveryGap
              ? deliveryGap.requirements.length
                ? `Main has ${deliveryGap.requirements.length} Requirement${deliveryGap.requirements.length === 1 ? "" : "s"} not yet deployed`
                : "Matches main"
              : revision && defaultRevision !== revision
                ? "Differs from the default model"
                : "Default model"}
          </small>
        </div>
        <Link className="outlineButton" href={`${root}/deployments?environment=${encodeURIComponent(environment)}`}>
          Deployment history ↗
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
              {shortIdentifier(d.deployment.id)} →
            </Link>
          ))}
          {hasMore && (
            <p>
              The first 100 deployments are shown; the Deployments page lists the
              complete history.
            </p>
          )}
        </section>
      )}
      {freshness && freshness.state !== "ready" && <section className="dashboardPanel" role="status" aria-label="Assessment freshness">
        <h2>{freshness.state === "expired" ? "Assessment overdue" : "Assessment updating"}</h2>
        <p>Evidence is being reassessed. Previous conclusions are not shown as current.
          {freshness.evaluated_at_ms ? <> Last evaluated <LocalTime value={freshness.evaluated_at_ms} />.</> : null}</p>
      </section>}
      <section aria-label="Verification declared">
        <p className="overline">VERIFICATION DECLARED · {state ? verified.length : "—"}</p>
        <div className="metricGrid">
          {(
            [
              ["supported", "Satisfied", "Current evidence satisfies the Requirement"],
              ["violated", "Not satisfied", "Evidence contradicts the Requirement"],
              ["unresolved", "Unresolved", "More evidence or Verification Review is needed"],
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
      </section>
      <div className="reviewMetrics">
        <section className="statusMetric" aria-label="Realization reviewed">
          <span>Realization reviewed</span>
          <strong>
            {state ? realizationAccepted.length : "—"}
            {state && <small> of {realizationDeclared.length} declared</small>}
          </strong>
          <small>An accepted Realization Review is an opinion about a code snapshot; it never makes a Requirement satisfied.</small>
          {state && drifted.length > 0 && (
            <ul className="driftList" aria-label="Code changed since Realization Review">
              {drifted.map((c) => (
                <li key={c.requirement}>
                  <Link href={modelLink(c.requirement)}>{title(c.requirement)}</Link>{" "}
                  <CodeDrift drift={c.realization_code_drift} commit={c.realization_reviewed_commit} />
                </li>
              ))}
            </ul>
          )}
        </section>
        <section className="statusMetric" aria-label="Specified only">
          <span>Specified only</span>
          <strong>{state ? specified.length : "—"}</strong>
          <small>Requirements that declare neither review: intent only, no review or evidence owed.</small>
        </section>
      </div>
      {state && notApplicable.length > 0 && (
        <section className="dashboardPanel" id="not-applicable" aria-label="Not applicable here">
          <div className="panelHeading">
            <div>
              <h2>Not applicable here</h2>
              <p>
                The required context of this deployment does not hold, so these
                Requirements are neither satisfied nor failing. They are not
                counted as passing.
              </p>
            </div>
            <span className="panelCount">{notApplicable.length}</span>
          </div>
          <div className="attentionList">
            {notApplicable.map((c) => (
              <Link className="attentionRow" key={c.requirement} href={modelLink(c.requirement)}>
                <span className="attentionIcon neutral">–</span>
                <div>
                  <div className="rowMeta">
                    <span>{moduleName(c.requirement)}</span>
                    <span className="badge neutral">Not applicable here</span>
                  </div>
                  <h3>{title(c.requirement)}</h3>
                  <p>{c.summary}</p>
                </div>
              </Link>
            ))}
          </div>
        </section>
      )}
      <>
        <section className="dashboardPanel" id="attention">
          <div className="panelHeading">
            <div>
              <h2>Needs attention</h2>
              <p>Requirements that declare verification. Violations first. Uncertainty stays visible.</p>
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
                All {verified.length - verified.filter((c) => c.status === "not-applicable").length} applicable Requirements that declare verification are satisfied for this
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
                    <p>{c.summary}</p>
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
      </>
      <section className="dashboardPanel" id="delivery-gap" aria-label="Delivery gap">
        <div className="panelHeading">
          <div>
            <h2>Delivery gap</h2>
            <p>What main has that the deployed release does not.</p>
          </div>
          <span className="panelCount">
            {deliveryGap ? deliveryGap.requirements.length : "—"}
          </span>
        </div>
        {!deliveryGap ? (
          <p className="tableEmpty">Not available until a deployment is selected and main has a published model.</p>
        ) : !deliveryGap.requirements.length ? (
          <p className="tableEmpty">
            The deployed release contains every Requirement of main.
          </p>
        ) : (
          <>
            <p className="modelDetailMuted">Status shown is the one measured on the deployed release.</p>
            <div className="tableScroll">
              <table>
                <thead>
                  <tr>
                    <th>Requirement</th>
                    <th>Difference</th>
                    <th>Declared reviews</th>
                    <th>Status on deployed release</th>
                  </tr>
                </thead>
                <tbody>
                  {deliveryGap.requirements.map((r) => (
                    <tr key={r.id}>
                      <td>{r.difference === "added" ? title(r.id) : <Link href={modelLink(r.id)}>{title(r.id)}</Link>}</td>
                      <td>{differenceLabel[r.difference]}</td>
                      <td>{declaredReviews(r)}</td>
                      <td>
                        {r.release_status ? (
                          <span className={`badge ${tone(r.release_status)}`}>
                            {statusLabel(r.release_status)}
                          </span>
                        ) : (
                          "Not in the deployed release"
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <Disclosure summary="Comparison details">
              <dl className="factGrid">
                <div><dt>Main commit</dt><dd><Identifier value={deliveryGap.main_commit} /></dd></div>
                <div><dt>Main sequence · publication revision</dt><dd>{deliveryGap.main_sequence} · {deliveryGap.publication_revision}</dd></div>
                <div><dt>Deployment</dt><dd><Identifier value={deliveryGap.deployment_id} /></dd></div>
              </dl>
            </Disclosure>
          </>
        )}
      </section>
      <section className="dashboardPanel" id="rollback-candidates" aria-label="Rollback candidates">
        <div className="panelHeading">
          <div>
            <h2>Rollback candidates</h2>
            <p>Earlier deployments whose Requirements that declare verification were all satisfied at their last stored status.</p>
          </div>
          <span className="panelCount">{rollbackCandidates.length}</span>
        </div>
        {!rollbackCandidates.length ? (
          <p className="tableEmpty">No earlier deployment qualifies as a rollback candidate.</p>
        ) : (
          <div className="tableScroll">
            <table>
              <thead>
                <tr>
                  <th>Release</th>
                  <th>Deployment</th>
                  <th>Last observed</th>
                  <th>Last stored status</th>
                  <th>Requirements declaring verification</th>
                </tr>
              </thead>
              <tbody>
                {rollbackCandidates.map((c) => {
                  const stale = Date.now() - c.last_status_at_ms > STALE_STATUS_MS;
                  return (
                    <tr key={c.deployment_id}>
                      <td>
                        <Link href={`${root}/releases/${encodeURIComponent(c.release_fingerprint)}?environment=${encodeURIComponent(environment)}`}>
                          {c.label}
                        </Link>
                      </td>
                      <td>
                        <Link href={`${root}?environment=${encodeURIComponent(environment)}&deployment=${encodeURIComponent(c.deployment_id)}`}>
                          {shortIdentifier(c.deployment_id)}
                        </Link>
                      </td>
                      <td><LocalTime value={c.last_observed_ms} /></td>
                      <td>
                        <LocalTime value={c.last_status_at_ms} />{" "}
                        {stale && <span className="badge warn">Stale</span>}
                      </td>
                      <td>{c.verification_requirements}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
        <Disclosure summary="About this list">
          <p>Rollback is performed by the deployment system; Azimuth only observes it. The timestamp is the last stored status of that deployment. Status older than 30 days is labeled stale: the evidence may no longer hold, so open the deployment before relying on it.</p>
        </Disclosure>
      </section>
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
                <th>Not applicable</th>
                <th>Specified only</th>
                <th>
                  <span className="srOnly">Details</span>
                </th>
              </tr>
            </thead>
            <tbody>
              {modules.map((m) => {
                const inModule = requirements.filter((c) => m.requirements.includes(c.requirement));
                const rows = inModule.filter((c) => declares(c.verification_review));
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
                          (c) => !["supported", "violated", "not-applicable"].includes(c.status),
                        ).length : "—"
                      }
                    </td>
                    <td>
                      {state ? rows.filter((c) => c.status === "not-applicable").length : "—"}
                    </td>
                    <td>{state ? inModule.filter(specifiedOnly).length : "—"}</td>
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
      <Disclosure summary="How to read this page">
        <p>Modules organize Requirements. Scenarios describe expected behavior. Checks produce Results. The Azimuth Assurance Service assesses the evidence against the deployment&apos;s own release model and shows Requirement Status for it. Status counts cover Requirements that declare verification. A passing Check or an accepted Verification Review alone does not make a Requirement satisfied, and a Realization Review never does: code drift since a Realization Review is shown as information, not as a failure. This view is read-only and assessed on page load.</p>
      </Disclosure>
    </div>
  );
}
