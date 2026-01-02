use super::IcalEvent;
use crate::{
    parser::{Component, ParserError},
    types::{CalDateOrDateTime, CalDateTime, DateOrDateTimeOrPeriod},
};
use chrono::Duration;
use rrule::RRuleSet;
use std::collections::HashMap;

impl IcalEvent<true> {
    // ;
    // ; The following are REQUIRED,
    // ; but MUST NOT occur more than once.
    // ;
    // dtstamp / uid /
    // TYPE: DATE-TIME
    pub fn get_dtstamp(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Option<CalDateTime>, ParserError> {
        let Some(prop) = self.get_property("DTSTAMP") else {
            return Ok(None);
        };
        Ok(Some(CalDateTime::parse_prop(prop, timezones)?))
    }
    // TYPE: TEXT
    pub fn get_uid(&self) -> &str {
        self.get_property("UID")
            .and_then(|prop| prop.value.as_deref())
            .expect("already verified that this must exist")
    }
    // ;
    // ; The following is REQUIRED if the component
    // ; appears in an iCalendar object that doesn't
    // ; specify the "METHOD" property; otherwise, it
    // ; is OPTIONAL; in any case, it MUST NOT occur
    // ; more than once.
    // ;
    // dtstart /
    // TYPE: DATE-TIME (or DATE)
    pub fn get_dtstart(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Option<CalDateOrDateTime> {
        self.safe_get_dtstart(timezones).unwrap()
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
    pub fn get_rrule(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Vec<rrule::RRule> {
        self.safe_get_rrule(timezones).expect("already validated")
    }

    pub fn get_rrule_set(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Option<RRuleSet> {
        let dtstart = self.get_dtstart(timezones).unwrap().into();
        let rrules = self.get_rrule(timezones);
        let rdates = self
            .get_rdate(timezones)
            .into_iter()
            .map(|d| d.start().into())
            .collect::<Vec<_>>();

        if rrules.is_empty() && rdates.is_empty() {
            return None;
        }

        Some(
            RRuleSet::new(dtstart)
                .set_rrules(rrules)
                .set_rdates(rdates)
                .set_exdates(
                    self.get_exdate(timezones)
                        .into_iter()
                        .map(|d| d.into())
                        .collect(),
                ),
        )
    }
    // ;
    // ; Either 'dtend' or 'duration' MAY appear in
    // ; a 'eventprop', but 'dtend' and 'duration'
    // ; MUST NOT occur in the same 'eventprop'.
    // ;
    // dtend /
    pub fn get_dtend(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Option<CalDateOrDateTime> {
        self.safe_get_dtend(timezones).unwrap()
    }
    // duration /
    pub fn get_duration(&self) -> Option<Duration> {
        self.safe_get_duration().unwrap()
    }
    // ;
    // ; The following are OPTIONAL,
    // ; and MAY occur more than once.
    // ;
    // attach / attendee / categories / comment /
    // contact
    // exdate
    pub fn get_exdate(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Vec<CalDateOrDateTime> {
        self.safe_get_exdate(timezones).expect("already validated")
    }
    // rstatus / related /
    // resources
    // rdate
    pub fn get_rdate(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Vec<DateOrDateTimeOrPeriod> {
        self.safe_get_rdate(timezones).expect("already validated")
    }
    //
    // / x-prop / iana-prop
    // ;
    // )
}
