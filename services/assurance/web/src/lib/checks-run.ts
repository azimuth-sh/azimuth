import "server-only";
import { readProject } from "./assurance";

export interface CheckRunUnit {
  id: string;
  input_fingerprint: string | null;
  matched: boolean;
}
export interface RunSubject {
  repository?: string;
  commit?: string;
  artifacts?: { id: string; digest: string }[];
}
export interface ReleaseCheckRun {
  run_fingerprint: string;
  run_id: string;
  subject: RunSubject | null;
  executed_at_ms: number;
  units: CheckRunUnit[];
  outcomes: Record<string, string>;
  skipped_tests: boolean;
}
export type ReleaseCheckState =
  | "executed"
  | "would-apply"
  | "skipped"
  | "inconclusive"
  | "missing";
export interface ReleaseCheck {
  check_id: string;
  scenarios: string[];
  state: ReleaseCheckState;
  executed_for_release: ReleaseCheckRun[];
  would_apply: ReleaseCheckRun[];
}
export interface ReleaseRequirementChecks {
  requirement: string;
  realization: 'none' | 'reviewed' | null;
  verification: 'none' | 'reviewed' | null;
  checks: ReleaseCheck[];
}

export interface DeploymentCheckRun {
  run_fingerprint: string;
  run_id: string;
  executed_for_release: boolean;
  reused: boolean;
  subject: RunSubject | null;
  units: CheckRunUnit[];
  executed_at_ms: number;
}
export type DeploymentCheckOutcome =
  | "supported"
  | "counterexample"
  | "skipped"
  | "inconclusive"
  | "missing";
export interface DeploymentCheck {
  check_id: string;
  scenarios: string[];
  outcome: DeploymentCheckOutcome;
  runs: DeploymentCheckRun[];
}
export interface DeploymentRequirementChecks {
  requirement: string;
  status: string;
  checks: DeploymentCheck[];
}

export async function readReleaseChecks(project: string, release: string) {
  const summary = await readProject<{
    requirements?: ReleaseRequirementChecks[];
  }>(project, `/releases/${encodeURIComponent(release)}/summary?include=checks`);
  return summary?.requirements ?? null;
}

export async function readDeploymentChecks(project: string, deployment: string) {
  const summary = await readProject<{
    requirements: DeploymentRequirementChecks[];
  }>(project, `/deployments/${encodeURIComponent(deployment)}/status/summary`);
  return summary?.requirements ?? null;
}
