use crate::{
    PropertyParser,
    parser::{Component, ComponentMut, ParserError, ical::component::IcalAlarm},
    property::Property,
};
use itertools::Itertools;
use std::io::BufRead;

#[derive(Debug, Clone, Default)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
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

    pub fn get_recurrence_id(&self) -> Option<&Property> {
        self.get_property("RECURRENCE-ID")
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

    #[cfg(feature = "chrono")]
    pub fn get_duration(&self) -> Option<chrono::Duration> {
        self.get_property("DURATION")
            .and_then(|prop| Option::<chrono::Duration>::try_from(prop).unwrap())
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
        line_parser: &mut PropertyParser<B>,
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

        #[cfg(feature = "chrono")]
        if let Some(prop) = self.get_property("DURATION") {
            Option::<chrono::Duration>::try_from(prop)?;
        }

        let verified = IcalTodo {
            properties: self.properties,
            alarms: self.alarms,
        };

        #[cfg(feature = "test")]
        {
            // Verify that the conditions for our getters are actually met
            verified.get_uid();
            verified.get_recurrence_id();
            verified.get_dtstamp();
            verified.get_dtstart();
            verified.get_due();
            #[cfg(feature = "chrono")]
            verified.get_duration();
            verified.get_rrule();
        }

        Ok(verified)
    }
}

impl<const VERIFIED: bool> IcalTodo<VERIFIED> {
    pub fn get_tzids(&self) -> Vec<&str> {
        self.properties
            .iter()
            .filter_map(|prop| prop.get_tzid())
            .unique()
            .collect()
    }
}
