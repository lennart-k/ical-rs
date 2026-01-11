use crate::{
    parser::{ParseProp, ParserError},
    types::Value,
};
use chrono::{Datelike, NaiveDate};
use std::sync::OnceLock;

static RE_DATE: OnceLock<[regex::Regex; 4]> = OnceLock::new();

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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PartialDate {
    pub(crate) year: Option<i32>,
    pub(crate) month: Option<u32>,
    pub(crate) day: Option<u32>,
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

impl ParseProp for PartialDate {
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
    use crate::types::{PartialDate, Value};
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
    #[case("19850432")]
    #[case("19851422")]
    #[case("198514222")]
    fn test_parse_date_invalid(#[case] input: &str) {
        assert!(PartialDate::parse(input).is_err());
    }
}
