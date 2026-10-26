import "server-only";
import { readDeliveryGap, readRollbackCandidates } from "./delivery";
import {
  readProject,
  type Page,
  type DeploymentRequirementStatuses,
} from "./assurance";

interface Deployment {
  deployment: { id: string; environment: string; verification_target_fingerprint?: string };
  release_fingerprint: string;
}
interface Catalog {
  modules: { id: string; title: string; requirements: string[] }[];
  requirements: { id: string; title?: string; statement: string }[];
}
interface Release {
  publication: {
    release: { label: string; model_revision: string };
    catalog?: Catalog;
  };
}

export type RequirementStatusKind =
  | "supported"
  | "violated"
  | "not-applicable"
  | "unresolved";

export function requirementStatus(status: string): RequirementStatusKind {
  return status === "supported" ||
    status === "violated" ||
    status === "not-applicable"
    ? status
    : "unresolved";
}

interface DeploymentSelection extends Deployment {
  revision: number;
  selected_by: string;
  selected_at: string;
  reason: string;
}

/** What the Requirement list needs of an environment: the deployment, its release, status and
 * catalog. The delivery, observation and rollback panels of the overview are not read, and the
 * deployment list is read only when there is no deployment to show. */
export async function requirementsView(
  project: string,
  selected: string | null,
  environment: string,
) {
  const candidate = selected
    ? await readProject<Deployment>(project, `/deployments/${encodeURIComponent(selected)}`)
    : await readProject<DeploymentSelection>(
        project,
        `/environments/${encodeURIComponent(environment)}/deployment`,
      );
  const deployment =
    candidate?.deployment.environment === environment ? candidate : undefined;
  const verificationTarget = deployment?.deployment.verification_target_fingerprint;
  const [release, assessment, deployments] = await Promise.all([
    deployment
      ? readProject<Release>(
          project,
          `/releases/${encodeURIComponent(deployment.release_fingerprint)}/summary`,
        )
      : Promise.resolve(null),
    deployment && verificationTarget
      ? readProject<DeploymentRequirementStatuses>(
          project,
          `/deployments/${encodeURIComponent(deployment.deployment.id)}/requirements`,
        )
      : Promise.resolve(null),
    deployment
      ? Promise.resolve(null)
      : readProject<Page<Deployment>>(project, "/deployments?limit=100"),
  ]);
  const revision = release?.publication.release.model_revision;
  const catalog = revision
    ? await readProject<{ catalog: Catalog }>(
        project,
        `/model-revisions/${encodeURIComponent(revision)}/catalog`,
      )
    : null;
  const recordedFreshness = assessment?.assessment;
  const freshness = recordedFreshness?.state === "ready" && (typeof recordedFreshness.valid_until_ms !== "number" || recordedFreshness.valid_until_ms <= Date.now())
    ? { ...recordedFreshness, state: "expired" as const } : recordedFreshness;
  return {
    environmentDeployments:
      deployments?.items.filter((d) => d.deployment.environment === environment) ?? [],
    deployment,
    release,
    revision,
    verificationTarget,
    state: freshness?.state === "ready" ? assessment : null,
    freshness,
    hasMore: !!deployments?.next,
    catalog: catalog?.catalog,
  };
}

export async function environmentView(
  project: string,
  selected: string | null,
  environment = "production",
) {
  const [
    deployments,
    revisions,
    currentSelection,
    exact,
    observation,
    deliveryGap,
    rollbackCandidates,
  ] = await Promise.all([
      readProject<Page<Deployment>>(project, "/deployments?limit=100"),
      readProject<Page<{ revision: string; selected: boolean }>>(project, "/model-revisions?limit=100"),
      readProject<DeploymentSelection>(
        project,
        `/environments/${encodeURIComponent(environment)}/deployment`,
      ),
      selected
        ? readProject<Deployment>(
            project,
            `/deployments/${encodeURIComponent(selected)}`,
          )
        : Promise.resolve(null),
      readProject<{
        fresh: boolean;
        deployment_id: string | null;
        received_at: string;
        observation: {
          state: string;
          observed_at_ms: number;
          release_fingerprint: string | null;
        };
      }>(
        project,
        `/environments/${encodeURIComponent(environment)}/observations`,
      ),
      readDeliveryGap(project, environment),
      readRollbackCandidates(project, environment),
    ]);
  const observationFresh =
    !!observation?.fresh &&
    observation.observation.state === "complete" &&
    observation.observation.release_fingerprint ===
      currentSelection?.release_fingerprint &&
    observation.deployment_id === currentSelection?.deployment.id;
  const environmentDeployments =
    deployments?.items.filter(
      (d) => d.deployment.environment === environment,
    ) ?? [];
  const candidate = selected ? exact : currentSelection;
  const deployment =
    candidate?.deployment.environment === environment ? candidate : undefined;
  const verificationTarget = deployment?.deployment.verification_target_fingerprint;
  const [release, assessment] = await Promise.all([
    deployment
      ? readProject<Release>(
          project,
          `/releases/${encodeURIComponent(deployment.release_fingerprint)}/summary`,
        )
      : Promise.resolve(null),
    deployment && verificationTarget
      ? readProject<DeploymentRequirementStatuses>(
          project,
          `/deployments/${encodeURIComponent(deployment.deployment.id)}/requirements`,
        )
      : Promise.resolve(null),
  ]);
  const revision = release?.publication.release.model_revision;
  const catalog = revision
    ? await readProject<{ catalog: Catalog }>(
        project,
        `/model-revisions/${encodeURIComponent(revision)}/catalog`,
      )
    : null;
  const recordedFreshness = assessment?.assessment;
  const freshness = recordedFreshness?.state === "ready" && (typeof recordedFreshness.valid_until_ms !== "number" || recordedFreshness.valid_until_ms <= Date.now())
    ? { ...recordedFreshness, state: "expired" as const } : recordedFreshness;
  const state = freshness?.state === "ready" ? assessment : null;
  return {
    observation,
    observationFresh,
    environmentDeployments,
    deployment,
    release,
    revision,
    verificationTarget,
    state,
    freshness,
    defaultRevision: revisions?.items.find((r) => r.selected)?.revision,
    deliveryGap,
    rollbackCandidates: rollbackCandidates?.candidates ?? [],
    currentSelection,
    catalog: catalog?.catalog,
    isCurrent:
      !!deployment &&
      deployment.deployment.id === currentSelection?.deployment.id,
    hasMore: !!deployments?.next,
  };
}
