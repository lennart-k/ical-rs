use crate::parser::ical::component::{IcalCalendar, IcalEvent, IcalTimeZone};
use crate::property::Property;
use crate::{
    ical_property,
    parser::{
        ComponentMut, ParserError,
        ical::component::{IcalAlarm, IcalJournal, IcalTodo},
    },
};

pub struct IcalCalendarBuilder {
    cal: IcalCalendar<false>,
}
pub struct CalScale(IcalCalendarBuilder);
pub struct ProdId(IcalCalendarBuilder);
pub struct Finalizer(IcalCalendarBuilder);

/// Builds a new [RFC 5545 - iCalendar Object](https://tools.ietf.org/html/rfc5545#section-3.4)
///
/// ```
/// # use ical::builder::calendar::*;
/// # use ical::generator::Property;
/// # use ical::ical_property;
/// #
/// let calendar = IcalCalendarBuilder::version("4.0")
///     .gregorian()
///     .prodid("my-calender-generator 1.0")
///     .set(ical_property!("METHOD", "PUBLISH"))
///     .build();
/// ```
impl IcalCalendarBuilder {
    pub fn version<S: Into<String>>(version: S) -> CalScale {
        let mut e = CalScale(Self {
            cal: IcalCalendar::new(),
        });
        e.0.cal.properties.push(ical_property!("VERSION", version));
        e
    }
}

impl CalScale {
    /// sets the calendar scale to GREGORIAN (the default)
    pub fn gregorian(mut self) -> ProdId {
        self.0
            .cal
            .properties
            .push(ical_property!("CALSCALE", "GREGORIAN"));
        ProdId(self.0)
    }

    /// sets no calendar scale.
    pub fn noscale(self) -> ProdId {
        ProdId(self.0)
    }
}

impl ProdId {
    /// Sets the Product Identifier of the calendar.
    /// [PRODID](https://www.rfc-editor.org/rfc/rfc5545#section-3.7.3)
    pub fn prodid<S: Into<String>>(mut self, prodid: S) -> Finalizer {
        self.0.cal.properties.push(ical_property!("PRODID", prodid));

        Finalizer(self.0)
    }
}

impl Finalizer {
    /// creates a complete IcalCalendar-object.
    pub fn build(self) -> Result<IcalCalendar, ParserError> {
        self.0.cal.verify()
    }

    pub fn set(mut self, property: Property) -> Self {
        self.0.cal.properties.push(property);
        self
    }

    pub fn add_event(self, ev: IcalEvent) -> Self {
        self.add_events(&[ev])
    }

    pub fn add_events(mut self, evs: &[IcalEvent]) -> Self {
        self.0.cal.events.extend_from_slice(evs);
        self
    }

    pub fn add_alarm(self, alarm: IcalAlarm) -> Self {
        self.add_alarms(&[alarm])
    }

    pub fn add_alarms(mut self, alarms: &[IcalAlarm]) -> Self {
        self.0.cal.alarms.extend_from_slice(alarms);
        self
    }

    pub fn add_todo(self, todo: IcalTodo) -> Self {
        self.add_todos(&[todo])
    }

    pub fn add_todos(mut self, todos: &[IcalTodo]) -> Self {
        self.0.cal.todos.extend_from_slice(todos);
        self
    }

    pub fn add_journal(self, journal: IcalJournal) -> Self {
        self.add_journals(&[journal])
    }

    pub fn add_journals(mut self, journals: &[IcalJournal]) -> Self {
        self.0.cal.journals.extend_from_slice(journals);
        self
    }

    pub fn add_timezone(self, tz: IcalTimeZone) -> Self {
        self.add_timezones(&[tz])
    }

    pub fn add_timezones(mut self, tzs: &[IcalTimeZone]) -> Self {
        self.0.cal.timezones.extend_from_slice(tzs);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        IcalParser,
        builder::{calendar::IcalCalendarBuilder, event::IcalEventBuilder},
        generator::Emitter,
        property::Property,
    };

    #[test]
    fn test_calendar_builder() {
        let cal = IcalCalendarBuilder::version("4.0")
            .gregorian()
            .prodid("github.com/lennart-k/ical-rs")
            .set(Property {
                name: "X-HELLO".to_string(),
                params: vec![],
                value: Some("Ok wow!".to_string()),
            })
            .add_event(
                IcalEventBuilder::tzid("Europe/Berlin")
                    .uid("asdasd")
                    .changed_utc("20250726T144426Z")
                    .start("20250726T144426Z")
                    .end("20250726T144426Z")
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();
        insta::assert_snapshot!(cal.generate());

        let ics = include_str!("../../tests/resources/ical_everything.ics");
        let mut ref_cal = IcalParser::new(ics.as_bytes()).next().unwrap().unwrap();

        let cal = IcalCalendarBuilder::version("4.0")
            .noscale()
            .prodid("github.com/lennart-k/ical-rs")
            .add_events(&ref_cal.events)
            .set(Property {
                name: "X-HELLO".to_string(),
                params: vec![],
                value: Some("Ok wow!".to_string()),
            })
            .add_todo(ref_cal.todos.pop().unwrap())
            .add_alarm(ref_cal.alarms.pop().unwrap())
            .add_timezone(ref_cal.timezones.pop().unwrap())
            .add_journal(ref_cal.journals.pop().unwrap())
            .build()
            .unwrap();
        insta::assert_snapshot!(cal.generate());
    }
}
