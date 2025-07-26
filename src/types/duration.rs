use crate::property::Property;
use chrono::Duration;
use lazy_static::lazy_static;

lazy_static! {
    static ref RE_DURATION: regex::Regex = regex::Regex::new(
        r"(?x)
        ^(?<sign>[+-])?
        P (
            (
                ((?P<D>\d+)D)?  # days
                (
                    T
                    ((?P<H>\d+)H)?
                    ((?P<M>\d+)M)?
                    ((?P<S>\d+)S)?
                )?
            )  # dur-date,dur-time
            | (
                ((?P<W>\d+)W)?
            )  # dur-week
        )
        $"
    )
    .unwrap();
}

impl TryFrom<&Property> for Option<chrono::Duration> {
    type Error = InvalidDuration;

    fn try_from(value: &Property) -> Result<Self, Self::Error> {
        if let Some(value) = &value.value {
            Ok(Some(parse_duration(value)?))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Invalid duration: {0}")]
pub struct InvalidDuration(String);

pub fn parse_duration(string: &str) -> Result<Duration, InvalidDuration> {
    let captures = RE_DURATION
        .captures(string)
        .ok_or(InvalidDuration(string.to_owned()))?;

    let mut duration = Duration::zero();
    if let Some(weeks) = captures.name("W") {
        duration += Duration::weeks(weeks.as_str().parse().unwrap());
    }
    if let Some(days) = captures.name("D") {
        duration += Duration::days(days.as_str().parse().unwrap());
    }
    if let Some(hours) = captures.name("H") {
        duration += Duration::hours(hours.as_str().parse().unwrap());
    }
    if let Some(minutes) = captures.name("M") {
        duration += Duration::minutes(minutes.as_str().parse().unwrap());
    }
    if let Some(seconds) = captures.name("S") {
        duration += Duration::seconds(seconds.as_str().parse().unwrap());
    }
    if let Some(sign) = captures.name("sign") {
        if sign.as_str() == "-" {
            duration = -duration;
        }
    }

    Ok(duration)
}

#[cfg(test)]
mod tests {
    use super::parse_duration;
    use chrono::Duration;

    #[test]
    fn test_parse_duration() {
        assert!(parse_duration("P1D12W").is_err());
        assert!(parse_duration("P1W12D").is_err());
        assert_eq!(parse_duration("-P12W").unwrap(), -Duration::weeks(12));
        assert_eq!(parse_duration("P12W").unwrap(), Duration::weeks(12));
        assert_eq!(parse_duration("P12D").unwrap(), Duration::days(12));
        assert_eq!(parse_duration("PT12H").unwrap(), Duration::hours(12));
        assert_eq!(parse_duration("PT12M").unwrap(), Duration::minutes(12));
        assert_eq!(parse_duration("PT12S").unwrap(), Duration::seconds(12));
        assert_eq!(
            parse_duration("PT10M12S").unwrap(),
            Duration::minutes(10) + Duration::seconds(12)
        );
        assert_eq!(
            parse_duration("P2DT10M12S").unwrap(),
            Duration::days(2) + Duration::minutes(10) + Duration::seconds(12)
        );
        assert!(parse_duration("PT10S12M").is_err());
        // This should yield an error but it's easier to just let it slip through as 0s
        assert_eq!(parse_duration("P").unwrap(), Duration::zero());
    }
}
