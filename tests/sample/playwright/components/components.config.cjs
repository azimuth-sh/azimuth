'use strict';
// The configuration of a suite whose components run in their own processes. COMPONENT_SERVERS is a
// JSON list of {name, port, url, host, report, mode}: an entry gives `url` when the description has
// one and only `port` otherwise, and no `name` when the description has none. After the first
// evaluation of this file COMPONENT_SERVERS_LATER, when set, replaces the list.
const fs = require('node:fs');

const evaluations = () => {
  if (!process.env.EVALUATION_FILE) return 1;
  fs.appendFileSync(process.env.EVALUATION_FILE, 'evaluated\n');
  return fs.readFileSync(process.env.EVALUATION_FILE, 'utf8').split('\n').length - 1;
};

fs.appendFileSync(process.env.SUITE_JOURNAL, JSON.stringify({ source: 'suite', event: 'config', pid: process.pid, argv1: process.argv[1] }) + '\n');

const described = JSON.parse(evaluations() > 1 && process.env.COMPONENT_SERVERS_LATER ? process.env.COMPONENT_SERVERS_LATER : process.env.COMPONENT_SERVERS);

module.exports = {
  testMatch: '*.spec.cjs',
  workers: 1,
  retries: 0,
  timeout: 60000,
  globalSetup: './setup.cjs',
  globalTeardown: './teardown.cjs',
  webServer: described.map(component => ({
    ...(component.name ? { name: component.name } : {}),
    command: 'node component-server.cjs',
    ...(component.url ? { url: component.url } : component.port ? { port: component.port } : { wait: { stdout: /COMPONENT_READY (?<free_port>\d+)/ } }),
    env: {
      SERVER_PORT: String(component.port || 0),
      SERVER_HOST: component.host || '127.0.0.1',
      REPORT_NAME: component.name || 'unnamed',
      REPORT_PATH: component.report,
      REPORT_MODE: component.mode || 'ok',
      SUITE_JOURNAL: process.env.SUITE_JOURNAL,
    },
    timeout: 20000,
    reuseExistingServer: false,
    gracefulShutdown: { signal: 'SIGTERM', timeout: 5000 },
  })),
};
