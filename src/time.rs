//! Time and environment layer (spec §1 TIME/DURATION/now rows, §2.8,
//! §12 environment variables).
//!
//! Two clocks meet here: the UTC wall clock (overridable through
//! `AGENT_COEXIST_NOW` for tests, strictly TIME-shaped) and the DURATION
//! grammar behind `--for`/`--since`. All arithmetic is checked — no panic
//! path for any input; results must stay within years 0001..=9999, the
//! range TIME can represent.

use chrono::{
    DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, SubsecRound, TimeDelta, Timelike, Utc,
};
use std::env;

use crate::consts::{DEFAULT_FOR_LOCKED, DEFAULT_FOR_OTHER, POST_WAIT_MS};
use crate::error::Fault;
use crate::names;

/// UTC instant at second precision: TIME is UTC, `Z`, seconds only
/// (§2.8.1), and sub-second precision never survives this module.
pub type Time = DateTime<Utc>;

const NOW_ENV: &str = "AGENT_COEXIST_NOW";
const LOCK_WAIT_ENV: &str = "AGENT_COEXIST_LOCK_WAIT_MS";

/// The DURATION grammar in words (spec §1 DURATION), as a literal so the
/// CLI's static help can `concat!` it: the top-level help and the `--for`
/// and `--since` messages state the same rule.
#[macro_export]
macro_rules! duration_rule {
    () => {
        "Nd, Nh, Nm, Ns in that order, each at most once, total above zero"
    };
}

/// [`duration_rule!`] as a value for runtime messages.
pub const DURATION_RULE: &str = duration_rule!();

/// Current time (§1 now): the system UTC clock truncated to whole
/// seconds, or `AGENT_COEXIST_NOW` when set. The override must be a
/// strict TIME — shape, separators and calendar values all validated
/// here; invalid Unicode is an error, never silently unset.
pub fn now() -> Result<Time, Fault> {
    invocation_clock().map(|(now, _fixed)| now)
}

/// Capture and validate the invocation clock once. The fixed-clock flag
/// lets queued writes retain an explicit test clock without reading
/// the environment again after opening a file.
pub(crate) fn invocation_clock() -> Result<(Time, bool), Fault> {
    match env::var_os(NOW_ENV) {
        None => Ok((system_now(), false)),
        Some(raw) => {
            let value = match raw.to_str() {
                Some(value) => value,
                None => {
                    return Err(Fault::Usage(
                        "AGENT_COEXIST_NOW is not valid Unicode; set it to a UTC time such as 2026-09-17T09:12:00Z, or unset it to use the system clock".into(),
                    ))
                }
            };
            match parse_strict_time(value) {
                Some(time) => Ok((time, true)),
                None => {
                    // When only edge whitespace is wrong (a trailing CR from
                    // a CRLF env file, `set X=... ` in cmd.exe), removing it
                    // keeps the fixed clock; unsetting is then the fallback.
                    // A space between date and time is the one other near
                    // miss worth naming (`--since` names it too), so the
                    // repair that keeps the fixed clock comes first.
                    let fix = names::edge_space_fix(value, |v| parse_strict_time(v).is_some())
                        .or_else(|| {
                            (value.len() == 20 && value.as_bytes()[10] == b' ').then(|| {
                                "separate the date and time with 'T', not a space".to_string()
                            })
                        });
                    Err(Fault::Usage(format!(
                        "AGENT_COEXIST_NOW must be a UTC time YYYY-MM-DDTHH:MM:SSZ such as 2026-09-17T09:12:00Z (got '{}'); {}",
                        names::echo_raw(value),
                        match fix {
                            Some(fix) => format!("{fix}, or unset it to use the system clock"),
                            None => "unset it to use the system clock".to_string(),
                        }
                    )))
                }
            }
        }
    }
}

