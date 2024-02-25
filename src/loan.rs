#![allow(unused_imports)]
use chrono::{Datelike, NaiveDate};
use log::{info, trace, warn};
use std::{collections::HashMap, fmt};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PmtSchedule {
    Weekly,
    Biweekly,
    SemiMonthly,
    Monthly,
    Quarterly,
    SemiAnnually,
    Annually,
}

impl fmt::Display for PmtSchedule {
    #[allow(clippy::recursive_format_impl)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Compounding {
    Daily,
    Monthly,
    Quarterly,
    SemiAnnually,
    Annually,
}

#[derive(PartialEq, Debug)]
pub struct LoanPayment {
    pub pmt_number: i32,
    pub pmt_date: NaiveDate,
    pub pmt_amount: f64,
    pub pmt_interest_paid: f64,
    pub pmt_end_balance: f64,
}

impl LoanPayment {
    pub fn new(
        pmt_number: i32,
        pmt_date: NaiveDate,
        pmt_amount: f64,
        pmt_interest_paid: f64,
        pmt_end_balance: f64,
    ) -> Self {
        Self {
            pmt_number,
            pmt_date,
            pmt_amount,
            pmt_interest_paid,
            pmt_end_balance,
        }
    }
}

impl fmt::Display for LoanPayment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "pmt number {}, date {}, payment ${:.4}, interest paid ${:.4}, ending balance ${:.4}",
            self.pmt_number,
            self.pmt_date,
            self.pmt_amount,
            self.pmt_interest_paid,
            self.pmt_end_balance
        )
    }
}

#[derive(PartialEq, Debug)]
pub struct Loan {
    pub principal: f64,
    pub term: f64,
    pub annual_rate: f64,
    pub pmt_schedule: PmtSchedule,
    pub compound_type: Compounding,
    pub loan_date: NaiveDate,
    pub first_pmt_date: NaiveDate,
    pub dec_places: f64,
    pmt_amount: f64,
    scheduled_pmts: Vec<LoanPayment>,
    actual_pmts: Vec<LoanPayment>,
}

impl Loan {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        principal: f64,
        term: f64,
        annual_rate: f64,
        pmt_schedule: PmtSchedule,
        compound_type: Compounding,
        loan_date: NaiveDate,
        first_pmt_date: NaiveDate,
        dec_places: f64,
    ) -> Self {
        let pmt_amount = get_pmt_amount(
            &principal,
            &term,
            &annual_rate,
            &pmt_schedule,
            &compound_type,
            &dec_places,
        );
        Self {
            principal,
            term,
            annual_rate,
            pmt_schedule,
            compound_type,
            loan_date,
            first_pmt_date,
            dec_places,
            pmt_amount,
            scheduled_pmts: add_scheduled_pmts(
                &principal,
                &loan_date,
                &first_pmt_date,
                &annual_rate,
                &pmt_schedule,
                &compound_type,
                &dec_places,
                pmt_amount,
            ),
            actual_pmts: Vec::new(),
        }
    }

    pub fn get_pmt_amount(&self) -> &f64 {
        &self.pmt_amount
    }

    pub fn get_pmt_count(&self) -> usize {
        self.scheduled_pmts.len()
    }

    pub fn get_pmt_info(&self, &pmt_number: &usize) -> String {
        if pmt_number <= self.get_pmt_count() {
            self.scheduled_pmts[pmt_number - 1].to_string()
        } else {
            "No payment information.".to_string()
        }
    }

    pub fn get_pmt_detail(&self, &pmt_number: &usize) -> Option<&LoanPayment> {
        if pmt_number <= self.get_pmt_count() {
            Some(&self.scheduled_pmts[pmt_number])
        } else {
            None
        }
    }

    pub fn show_amortization(&self) {
        for pmt in &self.scheduled_pmts {
            println!("{}", pmt);
        }
    }
}

fn round(amt: f64, dec: f64) -> f64 {
    if amt == 0. {
        0.
    } else {
        (amt * 10_f64.powf(dec)).round() / 10_f64.powf(dec)
    }
}

