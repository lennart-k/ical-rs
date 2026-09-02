use chrono::DateTime;

use crate::{
    ContentLineParser,
    component::{Component, ComponentMut, IcalAlarmBuilder, IcalEvent},
    parser::{ContentLine, ParserError, ParserOptions},
    property::{
        GetProperty, IcalDTENDProperty, IcalDTSTAMPProperty, IcalDTSTARTProperty,
        IcalDURATIONProperty, IcalEXDATEProperty, IcalEXRULEProperty, IcalMETHODProperty,
        IcalRDATEProperty, IcalRECURIDProperty, IcalRRULEProperty, IcalSUMMARYProperty,
        IcalUIDProperty,
    },
    types::{CalDateOrDateTime, CalDateTime, Tz},
};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

#[derive(Debug, Clone, Default)]
pub struct IcalEventBuilder {
    pub properties: Vec<ContentLine>,
    pub alarms: Vec<IcalAlarmBuilder>,
}

impl IcalEventBuilder {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
            alarms: Vec::new(),
        }
    }

    pub fn get_tzids(&self) -> HashSet<&str> {
        self.properties
            .iter()
            .filter_map(|prop| prop.params.get_tzid())
            .collect()
    }

    pub fn with_summary(mut self, summary: String) -> Self {
        self.properties
            .push(IcalSUMMARYProperty(summary, Default::default()).into());
        self
    }

    pub fn with_dtstamp(mut self, dtstamp: CalDateTime) -> Self {
        self.properties
            .push(IcalDTSTAMPProperty(dtstamp, Default::default()).into());
        self
    }

    pub fn with_dtstart(mut self, dtstart: CalDateOrDateTime) -> Self {
        self.properties
            .push(IcalDTSTARTProperty(dtstart, Default::default()).into());
        self
    }

    pub fn with_uid(mut self, uid: String) -> Self {
        self.properties.push(IcalUIDProperty::from(uid).into());
        self
    }
}

impl Component for IcalEventBuilder {
    const NAMES: &[&str] = &["VEVENT"];
    type Builder = Self;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Builder {
        self
    }
}

impl ComponentMut for IcalEventBuilder {
    type Verified = IcalEvent;

    fn get_properties_mut(&mut self) -> &mut Vec<ContentLine> {
        &mut self.properties
    }

    #[inline]
    fn add_sub_component<'a, I: Iterator<Item = Cow<'a, [u8]>>>(
        &mut self,
        value: &str,
        line_parser: &mut ContentLineParser<'a, I>,
        options: &ParserOptions,
    ) -> Result<(), ParserError> {
        match value {
            "VALARM" => {
                let mut alarm = IcalAlarmBuilder::new();
                alarm.parse(line_parser, options)?;
                self.alarms.push(alarm);
            }
            _ => return Err(ParserError::InvalidComponent(value.to_owned())),
        };

        Ok(())
    }

    fn build(
        self,
        options: &ParserOptions,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<IcalEvent, ParserError> {
        // The following are REQUIRED, but MUST NOT occur more than once: dtstamp / uid
        let dtstamp = self.safe_get_required(timezones)?;
        let IcalUIDProperty(uid, _) = self.safe_get_required(timezones)?;
        // REQUIRED if METHOD not specified:
        // For now just ensure that no METHOD property exists
        assert!(
            self.safe_get_optional::<IcalMETHODProperty>(timezones)?
                .is_none()
        );
        let dtstart: IcalDTSTARTProperty = self.safe_get_required(timezones)?;

        // OPTIONAL, but NOT MORE THAN ONCE: class / created / description / geo / last-mod / location / organizer / priority / seq / status / summary / transp / url / recurid / rrule
        let summary = self.safe_get_optional::<IcalSUMMARYProperty>(timezones)?;
        let recurid = self.safe_get_optional::<IcalRECURIDProperty>(timezones)?;
        if let Some(recurid) = &recurid {
            recurid.validate_dtstart(&dtstart.0)?;
        }

        // OPTIONAL, but MUTUALLY EXCLUSIVE
        if self.has_prop::<IcalDTENDProperty>() && self.has_prop::<IcalDURATIONProperty>() {
            return Err(ParserError::PropertyConflict(
                "both DTEND and DURATION are defined",
            ));
        }
        let dtend = self.safe_get_optional::<IcalDTENDProperty>(timezones)?;
        let duration = self.safe_get_optional::<IcalDURATIONProperty>(timezones)?;

        // OPTIONAL, allowed multiple times: attach / attendee / categories / comment / contact / exdate / rstatus / related / resources / rdate / x-prop / iana-prop
        let rrule_dtstart: DateTime<Tz> = dtstart.0.clone().into();
        let rdates = self.safe_get_all::<IcalRDATEProperty>(timezones)?;
        let exdates = self.safe_get_all::<IcalEXDATEProperty>(timezones)?;
        let rrules = self
            .safe_get_all::<IcalRRULEProperty>(timezones)?
            .into_iter()
            // RRules are crated against local times instead of UTC
            .map(|rrule| rrule.0.validate(rrule_dtstart))
            .collect::<Result<Vec<_>, _>>()?;
        let exrules = self
            .safe_get_all::<IcalEXRULEProperty>(timezones)?
            .into_iter()
            .map(|rrule| rrule.0.validate(rrule_dtstart))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(IcalEvent {
            uid,
            dtstamp,
            dtstart,
            dtend,
            duration,
            rdates,
            rrules,
            exdates,
            exrules,
            recurid,
            summary,
            properties: self.properties,
            alarms: self
                .alarms
                .into_iter()
                .map(|alarm| alarm.build(options, timezones))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        component::{Component, ComponentMut, IcalEvent},
        generator::Emitter,
        parser::ParserOptions,
    };
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_builder() {
        let ical_event = IcalEvent::builder()
            .with_dtstamp(Utc.with_ymd_and_hms(2026, 6, 28, 10, 3, 12).unwrap().into())
            .with_dtstart(Utc.with_ymd_and_hms(2026, 6, 28, 10, 3, 12).unwrap().into())
            .with_uid("alskdj".to_string())
            .with_summary("Hello World!".to_string())
            .build(&ParserOptions { rfc7809: false }, None)
            .unwrap();
        insta::assert_snapshot!(ical_event.generate(), @r"
        BEGIN:VEVENT
        DTSTAMP:20260628T100312Z
        DTSTART:20260628T100312Z
        UID:alskdj
        SUMMARY:Hello World!
        END:VEVENT
        ");
    }
}
