mod ical;
mod property;

pub use crate::parser::ical::component::{IcalCalendar, IcalEvent};
pub use crate::parser::vcard::component::VcardContact;
pub use crate::property::ContentLine;

///
/// Emits the content of the Component in ical-format.
///
pub trait Emitter {
    /// creates a textual-representation of this object and all it's properties
    /// in ical-format.
    fn generate(&self) -> String;
}

mod helper {
    /// Creates a param for a [`ContentLine`](property/struct.ContentLine.html).
    ///
    /// # Example
    /// ```
    /// # #[macro_use] extern crate ical;
    /// let param : (String, Vec<String>) = ical_param!("param2", "pvalue1", "pvalue2");
    /// assert_eq!(format!("{:?}", param), "(\"param2\", [\"pvalue1\", \"pvalue2\"])");
    /// ```
    #[macro_export]
    macro_rules! ical_param {
        ($key:literal, $($prop:expr),+) => {
            (String::from($key), vec![$(String::from($prop),)+])
        };
    }

    /// Creates a [`ContentLine`](property/struct.ContentLine.html) for use with
    /// [IcalCalendarBuilder](generator/struct.IcalCalendarBuilder.html),
    /// [IcalEventBuilder](generator/struct.IcalEventBuilder.html),
    /// [IcalVcardBuilder](generator/struct.IcalVcardBuilder.html),
    /// `IcalTodo`, `IcalJournal` ...
    ///
    /// # Example
    /// ```
    /// # #[macro_use] extern crate ical;
    /// # use ical::property::ContentLine;
    /// let prop = ical_property!(
    ///             "NAME",
    ///             "value",
    ///             ical_param!("param2", "pvalue1", "pvalue2"),
    ///             ical_param!("param3", "pvalue3")
    ///         );
    /// let debug_output = "ContentLine { \
    ///     name: \"NAME\", \
    ///     params: [\
    ///         (\"param2\", [\"pvalue1\", \"pvalue2\"]), \
    ///         (\"param3\", [\"pvalue3\"])\
    ///     ], \
    ///     value: Some(\"value\") \
    /// }";
    /// assert_eq!(debug_output, format!("{:?}", prop));
    /// ```
    #[macro_export]
    macro_rules! ical_property {
        ($name:literal, $value:expr) => {
            ContentLine {
                name: String::from($name),
                value: Some($value.into()),
                params: vec![],
            }
        };
        ($name:literal, $value:expr, $($params:expr),+) => {
            ContentLine {
                name: String::from($name),
                value: Some(String::from($value)),
                params: vec![$($params,)+],
            }
        };
    }
}
