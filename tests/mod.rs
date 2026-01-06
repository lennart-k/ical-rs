macro_rules! set_snapshot_suffix {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_suffix(format!($($expr,)*));
        let _guard = settings.bind_to_scope();
    }
}

pub mod property {
    extern crate ical;

    #[test]
    fn ical() {
        let input = include_str!("./resources/ical_multiple.ics");
        let reader = ical::PropertyParser::from_reader(input.as_bytes());
        for res in reader {
            let prop = res.unwrap();
            insta::assert_snapshot!(prop);
        }
    }

    #[test]
    fn vcard() {
        let input = include_str!("./resources/vcard_input.vcf");
        let reader = ical::PropertyParser::from_reader(input.as_bytes());
        for res in reader {
            let prop = res.unwrap();
            insta::assert_snapshot!(prop);
        }
    }

    #[test]
    fn errors() {
        let input = include_str!("./resources/property_error.vcf");
        let reader = ical::PropertyParser::from_reader(input.as_bytes());
        for res in reader {
            assert!(res.is_err());
        }
    }
}

pub mod line {
    extern crate ical;

    use insta::assert_snapshot;
    use itertools::Itertools;

    #[test]
    fn ical() {
        let input = include_bytes!("./resources/ical_multiple.ics");
        let lines = ical::LineReader::new(input.as_slice()).join("\n");
        assert_snapshot!(lines);
    }

    #[test]
    fn vcard() {
        let input = include_bytes!("./resources/vcard_input.vcf");
        let lines = ical::LineReader::new(input.as_slice()).join("\n");
        assert_snapshot!(lines);
    }
}

pub mod calendar_object {
    extern crate ical;
    use ical::generator::Emitter;

    #[rstest::rstest]
    #[case(0, include_str!("./resources/ical_example_1.ics"))]
    #[case(1, include_str!("./resources/ical_example_2.ics"))]
    #[case(2, include_str!("./resources/ical_example_rrule.ics"))]
    #[case(3, include_str!("./resources/ical_events.ics"))]
    #[case(4, include_str!("./resources/ical_special_symbols.ics"))]
    #[case(5, include_str!("./resources/ical_todos.ics"))]
    #[case(6, include_str!("./resources/ical_journals.ics"))]
    #[case(7, include_str!("./resources/recurring_wholeday.ics"))]
    fn valid_objects(#[case] case: usize, #[case] input: &str) {
        set_snapshot_suffix!("{case}");
        let generic_reader = ical::IcalParser::new(input.as_bytes());
        let reader = ical::IcalObjectParser::new(input.as_bytes());
        for (g_res, res) in generic_reader.zip(reader) {
            let g_cal = g_res.unwrap();
            let cal = res.unwrap();
            similar_asserts::assert_eq!(g_cal.generate(), cal.generate());
        }
    }

    #[rstest::rstest]
    #[case(0, include_str!("./resources/ical_freebusy.ics"))]
    fn invalid_objects(#[case] case: usize, #[case] input: &str) {
        set_snapshot_suffix!("{case}");
        let reader = ical::IcalObjectParser::new(input.as_bytes());
        for res in reader {
            assert!(res.is_err());
        }
    }

    #[rstest::rstest]
    #[case(0, include_str!("./resources/Recurring at 9am, third at 10am.ics"))]
    #[case(1, include_str!("./resources/recurring_wholeday.ics"))]
    fn rrule_expansion(#[case] case: usize, #[case] input: &str) {
        set_snapshot_suffix!("{case}");
        let reader = ical::IcalObjectParser::new(input.as_bytes());
        for (i, res) in reader.enumerate() {
            let cal = res.unwrap();
            let recurrence = cal.expand_recurrence(None, None);
            insta::assert_snapshot!(format!("{i}_ics"), recurrence.generate());
            insta::assert_debug_snapshot!(format!("{i}_data"), recurrence.get_inner());
        }
    }
}

pub mod parser {
    extern crate ical;
    use ical::generator::Emitter;

    #[test]
    fn ical_multiple() {
        let input = include_str!("./resources/ical_multiple.ics");
        let reader = ical::IcalParser::new(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            insta::assert_debug_snapshot!(cal);
        }
    }

    #[test]
    fn ical_example_1() {
        let input = include_str!("./resources/ical_example_1.ics");
        let reader = ical::IcalParser::new(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            insta::assert_debug_snapshot!(cal);
        }
    }

