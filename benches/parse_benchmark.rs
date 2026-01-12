use criterion::{Criterion, criterion_group, criterion_main};
use ical::{
    generator::{Emitter, IcalCalendar},
    parser::{ICalProperty, IcalDTSTARTProperty},
    property::ContentLine,
    types::{CalDate, CalDateTime, PartialDate},
};

fn parse_ical() -> IcalCalendar {
    let input = include_str!("../tests/resources/ical_everything.ics");
    let reader = ical::IcalParser::new(input.as_bytes());
    reader.into_iter().next().unwrap().unwrap()
}

fn benchmark(c: &mut Criterion) {
    c.bench_function("parse PartialDate", |b| {
        b.iter(|| {
            PartialDate::parse("--0329").unwrap();
        })
    });
    c.bench_function("parse CalDate", |b| {
        b.iter(|| {
            CalDate::parse("19700329", None).unwrap();
        })
    });
    c.bench_function("parse CalDateTime UTC", |b| {
        b.iter(|| {
            CalDateTime::parse("19700329T020000Z", None).unwrap();
        })
    });
    c.bench_function("parse CalDateTime Local", |b| {
        b.iter(|| {
            CalDateTime::parse("19700329T020000", None).unwrap();
        })
    });

    c.bench_function("ics parse DTSTART", |b| {
        b.iter(|| {
            let content_line = ContentLine {
                name: "DTSTART".to_owned(),
                value: Some("19700329T020000Z".to_owned()),
                params: vec![].into(),
            };
            IcalDTSTARTProperty::parse_prop(&content_line, None).unwrap();
        })
    });

    c.bench_function("line parse ical_everything.ics", |b| {
        b.iter(|| {
            let input = include_str!("../tests/resources/ical_everything.ics");
            let reader = ical::LineReader::new(input.as_bytes());
            // Consume reader
            for _ in reader {}
        })
    });
    c.bench_function("ics parse ical_everything.ics", |b| b.iter(parse_ical));
    let cal = parse_ical();
    c.bench_function("ics serialise ical_everything.ics", |b| {
        b.iter(|| cal.generate())
    });
    // #[cfg(feature = "rkyv")]
    // c.bench_function("rkyv serialise ical_everything.ics", |b| {
    //     b.iter(|| rkyv::to_bytes::<rkyv::rancor::Error>(&cal).unwrap())
    // });

    // let rkyv_bytes = include_bytes!("ical_everything.rkyv");
    // #[cfg(feature = "rkyv")]
    // c.bench_function("rkyv deserialise ical_everything.ics", |b| {
    //     b.iter(|| {
    //         use ical::parser::ical::component::ArchivedIcalCalendar;
    //
    //         let archived =
    //             rkyv::access::<ArchivedIcalCalendar, rkyv::rancor::Error>(rkyv_bytes).unwrap();
    //         rkyv::deserialize::<_, rkyv::rancor::Error>(archived).unwrap();
    //     })
    // });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
