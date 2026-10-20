import "server-only";
import { readProject, type Page, type SubjectState } from "./assurance";

interface Deployment {
  deployment: { id: string; environment: string; subject_fingerprint?: string };
  release_fingerprint: string;
}
interface Catalog {
  modules: { id: string; title: string; claims: string[] }[];
  claims: { id: string; title?: string; statement: string }[];
}
interface Release {
  publication: {
    release: { label: string; model_revision: string };
    catalog?: Catalog;
  };
  acceptance: unknown;
}

export function claimStatus(status: string) {
  return status === "supported" || status === "violated"
    ? status
    : "unresolved";
}

interface DeploymentSelection extends Deployment {
  revision: number;
  selected_by: string;
  selected_at: string;
  reason: string;
}

export async function environmentView(project: string, selected: string | null, environment = "production") {
  const [deployments, authority, currentSelection, exact] = await Promise.all([
    readProject<Page<Deployment>>(project, "/deployments?limit=100"),
    readProject<{ fingerprint: string }>(project, "/authority"),
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
  ]);
  const observation = await readProject<{
    fresh: boolean;
    deployment_id: string | null;
    received_at: string;
    observation: {
      state: string;
      observed_at_ms: number;
      release_fingerprint: string | null;
    };
  }>(project, `/environments/${encodeURIComponent(environment)}/observations`);
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
  const release = deployment
    ? await readProject<Release>(
        project,
        `/releases/${encodeURIComponent(deployment.release_fingerprint)}/summary`,
      )
    : null;
  const revision = release?.publication.release.model_revision;
  const subject = deployment?.deployment.subject_fingerprint;
  const state =
    revision && subject
      ? await readProject<SubjectState>(
          project,
          `/deployments/${encodeURIComponent(deployment!.deployment.id)}/status`,
        )
      : null;
  const catalog = revision
    ? await readProject<{ catalog: Catalog }>(
        project,
        `/model-revisions/${encodeURIComponent(revision)}/catalog`,
      )
    : null;
  return {
    observation,
    observationFresh,
    environmentDeployments,
    deployment,
    release,
    revision,
    subject,
    state,
    authority,
    currentSelection,
    catalog: catalog?.catalog,
    isCurrent:
      !!deployment &&
      deployment.deployment.id === currentSelection?.deployment.id,
    hasMore: !!deployments?.next,
  };
}
