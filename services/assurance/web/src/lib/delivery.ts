import "server-only";
import { readProject } from "./assurance";

export interface DeliveryGapRequirement {
  id: string;
  difference: "added" | "changed" | "removed";
  realization: 'none' | 'reviewed' | null;
  verification: 'none' | 'reviewed' | null;
  release_status: string | null;
}
export interface DeliveryGap {
  environment: string;
  deployment_id: string;
  release_fingerprint: string;
  release_model_revision: string;
  main_model_revision: string;
  main_commit: string;
  main_sequence: number;
  publication_revision: number;
  sources: Record<string, string>;
  requirements: DeliveryGapRequirement[];
}
export interface DeploymentHistoryEntry {
  deployment_id: string;
  release_fingerprint: string;
  label: string;
  first_observed_ms: number;
  last_observed_ms: number;
  rollout_id: string | null;
  current: boolean;
}
export interface DeploymentHistory {
  project: string;
  environment: string;
  deployments: DeploymentHistoryEntry[];
  next_page_token: string | null;
}
export interface RollbackCandidate {
  deployment_id: string;
  release_fingerprint: string;
  label: string;
  first_observed_ms: number;
  last_observed_ms: number;
  rollout_id: string | null;
  last_status_at_ms: number;
  verification_requirements: number;
}
export interface RollbackCandidates {
  project: string;
  environment: string;
  candidates: RollbackCandidate[];
}

const base = (environment: string) =>
  `/environments/${encodeURIComponent(environment)}`;

export function readDeliveryGap(project: string, environment: string) {
  return readProject<DeliveryGap>(project, `${base(environment)}/delivery-gap`);
}

export function readRollbackCandidates(
  project: string,
  environment: string,
  limit = 5,
) {
  return readProject<RollbackCandidates>(
    project,
    `${base(environment)}/rollback-candidates?limit=${limit}`,
  );
}

export function readDeploymentHistory(
  project: string,
  environment: string,
  limit = 25,
  pageToken = "",
) {
  const query = new URLSearchParams({ limit: String(limit) });
  if (pageToken) query.set("page_token", pageToken);
  return readProject<DeploymentHistory>(
    project,
    `${base(environment)}/deployment-history?${query}`,
  );
}
