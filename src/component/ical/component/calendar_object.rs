use crate::{
    ContentLineParser,
    component::{
        Component, ComponentMut, IcalCalendar, IcalEvent, IcalEventBuilder, IcalJournal,
        IcalJournalBuilder, IcalTimeZone, IcalTodo, IcalTodoBuilder,
    },
    generator::Emitter,
    parser::{ContentLine, ParserError, ParserOptions},
    property::{GetProperty, IcalCALSCALEProperty, IcalPRODIDProperty, IcalVERSIONProperty},
    types::CalDateTime,
};
use chrono::{DateTime, Utc};
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap, HashSet},
};

#[derive(Debug, Clone)]
pub enum CalendarInnerData {
    Event(IcalEvent, Vec<IcalEvent>),
    Todo(IcalTodo, Vec<IcalTodo>),
    Journal(IcalJournal, Vec<IcalJournal>),
}

#[derive(Debug, Clone)]
pub enum CalendarInnerDataBuilder {
    Event(Vec<IcalEventBuilder>),
    Todo(Vec<IcalTodoBuilder>),
    Journal(Vec<IcalJournalBuilder>),
}

impl CalendarInnerDataBuilder {
    pub fn get_tzids(&self) -> HashSet<&str> {
        match self {
            Self::Event(events) => events
                .iter()
                .flat_map(IcalEventBuilder::get_tzids)
                .collect(),
            Self::Todo(todos) => todos.iter().flat_map(IcalTodoBuilder::get_tzids).collect(),

            Self::Journal(journals) => journals
                .iter()
                .flat_map(IcalJournalBuilder::get_tzids)
                .collect(),
        }
    }
}

impl CalendarInnerData {
    pub fn get_uid(&self) -> &str {
        match self {
            Self::Event(main, _) => main.get_uid(),
            Self::Journal(main, _) => main.get_uid(),
            Self::Todo(main, _) => main.get_uid(),
        }
    }

