// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! The settings key table (docs/03 §3.6, verbatim): 68 keys with kind,
//! range/enum, default, and settings page. The table *is* the schema
//! (docs/02 §2.4 CMP-08 "settings (JSON + schema)"); the store validates
//! each key against it and unknown keys ride through untouched (03 §3.6;
//! REQ-SET-003).

use serde_json::Value;

/// Value kind and range of one settings key (03 §3.6 "Type / range").
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    /// `bool`
    Bool,
    /// `int min–max` (optional `step` alignment, e.g. uiZoom).
    Int {
        /// Inclusive lower bound.
        min: i64,
        /// Inclusive upper bound.
        max: i64,
        /// When set, `(value - min) % step == 0` is required.
        step: Option<i64>,
    },
    /// `float min–max`
    Float {
        /// Inclusive lower bound.
        min: f64,
        /// Inclusive upper bound.
        max: f64,
    },
    /// Non-empty string (IDs, device IDs, easing IDs, BCP 47 tags).
    Str,
    /// Non-empty string holding a filesystem path or the literal `default`
    /// where the row allows it (validated at the use site with SEC-04).
    Path,
    /// String member of a fixed set.
    Enum(&'static [&'static str]),
    /// Integer member of a fixed set (e.g. proxy heights).
    EnumInt(&'static [i64]),
    /// JSON object (free shape, e.g. keyboard overrides).
    Object,
    /// JSON array (free shape, e.g. saved workspaces).
    Array,
}

