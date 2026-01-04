use itertools::Itertools;

use crate::{
    PropertyParser,
    parser::{Component, ComponentMut, ParserError},
    property::ContentLine,
};
use std::{collections::HashMap, io::BufRead};

#[derive(Debug, Clone, Default)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct IcalAlarm<const VERIFIED: bool = true> {
    pub properties: Vec<ContentLine>,
}

impl IcalAlarm<false> {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }
}

impl<const VERIFIED: bool> Component for IcalAlarm<VERIFIED> {
    const NAMES: &[&str] = &["VALARM"];
    type Unverified = IcalAlarm<false>;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalAlarm {
            properties: self.properties,
        }
    }
}

impl ComponentMut for IcalAlarm<false> {
    type Verified = IcalAlarm<true>;

    fn get_properties_mut(&mut self) -> &mut Vec<ContentLine> {
        &mut self.properties
    }

    #[cfg(not(tarpaulin_include))]
    fn add_sub_component<B: BufRead>(
        &mut self,
        value: &str,
        _: &mut PropertyParser<B>,
    ) -> Result<(), ParserError> {
        Err(ParserError::InvalidComponent(value.to_owned()))
    }

    fn build(
        self,
        _timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<IcalAlarm<true>, ParserError> {
        Ok(IcalAlarm {
            properties: self.properties,
        })
    }
}

impl<const VERIFIED: bool> IcalAlarm<VERIFIED> {
    pub fn get_tzids(&self) -> Vec<&str> {
        self.properties
            .iter()
            .filter_map(|prop| prop.get_tzid())
            .unique()
            .collect()
    }
}
