import 'server-only';
import { webConfig } from './config';
import { requireViewer } from './auth';

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
  record: Record<string, unknown>;
  assessment: string;
  diagnostics: string[];
}
export interface ReviewPage {
  records: Review[];
  next: string | null;
}
export interface SubjectState {
  authority_fingerprint: string;
  model_fingerprint: string;
  claims: Array<{
    claim: string;
    status: string;
    judgment: string;
    diagnostics: string[];
    gaps: unknown;
    bindings: unknown;
  }>;
  method_qualifications: unknown;
}
export async function readProject<T>(
  project: string,
  path: string,
): Promise<T | null> {
  await requireViewer(project);
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
  const [authority, runs, subjects, reviews] = await Promise.all([
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
  ]);
  return { authority, runs, subjects, reviews };
}

export function routeFingerprint(value: string): string | null {
  try {
    const decoded = decodeURIComponent(value);
    return /^sha256:[a-f0-9]{64}$/.test(decoded) ? decoded : null;
  } catch {
    return null;
  }
}
