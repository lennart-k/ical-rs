use crate::parser::{
    Component, ComponentMut, GetProperty, IcalUIDProperty, ParserError, VcardANNIVERSARYProperty,
    VcardBDAYProperty, VcardFNProperty, VcardNProperty,
};
use crate::property::{ContentLine, PropertyParser};
use std::collections::HashMap;
use std::io::BufRead;

#[derive(Debug, Clone)]
pub struct VcardContact {
    pub uid: Option<String>,
    pub full_name: Vec<VcardFNProperty>,
    pub name: Option<VcardNProperty>,
    pub birthday: Option<VcardBDAYProperty>,
    pub anniversary: Option<VcardANNIVERSARYProperty>,
    pub properties: Vec<ContentLine>,
}

#[derive(Debug, Clone, Default)]
pub struct VcardContactBuilder {
    pub properties: Vec<ContentLine>,
}

impl VcardContact {
    pub fn get_uid(&self) -> Option<&str> {
        self.uid.as_deref()
    }
}

impl Component for VcardContactBuilder {
    const NAMES: &[&str] = &["VCARD"];
    type Unverified = VcardContactBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        self
    }
}

impl Component for VcardContact {
    const NAMES: &[&str] = &["VCARD"];
    type Unverified = VcardContactBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        VcardContactBuilder {
            properties: self.properties,
        }
    }
}

impl ComponentMut for VcardContactBuilder {
    type Verified = VcardContact;

    fn get_properties_mut(&mut self) -> &mut Vec<ContentLine> {
        &mut self.properties
    }

    fn add_sub_component<B: BufRead>(
        &mut self,
        name: &str,
        _: &mut PropertyParser<B>,
    ) -> Result<(), ParserError> {
        Err(ParserError::InvalidComponent(name.to_owned()))
    }

    fn build(
        self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Self::Verified, ParserError> {
        let uid = self
            .safe_get_optional(timezones)?
            .map(|IcalUIDProperty(uid, _)| uid);

        let name = self.safe_get_optional(timezones)?;
        let full_name = self.safe_get_all(timezones)?;
        let birthday = self.safe_get_optional(timezones)?;
        let anniversary = self.safe_get_optional(timezones)?;

        let verified = VcardContact {
            uid,
            name,
            full_name,
            birthday,
            anniversary,
            properties: self.properties,
        };

        Ok(verified)
    }
}
