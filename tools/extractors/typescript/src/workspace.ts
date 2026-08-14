import * as fs from 'node:fs';
import * as path from 'node:path';

export interface SurfaceTarget {
  surface: string;
  area: string;
  mount: string;
  enumerator: string;
  root: string;
}

interface WorkspaceDocument {
  format?: unknown;
  version?: unknown;
  areas?: unknown;
  surfaces?: unknown;
}

interface AreaDocument {
  id?: unknown;
  mounts?: unknown;
}

interface MountDocument {
  id?: unknown;
  path?: unknown;
}

interface SurfaceDocument {
  id?: unknown;
  contributions?: unknown;
}

interface ContributionDocument {
  area?: unknown;
  mount?: unknown;
  enumerator?: unknown;
}

export function surfaceTargets(workspacePath: string, repositoryRoot: string): SurfaceTarget[] {
  const document = JSON.parse(fs.readFileSync(workspacePath, 'utf8')) as WorkspaceDocument;
  if (document.format !== 'azimuth-workspace' || document.version !== 1) {
    throw new Error(`${workspacePath} is not an azimuth-workspace version 1 document`);
  }
  if (!Array.isArray(document.areas) || !Array.isArray(document.surfaces)) {
    throw new Error(`${workspacePath} must declare area and surface arrays`);
  }

  const mounts = new Map<string, string>();
  for (const rawArea of document.areas) {
    const area = rawArea as AreaDocument;
    if (typeof area.id !== 'string' || !Array.isArray(area.mounts)) {
      throw new Error(`${workspacePath} contains an area without an id or mounts`);
    }
    for (const rawMount of area.mounts) {
      const mount = rawMount as MountDocument;
      if (typeof mount.id !== 'string' || typeof mount.path !== 'string') {
        throw new Error(`${workspacePath} area ${area.id} contains an invalid mount`);
      }
      if (!normalizedRelativePath(mount.path)) {
        throw new Error(
          `${workspacePath} mount ${area.id}:${mount.id} is not a normalized relative path`,
        );
      }
      const key = `${area.id}:${mount.id}`;
      if (mounts.has(key)) {
        throw new Error(`${workspacePath} declares duplicate mount ${key}`);
      }
      mounts.set(key, mount.path);
    }
  }

  const targets: SurfaceTarget[] = [];
  for (const rawSurface of document.surfaces) {
    const surface = rawSurface as SurfaceDocument;
    if (typeof surface.id !== 'string' || !Array.isArray(surface.contributions)) {
      throw new Error(`${workspacePath} contains a surface without an id or contributions`);
    }
    for (const rawContribution of surface.contributions) {
      const contribution = rawContribution as ContributionDocument;
      if (typeof contribution.area !== 'string' || typeof contribution.mount !== 'string' ||
          typeof contribution.enumerator !== 'string') {
        throw new Error(`${workspacePath} surface ${surface.id} has an invalid contribution`);
      }
      const mountPath = mounts.get(`${contribution.area}:${contribution.mount}`);
      if (!mountPath) {
        throw new Error(
          `${workspacePath} surface ${surface.id} names unknown mount ` +
          `${contribution.area}:${contribution.mount}`,
        );
      }
      targets.push({
        surface: surface.id,
        area: contribution.area,
        mount: contribution.mount,
        enumerator: contribution.enumerator,
        root: path.resolve(repositoryRoot, mountPath),
      });
    }
  }
  return targets;
}

function normalizedRelativePath(value: string): boolean {
  return value.length > 0
    && !path.isAbsolute(value)
    && !value.includes('\\')
    && value.split('/').every((segment) => segment.length > 0 && segment !== '.' && segment !== '..');
}
