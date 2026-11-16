namespace LibraryDesk;

public sealed class Desk
{
    public const int LoanLimit = 5;

    private readonly Dictionary<string, int> _loans = new();

    public bool Lend(string borrower)
    {
        var held = _loans.GetValueOrDefault(borrower);
        if (held >= LoanLimit)
        {
            return false;
        }

        _loans[borrower] = held + 1;
        return true;
    }

    public void Return(string borrower) => _loans[borrower] = _loans.GetValueOrDefault(borrower) - 1;

    public int LoansOf(string borrower) => _loans.GetValueOrDefault(borrower);

    public static int FineDays(DateOnly due, DateOnly returned) =>
        returned > due ? returned.DayNumber - due.DayNumber : 0;
}
