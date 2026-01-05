use crate::types::CalDateOrDateTime;

super::property!(
    "DTSTART",
    "DATE-TIME",
    IcalDTSTARTProperty,
    CalDateOrDateTime
);

impl IcalDTSTARTProperty {
    pub fn utc_or_local(self) -> Self {
        let Self(dt, mut params) = self;
        params.remove("TZID");
        Self(dt.utc_or_local(), params)
    }
}

#[cfg(test)]
mod tests {
    use super::IcalDTSTARTProperty;
    use crate::{generator::Emitter, parser::ICalProperty, property::ContentLine};
    use rstest::rstest;
    use std::collections::HashMap;

    #[rstest]
    #[case("DTSTART:19980118T073000Z\r\n")]
    #[case("DTSTART;TZID=Europe/Berlin:19980118T073000Z\r\n")]
    #[case("DTSTART;TZID=W. Europe Standard Time:20210527T120000\r\n")]
    fn roundtrip(#[case] input: &str) {
        let content_line = crate::PropertyParser::from_reader(input.as_bytes())
            .next()
            .unwrap()
            .unwrap();
        let mut timezones = HashMap::new();
        timezones.insert("Europe/Berlin".to_owned(), Some(chrono_tz::Europe::Berlin));
        timezones.insert("W. Europe Standard Time".to_owned(), None);
        let prop = IcalDTSTARTProperty::parse_prop(&content_line, &timezones).unwrap();
        let roundtrip: ContentLine = prop.into();
        similar_asserts::assert_eq!(roundtrip.generate(), input);
    }
}
