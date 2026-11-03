//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Application coverage for every `EditOp` variant of the WP-02 categories:
//! each happy path and each documented rejection (T-EDT suite, 09 §9.3).

use alvs_core::id;
use alvs_core::rational::Rational;
use alvs_model::edit::{
    EditOp, EditResult, MarkerInput, MediaChanges, SequenceChanges, TrackChanges,
};
use alvs_model::history::Model;
use alvs_model::media::{Fingerprint, MediaItem, MediaKind, Probe};
use alvs_model::project::new_project;
use alvs_model::sequence::{SequenceFormat, new_sequence, sqp02_fallback_format};
use alvs_model::track::{Clip, ClipType, Deinterlace, TrackKind, new_track};

/// Flicks per frame at 30 fps.
const FPF_30: i64 = 23_520_000;
/// Flicks per frame at 25 fps.
const FPF_25: i64 = 28_224_000;

fn fmt() -> SequenceFormat {
    let mut format = sqp02_fallback_format();
    format.auto_match_pending = false;
    format
}

fn fps25() -> SequenceFormat {
    SequenceFormat {
        width: 1920,
        height: 1080,
        frame_rate: Rational::new(25, 1).unwrap_or(Rational { num: 25, den: 1 }),
        sample_rate: 48_000,
        audio_layout: "stereo".into(),
        color_space: "rec709-sdr".into(),
        drop_frame_timecode: false,
        auto_match_pending: false,
        extra: Default::default(),
    }
}

fn empty_model() -> Model {
    Model::new(new_project("Test", new_sequence("Main", fmt())))
}

