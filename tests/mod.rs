use caldata::LineReader;
use std::borrow::Cow;

macro_rules! set_snapshot_suffix {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_suffix(format!($($expr,)*));
        let _guard = settings.bind_to_scope();
    }
}

// Simple function for sorting properties and components
// to allow for order-invariant comparison of Emitter outputs
pub fn lines_normalise_prop_order<'a>(
    line_iter: &mut impl Iterator<Item = Cow<'a, str>>,
    header: Option<Cow<'a, str>>,
) -> Vec<Cow<'a, str>>
where
{
    let mut props = vec![];
    let mut comps = vec![];
    let mut end = None;
    while let Some(line) = line_iter.next() {
        if line.to_uppercase().starts_with("BEGIN:") {
            comps.push(lines_normalise_prop_order(line_iter, Some(line)));
        } else if line.to_uppercase().starts_with("END:") {
            end = Some(line);
            break;
        } else {
            props.push(line);
        }
    }
    assert_eq!(header.is_some(), end.is_some());
    props.sort();

    [
        header.map(|hdr| vec![hdr]).unwrap_or_default(),
        props,
        comps.into_iter().flatten().collect(),
        end.map(|end| vec![end]).unwrap_or_default(),
    ]
    .concat()
}

pub fn str_normalise_prop_order(input: &str) -> String {
    let mut lines = LineReader::from_slice(input.as_bytes()).map(|line| line.unwrap().inner);
    let sorted = lines_normalise_prop_order(&mut lines, None);
    sorted.join("\r\n") + "\r\n"
}

pub mod sort_lines {
    use crate::{lines_normalise_prop_order, str_normalise_prop_order};
    use caldata::LineReader;
    use itertools::Itertools;
    use rstest::rstest;

    #[test]
    fn test_sort_output_lines() {
        let lines = vec![
            "a",
            "c",
            "b",
            "begin:event",
            "d",
            "a",
            "begin:alarm",
            "g",
            "f",
            "end:alarm",
            "end:event",
            "begin:event",
            "p",
            "a",
            "end:event",
            "d",
        ];
        let input = lines.join("\r\n") + "\r\n";
        let mut lines = LineReader::from_slice(input.as_bytes()).map(|line| line.unwrap().inner);
        let sorted = lines_normalise_prop_order(&mut lines, None);
        assert_eq!(
            sorted.iter().collect_vec(),
            vec![
                "a",
                "b",
                "c",
                "d",
                "begin:event",
                "a",
                "d",
                "begin:alarm",
                "f",
                "g",
                "end:alarm",
                "end:event",
                "begin:event",
                "a",
                "p",
                "end:event",
            ]
        );
    }

    #[rstest]
    #[case(0, include_str!("./resources/ical_events.ics"))]
    #[case(1, include_str!("./resources/vcard_input.vcf"))]
    #[case(2, include_str!("./resources/ical_todos.ics"))]
    #[case(3, include_str!("./resources/ical_journals.ics"))]
    #[case(4, include_str!("./resources/recurring_wholeday.ics"))]
    fn test_sort_props(#[case] case: usize, #[case] input: &str) {
        set_snapshot_suffix!("{case}");
        insta::assert_snapshot!(str_normalise_prop_order(input));
    }
}

pub mod property {
    use caldata::ContentLineParser;

    #[test]
    fn ical() {
        let input = include_str!("./resources/ical_multiple.ics");
        let reader = ContentLineParser::from_slice(input.as_bytes());
        let lines = reader.collect::<Result<Vec<_>, _>>().unwrap();
        insta::assert_debug_snapshot!(lines);
    }

    #[test]
    fn vcard() {
        let input = include_str!("./resources/vcard_input.vcf");
        let reader = ContentLineParser::from_slice(input.as_bytes());
        let lines = reader.collect::<Result<Vec<_>, _>>().unwrap();
        insta::assert_debug_snapshot!(lines);
    }

    #[test]
    fn errors() {
        let input = include_str!("./resources/property_error.vcf");
        let reader = ContentLineParser::from_slice(input.as_bytes());
        for res in reader {
            assert!(res.is_err());
        }
    }
}

pub mod line {
    use caldata::LineReader;

    use insta::assert_snapshot;
    use itertools::Itertools;
    use rstest::rstest;

