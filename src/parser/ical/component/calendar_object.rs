use crate::{
    PropertyParser,
    generator::Emitter,
    parser::{
        Component, ComponentMut, ParserError,
        ical::component::{IcalEvent, IcalJournal, IcalTimeZone, IcalTodo},
    },
    property::Property,
};
use std::io::BufRead;

#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
enum CalendarInnerData {
    Event(IcalEvent, Vec<IcalEvent>),
    Todo(IcalTodo, Vec<IcalTodo>),
    Journal(IcalJournal, Vec<IcalJournal>),
}

impl CalendarInnerData {
    pub fn get_uid(&self) -> &str {
        match self {
            Self::Event(main, _) => main.get_uid(),
            Self::Todo(main, _) => main.get_uid(),
            Self::Journal(main, _) => main.get_uid(),
        }
    }
}

#[derive(Debug, Clone)]
/// An ICAL calendar object.
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct IcalCalendarObject {
    properties: Vec<Property>,
    inner: CalendarInnerData,
    timezones: Vec<IcalTimeZone>,
}

impl IcalCalendarObject {
    pub fn get_uid(&self) -> &str {
        self.inner.get_uid()
    }
}

#[derive(Debug, Clone, Default)]
/// An ICAL calendar object.
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct IcalCalendarObjectBuilder {
    properties: Vec<Property>,
    inner: Option<CalendarInnerData>,
    timezones: Vec<IcalTimeZone>,
}

impl IcalCalendarObjectBuilder {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
            timezones: Vec::new(),
            inner: None,
        }
    }
}

impl Component for IcalCalendarObject {
    const NAMES: &[&str] = &["VCALENDAR"];
    type Unverified = IcalCalendarObjectBuilder;

    fn get_properties(&self) -> &Vec<Property> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalCalendarObjectBuilder {
            properties: self.properties,
            timezones: self.timezones,
            inner: Some(self.inner),
        }
    }
}

impl Component for IcalCalendarObjectBuilder {
    const NAMES: &[&str] = &["VCALENDAR"];
    type Unverified = Self;

    fn get_properties(&self) -> &Vec<Property> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        self
    }
}

impl ComponentMut for IcalCalendarObjectBuilder {
    type Verified = IcalCalendarObject;

    fn get_properties_mut(&mut self) -> &mut Vec<Property> {
        &mut self.properties
    }

    fn add_sub_component<B: BufRead>(
        &mut self,
        value: &str,
        line_parser: &mut PropertyParser<B>,
    ) -> Result<(), ParserError> {
        match value {
            "VEVENT" => {
                let event = IcalEvent::from_parser(line_parser)?.verify()?;
                match &mut self.inner {
                    Some(CalendarInnerData::Event(main, overrides)) => {
                        if event.get_uid() != main.get_uid() {
                            return Err(ParserError::InvalidComponent);
                        }
                        overrides.push(event);
                    }
                    None => {
                        self.inner = Some(CalendarInnerData::Event(event, vec![]));
                    }
                    _ => return Err(ParserError::InvalidComponent),
                };
            }
            "VTODO" => {
                let todo = IcalTodo::from_parser(line_parser)?.verify()?;
                match &mut self.inner {
                    Some(CalendarInnerData::Todo(main, overrides)) => {
                        if todo.get_uid() != main.get_uid() {
                            return Err(ParserError::InvalidComponent);
                        }
                        overrides.push(todo);
                    }
                    None => {
                        self.inner = Some(CalendarInnerData::Todo(todo, vec![]));
                    }
                    _ => return Err(ParserError::InvalidComponent),
                };
            }
            "VJOURNAL" => {
                let journal = IcalJournal::from_parser(line_parser)?.verify()?;
                match &mut self.inner {
                    Some(CalendarInnerData::Journal(main, overrides)) => {
                        if journal.get_uid() != main.get_uid() {
                            return Err(ParserError::InvalidComponent);
                        }
                        overrides.push(journal);
                    }
                    None => {
                        self.inner = Some(CalendarInnerData::Journal(journal, vec![]));
                    }
                    _ => return Err(ParserError::InvalidComponent),
                };
            }
            "VTIMEZONE" => {
                let timezone = IcalTimeZone::from_parser(line_parser)?.verify()?;
                self.timezones.push(timezone);
            }
            _ => return Err(ParserError::InvalidComponent),
        };

        Ok(())
    }

    fn verify(self) -> Result<Self::Verified, ParserError> {
        Ok(IcalCalendarObject {
            properties: self.properties,
            timezones: self.timezones,
            inner: self.inner.ok_or(ParserError::NotComplete)?,
        })
    }
}

impl Emitter for CalendarInnerData {
    fn generate(&self) -> String {
        match self {
            Self::Event(main, overrides) => {
                main.generate() + &overrides.iter().map(Emitter::generate).collect::<String>()
            }
            Self::Todo(main, overrides) => {
                main.generate() + &overrides.iter().map(Emitter::generate).collect::<String>()
            }
            Self::Journal(main, overrides) => {
                main.generate() + &overrides.iter().map(Emitter::generate).collect::<String>()
            }
        }
    }
}

impl Emitter for IcalCalendarObject {
    fn generate(&self) -> String {
        format!(
            "BEGIN:VCALENDAR\r\n{props}{timezones}{inner}END:VCALENDAR\r\n",
            timezones = &self
                .timezones
                .iter()
                .map(Emitter::generate)
                .collect::<String>(),
            props = &self
                .properties
                .iter()
                .map(Emitter::generate)
                .collect::<String>(),
            inner = self.inner.generate()
        )
    }
}
