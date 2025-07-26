#[cfg(feature = "property")]
pub mod property {
    extern crate ical;

    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;

    #[test]
    fn ical() {
        let input = include_bytes!("./resources/ical_multiple.ics");

        let mut valids =
            BufReader::new(File::open("./tests/resources/ical_property.res").unwrap()).lines();

        let reader = ical::PropertyParser::from_reader(input.as_slice());

        for res in reader {
            let output = format!("{:?}", res.unwrap());
            assert_eq!(output, valids.next().unwrap().unwrap());
        }
    }

    #[test]
    fn vcard() {
        let input = include_bytes!("./resources/vcard_input.vcf");

        let mut valids =
            BufReader::new(File::open("./tests/resources/vcard_property.res").unwrap()).lines();

        let reader = ical::PropertyParser::from_reader(input.as_slice());

        for res in reader {
            let contact = match res {
                Ok(res) => res,
                Err(err) => panic!("Throw error: {err}"),
            };
            let output = format!("{contact:?}");
            assert_eq!(output, valids.next().unwrap().unwrap());
        }
    }

    #[test]
    fn errors() {
        let input = BufReader::new(File::open("./tests/resources/property_error.vcf").unwrap());

        let mut valids =
            BufReader::new(File::open("./tests/resources/property_error.res").unwrap()).lines();

        let reader = ical::PropertyParser::from_reader(input);

        for res in reader {
            let error = match res {
                Ok(res) => panic!("Should return an error: {res:?}"),
                Err(err) => err,
            };
            let output = format!("{error}");
            assert_eq!(output, valids.next().unwrap().unwrap());
        }
    }
}

#[cfg(feature = "line")]
pub mod line {
    extern crate ical;

    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;

    #[test]
    fn ical() {
        let input = include_bytes!("./resources/ical_multiple.ics");
        let mut valids =
            BufReader::new(File::open("./tests/resources/ical_line.res").unwrap()).lines();

        let reader = ical::LineReader::new(input.as_slice());
        for line in reader {
            let output = format!("{line:?}");
            assert_eq!(output, valids.next().unwrap().unwrap());
        }
    }

    #[test]
    fn vcard() {
        let input = include_bytes!("./resources/vcard_input.vcf");
        let mut valids =
            BufReader::new(File::open("./tests/resources/vcard_line.res").unwrap()).lines();

        let reader = ical::LineReader::new(input.as_slice());
        for line in reader {
            let output = format!("{line:?}");
            assert_eq!(output, valids.next().unwrap().unwrap());
        }
    }
}

#[cfg(any(feature = "ical", feature = "vcard"))]
pub mod parser {
    extern crate ical;

    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;

    #[test]
    fn ical() {
        let input = include_bytes!("./resources/ical_multiple.ics");
        let reader = ical::IcalParser::new(input.as_slice());
        let valids = include_str!("./resources/ical_multiple.res").lines();

        for (res, valid) in reader.zip(valids) {
            let output = format!("{:?}", res.unwrap());
            assert_eq!(output, valid);
        }
    }

    #[test]
    fn ical_example_1() {
        let input = include_bytes!("./resources/ical_example_1.ics");
        let reader = ical::IcalParser::new(input.as_slice());
        let valids = include_str!("./resources/ical_example_1.res").replace('\n', "");

        for res in reader {
            let output = format!("{:?}", res.unwrap());
            assert_eq!(output, valids);
        }
    }

    #[test]
    // same as ical_example_1 but with \r\n endings instead of \n.
    fn ical_example_2() {
        let input = include_bytes!("./resources/ical_example_2.ics");
        let reader = ical::IcalParser::new(input.as_slice());
        let valids = include_str!("./resources/ical_example_2.res").replace('\n', "");

        for res in reader {
            let output = format!("{:?}", res.unwrap());
            assert_eq!(output, valids);
        }
    }

