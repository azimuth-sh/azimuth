using Xunit;

namespace LibraryDesk.Tests;

// No marker: this test does not exist for Azimuth, and it fails so that a leak would show.
public class CatalogTests
{
    [Fact]
    public void Unmarked_failing_test()
    {
        Assert.Fail("an unmarked test must never be started by Azimuth");
    }
}
