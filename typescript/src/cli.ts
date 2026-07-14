#!/usr/bin/env node
//! `azimuth-emit` — scan TypeScript sources for `realizes`/`covers` marker calls and write a
//! language-neutral manifest for `rtm` to ingest.
//!
//!   azimuth-emit [--root <dir>] [--out <file>] [--traced-root <glob>]... <glob>...
//!
//! `--root` (default: cwd) is the codebase root that `file` paths are made relative to. `--out`
//! (default: stdout) is where the manifest JSON is written. `--traced-root` (repeatable) is a path
//! glob for an opt-in traced area: every test under it must carry `covers(...)` or `untraced(...)`
//! or it is emitted as untraced. Globs (default: **/*.ts, **/*.tsx) are resolved under `--root`.
//! Unresolvable marker calls are reported to stderr; they do not fail the run.

import * as fs from 'node:fs';
import * as path from 'node:path';
import { emit } from './emitter';

interface Args {
  root: string;
  out?: string;
  include: string[];
  tracedRoots: string[];
}

function parseArgs(argv: string[]): Args {
  const args: Args = { root: process.cwd(), include: [], tracedRoots: [] };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === undefined) {
      continue;
    }
    if (arg === '--root') {
      args.root = requireValue(argv, ++i, '--root');
    } else if (arg === '--out' || arg === '-o') {
      args.out = requireValue(argv, ++i, arg);
    } else if (arg === '--traced-root') {
      args.tracedRoots.push(requireValue(argv, ++i, '--traced-root'));
    } else if (arg === '--help' || arg === '-h') {
      printUsage();
      process.exit(0);
    } else if (arg.startsWith('-')) {
      fail(`unknown flag: ${arg}`);
    } else {
      args.include.push(arg);
    }
  }
  return args;
}

function requireValue(argv: string[], index: number, flag: string): string {
  const value = argv[index];
  if (value === undefined) {
    fail(`${flag} requires a value`);
  }
  return value;
}

function printUsage(): void {
  process.stdout.write(
    'usage: azimuth-emit [--root <dir>] [--out <file>] [--traced-root <glob>]... <glob>...\n',
  );
}

function fail(message: string): never {
  process.stderr.write(`azimuth-emit: ${message}\n`);
  process.exit(2);
}

function main(): void {
  const args = parseArgs(process.argv.slice(2));
  const { manifest, warnings } = emit({
    root: args.root,
    include: args.include,
    tracedRoots: args.tracedRoots,
  });

  for (const warning of warnings) {
    process.stderr.write(`azimuth-emit: ${warning.file}:${warning.line}: ${warning.message}\n`);
  }

  const json = `${JSON.stringify(manifest, null, 2)}\n`;
  if (args.out === undefined) {
    process.stdout.write(json);
  } else {
    const out = path.resolve(args.out);
    fs.mkdirSync(path.dirname(out), { recursive: true });
    fs.writeFileSync(out, json);
    process.stderr.write(
      `azimuth-emit: wrote ${manifest.realizes.length} realizes + ${manifest.covers.length} covers + ${manifest.untraced_tests.length} untraced to ${args.out}\n`,
    );
  }
}

main();
