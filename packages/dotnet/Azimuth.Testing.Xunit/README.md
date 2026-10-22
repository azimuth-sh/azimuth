# Azimuth.Testing.Xunit

Reusable xUnit **v2** fixture integration for startup-fixed effective Execution Contexts.
Targets .NET 8 and supports .NET 10 consumers. Tests still run with `dotnet test`.
No AAS connection, credential, publisher, Check mapping or Requirement assessment is included.

## Consumer fixture

Compose this session into the fixture that owns your application. Start the real application
and apply test configuration overrides before invoking `RunAsync`. Capture delegates read
resolved settings from that exact application instance, preferably its internal context endpoint.

```csharp
using Azimuth.Testing.Xunit;
using Xunit;

public sealed class ApplicationFixture : IAsyncLifetime
{
    public AzimuthTestSession Evidence { get; } = new(
        Path.Combine("TestResults", $"azimuth-{Guid.NewGuid():N}.json"),
        Environment.GetEnvironmentVariable("AZIMUTH_EXECUTION") ?? Guid.NewGuid().ToString("N"));

    public Task InitializeAsync() => Evidence.InitializeAsync();
    public Task DisposeAsync() => Evidence.DisposeAsync();
}

public sealed class PermissionTests(ApplicationFixture fixture) : IClassFixture<ApplicationFixture>
{
    [Fact]
    public Task RejectsAnUnauthorizedCaller() => fixture.Evidence.RunAsync(
        testId: "Example.PermissionTests.RejectsAnUnauthorizedCaller",
        attempt: 0,
        body: async cancellationToken =>
        {
            await Task.CompletedTask;
        },
        contexts: Array.Empty<ContextSource>());
}
```

An empty contexts list explicitly declares no configuration dependency for that test.
For configuration-dependent tests, supply real capture delegates:

```csharp
var source = new ContextSource("caller-authentication", async cancellationToken =>
{
    var json = await applicationClient.GetStringAsync(
        "/internal/execution-context", cancellationToken);
    return new ContextCapture(applicationInstanceId, ContextProtocol.Parse(json));
});
await fixture.Evidence.RunAsync(exactNativeTestId, attempt, testBody, new[] { source });
```

`applicationInstanceId` must identify the actual test application, not just its URL or
configuration profile. Reusing a deployment URL across process replacements is insufficient.
The reporting protocol is `azimuth-execution-context` version 2 with named contexts,
SHA-256 `contract_revision`, and flat typed values. Application endpoint protection and
allowed non-secret fields are application responsibilities. The integration never logs
capture exception messages because these may disclose credentials.

## Execution and failure semantics

- The wrapper records the body outcome, captures both sides of the execution, flushes the
  report atomically, and rethrows body failures so xUnit still fails the test.
- Each test has an explicit native identity and zero-based attempt. Supply distinct identities
  for theory rows; the integration never guesses identities from display names or infers Checks.
- Pass the real retry attempt if a retry extension executes a test again. Duplicate identity /
  attempt pairs fail rather than overwrite evidence. Retries do not erase failed attempts.
- Context reads have a ten-second deadline by default, configurable in the session constructor.
  Delegates must observe cancellation and perform read-only operations.
- Missing/invalid/unreadable contexts preserve test outcomes and add capture diagnostics.
  Context or application-instance changes add `context-changed:<name>` diagnostics. Those
  facts are inconclusive evidence; the downstream normalizer must reject them for support.
- Each selected context is retained independently; unrelated context values are omitted.
  Contracts and fingerprint semantics are validated by the shared adapter/core, not inferred here.
- Before/after comparison supports startup-fixed configuration. It does not establish that a
  reloadable setting remained unchanged between captures; that mode is not supported.
- Tests with different application overrides need their own capture delegates / instances.
- Concurrent tests in a session are supported. Each session/process must own a **unique**
  report path. Reports are sorted by identity and attempt. Completed tests are flushed without
  waiting for suite cleanup. In-flight tests are recorded as interrupted until completion.
- Hard process termination cannot guarantee a final flush. Native runner reports and selection
  reconciliation must detect missing tests. No absent fact means success.
- This fixture does not intercept xUnit discovery. `[Fact(Skip = ...)]` never enters the wrapper;
  native runner report reconciliation must retain such skips. `RecordSkippedAsync` supports
  explicit skips known to the consumer. Do not replace real skipped tests with passing wrappers.
- Dispose the session after all test wrappers finish; dispose the real application afterward.
  The session owns report resources only and does not dispose the consumer application.

## Wire output

The integration writes `azimuth-test-report` version 1, framework `xunit-v2`:
`execution`, millisecond start/end timestamps and `tests`. Every test carries `id`, `attempt`,
`outcome`, timestamps, `contexts` and `diagnostics`; a context entry has `name`, `before` and
`after`, each containing `captured_at_ms`, `instance`, and the context `report` envelope.
Raw TRX reports/logs remain separate artifacts. A result adapter reconciles this report with
selected native test identities and maps explicit Check/Scenario bindings to native Runs.
A separate publisher uploads those validated Runs, including unsuccessful executions.

## Use an unpublished exact Git revision in CI

Checkout the canonical Azimuth repository at a full commit SHA, then pack into a job-local feed:

```sh
dotnet pack azimuth/packages/dotnet/Azimuth.Testing.Xunit/Azimuth.Testing.Xunit.csproj   --configuration Release --output "$RUNNER_TEMP/azimuth-feed"   -p:PackageVersion=0.1.0-ci.COMMIT

dotnet restore path/to/Tests.csproj   --source "$RUNNER_TEMP/azimuth-feed" --source https://api.nuget.org/v3/index.json

dotnet test path/to/Tests.csproj --no-restore --logger trx
```

The consumer PackageReference version must match the exact packed version; replace COMMIT
with a version-safe commit identifier and record the full canonical SHA as tooling provenance.
NuGet publication is not required. Pin source and version together, use a fresh feed per job,
and never resolve an unpinned branch or floating package version. After tests, preserve typed
reports and TRX with an always-run artifact step before normalization/publication.
