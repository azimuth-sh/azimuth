#!/usr/bin/env node
/**
 * azimuth-emit-ts --output <path> [--root <dir>] <dir-or-file>...
 *
 * Writes the language-neutral manifest the core reads.
 */

import * as fs from 'node:fs';
import * as path from 'node:path';
import { emit, nextRoutes } from './emitter';
import { prometheusLinkage } from './prometheus';
import { surfaceTargets } from './workspace';

const USAGE = `usage: azimuth-emit-ts --output <path> [--root <dir>] [--workspace <file>] [--prometheus <rules>,<tests>] <dir-or-file>...
  --output    where the manifest is written
  --root      paths in the manifest are made relative to this (default: cwd)
  --workspace enumerate declared Next.js surface contributions from area mounts.
              Membership comes from the build, so an untagged route is still a member.
  --prometheus enumerate alert and rule-test artifacts from promtool-validated files.`;

function main(argv: string[]): number {
  let output: string | undefined;
  let root = process.cwd();
  const roots: string[] = [];
  let workspace: string | undefined;
  const prometheus: { rules: string; tests: string }[] = [];

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--output' || arg === '-o') {
      output = argv[++i];
    } else if (arg === '--root') {
      root = path.resolve(argv[++i] ?? '.');
    } else if (arg === '--workspace') {
      workspace = path.resolve(argv[++i] ?? '');
    } else if (arg === '--prometheus') {
      const value = argv[++i] ?? '';
      const split = value.indexOf(',');
      if (split <= 0 || split === value.length - 1) {
        console.error(`azimuth-emit: --prometheus wants <rules>,<tests>, got \`${value}\`\n${USAGE}`);
        return 2;
      }
      prometheus.push({
        rules: path.resolve(value.slice(0, split)),
        tests: path.resolve(value.slice(split + 1)),
      });
    } else if (arg.startsWith('-')) {
      console.error(`azimuth-emit: unknown option \`${arg}\`\n${USAGE}`);
      return 2;
    } else {
      roots.push(arg);
    }
  }

  if (!output || roots.length === 0) {
    console.error(USAGE);
    return 2;
  }

  for (const target of roots) {
    if (!fs.existsSync(target)) {
      console.error(`azimuth-emit: not found: ${target}`);
      return 2;
    }
  }

  const { manifest, warnings } = emit(roots, root);

  try {
    for (const target of workspace ? surfaceTargets(workspace, root) : []) {
      if (target.enumerator !== 'next-routes') continue;
      const routes = nextRoutes(target.surface, target.root, root, {
        area: target.area,
        mount: target.mount,
      });
      manifest.class_members.push(...routes.members);
      if (routes.enumeration) manifest.enumerations.push(routes.enumeration);
      warnings.push(...routes.warnings);
    }
  } catch (error) {
    console.error(`azimuth-emit: ${(error as Error).message}`);
    return 2;
  }

  try {
    for (const pair of prometheus) {
      const linkage = prometheusLinkage(pair.rules, pair.tests, root);
      manifest.artifacts.push(...linkage.artifacts);
      manifest.realizes.push(...linkage.realizes);
      manifest.covers.push(...linkage.covers);
    }
  } catch (error) {
    console.error(`azimuth-emit: ${(error as Error).message}`);
    return 2;
  }

  for (const warning of warnings) {
    console.error(`warning: ${warning.file}:${warning.line}: ${warning.message}`);
  }

  if (warnings.some((warning) => warning.message.includes('left out of the class') ||
      warning.message.includes('class will be narrower'))) {
    return 2;
  }

  fs.mkdirSync(path.dirname(path.resolve(output)), { recursive: true });
  fs.writeFileSync(output, `${JSON.stringify(manifest, null, 2)}\n`);
  console.error(
    `${manifest.realizes.length} realizes, ${manifest.covers.length} covers, ` +
      `${manifest.mechanism_implementations.length} mechanism implementations, ` +
      `${manifest.mechanism_covers.length} mechanism covers → ${output}`,
  );
  return 0;
}

process.exit(main(process.argv.slice(2)));
