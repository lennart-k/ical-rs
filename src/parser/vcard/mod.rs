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
//! use std::fs::read_to_string;
//!
//! let buf = read_to_string("./tests/resources/vcard_input.vcf")
//! .unwrap();
//!
//! let reader = ical::VcardParser::from_slice(buf.as_bytes());
//!
//! for contact in reader {
//!     println!("{:?}", contact);
//! }
//! ```

pub mod component;
use crate::parser::ComponentParser;
use component::VcardContact;

pub type VcardParser<'a, I> = ComponentParser<'a, VcardContact, I>;
