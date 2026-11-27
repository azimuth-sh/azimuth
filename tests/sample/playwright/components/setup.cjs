'use strict';
// The suite's own global setup. It leaves a trace and fails when FAIL_SETUP says so.
const fs = require('node:fs');
module.exports = async () => {
  fs.appendFileSync(process.env.SUITE_JOURNAL, JSON.stringify({ source: 'suite', event: 'setup', context: process.env.AZIMUTH_CONTEXT }) + '\n');
  if (process.env.FAIL_SETUP === 'yes') throw new Error('the suite setup failed on purpose');
};
