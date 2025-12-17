use itertools::Itertools;

use crate::parser::ical::component::{
    IcalAlarm, IcalCalendar, IcalEvent, IcalFreeBusy, IcalJournal, IcalTimeZone,
    IcalTimeZoneTransition, IcalTodo,
};
use crate::property::Property;
use crate::{PARAM_DELIMITER, PARAM_VALUE_DELIMITER, VALUE_DELIMITER};

///
/// Emits the content of the Component in ical-format.
///
pub trait Emitter {
    /// creates a textual-representation of this object and all it's properties
    /// in ical-format.
    fn generate(&self) -> String;
}

pub(crate) fn split_line<T: Into<String>>(str: T) -> String {
    let str = str.into();
    let mut chars = str.chars();
    let mut first = true;
    let sub_string = (0..)
        .map(|_| {
            chars
                .by_ref()
                .take(if first {
                    first = false;
                    75
                } else {
                    74
                })
                .collect::<String>()
        })
        .take_while(|s| !s.is_empty())
        .collect::<Vec<_>>();
    sub_string.join("\r\n ")
}

//
// @see: https://tools.ietf.org/html/rfc5545#section-3.3.11
//
// `text = *(TSAFE-CHAR / ":" / DQUOTE / ESCAPED-CHAR)`
//     Folded according to description above
//
// `ESCAPED-CHAR = ("\\" / "\;" / "\," / "\N" / "\n")`
//     \\ encodes \, \N or \n encodes newline
//     \; encodes ;, \, encodes ,
//
// `TSAFE-CHAR = WSP / %x21 / %x23-2B / %x2D-39 / %x3C-5B /
//              %x5D-7E / NON-US-ASCII`
//     Any character except CONTROLs not needed by the current
//     character set, DQUOTE, ";", ":", "\", ","
//
#[allow(clippy::ptr_arg)]
pub(crate) fn protect_param(param: &str) -> String {
    // let len = param.len() - 1;
    // starts and ends the param with quotes?
    let in_quotes = param.len() > 1 && param.starts_with('"') && param.ends_with('"');

    let mut escaped = String::new();
    let mut previous_char = None;
    for (pos, char) in param.chars().enumerate() {
        match char {
            '\n' => {
                escaped.push_str("\\n");
            }
            '"' if !in_quotes || (pos > 0 && pos < param.len() - 1) => {
                escaped.push_str("\\\"");
            }
            ';' | ':' | ',' | '\\' if !in_quotes && previous_char != Some('\\') => {
                escaped.push('\\');
                escaped.push(char)
            }
            _ => {
                escaped.push(char);
            }
        }
        previous_char = Some(char);
    }
    escaped
}

#[allow(unused)]
mod should {
    use crate::generator::protect_param;
    use crate::generator::split_line;

    #[test]
    fn split_long_line() {
        let text = "The ability to return a type that is only specified by the trait it impleme\r\n \
                     nts is especially useful in the context closures and iterators, which we c\r\n \
                     over in Chapter 13. Closures and iterators create types that only the comp\r\n \
                     iler knows or types that are very long to specify.";
        assert_eq!(text, split_line(text.replace("\r\n ", "")));
    }

    #[test]
    fn split_long_line_multibyte() {
        // the following text includes multibyte characters (UTF-8) at strategic places to ensure
        // split_line would panic if not multibyte aware
        let text = "DESCRIPTION:ABCDEFGHIJ\\n\\nKLMNOPQRSTUVWXYZ123456789üABCDEFGHIJKLMNOPQRS\\n\\n\r\n \
                     TUVWXYZ123456ä7890ABCDEFGHIJKLM\\n\\nNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOP\r\n \
                     QRSTUVWXöYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWX\\n\\nYZ1234567890abcdefghiÜjkl\r\n \
                     m\\nnopqrstuvwx";
        assert_eq!(text, split_line(text.replace("\r\n ", "")));
    }

    #[test]
    fn protect_chars_in_param() {
        assert_eq!(
            protect_param("\"value: in quotes;\""),
            "\"value: in quotes;\""
        );
        assert_eq!(
            protect_param("\"value, in quotes\""),
            "\"value, in quotes\""
        );
        assert_eq!(
            protect_param("value, \"with\" something"),
            "value\\, \\\"with\\\" something"
        );
        assert_eq!(
            protect_param("\"Directory; C:\\\\Programme\""),
            "\"Directory; C:\\\\Programme\""
        );
        assert_eq!(protect_param("First\nSecond"), "First\\nSecond");
        assert_eq!(
            protect_param(
                "\"42 Plantation St.\\nBaytown\\, LA 30314\\nUnited States o\r\nf America\""
            ),
            "\"42 Plantation St.\\nBaytown\\, LA 30314\\nUnited States o\r\\nf America\""
        );
        assert_eq!(protect_param("ÄÖÜßø"), "ÄÖÜßø");
        assert_eq!(protect_param("\""), "\\\"");
        assert_eq!(protect_param("ÄÖsÜa,ßø"), "ÄÖsÜa\\,ßø");
    }
}

fn get_params(params: &[(String, Vec<String>)]) -> String {
    params
        .iter()
        .map(|(name, values)| {
            let value: String = values
                .iter()
                .map(|value| protect_param(value))
                .join(&PARAM_VALUE_DELIMITER.to_string());
            format!("{name}={value}")
        })
        .join(&PARAM_DELIMITER.to_string())
}

impl Emitter for Property {
    fn generate(&self) -> String {
        let mut output = String::new();
        output.push_str(&self.name);
        let params = get_params(&self.params);
        if !params.is_empty() {
            output.push(PARAM_DELIMITER);
            output.push_str(&params);
        }
        output.push(VALUE_DELIMITER);
        if let Some(value) = self.value.as_ref() {
            output.push_str(value);
        }
        output.push_str("\r\n");
        split_line(output)
    }
}

impl Emitter for IcalTimeZoneTransition {
    fn generate(&self) -> String {
        use crate::parser::ical::component::IcalTimeZoneTransitionType::{DAYLIGHT, STANDARD};
        let key = match &self.transition {
            STANDARD => "STANDARD",
            DAYLIGHT => "DAYLIGHT",
        };
        String::from("BEGIN:")
            + key
            + "\r\n"
            + &self
                .properties
                .iter()
                .map(Emitter::generate)
                .collect::<String>()
            + "END:"
            + key
            + "\r\n"
    }
}

macro_rules! generate_emitter {
    ($struct:ty, $key:literal, $($prop:ident),+) => {
        impl Emitter for $struct {
            fn generate(&self) -> String {
                let mut text = String::from("BEGIN:") + $key + "\r\n";
                $(text += &self.$prop
                .iter()
                .map(Emitter::generate)
                .collect::<String>();)+

                text + "END:" + $key + "\r\n"
            }
        }
    };
}

#[cfg(feature = "vcard")]
use crate::parser::vcard::component::VcardContact;

#[cfg(feature = "vcard")]
generate_emitter!(VcardContact, "VCARD", properties);

generate_emitter!(IcalAlarm, "VALARM", properties);
generate_emitter!(IcalFreeBusy, "VFREEBUSY", properties);
generate_emitter!(IcalJournal, "VJOURNAL", properties);
generate_emitter!(IcalEvent, "VEVENT", properties, alarms);
generate_emitter!(IcalTodo, "VTODO", properties, alarms);
generate_emitter!(IcalTimeZone<true>, "VTIMEZONE", properties, transitions);
generate_emitter!(
    IcalCalendar,
    "VCALENDAR",
    properties,
    timezones,
    events,
    alarms,
    todos,
    journals,
    free_busys
);
