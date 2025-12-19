//! Parse the result of `LineReader` into parts.
//!
//! Split the result of `LineReader` into property. A property contains:
//! - A name formated in uppercase.
//! - An optional list of parameters represented by a vector of `(key/value)` tuple . The key is
//!   formatted in uppercase and the value stay untouched.
//! - A value stay untouched.
//!
//! It work for both the Vcard and Ical format.
//!
//! #### Warning
//!   The parsers `PropertyParser` only parse the content and set to uppercase the case-insensitive
//!   fields. No checks are made on the fields validity.
//!
//! # Examples
//!
//! ```toml
//! [dependencies.ical]
//! version = "0.3.*"
//! default-features = false
//! features = ["property"]
//! ```
//!
//! ```rust
//! extern crate ical;
//!
//! use std::io::BufReader;
//! use std::fs::File;
//!
//! let buf = BufReader::new(File::open("./tests/resources/vcard_input.vcf")
//!     .unwrap());
//!
//! let reader = ical::PropertyParser::from_reader(buf);
//!
//! for line in reader {
//!     println!("{:?}", line);
//! }
//! ```

// Sys mods
use std::fmt;
use std::io::BufRead;
use std::iter::Iterator;

// Internal mods
use crate::{
    PARAM_DELIMITER, PARAM_NAME_DELIMITER, PARAM_QUOTE, PARAM_VALUE_DELIMITER, VALUE_DELIMITER,
    line::{Line, LineReader},
};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PropertyError {
    #[error("Line {0}: Missing property name.")]
    MissingName(usize),
    #[error("Line {0}: Missing a closing quote.")]
    MissingClosingQuote(usize),
    #[error("Line {0}: Missing a \"{1}\" delimiter.")]
    MissingDelimiter(usize, char),
    #[error("Line {0}: Missing content after \"{1}\".")]
    MissingContentAfter(usize, char),
    #[error("Line {0}: Missing a parameter key.")]
    MissingParamKey(usize),
    #[error("Line {0}: Missing value.")]
    MissingValue(usize),
}

/// A VCARD/ICAL property.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Property {
    /// Property name.
    pub name: String,
    /// Property list of parameters.
    pub params: Vec<(String, Vec<String>)>,
    /// Property value.
    pub value: Option<String>,
}

impl Property {
    /// Return a new `Property` object.
    pub fn new() -> Property {
        Property {
            name: String::new(),
            params: vec![],
            value: None,
        }
    }

    pub fn get_param(&self, name: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|(key, _)| name == key)
            .and_then(|(_, value)| value.iter().map(String::as_str).next())
    }

    pub fn get_tzid(&self) -> Option<&str> {
        self.get_param("TZID")
    }

    pub fn get_value_type(&self) -> Option<&str> {
        self.get_param("VALUE")
    }
}

impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "name: {}\nparams: {:?}\nvalue: {:?}",
            self.name, self.params, self.value
        )
    }
}

pub struct PropertyParser<B: BufRead>(LineReader<B>);

impl<B: BufRead> PropertyParser<B> {
    pub fn new(line_reader: LineReader<B>) -> PropertyParser<B> {
        PropertyParser(line_reader)
    }

    pub fn from_reader(reader: B) -> PropertyParser<B> {
        PropertyParser(LineReader::new(reader))
    }

    fn parse(&self, line: Line) -> Result<Property, PropertyError> {
        let to_parse = line.as_str();

        // Find end of parameter name
        let Some(end_name_index) = to_parse.find([PARAM_DELIMITER, VALUE_DELIMITER]) else {
            return Err(PropertyError::MissingName(line.number()));
        };
        let (prop_name, mut to_parse) = to_parse.split_at(end_name_index);
        if prop_name.is_empty() {
            return Err(PropertyError::MissingName(line.number()));
        }

        // remainder either starts with ; or :
        // Fetch all parameters
        let mut params = vec![];
        while to_parse.starts_with(PARAM_DELIMITER) {
            to_parse = to_parse.split_at(1).1;

            // Split the param key and the rest of the line
            let Some((key, remainder)) = to_parse.split_once(PARAM_NAME_DELIMITER) else {
                return Err(PropertyError::MissingDelimiter(
                    line.number(),
                    PARAM_NAME_DELIMITER,
                ));
            };
            if key.is_empty() {
                return Err(PropertyError::MissingParamKey(line.number()));
            }
            to_parse = remainder;

            let mut values = Vec::new();

            // Parse parameter value.
            loop {
                if to_parse.starts_with('"') {
                    // This is a dquoted value. (NAME:Foo="Bar":value)
                    let mut elements = to_parse.splitn(3, PARAM_QUOTE).skip(1);
                    // unwrap is safe here as we have already check above if there is on '"'.
                    values.push(
                        elements
                            .next()
                            .ok_or_else(|| PropertyError::MissingClosingQuote(line.number()))?
                            .to_string(),
                    );

                    to_parse = elements
                        .next()
                        .ok_or_else(|| PropertyError::MissingClosingQuote(line.number()))?
                } else {
                    // This is a 'raw' value. (NAME;Foo=Bar:value)
                    // Try to find the next param separator.
                    let Some(end_param_value) =
                        to_parse.find([PARAM_DELIMITER, VALUE_DELIMITER, PARAM_VALUE_DELIMITER])
                    else {
                        return Err(PropertyError::MissingContentAfter(
                            line.number(),
                            PARAM_NAME_DELIMITER,
                        ));
                    };

                    let elements = to_parse.split_at(end_param_value);
                    values.push(elements.0.to_string());
                    to_parse = elements.1;
                }

                if !to_parse.starts_with(PARAM_VALUE_DELIMITER) {
                    break;
                }

                to_parse = to_parse.trim_start_matches(PARAM_VALUE_DELIMITER);
            }

            params.push((key.to_uppercase(), values));
        }

        // Parse value
        if !to_parse.starts_with(VALUE_DELIMITER) {
            return Err(PropertyError::MissingValue(line.number()));
        }
        to_parse = to_parse.split_at(1).1;
        Ok(Property {
            name: prop_name.to_string(),
            params,
            value: (!to_parse.is_empty()).then_some(to_parse.to_string()),
        })
    }
}

impl<B: BufRead> Iterator for PropertyParser<B> {
    type Item = Result<Property, PropertyError>;

    fn next(&mut self) -> Option<Result<Property, PropertyError>> {
        self.0.next().map(|line| self.parse(line))
    }
}
