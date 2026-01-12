use crate::types::{CalDateTimeError, Timezone, Value};
use crate::{property::ContentLine, types::CalDateTime};
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime};
use chrono_tz::Tz;
use std::{collections::HashMap, ops::Add};

pub const LOCAL_DATE: &str = "%Y%m%d";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalDate(pub NaiveDate, pub Timezone);

impl PartialOrd for CalDate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CalDate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_datetime().cmp(&other.as_datetime())
    }
}

impl Add<Duration> for CalDate {
    type Output = CalDateTime;

    fn add(self, duration: Duration) -> Self::Output {
        (self
            .0
            .and_time(NaiveTime::default())
            .and_local_timezone(self.1)
            .earliest()
            .expect("Local timezone has constant offset")
            + duration)
            .into()
    }
}

impl CalDate {
    pub fn parse_prop(
        prop: &ContentLine,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Self, CalDateTimeError> {
        let prop_value = prop
            .value
            .as_ref()
            .ok_or_else(|| CalDateTimeError::InvalidDatetimeFormat("empty property".into()))?;

        let timezone = if let Some(tzid) = prop.params.get_tzid() {
            if let Some(timezone) = timezones.and_then(|timezones| timezones.get(tzid)) {
                timezone.to_owned()
            } else {
                // TZID refers to timezone that does not exist
                return Err(CalDateTimeError::InvalidTZID(tzid.to_string()));
            }
        } else {
            // No explicit timezone specified.
            // This is valid and will be localtime or UTC depending on the value
            // We will stick to this default as documented in https://github.com/lennart-k/rustical/issues/102
            None
        };

        Self::parse(prop_value, timezone)
    }

    #[must_use]
    pub fn naive_date(&self) -> &NaiveDate {
        &self.0
    }

    #[must_use]
    pub fn format(&self) -> String {
        self.0.format(LOCAL_DATE).to_string()
    }

    #[must_use]
    pub fn as_datetime(&self) -> DateTime<Timezone> {
        self.0
            .and_time(NaiveTime::default())
            .and_local_timezone(self.1.to_owned())
            .earliest()
            .expect("Midnight always exists")
    }

    pub fn parse(value: &str, timezone: Option<Tz>) -> Result<Self, CalDateTimeError> {
        let timezone = timezone.map_or(Timezone::Local, Timezone::Olson);
        if let Ok(date) = NaiveDate::parse_from_str(value, LOCAL_DATE) {
            return Ok(Self(date, timezone));
        }
        Err(CalDateTimeError::InvalidDatetimeFormat(value.to_string()))
    }

    #[must_use]
    pub fn timezone(&self) -> &Timezone {
        &self.1
    }

    #[must_use]
    pub fn succ_opt(&self) -> Option<Self> {
        Some(Self(self.0.succ_opt()?, self.1.clone()))
    }
}

#[cfg(not(tarpaulin_include))]
impl Datelike for CalDate {
    fn year(&self) -> i32 {
        self.0.year()
    }
    fn month(&self) -> u32 {
        self.0.month()
    }

    fn month0(&self) -> u32 {
        self.0.month0()
    }
    fn day(&self) -> u32 {
        self.0.day()
    }
    fn day0(&self) -> u32 {
        self.0.day0()
    }
    fn ordinal(&self) -> u32 {
        self.0.ordinal()
    }
    fn ordinal0(&self) -> u32 {
        self.0.ordinal0()
    }
    fn weekday(&self) -> chrono::Weekday {
        self.0.weekday()
    }
    fn iso_week(&self) -> chrono::IsoWeek {
        self.0.iso_week()
    }
    fn with_year(&self, year: i32) -> Option<Self> {
        Some(Self(self.0.with_year(year)?, self.1.to_owned()))
    }
    fn with_month(&self, month: u32) -> Option<Self> {
        Some(Self(self.0.with_month(month)?, self.1.to_owned()))
    }
    fn with_month0(&self, month0: u32) -> Option<Self> {
        Some(Self(self.0.with_month0(month0)?, self.1.to_owned()))
    }
    fn with_day(&self, day: u32) -> Option<Self> {
        Some(Self(self.0.with_day(day)?, self.1.to_owned()))
    }
    fn with_day0(&self, day0: u32) -> Option<Self> {
        Some(Self(self.0.with_day0(day0)?, self.1.to_owned()))
    }
    fn with_ordinal(&self, ordinal: u32) -> Option<Self> {
        Some(Self(self.0.with_ordinal(ordinal)?, self.1.to_owned()))
    }
    fn with_ordinal0(&self, ordinal0: u32) -> Option<Self> {
        Some(Self(self.0.with_ordinal0(ordinal0)?, self.1.to_owned()))
    }
}

impl Value for CalDate {
    fn value_type(&self) -> Option<&'static str> {
        Some("DATE")
    }
    fn value(&self) -> String {
        self.format()
    }

    fn utc_or_local(self) -> Self {
        let tz = if self.1.is_local() {
            Timezone::Local
        } else {
            Timezone::utc()
        };
        Self(self.0, tz)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{CalDate, Value};
    use chrono::Duration;

    #[test]
    fn test_date() {
        let a = CalDate::parse("20121212", None).unwrap();
        let b = CalDate::parse("20121213", None).unwrap();
        let c = CalDate::parse("20121213", Some(chrono_tz::Europe::Berlin)).unwrap();
        let d = CalDate::parse("20121213", Some(chrono_tz::Europe::Kyiv)).unwrap();
        // Floating time and fixed time => different results
        assert_ne!(b.clone().utc_or_local(), c.clone().utc_or_local());
        // fixed timezones resolve to UTC
        assert_eq!(c.clone().utc_or_local(), d.utc_or_local());
        assert_eq!((a.clone() + Duration::days(1)).0, b.as_datetime());
        assert!(b > a);
        assert!(b >= a);
    }
}
