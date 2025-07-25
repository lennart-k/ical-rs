//! Parse an ICAL calendar.
//!
//! Wrap the result of the `PropertyParser` into components.
//!
//! Each component contains properties (ie: `Property`) or sub-components.
//!
//! * The `VcardParser` return `IcalCalendar` objects.
//!
//! # Examples
//!
//!
//! ```toml
//! [dependencies.ical]
//! version = "0.3.*"
//! default-features = false
//! features = ["ical-parser"]
//! ```
//!
//! ```rust
//! extern crate ical;
//!
//! use std::io::BufReader;
//! use std::fs::File;
//!
//! let buf = BufReader::new(File::open("./tests/ressources/ical_input.ics")
//! .unwrap());
//!
//! let reader = ical::IcalParser::new(buf);
//!
//! for line in reader {
//!     println!("{:?}", line);
//! }
//! ```

pub mod component;
use component::IcalCalendar;

use crate::parser::ComponentParser;

/// Reader returning `IcalCalendar` object from a `BufRead`.
pub type IcalParser<B> = ComponentParser<B, IcalCalendar>;