fn media_item(id: String) -> MediaItem {
    MediaItem {
        id,
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

fn clip(
    id: String,
    clip_type: ClipType,
    start: i64,
    duration: i64,
    media_id: Option<String>,
) -> Clip {
    Clip {
        id,
        clip_type,
        stream_index: media_id.as_ref().map(|_| 0),
        media_id,
        start,
        duration,
        source_in: 0,
        speed: Rational::new(1, 1).unwrap_or(Rational { num: 1, den: 1 }),
        reverse: false,
        freeze_at: None,
        enabled: true,
        name: None,
        label: None,
        link_group_id: None,
        group_id: None,
        intrinsic: None,
        audio: None,
        deinterlace: Deinterlace::Auto,
        effects: Vec::new(),
        masks: Vec::new(),
        markers: Vec::new(),
        title: None,
        generator: None,
        sequence_id: None,
        extra: Default::default(),
    }
}

/// A project with one media item and one media clip `[2f, 5f)` at 30 fps.
fn model_with_media_clip() -> (Model, String) {
    let media = media_item(id::new_id());
    let media_id = media.id.clone();
    let mut sequence = new_sequence("Main", fmt());
    let mut track = new_track(TrackKind::Video, "V1");
    track.items.push(clip(
        id::new_id(),
        ClipType::Media,
        2 * FPF_30,
        3 * FPF_30,
        Some(media.id.clone()),
    ));
    sequence.tracks.push(track);
    let mut project = new_project("Test", sequence);
    project.media.push(media);
    (Model::new(project), media_id)
}

fn ok(m: &mut Model, op: &EditOp) -> EditResult {
    match m.apply(op) {
        Ok(result) => result,
        Err(e) => panic!("expected {} to apply, got {e:?}", op.history_key()),
    }
}

fn rejected_code(m: &mut Model, op: &EditOp) -> String {
    match m.apply(op) {
        Err(e) => e.code().as_str().to_string(),
        Ok(_) => panic!("expected {} to be rejected", op.history_key()),
    }
}

#[test]
fn history_keys_cover_every_variant() {
    let ops: Vec<(EditOp, &str)> = vec![
        (
            EditOp::AddMedia {
                items: Vec::new(),
                bin_id: None,
            },
            "history.addMedia",
        ),
        (
            EditOp::RemoveMedia {
                media_ids: Vec::new(),
                also_remove_clips: false,
            },
            "history.removeMedia",
        ),
        (
            EditOp::UpdateMedia {
                media_id: String::new(),
                changes: Default::default(),
            },
            "history.updateMedia",
        ),
        (
            EditOp::CreateBin {
                name: String::new(),
                parent_id: None,
            },
            "history.createBin",
        ),
        (
            EditOp::UpdateBin {
                bin_id: String::new(),
                name: String::new(),
            },
            "history.updateBin",
        ),
        (
            EditOp::DeleteBin {
                bin_id: String::new(),
            },
            "history.deleteBin",
        ),
        (
            EditOp::CreateSequence {
                name: String::new(),
                format: None,
                from_preset: None,
            },
            "history.createSequence",
        ),
        (
            EditOp::DuplicateSequence {
                sequence_id: String::new(),
                name: String::new(),
            },
            "history.duplicateSequence",
        ),
        (
            EditOp::DeleteSequence {
                sequence_id: String::new(),
            },
            "history.deleteSequence",
        ),
        (
            EditOp::UpdateSequence {
                sequence_id: String::new(),
                changes: Default::default(),
            },
            "history.updateSequence",
        ),
        (
            EditOp::AddTrack {
                sequence_id: String::new(),
                kind: TrackKind::Video,
                index: None,
                name: None,
            },
            "history.addTrack",
        ),
        (
            EditOp::RemoveTrack {
                sequence_id: String::new(),
                track_id: String::new(),
            },
            "history.removeTrack",
        ),
        (
            EditOp::MoveTrack {
                sequence_id: String::new(),
                track_id: String::new(),
                index: 0,
            },
            "history.moveTrack",
        ),
        (
            EditOp::UpdateTrack {
                sequence_id: String::new(),
                track_id: String::new(),
                changes: Default::default(),
            },
            "history.updateTrack",
        ),
        (
            EditOp::AddMarkers {
                sequence_id: String::new(),
                markers: Vec::new(),
            },
            "history.addMarkers",
        ),
        (
            EditOp::UpdateMarker {
                sequence_id: String::new(),
                marker_id: String::new(),
                name: String::new(),
                note: String::new(),
            },
            "history.updateMarker",
        ),
        (
            EditOp::RemoveMarkers {
                sequence_id: String::new(),
                marker_ids: Vec::new(),
            },
            "history.removeMarkers",
        ),
        (
            EditOp::AddCaptionTrack {
                sequence_id: String::new(),
                name: String::new(),
                language: String::new(),
            },
            "history.addCaptionTrack",
        ),
        (
            EditOp::UpdateCaptionTrack {
                sequence_id: String::new(),
                caption_track_id: String::new(),
                name: None,
                enabled: None,
            },
            "history.updateCaptionTrack",
        ),
        (
            EditOp::RemoveCaptionTrack {
                sequence_id: String::new(),
                caption_track_id: String::new(),
            },
            "history.removeCaptionTrack",
        ),
        (
            EditOp::RenameProject {
                name: String::new(),
            },
            "history.renameProject",
        ),
        (
            EditOp::Batch {
                ops: Vec::new(),
                label_key: String::new(),
            },
            "history.batch",
        ),
    ];
    assert_eq!(ops.len(), 22);
    for (op, key) in ops {
        assert_eq!(op.history_key(), key);
    }
}

#[test]
fn add_media_assigns_the_requested_bin() {
    let mut m = empty_model();
    ok(
        &mut m,
        &EditOp::CreateBin {
            name: "Footage".into(),
            parent_id: None,
        },
    );
    let bin_id = m.root().bins[0].id.clone();
    ok(
        &mut m,
        &EditOp::AddMedia {
            items: vec![media_item(id::new_id())],
            bin_id: Some(bin_id.clone()),
        },
    );
    assert_eq!(m.root().media[0].bin_id.as_deref(), Some(bin_id.as_str()));
}

#[test]
fn remove_media_in_use_is_blocked_until_requested() {
    let (mut m, media_id) = model_with_media_clip();
    let code = rejected_code(
        &mut m,
        &EditOp::RemoveMedia {
            media_ids: vec![media_id.clone()],
            also_remove_clips: false,
        },
    );
    assert_eq!(code, "E1102");
    assert_eq!(m.root().media.len(), 1);
    assert_eq!(m.root().sequences[0].tracks[0].items.len(), 1);

    ok(
        &mut m,
        &EditOp::RemoveMedia {
            media_ids: vec![media_id],
            also_remove_clips: true,
        },
    );
    assert!(m.root().media.is_empty());
    assert!(m.root().sequences[0].tracks[0].items.is_empty());
}

#[test]
fn remove_unknown_media_is_a_silent_noop() {
    let mut m = empty_model();
    ok(
        &mut m,
        &EditOp::RemoveMedia {
            media_ids: vec!["nonexistent".into()],
            also_remove_clips: false,
        },
    );
    assert!(m.root().media.is_empty());
}

#[test]
fn update_media_changes_every_field_and_rejects_unknown() {
    let mut m = empty_model();
    ok(
        &mut m,
        &EditOp::CreateBin {
            name: "B".into(),
            parent_id: None,
        },
    );
    let bin_id = m.root().bins[0].id.clone();
    ok(
        &mut m,
        &EditOp::AddMedia {
            items: vec![media_item(id::new_id())],
            bin_id: None,
        },
    );
    let media_id = m.root().media[0].id.clone();
    ok(
        &mut m,
        &EditOp::UpdateMedia {
            media_id,
            changes: MediaChanges {
                name: Some("renamed.mov".into()),
                label: Some("CLR-01".into()),
                notes: Some("note".into()),
                bin_id: Some(bin_id),
            },
        },
    );
    let media = &m.root().media[0];
    assert_eq!(media.name, "renamed.mov");
    assert_eq!(media.label.as_deref(), Some("CLR-01"));
    assert_eq!(media.notes, "note");

    let code = rejected_code(
        &mut m,
        &EditOp::UpdateMedia {
            media_id: "nonexistent".into(),
            changes: Default::default(),
        },
    );
    assert_eq!(code, "E1102");
}

#[test]
fn bin_rename_and_missing_rejections() {
    let mut m = empty_model();
    ok(
        &mut m,
        &EditOp::CreateBin {
            name: "Old".into(),
            parent_id: None,
        },
    );
    let bin_id = m.root().bins[0].id.clone();
    ok(
        &mut m,
        &EditOp::UpdateBin {
            bin_id: bin_id.clone(),
            name: "New".into(),
        },
    );
    assert_eq!(m.root().bins[0].name, "New");

    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::UpdateBin {
                bin_id: "nonexistent".into(),
                name: "x".into()
            }
        ),
        "E1102"
    );
    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::DeleteBin {
                bin_id: "nonexistent".into()
            }
        ),
        "E1102"
    );
}

