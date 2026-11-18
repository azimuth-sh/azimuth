# Azimuth's Playwright adapter

This adapter lists marked Playwright tests, runs the requested Checks and reports
their first attempts to Azimuth. It uses the application's own `@playwright/test`.
This version supports Playwright 1.65.1, verified with the adapter's real-runner suite.
The resolved-config loader is pinned to that version because the public reporter
configuration does not expose the entries of a `webServer` array.

Install it as a development dependency of the application:

```sh
npm install --save-dev --save-exact @azimuth-sh/playwright@<version>
```

`azimuth init` prints that line with the CLI's version filled in. The suite's
adapter command is `npx --no @azimuth-sh/playwright`.

Put a Playwright config in each suite's folder. Its `webServer` must start the
application, with `reuseExistingServer` false. Mark a test or a `describe` group
with a tag such as `@Check:loan-is-refused`; several tags may mark one test.
An annotation is not a marker. A suite with another `marker_key` uses that key
in place of `Check`.