    #[test]
    fn multioctet_line_wrapping() {
        let input = b"\xc3\r\n \xbc";
        let line = LineReader::from_slice(input.as_slice())
            .next()
            .unwrap()
            .unwrap();
        assert_eq!(line.as_str(), "ü");
    }

    #[rstest]
    #[case(b"\xc3\r\n \x00")]
    #[case(b"\xc3\r\n ")]
    #[case(b"\xc3 \r\n \xbc")]
    #[case(b"\xc3 \r\n \n\xbc")]
    fn invalid_lines(#[case] input: &[u8]) {
        assert!(LineReader::from_slice(input).next().unwrap().is_err());
    }

    #[test]
    fn ical() {
        let input = include_bytes!("./resources/ical_multiple.ics");
        let lines = LineReader::from_slice(input.as_slice())
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .iter()
            .join("\n");
        assert_snapshot!(lines);
    }

    #[test]
    fn vcard() {
        let input = include_bytes!("./resources/vcard_input.vcf");
        let lines = LineReader::from_slice(input.as_slice())
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .iter()
            .join("\n");
        assert_snapshot!(lines);
    }
}

pub mod calendar_object {
    use caldata::{
        IcalObjectParser, IcalParser, component::CalendarInnerData, generator::Emitter, types::Tz,
    };
    use chrono::{DateTime, Timelike, Utc};
    use itertools::Itertools;

    #[rstest::rstest]
    #[case(0, include_str!("./resources/ical_example_1.ics"), "W. Europe Standard Time")]
    #[case(1, include_str!("./resources/ical_example_2.ics"), "W. Europe Standard Time")]
    #[case(2, include_str!("./resources/ical_example_rrule.ics"), "Europe/Berlin")]
    #[case(3, include_str!("./resources/ical_events.ics"), "")]
    #[case(4, include_str!("./resources/ical_special_symbols.ics"), "")]
    #[case(5, include_str!("./resources/ical_todos.ics"), "")]
    #[case(6, include_str!("./resources/ical_journals.ics"), "")]
    #[case(7, include_str!("./resources/recurring_wholeday.ics"), "")]
    // Calendar objects from Thunderbird are invalid since their VTIMEZONE object RRULEs contain an
    // UNTIL datetime in local time (this MUST be UTC)
    #[case(8, include_str!("./resources/ical_thunderbird.ics"), "Europe/Berlin")]
    #[case(9, include_str!("./resources/ical_recurrence_date.ics"), "")]
    #[case(10, include_str!("./resources/ical_recurrence_date_2.ics"), "")]
    fn valid_objects(#[case] case: usize, #[case] input: &str, #[case] tzids: &str) {
        set_snapshot_suffix!("{case}");
        let generic_reader = IcalParser::from_slice(input.as_bytes());
        let reader = IcalObjectParser::from_slice(input.as_bytes());
        for (g_res, res) in generic_reader.zip(reader) {
            let g_cal = g_res.unwrap();
            let cal = res.unwrap();
            similar_asserts::assert_eq!(g_cal.generate(), cal.generate());
            similar_asserts::assert_eq!(cal.get_tzids().iter().sorted().join(","), tzids);
        }
    }

    #[rstest::rstest]
    #[case(0, include_str!("./resources/ical_freebusy.ics"))]
    fn invalid_objects(#[case] case: usize, #[case] input: &str) {
        set_snapshot_suffix!("{case}");
        let reader = IcalObjectParser::from_slice(input.as_bytes());
        for res in reader {
            assert!(res.is_err());
        }
    }

    /// Expand a reccurent event across winter and summer time
    /// This will fail if we wrongly expand in UTC
    #[rstest::rstest]
    fn rrule_expansion_timezones() {
        let input = include_str!("./resources/ical_recurrence_winter_summer.ics");
        let obj = IcalObjectParser::from_slice(input.as_bytes())
            .expect_one()
            .unwrap();
        let CalendarInnerData::Event(event, _) = obj.get_inner() else {
            panic!()
        };
        let expanded = event.expand_recurrence(None, None, &[]);
        for recurrence in expanded {
            let datetime: DateTime<Tz> = recurrence.dtstart.0.clone().into();
            let datetime_local = datetime.with_timezone(&Tz::Olson(chrono_tz::Tz::Europe__Berlin));
            assert_eq!(datetime_local.hour(), 9);
        }
    }

