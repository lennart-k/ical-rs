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

    pub fn add_event(mut self, ev: IcalEvent) -> Self {
        self.0.cal.events.push(ev);
        self
    }

    pub fn add_alarm(mut self, ev: IcalAlarm) -> Self {
        self.0.cal.alarms.push(ev);
        self
    }

    pub fn add_todo(mut self, ev: IcalTodo) -> Self {
        self.0.cal.todos.push(ev);
        self
    }

    pub fn add_journal(mut self, ev: IcalJournal) -> Self {
        self.0.cal.journals.push(ev);
        self
    }

    pub fn add_tz(mut self, tz: IcalTimeZone) -> Self {
        self.0.cal.timezones.push(tz);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{
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
    }
}
