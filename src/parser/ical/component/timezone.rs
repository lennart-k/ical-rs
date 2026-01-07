use crate::{
    PropertyParser,
    parser::{Component, ComponentMut, ParserError},
    property::ContentLine,
};
use std::{collections::HashMap, io::BufRead};

#[derive(Debug, Clone, Default)]
pub struct IcalTimeZone<const VERIFIED: bool = true> {
    pub properties: Vec<ContentLine>,
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
impl From<&IcalTimeZone> for Option<chrono_tz::Tz> {
    fn from(value: &IcalTimeZone) -> Self {
        use crate::types::get_proprietary_tzid;
        use std::str::FromStr;

        // Try X-LIC-LOCATION
        if let Some(loc) = value.get_lic_location()
            && let Ok(tz) = chrono_tz::Tz::from_str(loc)
        {
            return Some(tz);
        };

        // Try using TZID in Olson DB
        let tzid = value.get_tzid();
        if let Ok(tz) = chrono_tz::Tz::from_str(tzid) {
            return Some(tz);
        }
        // Try map of proprietary timezone IDs (mostly for Microsoft products)
        get_proprietary_tzid(tzid)
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

    fn get_properties(&self) -> &Vec<ContentLine> {
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

    fn get_properties_mut(&mut self) -> &mut Vec<ContentLine> {
        &mut self.properties
    }

    fn add_sub_component<B: BufRead>(
        &mut self,
        value: &str,
        line_parser: &mut PropertyParser<B>,
    ) -> Result<(), ParserError> {
        use self::IcalTimeZoneTransitionType::{DAYLIGHT, STANDARD};

        match value {
            "STANDARD" => {
                let mut transition = IcalTimeZoneTransition::new(STANDARD);
                transition.parse(line_parser)?;
                self.transitions
                    .push(transition.build(&HashMap::default())?);
            }
            "DAYLIGHT" => {
                let mut transition = IcalTimeZoneTransition::new(DAYLIGHT);
                transition.parse(line_parser)?;
                self.transitions
                    .push(transition.build(&HashMap::default())?);
            }
            _ => return Err(ParserError::InvalidComponent(value.to_owned())),
        };

        Ok(())
    }

    fn build(
        self,
        _timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<IcalTimeZone<true>, ParserError> {
        if !matches!(
            self.get_property("TZID"),
            Some(&ContentLine { value: Some(_), .. }),
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
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum IcalTimeZoneTransitionType {
    #[default]
    STANDARD,
    DAYLIGHT,
}

#[derive(Debug, Clone, Default)]
pub struct IcalTimeZoneTransition<const VERIFIED: bool = true> {
    pub transition: IcalTimeZoneTransitionType,
    pub properties: Vec<ContentLine>,
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

    fn get_comp_name(&self) -> &'static str {
        match self.transition {
            IcalTimeZoneTransitionType::STANDARD => "STANDARD",
            IcalTimeZoneTransitionType::DAYLIGHT => "DAYLIGHT",
        }
    }

    fn get_properties(&self) -> &Vec<ContentLine> {
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

    fn get_properties_mut(&mut self) -> &mut Vec<ContentLine> {
        &mut self.properties
    }

    #[cfg(not(tarpaulin_include))]
    fn add_sub_component<B: BufRead>(
        &mut self,
        value: &str,
        _: &mut PropertyParser<B>,
    ) -> Result<(), ParserError> {
        Err(ParserError::InvalidComponent(value.to_owned()))
    }

    fn build(
        self,
        _timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<IcalTimeZoneTransition<true>, ParserError> {
        Ok(IcalTimeZoneTransition {
            transition: self.transition,
            properties: self.properties,
        })
    }
}