    #[test]
    fn ical_example_2() {
        let input = include_str!("./resources/ical_example_2.ics");
        let reader = ical::IcalParser::new(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            insta::assert_debug_snapshot!(cal);
        }
    }

    #[test]
    fn ical_example_rrule() {
        let input = include_str!("./resources/ical_example_rrule.ics");
        let reader = ical::IcalParser::new(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            similar_asserts::assert_eq!(cal.generate(), input);
            insta::assert_debug_snapshot!(cal);
        }
    }

    #[test]
    fn ical_example_events() {
        let input = include_str!("./resources/ical_events.ics");
        let reader = ical::IcalParser::new(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            similar_asserts::assert_eq!(cal.generate(), input);
            insta::assert_debug_snapshot!(cal);
        }
    }

    #[test]
    fn ical_special_symbols() {
        let input = include_str!("./resources/ical_special_symbols.ics");
        let reader = ical::IcalParser::new(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            insta::assert_debug_snapshot!(cal);
        }
    }

    #[test]
    fn ical_example_todos() {
        let input = include_str!("./resources/ical_todos.ics");
        let reader = ical::IcalParser::new(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            similar_asserts::assert_eq!(cal.generate(), input);
            insta::assert_debug_snapshot!(cal);
        }
    }

    #[test]
    fn ical_example_journals() {
        let input = include_str!("./resources/ical_journals.ics");
        let reader = ical::IcalParser::new(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            similar_asserts::assert_eq!(cal.generate(), input);
            insta::assert_debug_snapshot!(cal);
        }
    }

    #[test]
    fn ical_example_freebusy() {
        let input = include_str!("./resources/ical_freebusy.ics");
        let reader = ical::IcalParser::new(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            similar_asserts::assert_eq!(cal.generate(), input);
            insta::assert_debug_snapshot!(cal);
        }
    }

    // #[test]
    // fn ical_expand() {
    //     let input = include_str!("./resources/ical_expand.ics");
    //     let reader = ical::IcalParser::new(input.as_bytes());
    //     for res in reader {
    //         let cal = res.unwrap();
    //         similar_asserts::assert_eq!(cal.generate(), input);
    //         insta::assert_debug_snapshot!(cal.expand_calendar());
    //     }
    // }

    #[test]
    fn vcard() {
        let input = include_str!("./resources/vcard_input.vcf");
        let reader = ical::VcardParser::new(input.as_bytes());
        for res in reader {
            let card = res.unwrap();
            insta::assert_debug_snapshot!(card);
        }
    }

    #[test]
    fn vcard_lowercase() {
        let input = include_str!("./resources/vcard_lowercase.vcf");
        let reader = ical::VcardParser::new(input.as_bytes());
        for res in reader {
            let card = res.unwrap();
            insta::assert_debug_snapshot!(card);
            similar_asserts::assert_eq!(card.generate().to_lowercase(), input.to_lowercase());
        }
    }

    #[test]
    fn vcard_invalid() {
        let input = include_str!("./resources/vcard_invalid.vcf");
        let reader = ical::VcardParser::new(input.as_bytes());
        for res in reader {
            assert!(res.is_err());
        }
    }
}

pub mod generator {
    extern crate ical;
    use self::ical::generator::Emitter;

    #[test]
    fn generate_o365_test() {
        let input = include_str!("./resources/o365_meeting.ics");
        let reader = ical::IcalParser::new(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            similar_asserts::assert_eq!(cal.generate(), input);
            insta::assert_debug_snapshot!(cal);
        }
    }

    #[test]
    fn generate_sabre_test() {
        let input = include_str!("./resources/sabre_test.ics");
        let reader = ical::IcalParser::new(input.as_bytes());
        for res in reader {
            let cal = res.unwrap();
            similar_asserts::assert_eq!(cal.generate(), input);
            insta::assert_debug_snapshot!(cal);
        }
    }
}

#[cfg(feature = "chrono-tz")]
pub mod chrono_tz {
    extern crate ical;
    use self::ical::parser::ical::component::IcalTimeZone;
    use ical::parser::ComponentParser;
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
        let vtimezone: IcalTimeZone = ComponentParser::<_, IcalTimeZone>::new(input.as_bytes())
            .next()
            .unwrap()
            .unwrap();
        let extracted_tz: Option<chrono_tz::Tz> = (&vtimezone).into();
        assert_eq!(tz, extracted_tz.unwrap());
    }
}
