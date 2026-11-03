//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Exact time: flicks and timecode (docs/03 §3.3.1; REQ-PLY-005).
//!
//! 1 second = 705,600,000 flicks. Every catalog frame rate (docs/12 §12.9)
//! and both audio rates have an exact integer duration in flicks; this module
//! is the only conversion code for display (03 §3.3.1).

use crate::error::AppError;
use crate::rational::Rational;

/// Flicks per second (03 §3.3.1).
pub const FLICKS_PER_SECOND: i64 = 705_600_000;

/// Maximum sequence length: 24 hours in flicks (03 §3.3.1; < 2^53).
pub const MAX_SEQUENCE_LENGTH: i64 = 86_400 * FLICKS_PER_SECOND;

/// Flicks per frame at `rate`; `None` when the rate does not divide exactly
/// (03 §3.3.1: a custom rate is valid only if `705600000 × den` is divisible
/// by `num`).
pub fn flicks_per_frame(rate: Rational) -> Option<i64> {
    let total = (FLICKS_PER_SECOND as i128) * (rate.den as i128);
    if total % (rate.num as i128) != 0 {
        return None;
    }
    i64::try_from(total / rate.num as i128).ok()
}

/// Flicks per audio sample at `sample_rate` (48,000 → 14,700;
/// 44,100 → 16,000; 03 §3.3.1); `None` when not exact.
pub fn flicks_per_sample(sample_rate: u32) -> Option<i64> {
    if sample_rate == 0 || FLICKS_PER_SECOND % i64::from(sample_rate) != 0 {
        return None;
    }
    Some(FLICKS_PER_SECOND / i64::from(sample_rate))
}

/// Start flicks of frame `n` (03 §3.3.1: frame `n` starts at exactly
/// `n × frameFlicks`); `None` on overflow or an invalid rate.
pub fn frame_start(n: i64, rate: Rational) -> Option<i64> {
    n.checked_mul(flicks_per_frame(rate)?)
}

/// Whether a custom frame rate is valid (03 §3.3.1): `705600000 × den`
/// divisible by `num` and `1 ≤ num/den ≤ 120`.
pub fn is_valid_frame_rate(rate: Rational) -> bool {
    let fps = rate.as_f64();
    (1.0..=120.0).contains(&fps) && flicks_per_frame(rate).is_some()
}

/// Convert a tick count at a foreign (non-exact) rate to flicks with
/// round-half-to-even (03 §3.3.1; error < 1.5 ns).
pub fn flicks_from_ticks(ticks: i64, rate: Rational) -> i64 {
    let num = (ticks as i128) * FLICKS_PER_SECOND as i128 * rate.den as i128;
    let den = rate.num as i128;
    let q = num / den;
    let r = num % den;
    let twice = r.unsigned_abs() * 2u128;
    let den_abs = den.unsigned_abs();
    // Round half to even: |remainder|*2 vs denominator.
    let adjust: i128 = if twice > den_abs {
        1
    } else if twice == den_abs {
        i128::from(q % 2 != 0)
    } else {
        0
    };
    let v = if num < 0 { q - adjust } else { q + adjust };
    v.clamp(i64::MIN as i128, i64::MAX as i128) as i64
}

/// The parsed result of a typed time entry (REQ-PLY-005).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedTime {
    /// An absolute flicks position.
    Absolute(i64),
    /// A relative delta in flicks (`+`/`-` prefix).
    Relative(i64),
}

/// Timecode display parameters: the rate plus the drop-frame flag from the
/// sequence format (03 §3.4.5; 12 §12.9 FPS-04/FPS-07 default to drop-frame).
#[derive(Debug, Clone, Copy)]
pub struct TimecodeRate {
    /// Frame rate as a rational (e.g. 30000/1001).
    pub rate: Rational,
    /// Drop-frame timecode (`;` separator; SMPTE ST 12-1).
    pub drop_frame: bool,
}

