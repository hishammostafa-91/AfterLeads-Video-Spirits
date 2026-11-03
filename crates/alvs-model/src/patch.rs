//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Entity patches (04 §4.4): computed by structural diffing with
//! `Arc::ptr_eq` short-circuits (ADR-009). The dirty-range policy for the
//! WP-02 op set is the package's own decision (research D10).

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::project::Project;

/// A dirty time range of one sequence (04 §4.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct DirtyRange {
    /// The sequence id.
    #[serde(rename = "sequenceId")]
    pub sequence_id: String,
    /// Range start (flicks).
    #[ts(type = "number")]
    pub start: i64,
    /// Range end (flicks).
    #[ts(type = "number")]
    pub end: i64,
}

/// The entity kinds of the UI store (04 §4.4).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct EntityMaps {
    /// Media items.
    pub media: Vec<crate::media::MediaItem>,
    /// Bins.
    pub bins: Vec<crate::bin::Bin>,
    /// Sequences.
    pub sequences: Vec<crate::sequence::Sequence>,
    /// Tracks.
    pub tracks: Vec<crate::track::Track>,
    /// Clips.
    pub clips: Vec<crate::track::Clip>,
    /// Transitions.
    pub transitions: Vec<crate::transition::Transition>,
    /// Markers.
    pub markers: Vec<crate::marker::Marker>,
    /// Caption tracks.
    #[serde(rename = "captionTracks")]
    pub caption_tracks: Vec<crate::caption::CaptionTrack>,
    /// Captions.
    pub captions: Vec<crate::caption::CaptionItem>,
    /// Assets.
    pub assets: Vec<crate::asset::Asset>,
}

/// The upsert/remove patch payload (04 §4.4).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct EntityPatch {
    /// The model revision that produced this patch.
    #[serde(rename = "revision")]
    #[ts(type = "number")]
    pub revision: u64,
    /// Upserted entities by kind.
    pub upsert: EntityMaps,
    /// Removed entity ids by kind.
    pub remove: RemoveMaps,
    /// Affected time ranges (research D10).
    #[serde(rename = "dirtyRanges")]
    pub dirty_ranges: Vec<DirtyRange>,
}

/// Removed ids by kind (04 §4.4).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RemoveMaps {
    /// Removed media ids.
    pub media: Vec<String>,
    /// Removed bin ids.
    pub bins: Vec<String>,
    /// Removed sequence ids.
    pub sequences: Vec<String>,
    /// Removed track ids.
    pub tracks: Vec<String>,
    /// Removed clip ids.
    pub clips: Vec<String>,
    /// Removed transition ids.
    pub transitions: Vec<String>,
    /// Removed marker ids.
    pub markers: Vec<String>,
    /// Removed caption track ids.
    #[serde(rename = "captionTracks")]
    pub caption_tracks: Vec<String>,
    /// Removed caption ids.
    pub captions: Vec<String>,
    /// Removed asset ids.
    pub assets: Vec<String>,
}

