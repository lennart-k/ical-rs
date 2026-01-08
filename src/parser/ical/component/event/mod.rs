use std::collections::HashSet;

use crate::{
    component::IcalAlarm,
    parser::{
        Component, ICalProperty, IcalDTENDProperty, IcalDTSTAMPProperty, IcalDTSTARTProperty,
        IcalDURATIONProperty, IcalEXDATEProperty, IcalRDATEProperty, IcalRECURIDProperty,
        IcalSUMMARYProperty, RecurIdRange,
    },
    property::ContentLine,
    types::{CalDate, CalDateOrDateTime, CalDateTime, Timezone},
};
use chrono::{DateTime, Duration, Utc};
use itertools::Itertools;

pub use builder::IcalEventBuilder;
use rrule::{RRule, RRuleSet};
mod builder;

#[derive(Debug, Clone)]
pub struct IcalEvent {
    uid: String,
    dtstamp: IcalDTSTAMPProperty,
    pub dtstart: IcalDTSTARTProperty,
    dtend: Option<IcalDTENDProperty>,
    duration: Option<IcalDURATIONProperty>,
    rdates: Vec<IcalRDATEProperty>,
    rrules: Vec<RRule>,
    exdates: Vec<IcalEXDATEProperty>,
    exrules: Vec<RRule>,
    pub(crate) recurid: Option<IcalRECURIDProperty>,
    summary: Option<IcalSUMMARYProperty>,
    pub(crate) properties: Vec<ContentLine>,
    pub(crate) alarms: Vec<IcalAlarm>,
}

impl IcalEvent {
    pub fn get_uid(&self) -> &str {
        &self.uid
    }
}

impl Component for IcalEvent {
    const NAMES: &[&str] = &["VEVENT"];
    type Unverified = IcalEventBuilder;

    fn get_properties(&self) -> &Vec<ContentLine> {
        &self.properties
    }

    fn mutable(self) -> Self::Unverified {
        IcalEventBuilder {
            properties: self.properties,
            alarms: self.alarms.into_iter().map(Component::mutable).collect(),
        }
    }
}

impl IcalEvent {
    pub fn get_tzids(&self) -> HashSet<&str> {
        self.properties
            .iter()
            .filter_map(|prop| prop.params.get_tzid())
            .unique()
            .collect()
    }

    pub fn to_utc_or_local(self) -> Self {
        // Very naive way to replace known properties with UTC props
        let dtstart = self.dtstart.utc_or_local();
        let dtstamp = self.dtstamp.utc_or_local();
        let exdates = self
            .exdates
            .into_iter()
            .map(|dt| dt.utc_or_local())
            .collect();
        let rdates = self
            .rdates
            .into_iter()
            .map(|dt| dt.utc_or_local())
            .collect();
        let dtend = self.dtend.map(|dt| dt.utc_or_local());

        let mut ev = Self {
            uid: self.uid,
            dtstamp: dtstamp.clone(),
            dtstart: dtstart.clone(),
            dtend: dtend.clone(),
            duration: self.duration,
            rrules: self.rrules,
            rdates,
            exrules: self.exrules,
            exdates,
            summary: self.summary,
            recurid: self.recurid,
            properties: self.properties,
            alarms: self.alarms,
        };
        ev.replace_or_push_property(dtstart);
        ev.replace_or_push_property(dtstamp);
        if let Some(dtend) = dtend {
            ev.replace_or_push_property(dtend);
        }
        ev
    }

    pub fn get_duration(&self) -> Option<Duration> {
        if let Some(IcalDTENDProperty(dtend, _)) = self.dtend.as_ref() {
            return Some(dtend.clone() - &self.dtstart.0);
        };
        self.duration
            .as_ref()
            .map(|IcalDURATIONProperty(duration, _)| duration.to_owned())
    }

    pub fn has_rruleset(&self) -> bool {
        !self.rrules.is_empty()
            || !self.rdates.is_empty()
            || !self.exrules.is_empty()
            || !self.exdates.is_empty()
    }

