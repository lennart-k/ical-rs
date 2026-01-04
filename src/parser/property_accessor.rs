use crate::types::Value;
use crate::{
    parser::{Component, ParserError},
    property::ContentLine,
    types::{CalDateOrDateTime, parse_duration},
};
use chrono::Duration;
use std::collections::HashMap;
use std::str::FromStr;

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
        let mut props = self.get_named_properties(T::NAME).into_iter();
        let Some(prop) = props.next() else {
            return Ok(None);
        };
        if props.next().is_some() {
            return Err(ParserError::PropertyConflict(
                "Multiple instances of property",
            ));
        }
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

impl ParseProp for rrule::RRule<rrule::Unvalidated> {
    fn parse_prop(
        prop: &ContentLine,
        _timezones: &HashMap<String, Option<chrono_tz::Tz>>,
        _default_type: &str,
    ) -> Result<Self, ParserError> {
        Ok(rrule::RRule::from_str(
            prop.value.as_deref().unwrap_or_default(),
        )?)
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

        impl From<$prop> for ContentLine {
            fn from(prop: $prop) -> Self {
                let mut params = vec![];
                let value_type = Value::value_type(&prop.0);
                if value_type != $default_type {
                    params.push(("VALUE".to_owned(), vec![value_type.to_owned()]));
                }
                ContentLine {
                    name: $name.to_owned(),
                    params,
                    value: Some(Value::value(&prop.0)),
                }
            }
        }
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
property!("RDATE", "DATE-TIME", IcalRDATEProperty, CalDateOrDateTime);
property!("EXDATE", "DATE-TIME", IcalEXDATEProperty, CalDateOrDateTime);
property!(
    "RRULE",
    "RECUR",
    IcalRRULEProperty,
    rrule::RRule<rrule::Unvalidated>
);
property!(
    "EXRULE",
    "RECUR",
    IcalEXRULEProperty,
    rrule::RRule<rrule::Unvalidated>
);
property!("METHOD", "TEXT", IcalMETHODProperty, String);
property!("DURATION", "DURATION", IcalDURATIONProperty, Duration);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum RecurIdRange {
    #[default]
    This,
    ThisAndFuture,
}
#[derive(Debug, Clone)]
pub struct IcalRECURIDProperty(pub CalDateOrDateTime, pub RecurIdRange);
impl ICalProperty for IcalRECURIDProperty {
    const NAME: &'static str = "RECURRENCE-ID";
    const DEFAULT_TYPE: &'static str = "DATE-TIME";

    fn parse_prop(
        prop: &ContentLine,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Self, ParserError> {
        let dt = ParseProp::parse_prop(prop, timezones, Self::DEFAULT_TYPE)?;
        let range = match prop.get_param("RANGE") {
            Some("THISANDFUTURE") => RecurIdRange::ThisAndFuture,
            None => RecurIdRange::This,
            _ => panic!("Invalid range parameter"),
        };
        Ok(Self(dt, range))
    }
}
impl IcalRECURIDProperty {
    pub fn validate_dtstart(&self, dtstart: &CalDateOrDateTime) -> Result<(), ParserError> {
        assert_eq!(
            self.0.is_date(),
            dtstart.is_date(),
            "DTSTART and RECURRENCE-ID have different value types"
        );
        assert_eq!(
            self.0.timezone().is_local(),
            dtstart.timezone().is_local(),
            "DTSTART and RECURRENCE-ID have different timezone types"
        );

        Ok(())
    }
}

impl From<IcalRECURIDProperty> for crate::property::ContentLine {
    fn from(value: IcalRECURIDProperty) -> Self {
        let mut params = vec![];
        let value_type = value.0.value_type();
        if value_type != IcalRECURIDProperty::DEFAULT_TYPE {
            params.push(("VALUE".to_owned(), vec![value_type.to_owned()]));
        }
        if value.1 == RecurIdRange::ThisAndFuture {
            params.push(("RANGE".to_owned(), vec!["THISANDFUTURE".to_owned()]));
        }
        Self {
            name: IcalRECURIDProperty::NAME.to_owned(),
            params,
            value: Some(value.0.format()),
        }
    }
}
