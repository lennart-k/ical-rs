//! Parse a VCARD address book.
//!
//! Wrap the result of the `PropertyParser` into components.
//!
//! Each component contains properties (ie: Property) or sub-components.
//!
//! * The `VcardParser` return `VcardContact` objects.
//!
//! # Examples
//!
//! ```toml
//! [dependencies.ical]
//! version = "0.3.*"
//! default-features = false
//! features = ["vcard-parser"]
//! ```
//!
//! ```rust
//! extern crate ical;
//!
//! use std::io::BufReader;
//! use std::fs::File;
//!
//! let buf = BufReader::new(File::open("./tests/ressources/vcard_input.vcf")
//! .unwrap());
//!
//! let reader = ical::VcardParser::new(buf);
//!
//! for contact in reader {
//!     println!("{:?}", contact);
//! }
//! ```

pub mod component;
use crate::parser::ComponentParser;
use component::VcardContact;

pub type VcardParser<B> = ComponentParser<B, VcardContact>;
