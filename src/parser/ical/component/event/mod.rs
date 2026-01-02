use crate::{
    PropertyParser,
    parser::{Component, ComponentMut, ParserError, ical::component::IcalAlarm},
    property::Property,
    types::{CalDate, CalDateOrDateTime, CalDateTime, CalDateTimeError},
};
use chrono::{DateTime, Duration, Utc};
use itertools::Itertools;
use std::{collections::HashMap, io::BufRead};

mod fallible;
mod properties;

#[derive(Debug, Clone, Default)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
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
    pub fn get_recurrence_id(&self) -> Option<&Property> {
        self.get_property("RECURRENCE-ID")
    }

    pub fn get_last_occurence(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Option<CalDateOrDateTime>, CalDateTimeError> {
        if !self.get_rrule(timezones).is_empty() {
            // TODO: understand recurrence rules
            return Ok(None);
        }

        if let Some(dtend) = self.get_dtend(timezones) {
            return Ok(Some(dtend));
        }

        let duration = self.get_duration().unwrap_or(Duration::days(1));

        let first_occurence = self.get_dtstart(timezones);
        Ok(first_occurence.map(|first_occurence| (first_occurence + duration).into()))
    }

    pub fn expand_recurrence(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
        overrides: &[Self],
    ) -> Result<Vec<Self>, ParserError> {
        let Some(mut rrule_set) = self.get_rrule_set(timezones) else {
            return Ok(vec![self.clone()]);
        };

        if let Some(start) = start {
            rrule_set = rrule_set.after(start.with_timezone(&rrule::Tz::UTC));
        }
        if let Some(end) = end {
            rrule_set = rrule_set.before(end.with_timezone(&rrule::Tz::UTC));
        }
        let mut events = vec![];
        let dates = rrule_set.all(2048).dates;
        let dtstart = self
            .get_dtstart(timezones)
            .expect("We must have a DTSTART here");
        let computed_duration = self.get_dtend(timezones).map(|dtend| dtend - &dtstart);

        'recurrence: for date in dates {
            let date = CalDateTime::from(date);
            let dateformat = if dtstart.is_date() {
                CalDate(date.date_floor(), date.timezone()).format()
            } else {
                date.format()
            };

            for ev_override in overrides {
                if let Some(override_id) = &ev_override
                    .get_recurrence_id()
                    .as_ref()
                    .expect("overrides have a recurrence id")
                    .value
                    && override_id == &dateformat
                {
                    // We have an override for this occurence
                    //
                    events.push(ev_override.clone());
                    continue 'recurrence;
                }
            }

            let mut ev = self.clone().mutable();
            ev.remove_property("RRULE");
            ev.remove_property("RDATE");
            ev.remove_property("EXDATE");
            ev.remove_property("EXRULE");
            let dtstart_prop = ev
                .get_property("DTSTART")
                .expect("We must have a DTSTART here")
                .clone();
            ev.remove_property("DTSTART");
            ev.remove_property("DTEND");

            ev.set_property(Property {
                name: "RECURRENCE-ID".to_string(),
                value: Some(dateformat.clone()),
                params: vec![],
            });
            ev.set_property(Property {
                name: "DTSTART".to_string(),
                value: Some(dateformat),
                params: dtstart_prop.params.clone(),
            });
            if let Some(duration) = computed_duration {
                let dtend = date + duration;
                let dtendformat = if dtstart.is_date() {
                    CalDate(dtend.date_floor(), dtend.timezone()).format()
                } else {
                    dtend.format()
                };
                ev.set_property(Property {
                    name: "DTEND".to_string(),
                    value: Some(dtendformat),
                    params: dtstart_prop.params,
                });
            }
            // TODO: Remove unwrap
            events.push(ev.verify().unwrap());
        }
        Ok(events)
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
        line_parser: &mut PropertyParser<B>,
    ) -> Result<(), ParserError> {
        match value {
            "VALARM" => {
                let mut alarm = IcalAlarm::new();
                alarm.parse(line_parser)?;
                self.alarms.push(alarm.verify()?);
            }
            _ => return Err(ParserError::InvalidComponent(value.to_owned())),
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
            // verified.get_dtstart(&HashMap::new()).unwrap();
            verified.get_duration();
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
