import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, realpathSync, writeFileSync } from 'node:fs';
import { dirname, isAbsolute, relative, resolve, sep } from 'node:path';
import { parseDocument } from 'yaml';

interface Options {
  root: string;
  chart: string;
  helm: string;
  release: string;
  namespace: string;
  values: string[];
  output: string;
}

interface Resource {
  apiVersion?: unknown;
  kind?: unknown;
  metadata?: { name?: unknown; namespace?: unknown };
}

function within(root: string, path: string): string {
  const canonical = realpathSync(path);
  const locator = relative(root, canonical);
  if (!locator || locator === '..' || locator.startsWith(`..${sep}`) || isAbsolute(locator)) {
    throw new Error(`source must be within root: ${path}`);
  }
  return locator.split(sep).join('/');
}

function identity(value: unknown, label: string, source: string): string {
  if (typeof value !== 'string' || !value || /[\s:]/.test(value)) {
    throw new Error(`${source}: expected a nonempty ${label} without whitespace or colon`);
  }
  return value;
}

export function emit(options: Options): void {
  const root = realpathSync(options.root);
  const chart = realpathSync(options.chart);
  within(root, chart);
  if (!isAbsolute(options.helm)) throw new Error('--helm must be an absolute executable path');
  if (!options.release || !options.namespace) throw new Error('release and namespace must be nonempty');
  const values = options.values.map((path) => ({ path: realpathSync(path), locator: within(root, path) }));
  const helmVersion = execFileSync(options.helm, ['version', '--short'], { encoding: 'utf8' }).trim();
  const args = ['template', options.release, chart, '--namespace', options.namespace];
  for (const value of values) args.push('--values', value.path);
  const rendered = execFileSync(options.helm, args, { encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 });
  const chartName = identity(chart.split(sep).at(-1), 'chart name', chart);
  const links: object[] = [];
  const sites = new Set<string>();

  for (const document of rendered.split(/^---\s*$/m)) {
    if (!document.trim()) continue;
    const sourceMatch = document.match(/^# Source: ([^\r\n]+)$/m);
    const markerLines = document.split(/\r?\n/).filter((line) => line.startsWith('# azimuth:'));
    if (markerLines.length === 0) continue;
    if (markerLines.length !== 1) throw new Error('rendered document: expected one Azimuth marker');
    const marker = markerLines[0].match(/^# azimuth: Realizes\("([a-z0-9](?:[a-z0-9-]*[a-z0-9])?)"\)$/);
    if (!marker) throw new Error(`rendered document: invalid Azimuth marker: ${markerLines[0]}`);
    if (!sourceMatch) throw new Error('rendered document: missing Helm Source comment');
    const prefix = `${chartName}/`;
    if (!sourceMatch[1].startsWith(prefix)) throw new Error(`unexpected Helm Source: ${sourceMatch[1]}`);
    const source = resolve(chart, sourceMatch[1].slice(prefix.length));
    const file = within(root, source);
    const parsed = parseDocument(document, { uniqueKeys: true });
    if (parsed.errors.length) throw new Error(`${file}: ${parsed.errors[0].message}`);
    const resource = parsed.toJS() as Resource | null;
    if (!resource || typeof resource !== 'object') throw new Error(`${file}: expected a Kubernetes resource`);
    const apiVersion = identity(resource.apiVersion, 'apiVersion', file);
    const kind = identity(resource.kind, 'kind', file);
    const name = identity(resource.metadata?.name, 'metadata.name', file);
    const namespace = identity(resource.metadata?.namespace ?? options.namespace, 'namespace', file);
    const site = `helm:${chartName}:${options.release}:${apiVersion}:${kind}:${namespace}:${name}`;
    if (sites.has(site)) throw new Error(`${file}: duplicate Helm resource identity ${site}`);
    sites.add(site);
    const fingerprint = createHash('sha256');
    fingerprint.update(helmVersion);
    fingerprint.update(JSON.stringify([options.release, options.namespace, chartName]));
    fingerprint.update(readFileSync(source));
    const defaultValues = resolve(chart, 'values.yaml');
    fingerprint.update(readFileSync(defaultValues));
    for (const value of values) {
      fingerprint.update(value.locator);
      fingerprint.update(readFileSync(value.path));
    }
    fingerprint.update(JSON.stringify(resource));
    links.push({
      claim: marker[1], site, file, lang: 'helm',
      source_fingerprint: `sha256:${fingerprint.digest('hex')}`,
    });
  }

  mkdirSync(dirname(options.output), { recursive: true });
  writeFileSync(options.output, `${JSON.stringify({ realizes: links }, null, 2)}\n`);
}
