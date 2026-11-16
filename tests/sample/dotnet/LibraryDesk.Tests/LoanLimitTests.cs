using Xunit;

namespace LibraryDesk.Tests;

// The marker sits on the class, so it enrolls both tests.
[Trait("Check", "desk-refuses-a-sixth-loan")]
public class LoanLimitTests
{
    [Fact]
    public void Refuses_a_sixth_loan()
    {
        var desk = new Desk();
        for (var i = 0; i < Desk.LoanLimit; i++)
        {
            Assert.True(desk.Lend("ada"));
        }

        Assert.False(desk.Lend("ada"));
        Assert.Equal(Desk.LoanLimit, desk.LoansOf("ada"));
    }

    [Fact]
    public void Lends_again_after_a_return()
    {
        var desk = new Desk();
        for (var i = 0; i < Desk.LoanLimit; i++)
        {
            desk.Lend("ada");
        }

        desk.Return("ada");

        Assert.True(desk.Lend("ada"));
    }
}
