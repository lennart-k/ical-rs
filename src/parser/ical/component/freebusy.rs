use crate::{
    PropertyParser,
    parser::{Component, ComponentMut, ParserError},
    property::ContentLine,
};
use itertools::Itertools;
use std::{collections::HashMap, io::BufRead};

#[derive(Debug, Clone, Default)]
pub struct IcalFreeBusyBuilder {
    pub properties: Vec<ContentLine>,
}

#[derive(Debug, Clone, Default)]
pub struct IcalFreeBusy {
    pub properties: Vec<ContentLine>,
}

impl IcalFreeBusyBuilder {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }
}

impl Component for IcalFreeBusyBuilder {
    const NAMES: &[&str] = &["VFREEBUSY"];
    type Unverified = IcalFreeBusyBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        self
    }
}

impl Component for IcalFreeBusy {
    const NAMES: &[&str] = &["VFREEBUSY"];
    type Unverified = IcalFreeBusyBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalFreeBusyBuilder {
            properties: self.properties,
        }
    }
}

impl ComponentMut for IcalFreeBusyBuilder {
    type Verified = IcalFreeBusy;

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
    ) -> Result<IcalFreeBusy, ParserError> {
        Ok(IcalFreeBusy {
            properties: self.properties,
        })
    }
}

impl IcalFreeBusy {
    pub fn get_tzids(&self) -> Vec<&str> {
        self.properties
            .iter()
            .filter_map(|prop| prop.get_tzid())
            .unique()
            .collect()
    }
}
