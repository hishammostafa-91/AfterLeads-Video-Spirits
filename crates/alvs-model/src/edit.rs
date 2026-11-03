//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! `EditOp` — the tagged enum of 04 §4.4 and its application for the WP-02
//! categories (project, sequence, track, media, bin, marker, caption-track,
//! and `batch`). Clip-editing semantics arrive with WP-12 (card "Not
//! included").

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::caption::new_caption_track;
use crate::marker::Marker;
use crate::media::MediaItem;
use crate::project::Project;
use crate::sequence::{Sequence, SequenceFormat, new_sequence, sqp02_fallback_format};
use crate::track::TrackKind;

/// Field changes for `updateMedia` (04 §4.4).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaChanges {
    /// New name.
    pub name: Option<String>,
    /// New label.
    pub label: Option<String>,
    /// New notes.
    pub notes: Option<String>,
    /// New owning bin.
    pub bin_id: Option<String>,
}

/// Field changes for `updateTrack` (04 §4.4).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackChanges {
    /// New name.
    pub name: Option<String>,
    /// New height (32–240).
    pub height: Option<u32>,
    /// Locked.
    pub locked: Option<bool>,
    /// Visible.
    pub visible: Option<bool>,
    /// Muted.
    pub muted: Option<bool>,
    /// Solo.
    pub solo: Option<bool>,
    /// Volume dB.
    pub volume_db: Option<f64>,
    /// Pan.
    pub pan: Option<f64>,
    /// Targeted.
    pub targeted: Option<bool>,
}

/// Field changes for `updateSequence` (04 §4.4).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SequenceChanges {
    /// New name.
    pub name: Option<String>,
    /// New format (frame-rate changes snap items, E1104).
    pub format: Option<SequenceFormat>,
    /// In point.
    pub in_point: Option<Option<i64>>,
    /// Out point.
    pub out_point: Option<Option<i64>>,
}

/// A marker payload for `addMarkers` (04 §4.4).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerInput {
    /// Position (flicks).
    pub t: i64,
    /// Duration (flicks).
    pub duration: i64,
    /// Name.
    pub name: String,
    /// Color.
    pub color: String,
    /// Note.
    pub note: String,
    /// Chapter or standard.
    pub chapter: bool,
}

