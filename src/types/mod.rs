mod duration;
pub use duration::*;
mod timezone;
use itertools::Itertools;
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
mod guess_timezone;
pub use guess_timezone::*;

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
    #[error("Invalid period format: {0}")]
    InvalidPeriodFormat(String),
}

pub trait Value: Sized {
    fn utc_or_local(self) -> Self {
        self
    }

    fn value_type(&self) -> Option<&'static str>;

    fn value(&self) -> String;
}

impl Value for String {
    fn value_type(&self) -> Option<&'static str> {
        Some("TEXT")
    }

    fn value(&self) -> String {
        self.to_owned()
    }
}

impl Value for RRule<Unvalidated> {
    fn value_type(&self) -> Option<&'static str> {
        Some("RECUR")
    }

    fn value(&self) -> String {
        self.to_string()
    }
}

impl<V: Value> Value for Vec<V> {
    fn value_type(&self) -> Option<&'static str> {
        self.first().and_then(Value::value_type)
    }

    fn value(&self) -> String {
        self.iter().map(Value::value).join(",")
    }

    fn utc_or_local(self) -> Self {
        self.into_iter().map(Value::utc_or_local).collect()
    }
}
