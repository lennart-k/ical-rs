use crate::{
    parser::{Component, ParserError},
    property::ContentLine,
    types::{CalDateOrDateTime, parse_duration},
};
use chrono::Duration;
use std::collections::HashMap;

pub trait ICalProperty: Sized {
    const NAME: &'static str;
    const DEFAULT_TYPE: &'static str;

    fn parse_prop(
        prop: &ContentLine,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Self, ParserError>;
}

pub trait GetProperty: Component {
    fn safe_get_all<T: ICalProperty>(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Vec<T>, ParserError> {
        self.get_named_properties(T::NAME)
            .into_iter()
            .map(|prop| ICalProperty::parse_prop(prop, timezones))
            .collect::<Result<Vec<_>, _>>()
    }

    fn safe_get_optional<T: ICalProperty>(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Option<T>, ParserError> {
        let Some(prop) = self.get_property(T::NAME) else {
            return Ok(None);
        };
        ICalProperty::parse_prop(prop, timezones).map(Some)
    }

    fn safe_get_required<T: ICalProperty>(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<T, ParserError> {
        self.safe_get_optional(timezones)?
            .ok_or(ParserError::MissingProperty(T::NAME))
    }

    fn has_prop<T: ICalProperty>(&self) -> bool {
        self.get_property(T::NAME).is_some()
    }
}

impl<C: Component> GetProperty for C {}

pub trait ParseProp: Sized {
    fn parse_prop(
        prop: &ContentLine,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
        default_type: &str,
    ) -> Result<Self, ParserError>;
}

impl ParseProp for String {
    fn parse_prop(
        prop: &ContentLine,
        _timezones: &HashMap<String, Option<chrono_tz::Tz>>,
        _default_type: &str,
    ) -> Result<Self, ParserError> {
        Ok(prop.value.to_owned().unwrap_or_default())
    }
}

impl ParseProp for CalDateOrDateTime {
    fn parse_prop(
        prop: &ContentLine,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
        default_type: &str,
    ) -> Result<Self, ParserError> {
        Ok(Self::parse_prop(prop, timezones, default_type)?)
    }
}

impl ParseProp for chrono::Duration {
    fn parse_prop(
        prop: &ContentLine,
        _timezones: &HashMap<String, Option<chrono_tz::Tz>>,
        _default_type: &str,
    ) -> Result<Self, ParserError> {
        Ok(parse_duration(prop.value.as_deref().unwrap_or_default())?)
    }
}

macro_rules! property {
    ($name:literal, $default_type:literal, $prop:ty) => {
        impl ICalProperty for $prop {
            const NAME: &'static str = $name;
            const DEFAULT_TYPE: &'static str = $default_type;

            fn parse_prop(
                prop: &ContentLine,
                timezones: &HashMap<String, Option<chrono_tz::Tz>>,
            ) -> Result<Self, ParserError> {
                Ok(Self(ParseProp::parse_prop(prop, timezones, $default_type)?))
            }
        }
    };

    ($name:literal, $default_type:literal, $prop:ident, $inner:ty) => {
        #[derive(Debug, Clone, PartialEq, Eq, derive_more::Into)]
        pub struct $prop(pub $inner);
        property!($name, $default_type, $prop);
    };
}

property!("UID", "TEXT", IcalUIDProperty, String);
property!(
    "DTSTART",
    "DATE-TIME",
    IcalDTSTARTProperty,
    CalDateOrDateTime
);
property!(
    "DTSTAMP",
    "DATE-TIME",
    IcalDTSTAMPProperty,
    CalDateOrDateTime
);
property!("DTEND", "DATE-TIME", IcalDTENDProperty, CalDateOrDateTime);
property!("DUE", "DATE-TIME", IcalDUEProperty, CalDateOrDateTime);
property!("METHOD", "TEXT", IcalMETHODProperty, String);
property!("DURATION", "DURATION", IcalDURATIONProperty, Duration);
property!(
    "RECURRENCE-ID",
    "DATE-TIME",
    IcalRECURRENCEIDProperty,
    CalDateOrDateTime
);
