use crate::{
    PropertyParser,
    parser::{
        Component, ComponentMut, GetProperty, IcalDTSTAMPProperty, IcalDTSTARTProperty,
        IcalEXDATEProperty, IcalEXRULEProperty, IcalRDATEProperty, IcalRECURIDProperty,
        IcalRRULEProperty, IcalUIDProperty, ParserError,
    },
    property::ContentLine,
};
use rrule::RRule;
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
    pub dtstart: Option<IcalDTSTARTProperty>,
    pub properties: Vec<ContentLine>,
    rdates: Vec<IcalRDATEProperty>,
    rrules: Vec<RRule>,
    exdates: Vec<IcalEXDATEProperty>,
    exrules: Vec<RRule>,
    pub(crate) recurid: Option<IcalRECURIDProperty>,
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

    pub fn has_rruleset(&self) -> bool {
        !self.rrules.is_empty()
            || !self.rdates.is_empty()
            || !self.exrules.is_empty()
            || !self.exdates.is_empty()
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
        let rdates = self.safe_get_all::<IcalRDATEProperty>(timezones)?;
        let exdates = self.safe_get_all::<IcalEXDATEProperty>(timezones)?;
        let (rrules, exrules) = if let Some(dtstart) = dtstart.as_ref() {
            let rrule_dtstart = dtstart.0.utc().with_timezone(&rrule::Tz::UTC);
            let rrules = self
                .safe_get_all::<IcalRRULEProperty>(timezones)?
                .into_iter()
                .map(|rrule| rrule.0.validate(rrule_dtstart))
                .collect::<Result<Vec<_>, _>>()?;
            let exrules = self
                .safe_get_all::<IcalEXRULEProperty>(timezones)?
                .into_iter()
                .map(|rrule| rrule.0.validate(rrule_dtstart))
                .collect::<Result<Vec<_>, _>>()?;
            (rrules, exrules)
        } else {
            (vec![], vec![])
        };

        let verified = IcalJournal {
            uid,
            dtstamp,
            dtstart,
            rdates,
            rrules,
            exdates,
            exrules,
            recurid,
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
