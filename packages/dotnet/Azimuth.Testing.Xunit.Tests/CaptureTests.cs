using System.Diagnostics;
using System.Net;
using System.Net.Sockets;
using System.Text.Json;
using Azimuth.Testing.Xunit;
using Xunit;

namespace Azimuth.Testing.Xunit.Tests;

public sealed class CaptureTests : IAsyncLifetime
{
    private readonly string directory = Path.Combine(Path.GetTempPath(), "azimuth-xunit-" + Guid.NewGuid().ToString("N"));
    private string ReportPath => Path.Combine(directory, "report.json");
    private AzimuthTestSession session = null!;

    public async Task InitializeAsync()
    {
        session = new(ReportPath, "synthetic-execution");
        await session.InitializeAsync();
    }

    public async Task DisposeAsync()
    {
        await session.DisposeAsync();
        Directory.Delete(directory, recursive: true);
    }

    [Fact]
    public async Task Captures_actual_overridden_application_settings_and_test_failures()
    {
        await using var application = new SettingsApplication(enabled: false);
        var source = application.Source;
        await session.RunAsync("permissions.disabled", 0, async _ =>
        {
            Assert.False((await application.ReadAsync(CancellationToken.None)).Report.Contexts["authentication"].Values["enabled"].GetBoolean());
        }, new[] { source });
        await Assert.ThrowsAsync<InvalidOperationException>(() => session.RunAsync("permissions.failure", 0,
            _ => throw new InvalidOperationException("synthetic test failure"), new[] { source }));
        var report = Read();
        Assert.Equal("azimuth-test-report", report.Format);
        Assert.Equal("xunit-v2", report.Framework);
        Assert.Equal(new[] { "passed", "failed" }, report.Tests.Select(test => test.Outcome));
        foreach (var test in report.Tests)
        {
            var context = Assert.Single(test.Contexts);
            Assert.False(context.Before.Report.Contexts["authentication"].Values["enabled"].GetBoolean());
            Assert.Equal(context.Before.Instance, context.After.Instance);
            Assert.True(context.Before.CapturedAtMs <= context.After.CapturedAtMs);
            Assert.Empty(test.Diagnostics);
        }
    }

    [Fact]
    public async Task Preserves_parallel_attempts_and_explicit_skips_in_stable_identity_order()
    {
        await Task.WhenAll(Enumerable.Range(0, 24).Select(index => session.RunAsync("calculation." + index.ToString("D2"),
            0, _ => Task.CompletedTask, Array.Empty<ContextSource>())));
        await Assert.ThrowsAsync<Exception>(() => session.RunAsync("retried", 0,
            _ => throw new Exception("first attempt"), Array.Empty<ContextSource>()));
        await session.RunAsync("retried", 1, _ => Task.CompletedTask, Array.Empty<ContextSource>());
        await session.RecordSkippedAsync("skipped", 0, "Unavailable optional platform");
        await session.DisposeAsync();
        var report = Read();
        Assert.Equal(27, report.Tests.Count);
        Assert.Equal(new[] { "failed", "passed" }, report.Tests.Where(test => test.Id == "retried").Select(test => test.Outcome));
        Assert.Equal("skipped", report.Tests.Last().Outcome);
        Assert.Empty(Directory.GetFiles(directory, "*.tmp"));
    }

    [Fact]
    public async Task Missing_context_does_not_suppress_the_test_or_its_outcome()
    {
        var missing = new ContextSource("authentication", _ => throw new InvalidOperationException("secret must not leak"));
        var ran = false;
        await session.RunAsync("capture.failure", 0, _ => { ran = true; return Task.CompletedTask; }, new[] { missing });
        Assert.True(ran);
        var fact = Assert.Single(Read().Tests);
        Assert.Equal("passed", fact.Outcome);
        Assert.Empty(fact.Contexts);
        Assert.Equal(new[] { "context-capture-failed:authentication:before", "context-capture-failed:authentication:after" }, fact.Diagnostics);
        Assert.DoesNotContain("secret must not leak", File.ReadAllText(ReportPath));
    }

    [Fact]
    public async Task Detects_effective_context_change_and_instance_change()
    {
        await using var application = new SettingsApplication(enabled: true);
        await session.RunAsync("configuration.reload", 0, _ => { application.Enabled = false; return Task.CompletedTask; }, new[] { application.Source });
        var calls = 0;
        var replaced = new ContextSource("authentication", async token =>
            (await application.ReadAsync(token)) with { Instance = "instance-" + Interlocked.Increment(ref calls) });
        await session.RunAsync("instance.replaced", 0, _ => Task.CompletedTask, new[] { replaced });
        Assert.All(Read().Tests, fact => Assert.Contains("context-changed:authentication", fact.Diagnostics));
    }

