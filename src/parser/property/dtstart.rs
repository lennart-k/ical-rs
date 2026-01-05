use crate::types::CalDateOrDateTime;

super::property!(
    "DTSTART",
    "DATE-TIME",
    IcalDTSTARTProperty,
    CalDateOrDateTime
);

#[cfg(test)]
mod tests {
    use super::IcalDTSTARTProperty;
    use crate::{generator::Emitter, parser::ICalProperty, property::ContentLine};
    use rstest::rstest;
    use std::collections::HashMap;

    #[rstest]
    #[case("DTSTART:19980118T073000Z\r\n")]
    fn roundtrip(#[case] input: &str) {
        let content_line = crate::PropertyParser::from_reader(input.as_bytes())
            .next()
            .unwrap()
            .unwrap();
        let timezones = HashMap::new();
        let prop = IcalDTSTARTProperty::parse_prop(&content_line, &timezones).unwrap();
        let roundtrip: ContentLine = prop.into();
        similar_asserts::assert_eq!(roundtrip.generate(), input);
    }
}
