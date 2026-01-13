use crate::types::CalDateOrDateTime;
super::property!("DUE", "DATE-TIME", IcalDUEProperty, CalDateOrDateTime);

#[cfg(test)]
mod tests {
    use super::IcalDUEProperty;
    use crate::{generator::Emitter, parser::ICalProperty, property::ContentLine};
    use rstest::rstest;

    #[rstest]
    #[case("DUE:19960402T010000Z\r\n")]
    fn roundtrip(#[case] input: &str) {
        let content_line = crate::PropertyParser::from_slice(input.as_bytes())
            .next()
            .unwrap()
            .unwrap();
        let prop = IcalDUEProperty::parse_prop(&content_line, None).unwrap();
        let roundtrip: ContentLine = prop.into();
        similar_asserts::assert_eq!(roundtrip.generate(), input);
    }
}