    #[test]
    fn ical_example_rrule() {
        let input = include_bytes!("./resources/ical_example_rrule.ics");
        let reader = ical::IcalParser::new(input.as_slice());
        let reference = include_str!("./resources/ical_example_rrule.res").replace('\n', "");

        for res in reader {
            let output = format!("{:?}", res.unwrap());
            assert_eq!(output, reference);
        }
    }

    #[test]
    fn ical_example_todo() {
        let input = include_bytes!("./resources/ical_todos.ics");
        let reader = ical::IcalParser::new(input.as_slice());
        let references = include_str!("./resources/ical_todos.res").lines();

        for (res, reference) in reader.zip(references) {
            let output = format!("{:?}", res.unwrap());
            assert_eq!(output, reference);
        }
    }

    #[test]
    fn ical_example_journal() {
        let input = include_bytes!("./resources/ical_journals.ics");
        let reader = ical::IcalParser::new(input.as_slice());
        let references = include_str!("./resources/ical_journals.res").lines();

        for (res, reference) in reader.zip(references) {
            let output = format!("{:?}", res.unwrap());
            assert_eq!(output, reference);
        }
    }

    #[test]
    fn vcard() {
        let input = BufReader::new(File::open("./tests/resources/vcard_input.vcf").unwrap());

        let mut valids =
            BufReader::new(File::open("./tests/resources/vcard_parser.res").unwrap()).lines();

        let reader = ical::VcardParser::new(input);

        for res in reader {
            let contact = match res {
                Ok(res) => res,
                Err(err) => panic!("Throw error: {err}"),
            };

            let output = format!("{contact:?}");

            assert_eq!(output, valids.next().unwrap().unwrap());
        }
    }

    #[test]
    fn vcard_lowercase() {
        let input = BufReader::new(File::open("./tests/resources/vcard_lowercase.vcf").unwrap());

        let mut valids =
            BufReader::new(File::open("./tests/resources/vcard_lowercase.res").unwrap()).lines();

        let reader = ical::VcardParser::new(input);

        for res in reader {
            let contact = match res {
                Ok(res) => res,
                Err(err) => panic!("Throw error: {err:?}"),
            };

            let output = format!("{contact:?}");
            assert_eq!(output, valids.next().unwrap().unwrap());
        }
    }
}

#[cfg(all(feature = "ical", feature = "generator"))]
pub mod generator {
    extern crate ical;
    use self::ical::generator::Emitter;
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;

    #[test]
    fn generate_o365_test() {
        let filename = "./tests/resources/o365_meeting.ics";

        let original = BufReader::new(File::open(filename).unwrap())
            .lines()
            .map(|line| line.unwrap() + "\r\n")
            .collect::<String>();

        let input = BufReader::new(File::open(filename).unwrap());
        let mut reader = ical::IcalParser::new(input);
        let generated = reader.next().unwrap().ok().unwrap().generate();

        assert_eq!(&generated, &original);
    }

    #[test]
    fn generate_sabre_test() {
        let filename = "./tests/resources/sabre_test.ics";

        let original = BufReader::new(File::open(filename).unwrap())
            .lines()
            .map(|line| line.unwrap() + "\r\n")
            .collect::<String>();

        let input = BufReader::new(File::open(filename).unwrap());
        let mut reader = ical::IcalParser::new(input);
        let generated = reader.next().unwrap().ok().unwrap().generate();

        assert_eq!(&generated, &original);
    }
}

#[cfg(all(feature = "ical", feature = "chrono-tz"))]
pub mod chrono_tz {
    extern crate ical;
    use self::ical::parser::ical::component::IcalTimeZone;
    use ical::parser::ComponentParser;
    use std::convert::TryInto;

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

    #[test]
    fn try_from_icaldatetime() {
        for input in [VTIMEZONE_BERLIN, VTIMEZONE_DIFFERENT_TZID_BERLIN] {
            let vtimezone: IcalTimeZone = ComponentParser::<_, IcalTimeZone>::new(input.as_bytes())
                .next()
                .unwrap()
                .unwrap();

            assert_eq!(
                chrono_tz::Tz::Europe__Berlin,
                (&vtimezone).try_into().unwrap()
            );
        }
    }
}
