use crate::parser::{Component, ComponentMut, ParserError};
use crate::property::{ContentLine, PropertyParser};
use std::collections::HashMap;
use std::io::BufRead;

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub struct VcardContact<const VERIFIED: bool = true> {
    pub properties: Vec<ContentLine>,
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

    fn get_properties(&self) -> &Vec<ContentLine> {
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

    fn get_properties_mut(&mut self) -> &mut Vec<ContentLine> {
        &mut self.properties
    }

    fn add_sub_component<B: BufRead>(
        &mut self,
        name: &str,
        _: &mut PropertyParser<B>,
    ) -> Result<(), ParserError> {
        Err(ParserError::InvalidComponent(name.to_owned()))
    }

    fn build(
        self,
        _timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Self::Verified, ParserError> {
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
