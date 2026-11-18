using System.Diagnostics;
using System.Runtime.InteropServices;

namespace Azimuth.Xunit;

/// <summary>Starts <c>dotnet</c> for the build and the runner, with their output on standard error.</summary>
public static class Processes
{
    private static readonly object Gate = new();
    private static readonly ManualResetEventSlim StopFinished = new(false);
    private static Process? active;
    private static int state;

    [DllImport("libc", SetLastError = true)]
    private static extern int kill(int pid, int signal);

    public static void Stop()
    {
        if (Interlocked.Exchange(ref state, 1) == 1)
        {
            return;
        }
        try
        {
            int? runner;
            lock (Gate)
            {
                runner = active is { HasExited: false } ? active.Id : null;
            }
            if (runner is { } pid)
            {
                StopTree(pid);
            }
        }
        finally
        {
            StopFinished.Set();
        }
    }

    public static void WaitForStop()
    {
        if (Volatile.Read(ref state) == 1)
        {
            StopFinished.Wait();
        }
    }

    private static void ThrowIfStopping()
    {
        if (Volatile.Read(ref state) == 1)
        {
            throw new OperationCanceledException("the adapter was asked to stop");
        }
    }

    private static void StopTree(int root)
    {
        var start = new ProcessStartInfo("ps")
        {
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
        };
        start.ArgumentList.Add("-axo");
        start.ArgumentList.Add("pid=,ppid=");
        using var listing = Process.Start(start) ?? throw new AdapterException("could not list the runner's processes");
        var lines = listing.StandardOutput.ReadToEnd();
        listing.WaitForExit();
        var children = new Dictionary<int, List<int>>();
        foreach (var line in lines.Split('\n', StringSplitOptions.RemoveEmptyEntries))
        {
            var words = line.Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries);
            if (words.Length == 2 && int.TryParse(words[0], out var pid) && int.TryParse(words[1], out var parent))
            {
                if (!children.TryGetValue(parent, out var owned))
                {
                    children[parent] = owned = [];
                }
                owned.Add(pid);
            }
        }
        var processes = new List<Process>();
        void Collect(int pid)
        {
            if (children.TryGetValue(pid, out var owned))
            {
                foreach (var child in owned)
                {
                    Collect(child);
                }
            }
            try
            {
                processes.Add(Process.GetProcessById(pid));
            }
            catch (ArgumentException)
            {
                // A process that already exited needs no stop.
            }
        }
        Collect(root);
        foreach (var process in processes)
        {
            using (process)
            {
                // Let a child's cleanup finish before its parent can tear it down.
                if (!process.HasExited)
                {
                    if (kill(process.Id, 15) != 0 && Marshal.GetLastPInvokeError() != 3)
                    {
                        Console.Error.WriteLine($"azimuth-xunit: could not stop runner process {process.Id}: {Marshal.GetLastPInvokeError()}");
                    }
                    process.WaitForExit();
                }
            }
        }
    }

    public static bool WriteReport(string report)
    {
        if (Interlocked.CompareExchange(ref state, 2, 0) != 0)
        {
            return false;
        }
        Console.Out.Write(report);
        return true;
    }

    public static int Dotnet(string folder, params string[] arguments) => Start(folder, arguments, captureOutput: false).Status;

    /// <summary>Runs <c>dotnet</c> and returns what it wrote on standard output.</summary>
    public static (int Status, string Output) DotnetOutput(string folder, params string[] arguments) => Start(folder, arguments, captureOutput: true);

    private static (int Status, string Output) Start(string folder, string[] arguments, bool captureOutput)
    {
        var start = new ProcessStartInfo("dotnet")
        {
            WorkingDirectory = folder,
            RedirectStandardOutput = true,
            RedirectStandardInput = true,
            UseShellExecute = false,
        };
        foreach (var argument in arguments)
        {
            start.ArgumentList.Add(argument);
        }
        using var process = new Process { StartInfo = start };
        lock (Gate)
        {
            ThrowIfStopping();
            if (!process.Start())
            {
                throw new AdapterException("`dotnet` could not be started");
            }
            active = process;
        }
        try
        {
            process.StandardInput.Close();
            string output;
            if (captureOutput)
            {
                output = process.StandardOutput.ReadToEnd();
            }
            else
            {
                // Standard output belongs to the report, so the runner's output goes to standard error.
                process.StandardOutput.BaseStream.CopyTo(Console.OpenStandardError());
                output = "";
            }
            process.WaitForExit();
            ThrowIfStopping();
            return (process.ExitCode, output);
        }
        finally
        {
            lock (Gate)
            {
                active = null;
            }
        }
    }
}