impl TimecodeRate {
    /// Nominal frames-per-second label count (30 or 60 for NTSC rates, else
    /// the integer rate).
    fn nominal_fps(self) -> i64 {
        let f = self.rate.as_f64();
        if (f - 29.97).abs() < 0.01 {
            30
        } else if (f - 59.94).abs() < 0.01 {
            60
        } else {
            f.round() as i64
        }
    }

    /// Frames dropped per non-tenth minute for drop-frame rates (SMPTE ST
    /// 12-1: 2 at 30000/1001, 4 at 60000/1001); 0 otherwise.
    fn drop_per_minute(self) -> i64 {
        if !self.drop_frame {
            return 0;
        }
        match self.nominal_fps() {
            30 => 2,
            60 => 4,
            _ => 0,
        }
    }
}

/// Real frames per 10-minute block (18000-18 at 29.97 DF; 36000-36 at 59.94).
fn block_real(tr: TimecodeRate) -> i64 {
    tr.nominal_fps() * 600 - 9 * tr.drop_per_minute()
}

/// Drop count applied to real frame `n` to reach the nominal label (forward
/// SMPTE ST 12-1 adjustment).
fn drops_forward(tr: TimecodeRate, n: i64) -> i64 {
    let d = tr.drop_per_minute();
    if d == 0 {
        return 0;
    }
    let nominal_minute = tr.nominal_fps() * 60;
    let real_minute = nominal_minute - d;
    let blocks = n / block_real(tr);
    let m = n % block_real(tr);
    let within = if m < nominal_minute {
        0
    } else {
        1 + ((m - nominal_minute) / real_minute).min(8)
    };
    9 * d * blocks + d * within
}

/// Drop count subtracted from nominal label `f` to reach the real frame
/// (inverse SMPTE ST 12-1 adjustment).
fn drops_inverse(tr: TimecodeRate, f: i64) -> i64 {
    let d = tr.drop_per_minute();
    if d == 0 {
        return 0;
    }
    let nominal_minute = tr.nominal_fps() * 60;
    let blocks = f / (tr.nominal_fps() * 600);
    let r = f % (tr.nominal_fps() * 600);
    let within = if r < nominal_minute {
        0
    } else {
        1 + ((r - nominal_minute) / nominal_minute).min(8)
    };
    9 * d * blocks + d * within
}

/// Format `flicks` as `HH:MM:SS:FF` (or `HH;MM;SS;FF` drop-frame) at `tr`
/// (REQ-PLY-005); wraps at 24 hours.
pub fn format_timecode(flicks: i64, tr: TimecodeRate) -> String {
    let fpf = flicks_per_frame(tr.rate).unwrap_or(0);
    if fpf == 0 {
        return String::new();
    }
    let n = (flicks / fpf).max(0);
    let nominal = (n + drops_forward(tr, n)) % (tr.nominal_fps() * 86_400);
    let fps = tr.nominal_fps();
    let sep = if tr.drop_frame { ';' } else { ':' };
    // SMPTE drop-frame changes only the seconds:frames separator to ';'
    // (REQ-PLY-005); HH:MM:SS separators stay colons.
    format!(
        "{:02}:{:02}:{:02}{}{:02}",
        nominal / (fps * 3600),
        (nominal / (fps * 60)) % 60,
        (nominal / fps) % 60,
        sep,
        nominal % fps
    )
}

/// Parse a typed time (REQ-PLY-005): `HH:MM:SS:FF` / `HH;MM;SS;FF`
/// (absolute), a plain integer (frame number), a decimal number (seconds),
/// each optionally `+`/`-` prefixed (relative). A skipped drop-frame label
/// canonicalizes forward to the next real frame (ST 12-1).
pub fn parse_timecode(input: &str, tr: TimecodeRate) -> Result<ParsedTime, AppError> {
    let text = input.trim();
    if text.is_empty() {
        return Err(AppError::EditBlocked {
            detail: "empty time".into(),
        });
    }
    let (relative, body) = match text.as_bytes()[0] {
        b'+' => (true, &text[1..]),
        b'-' => (true, &text[1..]),
        _ => (false, text),
    };
    let negative = text.starts_with('-');
    let value = if body.contains(':') || body.contains(';') {
        parse_clock(body, tr)?
    } else if body.contains('.') {
        parse_seconds(body)?
    } else {
        parse_frames(body, tr)?
    };
    let signed = if negative { -value } else { value };
    Ok(if relative {
        ParsedTime::Relative(signed)
    } else {
        ParsedTime::Absolute(signed)
    })
}

