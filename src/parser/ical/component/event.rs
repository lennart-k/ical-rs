use itertools::Itertools;

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

impl IcalEvent<true> {
    pub fn get_uid(&self) -> &str {
        self.get_property("UID")
            .and_then(|prop| prop.value.as_deref())
            .expect("already verified that this must exist")
    }

    pub fn get_recurrence_id(&self) -> Option<&Property> {
        self.get_property("RECURRENCE-ID")
    }

    // pub fn get_dtstamp(&self) -> &str {
    //     self.get_property("DTSTAMP")
    //         .and_then(|prop| prop.value.as_deref())
    //         .expect("already verified that this must exist")
    // }

    pub fn get_dtstart(&self) -> Option<&Property> {
        self.get_property("DTSTART")
    }

    pub fn get_dtend(&self) -> Option<&Property> {
        self.get_property("DTEND")
    }

    #[cfg(feature = "chrono")]
    pub fn get_duration(&self) -> Option<chrono::Duration> {
        self.get_property("DURATION")
            .and_then(|prop| Option::<chrono::Duration>::try_from(prop).unwrap())
    }

    pub fn get_rrule(&self) -> Option<&Property> {
        self.get_property("RRULE")
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
        if self
            .get_property("UID")
            .and_then(|prop| prop.value.as_ref())
            .is_none()
        {
            return Err(ParserError::MissingProperty("UID"));
        }

        // if self
        //     .get_property("DTSTAMP")
        //     .and_then(|prop| prop.value.as_ref())
        //     .is_none()
        // {
        //     return Err(ParserError::MissingProperty("DTSTAMP"));
        // }

        if self.get_property("METHOD").is_none()
            && self
                .get_property("DTSTART")
                .and_then(|prop| prop.value.as_ref())
                .is_none()
        {
            return Err(ParserError::MissingProperty("DTSTART"));
        }

        if self.get_property("DTEND").is_some() && self.get_property("DURATION").is_some() {
            return Err(ParserError::PropertyConflict(
                "both DTEND and DURATION are defined",
            ));
        }

        #[cfg(feature = "chrono")]
        if let Some(prop) = self.get_property("DURATION") {
            Option::<chrono::Duration>::try_from(prop)?;
        }

        let verified = IcalEvent {
            properties: self.properties,
            alarms: self.alarms,
        };

        #[cfg(feature = "test")]
        {
            // Verify that the conditions for our getters are actually met
            verified.get_uid();
            verified.get_recurrence_id();
            // verified.get_dtstamp();
            verified.get_dtstart();
            verified.get_dtend();
            #[cfg(feature = "chrono")]
            verified.get_duration();
            verified.get_rrule();
        }

        Ok(verified)
    }
}

impl<const VERIFIED: bool> IcalEvent<VERIFIED> {
    pub fn get_tzids(&self) -> Vec<&str> {
        self.properties
            .iter()
            .filter_map(|prop| prop.get_tzid())
            .chain(self.alarms.iter().flat_map(|alarm| alarm.get_tzids()))
            .unique()
            .collect()
    }
}
