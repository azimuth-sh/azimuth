using System.Diagnostics;

namespace Azimuth.Xunit;

/// <summary>Starts <c>dotnet</c> for the build and the runner, with their output on standard error.</summary>
public static class Processes
{
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
        using var process = Process.Start(start) ?? throw new AdapterException("`dotnet` could not be started");
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
        return (process.ExitCode, output);
    }
}
