use crate::{
    parser::{ParseProp, ParserError},
    property::ContentLine,
    types::Value,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Calscale {
    Gregorian,
}

impl Value for Calscale {
    fn value_type(&self) -> Option<&'static str> {
        Some("TEXT")
    }

    fn value(&self) -> String {
        "GREGORIAN".to_owned()
    }
}

impl ParseProp for Calscale {
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
            "GREGORIAN" => Ok(Self::Gregorian),
            _ => Err(ParserError::InvalidCalscale),
        }
    }
}
super::property!("CALSCALE", "TEXT", IcalCALSCALEProperty, Calscale);
