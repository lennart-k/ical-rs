use crate::{
    PropertyParser,
    parser::{Component, ComponentMut, ParserError, ical::component::IcalAlarm},
    property::Property,
};
use std::{cell::RefCell, io::BufRead};

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
pub struct IcalTodo<const VERIFIED: bool = true> {
    pub properties: Vec<Property>,
    pub alarms: Vec<IcalAlarm>,
}

impl IcalTodo<false> {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
            alarms: Vec::new(),
        }
    }
}

impl IcalTodo<true> {
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

    pub fn get_due(&self) -> Option<&Property> {
        self.get_property("DUE")
    }

    pub fn get_duration(&self) -> Option<&Property> {
        self.get_property("DURATION")
    }

    pub fn get_rrule(&self) -> Option<&Property> {
        self.get_property("RRULE")
    }
}

impl<const VERIFIED: bool> Component for IcalTodo<VERIFIED> {
    const NAMES: &[&str] = &["VTODO"];
    type Unverified = IcalTodo<false>;

    fn get_properties(&self) -> &Vec<Property> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalTodo {
            properties: self.properties,
            alarms: self.alarms,
        }
    }
}

impl ComponentMut for IcalTodo<false> {
    type Verified = IcalTodo<true>;

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

    fn verify(self) -> Result<IcalTodo<true>, ParserError> {
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

        if self.get_property("DUE").is_some() && self.get_property("DURATION").is_some() {
            return Err(ParserError::PropertyConflict(
                "both DUE and DURATION are defined",
            ));
        }

        Ok(IcalTodo {
            properties: self.properties,
            alarms: self.alarms,
        })
    }
}
