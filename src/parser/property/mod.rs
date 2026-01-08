use crate::{
    parser::{Component, ParserError},
    property::ContentLine,
    types::{CalDateOrDateTime, DateOrDateTimeOrPeriod, parse_duration},
};
use std::collections::HashMap;
use std::str::FromStr;

mod duration;
pub use duration::*;
mod exdate;
pub use exdate::*;
mod rdate;
pub use rdate::*;
mod dtstart;
pub use dtstart::*;
mod recurid;
pub use recurid::*;
mod due;
pub use due::*;
mod dtstamp;
pub use dtstamp::*;
mod dtend;
pub use dtend::*;
mod calscale;
pub use calscale::*;
mod version;
pub use version::*;

pub trait ICalProperty: Sized {
    const NAME: &'static str;
    const DEFAULT_TYPE: &'static str;

    fn parse_prop(
        prop: &ContentLine,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Self, ParserError>;

    fn utc_or_local(self) -> Self;
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

impl ParseProp for DateOrDateTimeOrPeriod {
    fn parse_prop(
        prop: &ContentLine,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
        default_type: &str,
    ) -> Result<Self, ParserError> {
        Ok(Self::parse_prop(prop, timezones, default_type)?)
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

impl<T: ParseProp> ParseProp for Vec<T> {
    fn parse_prop(
        prop: &ContentLine,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
        default_type: &str,
    ) -> Result<Self, ParserError> {
        let mut out = vec![];
        for value in prop
            .value
            .as_deref()
            .unwrap_or_default()
            .trim_end_matches(',')
            .split(',')
        {
            let content_line = ContentLine {
                name: prop.name.to_owned(),
                params: prop.params.to_owned(),
                value: Some(value.to_owned()),
            };
            out.push(T::parse_prop(&content_line, timezones, default_type)?);
        }
        Ok(out)
    }
}

macro_rules! property {
    ($name:literal, $default_type:literal, $prop:ty) => {
        impl crate::parser::property::ICalProperty for $prop {
            const NAME: &'static str = $name;
            const DEFAULT_TYPE: &'static str = $default_type;

            fn parse_prop(
                prop: &crate::property::ContentLine,
                timezones: &std::collections::HashMap<String, Option<chrono_tz::Tz>>,
            ) -> Result<Self, crate::parser::ParserError> {
                Ok(Self(
                    crate::parser::ParseProp::parse_prop(prop, timezones, $default_type)?,
                    prop.params.clone(),
                ))
            }

            fn utc_or_local(self) -> Self {
                let Self(dt, mut params) = self;
                params.remove("TZID");
                Self(crate::types::Value::utc_or_local(dt), params)
            }
        }
    };

    ($name:literal, $default_type:literal, $prop:ident, $inner:ty) => {
        #[derive(Debug, Clone, PartialEq, Eq, derive_more::From)]
        pub struct $prop(pub $inner, pub crate::property::ContentLineParams);
        crate::parser::property!($name, $default_type, $prop);

        impl From<$prop> for crate::property::ContentLine {
            fn from(prop: $prop) -> Self {
                let $prop(inner, mut params) = prop;
                let value_type = crate::types::Value::value_type(&inner).unwrap_or($default_type);
                if value_type != $default_type {
                    params.replace_param("VALUE".to_owned(), value_type.to_owned());
                }
                crate::property::ContentLine {
                    name: $name.to_owned(),
                    params,
                    value: Some(crate::types::Value::value(&inner)),
                }
            }
        }
    };
}
pub(crate) use property;

property!("UID", "TEXT", IcalUIDProperty, String);

impl From<String> for IcalUIDProperty {
    fn from(value: String) -> Self {
        Self(value, Default::default())
    }
}

property!("SUMMARY", "TEXT", IcalSUMMARYProperty, String);

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
property!("PRODID", "TEXT", IcalPRODIDProperty, String);

property!("METHOD", "TEXT", IcalMETHODProperty, String);

property!("FN", "TEXT", VcardFNProperty, String);
property!("N", "TEXT", VcardNProperty, String);
property!("NICKNAME", "TEXT", VcardNICKNAMEProperty, String);
