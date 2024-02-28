# Finance Requirements

## Loans

### Features
users may add a new loan by providing the following:
- Principal amount
- Loan length in years ("term")
- Interest rate (decimal - i.e. 7.0 vs .07)
- Payment frequency from a defined list
- Compounding frequency from a defined list
- date of the loan
- date of the first payment
- desired precision

New loans calculate 
1. the scheduled payment
2. the amortization schedule

Users can add new payments to the loan by providing
1. the payment amount
2. the payment date
3. the payment sequence number (perhaps provided by the program?)

Payments may be entered out of sequence.

For each entered payment, the user is provided the amount of principal and interest paid and the loan balance based on the last actual payment (or loan origination for the first payment)

When an actual payment is entered, the succeeding scheduled payments in the amortization schedule are optionally recalculated
- if the difference between the scheduled payment and actual payment is more than 30 (30 days early), the following scheduled payment dates are reduced by 1 month and recalculated
- if the difference between the scheduled payment and actual payment is more than -30 (30 days late), the following scheduled payments dates increased by 1 month and recalculated

Users can query for
- the scheduled payment amount
- current loan balance (as of the system date)
- the amortization schedule showing
  - only scheduled payments
  - only actual payments
  - actual payments and remaining scheduled payments
- the amortization schedule will show the payment number, the date (scheduled or actual based on selection), the principal paid, the interest paid and the loan ending balance

Other output
- serialized loan parameters (principal, interest, term, etc.)
- serialized loan payment data (scheduled and actual)

### Roadmap
v0.1.0
- Interactive terminal program
- Entered data is volatile
- Entered data is validated. Erroneous data is rejected with reason and the user is prompted to enter correct data

v0.2.0
- Interactive terminal program
- Entered data is persistent
  - loans may be named and saved
  - loan actual payment data (date and amount) may be edited or deleted
  - loan actual payments that are deleted are replaced by scheduled payments

