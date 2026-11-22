// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Locale resolution for `AppInfo` (REQ-I18N-001): the Windows
//! display language when it maps to a shipped locale (12 s12.14),
//! else English. The mapping is pure and unit-tested; the Win32 read
//! (CMP-12 may use Win32, docs/02 s2.14) feeds it.

/// The shipped locales (12 s12.14 LOC-01..06).
pub const SHIPPED_LOCALES: [&str; 6] = ["en", "ar", "fr", "es", "de", "pt-BR"];

/// Map a Win32 `LANGID` to the shipped locale it resolves to.
///
/// Primary language ids per Microsoft's language identifiers table;
/// only Brazilian Portuguese (0x0416) is a distinct shipped locale -
/// every other sublanguage collapses to its primary language, and
/// unsupported primaries fall back to English.
pub fn locale_from_langid(langid: u16) -> &'static str {
    let primary = langid & 0x3FF;
    match primary {
        0x01 => "ar",
        0x07 => "de",
        0x09 => "en",
        0x0C => "fr",
        0x0A => "es",
        0x16 if langid == 0x0416 => "pt-BR",
        _ => "en",
    }
}

/// The user's default UI language as a `LANGID` (Win32).
pub fn display_langid() -> u16 {
    #[cfg(windows)]
    {
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetUserDefaultUILanguage() -> u16;
        }
        // SAFETY: GetUserDefaultUILanguage takes no arguments and
        // only returns the configured language id.
        unsafe { GetUserDefaultUILanguage() }
    }
    #[cfg(not(windows))]
    {
        0x0409 // en-US; the reference platform is Windows (ADR-001).
    }
}

/// The locale `app_info` reports.
pub fn display_locale() -> &'static str {
    locale_from_langid(display_langid())
}

#[cfg(test)]
mod tests {
    use super::{SHIPPED_LOCALES, locale_from_langid};

    #[test]
    fn maps_shipped_languages() {
        assert_eq!(locale_from_langid(0x0401), "ar"); // Arabic (Saudi)
        assert_eq!(locale_from_langid(0x0407), "de"); // German
        assert_eq!(locale_from_langid(0x0409), "en"); // English (US)
        assert_eq!(locale_from_langid(0x0809), "en"); // English (UK)
        assert_eq!(locale_from_langid(0x040C), "fr"); // French
        assert_eq!(locale_from_langid(0x0C0A), "es"); // Spanish (modern)
        assert_eq!(locale_from_langid(0x0416), "pt-BR"); // Portuguese (Brazil)
    }

    #[test]
    fn unsupported_or_partial_falls_back_to_english() {
        assert_eq!(locale_from_langid(0x0816), "en"); // Portuguese (Portugal)
        assert_eq!(locale_from_langid(0x041D), "en"); // Swedish
        assert_eq!(locale_from_langid(0), "en");
    }

    #[test]
    fn every_mapped_locale_is_shipped() {
        for langid in [0x0401u16, 0x0407, 0x0409, 0x040C, 0x0C0A, 0x0416, 0x0816] {
            assert!(
                SHIPPED_LOCALES.contains(&locale_from_langid(langid)),
                "langid {langid:#06x} mapped to an unshipped locale"
            );
        }
    }
}
