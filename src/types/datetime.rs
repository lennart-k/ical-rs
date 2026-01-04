use crate::types::Timezone;
use crate::{property::ContentLine, types::CalDateTimeError};
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime, Utc};
use chrono_tz::Tz;
use std::{collections::HashMap, ops::Add};

const LOCAL_DATE_TIME: &str = "%Y%m%dT%H%M%S";
const UTC_DATE_TIME: &str = "%Y%m%dT%H%M%SZ";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
// Form 1, example: 19980118T230000 -> Local
// Form 2, example: 19980119T070000Z -> UTC
// Form 3, example: TZID=America/New_York:19980119T020000 -> Olson
// https://en.wikipedia.org/wiki/Tz_database
pub struct CalDateTime(pub(crate) DateTime<Timezone>);

impl From<CalDateTime> for DateTime<rrule::Tz> {
    fn from(value: CalDateTime) -> Self {
        value.0.with_timezone(&value.timezone().into())
    }
}

impl From<DateTime<rrule::Tz>> for CalDateTime {
    fn from(value: DateTime<rrule::Tz>) -> Self {
        Self(value.with_timezone(&value.timezone().into()))
    }
}

impl From<DateTime<Timezone>> for CalDateTime {
    fn from(value: DateTime<Timezone>) -> Self {
        Self(value)
    }
}

impl From<DateTime<Local>> for CalDateTime {
    fn from(value: DateTime<Local>) -> Self {
        Self(value.with_timezone(&Timezone::Local))
    }
}

impl From<DateTime<Utc>> for CalDateTime {
    fn from(value: DateTime<Utc>) -> Self {
        Self(value.with_timezone(&Timezone::Olson(chrono_tz::UTC)))
    }
}

impl Add<Duration> for CalDateTime {
    type Output = Self;

    fn add(self, duration: Duration) -> Self::Output {
        Self(self.0 + duration)
    }
}

impl CalDateTime {
    pub fn parse_prop(
        prop: &ContentLine,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Self, CalDateTimeError> {
        let prop_value = prop
            .value
            .as_ref()
            .ok_or_else(|| CalDateTimeError::InvalidDatetimeFormat("empty property".into()))?;

        let timezone = if let Some(tzid) = prop.get_param("TZID") {
            if let Some(timezone) = timezones.get(tzid) {
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
    pub fn format(&self) -> String {
        match self.timezone() {
            Timezone::Olson(chrono_tz::UTC) => self.0.format(UTC_DATE_TIME).to_string(),
            _ => self.0.format(LOCAL_DATE_TIME).to_string(),
        }
    }

    pub fn parse(value: &str, timezone: Option<Tz>) -> Result<Self, CalDateTimeError> {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(value, LOCAL_DATE_TIME) {
            if let Some(timezone) = timezone {
                return Ok(Self(
                    datetime
                        .and_local_timezone(timezone.into())
                        .earliest()
                        .ok_or(CalDateTimeError::LocalTimeGap)?,
                ));
            }
            return Ok(Self(
                datetime
                    .and_local_timezone(Timezone::Local)
                    .earliest()
                    .ok_or(CalDateTimeError::LocalTimeGap)?,
            ));
        }

        if let Ok(datetime) = NaiveDateTime::parse_from_str(value, UTC_DATE_TIME) {
            return Ok(datetime.and_utc().into());
        }

        Err(CalDateTimeError::InvalidDatetimeFormat(value.to_string()))
    }

    #[must_use]
    pub fn utc(&self) -> DateTime<Utc> {
        self.0.to_utc()
    }

    #[must_use]
    pub fn timezone(&self) -> Timezone {
        self.0.timezone()
    }

    #[must_use]
    pub fn date_floor(&self) -> NaiveDate {
        self.0.date_naive()
    }
    #[must_use]
    pub fn date_ceil(&self) -> NaiveDate {
        let date = self.0.date_naive();
        date.succ_opt().unwrap_or(date)
    }
}

impl From<CalDateTime> for DateTime<Utc> {
    fn from(value: CalDateTime) -> Self {
        value.utc()
    }
}

impl Datelike for CalDateTime {
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
        Some(Self(self.0.with_year(year)?))
    }
    fn with_month(&self, month: u32) -> Option<Self> {
        Some(Self(self.0.with_month(month)?))
    }
    fn with_month0(&self, month0: u32) -> Option<Self> {
        Some(Self(self.0.with_month0(month0)?))
    }
    fn with_day(&self, day: u32) -> Option<Self> {
        Some(Self(self.0.with_day(day)?))
    }
    fn with_day0(&self, day0: u32) -> Option<Self> {
        Some(Self(self.0.with_day0(day0)?))
    }
    fn with_ordinal(&self, ordinal: u32) -> Option<Self> {
        Some(Self(self.0.with_ordinal(ordinal)?))
    }
    fn with_ordinal0(&self, ordinal0: u32) -> Option<Self> {
        Some(Self(self.0.with_ordinal0(ordinal0)?))
    }
}
