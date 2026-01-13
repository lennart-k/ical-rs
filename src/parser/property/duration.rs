use chrono::Duration;

super::property!("DURATION", "DURATION", IcalDURATIONProperty, Duration);

#[cfg(test)]
mod tests {
    use super::IcalDURATIONProperty;
    use crate::{generator::Emitter, parser::ICalProperty, property::ContentLine};
    use rstest::rstest;

    #[rstest]
    // #[case("DURATION:PT1H0M0S\r\n")]
    #[case("DURATION:PT1H\r\n")]
    #[case("DURATION:PT15M\r\n")]
    fn roundtrip(#[case] input: &str) {
        let content_line = crate::PropertyParser::from_slice(input.as_bytes())
            .next()
            .unwrap()
            .unwrap();
        let prop = IcalDURATIONProperty::parse_prop(&content_line, None).unwrap();
        let roundtrip: ContentLine = prop.into();
        similar_asserts::assert_eq!(roundtrip.generate(), input);
    }
}
