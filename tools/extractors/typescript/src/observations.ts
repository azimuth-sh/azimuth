import { createHash } from 'node:crypto';
import { Entry, Manifest, Observation, ObservationBinding } from './emitter';

export interface ObservationExport {
  id: string;
  kind: string;
  tool: string;
  tool_version: string;
  observed_at?: string;
  expires_at?: number;
  bindings: ObservationBinding[];
  payload?: unknown;
}

export interface ObservationImport {
  export: unknown;
  reportPath: string;
  reportSource: string;
  inputs: Array<{ path: string; source: string }>;
}

export function importObservation(input: ObservationImport): Manifest {
  const exported = readExport(input.export);
  const fingerprint = createHash('sha256')
    .update(input.reportSource)
    .update('\0')
    .update(exported.tool_version);
  for (const item of input.inputs) fingerprint.update('\0').update(item.source);

  const observation: Observation = {
    id: exported.id,
    kind: exported.kind,
    tool: exported.tool,
    tool_version: exported.tool_version,
    report: input.reportPath,
    inputs: input.inputs.map((item) => item.path),
    ...(exported.observed_at === undefined ? {} : { observed_at: exported.observed_at }),
    ...(exported.expires_at === undefined ? {} : { expires_at: exported.expires_at }),
    source_fingerprint: fingerprint.digest('hex'),
    bindings: exported.bindings,
    payload: exported.payload ?? null,
  };
  return emptyManifest([observation]);
}

export function subject(site: Entry): string {
  if (site.area && site.address_kind && site.address) {
    return `${site.area}|${site.address_kind}|${site.address}`;
  }
  return `${site.file}#${site.site}|${site.lang}`;
}

export function emptyManifest(observations: Observation[]): Manifest {
  return {
    realizes: [], covers: [], mechanism_implementations: [], mechanism_covers: [],
    class_members: [], enumerations: [], artifacts: [], observations,
  };
}

function readExport(value: unknown): ObservationExport {
  if (!record(value)) throw new Error('observation export must be an object');
  for (const field of ['id', 'kind', 'tool', 'tool_version']) {
    if (typeof value[field] !== 'string' || value[field].length === 0) {
      throw new Error(`observation export needs non-empty ${field}`);
    }
  }
  if (!Array.isArray(value.bindings) || value.bindings.length === 0) {
    throw new Error('observation export needs bindings');
  }
  const bindings = value.bindings.map((item, index) => binding(item, index));
  const hasEvidence = bindings.some((item) => item.role === 'evidence');
  if (hasEvidence && (typeof value.observed_at !== 'string'
      || typeof value.expires_at !== 'number' || !Number.isInteger(value.expires_at))) {
    throw new Error('an evidence observation needs observed_at and integer expires_at');
  }
  return {
    id: value.id as string,
    kind: value.kind as string,
    tool: value.tool as string,
    tool_version: value.tool_version as string,
    ...(typeof value.observed_at === 'string' ? { observed_at: value.observed_at } : {}),
    ...(typeof value.expires_at === 'number' ? { expires_at: value.expires_at } : {}),
    bindings,
    payload: value.payload,
  };
}

function binding(value: unknown, index: number): ObservationBinding {
  if (!record(value)) throw new Error(`binding ${index} must be an object`);
  const role = value.role;
  if (role !== 'evidence' && role !== 'challenge') {
    throw new Error(`binding ${index} has unknown role`);
  }
  for (const field of ['spec', 'scenario', 'assertion', 'outcome']) {
    if (typeof value[field] !== 'string' || value[field].length === 0) {
      throw new Error(`binding ${index} needs non-empty ${field}`);
    }
  }
  if (!Array.isArray(value.subjects)) throw new Error(`binding ${index} needs subjects`);
  const subjects = value.subjects.map((item, subjectIndex) => {
    if (!record(item)
        || !['realization', 'evidence', 'mechanism'].includes(item.relation as string)
        || typeof item.identity !== 'string' || item.identity.length === 0) {
      throw new Error(`binding ${index} subject ${subjectIndex} is invalid`);
    }
    return {
      relation: item.relation as 'realization' | 'evidence' | 'mechanism',
      identity: item.identity,
    };
  });
  if (role === 'evidence') {
    if (!['satisfied', 'violated'].includes(value.outcome as string)
        || typeof value.scope !== 'string'
        || typeof value.quantification !== 'string'
        || typeof value.oracle !== 'string') {
      throw new Error(`evidence binding ${index} needs an outcome and complete form`);
    }
  } else {
    if (!['clean', 'findings', 'inconclusive'].includes(value.outcome as string)
        || subjects.length === 0) {
      throw new Error(`challenge binding ${index} needs an outcome and subjects`);
    }
    if (value.scope !== undefined || value.quantification !== undefined || value.oracle !== undefined) {
      throw new Error(`challenge binding ${index} cannot declare an evidence form`);
    }
  }
  return {
    role,
    spec: value.spec as string,
    scenario: value.scenario as string,
    assertion: value.assertion as string,
    outcome: value.outcome as ObservationBinding['outcome'],
    subjects,
    ...(typeof value.scope === 'string' ? { scope: value.scope } : {}),
    ...(typeof value.quantification === 'string'
      ? { quantification: value.quantification } : {}),
    ...(typeof value.oracle === 'string' ? { oracle: value.oracle } : {}),
  };
}

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
