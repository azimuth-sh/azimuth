import * as fs from 'node:fs';
import * as path from 'node:path';
import { createHash } from 'node:crypto';
import { Artifact, Entry } from './emitter';

export interface PrometheusLinkage {
  artifacts: Artifact[];
  realizes: Entry[];
  covers: Entry[];
}

/**
 * Enumerates names only after promtool has validated these files in the repository check. The
 * strict line form avoids treating comments or annotation prose as executable rules.
 */
export function prometheusArtifacts(
  rulesFile: string,
  testsFile: string,
  repoRoot: string,
): Artifact[] {
  return prometheusLinkage(rulesFile, testsFile, repoRoot).artifacts;
}

export function prometheusLinkage(
  rulesFile: string,
  testsFile: string,
  repoRoot: string,
): PrometheusLinkage {
  const rulesSource = fs.readFileSync(rulesFile, 'utf8');
  const testsSource = fs.readFileSync(testsFile, 'utf8');
  const alerts = names(rulesSource, /^\s*-\s+alert:\s+([A-Za-z][A-Za-z0-9_]*)\s*$/gm);
  const tests = names(
    testsSource,
    /^\s*(?:-\s+)?alertname:\s+([A-Za-z][A-Za-z0-9_]*)\s*$/gm,
  );
  if (alerts.length === 0) throw new Error(`${rulesFile} contains no alert rules`);
  if (tests.length === 0) throw new Error(`${testsFile} contains no alert rule tests`);

  return {
    artifacts: [...alerts.map((name) => ({
      id: `prometheus-alert:${name}`,
      kind: 'prometheus-alert',
      file: relative(repoRoot, rulesFile),
    })),
    ...tests.map((name) => ({
      id: `prometheus-rule-test:${name}`,
      kind: 'prometheus-rule-test',
      file: relative(repoRoot, testsFile),
    }))],
    realizes: taggedRules(rulesSource, relative(repoRoot, rulesFile)),
    covers: taggedTests(testsSource, relative(repoRoot, testsFile)),
  };
}

function taggedRules(source: string, file: string): Entry[] {
  const pattern = /^\s*#\s*azimuth-realizes:\s+(\S+)\s+(\S+)\s*\n\s*-\s+alert:\s+([A-Za-z][A-Za-z0-9_]*)\s*$/gm;
  return [...source.matchAll(pattern)].map((match) => entry(match[1], match[2], match[3], file, source));
}

function taggedTests(source: string, file: string): Entry[] {
  const pattern = /^\s*#\s*azimuth-covers:\s+(\S+)\s+(\S+)\s+(unit|component|e2e)\s+(example|universal)\s+(direct|golden|relational|metamorphic|model-based|contract)\s*\n\s*(?:-\s+)?alertname:\s+([A-Za-z][A-Za-z0-9_]*)\s*$/gm;
  return [...source.matchAll(pattern)].map((match) => ({
    ...entry(match[1], match[2], match[6], file, source),
    scope: match[3],
    quantification: match[4],
    oracle: match[5],
  }));
}

function entry(spec: string, scenario: string, site: string, file: string, source: string): Entry {
  return {
    spec, scenario, site, file, lang: 'prometheus',
    source_fingerprint: createHash('sha256').update(source).digest('hex'),
  };
}

function names(source: string, pattern: RegExp): string[] {
  return [...source.matchAll(pattern)]
    .map((match) => match[1])
    .filter((value, index, all) => all.indexOf(value) === index)
    .sort((a, b) => a.localeCompare(b));
}

function relative(root: string, file: string): string {
  return path.relative(root, file).split(path.sep).join('/');
}
