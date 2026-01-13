use std::fs::read_to_string;

extern crate ical;

fn main() {
    let buf = read_to_string("./tests/ressources/ical_input.ics").unwrap();

    let reader = ical::PropertyParser::from_slice(buf.as_bytes());

    for line in reader {
        println!("{:?}", line);
    }
}
