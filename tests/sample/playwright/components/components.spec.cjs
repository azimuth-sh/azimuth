const fs = require('node:fs');
const { test } = require('@playwright/test');

test('pays out at once', { tag: '@Check:payout-is-sent-at-once' }, async () => {
  fs.appendFileSync(process.env.SUITE_JOURNAL, JSON.stringify({ source: 'suite', event: 'test', context: process.env.AZIMUTH_CONTEXT }) + '\n');
  if (process.env.KILL_RUNNER === 'yes') {
    // The runner is this worker's parent.
    process.kill(process.ppid, 'SIGKILL');
    await new Promise(resolve => setTimeout(resolve, 10000));
  }
});
