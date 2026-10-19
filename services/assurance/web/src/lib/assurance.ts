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
  subject_fingerprint: string;
  model_fingerprint: string;
  status: string;
  subject: unknown;
}
export interface SubjectSummary {
  fingerprint: string;
  subject: unknown;
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
export interface ClaimReviewState {
  authority_fingerprint: string;
  claims: Array<{
    claim: string;
    record: Record<string, unknown> | null;
    assessment: string;
    diagnostics: string[];
  }>;
}
export interface HealthObservation {
  check: string;
  case: string;
  run: string;
  bundle_fingerprint: string;
  observation: string;
  outcome: string;
  observed_at_ms: number;
  selected_in_basis: boolean;
  accepted_basis: boolean;
  diagnostics: string[];
}
export interface HealthExplanation {
  summary: string;
  status: string;
  reasons: Array<{ code: string; category: string; message: string; check?: string; case?: string; diagnostics?: string[] }>;
  review: { status: string; fingerprint: string | null; reviewer: string | null; reviewed_at_ms: number | null; rationale: string | null };
  health: { status: string; fingerprint: string | null; reviewer: string | null; reviewed_at_ms: number | null; rationale: string | null };
  coverage: Array<{ check: string; case: string; status: string; observations: HealthObservation[] }>;
  evidence: { supporting: HealthObservation[]; violations: HealthObservation[]; inconclusive: HealthObservation[]; unassessed: HealthObservation[] };
  residual_risks: unknown[];
}
export interface SubjectState {
  authority_fingerprint: string;
  model_fingerprint: string;
  claims: Array<{
    claim: string;
    status: string;
    explanation: HealthExplanation;
    review?: string;
    health?: string;
    diagnostics: string[];
    gaps: unknown;
    bindings: unknown;
    challenges?: unknown;
  }>;
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
  page: { runs?: string; subjects?: string; reviews?: string },
) {
  const [authority, runs, subjects, reviews, claimReviews] = await Promise.all([
    readProject<Record<string, unknown>>(project, '/authority'),
    readProject<Page<RunSummary>>(
      project,
      `/runs?after=${encodeURIComponent(page.runs ?? '')}`,
    ),
    readProject<Page<SubjectSummary>>(
      project,
      `/subjects?after=${encodeURIComponent(page.subjects ?? '')}`,
    ),
    readProject<ReviewPage>(
      project,
      `/reviews?after=${encodeURIComponent(page.reviews ?? '')}`,
    ),
    readProject<ClaimReviewState>(project, '/claim-reviews'),
  ]);
  return { authority, runs, subjects, reviews, claimReviews };
}

export function routeFingerprint(value: string): string | null {
  try {
    const decoded = decodeURIComponent(value);
    return /^sha256:[a-f0-9]{64}$/.test(decoded) ? decoded : null;
  } catch {
    return null;
  }
}
