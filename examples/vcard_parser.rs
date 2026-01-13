use std::fs::read_to_string;

extern crate ical;

fn main() {
    let buf = read_to_string("./tests/ressources/vcard_input.vcf").unwrap();

    let reader = ical::VcardParser::from_slice(buf.as_bytes());

    for line in reader {
        println!("{:?}", line);
    }
}
