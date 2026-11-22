// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Build script: the Tauri resource/embedding pass plus the two
//! build-stamp env vars `AppInfo` reports (04 s4.3.1). The commit is
//! read from git; both fall back to "unknown" outside a git checkout
//! (e.g. a source export).

use std::process::Command;

fn main() {
    tauri_build::build();
    println!("cargo:rustc-env=ALVS_COMMIT={}", short_commit());
    println!("cargo:rustc-env=ALVS_BUILD_DATE={}", utc_build_date());
    println!("cargo:rerun-if-changed=build.rs");
}

/// The short HEAD hash ("unknown" without git or a repository).
fn short_commit() -> String {
    let ok = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|out| out.status.success());
    match ok {
        Some(out) => String::from_utf8_lossy(&out.stdout).trim().to_owned(),
        None => "unknown".to_owned(),
    }
}

/// UTC `YYYY-MM-DD` from the epoch clock (Howard Hinnant's
/// civil-from-days algorithm; no time crate in the baseline).
fn utc_build_date() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    let days = (secs / 86_400) as i64;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Days since 1970-01-01 -> (y, m, d).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    (year, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::civil_from_days;

    #[test]
    fn civil_dates_match_known_days() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_723), (2024, 1, 1));
        assert_eq!(civil_from_days(20_651), (2026, 7, 19));
    }
}