/// Live UTC clock at the same whole-second precision as stored times.
/// Post and state declarations resample this only after acquiring their
/// exclusive file lock; reads keep their invocation-start snapshot.
pub(crate) fn system_now() -> Time {
    Utc::now().trunc_subsecs(0)
}

/// Strict TIME (§2.8.3): exactly `YYYY-MM-DDTHH:MM:SSZ` — 20 characters,
/// fixed separator positions, uppercase `T`/`Z`, ASCII digits elsewhere,
/// then calendar-validated fields. Variable-width fields, offsets,
/// fractional seconds and lowercase forms are all rejected.
fn parse_strict_time(s: &str) -> Option<Time> {
    let bytes = s.as_bytes();
    if bytes.len() != 20 {
        return None;
    }
    const SEPARATORS: [(usize, u8); 6] = [
        (4, b'-'),
        (7, b'-'),
        (10, b'T'),
        (13, b':'),
        (16, b':'),
        (19, b'Z'),
    ];
    for &(index, expected) in &SEPARATORS {
        if bytes[index] != expected {
            return None;
        }
    }
    if !bytes
        .iter()
        .enumerate()
        .all(|(index, &byte)| matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || byte.is_ascii_digit())
    {
        return None;
    }
    let year = s.get(0..4)?.parse::<i32>().ok()?;
    let month = s.get(5..7)?.parse::<u32>().ok()?;
    let day = s.get(8..10)?.parse::<u32>().ok()?;
    let hour = s.get(11..13)?.parse::<u32>().ok()?;
    let minute = s.get(14..16)?.parse::<u32>().ok()?;
    let second = s.get(17..19)?.parse::<u32>().ok()?;
    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    let time_of_day = NaiveTime::from_hms_opt(hour, minute, second)?;
    let time = DateTime::from_naive_utc_and_offset(NaiveDateTime::new(date, time_of_day), Utc);
    // TIME's domain is years 0001..=9999 (§1): chrono accepts year 0000,
    // but TIME cannot represent it.
    if in_representable_year(&time) {
        Some(time)
    } else {
        None
    }
}

/// Lenient RFC 3339 (§2.8.2) for file times and absolute `--since`: any
/// offset and fractional seconds accepted, converted to UTC and floored
/// to whole seconds. A space in place of `T`/`t` is rejected up front —
/// chrono would accept it, the spec does not.
pub fn parse(s: &str) -> Result<Time, Fault> {
    let invalid = || Fault::Usage(format!("invalid RFC 3339 time: {}", s));
    let bytes = s.as_bytes();
    if s.len() < 19 || (bytes[10] != b'T' && bytes[10] != b't') {
        return Err(invalid());
    }
    let parsed = DateTime::parse_from_rfc3339(s).map_err(|_| invalid())?;
    let utc = parsed.with_timezone(&Utc).trunc_subsecs(0);
    // An offset can push the UTC instant across the 0001..=9999 domain
    // (e.g. 9999-12-31T23:00:00-02:00 lands in year 10000): TIME cannot
    // represent the result, so the value is rejected.
    if !in_representable_year(&utc) {
        return Err(invalid());
    }
    Ok(utc)
}

/// The latest TIME: an end or start beyond it cannot be written.
const LATEST_TIME: &str = "9999-12-31T23:59:59Z";

/// Why a value is not a usable DURATION.
enum DurationError {
    /// The value breaks the grammar.
    Invalid,
    /// The value follows the grammar, but its total does not fit i64
    /// seconds — far beyond the 0001..=9999 range from any `now`.
    TooLarge,
}

