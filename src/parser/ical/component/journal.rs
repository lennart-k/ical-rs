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
        Ok(IcalJournal {
            properties: self.properties,
        })
    }
}
