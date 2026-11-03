// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Wall-clock UTC helpers (research D5): epoch milliseconds and their civil
//! forms — ISO-8601 (`YYYY-MM-DDTHH:MM:SS.sssZ`) for lock/session/SQL
//! `*_utc` text columns and `yyyyMMdd-HHmmss` for backup names (03 §3.9.2) —
//! via the standard days/civil conversion, with no `time`/`chrono`
//! dependency (not baseline rows, G-11).

use std::time::{SystemTime, UNIX_EPOCH};

const MS_PER_DAY: i64 = 86_400_000;
const MS_PER_HOUR: i64 = 3_600_000;
const MS_PER_MINUTE: i64 = 60_000;
const MS_PER_SECOND: i64 = 1_000;

/// Current UTC time as milliseconds since the Unix epoch.
pub fn epoch_ms_now() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => i64::try_from(d.as_millis()).unwrap_or(i64::MAX),
        Err(e) => -i64::try_from(e.duration().as_millis()).unwrap_or(i64::MAX),
    }
}

/// Civil date from days since 1970-01-01 (standard conversion; handles the
/// full proleptic Gregorian range including negative days).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    (y + i64::from(m <= 2), m as u32, d as u32)
}

/// Days since 1970-01-01 from a civil date (inverse of [`civil_from_days`]).
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = y - i64::from(m <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = i64::from(m) + if m > 2 { -3 } else { 9 }; // [0, 11]
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Days in a month, honoring the Gregorian leap rule.
fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
            u32::from(leap) + 28
        }
        _ => 0,
    }
}

/// (year, month, day, hour, minute, second, millisecond) from epoch ms.
fn components(ms: i64) -> (i64, u32, u32, u32, u32, u32, u32) {
    let days = ms.div_euclid(MS_PER_DAY);
    let rem = ms.rem_euclid(MS_PER_DAY);
    let (y, mo, d) = civil_from_days(days);
    let hour = (rem / MS_PER_HOUR) as u32;
    let minute = ((rem % MS_PER_HOUR) / MS_PER_MINUTE) as u32;
    let second = ((rem % MS_PER_MINUTE) / MS_PER_SECOND) as u32;
    let milli = (rem % MS_PER_SECOND) as u32;
    (y, mo, d, hour, minute, second, milli)
}

/// ISO-8601 UTC: `YYYY-MM-DDTHH:MM:SS.sssZ` (years 0001–9999).
///
/// Returns `None` for epochs outside that representable range.
pub fn iso_utc(ms: i64) -> Option<String> {
    let (y, mo, d, h, mi, s, ml) = components(ms);
    if !(1..=9999).contains(&y) {
        return None;
    }
    Some(format!(
        "{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}.{ml:03}Z"
    ))
}

/// Compact UTC for backup names: `yyyyMMdd-HHmmss` (03 §3.9.2).
///
/// Returns `None` for epochs outside years 0001–9999.
pub fn compact_utc(ms: i64) -> Option<String> {
    let (y, mo, d, h, mi, s, _) = components(ms);
    if !(1..=9999).contains(&y) {
        return None;
    }
    Some(format!("{y:04}{mo:02}{d:02}-{h:02}{mi:02}{s:02}"))
}

