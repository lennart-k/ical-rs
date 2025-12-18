extern crate ical;

use std::{fs::File, io::BufReader};

fn main() {
    let buf = BufReader::new(File::open("./tests/ressources/ical_input.ics").unwrap());

    let reader = ical::IcalParser::new(buf);

    for line in reader {
        println!("{:?}", &line);
        match &line {
            Err(_) => {}
            Ok(ical) => {
                let ev = ical as &dyn ical::generator::Emitter;
                println!("{}", ev.generate());
            }
        }
    }
}