    #[rstest::rstest]
    #[case(0, include_str!("./resources/Recurring at 9am, third at 10am.ics"))]
    #[case(1, include_str!("./resources/recurring_wholeday.ics"))]
    #[case(2, include_str!("./resources/ical_recurrence_date.ics"))]
    #[case(3, include_str!("./resources/ical_recurrence_date_2.ics"))]
    // Has no RRULE
    #[case(4, include_str!("./resources/ical_example_1.ics"))]
    fn rrule_expansion(#[case] case: usize, #[case] input: &str) {
        set_snapshot_suffix!("{case}");
        let reader = IcalObjectParser::from_slice(input.as_bytes());
        for (i, res) in reader.enumerate() {
            let cal = res.unwrap();
            let recurrence = cal.expand_recurrence(None, None).unwrap();
            assert!(recurrence.get_tzids().is_empty());
            insta::assert_snapshot!(format!("{i}_ics"), recurrence.generate());
            insta::assert_debug_snapshot!(format!("{i}_data"), recurrence.get_inner());
        }
    }

    #[rstest::rstest]
    #[case(include_str!("./resources/ical_recurrence_date_2.ics"))]
    fn rrule_expansion_empty(#[case] input: &str) {
        let reader = IcalObjectParser::from_slice(input.as_bytes());
        for (i, res) in reader.enumerate() {
            let cal = res.unwrap();
            let recurrence = cal.expand_recurrence(Some(DateTime::<Utc>::MAX_UTC), None);
            assert!(recurrence.is_none());
        }
    }
}

pub mod rfc7809 {
    use caldata::{IcalObjectParser, IcalParser, generator::Emitter, parser::ParserOptions};

    #[rstest::rstest]
    #[case(0, include_str!("./resources/ical_rfc7809.ics"))]
    #[case(1, include_str!("./resources/ical_rfc7809_journal.ics"))]
    #[case(2, include_str!("./resources/ical_rfc7809_todo.ics"))]
    fn rfc7809(#[case] case: usize, #[case] input: &str) {
        set_snapshot_suffix!("{case}");
        let reader = IcalObjectParser::from_slice(input.as_bytes());
        assert!(reader.expect_one().is_err());
        let reader = IcalObjectParser::from_slice(input.as_bytes())
            .with_options(ParserOptions { rfc7809: true });

        let cal = reader.expect_one().unwrap();
        insta::assert_snapshot!(cal.generate());

        let reader = IcalParser::from_slice(input.as_bytes());
        assert!(reader.expect_one().is_err());
        let reader =
            IcalParser::from_slice(input.as_bytes()).with_options(ParserOptions { rfc7809: true });

        let cal2 = reader.expect_one().unwrap();
        insta::assert_snapshot!("fullcal", cal2.generate());
    }

    #[cfg(feature = "chrono-tz")]
    #[test]
    fn test_tzdb_version() {
        assert_eq!(
            chrono_tz::IANA_TZDB_VERSION,
            vtimezones_rs::IANA_TZDB_VERSION
        );
    }
}

pub mod parser {
    use caldata::{
        IcalObjectParser, IcalParser, VcardParser, component::IcalCalendar, generator::Emitter,
    };

    use crate::str_normalise_prop_order;

    #[test]
    fn ical_parse_everything() {
        let input = include_str!("./resources/ical_everything.ics");
        let reader = IcalParser::from_slice(input.as_bytes());
        let cal = reader.expect_one();
        cal.unwrap();
    }

    #[test]
    fn ical_multiple() {
        let input = include_str!("./resources/ical_multiple.ics");
        let reader = IcalParser::from_slice(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            insta::assert_debug_snapshot!(cal);
        }
    }

    #[test]
    fn ical_example_1() {
        let input = include_str!("./resources/ical_example_1.ics");
        let reader = IcalParser::from_slice(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            insta::assert_debug_snapshot!(cal);
        }
    }

    #[test]
    fn ical_example_2() {
        let input = include_str!("./resources/ical_example_2.ics");
        let reader = IcalParser::from_slice(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            insta::assert_debug_snapshot!(cal);
        }
    }

    #[test]
    fn ical_example_rrule() {
        let input = include_str!("./resources/ical_example_rrule.ics");
        let reader = IcalParser::from_slice(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            similar_asserts::assert_eq!(cal.generate(), input);
            insta::assert_debug_snapshot!(cal);
        }
    }

