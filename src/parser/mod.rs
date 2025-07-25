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
use std::cell::RefCell;
use std::io::BufRead;

// Internal mods
use crate::property::{Property, PropertyError, PropertyParser};

#[derive(Debug, Error)]
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
}

/// An immutable interface for an Ical/Vcard component.
/// This is also implemented by verified components
pub trait Component {
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
pub trait ComponentMut: Component {
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
