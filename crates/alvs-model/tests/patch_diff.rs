//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! EntityPatch diff coverage (04 §4.4): upsert/remove for every entity kind
//! and the D10 dirty-range policy, including the `Arc::ptr_eq` short-circuit.

use std::sync::Arc;

use alvs_core::id;
use alvs_core::rational::Rational;
use alvs_model::edit::EditOp;
use alvs_model::history::Model;
use alvs_model::media::{Fingerprint, MediaItem, MediaKind, Probe};
use alvs_model::patch::diff;
use alvs_model::project::{Project, new_project};
use alvs_model::sequence::{SequenceFormat, new_sequence, sqp02_fallback_format};
use alvs_model::track::{Clip, ClipType, Deinterlace, TrackKind, new_track};

const FPF_30: i64 = 23_520_000;

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

fn clip(id: String, start: i64, duration: i64) -> Clip {
    Clip {
        id,
        clip_type: ClipType::Title,
        media_id: None,
        stream_index: None,
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

fn ok(m: &mut Model, op: &EditOp) {
    match m.apply(op) {
        Ok(_) => {}
        Err(e) => panic!("expected {} to apply, got {e:?}", op.history_key()),
    }
}

/// A project whose single track holds two title clips `[0,1f)` and `[1f,2f)`.
fn hand_project() -> Project {
    let mut sequence = new_sequence("Main", fmt());
    let mut track = new_track(TrackKind::Video, "V1");
    let c1 = clip(id::new_id(), 0, FPF_30);
    let c2 = clip(id::new_id(), FPF_30, FPF_30);
    track.items = vec![c1, c2];
    sequence.tracks.push(track);
    new_project("T", sequence)
}

#[test]
fn identical_arc_short_circuits_to_an_empty_patch() {
    let m = empty_model();
    let root = m.root();
    let patch = diff(&root, &root, 7);
    assert_eq!(patch.revision, 7);
    assert!(patch.upsert.media.is_empty());
    assert!(patch.upsert.bins.is_empty());
    assert!(patch.upsert.sequences.is_empty());
    assert!(patch.upsert.tracks.is_empty());
    assert!(patch.upsert.clips.is_empty());
    assert!(patch.upsert.markers.is_empty());
    assert!(patch.remove.media.is_empty());
    assert!(patch.remove.bins.is_empty());
    assert!(patch.remove.sequences.is_empty());
    assert!(patch.dirty_ranges.is_empty());
}

#[test]
fn bin_lifecycle_diffs_upsert_and_remove_with_reparenting() {
    let mut m = empty_model();
    let mut before = m.root();
    ok(
        &mut m,
        &EditOp::CreateBin {
            name: "P".into(),
            parent_id: None,
        },
    );
    let after_first = m.root();
    let patch = diff(&before, &after_first, 1);
    assert_eq!(patch.upsert.bins.len(), 1);
    assert!(patch.dirty_ranges.is_empty());

    before = after_first;
    let parent_id = m.root().bins[0].id.clone();
    ok(
        &mut m,
        &EditOp::CreateBin {
            name: "C".into(),
            parent_id: Some(parent_id.clone()),
        },
    );
    let after_second = m.root();
    let patch = diff(&before, &after_second, 2);
    assert_eq!(patch.upsert.bins.len(), 1);
    assert_eq!(patch.upsert.bins[0].name, "C");

    // Deleting P re-parents C to none: C is upserted, P removed.
    before = after_second;
    ok(&mut m, &EditOp::DeleteBin { bin_id: parent_id });
    let patch = diff(&before, &m.root(), 3);
    assert!(patch.remove.bins.len() == 1);
    assert!(
        patch
            .upsert
            .bins
            .iter()
            .any(|b| b.name == "C" && b.parent_id.is_none())
    );
}

#[test]
fn media_change_upserts_media_in_the_patch() {
    let mut m = empty_model();
    ok(
        &mut m,
        &EditOp::AddMedia {
            items: vec![media_item(id::new_id())],
            bin_id: None,
        },
    );
    let media_id = m.root().media[0].id.clone();
    let before = m.root();
    ok(
        &mut m,
        &EditOp::UpdateMedia {
            media_id: media_id.clone(),
            changes: alvs_model::edit::MediaChanges {
                name: Some("renamed.mov".into()),
                ..Default::default()
            },
        },
    );
    let patch = diff(&before, &m.root(), 2);
    assert_eq!(patch.upsert.media.len(), 1);
    assert_eq!(patch.upsert.media[0].id, media_id);
    assert_eq!(patch.upsert.media[0].name, "renamed.mov");
    assert!(patch.remove.media.is_empty());
    assert!(patch.dirty_ranges.is_empty());
}

#[test]
fn remove_media_with_clips_removes_entities_and_marks_the_sequence_dirty() {
    let mut project = hand_project();
    let media = media_item(id::new_id());
    let media_id = media.id.clone();
    project.media.push(media);
    project.sequences[0].tracks[0].items[0].clip_type = ClipType::Media;
    project.sequences[0].tracks[0].items[0].media_id = Some(media_id.clone());
    project.sequences[0].tracks[0].items[0].stream_index = Some(0);
    let mut m = Model::new(project);
    let sequence_id = m.root().sequences[0].id.clone();
    let clip_id = m.root().sequences[0].tracks[0].items[0].id.clone();
    let before = m.root();
    ok(
        &mut m,
        &EditOp::RemoveMedia {
            media_ids: vec![media_id],
            also_remove_clips: true,
        },
    );
    let patch = diff(&before, &m.root(), 4);
    assert_eq!(patch.remove.media.len(), 1);
    assert!(patch.remove.clips.contains(&clip_id));
    assert!(
        patch
            .dirty_ranges
            .iter()
            .any(|r| r.sequence_id == sequence_id && r.start == 0)
    );
}

#[test]
fn fps_change_upserts_sequence_clips_and_a_full_extent_range() {
    let mut m = Model::new(hand_project());
    let sequence_id = m.root().sequences[0].id.clone();
    let clip_id = m.root().sequences[0].tracks[0].items[0].id.clone();
    let before = m.root();
    ok(
        &mut m,
        &EditOp::UpdateSequence {
            sequence_id: sequence_id.clone(),
            changes: alvs_model::edit::SequenceChanges {
                format: Some(fps25()),
                ..Default::default()
            },
        },
    );
    let patch = diff(&before, &m.root(), 5);
    assert!(patch.upsert.sequences.iter().any(|s| s.id == sequence_id));
    assert!(patch.upsert.clips.iter().any(|c| c.id == clip_id));
    let end = 2 * 28_224_000; // second clip end snapped to the 25 fps grid
    assert!(
        patch
            .dirty_ranges
            .iter()
            .any(|r| r.sequence_id == sequence_id && r.start == 0 && r.end == end)
    );
}

#[test]
fn track_add_remove_diffs_upsert_remove_and_clips() {
    let mut m = empty_model();
    let sequence_id = m.root().sequences[0].id.clone();
    let mut before = m.root();
    ok(
        &mut m,
        &EditOp::AddTrack {
            sequence_id: sequence_id.clone(),
            kind: TrackKind::Video,
            index: None,
            name: Some("V9".into()),
        },
    );
    let patch = diff(&before, &m.root(), 1);
    assert!(patch.upsert.tracks.iter().any(|t| t.name == "V9"));

    before = m.root();
    let track_id = m.root().sequences[0].tracks[0].id.clone();
    ok(
        &mut m,
        &EditOp::RemoveTrack {
            sequence_id: sequence_id.clone(),
            track_id: track_id.clone(),
        },
    );
    let patch = diff(&before, &m.root(), 2);
    assert!(patch.remove.tracks.contains(&track_id));
    assert!(
        patch
            .dirty_ranges
            .iter()
            .any(|r| r.sequence_id == sequence_id && r.start == 0)
    );
}

#[test]
fn marker_add_and_remove_yield_point_ranges() {
    let mut m = empty_model();
    let sequence_id = m.root().sequences[0].id.clone();
    let t = 123_456;
    let mut before = m.root();
    ok(
        &mut m,
        &EditOp::AddMarkers {
            sequence_id: sequence_id.clone(),
            markers: vec![alvs_model::edit::MarkerInput {
                t,
                duration: 0,
                name: "point".into(),
                color: "CLR-03".into(),
                note: String::new(),
                chapter: false,
            }],
        },
    );
    let patch = diff(&before, &m.root(), 1);
    assert_eq!(patch.upsert.markers.len(), 1);
    assert!(
        patch
            .dirty_ranges
            .iter()
            .any(|r| r.sequence_id == sequence_id && r.start == t && r.end == t)
    );

    before = m.root();
    let marker_id = m.root().sequences[0].markers[0].id.clone();
    ok(
        &mut m,
        &EditOp::RemoveMarkers {
            sequence_id: sequence_id.clone(),
            marker_ids: vec![marker_id],
        },
    );
    let patch = diff(&before, &m.root(), 2);
    assert_eq!(patch.remove.markers.len(), 1);
    assert!(
        patch
            .dirty_ranges
            .iter()
            .any(|r| r.sequence_id == sequence_id && r.start == t && r.end == t)
    );
}

#[test]
fn caption_track_diffs_cover_upsert_update_and_remove() {
    let mut m = empty_model();
    let sequence_id = m.root().sequences[0].id.clone();
    let mut before = m.root();
    ok(
        &mut m,
        &EditOp::AddCaptionTrack {
            sequence_id: sequence_id.clone(),
            name: "CC".into(),
            language: "en".into(),
        },
    );
    let patch = diff(&before, &m.root(), 1);
    assert_eq!(patch.upsert.caption_tracks.len(), 1);

    before = m.root();
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
    let patch = diff(&before, &m.root(), 2);
    assert!(patch.upsert.caption_tracks.iter().any(|t| t.name == "CC2"));

    before = m.root();
    ok(
        &mut m,
        &EditOp::RemoveCaptionTrack {
            sequence_id: sequence_id.clone(),
            caption_track_id,
        },
    );
    let patch = diff(&before, &m.root(), 3);
    assert_eq!(patch.remove.caption_tracks.len(), 1);
    assert!(
        patch
            .dirty_ranges
            .iter()
            .any(|r| r.sequence_id == sequence_id && r.start == 0)
    );
}

#[test]
fn asset_diff_covers_upsert_change_and_remove() {
    let old = Arc::new(hand_project());
    let mut new = (*old).clone();
    let a1 = alvs_model::asset::Asset {
        id: id::new_id(),
        kind: "lut".into(),
        name: "one".into(),
        rel_path: "one.cube".into(),
        sha256: "aa".into(),
        lut_info: None,
        extra: Default::default(),
    };
    let mut a2 = alvs_model::asset::Asset {
        name: "two".into(),
        ..a1.clone()
    };
    a2.id = id::new_id();
    let mut a3 = alvs_model::asset::Asset {
        name: "three".into(),
        ..a1.clone()
    };
    a3.id = id::new_id();
    let mut old = (*old).clone();
    old.assets = vec![a1.clone(), a3.clone()];
    new.assets = vec![
        alvs_model::asset::Asset {
            name: "changed".into(),
            ..a1
        },
        a2.clone(),
    ];
    let old = Arc::new(old);
    let new = Arc::new(new);
    let patch = diff(&old, &new, 9);
    assert_eq!(patch.upsert.assets.len(), 2);
    assert!(patch.upsert.assets.iter().any(|a| a.name == "changed"));
    assert!(patch.upsert.assets.iter().any(|a| a.id == a2.id));
    assert_eq!(patch.remove.assets, vec![a3.id]);
}

#[test]
fn transition_upsert_and_clip_remove_diff_directly() {
    let old = Arc::new(hand_project());
    let mut new = (*old).clone();
    let c1 = new.sequences[0].tracks[0].items[0].id.clone();
    let c2 = new.sequences[0].tracks[0].items[1].id.clone();
    let transition = alvs_model::transition::new_transition("trs.crossDissolve", &c1, &c2, FPF_30);
    new.sequences[0].tracks[0].transitions.push(transition);
    new.sequences[0].tracks[0].items.retain(|c| c.id != c2);
    let new = Arc::new(new);
    let patch = diff(&old, &new, 11);
    assert_eq!(patch.upsert.transitions.len(), 1);
    assert!(patch.remove.clips.contains(&c2));
    assert!(patch.dirty_ranges.iter().any(|r| r.start == 0 && r.end > 0));
}

#[test]
fn rich_patch_application_equivalence() {
    // US4/AC4: applying a multi-kind EntityPatch to a normalized copy of the
    // prior state reproduces the new root's entities exactly. One batch op
    // produces upserts and removes across many kinds at once.
    use std::collections::BTreeMap;

    let mut project = hand_project();
    let media = media_item(id::new_id());
    project.sequences[0].tracks[0].items[0].clip_type = ClipType::Media;
    project.sequences[0].tracks[0].items[0].media_id = Some(media.id.clone());
    project.sequences[0].tracks[0].items[0].stream_index = Some(0);
    project.media.push(media);
    let mut m = Model::new(project);
    let sequence_id = m.root().sequences[0].id.clone();
    let old_track_id = m.root().sequences[0].tracks[0].id.clone();
    let before = m.root();

    ok(
        &mut m,
        &EditOp::Batch {
            ops: vec![
                EditOp::CreateBin {
                    name: "Kept".into(),
                    parent_id: None,
                },
                EditOp::AddMedia {
                    items: vec![media_item(id::new_id())],
                    bin_id: None,
                },
                EditOp::AddTrack {
                    sequence_id: sequence_id.clone(),
                    kind: TrackKind::Video,
                    index: None,
                    name: Some("V9".into()),
                },
                EditOp::AddMarkers {
                    sequence_id: sequence_id.clone(),
                    markers: vec![alvs_model::edit::MarkerInput {
                        t: 100,
                        duration: 0,
                        name: "point".into(),
                        color: "CLR-03".into(),
                        note: String::new(),
                        chapter: false,
                    }],
                },
                EditOp::AddCaptionTrack {
                    sequence_id: sequence_id.clone(),
                    name: "CC".into(),
                    language: "en".into(),
                },
                EditOp::RemoveTrack {
                    sequence_id: sequence_id.clone(),
                    track_id: old_track_id,
                },
            ],
            label_key: "history.batch.equivalence".into(),
        },
    );

    let after = m.root();
    let patch = diff(&before, &after, m.revision());
    // The batch touched many kinds at once.
    assert!(patch.upsert.bins.len() == 1);
    assert!(patch.upsert.media.len() == 1);
    assert!(patch.upsert.tracks.iter().any(|t| t.name == "V9"));
    assert!(patch.upsert.markers.len() == 1);
    assert!(patch.upsert.caption_tracks.len() == 1);
    assert!(patch.remove.tracks.len() == 1);
    assert!(patch.remove.clips.len() == 2);

    // Normalized copy of the prior state: entity maps keyed by id.
    fn keyed<T, F>(items: &[T], id_of: F) -> BTreeMap<String, T>
    where
        T: Clone,
        F: Fn(&T) -> &str,
    {
        items
            .iter()
            .map(|x| (id_of(x).to_string(), x.clone()))
            .collect()
    }
    let mut media_map = keyed(&before.media, |x: &alvs_model::media::MediaItem| {
        x.id.as_str()
    });
    let mut bin_map = keyed(&before.bins, |x: &alvs_model::bin::Bin| x.id.as_str());
    let mut asset_map = keyed(&before.assets, |x: &alvs_model::asset::Asset| x.id.as_str());
    let mut sequence_map = keyed(&before.sequences, |x: &alvs_model::sequence::Sequence| {
        x.id.as_str()
    });
    let mut track_map: BTreeMap<String, alvs_model::track::Track> = BTreeMap::new();
    let mut clip_map: BTreeMap<String, Clip> = BTreeMap::new();
    let mut transition_map: BTreeMap<String, alvs_model::transition::Transition> = BTreeMap::new();
    let mut marker_map: BTreeMap<String, alvs_model::marker::Marker> = BTreeMap::new();
    let mut caption_track_map: BTreeMap<String, alvs_model::caption::CaptionTrack> =
        BTreeMap::new();
    let mut caption_map: BTreeMap<String, alvs_model::caption::CaptionItem> = BTreeMap::new();
    for s in &before.sequences {
        for t in &s.tracks {
            track_map.insert(t.id.clone(), t.clone());
            for c in &t.items {
                clip_map.insert(c.id.clone(), c.clone());
            }
            for tr in &t.transitions {
                transition_map.insert(tr.id.clone(), tr.clone());
            }
        }
        for mk in &s.markers {
            marker_map.insert(mk.id.clone(), mk.clone());
        }
        for ct in &s.caption_tracks {
            caption_track_map.insert(ct.id.clone(), ct.clone());
            for item in &ct.items {
                caption_map.insert(item.id.clone(), item.clone());
            }
        }
    }

    // Apply the patch to the normalized copy.
    for x in &patch.upsert.media {
        media_map.insert(x.id.clone(), x.clone());
    }
    for x in &patch.remove.media {
        media_map.remove(x);
    }
    for x in &patch.upsert.bins {
        bin_map.insert(x.id.clone(), x.clone());
    }
    for x in &patch.remove.bins {
        bin_map.remove(x);
    }
    for x in &patch.upsert.assets {
        asset_map.insert(x.id.clone(), x.clone());
    }
    for x in &patch.remove.assets {
        asset_map.remove(x);
    }
    for x in &patch.upsert.sequences {
        sequence_map.insert(x.id.clone(), x.clone());
    }
    for x in &patch.remove.sequences {
        sequence_map.remove(x);
    }
    for x in &patch.upsert.tracks {
        track_map.insert(x.id.clone(), x.clone());
    }
    for x in &patch.remove.tracks {
        track_map.remove(x);
    }
    for x in &patch.upsert.clips {
        clip_map.insert(x.id.clone(), x.clone());
    }
    for x in &patch.remove.clips {
        clip_map.remove(x);
    }
    for x in &patch.upsert.transitions {
        transition_map.insert(x.id.clone(), x.clone());
    }
    for x in &patch.remove.transitions {
        transition_map.remove(x);
    }
    for x in &patch.upsert.markers {
        marker_map.insert(x.id.clone(), x.clone());
    }
    for x in &patch.remove.markers {
        marker_map.remove(x);
    }
    for x in &patch.upsert.caption_tracks {
        caption_track_map.insert(x.id.clone(), x.clone());
    }
    for x in &patch.remove.caption_tracks {
        caption_track_map.remove(x);
    }
    for x in &patch.upsert.captions {
        caption_map.insert(x.id.clone(), x.clone());
    }
    for x in &patch.remove.captions {
        caption_map.remove(x);
    }

    // The normalized copy now equals the new root, kind by kind.
    assert_eq!(
        media_map,
        keyed(&after.media, |x: &alvs_model::media::MediaItem| x
            .id
            .as_str())
    );
    assert_eq!(
        bin_map,
        keyed(&after.bins, |x: &alvs_model::bin::Bin| x.id.as_str())
    );
    assert_eq!(
        asset_map,
        keyed(&after.assets, |x: &alvs_model::asset::Asset| x.id.as_str())
    );
    assert_eq!(
        sequence_map,
        keyed(&after.sequences, |x: &alvs_model::sequence::Sequence| x
            .id
            .as_str())
    );
    let new_tracks: BTreeMap<String, alvs_model::track::Track> = after
        .sequences
        .iter()
        .flat_map(|s| s.tracks.iter().cloned())
        .map(|t| (t.id.clone(), t))
        .collect();
    assert_eq!(track_map, new_tracks);
    let new_clips: BTreeMap<String, Clip> = after
        .sequences
        .iter()
        .flat_map(|s| s.tracks.iter().flat_map(|t| t.items.iter().cloned()))
        .map(|c| (c.id.clone(), c))
        .collect();
    assert_eq!(clip_map, new_clips);
    let new_transitions: BTreeMap<String, alvs_model::transition::Transition> = after
        .sequences
        .iter()
        .flat_map(|s| s.tracks.iter().flat_map(|t| t.transitions.iter().cloned()))
        .map(|t| (t.id.clone(), t))
        .collect();
    assert_eq!(transition_map, new_transitions);
    let new_markers: BTreeMap<String, alvs_model::marker::Marker> = after
        .sequences
        .iter()
        .flat_map(|s| s.markers.iter().cloned())
        .map(|x| (x.id.clone(), x))
        .collect();
    assert_eq!(marker_map, new_markers);
    let new_caption_tracks: BTreeMap<String, alvs_model::caption::CaptionTrack> = after
        .sequences
        .iter()
        .flat_map(|s| s.caption_tracks.iter().cloned())
        .map(|x| (x.id.clone(), x))
        .collect();
    assert_eq!(caption_track_map, new_caption_tracks);
    let new_captions: BTreeMap<String, alvs_model::caption::CaptionItem> = after
        .sequences
        .iter()
        .flat_map(|s| {
            s.caption_tracks
                .iter()
                .flat_map(|t| t.items.iter().cloned())
        })
        .map(|x| (x.id.clone(), x))
        .collect();
    assert_eq!(caption_map, new_captions);
}
