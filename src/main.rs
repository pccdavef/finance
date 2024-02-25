#![allow(unused_imports)]
use finance::loan::*;
use simple_logger::SimpleLogger;
use chrono::NaiveDate;
use log::{info, warn};

fn main() {
    SimpleLogger::new().with_level(log::LevelFilter::Info).init().unwrap();
//    test_loan_new();

    let loan = Loan::new(
        200000.0,
        15.,
        7.0,
        PmtSchedule::Monthly,
        Compounding::Daily,
        NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
        NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
        4.0
    );

    loan.show_amortization();

}
