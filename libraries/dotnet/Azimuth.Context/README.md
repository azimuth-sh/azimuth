# Azimuth.Context

The Azimuth context library for .NET. It applies the values of an Azimuth launch to a component
and gives the component's report. It opens no port and knows nothing about tests.

## Install

```
dotnet add package Azimuth.Context --version <version>
```

The library is released together with the `azimuth` command line tool under one version, so
`<version>` is the version of your `azimuth`.

## Registration

The component registers its name, a condition under which applying is possible, and the mapping
of its keys. The condition is evaluated once, at start:

```csharp
var context = builder.Configuration.AddAzimuthContext(
    "payouts-api",
    () => builder.Environment.IsDevelopment(),
    map => map.Key("payouts.instant", "Payouts:Instant"));
```

When the condition holds, the library reads `AZIMUTH_CONTEXT` and applies the value of each
mapped key it names. Without the variable it applies nothing.

## Mapping

Every key the component uses is mapped by hand to its own setting. The shipped source reads and
writes a configuration path, and its value wins over the other configuration sources. A team that
keeps a flag in another store maps the key to a source of its own with `IContextSource`.

## Reporting

`context.Report()` returns the report as JSON text. For each mapped key it holds the value, the
source it came from and the time it was read. Expose it at the report path the component declares.

## Status

Azimuth is an early alpha. Formats and commands still change between versions.

- Site: https://azimuth.sh
- Source: https://github.com/azimuth-sh/azimuth
