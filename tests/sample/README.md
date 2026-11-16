# Library desk

An invented product used by Azimuth's own tests. A desk lends books to borrowers, limits how many each may hold, and fines late returns.

- `azimuth/` holds the project file and the model.
- `desk/` stands for the product code.
- `suites/desk-checks/` is a suite of plain text tests, run by the `text-suite-adapter` of Azimuth's test code.
- `dotnet/` is a real xUnit project, used only by the tests of the .NET adapter. The project file does not declare it as a suite.
