use crate::{
    PropertyParser,
    generator::Emitter,
    parser::{
        Component, ComponentMut, ParserError,
        ical::component::{IcalEvent, IcalJournal, IcalTimeZone, IcalTodo},
    },
    property::Property,
    types::{CalDateOrDateTime, CalDateTimeError},
};
use chrono::{DateTime, Utc};
use std::{collections::HashMap, io::BufRead, sync::OnceLock};

#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum CalendarInnerData<const VERIFIED: bool = true> {
    Event(IcalEvent<VERIFIED>, Vec<IcalEvent<VERIFIED>>),
    Todo(IcalTodo<VERIFIED>, Vec<IcalTodo<VERIFIED>>),
    Journal(IcalJournal<VERIFIED>, Vec<IcalJournal<VERIFIED>>),
}

impl CalendarInnerData<true> {
    pub fn mutable(self) -> CalendarInnerData<false> {
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
}

impl CalendarInnerData<false> {
    pub fn verify(self) -> Result<CalendarInnerData<true>, ParserError> {
        Ok(match self {
            Self::Event(event, overrides) => CalendarInnerData::Event(
                event.verify()?,
                overrides
                    .into_iter()
                    .map(ComponentMut::verify)
                    .collect::<Result<_, _>>()?,
            ),
            Self::Todo(todo, overrides) => CalendarInnerData::Todo(
                todo.verify()?,
                overrides
                    .into_iter()
                    .map(ComponentMut::verify)
                    .collect::<Result<_, _>>()?,
            ),
            Self::Journal(journal, overrides) => CalendarInnerData::Journal(
                journal.verify()?,
                overrides
                    .into_iter()
                    .map(ComponentMut::verify)
                    .collect::<Result<_, _>>()?,
            ),
        })
    }
}

impl CalendarInnerData {
    pub fn get_uid(&self) -> &str {
        match self {
            Self::Event(main, _) => main.get_uid(),
            Self::Todo(main, _) => main.get_uid(),
            Self::Journal(main, _) => main.get_uid(),
        }
    }

    pub fn get_first_occurence(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Option<CalDateOrDateTime>, CalDateTimeError> {
        // TODO: We actually have to check whether dtstart is overriden for the first occurence
        match self {
            Self::Event(main, overrides) => Ok(std::iter::once(main)
                .chain(overrides.iter())
                .map(|event| event.get_dtstart(timezones))
                .min()
                .unwrap()),
            Self::Todo(main, _overrides) => main.get_dtstart(timezones),
            Self::Journal(main, _overrides) => main.get_dtstart(timezones),
        }
    }

    /// Tries to give an estimate for the last occurence
    /// Only for optimisation of database queries by doing some prefiltering
    pub fn get_last_occurence(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Option<CalDateOrDateTime>, CalDateTimeError> {
        // TODO: We should verify before that these are not actually set
        match self {
            Self::Event(main, _) => main.get_last_occurence(timezones),
            _ => Ok(None),
        }
    }

    pub fn get_dtend(
        &self,
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Option<CalDateOrDateTime> {
        // TODO: We should verify before that these are not actually set
        match self {
            Self::Event(main, _) => main.get_dtend(timezones),
            _ => None,
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
    vtimezones: HashMap<String, IcalTimeZone>,
    #[cfg_attr(
        feature = "rkyv",
    rkyv(with = rkyv::with::Skip)
    )]
    timezones: OnceLock<HashMap<String, Option<chrono_tz::Tz>>>,
}

impl IcalCalendarObject {
    pub fn get_uid(&self) -> &str {
        self.inner.get_uid()
    }

    pub const fn get_inner(&self) -> &CalendarInnerData {
        &self.inner
    }

    pub const fn get_vtimezones(&self) -> &HashMap<String, IcalTimeZone> {
        &self.vtimezones
    }

    pub fn get_timezones(&self) -> &HashMap<String, Option<chrono_tz::Tz>> {
        self.timezones.get_or_init(|| {
            HashMap::from_iter(
                self.get_vtimezones()
                    .iter()
                    .map(|(name, value)| (name.clone(), value.try_into().ok())),
            )
        })
    }

    pub fn get_first_occurence(&self) -> Result<Option<CalDateOrDateTime>, CalDateTimeError> {
        self.inner.get_first_occurence(self.get_timezones())
    }

    pub fn get_last_occurence(&self) -> Result<Option<CalDateOrDateTime>, CalDateTimeError> {
        // TODO: implement
        self.inner.get_last_occurence(self.get_timezones())
    }

    pub fn expand_recurrence(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Result<Self, ParserError> {
        // Only events can be expanded
        match &self.inner {
            CalendarInnerData::Event(main, overrides) => {
                // TODO: Fix error
                let mut events: Vec<IcalEvent> =
                    main.expand_recurrence(start, end, self.get_timezones(), overrides)?;
                Ok(Self {
                    properties: self.properties.clone(),
                    inner: CalendarInnerData::Event(events.remove(0), events),
                    vtimezones: self.vtimezones.clone(),
                    timezones: OnceLock::from(self.get_timezones().clone()),
                })
            }
            _ => Ok(self.clone()),
        }
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
    inner: Option<CalendarInnerData<false>>,
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

    fn get_properties(&self) -> &Vec<Property> {
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
                let event = IcalEvent::from_parser(line_parser)?;
                match &mut self.inner {
                    // TODO: The main event is not necessarily the first component
                    Some(CalendarInnerData::Event(main, overrides)) => {
                        if event.safe_get_uid()? != main.safe_get_uid()? {
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
                let todo = IcalTodo::from_parser(line_parser)?;
                match &mut self.inner {
                    Some(CalendarInnerData::Todo(main, overrides)) => {
                        // if todo.get_uid() != main.get_uid() {
                        //     return Err(ParserError::InvalidComponent(value.to_owned()));
                        // }
                        overrides.push(todo);
                    }
                    None => {
                        self.inner = Some(CalendarInnerData::Todo(todo, vec![]));
                    }
                    _ => return Err(ParserError::InvalidComponent(value.to_owned())),
                };
            }
            "VJOURNAL" => {
                let journal = IcalJournal::from_parser(line_parser)?;
                match &mut self.inner {
                    Some(CalendarInnerData::Journal(main, overrides)) => {
                        // if journal.get_uid() != main.get_uid() {
                        //     return Err(ParserError::InvalidComponent(value.to_owned()));
                        // }
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
                self.vtimezones
                    .insert(timezone.clone().verify()?.get_tzid().to_owned(), timezone);
            }
            _ => return Err(ParserError::InvalidComponent(value.to_owned())),
        };

        Ok(())
    }

    fn verify(self) -> Result<Self::Verified, ParserError> {
        Ok(IcalCalendarObject {
            properties: self.properties,
            vtimezones: self
                .vtimezones
                .into_iter()
                .map(|(tzid, tz)| tz.verify().map(|tz| (tzid, tz)))
                .collect::<Result<_, _>>()?,
            inner: self.inner.ok_or(ParserError::NotComplete)?.verify()?,
            timezones: OnceLock::new(),
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
