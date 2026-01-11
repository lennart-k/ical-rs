use crate::{
    parser::{ParseProp, ParserError},
    types::Value,
};
use std::sync::OnceLock;

static RE_TIME: OnceLock<[regex::Regex; 2]> = OnceLock::new();

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
    pub(crate) hour: Option<u8>,
    pub(crate) minute: Option<u8>,
    pub(crate) second: Option<u8>,
    pub(crate) offset_hour: Option<i8>,
    pub(crate) offset_minute: Option<i8>,
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

impl ParseProp for PartialTime {
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
    use crate::types::{PartialTime, Value};
    use rstest::rstest;

    #[rstest]
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
    #[case("10-00")]
    #[case("250000")]
    #[case("236000")]
    #[case("235060")]
    #[case("100000+0070")]
    #[case("100000-4000")]
    fn test_parse_time_invalid(#[case] input: &str) {
        assert!(PartialTime::parse(input).is_err());
    }
}
