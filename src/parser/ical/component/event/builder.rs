use crate::{
    PropertyParser,
    component::{IcalAlarmBuilder, IcalEvent},
    parser::{
        Component, ComponentMut, GetProperty, IcalDTENDProperty, IcalDTSTAMPProperty,
        IcalDTSTARTProperty, IcalDURATIONProperty, IcalEXDATEProperty, IcalEXRULEProperty,
        IcalMETHODProperty, IcalRDATEProperty, IcalRECURIDProperty, IcalRRULEProperty,
        IcalUIDProperty, ParserError,
    },
    property::ContentLine,
};
use std::{collections::HashMap, io::BufRead};

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
}

impl Component for IcalEventBuilder {
    const NAMES: &[&str] = &["VEVENT"];
    type Unverified = Self;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        self
    }
}

impl ComponentMut for IcalEventBuilder {
    type Verified = IcalEvent;

    fn get_properties_mut(&mut self) -> &mut Vec<ContentLine> {
        &mut self.properties
    }

    fn add_sub_component<B: BufRead>(
        &mut self,
        value: &str,
        line_parser: &mut PropertyParser<B>,
    ) -> Result<(), ParserError> {
        match value {
            "VALARM" => {
                let mut alarm = IcalAlarmBuilder::new();
                alarm.parse(line_parser)?;
                self.alarms.push(alarm);
            }
            _ => return Err(ParserError::InvalidComponent(value.to_owned())),
        };

        Ok(())
    }

    fn build(
        self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<IcalEvent, ParserError> {
        // The following are REQUIRED, but MUST NOT occur more than once: dtstamp / uid
        let IcalDTSTAMPProperty(_dtstamp) = self.safe_get_required(timezones)?;
        let IcalUIDProperty(uid) = self.safe_get_required(timezones)?;
        // REQUIRED if METHOD not specified:
        // For now just ensure that no METHOD property exists
        assert!(
            self.safe_get_optional::<IcalMETHODProperty>(timezones)?
                .is_none()
        );
        let IcalDTSTARTProperty(dtstart) = self.safe_get_required(timezones)?;

        // OPTIONAL, but NOT MORE THAN ONCE: class / created / description / geo / last-mod / location / organizer / priority / seq / status / summary / transp / url / recurid / rrule
        let recurid = self.safe_get_optional::<IcalRECURIDProperty>(timezones)?;
        if let Some(recurid) = &recurid {
            recurid.validate_dtstart(&dtstart)?;
        }

        // OPTIONAL, but MUTUALLY EXCLUSIVE
        if self.has_prop::<IcalDTENDProperty>() && self.has_prop::<IcalDURATIONProperty>() {
            return Err(ParserError::PropertyConflict(
                "both DTEND and DURATION are defined",
            ));
        }
        let dtend = self
            .safe_get_optional::<IcalDTENDProperty>(timezones)?
            .map(Into::into);
        let duration = self
            .safe_get_optional::<IcalDURATIONProperty>(timezones)?
            .map(Into::into);

        // OPTIONAL, allowed multiple times: attach / attendee / categories / comment / contact / exdate / rstatus / related / resources / rdate / x-prop / iana-prop
        let rrule_dtstart = dtstart.utc().with_timezone(&rrule::Tz::UTC);
        let rdates = self
            .safe_get_all::<IcalRDATEProperty>(timezones)?
            .into_iter()
            .map(|exdate| exdate.0)
            .collect();
        let exdates = self
            .safe_get_all::<IcalEXDATEProperty>(timezones)?
            .into_iter()
            .map(|exdate| exdate.0)
            .collect();
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

        Ok(IcalEvent {
            uid,
            dtstart,
            dtend,
            duration,
            rdates,
            rrules,
            exdates,
            exrules,
            recurid,
            properties: self.properties,
            alarms: self
                .alarms
                .into_iter()
                .map(|alarm| alarm.build(timezones))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}
