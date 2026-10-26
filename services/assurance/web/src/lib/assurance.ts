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
export interface ReviewIssue {
  id: string;
  proposition: string;
  location: string;
  reason: string;
  correction: string;
  blocking: boolean;
}
export interface ReviewDisposition {
  issue: string;
  outcome: string;
  rationale: string;
}
interface ReviewRecordFields {
  reviewer: string;
  reviewed_at_ms: number;
  rationale: string;
  artifacts: Array<{ id: string; digest: string; locator: string }>;
  requirement: string;
  requirement_fingerprint: string;
  dependencies_fingerprint: string;
  decision: string;
  reassesses: string | null;
  issues?: ReviewIssue[];
  dispositions?: ReviewDisposition[];
  fingerprint: string;
}
export interface VerificationReviewRecord extends ReviewRecordFields {
  kind: 'verification-review';
  challenge_basis: unknown[];
}
export interface RealizationReviewRecord extends ReviewRecordFields {
  kind: 'realization-review';
  sites_fingerprint: string;
  reviewed_commit: string;
}
export type GlobalReviewRecord = VerificationReviewRecord | RealizationReviewRecord;
export type ReviewStatus = 'accepted' | 'missing' | 'stale' | 'rejected' | 'invalid' | 'unresolved' | 'not-declared';
export interface ReviewPage {
  records: Review[];
  next: string | null;
}
export interface RequirementReviewEntry {
  kind: GlobalReviewRecord['kind'];
  requirement: string;
  record: (Partial<Omit<VerificationReviewRecord, 'kind'> & Omit<RealizationReviewRecord, 'kind'>> & { kind?: GlobalReviewRecord['kind'] }) | null;
  assessment: ReviewStatus | 'pending';
  code_drift: boolean | null;
  diagnostics: string[];
}
export interface RequirementReviewState {
  assessment?: AssessmentFreshness;
  authority_fingerprint: string;
  requirements: RequirementReviewEntry[];
}
export interface ReviewDetail {
  status: ReviewStatus;
  fingerprint: string | null;
  reviewer: string | null;
  reviewed_at_ms: number | null;
  rationale: string | null;
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
  verification_review: ReviewDetail;
  realization_review: ReviewDetail & { reviewed_commit: string | null; code_drift: boolean | null };
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
export interface ContextDependency {
  component: string;
  leaves: string[];
}
export interface DeploymentCheckPolicy {
  verification_target: 'source' | 'artifact' | 'deployment';
  components: string[];
  contexts: ContextDependency[];
  contribution: 'support' | 'counterexample' | 'both';
  configuration_sources?: string[];
  units?: string[];
  controls?: { unit: string; contexts: ContextDependency[] }[];
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
export interface DeploymentRequirementStatuses {
  assessment: AssessmentFreshness;
  requirements: Array<{
    requirement: string;
    status: VerificationTargetState['requirements'][number]['status'];
    verification_review: string;
    realization_review: string;
    realization_code_drift: boolean | null;
    realization_reviewed_commit: string | null;
    summary: string;
  }>;
}
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
    verification_review: ReviewStatus;
    realization_review: ReviewStatus;
    realization_code_drift: boolean | null;
    diagnostics: string[];
    gaps: unknown;
    bindings: RequirementBindingStatus[];
    dependencies?: StatusDependency[];
    evaluated_at_ms?: number;
    challenges?: unknown;
  }>;
}
export type ProvenanceBasis = 'harness' | 'declared' | 'unknown';
export interface DescriptorField {
  value: string | null;
  basis: ProvenanceBasis;
}
export interface AgentDescriptor {
  kind: DescriptorField;
  model: DescriptorField;
  effort: DescriptorField;
  agent_type: DescriptorField;
  harness: { name: string | null; version: string | null; basis: ProvenanceBasis };
}
// Token fields keep the provider's own semantics, so input totals are not comparable across providers.
export interface TokenCounts {
  input_tokens: number;
  cache_read_input_tokens: number | null;
  cache_write_input_tokens: number | null;
  output_tokens: number;
  reasoning_output_tokens: number | null;
}
export interface AttemptUsage extends TokenCounts {
  basis: Exclude<ProvenanceBasis, 'unknown'>;
  model_calls: number;
  tool_calls: number;
  overlapping: boolean;
  delegated: TokenCounts & { agents: number; model_calls: number };
}
export interface ProvenanceEntity {
  kind: string;
  repository: string | null;
  ref: string;
  fingerprint: string | null;
  record?: string | null;
  record_fingerprint?: string | null;
}
export interface ProvenanceAttempt {
  id: string;
  instance: string;
  disposition: 'active' | 'completed' | 'superseded' | 'abandoned';
  sequence: number;
  action: string;
  task: string;
  scope: string;
  target: Array<{ name: string; value: string }>;
  actor: string;
  outcome: string;
  report: string | null;
  handoff_captured_at_ms: number | null;
  recorded_at_ms: number | null;
  agent: AgentDescriptor;
  guidance: Array<{ kind: string; id: string; fingerprint: string | null }>;
  trace: string | null;
  usage: AttemptUsage | null;
  used: ProvenanceEntity[];
  generated: ProvenanceEntity[];
}
export interface FailureAttribution {
  id: string;
  requirement: string;
  adverse: { kind: string; ref: string };
  investigator: string;
  agent: AgentDescriptor;
  recorded_at_ms: number;
  confidence: 'low' | 'medium' | 'high';
  attempts: Array<{ attempt: string; role: 'caused' | 'allowed-through' }>;
  revises: string | null;
  rationale: string;
  fingerprint: string;
}
export interface RequirementWorkHistory {
  requirement: string;
  releases: Array<{
    release: string;
    label: string;
    integration_sequence: number;
    model_revision: string;
    relation: 'intent' | 'dependency';
    changes: Array<{ change: string; attempts: ProvenanceAttempt[] }>;
  }>;
  attributions: FailureAttribution[];
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
  publication_revision: number;
  sources: Record<string, string>;
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
