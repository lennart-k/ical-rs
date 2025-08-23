use itertools::Itertools;

use crate::{
    PropertyParser,
    parser::{Component, ComponentMut, ParserError},
    property::Property,
};
use std::{cell::RefCell, io::BufRead};

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
pub struct IcalJournal<const VERIFIED: bool = true> {
    pub properties: Vec<Property>,
}

impl IcalJournal<false> {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }
}

impl IcalJournal<true> {
    pub fn get_uid(&self) -> &str {
        self.get_property("UID")
            .and_then(|prop| prop.value.as_deref())
            .expect("already verified that this must exist")
    }

    pub fn get_dtstamp(&self) -> &str {
        self.get_property("DTSTAMP")
            .and_then(|prop| prop.value.as_deref())
            .expect("already verified that this must exist")
    }

    pub fn get_dtstart(&self) -> Option<&Property> {
        self.get_property("DTSTART")
    }
}

impl<const VERIFIED: bool> Component for IcalJournal<VERIFIED> {
    const NAMES: &[&str] = &["VJOURNAL"];
    type Unverified = IcalJournal<false>;

    fn get_properties(&self) -> &Vec<Property> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalJournal {
            properties: self.properties,
        }
    }
}

impl ComponentMut for IcalJournal<false> {
    type Verified = IcalJournal<true>;

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

    fn verify(self) -> Result<IcalJournal<true>, ParserError> {
        if self
            .get_property("UID")
            .and_then(|prop| prop.value.as_ref())
            .is_none()
        {
            return Err(ParserError::MissingProperty("UID"));
        }

        if self
            .get_property("DTSTAMP")
            .and_then(|prop| prop.value.as_ref())
            .is_none()
        {
            return Err(ParserError::MissingProperty("DTSTAMP"));
        }

        let verified = IcalJournal {
            properties: self.properties,
        };

        #[cfg(feature = "test")]
        {
            // Verify that the conditions for our getters are actually met
            verified.get_uid();
            verified.get_dtstamp();
            verified.get_dtstart();
        }

        Ok(verified)
    }
}

impl<const VERIFIED: bool> IcalJournal<VERIFIED> {
    pub fn get_tzids(&self) -> Vec<&str> {
        self.properties
            .iter()
            .filter_map(|prop| prop.get_tzid())
            .unique()
            .collect()
    }
}
