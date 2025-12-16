//! Wrapper around `PropertyParser`
//!
//! #### Warning
//!   The parsers (`VcardParser` / `IcalParser`) only parse the content and set to uppercase
//!   the case-insensitive fields.  No checks are made on the fields validity.
//!
//!

pub mod ical;
pub mod vcard;

// Sys mods
use std::io::BufRead;
use std::{cell::RefCell, marker::PhantomData};

use crate::types::InvalidDuration;
// Internal mods
use crate::{
    LineReader,
    property::{Property, PropertyError, PropertyParser},
};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ParserError {
    #[error("invalid component")]
    InvalidComponent,
    #[error("incomplete object")]
    NotComplete,
    #[error("missing header")]
    MissingHeader,
    #[error("property error: {0}")]
    PropertyError(#[from] PropertyError),
    #[error("missing property: {0}")]
    MissingProperty(&'static str),
    #[error("property conflict: {0}")]
    PropertyConflict(&'static str),
    #[error(transparent)]
    InvalidDuration(#[from] InvalidDuration),
}

/// An immutable interface for an Ical/Vcard component.
/// This is also implemented by verified components
pub trait Component: Clone {
    const NAMES: &[&str];

    type Unverified: ComponentMut;

    fn get_properties(&self) -> &Vec<Property>;
    fn mutable(self) -> Self::Unverified;

    fn get_property<'c>(&'c self, name: &str) -> Option<&'c Property> {
        self.get_properties().iter().find(|p| p.name == name)
    }
}

/// A mutable interface for an Ical/Vcard component.
///
/// It take a `PropertyParser` and fill the component with. It's also able to create
/// sub-component used by event and alarms.
pub trait ComponentMut: Component + Default {
    type Verified: Component;

    /// Add the givent sub component.
    fn add_sub_component<B: BufRead>(
        &mut self,
        value: &str,
        line_parser: &RefCell<PropertyParser<B>>,
    ) -> Result<(), ParserError>;

    fn get_properties_mut(&mut self) -> &mut Vec<Property>;

    /// Add the givent property.
    fn add_property(&mut self, property: Property) {
        self.get_properties_mut().push(property);
    }

    fn get_property_mut<'c>(&'c mut self, name: &str) -> Option<&'c mut Property> {
        self.get_properties_mut()
            .iter_mut()
            .find(|p| p.name == name)
    }

    fn remove_property(&mut self, name: &str) {
        self.get_properties_mut().retain(|prop| prop.name != name);
    }
    fn set_property(&mut self, prop: Property) {
        self.remove_property(&prop.name);
        self.add_property(prop);
    }

    fn verify(self) -> Result<Self::Verified, ParserError>;

    /// Parse the content from `line_parser` and fill the component with.
    fn parse<B: BufRead>(
        &mut self,
        line_parser: &RefCell<PropertyParser<B>>,
    ) -> Result<(), ParserError> {
        loop {
            let line: Property;

            {
                line = match line_parser.borrow_mut().next() {
                    Some(val) => val.map_err(ParserError::PropertyError)?,
                    None => return Err(ParserError::NotComplete),
                };
            }

            match line.name.to_uppercase().as_str() {
                "END" => break,
                "BEGIN" => match line.value {
                    Some(v) => self.add_sub_component(v.as_str(), line_parser)?,
                    None => return Err(ParserError::NotComplete),
                },

                _ => self.add_property(line),
            };
        }
        Ok(())
    }
}

/// Reader returning `IcalCalendar` object from a `BufRead`.
pub struct ComponentParser<B, T: Component> {
    line_parser: RefCell<PropertyParser<B>>,
    _t: PhantomData<T>,
}

impl<B: BufRead, T: Component> ComponentParser<B, T> {
    /// Return a new `IcalParser` from a `Reader`.
    pub fn new(reader: B) -> ComponentParser<B, T> {
        let line_reader = LineReader::new(reader);
        let line_parser = PropertyParser::new(line_reader);

        ComponentParser {
            line_parser: RefCell::new(line_parser),
            _t: Default::default(),
        }
    }

    /// Read the next line and check if it's a valid VCALENDAR start.
    fn check_header(&mut self) -> Result<Option<()>, ParserError> {
        let line = match self.line_parser.borrow_mut().next() {
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
}

impl<B: BufRead, T: Component> Iterator for ComponentParser<B, T> {
    type Item = Result<<T::Unverified as ComponentMut>::Verified, ParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.check_header() {
            Ok(res) => res?,
            Err(err) => return Some(Err(err)),
        };

        let mut comp = T::Unverified::default();
        let result = match comp.parse(&self.line_parser) {
            Ok(_) => comp.verify(),
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