    pub fn mutable(self) -> CalendarInnerDataBuilder {
        match self {
            Self::Event(main, overrides) => CalendarInnerDataBuilder::Event(
                std::iter::once(main.mutable())
                    .chain(overrides.into_iter().map(Component::mutable))
                    .collect(),
            ),
            Self::Todo(main, overrides) => CalendarInnerDataBuilder::Todo(
                std::iter::once(main.mutable())
                    .chain(overrides.into_iter().map(Component::mutable))
                    .collect(),
            ),
            Self::Journal(main, overrides) => CalendarInnerDataBuilder::Journal(
                std::iter::once(main.mutable())
                    .chain(overrides.into_iter().map(Component::mutable))
                    .collect(),
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
                .chain(std::iter::once(main.due.as_ref().map(|dt| &dt.0)))
                .chain(
                    overrides
                        .iter()
                        .flat_map(|over| {
                            [
                                over.dtstart.as_ref().map(|dt| &dt.0),
                                over.due.as_ref().map(|dt| &dt.0),
                            ]
                        }),
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

    pub fn get_last_occurence(&self) -> Option<CalDateTime> {
        match self {
            Self::Event(main, overrides) => {
                if main.has_rruleset() {
                    return None;
                }
                std::iter::once(main)
                    .chain(overrides.iter())
                    .flat_map(IcalEvent::get_last_occurence)
                    .max()
                    .map(Into::into)
            }
            Self::Todo(main, overrides) => {
                if main.has_rruleset() {
                    return None;
                }
                std::iter::once(main)
                    .chain(overrides.iter())
                    .flat_map(IcalTodo::get_last_occurence)
                    .max()
                    .map(Into::into)
            }
            Self::Journal(_main, _overrides) => None,
        }
    }

    pub fn from_events(mut events: Vec<IcalEvent>) -> Result<Self, ParserError> {
        let main_idx = events
            .iter()
            .position(IcalEvent::has_rruleset)
            .unwrap_or_default();
        let main = events.remove(main_idx);
        if events.iter().any(|o| o.get_uid() != main.get_uid()) {
            return Err(ParserError::DifferingUIDs);
        }
        if events.iter().any(IcalEvent::has_rruleset) {
            return Err(ParserError::MultipleMainObjects);
        }
        let overrides = events;
        if overrides.iter().any(|e| e.recurid.is_none()) {
            return Err(ParserError::MissingRecurId);
        }
        if overrides.iter().any(|e| e.get_uid() != main.get_uid()) {
            return Err(ParserError::DifferingUIDs);
        }
        Ok(Self::Event(main, overrides))
    }

    pub fn from_todos(mut todos: Vec<IcalTodo>) -> Result<Self, ParserError> {
        let main_idx = todos
            .iter()
            .position(IcalTodo::has_rruleset)
            .unwrap_or_default();
        let main = todos.remove(main_idx);
        if todos.iter().any(|o| o.get_uid() != main.get_uid()) {
            return Err(ParserError::DifferingUIDs);
        }
        if todos.iter().any(IcalTodo::has_rruleset) {
            return Err(ParserError::MultipleMainObjects);
        }
        let overrides = todos;
        if overrides.iter().any(|t| t.recurid.is_none()) {
            return Err(ParserError::MissingRecurId);
        }
        if overrides.iter().any(|e| e.get_uid() != main.get_uid()) {
            return Err(ParserError::DifferingUIDs);
        }
        Ok(Self::Todo(main, overrides))
    }

    pub fn from_journals(mut journals: Vec<IcalJournal>) -> Result<Self, ParserError> {
        let main_idx = journals
            .iter()
            .position(IcalJournal::has_rruleset)
            .unwrap_or_default();
        let main = journals.remove(main_idx);
        if journals.iter().any(|o| o.get_uid() != main.get_uid()) {
            return Err(ParserError::DifferingUIDs);
        }
        if journals.iter().any(IcalJournal::has_rruleset) {
            return Err(ParserError::MultipleMainObjects);
        }
        let overrides = journals;
        if overrides.iter().any(|j| j.recurid.is_none()) {
            return Err(ParserError::MissingRecurId);
        }
        if overrides.iter().any(|e| e.get_uid() != main.get_uid()) {
            return Err(ParserError::DifferingUIDs);
        }
        Ok(Self::Journal(main, overrides))
    }
}

impl CalendarInnerDataBuilder {
    pub fn build(
        self,
        options: &ParserOptions,
        timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<CalendarInnerData, ParserError> {
        match self {
            Self::Event(events) => {
                let events = events
                    .into_iter()
                    .map(|builder| builder.build(options, timezones))
                    .collect::<Result<Vec<_>, _>>()?;
                CalendarInnerData::from_events(events)
            }
            Self::Todo(todos) => {
                let todos = todos
                    .into_iter()
                    .map(|builder| builder.build(options, timezones))
                    .collect::<Result<Vec<_>, _>>()?;
                CalendarInnerData::from_todos(todos)
            }
            Self::Journal(journals) => {
                let journals = journals
                    .into_iter()
                    .map(|builder| builder.build(options, timezones))
                    .collect::<Result<Vec<_>, _>>()?;
                CalendarInnerData::from_journals(journals)
            }
        }
    }
}

#[derive(Debug, Clone)]
/// An ICAL calendar object.
pub struct IcalCalendarObject {
    pub properties: Vec<ContentLine>,
    pub(crate) inner: CalendarInnerData,
    pub(crate) vtimezones: BTreeMap<String, IcalTimeZone>,
    pub(crate) timezones: HashMap<String, Option<chrono_tz::Tz>>,
}

impl IcalCalendarObject {
    pub const fn get_inner(&self) -> &CalendarInnerData {
        &self.inner
    }

    pub fn get_uid(&self) -> &str {
        self.inner.get_uid()
    }

    pub const fn get_vtimezones(&self) -> &BTreeMap<String, IcalTimeZone> {
        &self.vtimezones
    }

    pub fn get_timezones(&self) -> &HashMap<String, Option<chrono_tz::Tz>> {
        &self.timezones
    }

    pub fn expand_recurrence(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Option<Cow<'_, Self>> {
        match &self.inner {
            CalendarInnerData::Event(main, overrides) => {
                let mut events = main.expand_recurrence(start, end, overrides);
                if events.is_empty() {
                    return None;
                }
                let first = events.remove(0);
                Some(Cow::Owned(Self {
                    properties: self.properties.clone(),
                    inner: CalendarInnerData::Event(first, events),
                    timezones: HashMap::new(),
                    vtimezones: BTreeMap::new(),
                }))
            }
            _ => Some(Cow::Borrowed(self)),
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
        cal.timezones.extend(self.timezones);
    }
}

#[derive(Debug, Clone, Default)]
/// An ICAL calendar object.
pub struct IcalCalendarObjectBuilder {
    pub properties: Vec<ContentLine>,
    pub inner: Option<CalendarInnerDataBuilder>,
    pub vtimezones: BTreeMap<String, IcalTimeZone>,
}

impl IcalCalendarObjectBuilder {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
            vtimezones: BTreeMap::new(),
            inner: None,
        }
    }
}

impl Component for IcalCalendarObject {
    const NAMES: &[&str] = &["VCALENDAR"];
    type Builder = IcalCalendarObjectBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Builder {
        IcalCalendarObjectBuilder {
            properties: self.properties,
            vtimezones: self.vtimezones,
            inner: Some(self.inner.mutable()),
        }
    }
}

impl Component for IcalCalendarObjectBuilder {
    const NAMES: &[&str] = &["VCALENDAR"];
    type Builder = Self;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Builder {
        self
    }
}

impl ComponentMut for IcalCalendarObjectBuilder {
    type Verified = IcalCalendarObject;

    fn get_properties_mut(&mut self) -> &mut Vec<ContentLine> {
        &mut self.properties
    }

    fn add_sub_component<'a, I: Iterator<Item = Cow<'a, [u8]>>>(
        &mut self,
        value: &str,
        line_parser: &mut ContentLineParser<'a, I>,
        options: &ParserOptions,
    ) -> Result<(), ParserError> {
        match value {
            "VEVENT" => {
                let event = IcalEventBuilder::from_parser(line_parser, options)?;
                match &mut self.inner {
                    Some(CalendarInnerDataBuilder::Event(events)) => {
                        events.push(event);
                    }
                    None => {
                        self.inner = Some(CalendarInnerDataBuilder::Event(vec![event]));
                    }
                    _ => return Err(ParserError::InvalidComponent(value.to_owned())),
                };
            }
            "VTODO" => {
                let todo = IcalTodoBuilder::from_parser(line_parser, options)?;
                match &mut self.inner {
                    Some(CalendarInnerDataBuilder::Todo(todos)) => {
                        todos.push(todo);
                    }
                    None => {
                        self.inner = Some(CalendarInnerDataBuilder::Todo(vec![todo]));
                    }
                    _ => return Err(ParserError::InvalidComponent(value.to_owned())),
                };
            }
            "VJOURNAL" => {
                let journal = IcalJournalBuilder::from_parser(line_parser, options)?;
                match &mut self.inner {
                    Some(CalendarInnerDataBuilder::Journal(journals)) => {
                        journals.push(journal);
                    }
                    None => {
                        self.inner = Some(CalendarInnerDataBuilder::Journal(vec![journal]));
                    }
                    _ => return Err(ParserError::InvalidComponent(value.to_owned())),
                };
            }
            "VTIMEZONE" => {
                let timezone =
                    IcalTimeZone::from_parser(line_parser, options)?.build(options, None)?;
                self.vtimezones
                    .insert(timezone.get_tzid().to_owned(), timezone);
            }
            _ => return Err(ParserError::InvalidComponent(value.to_owned())),
        };

        Ok(())
    }

    fn build(
        self,
        options: &ParserOptions,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Self::Verified, ParserError> {
        let _version: IcalVERSIONProperty = self.safe_get_required(None)?;
        let _prodid: IcalPRODIDProperty = self.safe_get_required(None)?;
        let _calscale: Option<IcalCALSCALEProperty> = self.safe_get_optional(None)?;

        #[allow(unused_mut)]
        let mut vtimezones: BTreeMap<String, IcalTimeZone> = self.vtimezones;
        let inner = self.inner.ok_or(ParserError::NotComplete)?;

        #[allow(unused_mut)]
        let mut timezones = HashMap::from_iter(
            vtimezones
                .iter()
                .map(|(name, value)| (name.clone(), value.into())),
        );

        if options.rfc7809 {
            // Populate our map of chrono timezones with those we can populate ourselves
            use std::str::FromStr;
            for tzid in inner.get_tzids() {
                if let Ok(tz) = chrono_tz::Tz::from_str(tzid) {
                    timezones.insert(tzid.to_owned(), Some(tz));
                }
            }
        }
        let inner = inner.build(options, Some(&timezones))?;
        if options.rfc7809 {
            for tzid in inner.get_tzids() {
                if !vtimezones.contains_key(tzid)
                    && let Some(tz) = IcalTimeZone::from_tzid(tzid)
                    && let Some(start) = inner.get_first_occurence()
                {
                    // Just to be safe
                    let trunc_start = start.utc() - chrono::Duration::days(365);
                    let tz = tz.clone().truncate(trunc_start);
                    vtimezones.insert(tzid.to_owned(), tz);
                }
            }
        }

        Ok(IcalCalendarObject {
            properties: self.properties,
            vtimezones,
            inner,
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
