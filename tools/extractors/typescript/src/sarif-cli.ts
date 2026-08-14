#!/usr/bin/env node
import * as fs from 'node:fs';
import * as path from 'node:path';
import { Manifest } from './emitter';
import { importSarif } from './sarif-results';

const positional: string[] = [];
const inputs: string[] = [];
let root = '.';
for (let index = 2; index < process.argv.length; index++) {
  const value = process.argv[index];
  if (value === '--root') root = required(++index, '--root');
  else if (value === '--input') inputs.push(required(++index, '--input'));
  else if (value.startsWith('-')) fail(`unknown option ${value}`);
  else positional.push(value);
}
if (positional.length !== 3) {
  fail('usage: azimuth-import-sarif <report.sarif> <linkage.json> <output.json> --root <repo> [--input <config>...]');
}

try {
  const [reportPath, linkagePath, outputPath] = positional;
  const reportSource = fs.readFileSync(reportPath, 'utf8');
  const linkage = JSON.parse(fs.readFileSync(linkagePath, 'utf8')) as Manifest;
  const manifest = importSarif({ report: JSON.parse(reportSource) as unknown,
    reportPath: relative(root, reportPath), reportSource, linkage, root,
    inputs: inputs.map((input) => ({ path: relative(root, input), source: fs.readFileSync(input, 'utf8') })) });
  fs.mkdirSync(path.dirname(path.resolve(outputPath)), { recursive: true });
  fs.writeFileSync(outputPath, `${JSON.stringify(manifest, null, 2)}\n`);
  console.error(`${manifest.observations?.[0].bindings.length ?? 0} SARIF challenge binding(s) → ${outputPath}`);
} catch (error) {
  fail((error as Error).message);
}

function relative(root: string, file: string): string {
  return path.relative(root, file).split(path.sep).join('/');
}

function required(index: number, option: string): string {
  const value = process.argv[index];
  if (!value) fail(`${option} needs a value`);
  return value;
}

function fail(message: string): never {
  console.error(`azimuth-import-sarif: ${message}`);
  process.exit(2);
}
