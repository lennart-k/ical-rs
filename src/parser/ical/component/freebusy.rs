use itertools::Itertools;

use crate::{
    PropertyParser,
    parser::{Component, ComponentMut, ParserError},
    property::Property,
};
use std::io::BufRead;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
pub struct IcalFreeBusy<const VERIFIED: bool = true> {
    pub properties: Vec<Property>,
}

impl IcalFreeBusy<false> {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }
}

impl<const VERIFIED: bool> Component for IcalFreeBusy<VERIFIED> {
    const NAMES: &[&str] = &["VFREEBUSY"];
    type Unverified = IcalFreeBusy<false>;

    fn get_properties(&self) -> &Vec<Property> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalFreeBusy {
            properties: self.properties,
        }
    }
}

impl ComponentMut for IcalFreeBusy<false> {
    type Verified = IcalFreeBusy<true>;

    fn get_properties_mut(&mut self) -> &mut Vec<Property> {
        &mut self.properties
    }

    #[cfg(not(tarpaulin_include))]
    fn add_sub_component<B: BufRead>(
        &mut self,
        _: &str,
        _: &mut PropertyParser<B>,
    ) -> Result<(), ParserError> {
        Err(ParserError::InvalidComponent)
    }

    fn verify(self) -> Result<IcalFreeBusy<true>, ParserError> {
        Ok(IcalFreeBusy {
            properties: self.properties,
        })
    }
}

impl<const VERIFIED: bool> IcalFreeBusy<VERIFIED> {
    pub fn get_tzids(&self) -> Vec<&str> {
        self.properties
            .iter()
            .filter_map(|prop| prop.get_tzid())
            .unique()
            .collect()
    }
}
