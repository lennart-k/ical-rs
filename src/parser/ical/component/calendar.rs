use crate::{
    PropertyParser,
    component::{
        CalendarInnerData, IcalAlarmBuilder, IcalCalendarObject, IcalEventBuilder,
        IcalFreeBusyBuilder, IcalJournalBuilder, IcalTodoBuilder,
    },
    parser::{
        Calscale, Component, ComponentMut, GetProperty, IcalCALSCALEProperty, IcalPRODIDProperty,
        IcalVERSIONProperty, IcalVersion, ParserError,
        ical::component::{
            IcalAlarm, IcalEvent, IcalFreeBusy, IcalJournal, IcalTimeZone, IcalTodo,
        },
    },
    property::ContentLine,
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    io::BufRead,
};

#[derive(Debug, Clone, Default)]
/// An ICAL calendar.
pub struct IcalCalendar<
    const VERIFIED: bool = true,
    A = IcalAlarm,
    E = IcalEvent,
    F = IcalFreeBusy,
    J = IcalJournal,
    T = IcalTodo,
> {
    pub properties: Vec<ContentLine>,
    pub events: Vec<E>,
    pub alarms: Vec<A>,
    pub todos: Vec<T>,
    pub journals: Vec<J>,
    pub free_busys: Vec<F>,
    pub vtimezones: BTreeMap<String, IcalTimeZone>,
    pub timezones: HashMap<String, Option<chrono_tz::Tz>>,
}
pub type IcalCalendarBuilder = IcalCalendar<
    false,
    IcalAlarmBuilder,
    IcalEventBuilder,
    IcalFreeBusyBuilder,
    IcalJournalBuilder,
    IcalTodoBuilder,
>;

impl Component for IcalCalendar {
    const NAMES: &[&str] = &["VCALENDAR"];
    type Unverified = IcalCalendarBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalCalendarBuilder {
            properties: self.properties,
            events: self.events.into_iter().map(Component::mutable).collect(),
            alarms: self.alarms.into_iter().map(Component::mutable).collect(),
            todos: self.todos.into_iter().map(Component::mutable).collect(),
            journals: self.journals.into_iter().map(Component::mutable).collect(),
            free_busys: self
                .free_busys
                .into_iter()
                .map(Component::mutable)
                .collect(),
            vtimezones: self.vtimezones,
            timezones: self.timezones,
        }
    }
}

impl Component for IcalCalendarBuilder {
    const NAMES: &[&str] = &["VCALENDAR"];
    type Unverified = IcalCalendarBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        self
    }
}

impl ComponentMut for IcalCalendarBuilder {
    type Verified = IcalCalendar;

    fn get_properties_mut(&mut self) -> &mut Vec<ContentLine> {
        &mut self.properties
    }

    #[inline]
    fn add_sub_component<B: BufRead>(
        &mut self,
        value: &str,
        line_parser: &mut PropertyParser<B>,
    ) -> Result<(), ParserError> {
        match value {
            "VALARM" => {
                let mut alarm = IcalAlarmBuilder::new();
                alarm.parse(line_parser)?;
                self.alarms.push(alarm);
            }
            "VEVENT" => {
                let mut event = IcalEventBuilder::new();
                event.parse(line_parser)?;
                self.events.push(event);
            }
            "VTODO" => {
                let mut todo = IcalTodoBuilder::default();
                todo.parse(line_parser)?;
                self.todos.push(todo);
            }
            "VJOURNAL" => {
                let mut journal = IcalJournalBuilder::new();
                journal.parse(line_parser)?;
                self.journals.push(journal);
            }
            "VFREEBUSY" => {
                let mut free_busy = IcalFreeBusyBuilder::new();
                free_busy.parse(line_parser)?;
                self.free_busys.push(free_busy);
            }
            "VTIMEZONE" => {
                let mut timezone = IcalTimeZone::new();
                timezone.parse(line_parser)?;
                let timezone = timezone.build(None)?;
                self.vtimezones
                    .insert(timezone.get_tzid().to_owned(), timezone);
            }
            _ => return Err(ParserError::InvalidComponent(value.to_owned())),
        };

        Ok(())
    }

    fn build(
        self,
        _timezones: Option<&HashMap<String, Option<chrono_tz::Tz>>>,
    ) -> Result<Self::Verified, ParserError> {
        let _version: IcalVERSIONProperty = self.safe_get_required(None)?;
        let _prodid: IcalPRODIDProperty = self.safe_get_required(None)?;
        let _calscale: Option<IcalCALSCALEProperty> = self.safe_get_optional(None)?;

        let timezones = HashMap::from_iter(
            self.vtimezones
                .iter()
                .map(|(tzid, tz)| (tzid.to_owned(), tz.into())),
        );

        Ok(IcalCalendar {
            properties: self.properties,
            events: self
                .events
                .into_iter()
                .map(|builder| builder.build(Some(&timezones)))
                .collect::<Result<_, _>>()?,
            alarms: self
                .alarms
                .into_iter()
                .map(|builder| builder.build(Some(&timezones)))
                .collect::<Result<_, _>>()?,
            todos: self
                .todos
                .into_iter()
                .map(|builder| builder.build(Some(&timezones)))
                .collect::<Result<_, _>>()?,
            journals: self
                .journals
                .into_iter()
                .map(|builder| builder.build(Some(&timezones)))
                .collect::<Result<_, _>>()?,
            free_busys: self
                .free_busys
                .into_iter()
                .map(|builder| builder.build(Some(&timezones)))
                .collect::<Result<_, _>>()?,
            vtimezones: self.vtimezones,
            timezones,
        })
    }
}

