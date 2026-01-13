use crate::types::DateOrDateTimeOrPeriod;

super::property!(
    "RDATE",
    "DATE-TIME",
    IcalRDATEProperty,
    Vec<DateOrDateTimeOrPeriod>
);

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
        let content_line = crate::PropertyParser::from_slice(input.as_bytes())
            .next()
            .unwrap()
            .unwrap();
        let mut timezones = HashMap::new();
        timezones.insert("America/New_York".to_owned(), None);
        let prop = IcalRDATEProperty::parse_prop(&content_line, Some(&timezones)).unwrap();
        let roundtrip: ContentLine = prop.into();
        let roundtrip = roundtrip.generate();
        similar_asserts::assert_eq!(
            roundtrip.trim().trim_end_matches(','),
            input.trim().trim_end_matches(',')
        );
    }
}
