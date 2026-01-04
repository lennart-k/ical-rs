use criterion::{Criterion, criterion_group, criterion_main};
use ical::generator::{Emitter, IcalCalendar};

fn parse_ical() -> IcalCalendar {
    let input = include_str!("../tests/resources/ical_everything.ics");
    let reader = ical::IcalParser::new(input.as_bytes());
    reader.into_iter().next().unwrap().unwrap()
}

fn benchmark(c: &mut Criterion) {
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