impl IcalCalendar {
    pub fn from_objects(
        prodid: String,
        objects: Vec<IcalCalendarObject>,
        additional_properties: Vec<ContentLine>,
    ) -> Self {
        let mut cal = IcalCalendar {
            events: vec![],
            todos: vec![],
            journals: vec![],
            alarms: vec![],
            free_busys: vec![],
            properties: vec![
                IcalVERSIONProperty(IcalVersion::Version2_0, vec![].into()).into(),
                ContentLine {
                    name: "PRODID".to_owned(),
                    value: Some(prodid),
                    params: Default::default(),
                },
                IcalCALSCALEProperty(Calscale::Gregorian, vec![].into()).into(),
            ],
            vtimezones: BTreeMap::new(),
            timezones: HashMap::new(),
        };
        cal.properties.extend_from_slice(&additional_properties);
        for object in objects {
            object.add_to_calendar(&mut cal);
        }
        cal
    }

    pub fn into_objects(self) -> Result<Vec<IcalCalendarObject>, ParserError> {
        let mut out = vec![];

        let mut events: HashMap<String, Vec<IcalEvent>> = HashMap::new();
        for event in self.events {
            events
                .entry(event.get_uid().to_owned())
                .or_insert(vec![])
                .push(event);
        }
        for events in events.into_values() {
            let tzids: HashSet<_> = events
                .iter()
                .flat_map(|e| e.get_tzids())
                .map(ToOwned::to_owned)
                .collect();
            let inner = CalendarInnerData::from_events(events)?;
            out.push(IcalCalendarObject {
                properties: self.properties.clone(),
                vtimezones: self
                    .vtimezones
                    .iter()
                    .filter(|(tzid, _tz)| tzids.contains(tzid.as_str()))
                    .map(|(tzid, tz)| (tzid.to_owned(), tz.clone()))
                    .collect(),
                timezones: self
                    .timezones
                    .iter()
                    .filter(|(tzid, _tz)| tzids.contains(tzid.as_str()))
                    .map(|(tzid, tz)| (tzid.to_owned(), tz.to_owned()))
                    .collect(),
                inner,
            });
        }

        let mut todos: HashMap<String, Vec<IcalTodo>> = HashMap::new();
        for todo in self.todos {
            todos
                .entry(todo.get_uid().to_owned())
                .or_insert(vec![])
                .push(todo);
        }
        for todos in todos.into_values() {
            let tzids: HashSet<_> = todos
                .iter()
                .flat_map(|e| e.get_tzids())
                .map(ToOwned::to_owned)
                .collect();
            let inner = CalendarInnerData::from_todos(todos)?;
            out.push(IcalCalendarObject {
                properties: self.properties.clone(),
                vtimezones: self
                    .vtimezones
                    .iter()
                    .filter(|(tzid, _tz)| tzids.contains(tzid.as_str()))
                    .map(|(tzid, tz)| (tzid.to_owned(), tz.clone()))
                    .collect(),
                timezones: self
                    .timezones
                    .iter()
                    .filter(|(tzid, _tz)| tzids.contains(tzid.as_str()))
                    .map(|(tzid, tz)| (tzid.to_owned(), tz.to_owned()))
                    .collect(),
                inner,
            });
        }

        let mut journals: HashMap<String, Vec<IcalJournal>> = HashMap::new();
        for journal in self.journals {
            journals
                .entry(journal.get_uid().to_owned())
                .or_insert(vec![])
                .push(journal);
        }
        for journals in journals.into_values() {
            let tzids: HashSet<_> = journals
                .iter()
                .flat_map(|j| j.get_tzids())
                .map(ToOwned::to_owned)
                .collect();
            let inner = CalendarInnerData::from_journals(journals)?;
            out.push(IcalCalendarObject {
                properties: self.properties.clone(),
                vtimezones: self
                    .vtimezones
                    .iter()
                    .filter(|(tzid, _tz)| tzids.contains(tzid.as_str()))
                    .map(|(tzid, tz)| (tzid.to_owned(), tz.clone()))
                    .collect(),
                timezones: self
                    .timezones
                    .iter()
                    .filter(|(tzid, _tz)| tzids.contains(tzid.as_str()))
                    .map(|(tzid, tz)| (tzid.to_owned(), tz.to_owned()))
                    .collect(),
                inner,
            });
        }
        Ok(out)
    }
}
