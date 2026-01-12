use crate::{
    PropertyParser,
    parser::{
        Component, ComponentMut, GetProperty, IcalDTENDProperty, IcalDTSTAMPProperty,
        IcalDTSTARTProperty, IcalUIDProperty, ParserError,
    },
    property::ContentLine,
};
use std::{
    collections::{HashMap, HashSet},
    io::BufRead,
};

#[derive(Debug, Clone, Default)]
pub struct IcalFreeBusyBuilder {
    pub properties: Vec<ContentLine>,
}

#[derive(Debug, Clone)]
pub struct IcalFreeBusy {
    pub uid: String,
    pub dtstamp: IcalDTSTAMPProperty,
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
    #[inline]
    fn add_sub_component<B: BufRead>(
        &mut self,
        value: &str,
        _: &mut PropertyParser<B>,
    ) -> Result<(), ParserError> {
        Err(ParserError::InvalidComponent(value.to_owned()))
    }

    fn build(
        self,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<IcalFreeBusy, ParserError> {
        // REQUIRED, but NOT MORE THAN ONCE
        let IcalUIDProperty(uid, _) = self.safe_get_required(timezones)?;
        let dtstamp = self.safe_get_required(timezones)?;
        // OPTIONAL, but NOT MORE THAN ONCE: contact / dtstart / dtend / organizer / url /
        let _dtstart = self.safe_get_optional::<IcalDTSTARTProperty>(timezones)?;
        let _dtend = self.safe_get_optional::<IcalDTENDProperty>(timezones)?;
        // OPTIONAL, allowed multiple times: attendee / comment / freebusy / rstatus / x-prop / iana-prop

        Ok(IcalFreeBusy {
            uid,
            dtstamp,
            properties: self.properties,
        })
    }
}

impl IcalFreeBusy {
    pub fn get_tzids(&self) -> HashSet<&str> {
        self.properties
            .iter()
            .filter_map(|prop| prop.params.get_tzid())
            .collect()
    }
}
