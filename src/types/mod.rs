#[cfg(feature = "chrono")]
mod duration;
#[cfg(feature = "chrono")]
pub use duration::*;
#[cfg(feature = "chrono")]
mod timezone;
#[cfg(feature = "chrono")]
pub use timezone::*;
#[cfg(feature = "chrono")]
mod date;
#[cfg(feature = "chrono")]
pub use date::*;
#[cfg(feature = "chrono")]
mod datetime;
#[cfg(feature = "chrono")]
pub use datetime::*;
#[cfg(feature = "chrono")]
mod dateordatetime;
#[cfg(feature = "chrono")]
pub use dateordatetime::*;

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
