#!/usr/bin/env node
import * as fs from 'node:fs';
import * as path from 'node:path';
import { Manifest } from './emitter';
import { importPitResults } from './pit-results';

const positional: string[] = [];
let root = '.';
let config = '';
let selection = '';
let toolVersion = '';
for (let index = 2; index < process.argv.length; index++) {
  const value = process.argv[index];
  if (value === '--root') root = required(++index, '--root');
  else if (value === '--config') config = required(++index, '--config');
  else if (value === '--selection') selection = required(++index, '--selection');
  else if (value === '--tool-version') toolVersion = required(++index, '--tool-version');
  else if (value.startsWith('-')) fail(`unknown option ${value}`);
  else positional.push(value);
}
if (positional.length !== 3 || !config || !selection || !toolVersion) {
  fail('usage: azimuth-import-pit <mutations.xml> <linkage.json> <output.json> --root <repo> '
    + '--config <pit-config> --selection <selection.json> --tool-version <version>');
}

try {
  const [reportPath, linkagePath, outputPath] = positional;
  const reportSource = fs.readFileSync(reportPath, 'utf8');
  const configSource = fs.readFileSync(config, 'utf8');
  const selectionSource = fs.readFileSync(selection, 'utf8');
  const linkage = JSON.parse(fs.readFileSync(linkagePath, 'utf8')) as Manifest;
  const manifest = importPitResults({
    reportPath: relative(root, reportPath),
    reportSource,
    configPath: relative(root, config),
    configSource,
    selectionPath: relative(root, selection),
    selectionSource,
    linkage,
    toolVersion,
  });
  fs.mkdirSync(path.dirname(path.resolve(outputPath)), { recursive: true });
  fs.writeFileSync(outputPath, `${JSON.stringify(manifest, null, 2)}\n`);
  const bindings = manifest.observations?.flatMap((observation) => observation.bindings) ?? [];
  console.error(`${bindings.length} PIT challenge binding(s) → ${outputPath}`);
} catch (error) {
  fail((error as Error).message);
}

function relative(root: string, file: string): string {
  return path.relative(path.resolve(root), path.resolve(file)).split(path.sep).join('/');
}

function required(index: number, option: string): string {
  const value = process.argv[index];
  if (!value) fail(`${option} needs a value`);
  return value;
}

function fail(message: string): never {
  console.error(`azimuth-import-pit: ${message}`);
  process.exit(2);
}