/// DURATION (§1): `Nd`/`Nh`/`Nm`/`Ns` concatenated with units strictly
/// descending d > h > m > s (which also enforces "each unit at most
/// once"), at least one part, ASCII digits only — whitespace anywhere,
/// empty input, signs or stray characters are errors. The total in
/// seconds must be > 0 and fit i64. The error text names `--for`, the
/// only option whose failure reports it ([`since`] has its own message).
pub fn duration(s: &str) -> Result<i64, Fault> {
    parse_duration(s).map_err(|error| match error {
        DurationError::Invalid => Fault::Usage(format!(
            "--for must be a duration such as 1h30m: {DURATION_RULE} {}",
            names::got_raw(s, |v| parse_duration(v).is_ok())
        )),
        DurationError::TooLarge => Fault::Usage(format!(
            "--for '{}' is too long: the declaration would end after {LATEST_TIME}, the latest representable time; use a shorter --for",
            names::echo_raw(s)
        )),
    })
}

/// `--timeout` on the three wait commands: the same DURATION grammar as
/// `--for` (§1), read as a wall-clock budget rather than a declaration
/// length, so its errors name `--timeout`. The deadline is fixed against the
/// monotonic clock at parse time, before any file is opened; a duration the
/// clock cannot represent from now is a usage error rather than a panic or a
/// silently unbounded wait.
pub fn timeout(s: &str) -> Result<std::time::Instant, Fault> {
    let secs = parse_duration(s).map_err(|error| match error {
        DurationError::Invalid => Fault::Usage(format!(
            "--timeout must be a duration such as 30s: {DURATION_RULE} {}",
            names::got_raw(s, |v| parse_duration(v).is_ok())
        )),
        DurationError::TooLarge => too_long_timeout(s),
    })?;
    // parse_duration guarantees 0 < secs <= i64::MAX, so the cast is exact.
    std::time::Instant::now()
        .checked_add(std::time::Duration::from_secs(secs as u64))
        .ok_or_else(|| too_long_timeout(s))
}

fn too_long_timeout(s: &str) -> Fault {
    Fault::Usage(format!(
        "--timeout '{}' is too long to wait for; use a shorter --timeout, or omit it to wait without a limit",
        names::echo_raw(s)
    ))
}

/// The grammar is checked to the end before size is reported, so a value
/// that is both oversized and malformed gets the grammar message.
fn parse_duration(s: &str) -> Result<i64, DurationError> {
    const SECS_PER_UNIT: [u128; 4] = [86_400, 3_600, 60, 1];
    fn unit_rank(unit: u8) -> Option<usize> {
        match unit {
            b'd' => Some(0),
            b'h' => Some(1),
            b'm' => Some(2),
            b's' => Some(3),
            _ => None,
        }
    }
    let bytes = s.as_bytes();
    let mut total: Option<u128> = Some(0);
    let mut prev_rank: Option<usize> = None;
    let mut i = 0;
    while i < bytes.len() {
        let digits_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if digits_start == i {
            return Err(DurationError::Invalid); // leading unit, stray char, whitespace
        }
        // ASCII digits only, so a failed parse means the run exceeds u64.
        let number = std::str::from_utf8(&bytes[digits_start..i])
            .ok()
            .and_then(|digits| digits.parse::<u64>().ok());
        if i >= bytes.len() {
            return Err(DurationError::Invalid); // digits with no unit
        }
        let rank = unit_rank(bytes[i]).ok_or(DurationError::Invalid)?;
        if prev_rank.is_some_and(|prev| rank <= prev) {
            return Err(DurationError::Invalid); // not strictly descending (covers repeats)
        }
        prev_rank = Some(rank);
        total = match (total, number) {
            (Some(sum), Some(n)) => (n as u128)
                .checked_mul(SECS_PER_UNIT[rank])
                .and_then(|scaled| sum.checked_add(scaled)),
            _ => None,
        };
        i += 1;
    }
    match total {
        // Empty input, all-zero (`0s`, `0h0m`).
        Some(0) => Err(DurationError::Invalid),
        Some(sum) if sum <= i64::MAX as u128 => Ok(sum as i64),
        _ => Err(DurationError::TooLarge),
    }
}

/// TIME can only represent years 0001..=9999 (§1 DURATION row).
fn in_representable_year(t: &Time) -> bool {
    matches!(t.year(), 1..=9999)
}

