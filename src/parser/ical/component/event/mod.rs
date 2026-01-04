use crate::{component::IcalAlarm, parser::Component, property::ContentLine};
use itertools::Itertools;

pub use builder::IcalEventBuilder;
mod builder;

#[derive(Debug, Clone, Default)]
pub struct IcalEvent {
    pub uid: String,
    pub properties: Vec<ContentLine>,
    pub alarms: Vec<IcalAlarm>,
}

impl IcalEvent {
    pub fn get_uid(&self) -> &str {
        &self.uid
    }
}

impl Component for IcalEvent {
    const NAMES: &[&str] = &["VEVENT"];
    type Unverified = IcalEventBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalEventBuilder {
            properties: self.properties,
            alarms: self.alarms.into_iter().map(Component::mutable).collect(),
        }
    }
}

impl IcalEvent {
    pub fn get_tzids(&self) -> Vec<&str> {
        self.properties
            .iter()
            .filter_map(|prop| prop.get_tzid())
            .unique()
            .collect()
    }
}
