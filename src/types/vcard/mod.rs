use crate::{
    parser::{ParseProp, ParserError},
    types::Value,
};
use chrono::{Datelike, NaiveDate};
use std::sync::OnceLock;

static RE_DATE: OnceLock<[regex::Regex; 4]> = OnceLock::new();
static RE_TIME: OnceLock<[regex::Regex; 2]> = OnceLock::new();

#[inline]
fn re_date() -> &'static [regex::Regex] {
    RE_DATE.get_or_init(|| {
        [
            // Reduced precision basic notation
            regex::Regex::new(r"^(?<year>\d{4})(((?<month>\d{2})(?<day>\d{2}))?)?$").unwrap(),
            // Reduced precision notation notation
            regex::Regex::new(r"^(?<year>\d{4})(((?:-(?<month>\d{2}))(?:-(?<day>\d{2}))?)?)?$")
                .unwrap(),
            // Truncated basic notation
            regex::Regex::new(r"^-(?:(?<year>\d{4})|-)((?:(?<month>\d{2})|-)(?<day>\d{2})?)?$")
                .unwrap(),
            // Truncated extended notation
            regex::Regex::new(r"^(?:(?<year>\d{4})|-)-(?<month>\d{2})-(?<day>\d{2})$").unwrap(),
        ]
    })
}