/// Parse the strict form produced by [`iso_utc`]. Returns `None` for any
/// deviation (wrong length, non-digits, bad separators, invalid date).
pub fn parse_iso_utc(text: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    if bytes.len() != 24
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'.'
        || bytes[23] != b'Z'
    {
        return None;
    }
    let num = |range: std::ops::Range<usize>| -> Option<i64> {
        let mut v: i64 = 0;
        for b in bytes.get(range)? {
            if !b.is_ascii_digit() {
                return None;
            }
            v = v * 10 + i64::from(b - b'0');
        }
        Some(v)
    };
    let y = num(0..4)?;
    let mo = num(5..7)? as u32;
    let d = num(8..10)? as u32;
    let h = num(11..13)?;
    let mi = num(14..16)?;
    let s = num(17..19)?;
    let ml = num(20..23)?;
    if !(1..=12).contains(&mo) || d == 0 || d > days_in_month(y, mo) || h > 23 || mi > 59 || s > 59
    {
        return None;
    }
    Some(
        days_from_civil(y, mo, d) * MS_PER_DAY
            + h * MS_PER_HOUR
            + mi * MS_PER_MINUTE
            + s * MS_PER_SECOND
            + ml,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_dates_format_exactly() {
        let cases: Vec<(i64, &str)> = vec![
            (0, "1970-01-01T00:00:00.000Z"),
            (1, "1970-01-01T00:00:00.001Z"),
            (951_782_400_000, "2000-02-29T00:00:00.000Z"), // leap (÷400)
            (1_709_164_800_000, "2024-02-29T00:00:00.000Z"),
            (1_709_251_200_000 - 1, "2024-02-29T23:59:59.999Z"),
            (1_709_251_200_000, "2024-03-01T00:00:00.000Z"),
            (1_704_067_200_000 - 1, "2023-12-31T23:59:59.999Z"),
            (1_704_067_200_000, "2024-01-01T00:00:00.000Z"),
            (4_107_542_400_000, "2100-03-01T00:00:00.000Z"),
            (4_107_542_400_000 - 86_400_000, "2100-02-28T00:00:00.000Z"), // 2100: no Feb 29
            (-1, "1969-12-31T23:59:59.999Z"),
            (-86_400_000, "1969-12-31T00:00:00.000Z"),
        ];
        for (ms, want) in cases {
            assert_eq!(iso_utc(ms).as_deref(), Some(want), "ms={ms}");
        }
    }

    #[test]
    fn compact_matches_iso_components() {
        assert_eq!(compact_utc(0).as_deref(), Some("19700101-000000"));
        assert_eq!(
            compact_utc(1_709_251_199_999).as_deref(),
            Some("20240229-235959")
        );
        assert_eq!(compact_utc(-1).as_deref(), Some("19691231-235959"));
    }

    #[test]
    fn round_trip_over_strided_samples() {
        // Deterministic sweep across 1970..9999 (no RNG: fixed stride).
        let start: i64 = 0;
        let end: i64 = 253_402_300_799_999; // 9999-12-31T23:59:59.999Z
        let steps = 20_000;
        let stride = (end - start) / steps;
        let mut ms = start;
        while ms <= end {
            let iso = iso_utc(ms).unwrap_or_else(|| panic!("representable ms={ms}"));
            assert_eq!(parse_iso_utc(&iso), Some(ms), "round-trip {iso}");
            ms += stride;
        }
        // Negative side sweeps a bounded era (pre-epoch is valid input),
        // starting exactly at 0001-01-01T00:00:00.000Z (719,162 days).
        let mut ms: i64 = -62_135_596_800_000;
        while ms < 0 {
            let iso = iso_utc(ms).unwrap_or_else(|| panic!("representable ms={ms}"));
            assert_eq!(parse_iso_utc(&iso), Some(ms), "round-trip {iso}");
            ms += 1_000_000_000;
        }
    }

    #[test]
    fn parser_rejects_malformed_and_invalid_dates() {
        for bad in [
            "",
            "1970-01-01T00:00:00.000", // missing Z
            "1970-01-01 00:00:00.000Z",
            "1970-13-01T00:00:00.000Z",  // month 13
            "2023-02-29T00:00:00.000Z",  // not a leap year
            "2100-02-29T00:00:00.000Z",  // ÷100 not ÷400
            "2024-02-30T00:00:00.000Z",  // day beyond month
            "2024-02-29T24:00:00.000Z",  // hour 24
            "2024-02-29T23:60:00.000Z",  // minute 60
            "2024-02-29T23:59:60.000Z",  // second 60
            "abcd-01-01T00:00:00.000Z",  // non-digits
            "1970-01-01T00:00:00.000ZX", // trailing junk
        ] {
            assert_eq!(parse_iso_utc(bad), None, "should reject {bad:?}");
        }
        assert_eq!(
            parse_iso_utc("2024-02-29T00:00:00.000Z"),
            Some(1_709_164_800_000)
        );
    }

    #[test]
    fn out_of_range_years_return_none() {
        // Year 10000+ or year 0 cannot be represented in 4 digits; the
        // exact boundaries round the representable era (Python-verified:
        // 253,402,300,799,999 ms = 9999-12-31T23:59:59.999Z and
        // -62,135,596,800,000 = 0001-01-01T00:00:00.000Z).
        assert!(iso_utc(253_402_300_800_000).is_none()); // 10000-01-01
        assert!(compact_utc(253_402_300_800_000).is_none());
        assert!(iso_utc(-62_135_596_800_001).is_none()); // before 0001
        assert_eq!(
            iso_utc(253_402_300_799_999).as_deref(),
            Some("9999-12-31T23:59:59.999Z")
        );
        assert_eq!(
            iso_utc(-62_135_596_800_000).as_deref(),
            Some("0001-01-01T00:00:00.000Z")
        );
    }
}
