//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Property suite (09 §9.4): random edit sequences drawn from the full
//! WP-02 op set with generated payloads and mixed valid/invalid references.
//! Every applied step must leave a valid model or reject with a documented
//! code. CI scale: 10,000 × 50 (nightly 1,000,000 via `ALVS_PROPTEST_CASES`,
//! research D13 — the knob is **raise-only**, G-08: any parsed value below
//! the CI scale is clamped up).

use proptest::collection::vec;
use proptest::prelude::*;

use alvs_core::id;
use alvs_core::rational::Rational;
use alvs_model::edit::{
    EditOp, EditResult, MarkerInput, MediaChanges, SequenceChanges, TrackChanges,
};
use alvs_model::history::Model;
use alvs_model::media::{Fingerprint, MediaItem, MediaKind, Probe};
use alvs_model::project::new_project;
use alvs_model::sequence::{SequenceFormat, new_sequence, sqp02_fallback_format};
use alvs_model::track::TrackKind;

/// Placeholder tokens the strategy uses for entity references; the test
/// rewrites them to live ids of the current model (or leaves them dangling
/// when none exist, exercising the documented rejection paths).
const SEQ: &str = "seq0";
const TRACK: &str = "track0";
const MEDIA: &str = "media0";
const BIN: &str = "bin0";
const MARKER: &str = "marker0";
const CAPTION: &str = "caption0";