    #[test]
    fn ical_example_events() {
        let input = include_str!("./resources/ical_events.ics");
        let reader = IcalParser::from_slice(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            similar_asserts::assert_eq!(cal.generate(), input);
            insta::assert_debug_snapshot!(cal);
        }
    }

    #[test]
    fn ical_special_symbols() {
        let input = include_str!("./resources/ical_special_symbols.ics");
        let reader = IcalParser::from_slice(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            insta::assert_debug_snapshot!(cal);
        }
    }

    #[test]
    fn ical_example_todos() {
        let input = include_str!("./resources/ical_todos.ics");
        let reader = IcalParser::from_slice(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            similar_asserts::assert_eq!(cal.generate(), input);
            insta::assert_debug_snapshot!(cal);
        }
    }

    #[test]
    fn ical_example_journals() {
        let input = include_str!("./resources/ical_journals.ics");
        let reader = IcalParser::from_slice(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            similar_asserts::assert_eq!(cal.generate(), input);
            insta::assert_debug_snapshot!(cal);
        }
    }

    #[test]
    fn ical_example_freebusy() {
        let input = include_str!("./resources/ical_freebusy.ics");
        let reader = IcalParser::from_slice(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            similar_asserts::assert_eq!(cal.generate(), input);
            insta::assert_debug_snapshot!(cal);
        }
    }

    // #[test]
    // fn ical_expand() {
    //     let input = include_str!("./resources/ical_expand.ics");
    //     let reader = IcalParser::from_slice(input.as_bytes());
    //     for res in reader {
    //         let cal = res.unwrap();
    //         similar_asserts::assert_eq!(cal.generate(), input);
    //         insta::assert_debug_snapshot!(cal.expand_calendar());
    //     }
    // }

    #[test]
    fn ical_export() {
        let input1 = include_str!("./resources/ical_events.ics");
        let input2 = include_str!("./resources/ical_example_1.ics");
        let input3 = include_str!("./resources/ical_example_rrule.ics");
        let cal1 = IcalObjectParser::from_slice(input1.as_bytes())
            .expect_one()
            .unwrap();
        let cal2 = IcalObjectParser::from_slice(input2.as_bytes())
            .expect_one()
            .unwrap();
        let cal3 = IcalObjectParser::from_slice(input3.as_bytes())
            .expect_one()
            .unwrap();
        let export = IcalCalendar::from_objects(
            "caldata-rs test".to_owned(),
            vec![cal1.to_owned(), cal2.to_owned(), cal3.to_owned()],
            vec![],
        )
        .generate();
        insta::assert_snapshot!(export);
        // Ensure that exported calendar is valid
        let roundtrip_cal = IcalParser::from_slice(export.as_bytes())
            .expect_one()
            .unwrap();

        let mut reference = vec![cal1, cal2, cal3];
        let mut reimported = roundtrip_cal.into_objects().unwrap();
        reference.sort_by_key(|o| o.get_uid().to_owned());
        reimported.sort_by_key(|o| o.get_uid().to_owned());
        assert_eq!(reimported.len(), reference.len());
        for (mut reference, mut reimported) in reference.into_iter().zip(reimported) {
            // PRODID gets overwritten
            reference.properties = vec![];
            reimported.properties = vec![];
            similar_asserts::assert_eq!(
                str_normalise_prop_order(&reference.generate()),
                str_normalise_prop_order(&reimported.generate())
            );
        }
    }

    #[test]
    fn vcard() {
        let input = include_str!("./resources/vcard_input.vcf");
        let reader = VcardParser::from_slice(input.as_bytes());
        let card = reader.expect_one().unwrap();
        assert_eq!(card.get_uid(), Some("jdoelaskdjlaksjd"))
    }

    #[test]
    fn vcard_lowercase() {
        let input = include_str!("./resources/vcard_lowercase.vcf");
        let reader = VcardParser::from_slice(input.as_bytes());
        for res in reader {
            let card = res.unwrap();
            insta::assert_debug_snapshot!(card);
            similar_asserts::assert_eq!(card.generate().to_lowercase(), input.to_lowercase());
        }
    }

    #[test]
    fn vcard_invalid() {
        let input = include_str!("./resources/vcard_invalid.vcf");
        let reader = VcardParser::from_slice(input.as_bytes());
        for res in reader {
            assert!(res.is_err());
        }
    }
}

