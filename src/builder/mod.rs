//! Generates iCal- or vCard-output.
//!
//! A fair knowledge of the iCal/vCard-standards is necessary to create usable files,
//! even so the [IcalEventBuilder](struct.IcalCalendarBuilder.html) and
//! [IcalVcardBuilder](struct.IcalVcardBuilder.html) helps to stick to the
//! formalities.
//!
//! * iCal: <https://tools.ietf.org/html/rfc5545>
//! * vCard: <https://tools.ietf.org/html/rfc2426>
//!

pub mod calendar;
pub mod event;
pub mod vcard;