fn get_pmt_amount(
    &principal: &f64,             // loan principal
    &term: &f64,                  // term of loan (expected in years)
    &annual_rate: &f64,           // annual interest rate as decimal (i.e., 2.5, 7.0)
    &pmt_schedule: &PmtSchedule,  // payment frequency
    &compound_type: &Compounding, // interest compounding frequency
    &dec_places: &f64,            // calculate to dec_places
) -> f64 {
    let compounding_periods = get_compounding_periods(compound_type);
    let pmt_count = get_pmt_schedule(pmt_schedule);

    let pmt_rate = ((1. + ((annual_rate / 100.) / compounding_periods))
        .powf(compounding_periods / pmt_count))
        - 1.0;

    let total_pmts = term * pmt_count;
    let factor = (1. + pmt_rate).powf(total_pmts);

    // return the result to specified decimal places
    round((principal * pmt_rate * factor) / (factor - 1.), dec_places)
}

// calculate a vector of scheduled LoanPayment to add to Loan during New
#[allow(clippy::too_many_arguments)]
fn add_scheduled_pmts(
    &principal: &f64,
    &loan_date: &NaiveDate,
    &first_pmt_date: &NaiveDate,
    &annual_rate: &f64,
    &pmt_schedule: &PmtSchedule,
    &compound_type: &Compounding,
    &dec_places: &f64,
    pmt_amount: f64,
) -> Vec<LoanPayment> {
    let mut sched_pmt: Vec<LoanPayment> = Vec::new();

    let compounding_periods = get_compounding_periods(compound_type);
    let pmt_frequency = get_pmt_schedule(pmt_schedule);

    let mut end_balance = 1.; // arbitrary value > 0. Will be set by calculation in the loop.
    let mut begin_balance = principal; // beginning balance for the compounding period
    let mut pmt_number = 0; // incremental payment number
    let mut pmt_amt = pmt_amount; // the amount of each payment
    let mut begin_date: NaiveDate = loan_date; // beginning date of the compounding period
    let mut end_date: NaiveDate = first_pmt_date; // end date of the compounding period
    let mut period_interest_rate = 0.; // rate applied to the principal to determine interest
    let mut interest; // interest payment
    let mut days; // length of the compounding period in days
    let mut common_rates = HashMap::new(); // HashMap of common compound interest rates
    let daily_rate = (annual_rate / 100.) / compounding_periods;

    if compounding_periods == 365. {
        // create hashmap of period interest rates for common durations (28, 29, 30 and 31 days)
        for i in [28, 29, 30, 31] {
            common_rates.insert(i, (1. + daily_rate).powi(i) - 1.);
        }
    } else {
        // calculate the period interest rate based on payment schedule and compounding type
        if pmt_frequency == compounding_periods {
            period_interest_rate = daily_rate;
        } else {
            period_interest_rate = (1. + daily_rate).powf(compounding_periods / pmt_frequency) - 1.;
        }
    }

    while end_balance > 0. && pmt_number < 500 {
        if pmt_number > 0 {
            begin_date = end_date;
            end_date = get_next_pmt_date(&begin_date, &pmt_schedule);
            begin_balance = end_balance;
        }

        pmt_number += 1;

        if compounding_periods == 365. {
            days = end_date.signed_duration_since(begin_date).num_days() as i32;
            period_interest_rate = common_rates
                .get(&days)
                .copied()
                .unwrap_or((1. + daily_rate).powi(days) - 1.);
        }
        trace!(
            "pmt # {}, period interest rate {}",
            pmt_number,
            period_interest_rate
        );

        interest = begin_balance * period_interest_rate;

        if pmt_amt <= begin_balance {
            end_balance = begin_balance - (pmt_amt - interest);
        } else {
            pmt_amt = begin_balance + interest;
            end_balance = 0.;
        }
        trace!(
            "Pmt # {}, end date {}, interest {}, end bal {}",
            pmt_number,
            end_date,
            interest,
            end_balance
        );

        sched_pmt.push(LoanPayment::new(
            pmt_number,
            end_date,
            round(pmt_amt, dec_places),
            round(interest, dec_places),
            round(end_balance, dec_places),
        ));
    }
    sched_pmt
}