/// `--since`: an absolute RFC 3339 time wins; otherwise the value is a
/// DURATION subtracted from `now`. Checked arithmetic only — a result
/// outside representable years is a usage error, never a panic.
pub fn since(s: &str, now: Time) -> Result<Time, Fault> {
    if let Ok(absolute) = parse(s) {
        return Ok(absolute);
    }
    let out_of_range = || {
        Fault::Usage(format!(
            "--since '{}' reaches back before year 0001; use a shorter duration or an absolute time",
            names::echo_raw(s)
        ))
    };
    match parse_duration(s) {
        Ok(secs) => {
            let delta = TimeDelta::try_seconds(secs).ok_or_else(out_of_range)?;
            return now
                .checked_sub_signed(delta)
                .filter(in_representable_year)
                .ok_or_else(out_of_range);
        }
        Err(DurationError::TooLarge) => return Err(out_of_range()),
        Err(DurationError::Invalid) => {}
    }
    // A value that only edge whitespace makes invalid gets that repair
    // (the trimmed value cannot recurse further: it has no edge left).
    // Otherwise a date and time separated by a space is the one near miss
    // worth naming: RFC 3339 requires `T` there (§2.8.2). The DURATION rule
    // is spelled out because a duration-shaped value (`0s`, `30m1h`)
    // otherwise gets no reason.
    let fix = names::edge_space_fix(s, |v| since(v, now).is_ok()).or_else(|| {
        (s.len() >= 19 && s.as_bytes()[10] == b' ')
            .then(|| "separate the date and time with 'T', not a space".to_string())
    });
    Err(Fault::Usage(format!(
        "--since must be an RFC 3339 time such as 2026-09-17T09:12:00Z, or a duration back from now such as 1h30m: {DURATION_RULE} (got '{}'){}",
        names::echo_raw(s),
        fix.map(|fix| format!("; {fix}")).unwrap_or_default()
    )))
}

/// Lock/expiry horizon: `now + secs` via checked addition; outside
/// representable years (or an unrepresentable delta) it is a usage error,
/// never a panic, for any i64 input. Reached with an explicit `--for` and
/// with the default window of a `state` declaration, so the message names
/// both.
pub fn until(now: Time, secs: i64) -> Result<Time, Fault> {
    let out_of_range = || {
        Fault::Usage(format!(
            "the declaration would end after {LATEST_TIME}, the latest representable time; use a shorter --for (without --for, 'locked' lasts {}h and other statuses {}h)",
            DEFAULT_FOR_LOCKED / 3600,
            DEFAULT_FOR_OTHER / 3600
        ))
    };
    let delta = TimeDelta::try_seconds(secs).ok_or_else(out_of_range)?;
    now.checked_add_signed(delta)
        .filter(in_representable_year)
        .ok_or_else(out_of_range)
}

/// Render TIME (§2.8.1): manual zero padding through the chrono
/// accessors — no strftime padding involved. Chrono stores a leap
/// second as `second() == 59` with `nanosecond() >= 1_000_000_000`;
/// the printed second uses that representation so an accepted `:60`
/// is rewritten as `:60`, not the preceding non-leap second.
pub fn format(t: Time) -> String {
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        t.year(),
        t.month(),
        t.day(),
        t.hour(),
        t.minute(),
        t.second() + t.nanosecond() / 1_000_000_000
    )
}

