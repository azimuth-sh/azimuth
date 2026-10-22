import 'server-only';
import { webConfig } from './config';
import { requireViewer } from './auth';
import { developmentScenario } from './development';
import { developmentRead } from './development-data';

export interface Page<T> {
  items: T[];
  next: string | null;
}
export interface RunSummary {
  run_id: string;
  revision: number;
  fingerprint: string;
  verification_target_fingerprint: string;
  model_fingerprint: string;
  status: string;
  verification_target: unknown;
}
export interface VerificationTargetSummary {
  fingerprint: string;
  verification_target: unknown;
}
export interface Review {
  provenance_basis?: string;
  record: Record<string, unknown>;
  assessment: string;
  diagnostics: string[];
  historical: boolean;
  submitted_by: string;
  submitted_at: string;
}
export interface ReviewPage {
  records: Review[];
  next: string | null;
}
export interface RequirementReviewState {
  assessment?: AssessmentFreshness;
  authority_fingerprint: string;
  requirements: Array<{
    requirement: string;
    record: Record<string, unknown> | null;
    assessment: string;
    diagnostics: string[];
  }>;
}
export interface CheckResultReference {
  check: string;
  scenario: string;
  run: string;
  bundle_fingerprint: string;
  check_result: string;
  outcome: string;
  observed_at_ms: number;
  selected_in_basis: boolean;
  accepted_basis: boolean;
  diagnostics: string[];
  applicable?: boolean;
  verification_target_fingerprint?: string;
}
export interface StatusExplanation {
  summary: string;
  status: string;
  reasons: Array<{ code: string; category: string; message: string; check?: string; scenario?: string; diagnostics?: string[] }>;
  review: { status: string; fingerprint: string | null; reviewer: string | null; reviewed_at_ms: number | null; rationale: string | null };
  requirement_status: { status: string; fingerprint: string | null; reviewer: string | null; reviewed_at_ms: number | null; rationale: string | null };
  coverage: Array<{ check: string; scenario: string; status: string; check_results: CheckResultReference[] }>;
  evidence: { supporting: CheckResultReference[]; violations: CheckResultReference[]; inconclusive: CheckResultReference[]; unassessed: CheckResultReference[] };
  residual_risks: unknown[];
  dependencies?: StatusDependency[];
  adverse?: CheckResultReference[];
}
export interface StatusDependency {
  kind: string;
  fingerprint?: string;
  run?: string;
  bundle_fingerprint?: string;
}
export interface MonitorPolicy {
  id: string;
  interval_seconds: number;
  freshness_seconds: number;
}
export interface DeploymentCheckPolicy {
  verification_target: 'source' | 'artifact' | 'deployment';
  components: string[];
  contexts: string[];
  contribution: 'support' | 'counterexample' | 'both';
  configuration_sources?: string[];
  units?: string[];
  controls?: { unit: string; contexts: string[] }[];
  monitor: MonitorPolicy | null;
}
export interface RequirementBindingStatus {
  check: string;
  scenario: string;
  status?: string;
  monitor?: MonitorPolicy | null;
  reasons?: string[];
}
export interface AssessmentFreshness {
  state: 'ready' | 'pending' | 'expired';
  generation: number;
  published_generation: number | null;
  evaluated_at_ms: number | null;
  valid_until_ms: number | null;
}
export type DeploymentStatusSummary = Omit<VerificationTargetState, 'requirements'> & {
  requirements: Array<Pick<VerificationTargetState['requirements'][number], 'requirement' | 'status' | 'review' | 'diagnostics' | 'gaps'> & { explanation: Pick<StatusExplanation, 'summary'> }>;
};
export interface VerificationTargetState {
  assessment?: AssessmentFreshness;
  authority_fingerprint: string;
  model_fingerprint: string;
  format?: string;
  evaluated_at_ms?: number;
  deployment?: string;
  verification_target_fingerprint?: string;
  requirements: Array<{
    requirement: string;
    status: string;
    explanation: StatusExplanation;
    review?: string;
    requirement_status?: string;
    diagnostics: string[];
    gaps: unknown;
    bindings: RequirementBindingStatus[];
    dependencies?: StatusDependency[];
    evaluated_at_ms?: number;
    challenges?: unknown;
  }>;
}
export interface ModelRevision {
  revision: string;
  selected: boolean;
}
export interface ReleaseIntegration {
  commit: string;
  main_sequence: number;
}
export interface ReleaseModelReference {
  fingerprint?: string;
  label: string;
  model_revision: string;
  integration: ReleaseIntegration | null;
}
export interface ReleaseSummary {
  fingerprint: string;
  release: ReleaseModelReference;
}
export interface ReleaseDetail {
  publication: { release: ReleaseModelReference };
}
export interface DeploymentReference {
  deployment: { id: string; environment: string; verification_target_fingerprint?: string };
  release_fingerprint: string;
}
export interface DefaultAuthority {
  commit: string;
  main_sequence: number;
  ratification_status: string;
  [key: string]: unknown;
}
export async function readProject<T>(
  project: string,
  path: string,
): Promise<T | null> {
  await requireViewer(project);
  if (developmentScenario()) return developmentRead(project, path) as T | null;
  const config = webConfig();
  const token = config.projects[project]?.viewerToken;
  if (!token) throw new Error('Project read credential is unavailable');
  const response = await fetch(
    `${process.env.ASSURANCE_API_URL}/v1/projects/${encodeURIComponent(project)}${path}`,
    {
      headers: { authorization: `Bearer ${token}` },
      cache: 'no-store',
      signal: AbortSignal.timeout(15000),
      redirect: 'error',
    },
  );
  if (response.status === 404) return null;
  if (!response.ok)
    throw new Error(
      `Assurance API unavailable (${response.status}); no conclusion inferred`,
    );
  return (await response.json()) as T;
}
export function cursor(value: string | string[] | undefined): string {
  if (Array.isArray(value) || (value?.length ?? 0) > 1024)
    throw new Error('Invalid pagination cursor');
  return value ?? '';
}
export async function projectView(
  project: string,
  page: { runs?: string; verificationTargets?: string; reviews?: string },
) {
  const [authority, runs, verificationTargets, reviews, requirementReviews] = await Promise.all([
    readProject<Record<string, unknown>>(project, '/authority'),
    readProject<Page<RunSummary>>(
      project,
      `/runs?after=${encodeURIComponent(page.runs ?? '')}`,
    ),
    readProject<Page<VerificationTargetSummary>>(
      project,
      `/verification-targets?after=${encodeURIComponent(page.verificationTargets ?? '')}`,
    ),
    readProject<ReviewPage>(
      project,
      `/reviews?after=${encodeURIComponent(page.reviews ?? '')}`,
    ),
    readProject<RequirementReviewState>(project, '/requirement-reviews'),
  ]);
  return { authority, runs, verificationTargets, reviews, requirementReviews };
}

export function routeFingerprint(value: string): string | null {
  try {
    const decoded = decodeURIComponent(value);
    return /^sha256:[a-f0-9]{64}$/.test(decoded) ? decoded : null;
  } catch {
    return null;
  }
}
