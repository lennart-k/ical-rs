use chrono::Duration;
use chrono_tz::Tz;
use std::collections::HashMap;

use crate::{
    property::ContentLine,
    types::{CalDateOrDateTime, CalDateTime, CalDateTimeError, Value, parse_duration},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateTimeOrDuration {
    DateTime(CalDateTime),
    Duration(Duration),
}

impl DateTimeOrDuration {
    pub fn parse(value: &str, timezone: Option<Tz>) -> Result<Self, CalDateTimeError> {
        if let Ok(datetime) = CalDateTime::parse(value, timezone) {
            return Ok(Self::DateTime(datetime));
        }
        Ok(Self::Duration(parse_duration(value).unwrap()))
    }
}

impl Value for DateTimeOrDuration {
    fn utc_or_local(self) -> Self {
        match self {
            Self::DateTime(datetime) => Self::DateTime(datetime.utc_or_local()),
            Self::Duration(duration) => Self::Duration(duration),
        }
    }

    fn value_type(&self) -> Option<&'static str> {
        match self {
            Self::DateTime(dt) => dt.value_type(),
            Self::Duration(dur) => dur.value_type(),
        }
    }

    fn value(&self) -> String {
        match self {
            Self::DateTime(dt) => dt.value(),
            Self::Duration(dur) => dur.value(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Period(CalDateTime, DateTimeOrDuration);

impl Period {
    pub fn parse_prop(
        prop: &ContentLine,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Self, CalDateTimeError> {
        let prop_value = prop
            .value
            .as_ref()
            .ok_or_else(|| CalDateTimeError::InvalidDatetimeFormat("empty property".into()))?;

        let timezone = if let Some(tzid) = prop.params.get_tzid() {
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

    pub fn parse(value: &str, timezone: Option<Tz>) -> Result<Self, CalDateTimeError> {
        let (start, end) = value
            .split_once('/')
            .ok_or_else(|| CalDateTimeError::InvalidPeriodFormat(value.to_owned()))?;

        let start = CalDateTime::parse(start, timezone)?;
        let end = DateTimeOrDuration::parse(end, timezone)?;
        Ok(Self(start, end))
    }

    pub fn utc_or_local(self) -> Self {
        Self(self.0.utc_or_local(), self.1.utc_or_local())
    }
}

impl Value for Period {
    fn value_type(&self) -> Option<&'static str> {
        Some("PERIOD")
    }

    fn value(&self) -> String {
        format!(
            "{start}/{end}",
            start = self.0.value(),
            end = self.1.value()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateOrDateTimeOrPeriod {
    DateOrDateTime(CalDateOrDateTime),
    Period(Period),
}

impl DateOrDateTimeOrPeriod {
    pub fn parse_prop(
        prop: &ContentLine,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
        default_type: &str,
    ) -> Result<Self, CalDateTimeError> {
        let value_type = prop.params.get_value_type().unwrap_or(default_type);
        match value_type {
            "DATE" | "DATE-TIME" => Ok(Self::DateOrDateTime(CalDateOrDateTime::parse_prop(
                prop, timezones, value_type,
            )?)),
            "PERIOD" => Ok(Self::Period(Period::parse_prop(prop, timezones)?)),
            _ => panic!(),
        }
    }

    pub fn start(&self) -> CalDateOrDateTime {
        match self {
            Self::DateOrDateTime(dodt) => dodt.clone(),
            Self::Period(Period(start, _)) => start.clone().into(),
        }
    }

    pub fn utc_or_local(self) -> Self {
        match self {
            Self::DateOrDateTime(dodt) => Self::DateOrDateTime(dodt.utc_or_local()),
            Self::Period(period) => Self::Period(period.utc_or_local()),
        }
    }
}

impl Value for DateOrDateTimeOrPeriod {
    fn value_type(&self) -> Option<&'static str> {
        match self {
            Self::DateOrDateTime(dodt) => Value::value_type(dodt),
            Self::Period(period) => period.value_type(),
        }
    }

    fn value(&self) -> String {
        match self {
            Self::DateOrDateTime(dodt) => dodt.value(),
            Self::Period(period) => period.value(),
        }
    }
}