#[test]
fn delete_bin_moves_contents_to_parent_or_to_none() {
    let mut m = empty_model();
    ok(
        &mut m,
        &EditOp::CreateBin {
            name: "P".into(),
            parent_id: None,
        },
    );
    let parent_id = m.root().bins[0].id.clone();
    ok(
        &mut m,
        &EditOp::CreateBin {
            name: "C".into(),
            parent_id: Some(parent_id.clone()),
        },
    );
    let child_id = m.root().bins[1].id.clone();
    ok(
        &mut m,
        &EditOp::AddMedia {
            items: vec![media_item(id::new_id())],
            bin_id: Some(child_id.clone()),
        },
    );

    // Deleting C moves its media up to P.
    ok(&mut m, &EditOp::DeleteBin { bin_id: child_id });
    assert_eq!(
        m.root().media[0].bin_id.as_deref(),
        Some(parent_id.as_str())
    );
    assert_eq!(m.root().bins.len(), 1);

    // Deleting the root bin (no parent) orphans its contents to none.
    ok(&mut m, &EditOp::DeleteBin { bin_id: parent_id });
    assert_eq!(m.root().media[0].bin_id, None);
    assert!(m.root().bins.is_empty());
}

#[test]
fn create_sequence_resolves_format_and_presets() {
    let mut m = empty_model();
    ok(
        &mut m,
        &EditOp::CreateSequence {
            name: "S1".into(),
            format: None,
            from_preset: None,
        },
    );
    assert!(m.root().sequences[1].format.auto_match_pending);
    ok(
        &mut m,
        &EditOp::CreateSequence {
            name: "S2".into(),
            format: None,
            from_preset: Some("SQP-01".into()),
        },
    );
    assert!(m.root().sequences[2].format.auto_match_pending);
    ok(
        &mut m,
        &EditOp::CreateSequence {
            name: "S3".into(),
            format: None,
            from_preset: Some("SQP-03".into()),
        },
    );
    assert!(!m.root().sequences[3].format.auto_match_pending);
    ok(
        &mut m,
        &EditOp::CreateSequence {
            name: "S4".into(),
            format: Some(fps25()),
            from_preset: None,
        },
    );
    assert_eq!(m.root().sequences[4].format.frame_rate.as_f64(), 25.0);
}