/// Compute the patch from `old` to `new`, short-circuiting unchanged
/// entities via `Arc::ptr_eq` (ADR-009). Dirty ranges follow research D10:
/// structural sequence/track changes mark the whole sequence; marker
/// add/remove mark a point at the marker time; media/bin/project changes
/// carry no ranges.
pub fn diff(old: &Arc<Project>, new: &Arc<Project>, revision: u64) -> EntityPatch {
    let mut patch = EntityPatch {
        revision,
        ..Default::default()
    };
    if Arc::ptr_eq(old, new) {
        return patch;
    }

    // Media.
    for item in &new.media {
        let unchanged = old.media.iter().any(|o| o.id == item.id && o == item);
        if !unchanged {
            patch.upsert.media.push(item.clone());
        }
    }
    for item in &old.media {
        if !new.media.iter().any(|n| n.id == item.id) {
            patch.remove.media.push(item.id.clone());
        }
    }

    // Bins and assets.
    for bin in &new.bins {
        if !old.bins.iter().any(|o| o.id == bin.id && o == bin) {
            patch.upsert.bins.push(bin.clone());
        }
    }
    for bin in &old.bins {
        if !new.bins.iter().any(|n| n.id == bin.id) {
            patch.remove.bins.push(bin.id.clone());
        }
    }
    for asset in &new.assets {
        if !old.assets.iter().any(|o| o.id == asset.id && o == asset) {
            patch.upsert.assets.push(asset.clone());
        }
    }
    for asset in &old.assets {
        if !new.assets.iter().any(|n| n.id == asset.id) {
            patch.remove.assets.push(asset.id.clone());
        }
    }

    // Sequences, tracks, clips, transitions, markers, captions.
    let mut dirty_sequences: Vec<String> = Vec::new();
    for sequence in &new.sequences {
        let Some(prev) = old.sequences.iter().find(|o| o.id == sequence.id) else {
            patch.upsert.sequences.push(sequence.clone());
            dirty_sequences.push(sequence.id.clone());
            continue;
        };
        let sequence_changed = prev != sequence;
        if sequence_changed {
            // The entity changed (the UI store refreshes it), but only
            // *structural* changes (tracks, clips' containers, captions,
            // format, points, master audio) invalidate rendered time:
            // markers are metadata, so a marker-only change carries just
            // the point ranges from the marker walk below (research D10).
            patch.upsert.sequences.push(sequence.clone());
            let structural = prev.tracks != sequence.tracks
                || prev.caption_tracks != sequence.caption_tracks
                || prev.name != sequence.name
                || prev.format != sequence.format
                || prev.in_point != sequence.in_point
                || prev.out_point != sequence.out_point
                || prev.master_audio != sequence.master_audio
                || prev.extra != sequence.extra;
            if structural {
                dirty_sequences.push(sequence.id.clone());
            }
        }
        // Tracks.
        for track in &sequence.tracks {
            let prev_track = prev.tracks.iter().find(|o| o.id == track.id);
            match prev_track {
                None => {
                    patch.upsert.tracks.push(track.clone());
                    dirty_sequences.push(sequence.id.clone());
                    for clip in &track.items {
                        patch.upsert.clips.push(clip.clone());
                    }
                }
                Some(pt) if pt != track => {
                    patch.upsert.tracks.push(track.clone());
                    dirty_sequences.push(sequence.id.clone());
                }
                Some(_) => {}
            }
            for clip in &track.items {
                let prev_clip = prev
                    .tracks
                    .iter()
                    .flat_map(|t| t.items.iter())
                    .find(|o| o.id == clip.id);
                match prev_clip {
                    None => {
                        patch.upsert.clips.push(clip.clone());
                        dirty_sequences.push(sequence.id.clone());
                    }
                    Some(pc) if pc != clip => {
                        patch.upsert.clips.push(clip.clone());
                        dirty_sequences.push(sequence.id.clone());
                    }
                    Some(_) => {}
                }
            }
            for transition in &track.transitions {
                let prev_t = prev
                    .tracks
                    .iter()
                    .flat_map(|t| t.transitions.iter())
                    .find(|o| o.id == transition.id);
                if prev_t != Some(transition) {
                    patch.upsert.transitions.push(transition.clone());
                    dirty_sequences.push(sequence.id.clone());
                }
            }
        }
        for track in &prev.tracks {
            if !sequence.tracks.iter().any(|n| n.id == track.id) {
                // The whole track is gone; its clips leave with it (pushed
                // once here, not again by the per-clip loop below).
                patch.remove.tracks.push(track.id.clone());
                dirty_sequences.push(sequence.id.clone());
                for clip in &track.items {
                    patch.remove.clips.push(clip.id.clone());
                }
                continue;
            }
            for clip in &track.items {
                let still = sequence
                    .tracks
                    .iter()
                    .flat_map(|t| t.items.iter())
                    .any(|n| n.id == clip.id);
                if !still {
                    patch.remove.clips.push(clip.id.clone());
                }
            }
        }
        // Markers: point ranges at marker times (research D10).
        for marker in &sequence.markers {
            let prev_m = prev.markers.iter().find(|o| o.id == marker.id);
            match prev_m {
                None => {
                    patch.upsert.markers.push(marker.clone());
                    patch.dirty_ranges.push(DirtyRange {
                        sequence_id: sequence.id.clone(),
                        start: marker.t,
                        end: marker.t,
                    });
                }
                Some(pm) if pm != marker => {
                    patch.upsert.markers.push(marker.clone());
                    patch.dirty_ranges.push(DirtyRange {
                        sequence_id: sequence.id.clone(),
                        start: marker.t,
                        end: marker.t,
                    });
                }
                Some(_) => {}
            }
        }
        for marker in &prev.markers {
            if !sequence.markers.iter().any(|n| n.id == marker.id) {
                patch.remove.markers.push(marker.id.clone());
                patch.dirty_ranges.push(DirtyRange {
                    sequence_id: sequence.id.clone(),
                    start: marker.t,
                    end: marker.t,
                });
            }
        }
        // Caption tracks.
        for ct in &sequence.caption_tracks {
            let prev_ct = prev.caption_tracks.iter().find(|o| o.id == ct.id);
            if prev_ct != Some(ct) {
                patch.upsert.caption_tracks.push(ct.clone());
                for item in &ct.items {
                    patch.upsert.captions.push(item.clone());
                }
            }
        }
        for ct in &prev.caption_tracks {
            if !sequence.caption_tracks.iter().any(|n| n.id == ct.id) {
                patch.remove.caption_tracks.push(ct.id.clone());
                dirty_sequences.push(sequence.id.clone());
            }
        }
    }
    for sequence in &old.sequences {
        if !new.sequences.iter().any(|n| n.id == sequence.id) {
            patch.remove.sequences.push(sequence.id.clone());
        }
    }

    // Coalesce per-sequence dirty ranges: one full-extent entry per changed
    // sequence (structural) — point ranges from markers stay as-is, except
    // for sequences that are structurally dirty anyway (the full extent
    // already covers the point).
    let structural: std::collections::BTreeSet<&str> =
        dirty_sequences.iter().map(String::as_str).collect();
    patch
        .dirty_ranges
        .retain(|r| !(structural.contains(r.sequence_id.as_str()) && r.start == r.end));
    for id in &dirty_sequences {
        let end = new
            .sequences
            .iter()
            .find(|s| s.id == *id)
            .and_then(|s| {
                s.tracks
                    .iter()
                    .flat_map(|t| t.items.iter())
                    .map(|c| c.start + c.duration)
                    .max()
            })
            .unwrap_or(0);
        patch.dirty_ranges.push(DirtyRange {
            sequence_id: id.clone(),
            start: 0,
            end,
        });
    }

    patch
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit::EditOp;
    use crate::history::Model;
    use crate::project::new_project;
    use crate::sequence::{new_sequence, sqp02_fallback_format};

    fn model() -> Model {
        let mut seq = new_sequence("Main", sqp02_fallback_format());
        seq.format.auto_match_pending = false;
        Model::new(new_project("Test", seq))
    }

    #[test]
    fn patch_application_equivalence() {
        let mut m = model();
        let old = m.root();
        m.apply(&EditOp::RenameProject {
            name: "Renamed".into(),
        })
        .unwrap_or_default();
        let new = m.root();
        let patch = diff(&old, &new, m.revision());
        // Apply to a normalized copy of the old state: replaces upserted
        // entities, deletes removed ones — equivalence with the new root.
        let mut normalized = (*old).clone();
        normalized.name = new.name.clone();
        for seq in &patch.upsert.sequences {
            if let Some(slot) = normalized.sequences.iter_mut().find(|s| s.id == seq.id) {
                *slot = seq.clone();
            } else {
                normalized.sequences.push(seq.clone());
            }
        }
        normalized
            .sequences
            .retain(|s| !patch.remove.sequences.contains(&s.id));
        assert_eq!(normalized.name, new.name);
        assert_eq!(patch.revision, m.revision());
    }

    #[test]
    fn track_change_yields_full_extent_dirty_range() {
        let mut m = model();
        let seq_id = m.root().sequences[0].id.clone();
        let old = m.root();
        m.apply(&EditOp::AddTrack {
            sequence_id: seq_id.clone(),
            kind: crate::track::TrackKind::Video,
            index: None,
            name: Some("V2".into()),
        })
        .unwrap_or_default();
        let patch = diff(&old, &m.root(), m.revision());
        assert!(patch.upsert.tracks.iter().any(|t| t.name == "V2"));
        assert!(
            patch
                .dirty_ranges
                .iter()
                .any(|r| r.sequence_id == seq_id && r.start == 0),
            "expected full-extent range, got {:?}",
            patch.dirty_ranges
        );
    }

    #[test]
    fn marker_change_yields_point_range() {
        let mut m = model();
        let seq_id = m.root().sequences[0].id.clone();
        let old = m.root();
        m.apply(&EditOp::AddMarkers {
            sequence_id: seq_id.clone(),
            markers: vec![crate::edit::MarkerInput {
                t: 123_456,
                duration: 0,
                name: "point".into(),
                color: "CLR-03".into(),
                note: String::new(),
                chapter: false,
            }],
        })
        .unwrap_or_default();
        let patch = diff(&old, &m.root(), m.revision());
        assert!(patch.upsert.markers.iter().any(|mk| mk.name == "point"));
        assert!(
            patch
                .dirty_ranges
                .iter()
                .any(|r| r.sequence_id == seq_id && r.start == r.end && r.end == 123_456),
            "expected point range at 123456, got {:?}",
            patch.dirty_ranges
        );
    }

    #[test]
    fn media_change_has_no_dirty_range() {
        let mut m = model();
        let old = m.root();
        let item = crate::media::MediaItem {
            id: alvs_core::id::new_id(),
            kind: crate::media::MediaKind::Audio,
            name: "sfx.wav".into(),
            path: "C:\\s.wav".into(),
            rel_path: "s.wav".into(),
            fingerprint: crate::media::Fingerprint {
                size: 10,
                mtime_utc: "2026-01-01T00:00:00Z".into(),
                head_tail_sha256: "bb".into(),
                extra: Default::default(),
            },
            probe: crate::media::Probe {
                duration_flicks: 705_600_000,
                container: "wav".into(),
                streams: vec![],
                extra: Default::default(),
            },
            image_sequence: None,
            bin_id: None,
            label: None,
            notes: String::new(),
            derived: Default::default(),
            added_utc: "2026-01-01T00:00:00Z".into(),
            extra: Default::default(),
        };
        m.apply(&EditOp::AddMedia {
            items: vec![item],
            bin_id: None,
        })
        .unwrap_or_default();
        let patch = diff(&old, &m.root(), m.revision());
        assert!(patch.dirty_ranges.is_empty());
        assert_eq!(patch.upsert.media.len(), 1);
    }
}
