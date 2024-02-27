#![allow(unused_imports, dead_code)]
use chrono::NaiveDate;
use finance::loan::*;
use log::{info, warn};
use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();
    //    test_loan_new();

    let loan = Loan::new(
        200000.0,
        15.,
        7.0,
        PmtSchedule::Monthly,
        Compounding::Daily,
        NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
        NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
        4.0,
    );

    loan.show_amortization();
}

// verifies that types can implement the gated traits below
fn is_normal<T: Sized + Send + Sync + Unpin>() {}

#[test]
fn normal_types() {
    is_normal::<LoanPayment>();
}
