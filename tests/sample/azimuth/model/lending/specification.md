# Lending

A loan is one book lent to one borrower until its due date.

### Requirement: loan-limit-holds
A borrower holds at most five loans at a time.

#### Scenario: sixth-loan-is-refused
A borrower who holds five loans asks for a sixth book.
The desk refuses the loan and the borrower keeps five.

#### Scenario: returned-book-frees-a-slot
A borrower who holds five loans returns one book and asks for another.
The desk lends it.
