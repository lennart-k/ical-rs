use std::{
    collections::HashMap,
    ops::{Add, Sub},
};

use chrono::{DateTime, Duration, NaiveDate, NaiveTime, Utc};

use crate::{
    property::Property,
    types::{CalDate, CalDateTime, CalDateTimeError, Timezone},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalDateOrDateTime {
    DateTime(CalDateTime),
    Date(CalDate),
}

impl CalDateOrDateTime {
    pub fn parse_prop(
        prop: &Property,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Self, CalDateTimeError> {
        Ok(match prop.get_value_type() {
            Some("DATE") => Self::Date(CalDate::parse_prop(prop, timezones)?),
            Some("DATE-TIME") | None => Self::DateTime(CalDateTime::parse_prop(prop, timezones)?),
            _ => {
                panic!()
            }
        })
    }

    pub fn is_date(&self) -> bool {
        matches!(self, Self::Date(_))
    }

    pub fn date_floor(&self) -> NaiveDate {
        match self {
            Self::DateTime(datetime) => datetime.date_floor(),
            Self::Date(date) => date.naive_date().to_owned(),
        }
    }

    pub fn date_ceil(&self) -> NaiveDate {
        match self {
            Self::DateTime(datetime) => datetime.date_ceil(),
            Self::Date(date) => date.naive_date().to_owned(),
        }
    }

    pub fn timezone(&self) -> Timezone {
        match self {
            Self::DateTime(datetime) => datetime.timezone(),
            Self::Date(date) => date.timezone().clone(),
        }
    }

    pub fn utc(&self) -> DateTime<Utc> {
        match self {
            Self::DateTime(datetime) => datetime.utc(),
            Self::Date(date) => date.naive_date().and_time(NaiveTime::default()).and_utc(),
        }
    }
}

impl Sub<&CalDateOrDateTime> for CalDateOrDateTime {
    type Output = Duration;

    fn sub(self, rhs: &CalDateOrDateTime) -> Self::Output {
        self.utc() - rhs.utc()
    }
}

impl From<CalDateTime> for CalDateOrDateTime {
    fn from(value: CalDateTime) -> Self {
        Self::DateTime(value)
    }
}

impl From<CalDateOrDateTime> for CalDateTime {
    fn from(value: CalDateOrDateTime) -> Self {
        match value {
            CalDateOrDateTime::DateTime(datetime) => datetime,
            CalDateOrDateTime::Date(date) => date.as_datetime().into(),
        }
    }
}

impl From<CalDateOrDateTime> for DateTime<rrule::Tz> {
    fn from(value: CalDateOrDateTime) -> Self {
        match value {
            CalDateOrDateTime::DateTime(datetime) => datetime.into(),
            CalDateOrDateTime::Date(date) => date
                .as_datetime()
                .with_timezone(&date.timezone().to_owned().into()),
        }
    }
}

impl Add<Duration> for CalDateOrDateTime {
    type Output = CalDateTime;

    fn add(self, duration: Duration) -> Self::Output {
        CalDateTime::from(self) + duration
    }
}
