use crate::{
    PropertyParser,
    component::{IcalCalendar, IcalEventBuilder, IcalJournalBuilder, IcalTodoBuilder},
    generator::Emitter,
    parser::{
        Component, ComponentMut, GetProperty, IcalUIDProperty, ParserError,
        ical::component::{IcalEvent, IcalJournal, IcalTimeZone, IcalTodo},
    },
    property::ContentLine,
    types::CalDateTime,
};
use chrono::{DateTime, Utc};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    io::BufRead,
};

#[derive(Debug, Clone)]
pub enum CalendarInnerData<E = IcalEvent, T = IcalTodo, J = IcalJournal> {
    Event(E, Vec<E>),
    Todo(T, Vec<T>),
    Journal(J, Vec<J>),
}

type CalendarInnerDataBuilder =
    CalendarInnerData<IcalEventBuilder, IcalTodoBuilder, IcalJournalBuilder>;

impl CalendarInnerData<IcalEvent, IcalTodo, IcalJournal> {
    pub fn get_uid(&self) -> &str {
        match self {
            Self::Event(main, _) => main.get_uid(),
            Self::Journal(main, _) => main.get_uid(),
            Self::Todo(main, _) => main.get_uid(),
        }
    }

    pub fn mutable(self) -> CalendarInnerDataBuilder {
        match self {
            Self::Event(event, overrides) => CalendarInnerData::Event(
                event.mutable(),
                overrides.into_iter().map(Component::mutable).collect(),
            ),
            Self::Todo(todo, overrides) => CalendarInnerData::Todo(
                todo.mutable(),
                overrides.into_iter().map(Component::mutable).collect(),
            ),
            Self::Journal(journal, overrides) => CalendarInnerData::Journal(
                journal.mutable(),
                overrides.into_iter().map(Component::mutable).collect(),
            ),
        }
    }

    pub fn get_tzids(&self) -> HashSet<&str> {
        match self {
            Self::Event(main, overrides) => main
                .get_tzids()
                .into_iter()
                .chain(overrides.iter().flat_map(|e| e.get_tzids()))
                .collect(),
            Self::Todo(main, overrides) => main
                .get_tzids()
                .into_iter()
                .chain(overrides.iter().flat_map(|e| e.get_tzids()))
                .collect(),
            Self::Journal(main, overrides) => main
                .get_tzids()
                .into_iter()
                .chain(overrides.iter().flat_map(|e| e.get_tzids()))
                .collect(),
        }
    }

    pub fn get_first_occurence(&self) -> Option<CalDateTime> {
        match self {
            Self::Event(main, overrides) => std::iter::once(&main.dtstart.0)
                .chain(overrides.iter().map(|over| &over.dtstart.0))
                .min(),
            Self::Todo(main, overrides) => std::iter::once(main.dtstart.as_ref().map(|dt| &dt.0))
                .chain(
                    overrides
                        .iter()
                        .map(|over| over.dtstart.as_ref().map(|dt| &dt.0)),
                )
                .flatten()
                .min(),
            Self::Journal(main, overrides) => {
                std::iter::once(main.dtstart.as_ref().map(|dt| &dt.0))
                    .chain(
                        overrides
                            .iter()
                            .map(|over| over.dtstart.as_ref().map(|dt| &dt.0)),
                    )
                    .flatten()
                    .min()
            }
        }
        .cloned()
        .map(Into::into)
    }
}

impl CalendarInnerDataBuilder {
    pub fn build(
        self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<CalendarInnerData, ParserError> {
        Ok(match self {
            Self::Event(event, overrides) => CalendarInnerData::Event(
                event.build(timezones)?,
                overrides
                    .into_iter()
                    .map(|builder| builder.build(timezones))
                    .collect::<Result<_, _>>()?,
            ),
            Self::Todo(todo, overrides) => CalendarInnerData::Todo(
                todo.build(timezones)?,
                overrides
                    .into_iter()
                    .map(|builder| builder.build(timezones))
                    .collect::<Result<_, _>>()?,
            ),
            Self::Journal(journal, overrides) => CalendarInnerData::Journal(
                journal.build(timezones)?,
                overrides
                    .into_iter()
                    .map(|builder| builder.build(timezones))
                    .collect::<Result<_, _>>()?,
            ),
        })
    }
}

#[derive(Debug, Clone)]
/// An ICAL calendar object.
pub struct IcalCalendarObject {
    properties: Vec<ContentLine>,
    inner: CalendarInnerData,
    vtimezones: HashMap<String, IcalTimeZone>,
    timezones: HashMap<String, Option<chrono_tz::Tz>>,
}

impl IcalCalendarObject {
    pub const fn get_inner(&self) -> &CalendarInnerData {
        &self.inner
    }

    pub fn get_uid(&self) -> &str {
        self.inner.get_uid()
    }

    pub const fn get_vtimezones(&self) -> &HashMap<String, IcalTimeZone> {
        &self.vtimezones
    }

    pub fn get_timezones(&self) -> &HashMap<String, Option<chrono_tz::Tz>> {
        &self.timezones
    }

    pub fn expand_recurrence(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Cow<'_, Self> {
        match &self.inner {
            CalendarInnerData::Event(main, overrides) => {
                let mut events = main.expand_recurrence(start, end, overrides);
                let first = events.remove(0);
                Cow::Owned(Self {
                    properties: self.properties.clone(),
                    inner: CalendarInnerData::Event(first, events),
                    timezones: HashMap::new(),
                    vtimezones: HashMap::new(),
                })
            }
            _ => Cow::Borrowed(self),
        }
    }

