using Azimuth.Testing.Xunit;
using Xunit;

namespace Azimuth.Testing.Xunit.SyntheticRunner;

public sealed class ApplicationFixture : IAsyncLifetime
{
    private readonly string reportPath = Environment.GetEnvironmentVariable("AZIMUTH_SYNTHETIC_REPORT")
        ?? throw new InvalidOperationException("Synthetic execution requires an isolated report path.");
    public AzimuthTestSession Evidence { get; }

    public ApplicationFixture() => Evidence = new(reportPath, "native-failing-execution");
    public Task InitializeAsync() => Evidence.InitializeAsync();
    public async Task DisposeAsync()
    {
        await Evidence.DisposeAsync();
        await File.WriteAllTextAsync(reportPath + ".disposed", "disposed");
    }
}

public sealed class RunnerTests(ApplicationFixture fixture) : IClassFixture<ApplicationFixture>
{
    [Fact]
    public Task PassingTest() => fixture.Evidence.RunAsync("synthetic.passing", 0,
        _ => Task.CompletedTask, Array.Empty<ContextSource>());

    [Fact]
    public Task FailingTest() => fixture.Evidence.RunAsync("synthetic.failing", 0,
        _ => throw new InvalidOperationException("expected native runner failure"), Array.Empty<ContextSource>());

    [Fact(Skip = "Synthetic native skip must remain in TRX for reconciliation")]
    public Task SkippedTest() => fixture.Evidence.RunAsync("synthetic.skipped", 0,
        _ => Task.CompletedTask, Array.Empty<ContextSource>());
}