/// The `EditOp` enum (04 §4.4): `#[serde(tag = "op")]`, only the WP-02
/// categories (card Deliverables; FR-009). Every variant names its History
/// panel key via [`EditOp::history_key`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum EditOp {
    /// Adds probed media (`addMedia`).
    AddMedia {
        /// Items to add.
        items: Vec<MediaItem>,
        /// Optional owning bin.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bin_id: Option<String>,
    },
    /// Removes media (`removeMedia`); fails E1102 if clips use it unless
    /// `alsoRemoveClips` is set.
    RemoveMedia {
        /// Ids to remove.
        media_ids: Vec<String>,
        /// Also remove dependent clips.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        also_remove_clips: bool,
    },
    /// Updates media fields (`updateMedia`).
    UpdateMedia {
        /// Target id.
        media_id: String,
        /// Field changes.
        changes: MediaChanges,
    },
    /// Creates a bin (`createBin`).
    CreateBin {
        /// Bin name.
        name: String,
        /// Parent bin.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parent_id: Option<String>,
    },
    /// Updates a bin (`updateBin`).
    UpdateBin {
        /// Target id.
        bin_id: String,
        /// New name.
        name: String,
    },
    /// Deletes a bin (`deleteBin`); contents move to the parent bin.
    DeleteBin {
        /// Target id.
        bin_id: String,
    },
    /// Creates a sequence (`createSequence`).
    CreateSequence {
        /// Sequence name.
        name: String,
        /// Explicit format (overrides the preset).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        format: Option<SequenceFormat>,
        /// Preset id (12 §12.8); SQP-01 falls back to SQP-02 at 30 fps with
        /// `autoMatchPending` (REQ-PRJ-001).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        from_preset: Option<String>,
    },
    /// Duplicates a sequence (`duplicateSequence`).
    DuplicateSequence {
        /// Target id.
        sequence_id: String,
        /// New name.
        name: String,
    },
    /// Deletes a sequence (`deleteSequence`); the last one cannot be deleted
    /// (REQ-PRJ-005).
    DeleteSequence {
        /// Target id.
        sequence_id: String,
    },
    /// Updates a sequence (`updateSequence`); a frame-rate change snaps clip
    /// times and reports adjusted items (E1104).
    UpdateSequence {
        /// Target id.
        sequence_id: String,
        /// Field changes.
        changes: SequenceChanges,
    },
    /// Adds a track (`addTrack`).
    AddTrack {
        /// Owning sequence.
        sequence_id: String,
        /// Track kind.
        kind: TrackKind,
        /// Insert index (defaults to end).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        index: Option<usize>,
        /// Track name.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    /// Removes a track (`removeTrack`).
    RemoveTrack {
        /// Owning sequence.
        sequence_id: String,
        /// Target id.
        track_id: String,
    },
    /// Moves a track (`moveTrack`).
    MoveTrack {
        /// Owning sequence.
        sequence_id: String,
        /// Target id.
        track_id: String,
        /// New index.
        index: usize,
    },
    /// Updates track fields (`updateTrack`).
    UpdateTrack {
        /// Owning sequence.
        sequence_id: String,
        /// Target id.
        track_id: String,
        /// Field changes.
        changes: TrackChanges,
    },
    /// Adds markers (`addMarkers`).
    AddMarkers {
        /// Owning sequence (WP-02 scope; clip markers via `owner` arrive with
        /// the clip-editing package).
        sequence_id: String,
        /// Marker payloads.
        markers: Vec<MarkerInput>,
    },
    /// Updates a marker (`updateMarker`).
    UpdateMarker {
        /// Owning sequence.
        sequence_id: String,
        /// Target id.
        marker_id: String,
        /// New name.
        name: String,
        /// New note.
        #[serde(default)]
        note: String,
    },
    /// Removes markers (`removeMarkers`).
    RemoveMarkers {
        /// Owning sequence.
        sequence_id: String,
        /// Target ids.
        marker_ids: Vec<String>,
    },
    /// Adds a caption track (`addCaptionTrack`).
    AddCaptionTrack {
        /// Owning sequence.
        sequence_id: String,
        /// Track name.
        name: String,
        /// BCP-47 language.
        language: String,
    },
    /// Updates a caption track (`updateCaptionTrack`).
    UpdateCaptionTrack {
        /// Owning sequence.
        sequence_id: String,
        /// Target id.
        caption_track_id: String,
        /// New name.
        name: Option<String>,
        /// Enabled.
        enabled: Option<bool>,
    },
    /// Removes a caption track (`removeCaptionTrack`).
    RemoveCaptionTrack {
        /// Owning sequence.
        sequence_id: String,
        /// Target id.
        caption_track_id: String,
    },
    /// Renames the project (`renameProject`).
    RenameProject {
        /// New name.
        name: String,
    },
    /// Applies inner ops atomically as one undo step (`batch`).
    Batch {
        /// Inner ops.
        ops: Vec<EditOp>,
        /// History label key.
        label_key: String,
    },
}

