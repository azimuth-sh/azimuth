'use strict';
// Preloaded into the runner only. The runner offers no option to add a global setup, so the
// config's exports are changed as the runner loads them, and the suite's own entries stay between
// the two reads.
const fs = require('node:fs');
const path = require('node:path');
const Module = require('node:module');
const { PLAN } = require('./components');

// The runner starts its workers with the same options, and a worker has no plan to wrap anything
// with: the variable is gone from its environment.
if (process.env.AZIMUTH_PLAYWRIGHT_PLAN) {
  const plan = JSON.parse(fs.readFileSync(process.env.AZIMUTH_PLAYWRIGHT_PLAN, 'utf8'));
  // The suite's servers and tests inherit the environment, and none of this is theirs.
  delete process.env.AZIMUTH_PLAYWRIGHT_PLAN;
  globalThis[PLAN] = plan;
  wrap(plan);
}

function wrap(plan) {
  const wrapped = new WeakSet();
  const entries = value => (value === undefined || value === null ? [] : Array.isArray(value) ? value : [value]);
  const load = Module._load;
  Module._load = function (request, parent, isMain) {
    const exported = load.apply(this, arguments);
    if (typeof request !== 'string' || path.basename(request) !== path.basename(plan.config)) return exported;
    if (!exported || typeof exported !== 'object' || wrapped.has(exported)) return exported;
    if (Module._resolveFilename(request, parent, isMain) !== plan.config) return exported;
    const config = exported.default && typeof exported.default === 'object' ? exported.default : exported;
    config.globalSetup = [path.join(__dirname, 'start-read.js'), ...entries(config.globalSetup)];
    config.globalTeardown = [...entries(config.globalTeardown), path.join(__dirname, 'stop-read.js')];
    wrapped.add(exported);
    fs.writeFileSync(path.join(plan.folder, 'wrapped'), '');
    return exported;
  };
}
