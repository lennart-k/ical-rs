use crate::parser::{Component, ComponentMut, ParserError};
use crate::property::{Property, PropertyParser};
use std::io::BufRead;

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
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
    pub fn get_uid(&self) -> Option<&str> {
        self.get_property("UID")
            .and_then(|prop| prop.value.as_deref())
    }
}

impl<const VERIFIED: bool> Component for VcardContact<VERIFIED> {
    const NAMES: &[&str] = &["VCARD"];
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
        _: &mut PropertyParser<B>,
    ) -> Result<(), ParserError> {
        Err(ParserError::InvalidComponent)
    }

    fn verify(self) -> Result<Self::Verified, ParserError> {
        let verified = VcardContact {
            properties: self.properties,
        };

        #[cfg(feature = "test")]
        {
            verified.get_uid();
        }

        Ok(verified)
    }
}
