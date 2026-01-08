//! Wrapper around `PropertyParser`
//!
//! #### Warning
//!   The parsers (`VcardParser` / `IcalParser`) only parse the content and set to uppercase
//!   the case-insensitive fields.  No checks are made on the fields validity.
//!
//!
pub mod ical;
pub mod vcard;
use crate::types::{CalDateTimeError, InvalidDuration};
use crate::{
    LineReader,
    property::{ContentLine, PropertyError, PropertyParser},
};
use std::collections::HashMap;
use std::io::BufRead;
use std::marker::PhantomData;

mod property;
pub use property::*;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ParserError {
    #[error("empty input")]
    EmptyInput,
    #[error("too many components in input, expected one")]
    TooManyComponents,
    #[error("invalid component: {0}")]
    InvalidComponent(String),
    #[error("incomplete object")]
    NotComplete,
    #[error("missing header")]
    MissingHeader,
    #[error("property error: {0}")]
    PropertyError(#[from] PropertyError),
    #[error("missing property: {0}")]
    MissingProperty(&'static str),
    #[error("missing property: UID")]
    MissingUID,
    #[error("property conflict: {0}")]
    PropertyConflict(&'static str),
    #[error(transparent)]
    InvalidDuration(#[from] InvalidDuration),
    #[error(transparent)]
    RRule(#[from] rrule::RRuleError),
    #[error(transparent)]
    DateTime(#[from] CalDateTimeError),
}

/// An immutable interface for an Ical/Vcard component.
/// This is also implemented by verified components
pub trait Component: Clone {
    const NAMES: &[&str];

    fn get_comp_name(&self) -> &'static str {
        assert_eq!(
            Self::NAMES.len(),
            1,
            "Default implementation only applicable for fixed component name"
        );
        Self::NAMES[0]
    }

    type Unverified: ComponentMut;

    fn get_properties(&self) -> &Vec<ContentLine>;
    fn mutable(self) -> Self::Unverified;

    fn get_property<'c>(&'c self, name: &str) -> Option<&'c ContentLine> {
        self.get_properties().iter().find(|p| p.name == name)
    }

    fn get_named_properties<'c>(&'c self, name: &str) -> Vec<&'c ContentLine> {
        self.get_properties()
            .iter()
            .filter(|p| p.name == name)
            .collect()
    }
}

/// A mutable interface for an Ical/Vcard component.
///
/// It takes a `PropertyParser` and fills the component with. It's also able to create
/// sub-component used by event and alarms.
pub trait ComponentMut: Component + Default {
    type Verified: Component<Unverified = Self>;

    /// Add the givent sub component.
    fn add_sub_component<B: BufRead>(
        &mut self,
        value: &str,
        line_parser: &mut PropertyParser<B>,
    ) -> Result<(), ParserError>;

    fn get_properties_mut(&mut self) -> &mut Vec<ContentLine>;

    /// Add the given property.
    fn add_content_line(&mut self, property: ContentLine) {
        self.get_properties_mut().push(property);
    }

    fn build(
        self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Self::Verified, ParserError>;

    /// Parse the content from `line_parser` and fill the component with.
    fn parse<B: BufRead>(
        &mut self,
        line_parser: &mut PropertyParser<B>,
    ) -> Result<(), ParserError> {
        loop {
            let line = line_parser.next().ok_or(ParserError::NotComplete)??;

            match line.name.as_ref() {
                "END" => break,
                "BEGIN" => match line.value {
                    Some(v) => self.add_sub_component(v.as_str(), line_parser)?,
                    None => return Err(ParserError::NotComplete),
                },

                _ => self.add_content_line(line),
            };
        }
        Ok(())
    }

    fn from_parser<B: BufRead>(line_parser: &mut PropertyParser<B>) -> Result<Self, ParserError> {
        let mut out = Self::default();
        out.parse(line_parser)?;
        Ok(out)
    }
}

/// Reader returning `IcalCalendar` object from a `BufRead`.
pub struct ComponentParser<B: BufRead, T: Component> {
    line_parser: PropertyParser<B>,
    _t: PhantomData<T>,
}

impl<B: BufRead, T: Component> ComponentParser<B, T> {
    /// Return a new `IcalParser` from a `Reader`.
    pub fn new(reader: B) -> ComponentParser<B, T> {
        let line_reader = LineReader::new(reader);
        let line_parser = PropertyParser::new(line_reader);

        ComponentParser {
            line_parser,
            _t: Default::default(),
        }
    }

    /// Read the next line and check if it's a valid VCALENDAR start.
    fn check_header(&mut self) -> Result<Option<()>, ParserError> {
        let line = match self.line_parser.next() {
            Some(val) => val.map_err(ParserError::PropertyError)?,
            None => return Ok(None),
        };

        if line.name.to_uppercase() != "BEGIN"
            || line.value.is_none()
            || !T::NAMES.contains(&line.value.as_ref().unwrap().to_uppercase().as_str())
            || !line.params.is_empty()
        {
            return Err(ParserError::MissingHeader);
        }

        Ok(Some(()))
    }

    pub fn expect_one(mut self) -> Result<<T::Unverified as ComponentMut>::Verified, ParserError> {
        let item = self.next().ok_or(ParserError::EmptyInput)??;
        if self.next().is_some() {
            return Err(ParserError::TooManyComponents);
        }
        Ok(item)
    }
}

impl<B: BufRead, T: Component> Iterator for ComponentParser<B, T> {
    type Item = Result<<T::Unverified as ComponentMut>::Verified, ParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.check_header() {
            Ok(res) => res?,
            Err(err) => return Some(Err(err)),
        };

        let mut comp = T::Unverified::default();
        let result = match comp.parse(&mut self.line_parser) {
            Ok(_) => comp.build(&HashMap::default()),
            Err(err) => Err(err),
        };

        #[cfg(feature = "test")]
        {
            // Run this for more test coverage
            if let Ok(comp) = result.as_ref() {
                let mutable = comp.clone().mutable();
                mutable.get_properties();
            }
        }

        Some(result)
    }
}