impl EditOp {
    /// The History panel localization key `history.<op>` (04 §4.4).
    pub fn history_key(&self) -> &'static str {
        match self {
            EditOp::AddMedia { .. } => "history.addMedia",
            EditOp::RemoveMedia { .. } => "history.removeMedia",
            EditOp::UpdateMedia { .. } => "history.updateMedia",
            EditOp::CreateBin { .. } => "history.createBin",
            EditOp::UpdateBin { .. } => "history.updateBin",
            EditOp::DeleteBin { .. } => "history.deleteBin",
            EditOp::CreateSequence { .. } => "history.createSequence",
            EditOp::DuplicateSequence { .. } => "history.duplicateSequence",
            EditOp::DeleteSequence { .. } => "history.deleteSequence",
            EditOp::UpdateSequence { .. } => "history.updateSequence",
            EditOp::AddTrack { .. } => "history.addTrack",
            EditOp::RemoveTrack { .. } => "history.removeTrack",
            EditOp::MoveTrack { .. } => "history.moveTrack",
            EditOp::UpdateTrack { .. } => "history.updateTrack",
            EditOp::AddMarkers { .. } => "history.addMarkers",
            EditOp::UpdateMarker { .. } => "history.updateMarker",
            EditOp::RemoveMarkers { .. } => "history.removeMarkers",
            EditOp::AddCaptionTrack { .. } => "history.addCaptionTrack",
            EditOp::UpdateCaptionTrack { .. } => "history.updateCaptionTrack",
            EditOp::RemoveCaptionTrack { .. } => "history.removeCaptionTrack",
            EditOp::RenameProject { .. } => "history.renameProject",
            EditOp::Batch { .. } => "history.batch",
        }
    }
}

/// Marker factory from the wire input.
fn marker_from_input(input: &MarkerInput) -> Marker {
    Marker {
        id: alvs_core::id::new_id(),
        t: input.t,
        duration: input.duration,
        name: input.name.clone(),
        color: input.color.clone(),
        note: input.note.clone(),
        kind: if input.chapter {
            crate::marker::MarkerKind::Chapter
        } else {
            crate::marker::MarkerKind::Standard
        },
        extra: BTreeMap::new(),
    }
}

/// The result of applying one op (04 §4.4; contracts/crate-apis.md `edit`
/// row): the entity patch computed by structural diffing of the consecutive
/// roots, plus non-fatal warnings such as adjusted items (E1104 data) —
/// carried as data; the caller renders envelopes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditResult {
    /// The entity patch from the prior root to the new root (filled by
    /// `Model::apply` / `Model::undo` / `Model::redo`).
    pub patch: crate::patch::EntityPatch,
    /// Non-fatal warnings (code + detail).
    pub warnings: Vec<(String, String)>,
    /// Items moved by a frame-rate change (E1104 data).
    pub adjusted: Vec<String>,
}

/// The intrinsic parameter groups a `ParamTarget` can address (04 §4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ParamIntrinsic {
    /// The intrinsic transform block.
    Transform,
    /// The intrinsic crop block.
    Crop,
    /// The intrinsic opacity.
    Opacity,
    /// The audio clip object.
    Audio,
}

/// Where a parameter op applies (04 §4.4): one of the intrinsic blocks of a
/// clip, an effect/mask/title-layer instance of a clip or track, a
/// transition, or the master-bus effect of a sequence. The clip-editing and
/// parameter ops that consume it arrive with WP-12/WP-14 (FR-010).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(untagged, rename_all_fields = "camelCase")]
#[ts(export)]
pub enum ParamTarget {
    /// An intrinsic block of a clip.
    ClipIntrinsic {
        /// The clip.
        clip_id: String,
        /// Which intrinsic block.
        intrinsic: ParamIntrinsic,
    },
    /// An effect instance on a clip.
    ClipEffect {
        /// The clip.
        clip_id: String,
        /// The effect instance.
        effect_instance_id: String,
    },
    /// A mask of a clip.
    ClipMask {
        /// The clip.
        clip_id: String,
        /// The mask.
        mask_id: String,
    },
    /// A title layer of a clip.
    ClipTitleLayer {
        /// The clip.
        clip_id: String,
        /// The title layer.
        title_layer_id: String,
    },
    /// An effect instance on a track.
    TrackEffect {
        /// The track.
        track_id: String,
        /// The effect instance.
        effect_instance_id: String,
    },
    /// A transition's parameters.
    Transition {
        /// The transition.
        transition_id: String,
    },
    /// A master-bus effect of a sequence.
    SequenceMaster {
        /// The sequence.
        sequence_id: String,
        /// The master effect instance.
        master_effect_instance_id: String,
    },
}

