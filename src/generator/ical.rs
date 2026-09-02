use crate::component::IcalCalendarObject;
use crate::component::{
    IcalAlarm, IcalCalendar, IcalEvent, IcalFreeBusy, IcalJournal, IcalTimeZone,
    IcalTimeZoneTransition, IcalTodo,
};
use crate::generator::Emitter;

impl Emitter for IcalTimeZoneTransition {
    fn generate(&self) -> String {
        let compname = &crate::component::Component::get_comp_name(self);
        format!(
            "BEGIN:{compname}\r\n{inner}END:{compname}\r\n",
            inner = self
                .properties
                .iter()
                .map(Emitter::generate)
                .collect::<String>()
        )
    }
}

macro_rules! generate_emitter {
    ($struct:ty, $($prop:ident),*) => {
        impl Emitter for $struct {
            fn generate(&self) -> String {
                let compname = &crate::component::Component::get_comp_name(self);
                let mut text = format!("BEGIN:{compname}\r\n");
                text += &crate::component::Component::get_properties(self).generate();
                $(text += &self.$prop.generate();)*
                text + "END:" + compname + "\r\n"
            }
        }
    };
}

use crate::component::VcardContact;
generate_emitter!(VcardContact,);

generate_emitter!(IcalAlarm,);
generate_emitter!(IcalFreeBusy,);
generate_emitter!(IcalJournal,);
generate_emitter!(IcalEvent, alarms);
generate_emitter!(IcalTodo, alarms);
generate_emitter!(IcalTimeZone<true>, transitions);
generate_emitter!(
    IcalCalendar,
    vtimezones,
    events,
    alarms,
    todos,
    journals,
    free_busys
);
generate_emitter!(IcalCalendarObject, vtimezones, inner);
