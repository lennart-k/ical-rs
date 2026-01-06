use crate::{
    PropertyParser,
    parser::{
        Component, ComponentMut, GetProperty, IcalDTSTAMPProperty, IcalDTSTARTProperty,
        IcalRECURIDProperty, IcalUIDProperty, ParserError,
    },
    property::ContentLine,
};
use std::{
    collections::{HashMap, HashSet},
    io::BufRead,
};

#[derive(Debug, Clone, Default)]
pub struct IcalJournalBuilder {
    pub properties: Vec<ContentLine>,
}

#[derive(Debug, Clone)]
pub struct IcalJournal {
    uid: String,
    pub dtstamp: IcalDTSTAMPProperty,
    pub properties: Vec<ContentLine>,
}

impl IcalJournalBuilder {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }
}

impl IcalJournal {
    pub fn get_uid(&self) -> &str {
        &self.uid
    }
}

impl Component for IcalJournalBuilder {
    const NAMES: &[&str] = &["VJOURNAL"];
    type Unverified = IcalJournalBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        self
    }
}

impl Component for IcalJournal {
    const NAMES: &[&str] = &["VJOURNAL"];
    type Unverified = IcalJournalBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalJournalBuilder {
            properties: self.properties,
        }
    }
}

impl ComponentMut for IcalJournalBuilder {
    type Verified = IcalJournal;

    fn get_properties_mut(&mut self) -> &mut Vec<ContentLine> {
        &mut self.properties
    }

    fn add_sub_component<B: BufRead>(
        &mut self,
        value: &str,
        _: &mut PropertyParser<B>,
    ) -> Result<(), ParserError> {
        Err(ParserError::InvalidComponent(value.to_owned()))
    }

    fn build(
        self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<IcalJournal, ParserError> {
        // REQUIRED, ONLY ONCE
        let IcalUIDProperty(uid, _) = self.safe_get_required(timezones)?;
        let dtstamp = self.safe_get_required(timezones)?;

        // OPTIONAL, ONLY ONCE: class / created / dtstart / last-mod / organizer / recurid / seq / status / summary / url / rrule
        let dtstart = self.safe_get_optional::<IcalDTSTARTProperty>(timezones)?;
        let recurid = self.safe_get_optional::<IcalRECURIDProperty>(timezones)?;
        if let Some(IcalDTSTARTProperty(dtstart, _)) = &dtstart
            && let Some(recurid) = &recurid
        {
            recurid.validate_dtstart(dtstart)?;
        }

        // OPTIONAL, MULTIPLE ALLOWED: attach / attendee / categories / comment / contact / description / exdate / related / rdate / rstatus / x-prop / iana-prop
        let verified = IcalJournal {
            uid,
            dtstamp,
            properties: self.properties,
        };
        Ok(verified)
    }
}

impl IcalJournal {
    pub fn get_tzids(&self) -> HashSet<&str> {
        self.properties
            .iter()
            .filter_map(|prop| prop.params.get_tzid())
            .collect()
    }
}
