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
//! use std::fs::read_to_string;
//!
//! let buf = read_to_string("./tests/resources/ical_multiple.ics")
//! .unwrap();
//!
//! let reader = ical::IcalParser::from_slice(buf.as_bytes());
//!
//! for line in reader {
//!     println!("{:?}", line);
//! }
//! ```

pub mod component;
use component::IcalCalendar;

use crate::parser::{ComponentParser, ical::component::IcalCalendarObject};

/// Reader returning `IcalCalendar` object from a `BufRead`.
pub type IcalParser<'a, I> = ComponentParser<'a, IcalCalendar, I>;
pub type IcalObjectParser<'a, I> = ComponentParser<'a, IcalCalendarObject, I>;