#[test]
fn duplicate_sequence_copies_with_a_new_id_and_rejects_missing() {
    let mut m = empty_model();
    ok(
        &mut m,
        &EditOp::CreateSequence {
            name: "Empty".into(),
            format: None,
            from_preset: None,
        },
    );
    let source_id = m.root().sequences[1].id.clone();
    ok(
        &mut m,
        &EditOp::DuplicateSequence {
            sequence_id: source_id.clone(),
            name: "Copy".into(),
        },
    );
    assert_eq!(m.root().sequences.len(), 3);
    assert_eq!(m.root().sequences[2].name, "Copy");
    assert_ne!(m.root().sequences[2].id, source_id);

    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::DuplicateSequence {
                sequence_id: "nonexistent".into(),
                name: "x".into()
            }
        ),
        "E1102"
    );
}

#[test]
fn delete_sequence_switches_references_and_enforces_rules() {
    let mut first = new_sequence("Main", fmt());
    first.tracks.push(new_track(TrackKind::Video, "V1"));
    let first_id = first.id.clone();
    let mut project = new_project("T", first);
    project.sequences.push(new_sequence("S2", fmt()));
    project.ui.active_sequence_id = Some(first_id.clone());
    let mut m = Model::new(project);

    ok(
        &mut m,
        &EditOp::DeleteSequence {
            sequence_id: first_id,
        },
    );
    assert_eq!(m.root().sequences.len(), 1);
    let survivor = m.root().sequences[0].id.clone();
    assert_eq!(m.root().settings.default_sequence_id, survivor);
    assert_eq!(
        m.root().ui.active_sequence_id.as_deref(),
        Some(survivor.as_str())
    );

    // The last sequence cannot be deleted (REQ-PRJ-005).
    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::DeleteSequence {
                sequence_id: survivor.clone()
            }
        ),
        "E1102"
    );
    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::DeleteSequence {
                sequence_id: "nonexistent".into()
            }
        ),
        "E1102"
    );
}

#[test]
fn update_sequence_name_and_points() {
    let mut m = empty_model();
    let sequence_id = m.root().sequences[0].id.clone();
    ok(
        &mut m,
        &EditOp::UpdateSequence {
            sequence_id: sequence_id.clone(),
            changes: SequenceChanges {
                name: Some("Renamed".into()),
                in_point: Some(Some(10)),
                out_point: Some(None),
                ..Default::default()
            },
        },
    );
    assert_eq!(m.root().sequences[0].name, "Renamed");
    assert_eq!(m.root().sequences[0].in_point, Some(10));
    assert_eq!(m.root().sequences[0].out_point, None);

    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::UpdateSequence {
                sequence_id: "nonexistent".into(),
                changes: Default::default()
            }
        ),
        "E1102"
    );
}

#[test]
fn update_sequence_fps_snaps_clips_and_reports_e1104() {
    let (mut m, media_id) = model_with_media_clip();
    let _ = media_id;
    let sequence_id = m.root().sequences[0].id.clone();
    let clip_id = m.root().sequences[0].tracks[0].items[0].id.clone();
    let result = ok(
        &mut m,
        &EditOp::UpdateSequence {
            sequence_id,
            changes: SequenceChanges {
                format: Some(fps25()),
                ..Default::default()
            },
        },
    );
    assert_eq!(result.adjusted_items, vec![clip_id]);
    assert!(result.warnings.iter().any(|(code, _)| code == "E1104"));
    let clip = &m.root().sequences[0].tracks[0].items[0];
    assert_eq!(clip.start, 2 * FPF_25);
    assert_eq!(clip.duration, 3 * FPF_25);
    assert_eq!(m.root().sequences[0].format.frame_rate.as_f64(), 25.0);
}

