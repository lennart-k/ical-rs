use crate::{
    parser::{ICalProperty, ParseProp},
    property::ContentLine,
    types::DateOrDateTimeOrPeriod,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IcalRDATEProperty(pub Vec<DateOrDateTimeOrPeriod>, pub Option<String>);

impl ICalProperty for IcalRDATEProperty {
    const NAME: &'static str = "RDATE";
    const DEFAULT_TYPE: &'static str = "DATE-TIME";

    fn parse_prop(
        prop: &crate::property::ContentLine,
        timezones: &std::collections::HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Self, crate::parser::ParserError> {
        let dates = ParseProp::parse_prop(prop, timezones, Self::DEFAULT_TYPE)?;
        Ok(Self(dates, prop.get_tzid().map(ToOwned::to_owned)))
    }
}

impl From<IcalRDATEProperty> for ContentLine {
    fn from(prop: IcalRDATEProperty) -> Self {
        let mut params = vec![];
        let value_type =
            crate::types::Value::value_type(&prop.0).unwrap_or(IcalRDATEProperty::DEFAULT_TYPE);
        if value_type != IcalRDATEProperty::DEFAULT_TYPE {
            params.push(("VALUE".to_owned(), vec![value_type.to_owned()]));
        }
        if let Some(tzid) = prop.1 {
            params.push(("TZID".to_owned(), vec![tzid]));
        }
        crate::property::ContentLine {
            name: IcalRDATEProperty::NAME.to_owned(),
            params,
            value: Some(crate::types::Value::value(&prop.0)),
        }
    }
}

impl IcalRDATEProperty {
    pub fn utc_or_local(&self) -> Self {
        Self(self.0.iter().map(|dt| dt.utc_or_local()).collect(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::IcalRDATEProperty;
    use crate::{generator::Emitter, parser::ICalProperty, property::ContentLine};
    use rstest::rstest;
    use std::collections::HashMap;

    #[rstest]
    #[case("RDATE:19970714T123000Z\r\n")]
    #[case("RDATE;TZID=America/New_York:19970714T083000\r\n")]
    #[case("RDATE;VALUE=PERIOD:19960403T020000Z/19960403T040000Z,\r\n")]
    #[case("RDATE;VALUE=PERIOD:19960404T010000Z/PT3H\r\n")]
    #[case(
        "RDATE;VALUE=DATE:19970101,19970120,19970217,19970421,19970526,19970704,1997\r\n 0901,19971014,19971128,19971129,19971225\r\n"
    )]
    fn roundtrip(#[case] input: &str) {
        let content_line = crate::PropertyParser::from_reader(input.as_bytes())
            .next()
            .unwrap()
            .unwrap();
        let mut timezones = HashMap::new();
        timezones.insert("America/New_York".to_owned(), None);
        let prop = IcalRDATEProperty::parse_prop(&content_line, &timezones).unwrap();
        let roundtrip: ContentLine = prop.into();
        let roundtrip = roundtrip.generate();
        similar_asserts::assert_eq!(
            roundtrip.trim().trim_end_matches(','),
            input.trim().trim_end_matches(',')
        );
    }
}
