use crate::{
    parser::{ParseProp, ParserError},
    property::ContentLine,
    types::Value,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IcalVersion {
    Version1_0,
    Version2_0,
}

impl Value for IcalVersion {
    fn value_type(&self) -> Option<&'static str> {
        Some("TEXT")
    }

    fn value(&self) -> String {
        match self {
            Self::Version1_0 => "1.0".to_owned(),
            Self::Version2_0 => "2.0".to_owned(),
        }
    }
}

impl ParseProp for IcalVersion {
    fn parse_prop(
        prop: &ContentLine,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
        _default_type: &str,
    ) -> Result<Self, ParserError> {
        match prop
            .value
            .as_deref()
            .unwrap_or_default()
            .to_uppercase()
            .as_str()
        {
            "1.0" => Ok(Self::Version1_0),
            "2.0" => Ok(Self::Version2_0),
            _ => Err(ParserError::InvalidVersion),
        }
    }
}
super::property!("VERSION", "TEXT", IcalVERSIONProperty, IcalVersion);