    [Fact]
    public async Task Cancellation_timeout_and_duplicate_identity_remain_distinct()
    {
        using var canceled = new CancellationTokenSource();
        canceled.Cancel();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => session.RunAsync("canceled", 0,
            _ => Task.CompletedTask, Array.Empty<ContextSource>(), canceled.Token));
        await Assert.ThrowsAsync<TimeoutException>(() => session.RunAsync("timeout", 0,
            _ => throw new TimeoutException(), Array.Empty<ContextSource>()));
        await Assert.ThrowsAsync<InvalidOperationException>(() => session.RunAsync("timeout", 0,
            _ => Task.CompletedTask, Array.Empty<ContextSource>()));
        Assert.Equal(new[] { "interrupted", "timed-out" }, Read().Tests.Select(test => test.Outcome));
    }

    [Theory]
    [InlineData("null")]
    [InlineData("-1")]
    [InlineData("1.5")]
    [InlineData("9007199254740992")]
    [InlineData("{}")]
    public void Rejects_non_contract_scalar_values(string value)
    {
        var json = "{\"format\":\"azimuth-execution-context\",\"version\":2,\"contexts\":{\"configuration\":{\"contract_revision\":\"sha256:" + new string('a', 64) + "\",\"values\":{\"field\":" + value + "}}}}";
        Assert.Throws<JsonException>(() => ContextProtocol.Parse(json));
    }

    [Fact]
    public async Task Native_xunit_failure_keeps_report_and_executes_fixture_cleanup()
    {
        var childReport = Path.Combine(directory, "child.json");
        var project = Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "../../../SyntheticRunner/SyntheticRunner.csproj"));
        var start = new ProcessStartInfo("dotnet") { RedirectStandardOutput = true, RedirectStandardError = true };
        foreach (var argument in new[] { "test", project, "--nologo", "--logger", "trx", "--results-directory", Path.Combine(directory, "trx") })
            start.ArgumentList.Add(argument);
        start.Environment["AZIMUTH_SYNTHETIC_REPORT"] = childReport;
        using var child = Process.Start(start)!;
        var stdout = child.StandardOutput.ReadToEndAsync();
        var stderr = child.StandardError.ReadToEndAsync();
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        try { await child.WaitForExitAsync(deadline.Token); }
        catch { child.Kill(entireProcessTree: true); throw; }
        var output = await stdout;
        var errors = await stderr;
        Assert.True(child.ExitCode == 1, output + errors);
        Assert.Contains("Failed:", output);
        Assert.Contains("expected native runner failure", output);
        Assert.Matches(@"^\[xUnit\.net [^\]]+\]\s+Azimuth\.Testing\.Xunit\.SyntheticRunner\.RunnerTests\.FailingTest \[FAIL\]\s*$", errors);
        Assert.True(File.Exists(childReport + ".disposed"));
        var report = JsonSerializer.Deserialize<TestReport>(await File.ReadAllTextAsync(childReport), ContextProtocol.JsonOptions)!;
        Assert.Equal(new[] { "failed", "passed" }, report.Tests.Select(test => test.Outcome));
        var trx = await File.ReadAllTextAsync(Assert.Single(Directory.GetFiles(Path.Combine(directory, "trx"), "*.trx")));
        Assert.Contains("SkippedTest", trx);
        Assert.Contains("outcome=\"NotExecuted\"", trx);
    }

    [Fact]
    public async Task Context_capture_deadline_preserves_the_body_and_report()
    {
        var bounded = new AzimuthTestSession(Path.Combine(directory, "bounded.json"), "bounded", TimeSpan.FromMilliseconds(20));
        var source = new ContextSource("authentication", async token =>
        {
            await Task.Delay(Timeout.InfiniteTimeSpan, token);
            throw new InvalidOperationException();
        });
        var ran = false;
        await bounded.RunAsync("bounded", 0, _ => { ran = true; return Task.CompletedTask; }, new[] { source });
        await bounded.DisposeAsync();
        Assert.True(ran);
        var report = JsonSerializer.Deserialize<TestReport>(File.ReadAllText(Path.Combine(directory, "bounded.json")), ContextProtocol.JsonOptions)!;
        Assert.Equal(2, Assert.Single(report.Tests).Diagnostics.Count);
    }

    private TestReport Read() => JsonSerializer.Deserialize<TestReport>(File.ReadAllText(ReportPath), ContextProtocol.JsonOptions)!;

    private sealed class SettingsApplication : IAsyncDisposable
    {
        private readonly TcpListener listener = new(IPAddress.Loopback, 0);
        private readonly CancellationTokenSource shutdown = new();
        private readonly Task server;
        private readonly HttpClient client = new();
        private readonly string instance = Guid.NewGuid().ToString("N");
        public bool Enabled { get; set; }
        public ContextSource Source => new("authentication", ReadAsync);

        public SettingsApplication(bool enabled)
        {
            Enabled = enabled;
            listener.Start();
            client.BaseAddress = new Uri("http://127.0.0.1:" + ((IPEndPoint)listener.LocalEndpoint).Port);
            server = ServeAsync();
        }

        public async Task<ContextCapture> ReadAsync(CancellationToken cancellationToken)
        {
            var json = await client.GetStringAsync("/internal/execution-context", cancellationToken);
            return new(instance, ContextProtocol.Parse(json));
        }

        private async Task ServeAsync()
        {
            try
            {
                while (!shutdown.IsCancellationRequested)
                {
                    using var connection = await listener.AcceptTcpClientAsync(shutdown.Token);
                    await using var stream = connection.GetStream();
                    using var reader = new StreamReader(stream, leaveOpen: true);
                    while (!string.IsNullOrEmpty(await reader.ReadLineAsync(shutdown.Token))) { }
                    var report = ExecutionContextReport.Create(new Dictionary<string, ContextDefinition>
                    {
                        ["authentication"] = new("sha256:" + new string('a', 64), new Dictionary<string, JsonElement>
                        { ["enabled"] = JsonSerializer.SerializeToElement(Enabled) })
                    });
                    var bytes = System.Text.Encoding.UTF8.GetBytes(JsonSerializer.Serialize(report, ContextProtocol.JsonOptions));
                    var header = System.Text.Encoding.ASCII.GetBytes("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: " + bytes.Length + "\r\nConnection: close\r\n\r\n");
                    await stream.WriteAsync(header, shutdown.Token);
                    await stream.WriteAsync(bytes, shutdown.Token);
                }
            }
            catch (OperationCanceledException) when (shutdown.IsCancellationRequested) { }
        }

        public async ValueTask DisposeAsync()
        {
            await shutdown.CancelAsync();
            await server;
            listener.Stop();
            client.Dispose();
            shutdown.Dispose();
        }
    }
}