/// A default value literal (03 §3.6 "Default").
#[derive(Debug, Clone, PartialEq)]
pub enum Default {
    /// Boolean default.
    Bool(bool),
    /// Integer default.
    Int(i64),
    /// Float default.
    Float(f64),
    /// String default.
    Str(&'static str),
    /// JSON literal default (`{}`, `[]`).
    Json(&'static str),
}

/// One row of the 03 §3.6 table.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyDef {
    /// Settings key, e.g. `alvs.general.autosaveIntervalSec`.
    pub key: &'static str,
    /// Kind and range.
    pub kind: Kind,
    /// Documented default.
    pub default: Default,
    /// Settings page label from 03 §3.6.
    pub page: &'static str,
}

/// The full 03 §3.6 table, in document order. 68 rows.
pub const KEYS: &[KeyDef] = &[
    // alvs.general (General)
    KeyDef {
        key: "alvs.general.language",
        kind: Kind::Enum(&["system", "en", "ar", "fr", "es", "de", "pt-BR"]),
        default: Default::Str("system"),
        page: "General",
    },
    KeyDef {
        key: "alvs.general.autosaveIntervalSec",
        kind: Kind::Int {
            min: 15,
            max: 600,
            step: None,
        },
        default: Default::Int(60),
        page: "General",
    },
    KeyDef {
        key: "alvs.general.backupsKeep",
        kind: Kind::Int {
            min: 1,
            max: 100,
            step: None,
        },
        default: Default::Int(20),
        page: "General",
    },
    KeyDef {
        key: "alvs.general.restoreLastProject",
        kind: Kind::Bool,
        default: Default::Bool(false),
        page: "General",
    },
    KeyDef {
        key: "alvs.general.showWelcome",
        kind: Kind::Bool,
        default: Default::Bool(true),
        page: "General",
    },
    KeyDef {
        key: "alvs.general.recentLimit",
        kind: Kind::Int {
            min: 5,
            max: 50,
            step: None,
        },
        default: Default::Int(20),
        page: "General",
    },
    KeyDef {
        key: "alvs.general.defaultProjectFolder",
        kind: Kind::Path,
        default: Default::Str("%USERPROFILE%\\Videos\\AfterLeads-Video-Spirits"),
        page: "General",
    },
    KeyDef {
        key: "alvs.general.checkUpdates",
        kind: Kind::Bool,
        default: Default::Bool(true),
        page: "General",
    },
    KeyDef {
        key: "alvs.general.updateChannel",
        kind: Kind::Enum(&["stable", "beta"]),
        default: Default::Str("stable"),
        page: "General",
    },
    // alvs.appearance (Appearance)
    KeyDef {
        key: "alvs.appearance.theme",
        kind: Kind::Enum(&["dark", "light", "highContrast", "system"]),
        default: Default::Str("dark"),
        page: "Appearance",
    },
    KeyDef {
        key: "alvs.appearance.accent",
        kind: Kind::Enum(&["CLR-06", "CLR-07", "CLR-04", "CLR-02", "CLR-08"]),
        default: Default::Str("CLR-06"),
        page: "Appearance",
    },
    KeyDef {
        key: "alvs.appearance.uiZoom",
        kind: Kind::Int {
            min: 80,
            max: 200,
            step: Some(10),
        },
        default: Default::Int(100),
        page: "Appearance",
    },
    KeyDef {
        key: "alvs.appearance.density",
        kind: Kind::Enum(&["comfortable", "compact"]),
        default: Default::Str("comfortable"),
        page: "Appearance",
    },
    KeyDef {
        key: "alvs.appearance.reduceMotion",
        kind: Kind::Enum(&["system", "on", "off"]),
        default: Default::Str("system"),
        page: "Appearance",
    },
    // alvs.playback (Playback)
    KeyDef {
        key: "alvs.playback.previewResolution",
        kind: Kind::Enum(&["auto", "full", "half", "quarter"]),
        default: Default::Str("auto"),
        page: "Playback",
    },
    KeyDef {
        key: "alvs.playback.playEveryFrame",
        kind: Kind::Bool,
        default: Default::Bool(false),
        page: "Playback",
    },
    KeyDef {
        key: "alvs.playback.hardwareDecode",
        kind: Kind::Bool,
        default: Default::Bool(true),
        page: "Playback",
    },
    KeyDef {
        key: "alvs.playback.audioDevice",
        kind: Kind::Str,
        default: Default::Str("default"),
        page: "Playback",
    },
    KeyDef {
        key: "alvs.playback.audioBuffer",
        kind: Kind::Enum(&["low", "normal", "safe"]),
        default: Default::Str("normal"),
        page: "Playback",
    },
    KeyDef {
        key: "alvs.playback.audioScrub",
        kind: Kind::Bool,
        default: Default::Bool(true),
        page: "Playback",
    },
    KeyDef {
        key: "alvs.playback.timeDisplay",
        kind: Kind::Enum(&["timecode", "frames", "seconds"]),
        default: Default::Str("timecode"),
        page: "Playback",
    },
    KeyDef {
        key: "alvs.playback.dropFrameTimecode",
        kind: Kind::Bool,
        default: Default::Bool(true),
        page: "Playback",
    },
    KeyDef {
        key: "alvs.playback.dualViewer",
        kind: Kind::Enum(&["auto", "on", "off"]),
        default: Default::Str("auto"),
        page: "Playback",
    },
    KeyDef {
        key: "alvs.playback.gpuCacheMB",
        kind: Kind::Int {
            min: 256,
            max: 8192,
            step: None,
        },
        default: Default::Int(1024),
        page: "Playback",
    },
    // alvs.timeline (Timeline)
    KeyDef {
        key: "alvs.timeline.snapping",
        kind: Kind::Bool,
        default: Default::Bool(true),
        page: "Timeline",
    },
    KeyDef {
        key: "alvs.timeline.linkedSelection",
        kind: Kind::Bool,
        default: Default::Bool(true),
        page: "Timeline",
    },
    KeyDef {
        key: "alvs.timeline.thumbnails",
        kind: Kind::Enum(&["off", "first", "filmstrip"]),
        default: Default::Str("filmstrip"),
        page: "Timeline",
    },
    KeyDef {
        key: "alvs.timeline.waveforms",
        kind: Kind::Bool,
        default: Default::Bool(true),
        page: "Timeline",
    },
    KeyDef {
        key: "alvs.timeline.followPlayhead",
        kind: Kind::Enum(&["page", "smooth", "off"]),
        default: Default::Str("page"),
        page: "Timeline",
    },
    KeyDef {
        key: "alvs.timeline.insertFitMode",
        kind: Kind::Enum(&["fit", "fill", "none"]),
        default: Default::Str("fit"),
        page: "Timeline",
    },
    KeyDef {
        key: "alvs.timeline.trackHeightVideo",
        kind: Kind::Int {
            min: 32,
            max: 240,
            step: None,
        },
        default: Default::Int(64),
        page: "Timeline",
    },
    KeyDef {
        key: "alvs.timeline.trackHeightAudio",
        kind: Kind::Int {
            min: 32,
            max: 240,
            step: None,
        },
        default: Default::Int(56),
        page: "Timeline",
    },
    KeyDef {
        key: "alvs.timeline.defaultStillDurationSec",
        kind: Kind::Float {
            min: 0.1,
            max: 600.0,
        },
        default: Default::Float(5.0),
        page: "Timeline",
    },
    KeyDef {
        key: "alvs.timeline.defaultTitleDurationSec",
        kind: Kind::Float {
            min: 0.1,
            max: 600.0,
        },
        default: Default::Float(5.0),
        page: "Timeline",
    },
    KeyDef {
        key: "alvs.timeline.defaultGeneratorDurationSec",
        kind: Kind::Float {
            min: 0.1,
            max: 600.0,
        },
        default: Default::Float(5.0),
        page: "Timeline",
    },
    KeyDef {
        key: "alvs.timeline.defaultTransitionSec",
        kind: Kind::Float {
            min: 0.1,
            max: 10.0,
        },
        default: Default::Float(1.0),
        page: "Timeline",
    },
    KeyDef {
        key: "alvs.timeline.defaultEasing",
        kind: Kind::Str,
        default: Default::Str("linear"),
        page: "Timeline",
    },
    // alvs.captions (Timeline)
    KeyDef {
        key: "alvs.captions.maxCharsPerLine",
        kind: Kind::Int {
            min: 20,
            max: 80,
            step: None,
        },
        default: Default::Int(42),
        page: "Timeline",
    },
    KeyDef {
        key: "alvs.captions.maxLines",
        kind: Kind::Int {
            min: 1,
            max: 3,
            step: None,
        },
        default: Default::Int(2),
        page: "Timeline",
    },
    KeyDef {
        key: "alvs.captions.minDurationSec",
        kind: Kind::Float { min: 0.3, max: 5.0 },
        default: Default::Float(0.7),
        page: "Timeline",
    },
    KeyDef {
        key: "alvs.captions.maxDurationSec",
        kind: Kind::Float {
            min: 2.0,
            max: 20.0,
        },
        default: Default::Float(7.0),
        page: "Timeline",
    },
    KeyDef {
        key: "alvs.captions.minGapFrames",
        kind: Kind::Int {
            min: 0,
            max: 10,
            step: None,
        },
        default: Default::Int(2),
        page: "Timeline",
    },
    // alvs.media (Media & Cache)
    KeyDef {
        key: "alvs.media.proxyAuto",
        kind: Kind::Bool,
        default: Default::Bool(true),
        page: "Media & Cache",
    },
    KeyDef {
        key: "alvs.media.proxyThresholdHeight",
        kind: Kind::EnumInt(&[1080, 1440, 2160]),
        default: Default::Int(1440),
        page: "Media & Cache",
    },
    KeyDef {
        key: "alvs.media.proxyResolution",
        kind: Kind::EnumInt(&[540, 720, 1080]),
        default: Default::Int(720),
        page: "Media & Cache",
    },
    KeyDef {
        key: "alvs.media.proxyLocation",
        kind: Kind::Enum(&["cache", "projectFolder"]),
        default: Default::Str("cache"),
        page: "Media & Cache",
    },
    KeyDef {
        key: "alvs.media.deinterlaceDefault",
        kind: Kind::Enum(&["auto", "off", "force"]),
        default: Default::Str("auto"),
        page: "Media & Cache",
    },
    KeyDef {
        key: "alvs.media.cacheFolder",
        kind: Kind::Path,
        default: Default::Str("default"),
        page: "Media & Cache",
    },
    KeyDef {
        key: "alvs.media.cacheMaxGB",
        kind: Kind::Int {
            min: 2,
            max: 1000,
            step: None,
        },
        default: Default::Int(20),
        page: "Media & Cache",
    },
    // alvs.export (Export)
    KeyDef {
        key: "alvs.export.defaultFolder",
        kind: Kind::Path,
        default: Default::Str("projectExports"),
        page: "Export",
    },
    KeyDef {
        key: "alvs.export.useHardwareEncoder",
        kind: Kind::Bool,
        default: Default::Bool(true),
        page: "Export",
    },
    KeyDef {
        key: "alvs.export.lastPresetId",
        kind: Kind::Str,
        default: Default::Str("xp.h264.high"),
        page: "Export",
    },
    KeyDef {
        key: "alvs.export.openFolderWhenDone",
        kind: Kind::Bool,
        default: Default::Bool(false),
        page: "Export",
    },
    KeyDef {
        key: "alvs.export.notifyWhenDone",
        kind: Kind::Bool,
        default: Default::Bool(true),
        page: "Export",
    },
    // alvs.keyboard (Keyboard)
    KeyDef {
        key: "alvs.keyboard.overrides",
        kind: Kind::Object,
        default: Default::Json("{}"),
        page: "Keyboard",
    },
    // alvs.ai (AI)
    KeyDef {
        key: "alvs.ai.sttModel",
        kind: Kind::Str,
        default: Default::Str("small-q5_1"),
        page: "AI",
    },
    KeyDef {
        key: "alvs.ai.sttLanguage",
        kind: Kind::Str,
        default: Default::Str("auto"),
        page: "AI",
    },
    KeyDef {
        key: "alvs.ai.sttUseGpu",
        kind: Kind::Bool,
        default: Default::Bool(true),
        page: "AI",
    },
    KeyDef {
        key: "alvs.ai.modelsFolder",
        kind: Kind::Path,
        default: Default::Str("default"),
        page: "AI",
    },
    // alvs.advanced (Advanced)
    KeyDef {
        key: "alvs.advanced.gpuAdapter",
        kind: Kind::Str,
        default: Default::Str("auto"),
        page: "Advanced",
    },
    KeyDef {
        key: "alvs.advanced.forceWarp",
        kind: Kind::Bool,
        default: Default::Bool(false),
        page: "Advanced",
    },
    KeyDef {
        key: "alvs.advanced.maxParallelJobs",
        kind: Kind::Int {
            min: 1,
            max: 8,
            step: None,
        },
        default: Default::Int(2),
        page: "Advanced",
    },
    KeyDef {
        key: "alvs.advanced.logLevel",
        kind: Kind::Enum(&["info", "debug", "trace"]),
        default: Default::Str("info"),
        page: "Advanced",
    },
    KeyDef {
        key: "alvs.advanced.redactDiagnostics",
        kind: Kind::Bool,
        default: Default::Bool(true),
        page: "Advanced",
    },
    // alvs.workspaces (View menu)
    KeyDef {
        key: "alvs.workspaces.saved",
        kind: Kind::Array,
        default: Default::Json("[]"),
        page: "View menu",
    },
    KeyDef {
        key: "alvs.workspaces.active",
        kind: Kind::Str,
        default: Default::Str("WS-01"),
        page: "View menu",
    },
    // alvs.ui (internal)
    KeyDef {
        key: "alvs.ui.firstRunDone",
        kind: Kind::Bool,
        default: Default::Bool(false),
        page: "internal",
    },
    KeyDef {
        key: "alvs.ui.tourDone",
        kind: Kind::Bool,
        default: Default::Bool(false),
        page: "internal",
    },
];

/// Look up a key definition (03 §3.6).
pub fn find(key: &str) -> Option<&'static KeyDef> {
    KEYS.iter().find(|k| k.key == key)
}

