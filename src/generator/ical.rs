use crate::generator::Emitter;
use crate::parser::ical::component::{
    IcalAlarm, IcalCalendar, IcalEvent, IcalFreeBusy, IcalJournal, IcalTimeZone,
    IcalTimeZoneTransition, IcalTodo,
};

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

use crate::parser::vcard::component::VcardContact;
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
