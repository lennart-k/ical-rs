use crate::types::CalDateOrDateTime;
super::property!(
    "EXDATE",
    "DATE-TIME",
    IcalEXDATEProperty,
    Vec<CalDateOrDateTime>
);

#[cfg(test)]
mod tests {
    use super::IcalEXDATEProperty;
    use crate::{generator::Emitter, parser::ICalProperty, property::ContentLine};
    use rstest::rstest;
    use std::collections::HashMap;

    #[rstest]
    #[case("EXDATE:19960402T010000Z,19960403T010000Z,19960404T010000Z\r\n")]
    #[case("EXDATE:19970714T123000Z\r\n")]
    #[case("EXDATE;TZID=America/New_York:19970714T083000\r\n")]
    #[case(
        "EXDATE;VALUE=DATE:19970101,19970120,19970217,19970421,19970526,19970704,199\r\n 70901,19971014,19971128,19971129,19971225\r\n"
    )]
    fn roundtrip(#[case] input: &str) {
        let content_line = crate::PropertyParser::from_slice(input.as_bytes())
            .next()
            .unwrap()
            .unwrap();
        let mut timezones = HashMap::new();
        timezones.insert("America/New_York".to_owned(), None);
        let prop = IcalEXDATEProperty::parse_prop(&content_line, Some(&timezones)).unwrap();
        let roundtrip: ContentLine = prop.into();
        similar_asserts::assert_eq!(roundtrip.generate(), input);
    }
}
