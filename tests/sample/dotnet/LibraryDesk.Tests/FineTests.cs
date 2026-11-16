using Xunit;

namespace LibraryDesk.Tests;

public class FineTests
{
    [Fact]
    [Trait("Check", "desk-charges-nothing-on-time")]
    public void Charges_nothing_on_the_due_date()
    {
        var due = new DateOnly(2026, 3, 10);

        Assert.Equal(0, Desk.FineDays(due, due));
    }

    // The last row expects a wrong fine on purpose: the adapter must report the whole test as failed.
    [Theory]
    [Trait("Check", "desk-fines-each-late-day")]
    [InlineData(1, 1)]
    [InlineData(3, 3)]
    [InlineData(4, 5)]
    public void Fines_each_late_day(int daysLate, int expectedFine)
    {
        var due = new DateOnly(2026, 3, 10);

        Assert.Equal(expectedFine, Desk.FineDays(due, due.AddDays(daysLate)));
    }

    [Fact(Skip = "The desk has no rule for lost books yet")]
    [Trait("Check", "desk-charges-for-a-lost-book")]
    public void Charges_for_a_lost_book()
    {
    }
}
