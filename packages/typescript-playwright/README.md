# Azimuth Playwright integration

Runs under ordinary `playwright test`; no browser is required for request-only tests.
The fixture reads effective settings from the actual application before and after
**each attempt**, after project overrides. The reporter retains unsuccessful
attempts, retries and skipped tests in the shared `azimuth-test-report` v1 format.
It does not infer Claims, build native Runs, hold ASS credentials or publish.

```ts
// fixtures.ts
export { test, expect } from '@azimuth-sh/playwright';
```

```ts
// playwright.config.ts
import { defineConfig } from '@playwright/test';
import type { AzimuthOptions } from '@azimuth-sh/playwright';

export default defineConfig<AzimuthOptions>({
  reporter: [['@azimuth-sh/playwright/reporter', {
    outputFile: 'test-results/azimuth-report.json',
    execution: process.env.CI_EXECUTION_ID,
  }]],
  use: {
    azimuthContexts: [{
      name: 'authorization',
      url: 'http://127.0.0.1:8080/internal/verification-context',
      instance: 'application-under-test',
      // Optional exact expected contract identity:
      // contractRevision: 'sha256:<64 lowercase hexadecimal characters>',
      timeoutMs: 5000,
    }],
  },
});
```

Tests import the extended `test`, not the unextended Playwright base. Existing
fixtures can extend it with `test.extend`. Projects explicitly map native
`testInfo.testId` identities to Check/Case units, or set `azimuthTestId` through
`test.use` for an explicitly owned identity. Do not guess from display names.
Per-test context overrides remain per-attempt captures, not a suite-wide snapshot.
Use `azimuthContexts: []` only when the test explicitly has no configuration
context dependency. A context endpoint is configured for one actual test instance;
load-balanced or mutable instance routing is unsuitable for this binding.

The endpoint must return the general `azimuth-verification-context` v1 envelope.
Reports retain only the named context, reject unknown envelope fields, validate
contract fingerprints and primitive values, and never record response error bodies
or credentials. Contract field/type completeness is checked by the shared result
normalizer against project contracts. Only allowlisted, nonsecret settings belong
in an application context endpoint. Keep it private.

A missing/unreachable context yields diagnostics and no fabricated successful
capture. Changed contexts retain both snapshots and a diagnostic. Tests continue
so their actual outcomes survive; the downstream adapter must reject incomplete
or inconsistent context evidence. Capture adds a bounded fixture lifecycle budget
(default 15 seconds). Increase fixture timeout explicitly for many endpoints.
Startup-fixed configuration is supported; equal endpoint snapshots cannot prove
absence of an intervening reload, so reloadable configuration requires a stronger
application epoch contract before adoption.

The reporter sorts by identity and attempt, aggregates across workers in the
Playwright coordinator, and writes atomically. For separate shards use a distinct
output file/execution identity per shard; combine through the result adapter rather
than concurrent writes to one destination. Keep raw Playwright reports as separate
artifacts. Native failures remain failures even with an expected-failure annotation.

## Use before registry publication

Check out an exact trusted Azimuth Git commit in a separate CI directory, then:

```sh
cd azimuth/packages/typescript-playwright
npm ci
npm pack --pack-destination /tmp/azimuth-packages
```

Install that generated tarball in the consumer workspace using
`npm install --no-save --package-lock=false /tmp/azimuth-packages/azimuth-sh-playwright-0.1.0-alpha.1.tgz`.
Record the source commit and tarball digest with CI inputs. Avoid installing a
moving branch. The package declares Playwright as a peer so the consumer uses its
existing runner. Test execution stays unprivileged; a separate protected publisher
uploads validated native Runs. No registry publication or ASS access is needed to
execute tests.

## Verification

`npm test` compiles and launches a real multiworker Playwright run against a local
synthetic HTTP server. Intentional failing attempts are captured and asserted,
including retries, skipped tests, missing contexts and changed settings.
