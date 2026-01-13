use crate::types::CalDateOrDateTime;

super::property!("DTEND", "DATE-TIME", IcalDTENDProperty, CalDateOrDateTime);

#[cfg(test)]
mod tests {
    use super::IcalDTENDProperty;
    use crate::{generator::Emitter, parser::ICalProperty, property::ContentLine};
    use rstest::rstest;
    use std::collections::HashMap;

    #[rstest]
    #[case("DTEND:19980118T073000Z\r\n")]
    #[case("DTEND;TZID=Europe/Berlin:19980118T073000Z\r\n")]
    #[case("DTEND;TZID=W. Europe Standard Time:20210527T120000\r\n")]
    fn roundtrip(#[case] input: &str) {
        let content_line = crate::PropertyParser::from_slice(input.as_bytes())
            .next()
            .unwrap()
            .unwrap();
        let mut timezones = HashMap::new();
        timezones.insert("Europe/Berlin".to_owned(), Some(chrono_tz::Europe::Berlin));
        timezones.insert("W. Europe Standard Time".to_owned(), None);
        let prop = IcalDTENDProperty::parse_prop(&content_line, Some(&timezones)).unwrap();
        let roundtrip: ContentLine = prop.into();
        similar_asserts::assert_eq!(roundtrip.generate(), input);
    }
}
