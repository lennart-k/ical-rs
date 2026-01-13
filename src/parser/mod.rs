//! Wrapper around `PropertyParser`
//!
//! #### Warning
//!   The parsers (`VcardParser` / `IcalParser`) only parse the content and set to uppercase
//!   the case-insensitive fields.  No checks are made on the fields validity.
//!
//!
pub mod ical;
pub mod vcard;
use crate::line::BytesLines;
use crate::types::{CalDateTimeError, InvalidDuration};
use crate::{
    LineReader,
    property::{ContentLine, PropertyError, PropertyParser},
};
use std::borrow::Cow;
use std::collections::HashMap;
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
    #[error("invalid property value: {0}")]
    InvalidPropertyValue(String),
    #[error("invalid property value type for: {0}")]
    InvalidPropertyType(String),
    #[error(transparent)]
    RRule(#[from] rrule::RRuleError),
    #[error(transparent)]
    DateTime(#[from] CalDateTimeError),
    #[error("Invalid CALSCALE: Only GREGORIAN supported")]
    InvalidCalscale,
    #[error("Invalid VERSION: MUST be 1.0 or 2.0")]
    InvalidVersion,
    #[error("Multiple main events are not allowed in a calendar object")]
    MultipleMainObjects,
    #[error("Differing UIDs inside a calendar object")]
    DifferingUIDs,
    #[error("Override without RECURRENCE-ID")]
    MissingRecurId,
    #[error("DTSTART and RECURRENCE-ID must have the same value type and timezone")]
    DtstartNotMatchingRecurId,
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

    fn get_named_properties<'c>(&'c self, name: &'c str) -> impl Iterator<Item = &'c ContentLine> {
        self.get_properties().iter().filter(move |p| p.name == name)
    }
}

/// A mutable interface for an Ical/Vcard component.
///
/// It takes a `PropertyParser` and fills the component with. It's also able to create
/// sub-component used by event and alarms.
pub trait ComponentMut: Component + Default {
    type Verified: Component<Unverified = Self>;

    /// Add the givent sub component.
    fn add_sub_component<'a, T: Iterator<Item = Cow<'a, [u8]>>>(
        &mut self,
        value: &str,
        line_parser: &mut PropertyParser<'a, T>,
    ) -> Result<(), ParserError>;

    fn get_properties_mut(&mut self) -> &mut Vec<ContentLine>;

    fn remove_property(&mut self, name: &str) {
        self.get_properties_mut().retain(|prop| prop.name != name);
    }

    /// Add the given property.
    #[inline]
    fn add_content_line(&mut self, property: ContentLine) {
        self.get_properties_mut().push(property);
    }

    fn build(
        self,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Self::Verified, ParserError>;

    /// Parse the content from `line_parser` and fill the component with.
    fn parse<'a, T: Iterator<Item = Cow<'a, [u8]>>>(
        &mut self,
        line_parser: &mut PropertyParser<'a, T>,
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

    fn from_parser<'a, T: Iterator<Item = Cow<'a, [u8]>>>(
        line_parser: &mut PropertyParser<'a, T>,
    ) -> Result<Self, ParserError> {
        let mut out = Self::default();
        out.parse(line_parser)?;
        Ok(out)
    }
}

pub struct ComponentParser<'a, C: Component, I: Iterator<Item = Cow<'a, [u8]>>> {
    line_parser: PropertyParser<'a, I>,
    _t: PhantomData<C>,
}

impl<'a, C: Component> ComponentParser<'a, C, BytesLines<'a>> {
    /// Return a new `IcalParser` from a `Reader`.
    pub fn from_slice(slice: &'a [u8]) -> Self {
        let line_reader = LineReader::from_slice(slice);
        let line_parser = PropertyParser::new(line_reader);

        ComponentParser {
            line_parser,
            _t: Default::default(),
        }
    }
}

impl<'a, C: Component, I: Iterator<Item = Cow<'a, [u8]>>> ComponentParser<'a, C, I> {
    /// Read the next line and check if it's a valid VCALENDAR start.
    #[inline]
    fn check_header(&mut self) -> Result<Option<()>, ParserError> {
        let line = match self.line_parser.next() {
            Some(val) => val.map_err(ParserError::PropertyError)?,
            None => return Ok(None),
        };

        if line.name != "BEGIN"
            || line.value.is_none()
            || !C::NAMES.contains(&line.value.as_ref().unwrap().to_uppercase().as_str())
            || !line.params.is_empty()
        {
            return Err(ParserError::MissingHeader);
        }

        Ok(Some(()))
    }

    pub fn expect_one(mut self) -> Result<<C::Unverified as ComponentMut>::Verified, ParserError> {
        let item = self.next().ok_or(ParserError::EmptyInput)??;
        if self.next().is_some() {
            return Err(ParserError::TooManyComponents);
        }
        Ok(item)
    }
}

impl<'a, C: Component, I: Iterator<Item = Cow<'a, [u8]>>> Iterator for ComponentParser<'a, C, I> {
    type Item = Result<<C::Unverified as ComponentMut>::Verified, ParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.check_header() {
            Ok(res) => res?,
            Err(err) => return Some(Err(err)),
        };

        let mut comp = C::Unverified::default();
        let result = match comp.parse(&mut self.line_parser) {
            Ok(_) => comp.build(None),
            Err(err) => Err(err),
        };

        #[cfg(all(feature = "test", not(feature = "bench")))]
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
