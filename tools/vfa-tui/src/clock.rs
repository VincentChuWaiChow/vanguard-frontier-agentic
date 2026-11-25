//! Wall-clock timestamps for audit records, gate results and reports.
//!
//! Backed by the `time` crate (already in the dependency tree through ratatui and
//! tracing-subscriber) instead of `chrono`, which is flagged unmaintained by
//! RUSTSEC-2026-0543. Formatting is done by hand from the date/time components, so no
//! optional `time` features are needed and the output is identical on every platform.
//!
//! Only UTC is produced. The formatting functions take an explicit instant so tests are
//! deterministic; the `now_*` wrappers are the only places that read the clock.

use time::OffsetDateTime;

/// RFC 3339 UTC timestamp with millisecond precision, e.g. `2025-01-01T00:00:00.123Z`.
pub fn format_millis(at: OffsetDateTime) -> String {
    let at = at.to_offset(time::UtcOffset::UTC);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        at.year(),
        u8::from(at.month()),
        at.day(),
        at.hour(),
        at.minute(),
        at.second(),
        at.millisecond()
    )
}

/// Calendar date in UTC, e.g. `2025-01-01`.
pub fn format_date(at: OffsetDateTime) -> String {
    let at = at.to_offset(time::UtcOffset::UTC);
    format!(
        "{:04}-{:02}-{:02}",
        at.year(),
        u8::from(at.month()),
        at.day()
    )
}

/// Current instant as an RFC 3339 UTC timestamp with millisecond precision.
pub fn now_millis() -> String {
    format_millis(OffsetDateTime::now_utc())
}

/// Current UTC calendar date.
pub fn today() -> String {
    format_date(OffsetDateTime::now_utc())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(nanos: i128) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp_nanos(nanos).expect("valid instant")
    }

    #[test]
    fn millis_format_matches_the_audit_contract() {
        // 2025-01-01T00:00:00.123456789Z: sub-millisecond digits are truncated, not rounded.
        assert_eq!(
            format_millis(at(1_735_689_600_123_456_789)),
            "2025-01-01T00:00:00.123Z"
        );
        assert_eq!(
            format_millis(at(1_735_689_600_000_000_000)),
            "2025-01-01T00:00:00.000Z"
        );
    }

    #[test]
    fn non_utc_offsets_are_normalised_to_utc() {
        let plus_two =
            at(1_735_689_600_000_000_000).to_offset(time::UtcOffset::from_hms(2, 0, 0).unwrap());
        assert_eq!(format_millis(plus_two), "2025-01-01T00:00:00.000Z");
        assert_eq!(format_date(plus_two), "2025-01-01");
    }

    #[test]
    fn date_format_is_zero_padded() {
        // 2024-03-05T23:59:59Z
        assert_eq!(format_date(at(1_709_683_199_000_000_000)), "2024-03-05");
    }

    #[test]
    fn now_values_have_the_expected_shape() {
        let stamp = now_millis();
        assert_eq!(stamp.len(), 24, "{stamp}");
        assert!(
            stamp.ends_with('Z') && stamp.as_bytes()[10] == b'T',
            "{stamp}"
        );
        assert_eq!(today().len(), 10);
    }
}