pub mod generator {
    use caldata::IcalParser;
    use caldata::generator::Emitter;

    #[test]
    fn generate_o365_test() {
        let input = include_str!("./resources/o365_meeting.ics");
        let reader = IcalParser::from_slice(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            similar_asserts::assert_eq!(cal.generate(), input);
            insta::assert_debug_snapshot!(cal);
        }
    }

    #[test]
    fn generate_sabre_test() {
        let input = include_str!("./resources/sabre_test.ics");
        let reader = IcalParser::from_slice(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            similar_asserts::assert_eq!(cal.generate(), input);
            insta::assert_debug_snapshot!(cal);
        }
    }
}

#[cfg(feature = "chrono-tz")]
pub mod chrono_tz {
    use caldata::component::IcalTimeZone;
    use caldata::parser::ComponentParser;
    use rstest::rstest;
    const VTIMEZONE_DIFFERENT_TZID_BERLIN: &str = r#"
BEGIN:VTIMEZONE
TZID:HELLO_Europe/Berlin
LAST-MODIFIED:20250723T154628Z
X-LIC-LOCATION:Europe/Berlin
BEGIN:DAYLIGHT
TZNAME:CEST
TZOFFSETFROM:+0100
TZOFFSETTO:+0200
DTSTART:19700329T020000
RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU
END:DAYLIGHT
BEGIN:STANDARD
TZNAME:CET
TZOFFSETFROM:+0200
TZOFFSETTO:+0100
DTSTART:19701025T030000
RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU
END:STANDARD
END:VTIMEZONE
    "#;

    const VTIMEZONE_BERLIN: &str = r#"
BEGIN:VTIMEZONE
TZID:Europe/Berlin
LAST-MODIFIED:20250723T154628Z
X-LIC-LOCATION:Europe/Berlin
BEGIN:DAYLIGHT
TZNAME:CEST
TZOFFSETFROM:+0100
TZOFFSETTO:+0200
DTSTART:19700329T020000
RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU
END:DAYLIGHT
BEGIN:STANDARD
TZNAME:CET
TZOFFSETFROM:+0200
TZOFFSETTO:+0100
DTSTART:19701025T030000
RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU
END:STANDARD
END:VTIMEZONE
    "#;

    const VTIMEZONE_LOWERCASE: &str = r#"
BEGIN:VTIMEZONE
tzid:W. Europe Standard Time
LAST-MODIFIED:20250723T154628Z
BEGIN:DAYLIGHT
TZNAME:CEST
TZOFFSETFROM:+0100
TZOFFSETTO:+0200
DTSTART:19700329T020000
RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU
END:DAYLIGHT
BEGIN:STANDARD
TZNAME:CET
TZOFFSETFROM:+0200
TZOFFSETTO:+0100
DTSTART:19701025T030000
RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU
END:STANDARD
END:VTIMEZONE
    "#;

    const VTIMEZONE_PROPRIETARY: &str = r#"
BEGIN:VTIMEZONE
TZID:W. Europe Standard Time
LAST-MODIFIED:20250723T154628Z
BEGIN:DAYLIGHT
TZNAME:CEST
TZOFFSETFROM:+0100
TZOFFSETTO:+0200
DTSTART:19700329T020000
RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU
END:DAYLIGHT
BEGIN:STANDARD
TZNAME:CET
TZOFFSETFROM:+0200
TZOFFSETTO:+0100
DTSTART:19701025T030000
RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU
END:STANDARD
END:VTIMEZONE
    "#;

    #[rstest]
    #[case(VTIMEZONE_BERLIN, chrono_tz::Europe::Berlin)]
    #[case(VTIMEZONE_DIFFERENT_TZID_BERLIN, chrono_tz::Europe::Berlin)]
    #[case(VTIMEZONE_LOWERCASE, chrono_tz::Europe::Berlin)]
    #[case(VTIMEZONE_PROPRIETARY, chrono_tz::Europe::Berlin)]
    fn try_from_icaldatetime(#[case] input: &str, #[case] tz: chrono_tz::Tz) {
        let vtimezone: IcalTimeZone =
            ComponentParser::<'_, IcalTimeZone, _>::from_slice(input.as_bytes())
                .next()
                .unwrap()
                .unwrap();
        let extracted_tz: Option<chrono_tz::Tz> = (&vtimezone).into();
        assert_eq!(tz, extracted_tz.unwrap());
    }
}
