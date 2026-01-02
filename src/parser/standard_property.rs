use std::collections::HashMap;

use crate::{
    parser::{Component, ParserError},
    property::Property,
    types::CalDateOrDateTime,
};

pub trait ICalProperty<'p>: ParseProp {
    const NAME: &'static str;
}

// pub trait GetProperty<'p, T: ICalProperty<'p>>: Component {
//     fn get_all(&'p self) -> Result<Vec<T>, ParserError> {
//         self.get_named_properties(T::NAME)
//             .into_iter()
//             .map(TryInto::try_into)
//             .collect::<Result<Vec<_>, _>>()
//     }
//
//     fn get(&'p self) -> Result<Option<T>, ParserError> {
//         let Some(prop) = self.get_property(T::NAME) else {
//             return Ok(None);
//         };
//         prop.try_into().map(Some)
//     }
// }

pub trait ParseProp: Sized {
    fn parse_prop(
        prop: &Property,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Self, ParserError>;
}

impl ParseProp for String {
    fn parse_prop(
        prop: &Property,
        _timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Self, ParserError> {
        Ok(prop.value.to_owned().unwrap_or_else(String::new))
    }
}

impl ParseProp for CalDateOrDateTime {
    fn parse_prop(
        prop: &Property,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Self, ParserError> {
        Ok(Self::parse_prop(prop, timezones)?)
    }
}

macro_rules! property {
    ($name:literal, $prop:ty, $inner:ty) => {
        impl ParseProp for $prop {
            fn parse_prop(
                prop: &Property,
                timezones: &HashMap<String, Option<chrono_tz::Tz>>,
            ) -> Result<Self, ParserError> {
                Ok(Self(ParseProp::parse_prop(prop, timezones)?))
            }
        }

        impl<'p> ICalProperty<'p> for $prop {
            const NAME: &'static str = $name;
        }
    };
}

pub struct IcalUIDProperty(pub String);
// property!("UID", IcalUIDProperty, String);
pub struct IcalDTSTARTProperty(pub CalDateOrDateTime);
property!("DTSTART", IcalDTSTARTProperty, CalDateOrDateTime);