fn get_compounding_periods(compound_type: Compounding) -> f64 {
    match compound_type {
        Compounding::Daily => 365.,
        Compounding::Monthly => 12.,
        Compounding::Quarterly => 4.,
        Compounding::SemiAnnually => 2.,
        Compounding::Annually => 1.,
    }
}

fn get_pmt_schedule(pmt_schedule: PmtSchedule) -> f64 {
    match pmt_schedule {
        PmtSchedule::Weekly => 52.,
        PmtSchedule::Biweekly => 26.,
        PmtSchedule::SemiMonthly => 24.,
        PmtSchedule::Monthly => 12.,
        PmtSchedule::Quarterly => 4.,
        PmtSchedule::SemiAnnually => 2.,
        PmtSchedule::Annually => 1.,
    }
}

fn get_next_pmt_date(&begin_date: &NaiveDate, &pmt_schedule: &PmtSchedule) -> NaiveDate {
    let day = begin_date.day();
    let mon = begin_date.month();
    let yr = begin_date.year();
    let end_date: Option<NaiveDate>;

    match &pmt_schedule {
        PmtSchedule::Weekly => {
            end_date = begin_date.checked_add_days(chrono::Days::new(7));
        }
        PmtSchedule::Biweekly => {
            end_date = begin_date.checked_add_days(chrono::Days::new(14));
        }
        // semi-monthly payments are presumed to be made on the 1st and 15th of each month
        PmtSchedule::SemiMonthly => {
            if day == 1 {
                end_date = NaiveDate::from_ymd_opt(yr, mon, 15);
            } else if mon == 12 {
                end_date = NaiveDate::from_ymd_opt(yr + 1, 1, 1);
            } else {
                end_date = NaiveDate::from_ymd_opt(yr, mon + 1, 1);
            }
        }
        PmtSchedule::Monthly => {
            end_date = begin_date.checked_add_months(chrono::Months::new(1));
        }
        PmtSchedule::Quarterly => {
            end_date = begin_date.checked_add_months(chrono::Months::new(3));
        }
        PmtSchedule::SemiAnnually => {
            end_date = begin_date.checked_add_months(chrono::Months::new(6));
        }
        PmtSchedule::Annually => {
            end_date = begin_date.checked_add_months(chrono::Months::new(12));
        }
    }

    match end_date {
        Some(end_date) => end_date,
        None => panic!("{} does not return a new payment date", begin_date),
    }
}

#[cfg(test)]
mod tests {
    use super::{get_next_pmt_date, get_pmt_amount, Compounding, Loan, LoanPayment, PmtSchedule};
    use chrono::NaiveDate;
    use test_log::test;

    #[test]
    fn test_get_next_pmt_date() {
        let mut begin_date = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();

        // base cases
        assert_eq!(
            get_next_pmt_date(&begin_date, &PmtSchedule::Weekly),
            NaiveDate::from_ymd_opt(2024, 2, 8).unwrap()
        );
        assert_eq!(
            get_next_pmt_date(&begin_date, &PmtSchedule::Biweekly),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap()
        );
        assert_eq!(
            get_next_pmt_date(&begin_date, &PmtSchedule::SemiMonthly),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap()
        );
        assert_eq!(
            get_next_pmt_date(&begin_date, &PmtSchedule::Monthly),
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()
        );
        assert_eq!(
            get_next_pmt_date(&begin_date, &PmtSchedule::Quarterly),
            NaiveDate::from_ymd_opt(2024, 5, 1).unwrap()
        );
        assert_eq!(
            get_next_pmt_date(&begin_date, &PmtSchedule::SemiAnnually),
            NaiveDate::from_ymd_opt(2024, 8, 1).unwrap()
        );
        assert_eq!(
            get_next_pmt_date(&begin_date, &PmtSchedule::Annually),
            NaiveDate::from_ymd_opt(2025, 2, 1).unwrap()
        );

        begin_date = NaiveDate::from_ymd_opt(2023, 12, 15).unwrap();
        assert_eq!(
            get_next_pmt_date(&begin_date, &PmtSchedule::SemiMonthly),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
        );

        begin_date = NaiveDate::from_ymd_opt(2022, 8, 30).unwrap();
        assert_eq!(
            get_next_pmt_date(&begin_date, &PmtSchedule::SemiAnnually),
            NaiveDate::from_ymd_opt(2023, 2, 28).unwrap()
        );

        begin_date = NaiveDate::from_ymd_opt(2022, 11, 30).unwrap();
        assert_eq!(
            get_next_pmt_date(&begin_date, &PmtSchedule::Quarterly),
            NaiveDate::from_ymd_opt(2023, 2, 28).unwrap()
        );
    }

