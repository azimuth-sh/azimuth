# Azimuth.Xunit

The Azimuth test adapter for xUnit. It builds a suite, lists its marked tests, runs them and
reports what happened.

The `azimuth` command line tool starts the adapter when it runs a suite. A person normally does
not start it.

## Install

In the repository, install it as a local tool:

```
dotnet tool install --local --create-manifest-if-needed Azimuth.Xunit --version <version>
```

The adapter and the `azimuth` command line tool are released together under one version, so
`<version>` is the version of your `azimuth`. `azimuth init` prints this line with the version
filled in.

## Mark a test

A test stands for a Check when it carries the trait `[Trait("Check", "<id>")]`, with the id of
the Check:

```csharp
[Fact]
[Trait("Check", "sixth-loan-is-refused")]
public void Refuses_a_sixth_loan() { ... }
```

## Status

Azimuth is an early alpha. Formats and commands still change between versions, so a repository
pins one exact version.

- Site: https://azimuth.sh
- Source: https://github.com/azimuth-sh/azimuth