/// The outcome of an application attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum ApplyOutcome {
    /// The op applied; the model changed.
    Applied(Box<EditResult>),
    /// The op was rejected with the given error; the model is unchanged
    /// (rollback, E0002 per 03 §3.5 when an invariant would break).
    Rejected(alvs_core::error::AppError),
}

/// Apply one op to a project in place (the `Model` wrapper drives history;
/// see `history.rs`). Returns the per-op result without running validation —
/// the caller validates after and rolls back on violation (03 §3.5).
pub fn apply_op(project: &mut Project, op: &EditOp) -> ApplyOutcome {
    match op {
        EditOp::AddMedia { items, bin_id } => {
            for item in items {
                let mut item = item.clone();
                if let Some(bin) = bin_id {
                    item.bin_id = Some(bin.clone());
                }
                project.media.push(item);
            }
            Applied(Box::default())
        }
        EditOp::RemoveMedia {
            media_ids,
            also_remove_clips,
        } => {
            for id in media_ids {
                let used_by_clips = project.sequences.iter().any(|s| {
                    s.tracks.iter().any(|t| {
                        t.items
                            .iter()
                            .any(|c| c.media_id.as_deref() == Some(id.as_str()))
                    })
                });
                if used_by_clips && !*also_remove_clips {
                    return Rejected(alvs_core::error::AppError::EditBlocked {
                        detail: format!("media {id} is used by clips (E1102)"),
                    });
                }
                project.media.retain(|m| m.id != *id);
                if *also_remove_clips {
                    for sequence in &mut project.sequences {
                        for track in &mut sequence.tracks {
                            track
                                .items
                                .retain(|c| c.media_id.as_deref() != Some(id.as_str()));
                        }
                    }
                }
            }
            Applied(Box::default())
        }
        EditOp::UpdateMedia { media_id, changes } => {
            let Some(media) = project.media.iter_mut().find(|m| m.id == *media_id) else {
                return Rejected(missing("media", media_id));
            };
            if let Some(name) = &changes.name {
                media.name = name.clone();
            }
            if let Some(label) = &changes.label {
                media.label = Some(label.clone());
            }
            if let Some(notes) = &changes.notes {
                media.notes = notes.clone();
            }
            if let Some(bin) = &changes.bin_id {
                media.bin_id = Some(bin.clone());
            }
            Applied(Box::default())
        }
        EditOp::CreateBin { name, parent_id } => {
            project.bins.push(crate::bin::Bin {
                id: alvs_core::id::new_id(),
                name: name.clone(),
                parent_id: parent_id.clone(),
                label: None,
                expanded: true,
                extra: BTreeMap::new(),
            });
            Applied(Box::default())
        }
        EditOp::UpdateBin { bin_id, name } => {
            let Some(bin) = project.bins.iter_mut().find(|b| b.id == *bin_id) else {
                return Rejected(missing("bin", bin_id));
            };
            bin.name = name.clone();
            Applied(Box::default())
        }
        EditOp::DeleteBin { bin_id } => {
            let Some(bin) = project.bins.iter().find(|b| b.id == *bin_id) else {
                return Rejected(missing("bin", bin_id));
            };
            let parent = bin.parent_id.clone();
            project.bins.retain(|b| b.id != *bin_id);
            // Contents move to the parent bin (04 §4.4).
            let orphan_target = parent.unwrap_or_default();
            for media in &mut project.media {
                if media.bin_id.as_deref() == Some(bin_id.as_str()) {
                    media.bin_id = if orphan_target.is_empty() {
                        None
                    } else {
                        Some(orphan_target.clone())
                    };
                }
            }
            for child in &mut project.bins {
                if child.parent_id.as_deref() == Some(bin_id.as_str()) {
                    child.parent_id = if orphan_target.is_empty() {
                        None
                    } else {
                        Some(orphan_target.clone())
                    };
                }
            }
            Applied(Box::default())
        }
        EditOp::CreateSequence {
            name,
            format,
            from_preset,
        } => {
            let fmt = format
                .clone()
                .unwrap_or_else(|| match from_preset.as_deref() {
                    // Named presets resolve to a concrete format; SQP-01
                    // (and no preset) keep auto-match with the SQP-02
                    // fallback (12 §12.8, REQ-PRJ-001).
                    Some("SQP-01") | None => sqp02_fallback_format(),
                    Some(_) => sqp02_fallback_with_preset_clear(),
                });
            project.sequences.push(new_sequence(name, fmt));
            Applied(Box::default())
        }
        EditOp::DuplicateSequence { sequence_id, name } => {
            let Some(source) = project.sequences.iter().find(|s| s.id == *sequence_id) else {
                return Rejected(missing("sequence", sequence_id));
            };
            let mut copy: Sequence = source.clone();
            copy.id = alvs_core::id::new_id();
            copy.name = name.clone();
            project.sequences.push(copy);
            Applied(Box::default())
        }
        EditOp::DeleteSequence { sequence_id } => {
            if project.sequences.len() <= 1 {
                return Rejected(alvs_core::error::AppError::EditBlocked {
                    detail: "cannot delete the last sequence (REQ-PRJ-005)".into(),
                });
            }
            let had = project.sequences.len();
            project.sequences.retain(|s| s.id != *sequence_id);
            if project.sequences.len() == had {
                return Rejected(missing("sequence", sequence_id));
            }
            if project.settings.default_sequence_id == *sequence_id {
                project.settings.default_sequence_id = project
                    .sequences
                    .first()
                    .map(|s| s.id.clone())
                    .unwrap_or_default();
            }
            if project.ui.active_sequence_id.as_deref() == Some(sequence_id.as_str()) {
                project.ui.active_sequence_id = project.sequences.first().map(|s| s.id.clone());
            }
            Applied(Box::default())
        }
        EditOp::UpdateSequence {
            sequence_id,
            changes,
        } => apply_update_sequence(project, sequence_id, changes),
        EditOp::AddTrack {
            sequence_id,
            kind,
            index,
            name,
        } => {
            let Some(sequence) = project.sequences.iter_mut().find(|s| s.id == *sequence_id) else {
                return Rejected(missing("sequence", sequence_id));
            };
            let default_name = match kind {
                TrackKind::Video => format!(
                    "V{}",
                    sequence
                        .tracks
                        .iter()
                        .filter(|t| t.kind == TrackKind::Video)
                        .count()
                        + 1
                ),
                TrackKind::Audio => format!(
                    "A{}",
                    sequence
                        .tracks
                        .iter()
                        .filter(|t| t.kind == TrackKind::Audio)
                        .count()
                        + 1
                ),
            };
            let track = crate::track::new_track(*kind, name.as_deref().unwrap_or(&default_name));
            let at = index.unwrap_or(sequence.tracks.len());
            sequence.tracks.insert(at.min(sequence.tracks.len()), track);
            Applied(Box::default())
        }
        EditOp::RemoveTrack {
            sequence_id,
            track_id,
        } => {
            let Some(sequence) = project.sequences.iter_mut().find(|s| s.id == *sequence_id) else {
                return Rejected(missing("sequence", sequence_id));
            };
            let had = sequence.tracks.len();
            sequence.tracks.retain(|t| t.id != *track_id);
            if sequence.tracks.len() == had {
                return Rejected(missing("track", track_id));
            }
            Applied(Box::default())
        }
        EditOp::MoveTrack {
            sequence_id,
            track_id,
            index,
        } => {
            let Some(sequence) = project.sequences.iter_mut().find(|s| s.id == *sequence_id) else {
                return Rejected(missing("sequence", sequence_id));
            };
            let Some(current) = sequence.tracks.iter().position(|t| t.id == *track_id) else {
                return Rejected(missing("track", track_id));
            };
            let track = sequence.tracks.remove(current);
            sequence
                .tracks
                .insert((*index).min(sequence.tracks.len()), track);
            Applied(Box::default())
        }
        EditOp::UpdateTrack {
            sequence_id,
            track_id,
            changes,
        } => {
            let Some(sequence) = project.sequences.iter_mut().find(|s| s.id == *sequence_id) else {
                return Rejected(missing("sequence", sequence_id));
            };
            let Some(track) = sequence.tracks.iter_mut().find(|t| t.id == *track_id) else {
                return Rejected(missing("track", track_id));
            };
            if let Some(name) = &changes.name {
                track.name = name.clone();
            }
            if let Some(height) = changes.height {
                track.height = height;
            }
            if let Some(locked) = changes.locked {
                track.locked = locked;
            }
            if let Some(visible) = changes.visible {
                track.visible = visible;
            }
            if let Some(muted) = changes.muted {
                track.muted = muted;
            }
            if let Some(solo) = changes.solo {
                track.solo = solo;
            }
            if let Some(volume) = changes.volume_db {
                track.volume_db = volume;
            }
            if let Some(pan) = changes.pan {
                track.pan = pan;
            }
            if let Some(targeted) = changes.targeted {
                track.targeted = targeted;
            }
            Applied(Box::default())
        }
        EditOp::AddMarkers {
            sequence_id,
            markers,
        } => {
            let Some(sequence) = project.sequences.iter_mut().find(|s| s.id == *sequence_id) else {
                return Rejected(missing("sequence", sequence_id));
            };
            for input in markers {
                sequence.markers.push(marker_from_input(input));
            }
            Applied(Box::default())
        }
        EditOp::UpdateMarker {
            sequence_id,
            marker_id,
            name,
            note,
        } => {
            let Some(sequence) = project.sequences.iter_mut().find(|s| s.id == *sequence_id) else {
                return Rejected(missing("sequence", sequence_id));
            };
            let Some(marker) = sequence.markers.iter_mut().find(|m| m.id == *marker_id) else {
                return Rejected(missing("marker", marker_id));
            };
            marker.name = name.clone();
            marker.note = note.clone();
            Applied(Box::default())
        }
        EditOp::RemoveMarkers {
            sequence_id,
            marker_ids,
        } => {
            let Some(sequence) = project.sequences.iter_mut().find(|s| s.id == *sequence_id) else {
                return Rejected(missing("sequence", sequence_id));
            };
            let had = sequence.markers.len();
            sequence.markers.retain(|m| !marker_ids.contains(&m.id));
            if sequence.markers.len() == had {
                return Rejected(missing(
                    "marker",
                    &marker_ids.first().cloned().unwrap_or_default(),
                ));
            }
            Applied(Box::default())
        }
        EditOp::AddCaptionTrack {
            sequence_id,
            name,
            language,
        } => {
            let Some(sequence) = project.sequences.iter_mut().find(|s| s.id == *sequence_id) else {
                return Rejected(missing("sequence", sequence_id));
            };
            sequence
                .caption_tracks
                .push(new_caption_track(name, language));
            Applied(Box::default())
        }
        EditOp::UpdateCaptionTrack {
            sequence_id,
            caption_track_id,
            name,
            enabled,
        } => {
            let Some(sequence) = project.sequences.iter_mut().find(|s| s.id == *sequence_id) else {
                return Rejected(missing("sequence", sequence_id));
            };
            let Some(track) = sequence
                .caption_tracks
                .iter_mut()
                .find(|t| t.id == *caption_track_id)
            else {
                return Rejected(missing("captionTrack", caption_track_id));
            };
            if let Some(name) = name {
                track.name = name.clone();
            }
            if let Some(enabled) = enabled {
                track.enabled = *enabled;
            }
            Applied(Box::default())
        }
        EditOp::RemoveCaptionTrack {
            sequence_id,
            caption_track_id,
        } => {
            let Some(sequence) = project.sequences.iter_mut().find(|s| s.id == *sequence_id) else {
                return Rejected(missing("sequence", sequence_id));
            };
            let had = sequence.caption_tracks.len();
            sequence
                .caption_tracks
                .retain(|t| t.id != *caption_track_id);
            if sequence.caption_tracks.len() == had {
                return Rejected(missing("captionTrack", caption_track_id));
            }
            Applied(Box::default())
        }
        EditOp::RenameProject { name } => {
            project.name = name.clone();
            Applied(Box::default())
        }
        EditOp::Batch { ops, label_key: _ } => {
            // All-or-nothing: any rejection leaves the project untouched
            // (the caller snapshots; this loop applies on the snapshot).
            let mut result = EditResult::default();
            for inner in ops {
                match apply_op(project, inner) {
                    Applied(inner_result) => {
                        result.adjusted.extend(inner_result.adjusted);
                        result.warnings.extend(inner_result.warnings);
                    }
                    Rejected(err) => return Rejected(err),
                }
            }
            Applied(Box::new(result))
        }
    }
}

