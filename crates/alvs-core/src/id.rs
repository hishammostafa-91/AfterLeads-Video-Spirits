//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! UUID v7 identifiers (docs/03 §3.2.2): lowercase, hyphenated, unique per
//! project, never reused.

/// Mint a fresh UUID v7 as a lowercase hyphenated string (03 §3.2.2).
pub fn new_id() -> String {
    uuid::Uuid::now_v7().to_string()
}

/// Whether `s` is a valid project identifier: a UUID v7 in lowercase
/// hyphenated form (03 §3.2.2).
pub fn is_valid_id(s: &str) -> bool {
    let Ok(parsed) = uuid::Uuid::parse_str(s) else {
        return false;
    };
    parsed.get_version_num() == 7 && parsed.hyphenated().to_string() == s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ids_are_valid_v7_lowercase_hyphenated() {
        for _ in 0..100 {
            let id = new_id();
            assert!(is_valid_id(&id), "invalid id minted: {id}");
            assert!(
                id.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            );
        }
    }

    #[test]
    fn ids_are_unique_and_time_ordered() {
        let a = new_id();
        let b = new_id();
        assert_ne!(a, b);
        // UUID v7 leads with a big-endian millisecond timestamp (03 §3.2.2):
        // consecutive mints must not go backwards.
        let ms = |id: &str| u64::from_str_radix(&id[..12], 16).unwrap_or(0);
        assert!(ms(&b) >= ms(&a));
    }

    #[test]
    fn rejects_non_v7_and_malformed() {
        assert!(!is_valid_id("not-a-uuid"));
        assert!(!is_valid_id(""));
        let v4 = "9c858901-8a57-4791-81fe-4c455b079579"; // a valid v4 literal
        assert!(!is_valid_id(v4));
        let v7 = uuid::Uuid::now_v7();
        assert!(!is_valid_id(&v7.simple().to_string()));
        assert!(!is_valid_id(&v7.hyphenated().to_string().to_uppercase()));
    }
}
