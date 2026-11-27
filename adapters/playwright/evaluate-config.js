'use strict';
// Evaluates the resolved config in a process of its own and writes its `webServer` entries to the
// file named by the first argument. The second argument is the config path, empty when the runner
// is to find it.
const fs = require('node:fs');
const path = require('node:path');
const { createRequire } = require('node:module');

(async () => {
  const [output, configPath] = process.argv.slice(2);
  const application = createRequire(path.join(process.cwd(), 'package.json'));
  const runner = createRequire(application.resolve('@playwright/test/package.json'));
  // The public FullConfig loses the individual entries when webServer is an array.
  const { configLoader } = runner('playwright/lib/common');
  const config = await configLoader.loadConfigFromFile(configPath || undefined, { retries: 0 });
  fs.writeFileSync(output, JSON.stringify({
    configFile: config.config.configFile,
    webServers: config.webServers.map(({ name, url, port, reuseExistingServer }) => ({ name, url, port, reuseExistingServer })),
  }));
})().catch(error => {
  console.error(error);
  process.exit(1);
});
