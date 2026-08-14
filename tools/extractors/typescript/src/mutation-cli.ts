#!/usr/bin/env node
import * as fs from 'node:fs';
import * as path from 'node:path';
import { Manifest } from './emitter';
import { importMutationResults } from './mutation-results';

const positional: string[] = [];
let root = '.';
let config = '';
let toolVersion = '';
for (let index = 2; index < process.argv.length; index++) {
  const value = process.argv[index];
  if (value === '--root') root = required(++index, '--root');
  else if (value === '--config') config = required(++index, '--config');
  else if (value === '--tool-version') toolVersion = required(++index, '--tool-version');
  else if (value.startsWith('-')) fail(`unknown option ${value}`);
  else positional.push(value);
}
if (positional.length !== 3 || !config || !toolVersion) {
  fail('usage: azimuth-import-mutation <stryker-report.json> <linkage-manifest.json> <output.json> --root <repo> --config <stryker-config.json> --tool-version <version>');
}

try {
  const [reportPath, linkagePath, outputPath] = positional;
  const reportSource = fs.readFileSync(reportPath, 'utf8');
  const configSource = fs.readFileSync(config, 'utf8');
  const linkage = JSON.parse(fs.readFileSync(linkagePath, 'utf8')) as Manifest;
  const manifest = importMutationResults({
    report: JSON.parse(reportSource) as unknown,
    reportPath: path.relative(root, reportPath).split(path.sep).join('/'),
    reportSource,
    configPath: path.relative(root, config).split(path.sep).join('/'),
    configSource,
    linkage,
    root,
    toolVersion,
  });
  fs.mkdirSync(path.dirname(path.resolve(outputPath)), { recursive: true });
  fs.writeFileSync(outputPath, `${JSON.stringify(manifest, null, 2)}\n`);
  const bindings = manifest.observations?.flatMap((observation) => observation.bindings) ?? [];
  console.error(`${bindings.length} mutation challenge binding(s) → ${outputPath}`);
} catch (error) {
  fail((error as Error).message);
}

function required(index: number, option: string): string {
  const value = process.argv[index];
  if (!value) fail(`${option} needs a value`);
  return value;
}

function fail(message: string): never {
  console.error(`azimuth-import-mutation: ${message}`);
  process.exit(2);
}
