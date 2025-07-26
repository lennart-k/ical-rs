use crate::{
    PropertyParser,
    parser::{Component, ComponentMut, ParserError},
    property::Property,
};
use std::{cell::RefCell, io::BufRead};

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
pub struct IcalTimeZone<const VERIFIED: bool = true> {
    pub properties: Vec<Property>,
    pub transitions: Vec<IcalTimeZoneTransition<true>>,
}

impl IcalTimeZone {
    pub fn get_tzid(&self) -> &str {
        self.get_property("TZID")
            .and_then(|prop| prop.value.as_ref())
            .expect("we already verified this exists")
    }

    /// This is a common property containing a timezone identifier from the IANA TZDB
    pub fn get_lic_location(&self) -> Option<&str> {
        self.get_property("X-LIC-LOCATION")
            .and_then(|prop| prop.value.as_deref())
    }
}

#[cfg(feature = "chrono-tz")]
impl TryFrom<&IcalTimeZone> for chrono_tz::Tz {
    type Error = chrono_tz::ParseError;

    fn try_from(value: &IcalTimeZone) -> Result<Self, Self::Error> {
        use std::str::FromStr;

        if let Some(loc) = value.get_lic_location() {
            return chrono_tz::Tz::from_str(loc);
        }

        chrono_tz::Tz::from_str(value.get_tzid())
    }
}

impl IcalTimeZone<false> {
    pub fn new() -> IcalTimeZone<false> {
        IcalTimeZone {
            properties: Vec::new(),
            transitions: Vec::new(),
        }
    }
}

impl<const VERIFIED: bool> Component for IcalTimeZone<VERIFIED> {
    const NAMES: &[&str] = &["VTIMEZONE"];
    type Unverified = IcalTimeZone<false>;

    fn get_properties(&self) -> &Vec<Property> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalTimeZone {
            properties: self.properties,
            transitions: self.transitions,
        }
    }
}

impl ComponentMut for IcalTimeZone<false> {
    type Verified = IcalTimeZone<true>;

    fn get_properties_mut(&mut self) -> &mut Vec<Property> {
        &mut self.properties
    }

    fn add_sub_component<B: BufRead>(
        &mut self,
        value: &str,
        line_parser: &RefCell<PropertyParser<B>>,
    ) -> Result<(), ParserError> {
        use self::IcalTimeZoneTransitionType::{DAYLIGHT, STANDARD};

        match value {
            "STANDARD" => {
                let mut transition = IcalTimeZoneTransition::new(STANDARD);
                transition.parse(line_parser)?;
                self.transitions.push(transition.verify()?);
            }
            "DAYLIGHT" => {
                let mut transition = IcalTimeZoneTransition::new(DAYLIGHT);
                transition.parse(line_parser)?;
                self.transitions.push(transition.verify()?);
            }
            _ => return Err(ParserError::InvalidComponent),
        };

        Ok(())
    }

    fn verify(self) -> Result<IcalTimeZone<true>, ParserError> {
        if !matches!(
            self.get_property("TZID"),
            Some(&Property { value: Some(_), .. }),
        ) {
            return Err(ParserError::MissingProperty("TZID"));
        }

        let verified = IcalTimeZone {
            properties: self.properties,
            transitions: self.transitions,
        };

        #[cfg(feature = "test")]
        {
            // Verify that the conditions for our getters are actually met
            verified.get_tzid();
            verified.get_lic_location();
        }

        Ok(verified)
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
pub enum IcalTimeZoneTransitionType {
    #[default]
    STANDARD,
    DAYLIGHT,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
pub struct IcalTimeZoneTransition<const VERIFIED: bool = true> {
    pub transition: IcalTimeZoneTransitionType,
    pub properties: Vec<Property>,
}

impl IcalTimeZoneTransition<false> {
    pub fn new(transition: IcalTimeZoneTransitionType) -> Self {
        Self {
            transition,
            properties: Vec::new(),
        }
    }
}

impl<const VERIFIED: bool> Component for IcalTimeZoneTransition<VERIFIED> {
    const NAMES: &[&str] = &["STANDARD", "DAYLIGHT"];
    type Unverified = IcalTimeZoneTransition<false>;

    fn get_properties(&self) -> &Vec<Property> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalTimeZoneTransition {
            transition: self.transition,
            properties: self.properties,
        }
    }
}

impl ComponentMut for IcalTimeZoneTransition<false> {
    type Verified = IcalTimeZoneTransition<true>;

    fn get_properties_mut(&mut self) -> &mut Vec<Property> {
        &mut self.properties
    }

    fn add_sub_component<B: BufRead>(
        &mut self,
        _: &str,
        _: &RefCell<PropertyParser<B>>,
    ) -> Result<(), ParserError> {
        Err(ParserError::InvalidComponent)
    }

    fn verify(self) -> Result<IcalTimeZoneTransition<true>, ParserError> {
        Ok(IcalTimeZoneTransition {
            transition: self.transition,
            properties: self.properties,
        })
    }
}