/// The default value of a key as JSON (03 §3.6).
pub fn default_value(def: &KeyDef) -> Value {
    match &def.default {
        Default::Bool(b) => Value::Bool(*b),
        Default::Int(i) => Value::from(*i),
        Default::Float(f) => Value::from(*f),
        Default::Str(s) => Value::String((*s).to_string()),
        Default::Json(lit) => serde_json::from_str(lit).unwrap_or(Value::Null),
    }
}

/// Validate a value against its key definition (03 §3.6); `Err` carries the
/// reason for the collective E8002 notice (REQ-SET-003).
pub fn validate(def: &KeyDef, value: &Value) -> Result<(), String> {
    let reason = |what: &str| format!("{} expects {}", def.key, what);
    match &def.kind {
        Kind::Bool => {
            if value.is_boolean() {
                Ok(())
            } else {
                Err(reason("a boolean"))
            }
        }
        Kind::Int { min, max, step } => {
            let Some(v) = value.as_i64() else {
                return Err(reason("an integer"));
            };
            if v < *min || v > *max {
                return Err(reason(&format!("an integer in {min}..={max}")));
            }
            if let Some(step) = step
                && (v - min) % step != 0
            {
                return Err(reason(&format!("a multiple of {step} in {min}..={max}")));
            }
            Ok(())
        }
        Kind::Float { min, max } => {
            let Some(v) = value.as_f64() else {
                return Err(reason("a number"));
            };
            if !v.is_finite() || v < *min || v > *max {
                return Err(reason(&format!("a finite number in {min}..={max}")));
            }
            Ok(())
        }
        Kind::Str => {
            if value.as_str().is_some_and(|s| !s.is_empty()) {
                Ok(())
            } else {
                Err(reason("a non-empty string"))
            }
        }
        Kind::Path => {
            if value.as_str().is_some_and(|s| !s.is_empty()) {
                Ok(())
            } else {
                Err(reason("a non-empty path string"))
            }
        }
        Kind::Enum(members) => {
            if value.as_str().is_some_and(|s| members.contains(&s)) {
                Ok(())
            } else {
                Err(reason(&format!("one of {}", members.join("|"))))
            }
        }
        Kind::EnumInt(members) => {
            if value.as_i64().is_some_and(|v| members.contains(&v)) {
                Ok(())
            } else {
                Err(reason(&format!(
                    "one of {}",
                    members
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join("|")
                )))
            }
        }
        Kind::Object => {
            if value.is_object() {
                Ok(())
            } else {
                Err(reason("an object"))
            }
        }
        Kind::Array => {
            if value.is_array() {
                Ok(())
            } else {
                Err(reason("an array"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn table_has_68_unique_keys_in_document_order() {
        assert_eq!(KEYS.len(), 68, "03 §3.6 has exactly 68 rows");
        let mut seen = HashSet::new();
        for k in KEYS {
            assert!(seen.insert(k.key), "duplicate key {}", k.key);
        }
        assert_eq!(
            find("alvs.general.language").map(|k| k.page),
            Some("General")
        );
        assert_eq!(KEYS.first().map(|k| k.key), Some("alvs.general.language"));
        assert_eq!(KEYS.last().map(|k| k.key), Some("alvs.ui.tourDone"));
        assert_eq!(find("nope"), None);
    }

    #[test]
    fn every_default_validates_against_its_own_kind() {
        for k in KEYS {
            let d = default_value(k);
            assert!(
                validate(k, &d).is_ok(),
                "{} default {d:?} must validate",
                k.key
            );
        }
    }

    #[test]
    fn integer_ranges_reject_outside_and_accept_boundaries() {
        for k in KEYS {
            let Kind::Int { min, max, step } = k.kind else {
                continue;
            };
            let v = |i: i64| Value::from(i);
            assert!(validate(k, &v(min)).is_ok(), "{} min", k.key);
            assert!(validate(k, &v(max)).is_ok(), "{} max", k.key);
            assert!(validate(k, &v(min - 1)).is_err(), "{} below min", k.key);
            assert!(validate(k, &v(max + 1)).is_err(), "{} above max", k.key);
            assert!(
                validate(k, &Value::from(1.5)).is_err(),
                "{} fractional",
                k.key
            );
            assert!(
                validate(k, &Value::String("60".into())).is_err(),
                "{} string",
                k.key
            );
            let _ = step;
        }
        // uiZoom's step alignment: 80 and 90 are valid, 85 is not (§3.6).
        let zoom = find("alvs.appearance.uiZoom").cloned();
        let Some(zoom) = zoom else {
            panic!("uiZoom must exist");
        };
        assert!(validate(&zoom, &Value::from(80)).is_ok());
        assert!(validate(&zoom, &Value::from(90)).is_ok());
        assert!(validate(&zoom, &Value::from(85)).is_err());
        assert!(validate(&zoom, &Value::from(200)).is_ok());
        assert!(validate(&zoom, &Value::from(201)).is_err());
    }

    #[test]
    fn float_ranges_reject_outside_and_nonfinite_strings() {
        for k in KEYS {
            let Kind::Float { min, max } = k.kind else {
                continue;
            };
            assert!(validate(k, &Value::from(min)).is_ok(), "{} min", k.key);
            assert!(validate(k, &Value::from(max)).is_ok(), "{} max", k.key);
            assert!(
                validate(k, &Value::from(min - 0.01)).is_err(),
                "{} below",
                k.key
            );
            assert!(
                validate(k, &Value::from(max + 0.01)).is_err(),
                "{} above",
                k.key
            );
            assert!(validate(k, &Value::Bool(false)).is_err(), "{} bool", k.key);
        }
    }

    #[test]
    fn enums_accept_members_and_reject_unknown() {
        for k in KEYS {
            match k.kind {
                Kind::Enum(members) => {
                    for m in members {
                        assert!(validate(k, &Value::String((*m).into())).is_ok());
                    }
                    assert!(validate(k, &Value::String("no-such-member".into())).is_err());
                }
                Kind::EnumInt(members) => {
                    for m in members {
                        assert!(validate(k, &Value::from(*m)).is_ok());
                    }
                    assert!(validate(k, &Value::from(-1)).is_err());
                    assert!(validate(k, &Value::String("1440".into())).is_err());
                }
                _ => {}
            }
        }
    }

    #[test]
    fn containers_and_strings_check_shape() {
        let obj = find("alvs.keyboard.overrides").cloned();
        let Some(obj) = obj else {
            panic!("overrides must exist")
        };
        assert!(validate(&obj, &serde_json::json!({})).is_ok());
        assert!(validate(&obj, &serde_json::json!({ "cmd.trim": ["Ctrl+T"] })).is_ok());
        assert!(validate(&obj, &serde_json::json!([])).is_err());
        let arr = find("alvs.workspaces.saved").cloned();
        let Some(arr) = arr else {
            panic!("workspaces.saved must exist")
        };
        assert!(validate(&arr, &serde_json::json!([])).is_ok());
        assert!(validate(&arr, &serde_json::json!([{"id":"WS-05"}])).is_ok());
        assert!(validate(&arr, &serde_json::json!({})).is_err());
        let dev = find("alvs.playback.audioDevice").cloned();
        let Some(dev) = dev else {
            panic!("audioDevice must exist")
        };
        assert!(validate(&dev, &Value::String("default".into())).is_ok());
        assert!(validate(&dev, &Value::String("{0.0.0.00000000}.{...}".into())).is_ok());
        assert!(validate(&dev, &Value::String(String::new())).is_err());
        assert!(validate(&dev, &Value::from(3)).is_err());
    }

    #[test]
    fn spot_defaults_match_the_3_6_table() {
        let want = [
            ("alvs.general.autosaveIntervalSec", Value::from(60)),
            ("alvs.general.backupsKeep", Value::from(20)),
            ("alvs.general.recentLimit", Value::from(20)),
            ("alvs.general.updateChannel", Value::String("stable".into())),
            ("alvs.appearance.theme", Value::String("dark".into())),
            ("alvs.appearance.accent", Value::String("CLR-06".into())),
            ("alvs.appearance.uiZoom", Value::from(100)),
            ("alvs.playback.gpuCacheMB", Value::from(1024)),
            ("alvs.timeline.defaultTransitionSec", Value::from(1.0)),
            ("alvs.captions.maxCharsPerLine", Value::from(42)),
            ("alvs.media.proxyThresholdHeight", Value::from(1440)),
            ("alvs.media.proxyResolution", Value::from(720)),
            ("alvs.media.cacheMaxGB", Value::from(20)),
            (
                "alvs.export.lastPresetId",
                Value::String("xp.h264.high".into()),
            ),
            ("alvs.advanced.logLevel", Value::String("info".into())),
            ("alvs.advanced.redactDiagnostics", Value::Bool(true)),
            ("alvs.workspaces.active", Value::String("WS-01".into())),
        ];
        for (key, value) in want {
            let def = find(key).cloned();
            let Some(def) = def else {
                panic!("{key} must exist")
            };
            assert_eq!(default_value(&def), value, "default of {key}");
        }
    }
}