/// Frame-rate change: snap clip starts/durations to the new grid and report
/// adjusted items (REQ-PRJ-005, E1104).
fn apply_update_sequence(
    project: &mut Project,
    sequence_id: &str,
    changes: &SequenceChanges,
) -> ApplyOutcome {
    let Some(sequence) = project.sequences.iter_mut().find(|s| s.id == *sequence_id) else {
        return Rejected(missing("sequence", sequence_id));
    };
    let mut result = EditResult::default();
    if let Some(name) = &changes.name {
        sequence.name = name.clone();
    }
    if let Some(format) = &changes.format {
        let old = sequence.format.frame_rate;
        let new = format.frame_rate;
        if old != new {
            if let (Some(old_fpf), Some(new_fpf)) = (
                alvs_core::time::flicks_per_frame(old),
                alvs_core::time::flicks_per_frame(new),
            ) {
                let mut adjusted = Vec::new();
                for track in &mut sequence.tracks {
                    for clip in &mut track.items {
                        let snapped_start = snap(clip.start, old_fpf, new_fpf);
                        let snapped_duration = snap(clip.duration, old_fpf, new_fpf);
                        if snapped_start != clip.start || snapped_duration != clip.duration {
                            adjusted.push(clip.id.clone());
                            clip.start = snapped_start;
                            clip.duration = snapped_duration;
                        }
                    }
                }
                if !adjusted.is_empty() {
                    result.adjusted = adjusted;
                    result.warnings.push((
                        "E1104".into(),
                        format!(
                            "{} items moved to match the new frame rate",
                            result.adjusted.len()
                        ),
                    ));
                }
            }
            sequence.format = format.clone();
        } else {
            sequence.format = format.clone();
        }
    }
    if let Some(in_point) = &changes.in_point {
        sequence.in_point = *in_point;
    }
    if let Some(out_point) = &changes.out_point {
        sequence.out_point = *out_point;
    }
    Applied(Box::new(result))
}

