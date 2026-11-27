'use strict';
// The suite's own global teardown. It leaves a trace.
const fs = require('node:fs');
module.exports = async () => {
  fs.appendFileSync(process.env.SUITE_JOURNAL, JSON.stringify({ source: 'suite', event: 'teardown' }) + '\n');
};