/// Lock wait override (§10.2.9, §12): `AGENT_COEXIST_LOCK_WAIT_MS` as a
/// non-empty run of ASCII decimal digits that parses as u64 — no sign,
/// whitespace or separators. Absent falls back to `POST_WAIT_MS`; invalid
/// Unicode is an error, never silently unset. A digit run beyond u64 is
/// told apart from a malformed value.
pub fn post_wait_ms() -> Result<u64, Fault> {
    let Some(raw) = env::var_os(LOCK_WAIT_ENV) else {
        return Ok(POST_WAIT_MS);
    };
    let value = raw.to_str();
    if let Some(ms) = value.and_then(parse_lock_wait_ms) {
        return Ok(ms);
    }
    // Invalid Unicode has its own message, parallel to AGENT_COEXIST_NOW:
    // the raw value has no text to echo, and the lossy U+FFFD would show a
    // character the user never typed.
    if value.is_none() {
        return Err(Fault::Usage(format!(
            "AGENT_COEXIST_LOCK_WAIT_MS is not valid Unicode; set it to digits such as {POST_WAIT_MS}, or unset it to use the default {POST_WAIT_MS}"
        )));
    }
    let lossy = raw.to_string_lossy();
    let shown = names::echo_raw(&lossy);
    let all_digits = value.is_some_and(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()));
    Err(Fault::Usage(if all_digits {
        format!(
            "AGENT_COEXIST_LOCK_WAIT_MS '{shown}' is larger than {}, the largest accepted number of milliseconds; use a smaller value such as {POST_WAIT_MS}, or unset it to use the default {POST_WAIT_MS}",
            u64::MAX
        )
    } else {
        let fix = names::edge_space_fix(&lossy, |v| parse_lock_wait_ms(v).is_some());
        format!(
            "AGENT_COEXIST_LOCK_WAIT_MS must be a whole number of milliseconds from 0 to {} such as {POST_WAIT_MS}, digits only (got '{shown}'); {}",
            u64::MAX,
            match fix {
                Some(fix) => format!("{fix}, or unset it to use the default {POST_WAIT_MS}"),
                None => format!("unset it to use the default {POST_WAIT_MS}"),
            }
        )
    }))
}

