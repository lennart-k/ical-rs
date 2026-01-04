use crate::{
    PropertyParser,
    component::IcalEvent,
    parser::{
        Component, ComponentMut, GetProperty, IcalDTENDProperty, IcalDTSTARTProperty,
        IcalDURATIONProperty, IcalMETHODProperty, IcalUIDProperty, ParserError,
        ical::component::IcalAlarm,
    },
    property::ContentLine,
};
use std::{collections::HashMap, io::BufRead};

#[derive(Debug, Clone, Default)]
pub struct IcalEventBuilder {
    pub properties: Vec<ContentLine>,
    pub alarms: Vec<IcalAlarm<false>>,
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
                let mut alarm = IcalAlarm::new();
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
        let IcalUIDProperty(uid) = self.safe_get_required(timezones)?;

        // For now just ensure that no METHOD property exists
        assert!(
            self.safe_get_optional::<IcalMETHODProperty>(timezones)?
                .is_none()
        );

        // If METHOD is undefined, DTSTART MUST be defined
        let IcalDTSTARTProperty(_dtstart) = self.safe_get_required(timezones)?;

        if self.has_prop::<IcalDTENDProperty>() && self.has_prop::<IcalDURATIONProperty>() {
            return Err(ParserError::PropertyConflict(
                "both DTEND and DURATION are defined",
            ));
        }

        Ok(IcalEvent {
            uid,
            properties: self.properties,
            alarms: self
                .alarms
                .into_iter()
                .map(|alarm| alarm.build(timezones))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}