    pub fn get_tzids(&self) -> HashSet<&str> {
        self.inner.get_tzids()
    }

    pub fn add_to_calendar(self, cal: &mut IcalCalendar) {
        match self.inner {
            CalendarInnerData::Event(main, overrides) => {
                cal.events.push(main);
                cal.events.extend_from_slice(&overrides);
            }
            CalendarInnerData::Journal(main, overrides) => {
                cal.journals.push(main);
                cal.journals.extend_from_slice(&overrides);
            }
            CalendarInnerData::Todo(main, overrides) => {
                cal.todos.push(main);
                cal.todos.extend_from_slice(&overrides);
            }
        }
        cal.vtimezones.extend(self.vtimezones);
    }
}

#[derive(Debug, Clone, Default)]
/// An ICAL calendar object.
pub struct IcalCalendarObjectBuilder {
    properties: Vec<ContentLine>,
    inner: Option<CalendarInnerDataBuilder>,
    vtimezones: HashMap<String, IcalTimeZone<false>>,
}

impl IcalCalendarObjectBuilder {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
            vtimezones: HashMap::new(),
            inner: None,
        }
    }
}

impl Component for IcalCalendarObject {
    const NAMES: &[&str] = &["VCALENDAR"];
    type Unverified = IcalCalendarObjectBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalCalendarObjectBuilder {
            properties: self.properties,
            vtimezones: self
                .vtimezones
                .into_iter()
                .map(|(tzid, tz)| (tzid, tz.mutable()))
                .collect(),
            inner: Some(self.inner.mutable()),
        }
    }
}

impl Component for IcalCalendarObjectBuilder {
    const NAMES: &[&str] = &["VCALENDAR"];
    type Unverified = Self;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        self
    }
}

impl ComponentMut for IcalCalendarObjectBuilder {
    type Verified = IcalCalendarObject;

    fn get_properties_mut(&mut self) -> &mut Vec<ContentLine> {
        &mut self.properties
    }

    fn add_sub_component<B: BufRead>(
        &mut self,
        value: &str,
        line_parser: &mut PropertyParser<B>,
    ) -> Result<(), ParserError> {
        match value {
            "VEVENT" => {
                let event = IcalEventBuilder::from_parser(line_parser)?;
                match &mut self.inner {
                    // TODO: The main event is not necessarily the first component
                    Some(CalendarInnerData::Event(main, overrides)) => {
                        if event.safe_get_required::<IcalUIDProperty>(&HashMap::default())?
                            != main.safe_get_required::<IcalUIDProperty>(&HashMap::default())?
                        {
                            return Err(ParserError::InvalidComponent(value.to_owned()));
                        }
                        overrides.push(event);
                    }
                    None => {
                        self.inner = Some(CalendarInnerData::Event(event, vec![]));
                    }
                    _ => return Err(ParserError::InvalidComponent(value.to_owned())),
                };
            }
            "VTODO" => {
                let todo = IcalTodoBuilder::from_parser(line_parser)?;
                match &mut self.inner {
                    Some(CalendarInnerData::Todo(main, overrides)) => {
                        if todo.safe_get_required::<IcalUIDProperty>(&HashMap::default())?
                            != main.safe_get_required::<IcalUIDProperty>(&HashMap::default())?
                        {
                            return Err(ParserError::InvalidComponent(value.to_owned()));
                        }
                        overrides.push(todo);
                    }
                    None => {
                        self.inner = Some(CalendarInnerData::Todo(todo, vec![]));
                    }
                    _ => return Err(ParserError::InvalidComponent(value.to_owned())),
                };
            }
            "VJOURNAL" => {
                let journal = IcalJournalBuilder::from_parser(line_parser)?;
                match &mut self.inner {
                    Some(CalendarInnerData::Journal(main, overrides)) => {
                        if journal.safe_get_required::<IcalUIDProperty>(&HashMap::default())?
                            != main.safe_get_required::<IcalUIDProperty>(&HashMap::default())?
                        {
                            return Err(ParserError::InvalidComponent(value.to_owned()));
                        }
                        overrides.push(journal);
                    }
                    None => {
                        self.inner = Some(CalendarInnerData::Journal(journal, vec![]));
                    }
                    _ => return Err(ParserError::InvalidComponent(value.to_owned())),
                };
            }
            "VTIMEZONE" => {
                let timezone = IcalTimeZone::from_parser(line_parser)?;
                self.vtimezones.insert(
                    timezone
                        .clone()
                        .build(&HashMap::default())?
                        .get_tzid()
                        .to_owned(),
                    timezone,
                );
            }
            _ => return Err(ParserError::InvalidComponent(value.to_owned())),
        };

        Ok(())
    }

    fn build(
        self,
        _timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Self::Verified, ParserError> {
        let vtimezones: HashMap<String, IcalTimeZone> = self
            .vtimezones
            .into_iter()
            .map(|(tzid, tz)| tz.build(&HashMap::default()).map(|tz| (tzid, tz)))
            .collect::<Result<_, _>>()?;

        let timezones = HashMap::from_iter(
            vtimezones
                .iter()
                .map(|(name, value)| (name.clone(), value.into())),
        );

        Ok(IcalCalendarObject {
            properties: self.properties,
            vtimezones,
            inner: self
                .inner
                .ok_or(ParserError::NotComplete)?
                .build(&timezones)?,
            timezones,
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
                .vtimezones
                .values()
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
