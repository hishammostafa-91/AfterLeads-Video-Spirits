// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Path redaction for logs and diagnostics (docs/02 §2.11; REQ-REL-003;
// CMP-01 "logging setup helpers"): user file paths never appear in
//! detail messages — drive-letter absolutes and UNC paths become
//! `<path>`.

/// Replace path-shaped substrings with `<path>`: drive-letter absolutes
/// (`C:\Users\me\v.mp4`, `D:/clips/a.mov`) and UNC paths
/// (`\\server\share\a.mp4`). Everything else passes through unchanged.
///
/// A path token ends at whitespace, a quote, an angle bracket, or the end
/// of the text — so surrounding prose and punctuation survive.
pub fn redact_paths(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0usize;
    while i < chars.len() {
        if starts_path(&chars, i) {
            let end = path_end(&chars, i);
            out.push_str("<path>");
            i = end;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// Whether a path token starts at `i`: `X:\` / `X:/` (drive letter) or
/// `\\` (UNC).
fn starts_path(chars: &[char], i: usize) -> bool {
    let c = chars[i];
    // Drive-letter form: [A-Za-z] followed by ':' then a separator.
    if c.is_ascii_alphabetic()
        && chars.get(i + 1) == Some(&':')
        && matches!(chars.get(i + 2), Some('\\') | Some('/'))
    {
        return true;
    }
    // UNC form: two leading backslashes.
    c == '\\' && chars.get(i + 1) == Some(&'\\')
}

/// The index just past the path token starting at `i`.
fn path_end(chars: &[char], i: usize) -> usize {
    let mut j = i;
    while j < chars.len() {
        let c = chars[j];
        if c.is_whitespace() || c == '"' || c == '\'' || c == '<' || c == '>' {
            break;
        }
        j += 1;
    }
    j
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drive_letter_and_unc_paths_become_placeholders() {
        assert_eq!(
            redact_paths(r"Opened C:\Users\me\vid.mp4 and D:/clips/a.mov"),
            "Opened <path> and <path>"
        );
        assert_eq!(
            redact_paths(r"failed on \\server\share\a.mp4 today"),
            "failed on <path> today"
        );
    }

    #[test]
    fn paths_at_edges_and_repeated() {
        assert_eq!(redact_paths(r"C:\solo.alvs"), "<path>");
        assert_eq!(
            redact_paths(r"from \\nas\p\x then C:\y\z end"),
            "from <path> then <path> end"
        );
    }

    #[test]
    fn text_without_paths_is_untouched() {
        let text = "E1008: the project could not be saved (retry: yes)";
        assert_eq!(redact_paths(text), text);
        assert_eq!(redact_paths(""), "");
        // Lower-case drive letters are still paths.
        assert_eq!(redact_paths(r"at c:\temp\x.alvs now"), "at <path> now");
    }

    #[test]
    fn surrounding_punctuation_survives() {
        // The token ends at whitespace or a quote: `C:\a` is one path,
        // the rest is prose.
        assert_eq!(
            redact_paths("see \"C:\\a b\\c.mp4\" done"),
            "see \"<path> b\\c.mp4\" done"
        );
        assert_eq!(
            redact_paths("at C:\\x.mp4, then done"),
            "at <path> then done",
            "the comma is consumed (paths may contain commas; fragments must not leak)"
        );
    }
}