#[test]
fn update_sequence_same_fps_still_replaces_the_format() {
    let mut m = empty_model();
    let sequence_id = m.root().sequences[0].id.clone();
    let mut same_fps = fmt();
    same_fps.width = 1280;
    same_fps.height = 720;
    let result = ok(
        &mut m,
        &EditOp::UpdateSequence {
            sequence_id,
            changes: SequenceChanges {
                format: Some(same_fps),
                ..Default::default()
            },
        },
    );
    assert!(result.adjusted_items.is_empty());
    assert_eq!(m.root().sequences[0].format.width, 1280);
    assert_eq!(m.root().sequences[0].format.height, 720);
}

#[test]
fn add_track_names_index_and_rejections() {
    let mut m = empty_model();
    let sequence_id = m.root().sequences[0].id.clone();
    ok(
        &mut m,
        &EditOp::AddTrack {
            sequence_id: sequence_id.clone(),
            kind: TrackKind::Video,
            index: None,
            name: None,
        },
    );
    assert_eq!(m.root().sequences[0].tracks[0].name, "V1");
    ok(
        &mut m,
        &EditOp::AddTrack {
            sequence_id: sequence_id.clone(),
            kind: TrackKind::Audio,
            index: None,
            name: None,
        },
    );
    assert_eq!(m.root().sequences[0].tracks[1].name, "A1");
    ok(
        &mut m,
        &EditOp::AddTrack {
            sequence_id: sequence_id.clone(),
            kind: TrackKind::Video,
            index: None,
            name: Some("Custom".into()),
        },
    );
    assert_eq!(m.root().sequences[0].tracks[2].name, "Custom");
    // Index 0 inserts at the head; a huge index clamps to the end.
    ok(
        &mut m,
        &EditOp::AddTrack {
            sequence_id: sequence_id.clone(),
            kind: TrackKind::Video,
            index: Some(0),
            name: Some("Head".into()),
        },
    );
    assert_eq!(m.root().sequences[0].tracks[0].name, "Head");
    ok(
        &mut m,
        &EditOp::AddTrack {
            sequence_id: sequence_id.clone(),
            kind: TrackKind::Audio,
            index: Some(9_999),
            name: Some("Tail".into()),
        },
    );
    let tracks = &m.root().sequences[0].tracks;
    assert_eq!(tracks[tracks.len() - 1].name, "Tail");

    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::AddTrack {
                sequence_id: "nonexistent".into(),
                kind: TrackKind::Video,
                index: None,
                name: None
            }
        ),
        "E1102"
    );
}

#[test]
fn remove_track_and_missing_rejections() {
    let mut m = empty_model();
    let sequence_id = m.root().sequences[0].id.clone();
    ok(
        &mut m,
        &EditOp::AddTrack {
            sequence_id: sequence_id.clone(),
            kind: TrackKind::Video,
            index: None,
            name: None,
        },
    );
    let track_id = m.root().sequences[0].tracks[0].id.clone();
    ok(
        &mut m,
        &EditOp::RemoveTrack {
            sequence_id: sequence_id.clone(),
            track_id,
        },
    );
    assert!(m.root().sequences[0].tracks.is_empty());
    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::RemoveTrack {
                sequence_id: sequence_id.clone(),
                track_id: "nonexistent".into()
            }
        ),
        "E1102"
    );
    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::RemoveTrack {
                sequence_id: "nonexistent".into(),
                track_id: "x".into()
            }
        ),
        "E1102"
    );
}

#[test]
fn move_track_reorders_and_rejects_missing() {
    let mut m = empty_model();
    let sequence_id = m.root().sequences[0].id.clone();
    for name in ["V1", "V2", "V3"] {
        ok(
            &mut m,
            &EditOp::AddTrack {
                sequence_id: sequence_id.clone(),
                kind: TrackKind::Video,
                index: None,
                name: Some(name.into()),
            },
        );
    }
    let third = m.root().sequences[0].tracks[2].id.clone();
    ok(
        &mut m,
        &EditOp::MoveTrack {
            sequence_id: sequence_id.clone(),
            track_id: third.clone(),
            index: 0,
        },
    );
    assert_eq!(m.root().sequences[0].tracks[0].id, third);
    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::MoveTrack {
                sequence_id: sequence_id.clone(),
                track_id: "nonexistent".into(),
                index: 0
            }
        ),
        "E1102"
    );
    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::MoveTrack {
                sequence_id: "nonexistent".into(),
                track_id: "x".into(),
                index: 0
            }
        ),
        "E1102"
    );
}