    pub fn get_rruleset(&self, dtstart: DateTime<rrule::Tz>) -> Option<RRuleSet> {
        if !self.has_rruleset() {
            return None;
        }
        Some(
            RRuleSet::new(dtstart)
                .set_rrules(self.rrules.to_owned())
                .set_rdates(
                    self.rdates
                        .iter()
                        .flat_map(|IcalRDATEProperty(dates, _)| {
                            // TODO: Support periods
                            dates.iter().map(|date| date.start().into())
                        })
                        .collect(),
                )
                .set_exrules(self.exrules.to_owned())
                .set_exdates(
                    self.exdates
                        .iter()
                        .flat_map(|IcalEXDATEProperty(dates, _)| {
                            dates.iter().map(|date| date.to_owned().into())
                        })
                        .collect(),
                ),
        )
    }

    fn replace_or_push_property<T: ICalProperty + Into<ContentLine>>(&mut self, prop: T) {
        let position = self.properties.iter().position(|prop| T::NAME == prop.name);
        if let Some(pos) = position {
            self.properties.retain(|line| line.name != T::NAME);
            self.properties.insert(pos, prop.into());
        } else {
            self.properties.push(prop.into());
        }
    }

    pub fn expand_recurrence(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        overrides: &[Self],
    ) -> Vec<Self> {
        let main = self.clone().to_utc_or_local();
        let mut overrides: Vec<Self> = overrides
            .iter()
            .map(|over| over.clone().to_utc_or_local())
            .collect();
        overrides.sort_by_key(|over| over.recurid.as_ref().unwrap().0.clone());
        let dtstart_utc = main.dtstart.0.utc().with_timezone(&rrule::Tz::UTC);
        let Some(mut rrule_set) = main.get_rruleset(dtstart_utc) else {
            return std::iter::once(main).chain(overrides).collect();
        };

        if let Some(start) = start {
            rrule_set = rrule_set.after(start.with_timezone(&rrule::Tz::UTC));
        }
        if let Some(end) = end {
            rrule_set = rrule_set.before(end.with_timezone(&rrule::Tz::UTC));
        }

        let mut events = vec![];

        let mut template = &main;
        'recurrence: for instance in rrule_set.all(2048).dates {
            let recurid = if main.dtstart.0.is_date() {
                CalDateOrDateTime::Date(CalDate(instance.to_utc().date_naive(), Timezone::utc()))
            } else {
                CalDateOrDateTime::DateTime(CalDateTime::from(instance))
            };

            dbg!(&recurid);

            for over in &overrides {
                let IcalRECURIDProperty(override_recurid, range) = over.recurid.as_ref().unwrap();
                if override_recurid != &recurid {
                    continue;
                }
                // RECURRENCE IDs match
                events.push(over.clone());

                if range == &RecurIdRange::ThisAndFuture {
                    // Set this override as the base event for the future
                    template = over;
                }
                continue 'recurrence;
            }

            // We were not overriden, construct recurrence instance:
            let mut properties = template.properties.clone();
            // Remove recurrence props
            properties.retain(|prop| {
                !["RRULE", "RDATE", "EXRULE", "EXDATE"].contains(&prop.name.as_str())
            });
            properties.retain(|prop| prop.name != "DTEND");
            let mut ev = IcalEvent {
                uid: template.uid.clone(),
                dtstamp: template.dtstamp.clone(),
                summary: template.summary.clone(),
                dtstart: IcalDTSTARTProperty(recurid.clone(), Default::default()),
                recurid: Some(IcalRECURIDProperty(recurid.clone(), RecurIdRange::This)),
                dtend: template.get_duration().map(|duration| {
                    IcalDTENDProperty((recurid.clone() + duration).into(), Default::default())
                }),
                alarms: vec![],
                duration: None, // Set by DTEND
                rdates: vec![],
                rrules: vec![],
                exdates: vec![],
                exrules: vec![],
                properties,
            };
            ev.replace_or_push_property(IcalDTSTARTProperty(recurid.clone(), Default::default()));
            ev.replace_or_push_property(IcalRECURIDProperty(recurid, RecurIdRange::This));
            if let Some(duration) = template.get_duration() {
                ev.replace_or_push_property(IcalDURATIONProperty(duration, Default::default()));
            }

            events.push(ev);
        }

        events
    }
}
