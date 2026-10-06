import 'server-only';
import { readFileSync } from 'node:fs';

export interface WebConfig {
  github: { clientId: string; clientSecret: string };
  users: Record<string, { projects: string[] }>;
  projects: Record<string, { viewerToken: string }>;
}
function map(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
function fields(
  value: unknown,
  names: string[],
): asserts value is Record<string, unknown> {
  if (
    !map(value) ||
    Object.keys(value).length !== names.length ||
    names.some((name) => !Object.hasOwn(value, name))
  ) {
    throw new Error('Invalid Assurance Web configuration fields');
  }
}
function text(value: unknown, minimum = 1): value is string {
  return (
    typeof value === 'string' && value.length >= minimum && !/\s/.test(value)
  );
}
export function webConfig(): WebConfig {
  const location = process.env.ASSURANCE_WEB_CONFIG_FILE;
  const origin = process.env.NEXTAUTH_URL;
  const secret = process.env.NEXTAUTH_SECRET;
  const api = process.env.ASSURANCE_API_URL;
  if (!location || !origin || !secret || !text(secret, 32) || !api)
    throw new Error('Assurance Web configuration is required');
  const url = new URL(origin);
  if (
    url.protocol !== 'https:' ||
    url.username ||
    url.password ||
    url.pathname !== '/' ||
    url.search ||
    url.hash
  ) {
    throw new Error('NEXTAUTH_URL must be a canonical HTTPS origin');
  }
  const apiUrl = new URL(api);
  if (
    !['http:', 'https:'].includes(apiUrl.protocol) ||
    apiUrl.username ||
    apiUrl.password ||
    apiUrl.pathname !== '/' ||
    apiUrl.search ||
    apiUrl.hash
  ) {
    throw new Error('ASSURANCE_API_URL must be an internal API origin');
  }
  const content = readFileSync(location);
  if (content.length > 1024 * 1024)
    throw new Error('Assurance Web configuration exceeds 1 MiB');
  const source = content.toString('utf8');
  const config: unknown = JSON.parse(source);
  rejectDuplicateKeys(source);
  fields(config, ['github', 'users', 'projects']);
  fields(config.github, ['clientId', 'clientSecret']);
  if (!text(config.github.clientId) || !text(config.github.clientSecret))
    throw new Error('GitHub credentials are required');
  if (
    !map(config.projects) ||
    !Object.keys(config.projects).length ||
    !map(config.users) ||
    !Object.keys(config.users).length
  ) {
    throw new Error('Explicit projects and GitHub users are required');
  }
  const tokens = new Set<string>();
  for (const [id, project] of Object.entries(config.projects)) {
    fields(project, ['viewerToken']);
    if (
      !/^[a-z0-9](?:[a-z0-9-]*[a-z0-9])?$/.test(id) ||
      !text(project.viewerToken, 32) ||
      tokens.has(project.viewerToken)
    ) {
      throw new Error('Projects require distinct viewer credentials');
    }
    tokens.add(project.viewerToken);
  }
  for (const [id, user] of Object.entries(config.users)) {
    fields(user, ['projects']);
    if (
      !/^[1-9][0-9]*$/.test(id) ||
      !Array.isArray(user.projects) ||
      !user.projects.length ||
      user.projects.some(
        (p) =>
          typeof p !== 'string' || !Object.hasOwn(config.projects as object, p),
      ) ||
      new Set(user.projects).size !== user.projects.length
    ) {
      throw new Error(
        'Each stable GitHub user ID requires explicit project access',
      );
    }
  }
  return config as unknown as WebConfig;
}

// Ambiguous allowlist entries must not silently replace an earlier project authorization.
function rejectDuplicateKeys(source: string) {
  let position = 0;
  function whitespace() {
    while (/\s/.test(source[position] ?? '') && position < source.length)
      position++;
  }
  function string(): string {
    const start = position++;
    while (position < source.length) {
      const character = source[position++];
      if (character === '\\') position++;
      else if (character === '"')
        return JSON.parse(source.slice(start, position)) as string;
    }
    throw new Error('Invalid JSON string in Web configuration');
  }
  function value(depth: number) {
    if (depth > 32) throw new Error('Web configuration is too deeply nested');
    whitespace();
    if (source[position] === '{') {
      position++;
      whitespace();
      const keys = new Set<string>();
      if (source[position] === '}') {
        position++;
        return;
      }
      while (position < source.length) {
        whitespace();
        const key = string();
        if (keys.has(key))
          throw new Error('Duplicate object key in Web configuration');
        keys.add(key);
        whitespace();
        position++;
        value(depth + 1);
        whitespace();
        if (source[position++] === '}') return;
      }
    } else if (source[position] === '[') {
      position++;
      whitespace();
      if (source[position] === ']') {
        position++;
        return;
      }
      while (position < source.length) {
        value(depth + 1);
        whitespace();
        if (source[position++] === ']') return;
      }
    } else if (source[position] === '"') string();
    else
      while (position < source.length && !/[\s,}\]]/.test(source[position]))
        position++;
  }
  value(0);
}
