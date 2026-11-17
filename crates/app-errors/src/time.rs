//! Tiny UTC time helpers so the workspace does not need a full date/time dependency.

use std::time::{SystemTime, UNIX_EPOCH};

/// Seconds since the Unix epoch (0 if the clock is before 1970).
pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Milliseconds since the Unix epoch.
pub fn now_unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Convert days since 1970-01-01 to a (year, month, day) civil date (Howard Hinnant's algorithm).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// `YYYY-MM-DD HH:MM:SS` in UTC.
pub fn format_utc(unix_secs: u64) -> String {
    let days = (unix_secs / 86_400) as i64;
    let rem = unix_secs % 86_400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}",
        rem / 3_600,
        (rem % 3_600) / 60,
        rem % 60
    )
}

/// `YYYY-MM-DD` in UTC.
pub fn format_date(unix_secs: u64) -> String {
    format_utc(unix_secs)[..10].to_string()
}

/// `HH:MM:SS` in UTC.
pub fn format_clock(unix_secs: u64) -> String {
    format_utc(unix_secs)[11..].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_formats_correctly() {
        assert_eq!(format_utc(0), "1970-01-01 00:00:00");
    }

    #[test]
    fn known_timestamp_formats_correctly() {
        // 2024-02-29 12:34:56 UTC (leap day)
        assert_eq!(format_utc(1_709_210_096), "2024-02-29 12:34:56");
    }

    #[test]
    fn date_and_clock_slices() {
        assert_eq!(format_date(1_709_210_096), "2024-02-29");
        assert_eq!(format_clock(1_709_210_096), "12:34:56");
    }
}