fn parse_lock_wait_ms(s: &str) -> Option<u64> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_time_accepts_only_canonical_shape() {
        assert_eq!(
            parse_strict_time("2026-09-17T09:12:00Z").map(format),
            Some("2026-09-17T09:12:00Z".to_string())
        );
        for bad in [
            "2026-09-17T09:12:00+00:00", // offset instead of Z
            "2026-09-17T09:12:00.500Z",  // fractional seconds
            "2026-09-17t09:12:00Z",      // lowercase t
            "2026-09-17T09:12:00z",      // lowercase z
            "2026-09-17 09:12:00Z",      // space instead of T
            "2026-9-17T09:12:00Z",       // variable-width month
            "2026-09-17T09:12Z",         // short
            "2026-09-17T09:12:00",       // missing Z
            "2026-13-17T09:12:00Z",      // calendar-invalid month
            "2026-02-30T09:12:00Z",      // calendar-invalid day
            "2026-09-17T24:00:00Z",      // hour out of range
            "2026-09-17T09:12:60Z",      // second out of range
            "0000-01-01T00:00:00Z",      // year 0000 is not a TIME year
        ] {
            assert!(parse_strict_time(bad).is_none(), "should reject {:?}", bad);
        }
    }

    #[test]
    fn format_pads_and_round_trips() {
        let t = parse_strict_time("2026-01-05T09:02:03Z").unwrap();
        assert_eq!(format(t), "2026-01-05T09:02:03Z");
        let t = parse_strict_time("0999-12-31T23:59:59Z").unwrap();
        assert_eq!(format(t), "0999-12-31T23:59:59Z");
    }

    #[test]
    fn rfc3339_converts_offsets_and_floors_fractional() {
        let t = parse("2026-09-17T11:12:00.750+02:00").unwrap();
        assert_eq!(format(t), "2026-09-17T09:12:00Z");
        let t = parse("2026-09-17T09:12:00-05:00").unwrap();
        assert_eq!(format(t), "2026-09-17T14:12:00Z");
        // Offset conversion must stay inside years 0001..=9999: this
        // form converts to 9999-12-31T21:00:00Z (in range) ...
        let t = parse("9999-12-31T23:00:00+02:00").unwrap();
        assert_eq!(format(t), "9999-12-31T21:00:00Z");
        // ... while this one converts to 10000-01-01T01:00:00Z (out).
        assert!(
            parse("9999-12-31T23:00:00-02:00").is_err(),
            "year-10000 crossing"
        );
        assert!(parse("2026-09-17 09:12:00Z").is_err(), "space separator");
        assert!(parse("2026-09-17T09:12").is_err(), "too short");
        assert!(parse("").is_err());
        assert!(parse("not-a-time").is_err());
    }

    #[test]
    fn rfc3339_leap_second_preserves_represented_second() {
        // Legal RFC3339 leap second must round-trip as :60, not :59.
        assert_eq!(
            format(parse("2016-12-31T23:59:60Z").unwrap()),
            "2016-12-31T23:59:60Z"
        );
        // Fractional flooring drops subseconds but keeps the leap second.
        assert_eq!(
            format(parse("2016-12-31T23:59:60.750Z").unwrap()),
            "2016-12-31T23:59:60Z"
        );
        // Offset form of the same instant normalizes to UTC Z.
        assert_eq!(
            format(parse("2017-01-01T00:59:60+01:00").unwrap()),
            "2016-12-31T23:59:60Z"
        );
        // The preceding ordinary second stays a distinct represented second.
        assert_eq!(
            format(parse("2016-12-31T23:59:59Z").unwrap()),
            "2016-12-31T23:59:59Z"
        );
        assert_eq!(
            format(parse("2016-12-31T23:59:59.750Z").unwrap()),
            "2016-12-31T23:59:59Z"
        );
    }

    #[test]
    fn duration_accepts_descending_units_only() {
        assert_eq!(duration("1d").unwrap(), 86_400);
        assert_eq!(duration("2h").unwrap(), 7_200);
        assert_eq!(duration("1h30m").unwrap(), 5_400);
        assert_eq!(duration("1d2h3m4s").unwrap(), 93_784);
        for bad in [
            "",
            "0s",
            "0h0m",
            "1s0s",
            "2m1h",
            "1h1h",
            " 1h",
            "1h ",
            "1.5h",
            "h",
            "1x",
            "1",
            "1d1d",
            "999999999999999999999999s",
        ] {
            assert!(duration(bad).is_err(), "should reject {:?}", bad);
        }
    }

    #[test]
    fn since_and_until_stay_in_representable_years() {
        let base = parse_strict_time("2026-09-17T00:00:00Z").unwrap();
        // An absolute time wins over the duration interpretation.
        assert_eq!(
            format(since("2026-09-16T00:00:00Z", base).unwrap()),
            "2026-09-16T00:00:00Z"
        );
        assert_eq!(format(since("1d", base).unwrap()), "2026-09-16T00:00:00Z");
        assert_eq!(format(since("90m", base).unwrap()), "2026-09-16T22:30:00Z");
        // Roughly 25,000 years of seconds: both directions leave 0001..=9999.
        assert!(since("788400000000s", base).is_err());
        assert!(until(base, 788_400_000_000).is_err());
        assert!(until(base, i64::MAX).is_err());
        assert!(since("not-a-time-or-duration", base).is_err());
    }

    #[test]
    fn lock_wait_ms_parsing() {
        assert_eq!(parse_lock_wait_ms("0"), Some(0));
        assert_eq!(parse_lock_wait_ms("250"), Some(250));
        assert_eq!(parse_lock_wait_ms("18446744073709551615"), Some(u64::MAX));
        assert_eq!(parse_lock_wait_ms(""), None);
        assert_eq!(parse_lock_wait_ms("+5"), None);
        assert_eq!(parse_lock_wait_ms("-1"), None);
        assert_eq!(parse_lock_wait_ms(" 5"), None);
        assert_eq!(parse_lock_wait_ms("1,000"), None);
        assert_eq!(parse_lock_wait_ms("18446744073709551616"), None); // u64 overflow
    }
}
