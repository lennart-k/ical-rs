use crate::{
    PropertyParser,
    component::{
        IcalAlarmBuilder, IcalCalendarObject, IcalEventBuilder, IcalFreeBusyBuilder,
        IcalJournalBuilder, IcalTodoBuilder,
    },
    parser::{
        Component, ComponentMut, GetProperty, IcalCALSCALEProperty, IcalPRODIDProperty,
        IcalVERSIONProperty, ParserError,
        ical::component::{
            IcalAlarm, IcalEvent, IcalFreeBusy, IcalJournal, IcalTimeZone, IcalTodo,
        },
    },
    property::ContentLine,
};
use std::{
    collections::{BTreeMap, HashMap},
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
                let timezone = timezone.build(&HashMap::new())?;
                self.vtimezones
                    .insert(timezone.get_tzid().to_owned(), timezone);
            }
            _ => return Err(ParserError::InvalidComponent(value.to_owned())),
        };

        Ok(())
    }

    fn build(
        self,
        _timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Self::Verified, ParserError> {
        let _version: IcalVERSIONProperty = self.safe_get_required(&HashMap::new())?;
        let _prodid: IcalPRODIDProperty = self.safe_get_required(&HashMap::new())?;
        let _calscale: Option<IcalCALSCALEProperty> = self.safe_get_optional(&HashMap::new())?;
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
                .map(|builder| builder.build(&timezones))
                .collect::<Result<_, _>>()?,
            alarms: self
                .alarms
                .into_iter()
                .map(|builder| builder.build(&timezones))
                .collect::<Result<_, _>>()?,
            todos: self
                .todos
                .into_iter()
                .map(|builder| builder.build(&timezones))
                .collect::<Result<_, _>>()?,
            journals: self
                .journals
                .into_iter()
                .map(|builder| builder.build(&timezones))
                .collect::<Result<_, _>>()?,
            free_busys: self
                .free_busys
                .into_iter()
                .map(|builder| builder.build(&timezones))
                .collect::<Result<_, _>>()?,
            vtimezones: self.vtimezones,
        })
    }
}

impl IcalCalendar {
    pub fn from_objects(
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
                ContentLine {
                    name: "VERSION".to_owned(),
                    value: Some("2.0".to_owned()),
                    params: Default::default(),
                },
                ContentLine {
                    name: "CALSCALE".to_owned(),
                    value: Some("GREGORIAN".to_owned()),
                    params: Default::default(),
                },
            ],
            vtimezones: BTreeMap::new(),
        };
        cal.properties.extend_from_slice(&additional_properties);
        for object in objects {
            object.add_to_calendar(&mut cal);
        }
        cal
    }
}