#[test]
fn update_track_changes_every_field_and_rejects_missing() {
    let mut m = empty_model();
    let sequence_id = m.root().sequences[0].id.clone();
    ok(
        &mut m,
        &EditOp::AddTrack {
            sequence_id: sequence_id.clone(),
            kind: TrackKind::Audio,
            index: None,
            name: None,
        },
    );
    let track_id = m.root().sequences[0].tracks[0].id.clone();
    ok(
        &mut m,
        &EditOp::UpdateTrack {
            sequence_id: sequence_id.clone(),
            track_id: track_id.clone(),
            changes: TrackChanges {
                name: Some("A1x".into()),
                height: Some(120),
                locked: Some(true),
                visible: Some(false),
                muted: Some(true),
                solo: Some(true),
                volume_db: Some(-6.0),
                pan: Some(0.25),
                targeted: Some(false),
            },
        },
    );
    let track = &m.root().sequences[0].tracks[0];
    assert_eq!(track.name, "A1x");
    assert_eq!(track.height, 120);
    assert!(track.locked);
    assert!(!track.visible);
    assert!(track.muted);
    assert!(track.solo);
    assert_eq!(track.volume_db, -6.0);
    assert_eq!(track.pan, 0.25);
    assert!(!track.targeted);

    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::UpdateTrack {
                sequence_id: sequence_id.clone(),
                track_id: "nonexistent".into(),
                changes: Default::default()
            }
        ),
        "E1102"
    );
    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::UpdateTrack {
                sequence_id: "nonexistent".into(),
                track_id: "x".into(),
                changes: Default::default()
            }
        ),
        "E1102"
    );
}

#[test]
fn markers_chapter_and_standard_add_update_remove() {
    let mut m = empty_model();
    let sequence_id = m.root().sequences[0].id.clone();
    ok(
        &mut m,
        &EditOp::AddMarkers {
            sequence_id: sequence_id.clone(),
            markers: vec![
                MarkerInput {
                    t: 5 * FPF_30,
                    duration: 0,
                    name: "std".into(),
                    color: "CLR-03".into(),
                    note: String::new(),
                    chapter: false,
                },
                MarkerInput {
                    t: 10 * FPF_30,
                    duration: FPF_30,
                    name: "chap".into(),
                    color: "CLR-05".into(),
                    note: "note".into(),
                    chapter: true,
                },
            ],
        },
    );
    let markers = &m.root().sequences[0].markers;
    assert_eq!(markers.len(), 2);
    assert_eq!(markers[0].kind, alvs_model::marker::MarkerKind::Standard);
    assert_eq!(markers[1].kind, alvs_model::marker::MarkerKind::Chapter);

    let marker_id = markers[0].id.clone();
    ok(
        &mut m,
        &EditOp::UpdateMarker {
            sequence_id: sequence_id.clone(),
            marker_id: marker_id.clone(),
            name: "renamed".into(),
            note: "text".into(),
        },
    );
    assert_eq!(m.root().sequences[0].markers[0].name, "renamed");
    assert_eq!(m.root().sequences[0].markers[0].note, "text");

    ok(
        &mut m,
        &EditOp::RemoveMarkers {
            sequence_id: sequence_id.clone(),
            marker_ids: vec![marker_id],
        },
    );
    assert_eq!(m.root().sequences[0].markers.len(), 1);

    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::AddMarkers {
                sequence_id: "nonexistent".into(),
                markers: Vec::new()
            }
        ),
        "E1102"
    );
    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::UpdateMarker {
                sequence_id: sequence_id.clone(),
                marker_id: "nonexistent".into(),
                name: "x".into(),
                note: String::new()
            }
        ),
        "E1102"
    );
    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::UpdateMarker {
                sequence_id: "nonexistent".into(),
                marker_id: "x".into(),
                name: "x".into(),
                note: String::new()
            }
        ),
        "E1102"
    );
    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::RemoveMarkers {
                sequence_id: sequence_id.clone(),
                marker_ids: vec!["nonexistent".into()]
            }
        ),
        "E1102"
    );
    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::RemoveMarkers {
                sequence_id: "nonexistent".into(),
                marker_ids: vec!["x".into()]
            }
        ),
        "E1102"
    );
}

