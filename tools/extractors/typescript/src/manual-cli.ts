#!/usr/bin/env node
import * as fs from 'node:fs';
import * as path from 'node:path';
import { importManualResults } from './manual-results';

const [input, output] = process.argv.slice(2);
if (!input || !output) {
  console.error('usage: azimuth-import-manual <provider-neutral-export.json> <manifest.json>');
  process.exit(2);
}

try {
  const source = JSON.parse(fs.readFileSync(input, 'utf8')) as unknown;
  const manifest = importManualResults(source);
  fs.mkdirSync(path.dirname(path.resolve(output)), { recursive: true });
  fs.writeFileSync(output, `${JSON.stringify(manifest, null, 2)}\n`);
  console.error(`${manifest.covers.length} manual result(s) → ${output}`);
} catch (error) {
  console.error(`azimuth-import-manual: ${(error as Error).message}`);
  process.exit(2);
}