#[inline]
fn re_time() -> &'static [regex::Regex] {
    RE_TIME.get_or_init(|| {
        [
        regex::Regex::new(
            r"^(?:(?<hour>\d{2})|-)((?:(?<minute>\d{2})|-)(?<second>\d{2})?)?(?:(?<utc>Z)|(?:(?<offsign>[-+])(?<offhour>\d{2})(?<offminute>\d{2})?))?$",
        )
        .unwrap(),
        regex::Regex::new(
                r"^(?:(?<hour>\d{2})|-)(:(?:(?<minute>\d{2})|-)(?::(?<second>\d{2}))?)?(?:(?<utc>Z)|(?:(?<offsign>[-+])(?<offhour>\d{2})(?::(?<offminute>\d{2}))?))?$"
        )
        .unwrap(),
        ]
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PartialDate {
    year: Option<i32>,
    month: Option<u32>,
    day: Option<u32>,
}

/// A unified type meant to encapsulate TIME, DATE-TIME, and DATE-AND-OR-TIME from RFC 6350
/// Allowed combinations:
/// hour:minute:second
/// hour:minute
/// hour
/// minute:second
/// minute
/// second
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PartialTime {
    hour: Option<u8>,
    minute: Option<u8>,
    second: Option<u8>,
    offset_hour: Option<i8>,
    offset_minute: Option<i8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PartialDateTime {
    date: PartialDate,
    time: PartialTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PartialDateAndOrTime {
    date: Option<PartialDate>,
    time: Option<PartialTime>,
}

impl From<NaiveDate> for PartialDate {
    fn from(value: NaiveDate) -> Self {
        Self {
            year: Some(value.year()),
            month: Some(value.month()),
            day: Some(value.day()),
        }
    }
}

impl PartialDate {
    pub fn parse(value: &str) -> Result<Self, ParserError> {
        // A calendar date as specified in [ISO.8601.2004], Section 4.1.2.
        // https://dotat.at/tmp/ISO_8601-2004_E.pdf
        // Basic format: YYYYMMDD Example: 19850412
        // Extended format: YYYY-MM-DD Example: 1985-04-12
        // Reduced accuracy, as specified in [ISO.8601.2004], Sections 4.1.2.3
        // a) and b), but not c), is permitted.
        //
        // Expanded representation, as specified in [ISO.8601.2004], Section
        // 4.1.4, is forbidden.
        //
        // Truncated representation, as specified in [ISO.8601.2000], Sections
        // 5.2.1.3 d), e), and f), is permitted.
        // d) A specific day of a month in the implied year
        // Basic format: --MMDD EXAMPLE --0412
        // Extended format: --MM-DD EXAMPLE --04-12
        // e) A specific month in the implied year
        // Basic format: --MM EXAMPLE --04
        // Extended format: not applicable
        // f) A specific day in the implied month
        // Basic format: ---DD EXAMPLE ---12
        // Extended format: not applicable
        if let Some(captures) = re_date().iter().find_map(|pattern| pattern.captures(value)) {
            let year = captures.name("year").map(|y| y.as_str().parse().unwrap());
            let month = captures.name("month").map(|m| m.as_str().parse().unwrap());
            let day = captures.name("day").map(|d| d.as_str().parse().unwrap());
            if let Some(month) = month
                && month > 12
            {
                return Err(ParserError::InvalidPropertyValue(value.to_owned()));
            }
            if let Some(day) = day
                && day > 31
            {
                return Err(ParserError::InvalidPropertyValue(value.to_owned()));
            }
            return Ok(Self { year, month, day });
        }
        Err(ParserError::InvalidPropertyValue(value.to_owned()))
    }
}

impl PartialTime {
    pub fn parse(value: &str) -> Result<Self, ParserError> {
        // A time of day as specified in [ISO.8601.2004], Section 4.2.
        //
        // Reduced accuracy, as specified in [ISO.8601.2004], Section 4.2.2.3,
        // is permitted.
        //
        // Representation with decimal fraction, as specified in
        // [ISO.8601.2004], Section 4.2.2.4, is forbidden.
        //
        // The midnight hour is always represented by 00, never 24 (see
        // [ISO.8601.2004], Section 4.2.3).
        //
        // Truncated representation, as specified in [ISO.8601.2000], Sections
        // 5.3.1.4 a), b), and c), is permitted.
        if let Some(captures) = re_time().iter().find_map(|pattern| pattern.captures(value)) {
            let (offset_hour, offset_minute) = if captures.name("utc").is_some() {
                (Some(0), Some(0))
            } else {
                (
                    captures.name("offhour").map(|s| {
                        let sign =
                            matches!(captures.name("offsign").map(|s| s.as_str()), Some("-"))
                                .then_some(-1)
                                .unwrap_or(1);
                        sign * s.as_str().parse::<i8>().unwrap()
                    }),
                    captures.name("offminute").map(|s| {
                        let sign =
                            matches!(captures.name("offsign").map(|s| s.as_str()), Some("-"))
                                .then_some(-1)
                                .unwrap_or(1);
                        sign * s.as_str().parse::<i8>().unwrap()
                    }),
                )
            };
            if let Some(offset_hour) = offset_hour
                && !(-12..=14).contains(&offset_hour)
            {
                return Err(ParserError::InvalidPropertyValue(value.to_owned()));
            }
            if let Some(offset_minute) = offset_minute
                && offset_minute.abs() > 59
            {
                return Err(ParserError::InvalidPropertyValue(value.to_owned()));
            }
            let hour = captures.name("hour").map(|h| h.as_str().parse().unwrap());
            let minute = captures.name("minute").map(|m| m.as_str().parse().unwrap());
            let second = captures.name("second").map(|s| s.as_str().parse().unwrap());
            if let Some(hour) = hour
                && hour > 23
            {
                return Err(ParserError::InvalidPropertyValue(value.to_owned()));
            }
            if let Some(minute) = minute
                && minute > 59
            {
                return Err(ParserError::InvalidPropertyValue(value.to_owned()));
            }
            if let Some(second) = second
                && second > 59
            {
                return Err(ParserError::InvalidPropertyValue(value.to_owned()));
            }
            if hour.is_some() && minute.is_none() && second.is_some() {
                return Err(ParserError::InvalidPropertyValue(value.to_owned()));
            }
            return Ok(Self {
                hour,
                minute,
                second,
                offset_hour,
                offset_minute,
            });
        }

        Err(ParserError::InvalidPropertyValue(value.to_owned()))
    }
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
        _timezones: &std::collections::HashMap<String, Option<chrono_tz::Tz>>,
        _default_type: &str,
    ) -> Result<Self, ParserError> {
        Self::parse(prop.value.as_deref().unwrap_or_default())
    }
}

impl Value for PartialDate {
    fn value_type(&self) -> Option<&'static str> {
        Some("DATE")
    }

    fn value(&self) -> String {
        if let Some(year) = &self.year {
            assert!(!(self.month.is_none() && self.day.is_some()));
            let month = self
                .month
                .map(|month| format!("-{month:02}"))
                .unwrap_or_default();
            let day = self.day.map(|day| format!("-{day:02}")).unwrap_or_default();
            format!("{year:04}{month}{day}")
        } else {
            let month = self
                .month
                .map(|month| format!("{month:02}"))
                .unwrap_or("-".to_owned());
            let day = self.day.map(|day| format!("{day:02}")).unwrap_or_default();
            format!("--{month}{day}")
        }
    }
}

impl Value for PartialTime {
    fn value_type(&self) -> Option<&'static str> {
        Some("TIME")
    }

    fn value(&self) -> String {
        let tz_suffix = if let (Some(0), Some(0)) = (self.offset_hour, self.offset_minute) {
            "Z".to_owned()
        } else if self.offset_hour.is_some() || self.offset_minute.is_some() {
            // Must have same sign
            assert!(
                self.offset_hour.unwrap_or_default() * self.offset_minute.unwrap_or_default() >= 0
            );
            let off_hour = self.offset_hour.unwrap_or_default();
            let off_minute = self
                .offset_minute
                .map(|min| format!("{min:02}", min = min.abs()))
                .unwrap_or_default();
            let sign = if off_hour >= 0 { "+" } else { "-" };
            format!("{sign}{hour:02}{off_minute}", hour = off_hour.abs())
        } else {
            String::new()
        };
        if let Some(hour) = self.hour {
            assert!(!(self.minute.is_none() && self.second.is_some()));
            let minute = self
                .minute
                .map(|minute| format!("{minute:02}"))
                .unwrap_or_default();
            let second = self
                .second
                .map(|second| format!("{second:02}"))
                .unwrap_or_default();
            format!("{hour:02}{minute}{second}{tz_suffix}")
        } else {
            let minute = self
                .minute
                .map(|minute| format!("{minute:02}"))
                .unwrap_or("-".to_owned());
            let second = self
                .second
                .map(|second| format!("{second:02}"))
                .unwrap_or_default();
            format!("-{minute}{second}{tz_suffix}")
        }
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

impl ParseProp for PartialDate {
    fn parse_prop(
        prop: &crate::property::ContentLine,
        _timezones: &std::collections::HashMap<String, Option<chrono_tz::Tz>>,
        _default_type: &str,
    ) -> Result<Self, ParserError> {
        Self::parse(prop.value.as_deref().unwrap_or_default())
    }
}

impl ParseProp for PartialTime {
    fn parse_prop(
        prop: &crate::property::ContentLine,
        _timezones: &std::collections::HashMap<String, Option<chrono_tz::Tz>>,
        _default_type: &str,
    ) -> Result<Self, ParserError> {
        Self::parse(prop.value.as_deref().unwrap_or_default())
    }
}

impl ParseProp for PartialDateTime {
    fn parse_prop(
        prop: &crate::property::ContentLine,
        _timezones: &std::collections::HashMap<String, Option<chrono_tz::Tz>>,
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
    // DATE
    #[case("19850412", PartialDate{year: Some(1985), month: Some(4), day: Some(12)})]
    #[case("1985-04-12", PartialDate{year: Some(1985), month: Some(4), day: Some(12)})]
    #[case("1985-04", PartialDate{year: Some(1985), month: Some(4), ..Default::default()})]
    #[case("1985", PartialDate{year: Some(1985), ..Default::default()})]
    #[case("--0412", PartialDate{month: Some(4), day: Some(12), ..Default::default()})]
    #[case("--04-12", PartialDate{month: Some(4), day: Some(12), ..Default::default()})]
    #[case("--04", PartialDate{month: Some(4), ..Default::default()})]
    #[case("---12", PartialDate{day: Some(12), ..Default::default()})]
    fn test_parse_date(#[case] input: &str, #[case] value: PartialDate) {
        let parsed = PartialDate::parse(input).unwrap();
        assert!(!parsed.value().ends_with('-'));
        let roundtrip = PartialDate::parse(&parsed.value()).unwrap();
        assert_eq!(parsed, value);
        assert_eq!(roundtrip, value);
    }

    #[rstest]
    // TIME
    #[case("19850432")]
    #[case("19851422")]
    #[case("198514222")]
    fn test_parse_date_invalid(#[case] input: &str) {
        assert!(PartialDate::parse(input).is_err());
    }

    #[rstest]
    // TIME
    #[case("102200", PartialTime {hour: Some(10), minute: Some(22), second: Some(0), ..Default::default()})]
    #[case("1022", PartialTime {hour: Some(10), minute: Some(22), ..Default::default()})]
    #[case("10", PartialTime {hour: Some(10), ..Default::default()})]
    #[case("-2200", PartialTime {minute: Some(22), second: Some(0), ..Default::default()})]
    #[case("--00", PartialTime {second: Some(0), ..Default::default()})]
    #[case("102200Z", PartialTime {hour: Some(10), minute: Some(22), second: Some(0), offset_hour: Some(0), offset_minute: Some(0)})]
    #[case("102200-08", PartialTime {hour: Some(10), minute: Some(22), second: Some(0), offset_hour: Some(-8), ..Default::default()})]
    #[case("102200-0800", PartialTime {hour: Some(10), minute: Some(22), second: Some(0), offset_hour: Some(-8), offset_minute: Some(0)})]
    #[case("10:22:00", PartialTime {hour: Some(10), minute: Some(22), second: Some(0), ..Default::default()})]
    #[case("10:22", PartialTime {hour: Some(10), minute: Some(22), ..Default::default()})]
    #[case("-:-:00", PartialTime {second: Some(0), ..Default::default()})]
    #[case("152746+0100", PartialTime {hour: Some(15), minute: Some(27), second: Some(46), offset_hour: Some(1), offset_minute: Some(0)})]
    #[case("15:27:46+01", PartialTime {hour: Some(15), minute: Some(27), second: Some(46), offset_hour: Some(1), ..Default::default()})]
    #[case("15:27:46-05:00", PartialTime {hour: Some(15), minute: Some(27), second: Some(46), offset_hour: Some(-5), offset_minute: Some(0)})]
    fn test_parse_time(#[case] input: &str, #[case] value: PartialTime) {
        let parsed = PartialTime::parse(input).unwrap();
        assert!(!parsed.value().ends_with('-'));
        let roundtrip = PartialTime::parse(&parsed.value()).unwrap();
        assert_eq!(parsed, value);
        assert_eq!(roundtrip, value);
    }

    #[rstest]
    // TIME
    #[case("10-00")]
    #[case("250000")]
    #[case("236000")]
    #[case("235060")]
    #[case("100000+0070")]
    #[case("100000-4000")]
    fn test_parse_time_invalid(#[case] input: &str) {
        assert!(PartialTime::parse(input).is_err());
    }

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
