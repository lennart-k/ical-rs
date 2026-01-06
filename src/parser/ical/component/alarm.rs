use crate::{
    PropertyParser,
    parser::{Component, ComponentMut, ParserError},
    property::ContentLine,
};
use std::{
    collections::{HashMap, HashSet},
    io::BufRead,
};

#[derive(Debug, Clone, Default)]
pub struct IcalAlarmBuilder {
    pub properties: Vec<ContentLine>,
}

#[derive(Debug, Clone)]
pub struct IcalAlarm {
    pub properties: Vec<ContentLine>,
}

impl IcalAlarmBuilder {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }
}

impl Component for IcalAlarmBuilder {
    const NAMES: &[&str] = &["VALARM"];
    type Unverified = IcalAlarmBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        self
    }
}

impl Component for IcalAlarm {
    const NAMES: &[&str] = &["VALARM"];
    type Unverified = IcalAlarmBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalAlarmBuilder {
            properties: self.properties,
        }
    }
}

impl ComponentMut for IcalAlarmBuilder {
    type Verified = IcalAlarm;

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
    ) -> Result<IcalAlarm, ParserError> {
        Ok(IcalAlarm {
            properties: self.properties,
        })
    }
}

impl IcalAlarm {
    pub fn get_tzids(&self) -> HashSet<&str> {
        self.properties
            .iter()
            .filter_map(|prop| prop.params.get_tzid())
            .collect()
    }
}
