#!/usr/bin/env node

import { emit } from './emitter';

const options: Record<string, string | string[]> = { values: [] };
const args = process.argv.slice(2);
for (let index = 0; index < args.length; index += 2) {
  const key = args[index];
  const value = args[index + 1];
  if (!key.startsWith('--') || !value || value.startsWith('--')) {
    console.error(`invalid option ${key}`);
    process.exit(2);
  }
  if (key === '--values') {
    (options.values as string[]).push(value);
  } else if (['--root', '--chart', '--helm', '--release', '--namespace', '--output'].includes(key)) {
    options[key.slice(2)] = value;
  } else {
    console.error(`unknown option ${key}`);
    process.exit(2);
  }
}

for (const required of ['root', 'chart', 'helm', 'release', 'output']) {
  if (!options[required]) {
    console.error(`missing --${required}`);
    process.exit(2);
  }
}

try {
  emit({
    root: options.root as string,
    chart: options.chart as string,
    helm: options.helm as string,
    release: options.release as string,
    namespace: (options.namespace as string | undefined) ?? 'default',
    values: options.values as string[],
    output: options.output as string,
  });
} catch (error) {
  console.error(`azimuth-emit-helm: ${(error as Error).message}`);
  process.exit(1);
}
