// Sys mods
use std::cell::RefCell;
#[cfg(feature = "chrono-tz")]
use std::convert::TryFrom;
use std::io::BufRead;

#[cfg(feature = "serde-derive")]
extern crate serde;

// Internal mods
use crate::parser::Component;
use crate::parser::ComponentMut;
use crate::parser::ParserError;
use crate::property::{Property, PropertyParser};

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
/// An ICAL calendar.
pub struct IcalCalendar<const VERIFIED: bool = true> {
    pub properties: Vec<Property>,
    pub events: Vec<IcalEvent>,
    pub alarms: Vec<IcalAlarm>,
    pub todos: Vec<IcalTodo>,
    pub journals: Vec<IcalJournal>,
    pub free_busys: Vec<IcalFreeBusy>,
    pub timezones: Vec<IcalTimeZone>,
}

impl IcalCalendar<false> {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
            events: Vec::new(),
            alarms: Vec::new(),
            todos: Vec::new(),
            journals: Vec::new(),
            free_busys: Vec::new(),
            timezones: Vec::new(),
        }
    }
}

impl<const VERIFIED: bool> Component for IcalCalendar<VERIFIED> {
    type Unverified = IcalCalendar<false>;

    fn get_properties(&self) -> &Vec<Property> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalCalendar {
            properties: self.properties,
            events: self.events,
            alarms: self.alarms,
            todos: self.todos,
            journals: self.journals,
            free_busys: self.free_busys,
            timezones: self.timezones,
        }
    }
}

impl ComponentMut for IcalCalendar<false> {
    type Verified = IcalCalendar<true>;

    fn get_properties_mut(&mut self) -> &mut Vec<Property> {
        &mut self.properties
    }

    fn add_sub_component<B: BufRead>(
        &mut self,
        value: &str,
        line_parser: &RefCell<PropertyParser<B>>,
    ) -> Result<(), ParserError> {
        match value {
            "VALARM" => {
                let mut alarm = IcalAlarm::new();
                alarm.parse(line_parser)?;
                self.alarms.push(alarm.verify()?);
            }
            "VEVENT" => {
                let mut event = IcalEvent::new();
                event.parse(line_parser)?;
                self.events.push(event.verify()?);
            }
            "VTODO" => {
                let mut todo = IcalTodo::new();
                todo.parse(line_parser)?;
                self.todos.push(todo.verify()?);
            }
            "VJOURNAL" => {
                let mut journal = IcalJournal::new();
                journal.parse(line_parser)?;
                self.journals.push(journal.verify()?);
            }
            "VFREEBUSY" => {
                let mut free_busy = IcalFreeBusy::new();
                free_busy.parse(line_parser)?;
                self.free_busys.push(free_busy.verify()?);
            }
            "VTIMEZONE" => {
                let mut timezone = IcalTimeZone::new();
                timezone.parse(line_parser)?;
                self.timezones.push(timezone.verify()?);
            }
            _ => return Err(ParserError::InvalidComponent),
        };

        Ok(())
    }

    fn verify(self) -> Result<Self::Verified, ParserError> {
        Ok(IcalCalendar {
            properties: self.properties,
            events: self.events,
            alarms: self.alarms,
            todos: self.todos,
            journals: self.journals,
            free_busys: self.free_busys,
            timezones: self.timezones,
        })
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
pub struct IcalAlarm<const VERIFIED: bool = true> {
    pub properties: Vec<Property>,
}

impl IcalAlarm<false> {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }
}

impl<const VERIFIED: bool> Component for IcalAlarm<VERIFIED> {
    type Unverified = IcalAlarm<false>;

    fn get_properties(&self) -> &Vec<Property> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalAlarm {
            properties: self.properties,
        }
    }
}

impl ComponentMut for IcalAlarm<false> {
    type Verified = IcalAlarm<true>;

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

    fn verify(self) -> Result<IcalAlarm<true>, ParserError> {
        Ok(IcalAlarm {
            properties: self.properties,
        })
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
pub struct IcalEvent<const VERIFIED: bool = true> {
    pub properties: Vec<Property>,
    pub alarms: Vec<IcalAlarm>,
}

impl IcalEvent<false> {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
            alarms: Vec::new(),
        }
    }
}

impl<const VERIFIED: bool> Component for IcalEvent<VERIFIED> {
    type Unverified = IcalEvent<false>;