fn parse_frames(body: &str, tr: TimecodeRate) -> Result<i64, AppError> {
    let n: i64 = body.parse().map_err(|_| AppError::EditBlocked {
        detail: format!("bad frame count: {body:?}"),
    })?;
    let fpf = flicks_per_frame(tr.rate).ok_or_else(|| AppError::EditBlocked {
        detail: "invalid rate".into(),
    })?;
    Ok(n * fpf)
}

fn parse_seconds(body: &str) -> Result<i64, AppError> {
    let secs: f64 = body.parse().map_err(|_| AppError::EditBlocked {
        detail: format!("bad seconds value: {body:?}"),
    })?;
    if !secs.is_finite() || secs < 0.0 {
        return Err(AppError::EditBlocked {
            detail: "seconds out of range".into(),
        });
    }
    Ok((secs * FLICKS_PER_SECOND as f64).round() as i64)
}

fn parse_clock(body: &str, tr: TimecodeRate) -> Result<i64, AppError> {
    let drop_sep = body.contains(';');
    if drop_sep != tr.drop_frame {
        return Err(AppError::EditBlocked {
            detail: format!("separator does not match the timecode mode: {body:?}"),
        });
    }
    let parts: Vec<&str> = body.split([':', ';']).collect();
    if parts.len() != 4 {
        return Err(AppError::EditBlocked {
            detail: format!("expected HH:MM:SS:FF: {body:?}"),
        });
    }
    let mut nums = [0i64; 4];
    for (slot, part) in nums.iter_mut().zip(parts) {
        *slot = part.parse().map_err(|_| AppError::EditBlocked {
            detail: format!("bad timecode field: {part:?}"),
        })?;
    }
    let [h, m, s, f] = nums;
    let fps = tr.nominal_fps();
    if m > 59 || s > 59 || f >= fps {
        return Err(AppError::EditBlocked {
            detail: format!("timecode out of range: {body:?}"),
        });
    }
    // SMPTE ST 12-1 skips the first `d` frame labels of every non-tenth
    // minute. A typed skipped label canonicalizes FORWARD to the next real
    // frame (00:01:00;00 at 29.97 DF = frame 1800, the label 00:01:00;02).
    let d = if tr.drop_frame {
        tr.drop_per_minute()
    } else {
        0
    };
    let skipped = d > 0 && m >= 1 && m % 10 != 0 && s == 0 && f < d;
    let bump = if skipped { d - f } else { 0 };
    let nominal = h * fps * 3600 + m * fps * 60 + s * fps + f + bump;
    let real = nominal - drops_inverse(tr, nominal);
    let fpf = flicks_per_frame(tr.rate).ok_or_else(|| AppError::EditBlocked {
        detail: "invalid rate".into(),
    })?;
    Ok(real * fpf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rational::Rational;

    fn r(num: u32, den: u32) -> Rational {
        Rational::new(num, den).unwrap_or(Rational { num: 1, den: 1 })
    }

    /// The literal table of 03 §3.3.1, asserted exactly.
    #[test]
    fn literal_flicks_table_matches_doc() {
        let cases = [
            ((24_000, 1001), 29_429_400i64),
            ((24, 1), 29_400_000),
            ((25, 1), 28_224_000),
            ((30_000, 1001), 23_543_520),
            ((30, 1), 23_520_000),
            ((50, 1), 14_112_000),
            ((60_000, 1001), 11_771_760),
            ((60, 1), 11_760_000),
        ];
        for ((num, den), expect) in cases {
            assert_eq!(flicks_per_frame(r(num, den)), Some(expect), "{num}/{den}");
        }
        assert_eq!(flicks_per_sample(48_000), Some(14_700));
        assert_eq!(flicks_per_sample(44_100), Some(16_000));
        assert_eq!(MAX_SEQUENCE_LENGTH, 60_963_840_000_000);
    }

    #[test]
    fn frame_starts_are_exact_multiples() {
        let rate = r(30, 1);
        assert_eq!(frame_start(0, rate), Some(0));
        assert_eq!(frame_start(7, rate), Some(7 * 23_520_000));
        assert_eq!(frame_start(-3, rate), Some(-3 * 23_520_000));
    }

    #[test]
    fn custom_rate_validity() {
        assert!(is_valid_frame_rate(r(30, 1)));
        assert!(is_valid_frame_rate(r(60_000, 1001)));
        assert!(is_valid_frame_rate(r(120, 1)));
        assert!(!is_valid_frame_rate(r(121, 1))); // above 120
        assert!(!is_valid_frame_rate(r(1, 2))); // below 1
        assert!(!is_valid_frame_rate(r(7_056_003, 1000))); // not exact
    }

    #[test]
    fn foreign_conversion_rounds_half_to_even() {
        // 3 ticks at 7 ticks/second -> 3/7 s in flicks.
        let f = flicks_from_ticks(3, r(7, 1));
        let expect_num = 3.0 * FLICKS_PER_SECOND as f64 / 7.0;
        assert!((f as f64 - expect_num).abs() <= 1.0);
        // Half-to-even: 1 tick at 2 ticks/sec = 0.5s exactly -> even.
        assert_eq!(flicks_from_ticks(1, r(2, 1)), FLICKS_PER_SECOND / 2);
        // 3 ticks at 6 ticks/sec = 0.5s -> rounds to the even neighbor.
        assert_eq!(
            flicks_from_ticks(3, r(6, 1)),
            (3 * FLICKS_PER_SECOND / 6) + 1 - (3 % 2)
        );
    }

    fn tr(num: u32, den: u32, drop: bool) -> TimecodeRate {
        TimecodeRate {
            rate: r(num, den),
            drop_frame: drop,
        }
    }

    #[test]
    fn formats_non_drop_rates() {
        let t = tr(30, 1, false);
        let fpf = 23_520_000;
        assert_eq!(format_timecode(0, t), "00:00:00:00");
        assert_eq!(format_timecode(fpf, t), "00:00:00:01");
        assert_eq!(format_timecode(30 * fpf, t), "00:00:01:00");
        let p = tr(25, 1, false);
        assert_eq!(format_timecode(3600 * FLICKS_PER_SECOND, p), "01:00:00:00");
    }

    #[test]
    fn drop_frame_boundaries_at_29_97() {
        let t = tr(30_000, 1001, true);
        let fpf = 23_543_520;
        // Minute 0 end: real 1798 -> 00:00:59;28 (labels ;29 exists too).
        assert_eq!(format_timecode(1798 * fpf, t), "00:00:59;28");
        // First frame of minute 1: real 1800 (labels 1800..1801 skipped).
        assert_eq!(format_timecode(1800 * fpf, t), "00:01:00;02");
        assert_eq!(format_timecode(1799 * fpf, t), "00:00:59;29");
        // Minute 2 boundary skips again; minute 10 does not.
        assert_eq!(format_timecode((1800 + 1798) * fpf, t), "00:02:00;02");
        let block = 17_982;
        assert_eq!(format_timecode(block * fpf, t), "00:10:00;00");
    }

    #[test]
    fn drop_frame_boundaries_at_59_94() {
        let t = tr(60_000, 1001, true);
        let fpf = 11_771_760;
        assert_eq!(format_timecode(0, t), "00:00:00;00");
        // Minute 1 first frame: 4 labels skipped.
        assert_eq!(format_timecode(3600 * fpf, t), "00:01:00;04");
        assert_eq!(format_timecode(35_964 * fpf, t), "00:10:00;00");
    }

    #[test]
    fn skipped_drop_frame_labels_canonicalize_forward() {
        // A typed skipped label (ST 12-1) resolves to the first real frame
        // after the skip, and re-formats to the canonical label (F-10).
        let t = tr(30_000, 1001, true);
        let fpf = 23_543_520;
        for label in ["00:01:00;00", "00:01:00;01"] {
            match parse_timecode(label, t) {
                Ok(crate::time::ParsedTime::Absolute(flicks)) => {
                    assert_eq!(flicks, 1800 * fpf, "{label} must be frame 1800");
                    assert_eq!(format_timecode(flicks, t), "00:01:00;02");
                }
                other => panic!("{label} did not parse: {other:?}"),
            }
        }
        // Non-skipped labels are untouched: minute 10 keeps ;00.
        match parse_timecode("00:10:00;00", t) {
            Ok(crate::time::ParsedTime::Absolute(flicks)) => {
                assert_eq!(flicks, 17_982 * fpf);
            }
            other => panic!("minute-10 label did not parse: {other:?}"),
        }
        // 59.94 skips four labels; all four canonicalize to 00:01:00;04.
        let t60 = tr(60_000, 1001, true);
        let fpf60 = 11_771_760;
        for label in ["00:01:00;00", "00:01:00;01", "00:01:00;02", "00:01:00;03"] {
            match parse_timecode(label, t60) {
                Ok(crate::time::ParsedTime::Absolute(flicks)) => {
                    assert_eq!(flicks, 3600 * fpf60, "{label} must be frame 3600");
                }
                other => panic!("{label} did not parse: {other:?}"),
            }
        }
    }

    #[test]
    fn parses_absolute_relative_frames_and_seconds() {
        let t = tr(30, 1, false);
        let fpf = 23_520_000;
        assert_eq!(
            parse_timecode("00:00:01:00", t),
            Ok(ParsedTime::Absolute(30 * fpf))
        );
        assert_eq!(parse_timecode("12", t), Ok(ParsedTime::Absolute(12 * fpf)));
        assert_eq!(
            parse_timecode("1.5", t),
            Ok(ParsedTime::Absolute(
                FLICKS_PER_SECOND + FLICKS_PER_SECOND / 2
            ))
        );
        assert_eq!(
            parse_timecode("+1.5", t),
            Ok(ParsedTime::Relative(3 * FLICKS_PER_SECOND / 2))
        );
        assert_eq!(
            parse_timecode("-00:00:00:01", t),
            Ok(ParsedTime::Relative(-fpf))
        );
        assert!(parse_timecode("", t).is_err());
        assert!(parse_timecode("00:00:01;00", t).is_err()); // wrong mode separator
        assert!(parse_timecode("00:00:01:30", t).is_err()); // ff >= fps
    }

    #[test]
    fn format_parse_round_trip_at_every_catalog_rate() {
        let rates = [
            (tr(24_000, 1001, false), 29_429_400i64),
            (tr(24, 1, false), 29_400_000),
            (tr(25, 1, false), 28_224_000),
            (tr(30_000, 1001, true), 23_543_520),
            (tr(30_000, 1001, false), 23_543_520),
            (tr(30, 1, false), 23_520_000),
            (tr(50, 1, false), 14_112_000),
            (tr(60_000, 1001, true), 11_771_760),
            (tr(60, 1, false), 11_760_000),
        ];
        let boundaries: Vec<i64> = [
            0i64, 1, 2, 1797, 1798, 1799, 1800, 1801, 1802, 3597, 3598, 17_981, 17_982, 17_983,
            43_180, 107_880,
        ]
        .to_vec();
        for (t, fpf) in rates {
            let mut frames: Vec<i64> = boundaries.clone();
            (0..500).for_each(|i| frames.push(i * 37));
            for f in frames {
                let flicks = f * fpf;
                let text = format_timecode(flicks, t);
                let parsed = parse_timecode(&text, t).unwrap_or(ParsedTime::Absolute(-1));
                let ParsedTime::Absolute(back) = parsed else {
                    panic!("expected absolute");
                };
                assert_eq!(back, flicks, "rate {:?} frame {f} text {text}", t.rate);
            }
        }
    }
}
