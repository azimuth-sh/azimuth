# @azimuth-sh/context

The Azimuth context library for Node. It applies the values of an Azimuth launch to a component
and gives the component's report. It opens no port and knows nothing about tests.

## Install

```sh
npm install --save-exact @azimuth-sh/context@<version>
```

The library is released together with the `azimuth` command line tool under one version, so
`<version>` is the version of your `azimuth`.

## Registration

The component registers its name, a condition under which applying is possible, and the mapping
of its keys. The condition is evaluated once, at start:

```js
import { registerAzimuthContext } from "@azimuth-sh/context";

const context = registerAzimuthContext("payouts-web", {
  applyingWhen: () => process.env.NODE_ENV !== "production",
  map: { "payouts.instant": "PAYOUTS_INSTANT" },
});
```

When the condition holds, the library reads `AZIMUTH_CONTEXT` and applies the value of each
mapped key it names. Without the variable it applies nothing.

## Mapping

Every key the component uses is mapped by hand. A text names an environment variable, which the
library reads and writes in `process.env`. A team that keeps a flag in another store maps the key
to a source of its own, an object with `read()` and `apply(text)`.

## Reporting

`context.report()` returns the report as JSON text. For each mapped key it holds the value, the
source it came from and the time it was read. Expose it at the report path the component declares.

## Status

Azimuth is an early alpha. Formats and commands still change between versions.

- Site: https://azimuth.sh
- Source: https://github.com/azimuth-sh/azimuth