    fn get_properties(&self) -> &Vec<Property> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalEvent {
            properties: self.properties,
            alarms: self.alarms,
        }
    }
}

impl ComponentMut for IcalEvent<false> {
    type Verified = IcalEvent<true>;

    fn get_properties_mut(&mut self) -> &mut Vec<Property> {
        &mut self.properties
    }

    fn add_sub_component<B: BufRead>(
        &mut self,
        value: &str,
        line_parser: &RefCell<PropertyParser<B>>,
    ) -> Result<(), ParserError> {
        match value {
            "VALARM" => {
                let mut alarm = IcalAlarm::new();
                alarm.parse(line_parser)?;
                self.alarms.push(alarm.verify()?);
            }
            _ => return Err(ParserError::InvalidComponent),
        };

        Ok(())
    }

    fn verify(self) -> Result<IcalEvent<true>, ParserError> {
        Ok(IcalEvent {
            properties: self.properties,
            alarms: self.alarms,
        })
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
pub struct IcalJournal<const VERIFIED: bool = true> {
    pub properties: Vec<Property>,
}

impl IcalJournal<false> {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }
}

impl<const VERIFIED: bool> Component for IcalJournal<VERIFIED> {
    type Unverified = IcalJournal<false>;

    fn get_properties(&self) -> &Vec<Property> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalJournal {
            properties: self.properties,
        }
    }
}

impl ComponentMut for IcalJournal<false> {
    type Verified = IcalJournal<true>;

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

    fn verify(self) -> Result<IcalJournal<true>, ParserError> {
        Ok(IcalJournal {
            properties: self.properties,
        })
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
pub struct IcalTodo<const VERIFIED: bool = true> {
    pub properties: Vec<Property>,
    pub alarms: Vec<IcalAlarm>,
}

impl IcalTodo<false> {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
            alarms: Vec::new(),
        }
    }
}

impl<const VERIFIED: bool> Component for IcalTodo<VERIFIED> {
    type Unverified = IcalTodo<false>;

    fn get_properties(&self) -> &Vec<Property> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalTodo {
            properties: self.properties,
            alarms: self.alarms,
        }
    }
}

impl ComponentMut for IcalTodo<false> {
    type Verified = IcalTodo<true>;

    fn get_properties_mut(&mut self) -> &mut Vec<Property> {
        &mut self.properties
    }

    fn add_sub_component<B: BufRead>(
        &mut self,
        value: &str,
        line_parser: &RefCell<PropertyParser<B>>,
    ) -> Result<(), ParserError> {
        match value {
            "VALARM" => {
                let mut alarm = IcalAlarm::new();
                alarm.parse(line_parser)?;
                self.alarms.push(alarm.verify()?);
            }
            _ => return Err(ParserError::InvalidComponent),
        };

        Ok(())
    }

    fn verify(self) -> Result<IcalTodo<true>, ParserError> {
        Ok(IcalTodo {
            properties: self.properties,
            alarms: self.alarms,
        })
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
pub struct IcalTimeZone<const VERIFIED: bool = true> {
    pub properties: Vec<Property>,
    pub transitions: Vec<IcalTimeZoneTransition<true>>,
}

impl IcalTimeZone {
    pub fn get_tzid(&self) -> &str {
        self.get_property("TZID")
            .expect("we already verified this exists")
            .value
            .as_ref()
            .expect("we already verified this exists")
    }

    /// This is a common property containing a timezone identifier from the IANA TZDB
    pub fn get_lic_location(&self) -> Option<&str> {
        self.get_property("X-LIC-LOCATION")
            .as_ref()
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

        Ok(IcalTimeZone {
            properties: self.properties,
            transitions: self.transitions,
        })
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

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
pub struct IcalFreeBusy<const VERIFIED: bool = true> {
    pub properties: Vec<Property>,
}

impl IcalFreeBusy<false> {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }
}

impl<const VERIFIED: bool> Component for IcalFreeBusy<VERIFIED> {
    type Unverified = IcalFreeBusy<false>;

    fn get_properties(&self) -> &Vec<Property> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalFreeBusy {
            properties: self.properties,
        }
    }
}

impl ComponentMut for IcalFreeBusy<false> {
    type Verified = IcalFreeBusy<true>;

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

    fn verify(self) -> Result<IcalFreeBusy<true>, ParserError> {
        Ok(IcalFreeBusy {
            properties: self.properties,
        })
    }
}
