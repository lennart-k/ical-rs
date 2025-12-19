use criterion::{Criterion, criterion_group, criterion_main};
use ical::generator::{Emitter, IcalCalendar};

fn parse_ical() -> IcalCalendar {
    let input = include_str!("../tests/resources/ical_everything.ics");
    let reader = ical::IcalParser::new(input.as_bytes());
    reader.into_iter().next().unwrap().unwrap()
}

fn benchmark(c: &mut Criterion) {
    c.bench_function("parse ical_everything.ics", |b| b.iter(parse_ical));
    let cal = parse_ical();
    c.bench_function("serialise ical_everything.ics", |b| {
        b.iter(|| cal.generate())
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
