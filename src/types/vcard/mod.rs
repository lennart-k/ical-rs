use std::collections::HashMap;

use crate::{
    parser::{ParseProp, ParserError},
    types::Value,
};

mod partial_date;
pub use partial_date::*;
mod partial_time;
pub use partial_time::*;
mod significant_date;
pub use significant_date::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PartialDateTime {
    pub date: PartialDate,
    pub time: PartialTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PartialDateAndOrTime {
    pub date: Option<PartialDate>,
    pub time: Option<PartialTime>,
}

impl PartialDateTime {
    pub fn parse(value: &str) -> Result<Self, ParserError> {
        let (date, time) = value.split_once('T').unwrap();
        let date = PartialDate::parse(date)?;
        let time = PartialTime::parse(time)?;
        Ok(Self { date, time })
    }
}

impl PartialDateAndOrTime {
    pub fn parse(value: &str) -> Result<Self, ParserError> {
        let Some((date, time)) = value.split_once('T') else {
            return Ok(Self {
                date: Some(PartialDate::parse(value)?),
                time: None,
            });
        };
        let date = if !date.is_empty() {
            Some(PartialDate::parse(date)?)
        } else {
            None
        };
        let time = if !time.is_empty() {
            Some(PartialTime::parse(time)?)
        } else {
            None
        };
        Ok(Self { date, time })
    }
}

impl ParseProp for PartialDateAndOrTime {
    fn parse_prop(
        prop: &crate::property::ContentLine,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
        _default_type: &str,
    ) -> Result<Self, ParserError> {
        Self::parse(prop.value.as_deref().unwrap_or_default())
    }
}
impl Value for PartialDateAndOrTime {
    fn value_type(&self) -> Option<&'static str> {
        Some("DATE-AND-OR-TIME")
    }

    fn value(&self) -> String {
        format!(
            "{}T{}",
            self.date.as_ref().map(Value::value).unwrap_or_default(),
            self.time.as_ref().map(Value::value).unwrap_or_default(),
        )
    }
}

impl Value for PartialDateTime {
    fn value_type(&self) -> Option<&'static str> {
        Some("DATE-TIME")
    }

    fn value(&self) -> String {
        format!("{}T{}", self.date.value(), self.time.value())
    }
}

impl ParseProp for PartialDateTime {
    fn parse_prop(
        prop: &crate::property::ContentLine,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
        _default_type: &str,
    ) -> Result<Self, ParserError> {
        Self::parse(prop.value.as_deref().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{PartialDate, PartialDateAndOrTime, PartialDateTime, PartialTime, Value};
    use rstest::rstest;

    #[rstest]
    // DATE-TIME
    #[case("19961022T140000", PartialDateTime {date: PartialDate {year: Some(1996), month: Some(10), day: Some(22)}, time: PartialTime {hour: Some(14), minute: Some(0), second: Some(0), ..Default::default()}})]
    #[case("--1022T1400", PartialDateTime {date: PartialDate {month: Some(10), day: Some(22), ..Default::default()}, time: PartialTime {hour: Some(14), minute: Some(0), ..Default::default()}})]
    #[case("---22T14", PartialDateTime {date: PartialDate {day: Some(22), ..Default::default()}, time: PartialTime {hour: Some(14), ..Default::default()}})]
    fn test_parse_datetime(#[case] input: &str, #[case] value: PartialDateTime) {
        let parsed = PartialDateTime::parse(input).unwrap();
        assert_eq!(parsed, value);
        let roundtrip = PartialDateTime::parse(&parsed.value()).unwrap();
        assert_eq!(parsed, value);
        assert_eq!(roundtrip, value);
    }

    #[rstest]
    // DATE-AND-OR-TIME
    #[case("19850412", PartialDateAndOrTime {date: Some(PartialDate {year: Some(1985), month: Some(4), day: Some(12)}),time: None})]
    #[case("1985-04", PartialDateAndOrTime {date: Some(PartialDate {year: Some(1985), month: Some(4), ..Default::default()}), time: None})]
    #[case("1985", PartialDateAndOrTime {date: Some(PartialDate {year: Some(1985), ..Default::default()}), time: None})]
    #[case("--0412", PartialDateAndOrTime {date: Some(PartialDate {month: Some(4), day: Some(12), ..Default::default()}), time: None})]
    #[case("---12", PartialDateAndOrTime {date: Some(PartialDate {day: Some(12), ..Default::default()}), time: None})]
    #[case("---12T102200", PartialDateAndOrTime {date: Some(PartialDate {day: Some(12), ..Default::default()}), time: Some(PartialTime {hour: Some(10), minute: Some(22), second: Some(0), ..Default::default()})})]
    #[case("T102200", PartialDateAndOrTime {time: Some(PartialTime {hour: Some(10), minute: Some(22), second: Some(0), ..Default::default()}), date: None})]
    #[case("T1022", PartialDateAndOrTime {time: Some(PartialTime {hour: Some(10), minute: Some(22), ..Default::default()}), date: None})]
    #[case("T10", PartialDateAndOrTime {time: Some(PartialTime {hour: Some(10), ..Default::default()}), date: None})]
    #[case("T-2200", PartialDateAndOrTime {time: Some(PartialTime {minute: Some(22), second: Some(0), ..Default::default()}), date: None})]
    #[case("T--00", PartialDateAndOrTime {time: Some(PartialTime {second: Some(0), ..Default::default()}), date: None})]
    #[case("T102200Z", PartialDateAndOrTime {time: Some(PartialTime {hour: Some(10), minute: Some(22), second: Some(0), offset_hour: Some(0), offset_minute: Some(0)}), date: None})]
    #[case("T102200-0800", PartialDateAndOrTime {time: Some(PartialTime {hour: Some(10), minute: Some(22), second: Some(0), offset_hour: Some(-8), offset_minute: Some(0)}), date: None})]
    fn test_parse_date_and_or_time(#[case] input: &str, #[case] value: PartialDateAndOrTime) {
        let parsed = PartialDateAndOrTime::parse(input).unwrap();
        assert_eq!(parsed, value);
        let roundtrip = PartialDateAndOrTime::parse(&parsed.value()).unwrap();
        assert_eq!(parsed, value);
        assert_eq!(roundtrip, value);
    }
}
