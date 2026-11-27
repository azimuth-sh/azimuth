# Library desk

An invented product used by Azimuth's own tests. A desk lends books to borrowers, limits how many each may hold, and fines late returns.

- `azimuth/` holds the project file and the model.
- `desk/` stands for the product code.
- `suites/desk-checks/` is a suite of plain text tests, run by the `text-suite-adapter` of Azimuth's test code.
- `dotnet/` is a real xUnit project, used only by the tests of the .NET adapter. The project file does not declare it as a suite.
- `playwright/` pins the real Playwright runner with an npm lock file. Its lending and returns suites each keep a config beside their tests. They use no browser fixture. The tests choose the server ports and install the locally packed adapter into each temporary copy of the application.
- `playwright/components/` is a suite whose components run in their own processes. Its config, named so that discovery does not list it, makes one `webServer` entry for each component it is told of, a small server serves each component's report path in the way a test asks, and the suite's own global setup, test and teardown leave a trace in one journal beside the servers' own lines.
- `reading-room/application/` is another application below a repository root with no package manifest. Its two config folders exercise discovery from the folder where the adapter is installed.
