//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Property suite (09 §9.4): random edit sequences drawn from the WP-02 op
//! set. Every applied step must leave a valid model or reject with a
//! documented code. CI scale: 10,000 × 50 (nightly 1,000,000 via
//! `ALVS_PROPTEST_CASES`, research D13 — raising only, G-08).

use proptest::collection::vec;
use proptest::prelude::*;

use alvs_model::edit::EditOp;
use alvs_model::history::Model;
use alvs_model::project::new_project;
use alvs_model::sequence::{SequenceFormat, new_sequence, sqp02_fallback_format};
use alvs_model::track::TrackKind;

fn ci_cases() -> u32 {
    std::env::var("ALVS_PROPTEST_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10_000)
}

fn valid_project() -> Model {
    let mut seq = new_sequence("Main", sqp02_fallback_format());
    seq.format.auto_match_pending = false;
    Model::new(new_project("Test", seq))
}

/// A random op strategy over the WP-02 categories on the current state.
fn arb_op(seq_id: String, track_id: Option<String>) -> BoxedStrategy<EditOp> {
    prop_oneof![
        2 => Just(EditOp::RenameProject { name: "Renamed".into() }),
        2 => Just(EditOp::AddTrack {
            sequence_id: seq_id.clone(),
            kind: TrackKind::Video,
            index: None,
            name: None,
        }),
        2 => Just(EditOp::AddTrack {
            sequence_id: seq_id.clone(),
            kind: TrackKind::Audio,
            index: None,
            name: None,
        }),
        2 => Just(EditOp::AddMarkers {
            sequence_id: seq_id.clone(),
            markers: vec![alvs_model::edit::MarkerInput {
                t: 0,
                duration: 0,
                name: "m".into(),
                color: "CLR-03".into(),
                note: String::new(),
                chapter: false,
            }],
        }),
        1 => Just(EditOp::AddCaptionTrack {
            sequence_id: seq_id.clone(),
            name: "CC".into(),
            language: "en".into(),
        }),
        1 => match track_id.clone() {
            Some(tid) => Just(EditOp::UpdateTrack {
                sequence_id: seq_id.clone(),
                track_id: tid,
                changes: alvs_model::edit::TrackChanges {
                    name: Some("T".into()),
                    height: Some(64),
                    ..Default::default()
                },
            })
            .boxed(),
            None => Just(EditOp::RenameProject { name: "noop".into() }).boxed(),
        },
        1 => Just(EditOp::UpdateSequence {
            sequence_id: seq_id,
            changes: alvs_model::edit::SequenceChanges {
                name: Some("S".into()),
                format: Some(SequenceFormat {
                    width: 1920,
                    height: 1080,
                    frame_rate: alvs_core::rational::Rational::new(25, 1)
                        .unwrap_or(alvs_core::rational::Rational { num: 25, den: 1 }),
                    sample_rate: 48_000,
                    audio_layout: "stereo".into(),
                    color_space: "rec709-sdr".into(),
                    drop_frame_timecode: false,
                    auto_match_pending: false,
                    extra: Default::default(),
                }),
                ..Default::default()
            },
        }),
    ]
    .boxed()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(ci_cases()))]

    /// 09 §9.4: random edit sequences — every step either applies with all
    /// invariants held or rejects with a documented code.
    #[test]
    fn random_edit_sequences_hold_invariants(ops in (0..50u32).prop_flat_map(|n| {
        vec(arb_op("seq0".into(), Some("track0".into())), n as usize)
    })) {
        let mut model = valid_project();
        // Replace the generated sequence id with the real one.
        let seq_id = model.root().sequences[0].id.clone();
        for op in &ops {
            let op = rewrite_ids(op, &seq_id);
            match model.apply(&op) {
                Ok(_) => {
                    // Invariants held (apply validates in tests).
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

/// Replace the fixed test ids with the model's actual ids.
fn rewrite_ids(op: &EditOp, seq_id: &str) -> EditOp {
    match op {
        EditOp::RenameProject { name } => EditOp::RenameProject { name: name.clone() },
        EditOp::AddTrack {
            sequence_id: _,
            kind,
            index,
            name,
        } => EditOp::AddTrack {
            sequence_id: seq_id.into(),
            kind: *kind,
            index: *index,
            name: name.clone(),
        },
        EditOp::AddMarkers {
            sequence_id: _,
            markers,
        } => EditOp::AddMarkers {
            sequence_id: seq_id.into(),
            markers: markers.clone(),
        },
        EditOp::AddCaptionTrack {
            sequence_id: _,
            name,
            language,
        } => EditOp::AddCaptionTrack {
            sequence_id: seq_id.into(),
            name: name.clone(),
            language: language.clone(),
        },
        EditOp::UpdateTrack {
            sequence_id: _,
            track_id,
            changes,
        } => EditOp::UpdateTrack {
            sequence_id: seq_id.into(),
            track_id: track_id.clone(),
            changes: changes.clone(),
        },
        EditOp::UpdateSequence {
            sequence_id: _,
            changes,
        } => EditOp::UpdateSequence {
            sequence_id: seq_id.into(),
            changes: changes.clone(),
        },
        other => other.clone(),
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
