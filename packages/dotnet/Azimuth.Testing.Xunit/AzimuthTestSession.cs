using System.Text.Json;
using Xunit;

namespace Azimuth.Testing.Xunit;

public sealed class AzimuthTestSession : IAsyncLifetime
{
    private readonly string reportPath;
    private readonly string execution;
    private readonly TimeSpan captureTimeout;
    private readonly long startedAtMs = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();
    private readonly SemaphoreSlim writer = new(1, 1);
    private readonly Dictionary<(string Id, int Attempt), TestFact?> tests = new();
    private bool disposed;

    public AzimuthTestSession(string reportPath, string execution, TimeSpan? captureTimeout = null)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(reportPath);
        ArgumentException.ThrowIfNullOrWhiteSpace(execution);
        this.reportPath = Path.GetFullPath(reportPath);
        this.execution = execution;
        this.captureTimeout = captureTimeout ?? TimeSpan.FromSeconds(10);
        if (this.captureTimeout <= TimeSpan.Zero || this.captureTimeout > TimeSpan.FromMinutes(5))
            throw new ArgumentOutOfRangeException(nameof(captureTimeout));
    }

    public Task InitializeAsync() => FlushAsync();

    public async Task RunAsync(
        string testId,
        int attempt,
        Func<CancellationToken, Task> body,
        IReadOnlyList<ContextSource> contexts,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(body);
        ArgumentNullException.ThrowIfNull(contexts);
        if (contexts.Any(source => source is null || string.IsNullOrWhiteSpace(source.Name) || source.Capture is null) ||
            contexts.Select(source => source.Name).Distinct(StringComparer.Ordinal).Count() != contexts.Count)
            throw new ArgumentException("Context sources must have unique explicit names and capture delegates.", nameof(contexts));
        await ReserveAsync(testId, attempt);
        var started = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();
        var diagnostics = new List<string>();
        var before = await CaptureAsync(contexts, diagnostics, "before", cancellationToken);
        var outcome = "passed";
        try
        {
            cancellationToken.ThrowIfCancellationRequested();
            await body(cancellationToken);
        }
        catch (OperationCanceledException)
        {
            outcome = "interrupted";
            throw;
        }
        catch (TimeoutException)
        {
            outcome = "timed-out";
            throw;
        }
        catch
        {
            outcome = "failed";
            throw;
        }
        finally
        {
            // A canceled test must still retain its completion and context-capture failure.
            var after = await CaptureAsync(contexts, diagnostics, "after", CancellationToken.None);
            var captures = new List<TestContext>();
            foreach (var source in contexts)
            {
                if (!before.TryGetValue(source.Name, out var first) || !after.TryGetValue(source.Name, out var last))
                    continue;
                captures.Add(new(source.Name, first, last));
                if (!ContextProtocol.Equal(first, last))
                    diagnostics.Add($"context-changed:{source.Name}");
            }
            await CompleteAsync(new(testId, attempt, outcome, started,
                DateTimeOffset.UtcNow.ToUnixTimeMilliseconds(), captures, diagnostics));
        }
    }

    public async Task RecordSkippedAsync(string testId, int attempt, string reason)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(reason);
        await ReserveAsync(testId, attempt);
        var now = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();
        await CompleteAsync(new(testId, attempt, "skipped", now, now, Array.Empty<TestContext>(), new[] { reason }));
    }

    public async Task DisposeAsync()
    {
        await writer.WaitAsync();
        try
        {
            if (disposed) return;
            disposed = true;
            var now = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();
            foreach (var key in tests.Where(pair => pair.Value is null).Select(pair => pair.Key).ToArray())
                tests[key] = new(key.Id, key.Attempt, "interrupted", startedAtMs, now,
                    Array.Empty<TestContext>(), new[] { "session-ended-before-test-completion" });
            await WriteAsync();
        }
        finally { writer.Release(); }
    }

    private async Task ReserveAsync(string id, int attempt)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(id);
        ArgumentOutOfRangeException.ThrowIfNegative(attempt);
        await writer.WaitAsync();
        try
        {
            ObjectDisposedException.ThrowIf(disposed, this);
            if (!tests.TryAdd((id, attempt), null))
                throw new InvalidOperationException("A test identity and attempt can be recorded only once per execution.");
            await WriteAsync();
        }
        finally { writer.Release(); }
    }

    private async Task CompleteAsync(TestFact fact)
    {
        await writer.WaitAsync();
        try
        {
            if (disposed) return;
            tests[(fact.Id, fact.Attempt)] = fact;
            await WriteAsync();
        }
        finally { writer.Release(); }
    }

    private async Task FlushAsync()
    {
        await writer.WaitAsync();
        try { await WriteAsync(); }
        finally { writer.Release(); }
    }

    private async Task WriteAsync()
    {
        var facts = tests.Select(pair => pair.Value ?? new TestFact(pair.Key.Id, pair.Key.Attempt, "interrupted",
                startedAtMs, DateTimeOffset.UtcNow.ToUnixTimeMilliseconds(), Array.Empty<TestContext>(),
                new[] { "test-execution-incomplete" }))
            .OrderBy(fact => fact.Id, StringComparer.Ordinal).ThenBy(fact => fact.Attempt).ToArray();
        var report = new TestReport("azimuth-test-report", 1, "xunit-v2", execution, startedAtMs,
            DateTimeOffset.UtcNow.ToUnixTimeMilliseconds(), facts);
        Directory.CreateDirectory(Path.GetDirectoryName(reportPath)!);
        var temporary = reportPath + "." + Guid.NewGuid().ToString("N") + ".tmp";
        try
        {
            await File.WriteAllTextAsync(temporary, JsonSerializer.Serialize(report, ContextProtocol.JsonOptions));
            File.Move(temporary, reportPath, overwrite: true);
        }
        finally { if (File.Exists(temporary)) File.Delete(temporary); }
    }

    private async Task<Dictionary<string, ContextSnapshot>> CaptureAsync(
        IReadOnlyList<ContextSource> sources, List<string> diagnostics, string phase, CancellationToken cancellationToken)
    {
        var result = new Dictionary<string, ContextSnapshot>(StringComparer.Ordinal);
        foreach (var source in sources)
        {
            try
            {
                using var deadline = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
                deadline.CancelAfter(captureTimeout);
                var capture = await source.Capture(deadline.Token).WaitAsync(deadline.Token);
                ArgumentException.ThrowIfNullOrWhiteSpace(capture.Instance);
                var report = ContextProtocol.Freeze(capture.Report);
                if (!report.Contexts.ContainsKey(source.Name))
                    throw new JsonException("Required context is absent from the application report.");
                // A Check depends on its selected context, not unrelated application settings.
                var selected = ExecutionContextReport.Create(new Dictionary<string, ContextDefinition>
                    { [source.Name] = report.Contexts[source.Name] });
                result.Add(source.Name, new(DateTimeOffset.UtcNow.ToUnixTimeMilliseconds(), capture.Instance, selected));
            }
            catch (Exception)
            {
                // Exception messages may contain endpoints or secrets; native logs remain separate artifacts.
                diagnostics.Add($"context-capture-failed:{source.Name}:{phase}");
            }
        }
        return result;
    }
}
