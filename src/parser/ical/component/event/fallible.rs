use super::IcalEvent;
use crate::{
    parser::{Component, ParserError},
    types::{CalDateOrDateTime, CalDateTime, DateOrDateTimeOrPeriod, parse_duration},
};
use rrule::RRule;
use std::{collections::HashMap, str::FromStr};

// Implementation of fallible field accessors
// During verification these fields will all be accessed such that the verified variant can simply
// unwrap the properties
impl<const VERIFIED: bool> IcalEvent<VERIFIED> {
    // ;
    // ; The following are REQUIRED,
    // ; but MUST NOT occur more than once.
    // ;
    // dtstamp / uid /
    // TYPE: DATE-TIME
    pub fn safe_get_dtstamp(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Option<CalDateTime>, ParserError> {
        let Some(prop) = self.get_property("DTSTAMP") else {
            return Ok(None);
        };
        Ok(Some(CalDateTime::parse_prop(prop, timezones)?))
    }
    // TYPE: TEXT
    pub fn safe_get_uid(&self) -> Result<&str, ParserError> {
        self.get_property("UID")
            .and_then(|prop| prop.value.as_deref())
            .ok_or(ParserError::MissingUID)
    }
    // ;
    // ; The following is REQUIRED if the component
    // ; appears in an iCalendar object that doesn't
    // ; specify the "METHOD" property; otherwise, it
    // ; is OPTIONAL; in any case, it MUST NOT occur
    // ; more than once.
    // ;
    // dtstart TYPE: DATE-TIME (or DATE)
    pub fn safe_get_dtstart(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Option<CalDateOrDateTime>, ParserError> {
        let Some(prop) = self.get_property("DTSTART") else {
            return Ok(None);
        };
        Ok(Some(CalDateOrDateTime::parse_prop(prop, timezones)?))
    }
    // ;
    // ; The following are OPTIONAL,
    // ; but MUST NOT occur more than once.
    // ;
    // class / created / description / geo /
    // last-mod / location / organizer / priority /
    // seq / status / summary / transp /
    // url / recurid /
    // ;
    // ; The following is OPTIONAL,
    // ; but SHOULD NOT occur more than once.
    // ;
    // rrule /
    pub fn safe_get_rrule(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Vec<RRule>, ParserError> {
        let dtstart = self.safe_get_dtstart(timezones)?.unwrap().into();
        self.get_named_properties("RRULE")
            .into_iter()
            .map(|prop| {
                let value = prop.value.as_ref().ok_or_else(|| {
                    ParserError::RRule(rrule::ParseError::MissingDateGenerationRules.into())
                })?;
                Ok(RRule::from_str(value)?.validate(dtstart)?)
            })
            .collect::<Result<Vec<_>, _>>()
    }
    // ;
    // ; Either 'dtend' or 'duration' MAY appear in
    // ; a 'eventprop', but 'dtend' and 'duration'
    // ; MUST NOT occur in the same 'eventprop'.
    // ;
    // dtend /
    pub fn safe_get_dtend(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Option<CalDateOrDateTime>, ParserError> {
        let Some(prop) = self.get_property("DTEND") else {
            return Ok(None);
        };
        Ok(Some(CalDateOrDateTime::parse_prop(prop, timezones)?))
    }
    // duration /
    pub fn safe_get_duration(&self) -> Result<Option<chrono::Duration>, ParserError> {
        let Some(prop) = self.get_property("DURATION") else {
            return Ok(None);
        };
        Ok(Some(parse_duration(
            prop.value.as_deref().unwrap_or_default(),
        )?))
    }
    // ;
    // ; The following are OPTIONAL,
    // ; and MAY occur more than once.
    // ;
    // attach / attendee / categories / comment /
    // contact /
    // exdate TYPE: DATE-TIME (or DATE)
    pub fn safe_get_exdate(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Vec<CalDateOrDateTime>, ParserError> {
        self.get_named_properties("EXDATE")
            .into_iter()
            .map(|prop| Ok(CalDateOrDateTime::parse_prop(prop, timezones)?))
            .collect::<Result<Vec<_>, _>>()
    }
    //
    // rstatus / related /
    // resources
    // rdate TYPE: DATE-TIME (or DATE, PERIOD)
    pub fn safe_get_rdate(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Vec<DateOrDateTimeOrPeriod>, ParserError> {
        self.get_named_properties("RDATE")
            .into_iter()
            .map(|prop| {
                Ok(DateOrDateTimeOrPeriod::parse_prop(
                    prop,
                    timezones,
                    "DATE-TIME",
                )?)
            })
            .collect::<Result<Vec<_>, _>>()
    }
    //
    // x-prop / iana-prop
    // ;
    // )
}