#[test]
fn caption_track_lifecycle_and_missing_rejections() {
    let mut m = empty_model();
    let sequence_id = m.root().sequences[0].id.clone();
    ok(
        &mut m,
        &EditOp::AddCaptionTrack {
            sequence_id: sequence_id.clone(),
            name: "CC".into(),
            language: "en".into(),
        },
    );
    assert_eq!(m.root().sequences[0].caption_tracks[0].name, "CC");
    let caption_track_id = m.root().sequences[0].caption_tracks[0].id.clone();
    ok(
        &mut m,
        &EditOp::UpdateCaptionTrack {
            sequence_id: sequence_id.clone(),
            caption_track_id: caption_track_id.clone(),
            name: Some("CC2".into()),
            enabled: Some(false),
        },
    );
    assert_eq!(m.root().sequences[0].caption_tracks[0].name, "CC2");
    assert!(!m.root().sequences[0].caption_tracks[0].enabled);

    ok(
        &mut m,
        &EditOp::RemoveCaptionTrack {
            sequence_id: sequence_id.clone(),
            caption_track_id: caption_track_id.clone(),
        },
    );
    assert!(m.root().sequences[0].caption_tracks.is_empty());

    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::AddCaptionTrack {
                sequence_id: "nonexistent".into(),
                name: "x".into(),
                language: "en".into()
            }
        ),
        "E1102"
    );
    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::UpdateCaptionTrack {
                sequence_id: "nonexistent".into(),
                caption_track_id: "x".into(),
                name: None,
                enabled: None
            }
        ),
        "E1102"
    );
    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::UpdateCaptionTrack {
                sequence_id: sequence_id.clone(),
                caption_track_id: "nonexistent".into(),
                name: None,
                enabled: None
            }
        ),
        "E1102"
    );
    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::RemoveCaptionTrack {
                sequence_id: "nonexistent".into(),
                caption_track_id: "x".into()
            }
        ),
        "E1102"
    );
    assert_eq!(
        rejected_code(
            &mut m,
            &EditOp::RemoveCaptionTrack {
                sequence_id: sequence_id.clone(),
                caption_track_id: "nonexistent".into()
            }
        ),
        "E1102"
    );
}

#[test]
fn rename_project_applies() {
    let mut m = empty_model();
    ok(
        &mut m,
        &EditOp::RenameProject {
            name: "Renamed".into(),
        },
    );
    assert_eq!(m.root().name, "Renamed");
}

#[test]
fn batch_is_atomic_and_a_single_undo_step() {
    let mut m = empty_model();
    ok(
        &mut m,
        &EditOp::Batch {
            ops: vec![
                EditOp::RenameProject { name: "A".into() },
                EditOp::RenameProject { name: "B".into() },
            ],
            label_key: "history.batch.test".into(),
        },
    );
    assert_eq!(m.root().name, "B");
    // One undo step returns to the pre-batch state.
    assert!(m.undo());
    assert_eq!(m.root().name, "Test");

    // A failing inner op rejects the whole batch and leaves the model unchanged.
    let code = rejected_code(
        &mut m,
        &EditOp::Batch {
            ops: vec![
                EditOp::RenameProject { name: "C".into() },
                EditOp::UpdateMedia {
                    media_id: "nonexistent".into(),
                    changes: Default::default(),
                },
            ],
            label_key: "history.batch.test".into(),
        },
    );
    assert_eq!(code, "E1102");
    assert_eq!(m.root().name, "Test");
}

#[test]
fn undo_redo_and_step_on_a_fresh_model_are_noops() {
    let mut m = empty_model();
    assert!(!m.undo());
    assert!(!m.redo());
    assert!(!m.step(0));
}
