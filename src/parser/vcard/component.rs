// Sys mods
use std::cell::RefCell;
use std::io::BufRead;

#[cfg(feature = "serde-derive")]
extern crate serde;

// Internal mods
use crate::parser::{Component, ComponentMut, ParserError};
use crate::property::{Property, PropertyParser};

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
/// A VCARD contact.
pub struct VcardContact<const VERIFIED: bool = true> {
    pub properties: Vec<Property>,
}

impl VcardContact<false> {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }
}

impl VcardContact<true> {
    pub fn get_uid(&self) -> &str {
        self.get_property("UID")
            .and_then(|prop| prop.value.as_ref())
            .expect("we already verified this exists")
    }
}

impl<const VERIFIED: bool> Component for VcardContact<VERIFIED> {
    type Unverified = VcardContact<false>;

    fn get_properties(&self) -> &Vec<Property> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        VcardContact {
            properties: self.properties,
        }
    }
}

impl ComponentMut for VcardContact<false> {
    type Verified = VcardContact<true>;

    fn get_properties_mut(&mut self) -> &mut Vec<Property> {
        &mut self.properties
    }

    fn add_sub_component<B: BufRead>(
        &mut self,
        _: &str,
        _: &RefCell<PropertyParser<B>>,
    ) -> Result<(), ParserError> {
        Err(ParserError::InvalidComponent)
    }

    fn verify(self) -> Result<Self::Verified, ParserError> {
        if self
            .get_property("UID")
            .and_then(|prop| prop.value.as_ref())
            .is_none()
        {
            return Err(ParserError::MissingProperty("UID"));
        }

        Ok(VcardContact {
            properties: self.properties,
        })
    }
}
