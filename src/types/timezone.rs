use chrono::{MappedLocalTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use derive_more::{Display, From};

#[derive(Debug, Clone, From, PartialEq, Eq)]
pub enum Timezone {
    Local,
    Olson(Tz),
}

impl Timezone {
    pub fn is_local(&self) -> bool {
        matches!(self, Self::Local)
    }

    pub fn utc() -> Self {
        Self::Olson(chrono_tz::UTC)
    }
}

impl From<Timezone> for rrule::Tz {
    fn from(value: Timezone) -> Self {
        match value {
            Timezone::Local => Self::LOCAL,
            Timezone::Olson(tz) => Self::Tz(tz),
        }
    }
}

impl From<rrule::Tz> for Timezone {
    fn from(value: rrule::Tz) -> Self {
        match value {
            rrule::Tz::Local(_) => Self::Local,
            rrule::Tz::Tz(tz) => Self::Olson(tz),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum CalTimezoneOffset {
    Local,
    Olson(chrono_tz::TzOffset),
}

impl chrono::Offset for CalTimezoneOffset {
    fn fix(&self) -> chrono::FixedOffset {
        match self {
            Self::Local => Utc.fix(),
            Self::Olson(olson) => olson.fix(),
        }
    }
}

impl TimeZone for Timezone {
    type Offset = CalTimezoneOffset;

    fn from_offset(offset: &Self::Offset) -> Self {
        match offset {
            CalTimezoneOffset::Local => Self::Local,
            CalTimezoneOffset::Olson(offset) => Self::Olson(Tz::from_offset(offset)),
        }
    }

    fn offset_from_local_date(&self, local: &NaiveDate) -> chrono::MappedLocalTime<Self::Offset> {
        match self {
            Self::Local => MappedLocalTime::Single(CalTimezoneOffset::Local),
            Self::Olson(tz) => tz
                .offset_from_local_date(local)
                .map(CalTimezoneOffset::Olson),
        }
    }

    fn offset_from_local_datetime(
        &self,
        local: &NaiveDateTime,
    ) -> chrono::MappedLocalTime<Self::Offset> {
        match self {
            Self::Local => MappedLocalTime::Single(CalTimezoneOffset::Local),
            Self::Olson(tz) => tz
                .offset_from_local_datetime(local)
                .map(CalTimezoneOffset::Olson),
        }
    }

    fn offset_from_utc_datetime(&self, utc: &NaiveDateTime) -> Self::Offset {
        match self {
            Self::Local => CalTimezoneOffset::Local,
            Self::Olson(tz) => CalTimezoneOffset::Olson(tz.offset_from_utc_datetime(utc)),
        }
    }

    fn offset_from_utc_date(&self, utc: &NaiveDate) -> Self::Offset {
        match self {
            Self::Local => CalTimezoneOffset::Local,
            Self::Olson(tz) => CalTimezoneOffset::Olson(tz.offset_from_utc_date(utc)),
        }
    }
}
