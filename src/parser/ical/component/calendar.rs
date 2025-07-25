use std::{cell::RefCell, io::BufRead};

use crate::{
    PropertyParser,
    parser::{
        Component, ComponentMut, ParserError,
        ical::component::{
            IcalAlarm, IcalEvent, IcalFreeBusy, IcalJournal, IcalTimeZone, IcalTodo,
        },
    },
    property::Property,
};

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
    const NAMES: &[&str] = &["VCALENDAR"];
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
