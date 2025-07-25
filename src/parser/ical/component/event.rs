use crate::{
    PropertyParser,
    parser::{Component, ComponentMut, ParserError, ical::component::IcalAlarm},
    property::Property,
};
use std::{cell::RefCell, io::BufRead};

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
pub struct IcalEvent<const VERIFIED: bool = true> {
    pub properties: Vec<Property>,
    pub alarms: Vec<IcalAlarm>,
}

impl IcalEvent<false> {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
            alarms: Vec::new(),
        }
    }
}

impl<const VERIFIED: bool> Component for IcalEvent<VERIFIED> {
    const NAMES: &[&str] = &["VEVENT"];
    type Unverified = IcalEvent<false>;

    fn get_properties(&self) -> &Vec<Property> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalEvent {
            properties: self.properties,
            alarms: self.alarms,
        }
    }
}

impl ComponentMut for IcalEvent<false> {
    type Verified = IcalEvent<true>;

    fn get_properties_mut(&mut self) -> &mut Vec<Property> {
        &mut self.properties
    }

    fn add_sub_component<B: BufRead>(
        &mut self,
        value: &str,
        line_parser: &RefCell<PropertyParser<B>>,
    ) -> Result<(), ParserError> {
        match value {
            "VALARM" => {
                let mut alarm = IcalAlarm::new();
                alarm.parse(line_parser)?;
                self.alarms.push(alarm.verify()?);
            }
            _ => return Err(ParserError::InvalidComponent),
        };

        Ok(())
    }

    fn verify(self) -> Result<IcalEvent<true>, ParserError> {
        Ok(IcalEvent {
            properties: self.properties,
            alarms: self.alarms,
        })
    }
}
