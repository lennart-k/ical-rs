use crate::{
    PropertyParser,
    component::IcalAlarmBuilder,
    parser::{
        Component, ComponentMut, GetProperty, IcalDTSTAMPProperty, IcalDTSTARTProperty,
        IcalDUEProperty, IcalDURATIONProperty, IcalRECURIDProperty, IcalUIDProperty, ParserError,
        ical::component::IcalAlarm,
    },
    property::ContentLine,
};
use itertools::Itertools;
use std::{collections::HashMap, io::BufRead};

#[derive(Debug, Clone)]
pub struct IcalTodo {
    uid: String,
    pub properties: Vec<ContentLine>,
    pub alarms: Vec<IcalAlarm>,
}

#[derive(Debug, Clone, Default)]
pub struct IcalTodoBuilder {
    pub properties: Vec<ContentLine>,
    pub alarms: Vec<IcalAlarmBuilder>,
}

impl IcalTodo {
    pub fn get_uid(&self) -> &str {
        &self.uid
    }
}

impl Component for IcalTodo {
    const NAMES: &[&str] = &["VTODO"];
    type Unverified = IcalTodoBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalTodoBuilder {
            properties: self.properties,
            alarms: self
                .alarms
                .into_iter()
                .map(|alarm| alarm.mutable())
                .collect(),
        }
    }
}

impl Component for IcalTodoBuilder {
    const NAMES: &[&str] = &["VTODO"];
    type Unverified = IcalTodoBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        self
    }
}

impl ComponentMut for IcalTodoBuilder {
    type Verified = IcalTodo;

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
    ) -> Result<IcalTodo, ParserError> {
        // REQUIRED, but ONLY ONCE
        let IcalUIDProperty(uid) = self.safe_get_required(timezones)?;
        let IcalDTSTAMPProperty(_dtstamp) = self.safe_get_required(timezones)?;

        // OPTIONAL, but ONLY ONCE: class / completed / created / description / dtstart / geo / last-mod / location / organizer / percent / priority / recurid / seq / status / summary / url / rrule
        let _dtstart = self.safe_get_optional::<IcalDTSTARTProperty>(timezones)?;
        let _recurid = self.safe_get_optional::<IcalRECURIDProperty>(timezones)?;
        // OPTIONAL, but MUTUALLY EXCLUSIVE
        if self.has_prop::<IcalDURATIONProperty>() && self.has_prop::<IcalDUEProperty>() {
            return Err(ParserError::PropertyConflict(
                "both DUE and DURATION are defined",
            ));
        }
        let _duration = self.safe_get_optional::<IcalDURATIONProperty>(timezones)?;
        let _due = self.safe_get_optional::<IcalDUEProperty>(timezones)?;

        // OPTIONAL, MULTIPLE ALLOWED: attach / attendee / categories / comment / contact / exdate / rstatus / related / resources / rdate / x-prop / iana-prop

        let verified = IcalTodo {
            uid,
            properties: self.properties,
            alarms: self
                .alarms
                .into_iter()
                .map(|alarm| alarm.build(timezones))
                .collect::<Result<Vec<_>, _>>()?,
        };

        Ok(verified)
    }
}

impl IcalTodo {
    pub fn get_tzids(&self) -> Vec<&str> {
        self.properties
            .iter()
            .filter_map(|prop| prop.get_tzid())
            .unique()
            .collect()
    }
}
