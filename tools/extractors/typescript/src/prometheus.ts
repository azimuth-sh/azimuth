import * as fs from 'node:fs';
import * as path from 'node:path';
import { createHash } from 'node:crypto';
import { Artifact, CheckImplementation, Entry } from './emitter';

export interface PrometheusLinkage {
  artifacts: Artifact[];
  realizes: Entry[];
  check_implementations: CheckImplementation[];
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
  rejectRetiredMarkers(rulesSource, rulesFile);
  rejectRetiredMarkers(testsSource, testsFile);
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
    check_implementations: taggedChecks(testsSource, relative(repoRoot, testsFile)),
  };
}

function taggedRules(source: string, file: string): Entry[] {
  const pattern = /^\s*#\s*azimuth-realizes:\s+(\S+)\s+(\S+)\s*\n\s*-\s+alert:\s+([A-Za-z][A-Za-z0-9_]*)\s*$/gm;
  return [...source.matchAll(pattern)].map((match) => entry(match[1], match[2], match[3], file, source));
}

function taggedChecks(source: string, file: string): CheckImplementation[] {
  const pattern = /^\s*#\s*azimuth-implements-check:\s+(\S+)\s*\n\s*(?:-\s+)?alertname:\s+([A-Za-z][A-Za-z0-9_]*)\s*$/gm;
  const matches = [...source.matchAll(pattern)];
  const implementations = matches.map((match, index) => {
    const start = match.index ?? 0;
    const end = matches[index + 1]?.index ?? source.length;
    return {
      check: match[1],
      site: match[2],
      file,
      lang: 'prometheus',
      source_fingerprint: fingerprint(source.slice(start, end)),
    };
  });
  return implementations.sort((left, right) =>
    left.check.localeCompare(right.check) || left.site.localeCompare(right.site));
}

function entry(spec: string, scenario: string, site: string, file: string, source: string): Entry {
  return {
    spec, scenario, site, file, lang: 'prometheus',
    source_fingerprint: fingerprint(source),
  };
}

function rejectRetiredMarkers(source: string, file: string): void {
  const match = /^\s*#\s*(azimuth-(?:covers|covers-mechanism))\s*:/m.exec(source);
  if (!match) return;
  const line = source.slice(0, match.index).split('\n').length;
  throw new Error(
    `${file}:${line}: retired alpha 1 marker \`${match[1]}\` is not supported`,
  );
}

function fingerprint(source: string): string {
  return `sha256:${createHash('sha256').update(source).digest('hex')}`;
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
