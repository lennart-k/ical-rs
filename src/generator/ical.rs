use crate::component::IcalCalendarObject;
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
        format!(
            "BEGIN:{key}\r\n{inner}END:{key}\r\n",
            inner = &self
                .properties
                .iter()
                .map(Emitter::generate)
                .collect::<String>()
        )
    }
}

macro_rules! generate_emitter {
    ($struct:ty, $key:literal, $($prop:ident),*) => {
        impl Emitter for $struct {
            fn generate(&self) -> String {
                let mut text = format!("BEGIN:{key}\r\n", key = $key);
                text += &crate::parser::Component::get_properties(self).generate();
                $(text += &self.$prop.generate();)*
                text + "END:" + $key + "\r\n"
            }
        }
    };
}

use crate::parser::vcard::component::VcardContact;
generate_emitter!(VcardContact, "VCARD",);

generate_emitter!(IcalAlarm, "VALARM",);
generate_emitter!(IcalFreeBusy, "VFREEBUSY",);
generate_emitter!(IcalJournal, "VJOURNAL",);
generate_emitter!(IcalEvent, "VEVENT", alarms);
generate_emitter!(IcalTodo, "VTODO", alarms);
generate_emitter!(IcalTimeZone<true>, "VTIMEZONE", transitions);
generate_emitter!(
    IcalCalendar,
    "VCALENDAR",
    vtimezones,
    events,
    alarms,
    todos,
    journals,
    free_busys
);
generate_emitter!(IcalCalendarObject, "VCALENDAR", vtimezones, inner);
