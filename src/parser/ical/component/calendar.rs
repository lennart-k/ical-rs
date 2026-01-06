use crate::{
    PropertyParser,
    component::{
        IcalAlarmBuilder, IcalEventBuilder, IcalFreeBusyBuilder, IcalJournalBuilder,
        IcalTodoBuilder,
    },
    parser::{
        Component, ComponentMut, ParserError,
        ical::component::{
            IcalAlarm, IcalEvent, IcalFreeBusy, IcalJournal, IcalTimeZone, IcalTodo,
        },
    },
    property::ContentLine,
};
use std::{collections::HashMap, io::BufRead};

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
    pub vtimezones: Vec<IcalTimeZone<VERIFIED>>,
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
            vtimezones: self
                .vtimezones
                .into_iter()
                .map(Component::mutable)
                .collect(),
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
                self.vtimezones.push(timezone);
            }
            _ => return Err(ParserError::InvalidComponent(value.to_owned())),
        };

        Ok(())
    }

    fn build(
        self,
        _timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Self::Verified, ParserError> {
        let vtimezones: Vec<IcalTimeZone> = self
            .vtimezones
            .into_iter()
            .map(|builder| builder.build(&HashMap::default()))
            .collect::<Result<_, _>>()?;

        let timezones = HashMap::from_iter(
            vtimezones
                .iter()
                .map(|tz| (tz.get_tzid().to_owned(), tz.into())),
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
            vtimezones,
        })
    }
}
