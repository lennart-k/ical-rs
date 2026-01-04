mod duration;
pub use duration::*;
mod timezone;
use rrule::{RRule, Unvalidated};
pub use timezone::*;
mod date;
mod period;
pub use date::*;
mod datetime;
pub use datetime::*;
mod dateordatetime;
pub use dateordatetime::*;
pub use period::*;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CalDateTimeError {
    #[error(
        "Timezone has X-LIC-LOCATION property to specify a timezone from the Olson database, however its value {0} is invalid"
    )]
    InvalidOlson(String),
    #[error("TZID {0} does not refer to a valid timezone")]
    InvalidTZID(String),
    #[error("Timestamp doesn't exist because of gap in local time")]
    LocalTimeGap,
    #[error("Datetime string {0} has an invalid format")]
    InvalidDatetimeFormat(String),
    #[error("Could not parse datetime {0}")]
    ParseError(String),
    #[error("Duration string {0} has an invalid format")]
    InvalidDurationFormat(String),
}

pub trait Value {
    fn value_type(&self) -> &'static str;

    fn value(&self) -> String;
}

impl Value for String {
    fn value_type(&self) -> &'static str {
        "TEXT"
    }

    fn value(&self) -> String {
        self.to_owned()
    }
}

impl Value for RRule<Unvalidated> {
    fn value_type(&self) -> &'static str {
        "RECUR"
    }

    fn value(&self) -> String {
        self.to_string()
    }
}
