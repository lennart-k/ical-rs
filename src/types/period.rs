use std::collections::HashMap;

use chrono::{DateTime, Duration};
use chrono_tz::Tz;

use crate::{
    property::Property,
    types::{CalDateOrDateTime, CalDateTime, CalDateTimeError, parse_duration},
};

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

pub struct Period(CalDateTime, DateTimeOrDuration);

impl Period {
    pub fn parse_prop(
        prop: &Property,
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

    pub fn parse(value: &str, timezone: Option<Tz>) -> Result<Self, CalDateTimeError> {
        let (start, end) = value.split_once('/').unwrap();

        let start = CalDateTime::parse(start, timezone)?;
        let end = DateTimeOrDuration::parse(end, timezone)?;
        Ok(Self(start, end))
    }
}

pub enum DateOrDateTimeOrPeriod {
    DateOrDateTime(CalDateOrDateTime),
    Period(Period),
}

impl DateOrDateTimeOrPeriod {
    pub fn parse_prop(
        prop: &Property,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
        default_type: &str,
    ) -> Result<Self, CalDateTimeError> {
        match prop.get_param("VALUE").unwrap_or(default_type) {
            "DATE" | "DATE-TIME" => Ok(Self::DateOrDateTime(CalDateOrDateTime::parse_prop(
                prop, timezones,
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
}
