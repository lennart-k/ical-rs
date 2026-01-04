use crate::{
    PropertyParser,
    parser::{
        Component, ComponentMut, GetProperty, IcalDURATIONProperty, IcalUIDProperty, ParserError,
        ical::component::IcalAlarm,
    },
    property::ContentLine,
};
use chrono::Duration;
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
    pub alarms: Vec<IcalAlarm<false>>,
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
    ) -> Result<IcalTodo, ParserError> {
        let IcalUIDProperty(uid) = self.safe_get_required(timezones)?;
        let _duration: Option<Duration> = self
            .safe_get_optional::<IcalDURATIONProperty>(timezones)?
            .map(Into::into);

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
