export type LifecycleStage = 'merge' | 'release' | 'canary' | 'rollout' | 'operation';
export type GateStatus = 'open' | 'closed';

export interface Project {
  id: string;
  name: string;
  createdAt: number;
}
export interface ExecutionSubject {
  projectSnapshot: string;
  revision: string;
  artifactDigest: string | null;
  deploymentId: string | null;
  environment: string | null;
  cohort: string | null;
}

export interface GateDecisionRecord {
  sequence: number;
  id: string;
  request: {
    definitionId: string;
    stage: LifecycleStage;
    subject: ExecutionSubject;
    at: number;
  };
  decision: {
    status: GateStatus;
    definitionFingerprint: string | null;
    qualificationId: string | null;
    observationId: string | null;
    reasons: string[];
    work: string[];
  };
  evaluatedAt: number;
}

export interface WorkItem {
  decisionId: string;
  definitionId: string;
  stage: LifecycleStage;
  subject: ExecutionSubject;
  reasons: string[];
  work: string[];
  evaluatedAt: number;
}

export interface ClaimReference {
  spec: string;
  claim: string;
  contractFingerprint: string;
}

export interface ProjectModelSnapshot {
  id: string;
  project: string;
  modelFingerprint: string;
  claims: Array<{
    contractFingerprint: string;
    spec: string;
    claim: string;
    criticality: string;
    surface: { id: string } | null;
    obligatedAreas: Array<{ id: string }>;
  }>;
}

export interface ProjectSnapshot {
  project: Project;
  account: {
    projectSnapshots: ProjectModelSnapshot[];
    definitions: Array<{
      id: string;
      claim: ClaimReference;
      assertion: string;
      stage: LifecycleStage;
      declaredAt: number;
    }>;
    qualifications: Array<{ id: string }>;
    observations: Array<{ id: string }>;
    challenges: Array<{ id: string }>;
  };
  gateDecisions: GateDecisionRecord[];
}

const apiUrl = process.env.ASSURANCE_API_URL ?? 'http://127.0.0.1:8080';

export async function getProjects(): Promise<Project[]> {
  return get<Project[]>('/v1/projects');
}

export async function getProjectView(projectId: string): Promise<{
  snapshot: ProjectSnapshot;
  gates: GateDecisionRecord[];
  workItems: WorkItem[];
}> {
  const root = `/v1/projects/${encodeURIComponent(projectId)}`;
  const [snapshot, gates, workItems] = await Promise.all([
    get<ProjectSnapshot>(`${root}/snapshot`),
    get<GateDecisionRecord[]>(`${root}/gates`),
    get<WorkItem[]>(`${root}/work-items`),
  ]);
  return { snapshot, gates, workItems };
}

async function get<T>(path: string): Promise<T> {
  const response = await fetch(`${apiUrl}${path}`, { cache: 'no-store' });
  if (!response.ok) {
    const problem = (await response.json().catch(() => null)) as { detail?: string } | null;
    throw new Error(problem?.detail ?? `Assurance API returned ${response.status}`);
  }
  return (await response.json()) as T;
}