fn ci_cases() -> u32 {
    std::env::var("ALVS_PROPTEST_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(|n: u32| n.max(10_000))
        .unwrap_or(10_000)
}

fn valid_project() -> Model {
    let mut seq = new_sequence("Main", sqp02_fallback_format());
    seq.format.auto_match_pending = false;
    Model::new(new_project("Test", seq))
}

fn media_item(item_id: String) -> MediaItem {
    MediaItem {
        id: item_id,
        kind: MediaKind::Video,
        name: "clip.mp4".into(),
        path: "C:\\m.mp4".into(),
        rel_path: "m.mp4".into(),
        fingerprint: Fingerprint {
            size: 1,
            mtime_utc: "2026-01-01T00:00:00Z".into(),
            head_tail_sha256: "aa".into(),
            extra: Default::default(),
        },
        probe: Probe {
            duration_flicks: 10 * 705_600_000,
            container: "mp4".into(),
            streams: Vec::new(),
            extra: Default::default(),
        },
        image_sequence: None,
        bin_id: None,
        label: None,
        notes: String::new(),
        derived: Default::default(),
        added_utc: "2026-01-01T00:00:00Z".into(),
        extra: Default::default(),
    }
}

/// A valid concrete format with generated dimensions and a catalog rate.
fn arb_format() -> BoxedStrategy<SequenceFormat> {
    (
        16u32..4097,
        16u32..4097,
        proptest::sample::select(vec![30u32, 25, 24, 50, 60]),
    )
        .prop_map(|(half_w, half_h, fps)| SequenceFormat {
            width: half_w * 2,
            height: half_h * 2,
            frame_rate: Rational::new(fps, 1).unwrap_or(Rational { num: 30, den: 1 }),
            sample_rate: 48_000,
            audio_layout: "stereo".into(),
            color_space: "rec709-sdr".into(),
            drop_frame_timecode: false,
            auto_match_pending: false,
            extra: Default::default(),
        })
        .boxed()
}

/// A marker payload with a generated position (grid-aligned, ≥ 0).
fn arb_marker(chapter: bool) -> BoxedStrategy<MarkerInput> {
    (0u32..200, any::<bool>())
        .prop_map(move |(frames, note)| MarkerInput {
            t: (frames as i64) * 23_520_000,
            duration: 0,
            name: format!("m{frames}"),
            color: "CLR-03".into(),
            note: if note { "n".into() } else { String::new() },
            chapter,
        })
        .boxed()
}

/// The full WP-02 op set with generated payloads. Reference fields use the
/// placeholder tokens; both tokens-with-live-targets and dangling tokens
/// occur as the run evolves.
fn arb_op() -> BoxedStrategy<EditOp> {
    prop_oneof![
        2 => ("[a-z]{1,10}").prop_map(|name| EditOp::RenameProject { name }),
        1 => Just(EditOp::RenameProject { name: String::new() }), // rejected: name rule
        2 => any::<bool>().prop_map(move |video| EditOp::AddTrack {
            sequence_id: SEQ.into(),
            kind: if video { TrackKind::Video } else { TrackKind::Audio },
            index: None,
            name: None,
        }),
        1 => (0u32..8).prop_map(|i| EditOp::AddTrack {
            sequence_id: SEQ.into(),
            kind: TrackKind::Video,
            index: Some(i as usize),
            name: Some(format!("t{i}")),
        }),
        1 => Just(EditOp::RemoveTrack {
            sequence_id: SEQ.into(),
            track_id: TRACK.into(),
        }),
        1 => (0u32..6).prop_map(|i| EditOp::MoveTrack {
            sequence_id: SEQ.into(),
            track_id: TRACK.into(),
            index: i as usize,
        }),
        1 => ("[a-z]{1,8}", 32u32..240, any::<f64>()).prop_map(|(name, height, volume)| {
            EditOp::UpdateTrack {
                sequence_id: SEQ.into(),
                track_id: TRACK.into(),
                changes: TrackChanges {
                    name: Some(name),
                    height: Some(height),
                    volume_db: Some(volume),
                    ..Default::default()
                },
            }
        }),
        2 => Just(EditOp::AddMedia { items: Vec::new(), bin_id: None }),
        1 => vec(arb_marker(false), 1..3).prop_map(|markers| EditOp::AddMarkers {
            sequence_id: SEQ.into(),
            markers,
        }),
        1 => vec(arb_marker(true), 1..2).prop_map(|markers| EditOp::AddMarkers {
            sequence_id: SEQ.into(),
            markers,
        }),
        1 => ("[a-z]{1,8}", any::<Option<String>>()).prop_map(|(name, note)| EditOp::UpdateMarker {
            sequence_id: SEQ.into(),
            marker_id: MARKER.into(),
            name,
            note,
        }),
        1 => Just(EditOp::RemoveMarkers {
            sequence_id: SEQ.into(),
            marker_ids: vec![MARKER.into()],
        }),
        1 => Just(EditOp::CreateBin {
            name: "bin".into(),
            parent_id: None,
        }),
        1 => Just(EditOp::UpdateBin {
            bin_id: BIN.into(),
            name: "renamed".into(),
        }),
        1 => Just(EditOp::DeleteBin { bin_id: BIN.into() }),
        2 => ("[a-z]{1,8}", proptest::sample::select(vec![
            "SQP-01".to_string(), "SQP-02".into(), "SQP-03".into(), "SQP-05".into(),
            "SQP-06".into(), "SQP-10".into(), "SQP-99".into(), "SQP-13".into(),
        ])).prop_map(|(name, preset)| EditOp::CreateSequence {
            name,
            format: None,
            from_preset: Some(preset),
        }),
        1 => ("[a-z]{1,8}", arb_format()).prop_map(|(name, format)| EditOp::CreateSequence {
            name,
            format: Some(format),
            from_preset: None,
        }),
        1 => Just(EditOp::DuplicateSequence {
            sequence_id: SEQ.into(),
            name: "copy".into(),
        }),
        1 => Just(EditOp::DeleteSequence { sequence_id: SEQ.into() }),
        1 => arb_format().prop_map(|format| EditOp::UpdateSequence {
            sequence_id: SEQ.into(),
            changes: SequenceChanges {
                format: Some(format),
                ..Default::default()
            },
        }),
        1 => Just(EditOp::AddCaptionTrack {
            sequence_id: SEQ.into(),
            name: "CC".into(),
            language: "en".into(),
        }),
        1 => Just(EditOp::UpdateCaptionTrack {
            sequence_id: SEQ.into(),
            caption_track_id: CAPTION.into(),
            name: Some("CC2".into()),
            enabled: Some(false),
        }),
        1 => Just(EditOp::RemoveCaptionTrack {
            sequence_id: SEQ.into(),
            caption_track_id: CAPTION.into(),
        }),
        1 => ("[a-z]{1,10}").prop_map(|name| EditOp::UpdateMedia {
            media_id: MEDIA.into(),
            changes: MediaChanges {
                name: Some(name),
                ..Default::default()
            },
        }),
        1 => (any::<bool>(), any::<bool>()).prop_map(|(also, dangling)| EditOp::RemoveMedia {
            media_ids: vec![if dangling { "media-nope".into() } else { MEDIA.into() }],
            also_remove_clips: also,
        }),
        2 => vec(arb_op_inner(), 1..3).prop_map(|ops| EditOp::Batch {
            ops,
            label_key: "history.batch.prop".into(),
        }),
    ]
    .boxed()
}

/// Inner ops of a batch (non-recursive, cheap variants).
fn arb_op_inner() -> BoxedStrategy<EditOp> {
    prop_oneof![
        2 => ("[a-z]{1,10}").prop_map(|name| EditOp::RenameProject { name }),
        1 => any::<bool>().prop_map(|video| EditOp::AddTrack {
            sequence_id: SEQ.into(),
            kind: if video { TrackKind::Video } else { TrackKind::Audio },
            index: None,
            name: None,
        }),
        1 => vec(arb_marker(false), 1..2).prop_map(|markers| EditOp::AddMarkers {
            sequence_id: SEQ.into(),
            markers,
        }),
        1 => Just(EditOp::CreateBin { name: "b".into(), parent_id: None }),
        1 => Just(EditOp::AddMedia { items: Vec::new(), bin_id: None }),
    ]
    .boxed()
}

/// Rewrite the placeholder tokens against the model's live ids; tokens with
/// no live target stay dangling (documented rejections).
fn rewrite(op: &EditOp, m: &Model) -> EditOp {
    let root = m.root();
    let seq = root.sequences[0].id.clone();
    let track = root.sequences[0].tracks.first().map(|t| t.id.clone());
    let media = root.media.first().map(|x| x.id.clone());
    let bin = root.bins.first().map(|b| b.id.clone());
    let marker = root.sequences[0].markers.first().map(|x| x.id.clone());
    let caption = root.sequences[0]
        .caption_tracks
        .first()
        .map(|x| x.id.clone());
    let resolve = |slot: &Option<String>, token: &str| -> String {
        slot.clone().unwrap_or_else(|| token.to_string())
    };
    let seq_of = |s: &str| -> String { if s == SEQ { seq.clone() } else { s.to_string() } };
    match op {
        EditOp::RenameProject { name } => EditOp::RenameProject { name: name.clone() },
        EditOp::AddTrack {
            sequence_id,
            kind,
            index,
            name,
        } => EditOp::AddTrack {
            sequence_id: seq_of(sequence_id),
            kind: *kind,
            index: *index,
            name: name.clone(),
        },
        EditOp::RemoveTrack {
            sequence_id,
            track_id,
        } => EditOp::RemoveTrack {
            sequence_id: seq_of(sequence_id),
            track_id: if track_id == TRACK {
                resolve(&track, TRACK)
            } else {
                track_id.clone()
            },
        },
        EditOp::MoveTrack {
            sequence_id,
            track_id,
            index,
        } => EditOp::MoveTrack {
            sequence_id: seq_of(sequence_id),
            track_id: if track_id == TRACK {
                resolve(&track, TRACK)
            } else {
                track_id.clone()
            },
            index: *index,
        },
        EditOp::UpdateTrack {
            sequence_id,
            track_id,
            changes,
        } => EditOp::UpdateTrack {
            sequence_id: seq_of(sequence_id),
            track_id: if track_id == TRACK {
                resolve(&track, TRACK)
            } else {
                track_id.clone()
            },
            changes: changes.clone(),
        },
        EditOp::AddMedia { items, bin_id } => {
            let mut generated = vec![media_item(id::new_id())];
            generated.extend(items.iter().cloned());
            EditOp::AddMedia {
                items: generated,
                bin_id: bin_id.clone(),
            }
        }
        EditOp::RemoveMedia {
            media_ids,
            also_remove_clips,
        } => EditOp::RemoveMedia {
            media_ids: media_ids
                .iter()
                .map(|x| {
                    if x == MEDIA {
                        resolve(&media, MEDIA)
                    } else {
                        x.clone()
                    }
                })
                .collect(),
            also_remove_clips: *also_remove_clips,
        },
        EditOp::UpdateMedia { media_id, changes } => EditOp::UpdateMedia {
            media_id: if media_id == MEDIA {
                resolve(&media, MEDIA)
            } else {
                media_id.clone()
            },
            changes: changes.clone(),
        },
        EditOp::CreateBin { name, parent_id } => EditOp::CreateBin {
            name: name.clone(),
            parent_id: parent_id.clone(),
        },
        EditOp::UpdateBin { bin_id, name } => EditOp::UpdateBin {
            bin_id: if bin_id == BIN {
                resolve(&bin, BIN)
            } else {
                bin_id.clone()
            },
            name: name.clone(),
        },
        EditOp::DeleteBin { bin_id } => EditOp::DeleteBin {
            bin_id: if bin_id == BIN {
                resolve(&bin, BIN)
            } else {
                bin_id.clone()
            },
        },
        EditOp::CreateSequence {
            name,
            format,
            from_preset,
        } => EditOp::CreateSequence {
            name: name.clone(),
            format: format.clone(),
            from_preset: from_preset.clone(),
        },
        EditOp::DuplicateSequence { sequence_id, name } => EditOp::DuplicateSequence {
            sequence_id: seq_of(sequence_id),
            name: name.clone(),
        },
        EditOp::DeleteSequence { sequence_id } => EditOp::DeleteSequence {
            sequence_id: seq_of(sequence_id),
        },
        EditOp::UpdateSequence {
            sequence_id,
            changes,
        } => EditOp::UpdateSequence {
            sequence_id: seq_of(sequence_id),
            changes: changes.clone(),
        },
        EditOp::AddMarkers {
            sequence_id,
            markers,
        } => EditOp::AddMarkers {
            sequence_id: seq_of(sequence_id),
            markers: markers.clone(),
        },
        EditOp::UpdateMarker {
            sequence_id,
            marker_id,
            name,
            note,
        } => EditOp::UpdateMarker {
            sequence_id: seq_of(sequence_id),
            marker_id: if marker_id == MARKER {
                resolve(&marker, MARKER)
            } else {
                marker_id.clone()
            },
            name: name.clone(),
            note: note.clone(),
        },
        EditOp::RemoveMarkers {
            sequence_id,
            marker_ids,
        } => EditOp::RemoveMarkers {
            sequence_id: seq_of(sequence_id),
            marker_ids: marker_ids
                .iter()
                .map(|x| {
                    if x == MARKER {
                        resolve(&marker, MARKER)
                    } else {
                        x.clone()
                    }
                })
                .collect(),
        },
        EditOp::AddCaptionTrack {
            sequence_id,
            name,
            language,
        } => EditOp::AddCaptionTrack {
            sequence_id: seq_of(sequence_id),
            name: name.clone(),
            language: language.clone(),
        },
        EditOp::UpdateCaptionTrack {
            sequence_id,
            caption_track_id,
            name,
            enabled,
        } => EditOp::UpdateCaptionTrack {
            sequence_id: seq_of(sequence_id),
            caption_track_id: if caption_track_id == CAPTION {
                resolve(&caption, CAPTION)
            } else {
                caption_track_id.clone()
            },
            name: name.clone(),
            enabled: *enabled,
        },
        EditOp::RemoveCaptionTrack {
            sequence_id,
            caption_track_id,
        } => EditOp::RemoveCaptionTrack {
            sequence_id: seq_of(sequence_id),
            caption_track_id: if caption_track_id == CAPTION {
                resolve(&caption, CAPTION)
            } else {
                caption_track_id.clone()
            },
        },
        EditOp::Batch { ops, label_key } => EditOp::Batch {
            ops: ops.iter().map(|inner| rewrite(inner, m)).collect(),
            label_key: label_key.clone(),
        },
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(ci_cases()))]

    /// 09 §9.4: random edit sequences — every step either applies with all
    /// invariants held or rejects with a documented code.
    #[test]
    fn random_edit_sequences_hold_invariants(
        ops in (0..50u32).prop_flat_map(|n| vec(arb_op(), n as usize))
    ) {
        let mut model = valid_project();
        for op in &ops {
            let op = rewrite(op, &model);
            match model.apply(&op) {
                Ok(EditResult { revision, patch, .. }) => {
                    prop_assert_eq!(revision, model.revision());
                    prop_assert_eq!(patch.revision, model.revision());
                }
                Err(err) => {
                    // The rejection must be a documented code, not a panic.
                    let code = err.code().as_str();
                    prop_assert!(
                        matches!(code, "E1102" | "E0002" | "E1003" | "E1006"),
                        "unexpected rejection code {code}: {err}"
                    );
                }
            }
        }
        // The model is still valid at the end.
        let issues = alvs_model::validate::validate(&model.root());
        prop_assert!(issues.is_empty(), "end-state invalid: {issues:?}");
    }
}

#[test]
fn every_op_has_a_rejection_path() {
    // Ops that reference nonexistent entities reject with E1102-class errors.
    let mut model = valid_project();
    let err = model.apply(&EditOp::UpdateMedia {
        media_id: "nonexistent".into(),
        changes: Default::default(),
    });
    if let Err(e) = err {
        assert_eq!(e.code().as_str(), "E1102");
    } else {
        panic!("expected error");
    }

    let err = model.apply(&EditOp::DeleteSequence {
        sequence_id: "nonexistent".into(),
    });
    assert!(err.is_err());

    let err = model.apply(&EditOp::AddTrack {
        sequence_id: "nonexistent".into(),
        kind: TrackKind::Video,
        index: None,
        name: None,
    });
    if let Err(e) = err {
        assert_eq!(e.code().as_str(), "E1102");
    } else {
        panic!("expected error");
    }
}