/// Snap a flicks value from the old grid to the nearest new-grid position.
fn snap(value: i64, old_fpf: i64, new_fpf: i64) -> i64 {
    let frames = (value as f64 / old_fpf as f64).round() as i64;
    frames * new_fpf
}

fn missing(kind: &str, id: &str) -> alvs_core::error::AppError {
    alvs_core::error::AppError::EditBlocked {
        detail: format!("{kind} {id} not found"),
    }
}

use ApplyOutcome::Applied;
use ApplyOutcome::Rejected;

/// SQP-02 without auto-match pending (named presets are concrete).
fn sqp02_fallback_with_preset_clear() -> SequenceFormat {
    let mut f = sqp02_fallback_format();
    f.auto_match_pending = false;
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn param_target_serializes_per_doc_shapes() {
        // Clip intrinsic (04 §4.4): { clipId, intrinsic }.
        let json = serde_json::json!({ "clipId": "c1", "intrinsic": "transform" });
        let target: ParamTarget = serde_json::from_value(json).unwrap_or(ParamTarget::Transition {
            transition_id: String::new(),
        });
        assert!(
            matches!(target, ParamTarget::ClipIntrinsic { ref clip_id, intrinsic: ParamIntrinsic::Transform } if clip_id == "c1")
        );
        // Round-trip keeps the doc shape.
        assert_eq!(
            serde_json::to_value(&target).unwrap_or(serde_json::Value::Null),
            serde_json::json!({ "clipId": "c1", "intrinsic": "transform" })
        );

        // Clip effect instance: { clipId, effectInstanceId }.
        let json = serde_json::json!({ "clipId": "c1", "effectInstanceId": "e1" });
        let target: ParamTarget = serde_json::from_value(json).unwrap_or(ParamTarget::Transition {
            transition_id: String::new(),
        });
        assert!(matches!(target, ParamTarget::ClipEffect { .. }));

        // Track effect, mask, title layer, transition, and sequence master.
        let json = serde_json::json!({ "trackId": "t1", "effectInstanceId": "e1" });
        assert!(matches!(
            serde_json::from_value::<ParamTarget>(json).unwrap_or(ParamTarget::Transition {
                transition_id: String::new()
            }),
            ParamTarget::TrackEffect { .. }
        ));
        let json = serde_json::json!({ "clipId": "c1", "maskId": "m1" });
        assert!(matches!(
            serde_json::from_value::<ParamTarget>(json).unwrap_or(ParamTarget::Transition {
                transition_id: String::new()
            }),
            ParamTarget::ClipMask { .. }
        ));
        let json = serde_json::json!({ "clipId": "c1", "titleLayerId": "l1" });
        assert!(matches!(
            serde_json::from_value::<ParamTarget>(json).unwrap_or(ParamTarget::Transition {
                transition_id: String::new()
            }),
            ParamTarget::ClipTitleLayer { .. }
        ));
        let json = serde_json::json!({ "transitionId": "t1" });
        assert!(matches!(
            serde_json::from_value::<ParamTarget>(json).unwrap_or(ParamTarget::ClipMask {
                clip_id: String::new(),
                mask_id: String::new()
            }),
            ParamTarget::Transition { .. }
        ));
        let json = serde_json::json!({ "sequenceId": "s1", "masterEffectInstanceId": "e1" });
        assert!(matches!(
            serde_json::from_value::<ParamTarget>(json).unwrap_or(ParamTarget::Transition {
                transition_id: String::new()
            }),
            ParamTarget::SequenceMaster { .. }
        ));
    }

    #[test]
    fn serde_tag_is_op_with_camel_case_variants() {
        let json = serde_json::json!({ "op": "renameProject", "name": "Cut" });
        let op: EditOp = serde_json::from_value(json).unwrap_or(EditOp::RenameProject {
            name: String::new(),
        });
        assert!(matches!(op, EditOp::RenameProject { ref name } if name == "Cut"));
        assert_eq!(op.history_key(), "history.renameProject");

        let json = serde_json::json!({
            "op": "addTrack",
            "sequenceId": "s1",
            "kind": "video"
        });
        let op: EditOp = serde_json::from_value(json).unwrap_or(EditOp::RenameProject {
            name: String::new(),
        });
        assert!(matches!(op, EditOp::AddTrack { .. }));
        assert_eq!(op.history_key(), "history.addTrack");
    }

    #[test]
    fn batch_serializes_with_label_key() {
        let json = serde_json::json!({
            "op": "batch",
            "ops": [{ "op": "renameProject", "name": "A" }],
            "labelKey": "history.batch.import"
        });
        let op: Result<EditOp, _> = serde_json::from_value(json);
        let op = op.unwrap_or_else(|e| panic!("batch deserialization failed: {e}"));
        assert!(matches!(op, EditOp::Batch { .. }));
    }
}