    #[test]
    fn test_get_pmt_amount() {
        // exhaustive test of payment calculations
        let principal = 200000.;
        let term = 15.;
        let annual_rate = 7.;
        let dec_places = 2.;

        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Weekly,
                &Compounding::Daily,
                &dec_places
            ),
            414.42
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Biweekly,
                &Compounding::Daily,
                &dec_places
            ),
            829.40
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::SemiMonthly,
                &Compounding::Daily,
                &dec_places
            ),
            898.62
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Monthly,
                &Compounding::Daily,
                &dec_places
            ),
            1799.87
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Quarterly,
                &Compounding::Daily,
                &dec_places
            ),
            5431.26
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::SemiAnnually,
                &Compounding::Daily,
                &dec_places
            ),
            10958.39
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Annually,
                &Compounding::Daily,
                &dec_places
            ),
            22307.07
        );

        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Weekly,
                &Compounding::Monthly,
                &dec_places
            ),
            413.92
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Biweekly,
                &Compounding::Monthly,
                &dec_places
            ),
            828.39
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::SemiMonthly,
                &Compounding::Monthly,
                &dec_places
            ),
            897.52
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Monthly,
                &Compounding::Monthly,
                &dec_places
            ),
            1797.66
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Quarterly,
                &Compounding::Monthly,
                &dec_places
            ),
            5424.49
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::SemiAnnually,
                &Compounding::Monthly,
                &dec_places
            ),
            10944.46
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Annually,
                &Compounding::Monthly,
                &dec_places
            ),
            22277.61
        );

        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Weekly,
                &Compounding::Quarterly,
                &dec_places
            ),
            412.88
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Biweekly,
                &Compounding::Quarterly,
                &dec_places
            ),
            826.31
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::SemiMonthly,
                &Compounding::Quarterly,
                &dec_places
            ),
            895.27
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Monthly,
                &Compounding::Quarterly,
                &dec_places
            ),
            1793.14
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Quarterly,
                &Compounding::Quarterly,
                &dec_places
            ),
            5410.67
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::SemiAnnually,
                &Compounding::Quarterly,
                &dec_places
            ),
            10916.03
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Annually,
                &Compounding::Quarterly,
                &dec_places
            ),
            22217.470
        );

        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Weekly,
                &Compounding::SemiAnnually,
                &dec_places
            ),
            411.36
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Biweekly,
                &Compounding::SemiAnnually,
                &dec_places
            ),
            823.27
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::SemiMonthly,
                &Compounding::SemiAnnually,
                &dec_places
            ),
            891.97
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Monthly,
                &Compounding::SemiAnnually,
                &dec_places
            ),
            1786.50
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Quarterly,
                &Compounding::SemiAnnually,
                &dec_places
            ),
            5390.37
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::SemiAnnually,
                &Compounding::SemiAnnually,
                &dec_places
            ),
            10874.27
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Annually,
                &Compounding::SemiAnnually,
                &dec_places
            ),
            22129.13
        );

        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Weekly,
                &Compounding::Annually,
                &dec_places
            ),
            408.43
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Biweekly,
                &Compounding::Annually,
                &dec_places
            ),
            817.39
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::SemiMonthly,
                &Compounding::Annually,
                &dec_places
            ),
            885.60
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Monthly,
                &Compounding::Annually,
                &dec_places
            ),
            1773.70
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Quarterly,
                &Compounding::Annually,
                &dec_places
            ),
            5351.24
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::SemiAnnually,
                &Compounding::Annually,
                &dec_places
            ),
            10793.77
        );
        assert_eq!(
            get_pmt_amount(
                &principal,
                &term,
                &annual_rate,
                &PmtSchedule::Annually,
                &Compounding::Annually,
                &dec_places
            ),
            21958.92
        );
    }

    #[test]
    fn test_daily_compound_loan() {
        let loan = Loan::new(
            200000.,
            15.,
            7.,
            PmtSchedule::Monthly,
            Compounding::Daily,
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
            4.,
        );

        assert_eq!(loan.get_pmt_amount(), &1799.8691);
        assert_eq!(loan.get_pmt_count(), 182);
        assert_eq!(loan.get_pmt_info(&1), "pmt number 1, date 2024-04-01, payment $1799.8691, interest paid $1772.0185, ending balance $199972.1494");
        assert_eq!(loan.get_pmt_info(&2), "pmt number 2, date 2024-05-01, payment $1799.8691, interest paid $1153.7298, ending balance $199326.0101");
        assert_eq!(loan.get_pmt_info(&20), "pmt number 20, date 2025-11-01, payment $1799.8691, interest paid $1121.3342, ending balance $187390.9439");
        assert_eq!(loan.get_pmt_info(&21), "pmt number 21, date 2025-12-01, payment $1799.8691, interest paid $1081.1432, ending balance $186672.2180");
        assert_eq!(loan.get_pmt_info(&22), "pmt number 22, date 2026-01-01, payment $1799.8691, interest paid $1113.0032, ending balance $185985.3521");
        assert_eq!(loan.get_pmt_info(&182), "pmt number 182, date 2039-05-01, payment $93.7322, interest paid $0.5377, ending balance $0.0000");
    }

    #[test]
    fn test_monthly_compound_loan() {
        let loan = Loan::new(
            200000.,
            15.,
            7.,
            PmtSchedule::Monthly,
            Compounding::Monthly,
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
            4.,
        );

        assert_eq!(loan.get_pmt_amount(), &1797.6565);
        assert_eq!(loan.get_pmt_count(), 180);
        assert_eq!(loan.get_pmt_info(&1), "pmt number 1, date 2024-04-01, payment $1797.6565, interest paid $1166.6667, ending balance $199369.0102");
        assert_eq!(loan.get_pmt_info(&2), "pmt number 2, date 2024-05-01, payment $1797.6565, interest paid $1162.9859, ending balance $198734.3396");
        assert_eq!(loan.get_pmt_info(&20), "pmt number 20, date 2025-11-01, payment $1797.6565, interest paid $1092.9361, ending balance $186655.7608");
        assert_eq!(loan.get_pmt_info(&30), "pmt number 30, date 2026-09-01, payment $1797.6565, interest paid $1050.7314, ending balance $179378.4562");
        assert_eq!(loan.get_pmt_info(&40), "pmt number 40, date 2027-07-01, payment $1797.6565, interest paid $1005.9991, ending balance $171665.3236");
        assert_eq!(loan.get_pmt_info(&180), "pmt number 180, date 2039-03-01, payment $1797.6697, interest paid $10.4256, ending balance $0.0000");
    }

    #[test]
    fn test_quarter_compound_loan() {
        let loan = Loan::new(
            200000.,
            15.,
            7.,
            PmtSchedule::Monthly,
            Compounding::Quarterly,
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
            4.,
        );

        assert_eq!(loan.get_pmt_amount(), &1793.1377);
        assert_eq!(loan.get_pmt_count(), 180);
        assert_eq!(loan.get_pmt_info(&1), "pmt number 1, date 2024-04-01, payment $1793.1377, interest paid $1159.9265, ending balance $199366.7888");
        assert_eq!(loan.get_pmt_info(&2), "pmt number 2, date 2024-05-01, payment $1793.1377, interest paid $1156.2541, ending balance $198729.9052");
        assert_eq!(loan.get_pmt_info(&20), "pmt number 20, date 2025-11-01, payment $1793.1377, interest paid $1086.3865, ending balance $186613.1317");
        assert_eq!(loan.get_pmt_info(&30), "pmt number 30, date 2026-09-01, payment $1793.1377, interest paid $1044.3111, ending balance $179316.2120");
        assert_eq!(loan.get_pmt_info(&40), "pmt number 40, date 2027-07-01, payment $1793.1377, interest paid $999.7307, ending balance $171584.8806");
        assert_eq!(loan.get_pmt_info(&180), "pmt number 180, date 2039-03-01, payment $1793.1302, interest paid $10.3395, ending balance $0.0000");
    }
}
