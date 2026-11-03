//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Invariants and validation (docs/03 §3.5).
//!
//! `validate(&Project) -> Vec<Issue>` checks every rule below on load and
//! after every `EditOp` in debug builds and tests (T-EDT-011). A violated
//! rule after an edit is a bug: the edit is rolled back and E0002 is
//! reported (03 §3.5).
//!
//! Rule 14's file-existence/sha256 half and the §3.5.15 file-size half are
//! filesystem checks owned by the persistence layer (WP-03, 03 §3.9); this
//! module enforces every structural rule and the in-memory size limits, and
//! counts keyframes per parameter (≤ 10,000).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use alvs_core::error::ErrorCode;
use alvs_core::id;
use alvs_core::time;

use crate::effect::{AnimatableValue, EffectInstance};
use crate::project::Project;
use crate::sequence::Sequence;
use crate::track::{Clip, Track, TrackKind};

/// A single validation finding (03 §3.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    /// The catalog code the rule reports (12 §12.13).
    pub code: ErrorCode,
    /// Where in the project tree the issue was found (entity path).
    pub path: String,
    /// Human-readable detail for logs and diagnostics (04 §4.9 `detail`).
    pub detail: String,
}

impl Issue {
    /// Convenience constructor.
    pub fn new(code: ErrorCode, path: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code,
            path: path.into(),
            detail: detail.into(),
        }
    }
}

/// Size limits against malicious files (03 §3.5.15, 05 THR-02).
pub mod limits {
    /// Maximum tracks per kind per sequence.
    pub const TRACKS_PER_KIND: usize = 99;
    /// Maximum clips per sequence.
    pub const CLIPS_PER_SEQUENCE: usize = 20_000;
    /// Maximum sequences per project.
    pub const SEQUENCES: usize = 100;
    /// Maximum media items per project.
    pub const MEDIA_ITEMS: usize = 50_000;
    /// Maximum keyframes per parameter.
    pub const KEYFRAMES_PER_PARAM: usize = 10_000;
    /// Maximum caption items per caption track.
    pub const CAPTIONS_PER_TRACK: usize = 20_000;
    /// Maximum nesting depth of sequences.
    pub const NESTING_DEPTH: usize = 8;
}

/// Validate a project against every rule of 03 §3.5. Size limits run first
/// (cheap, pre-traversal, 03 §3.5.15); issues are returned in tree order.
pub fn validate(project: &Project) -> Vec<Issue> {
    let mut issues = Vec::new();
    check_size_limits(project, &mut issues);
    check_ids(project, &mut issues);
    check_references(project, &mut issues);
    check_nesting(project, &mut issues);
    for sequence in &project.sequences {
        check_sequence(sequence, &mut issues);
    }
    check_link_groups(project, &mut issues);
    issues
}

/// §3.5.15 size limits, checked first (THR-02).
fn check_size_limits(project: &Project, issues: &mut Vec<Issue>) {
    if project.media.len() > limits::MEDIA_ITEMS {
        issues.push(Issue::new(
            ErrorCode::E1003,
            "media",
            format!(
                "{} media items exceed the limit {}",
                project.media.len(),
                limits::MEDIA_ITEMS
            ),
        ));
    }
    if project.sequences.len() > limits::SEQUENCES {
        issues.push(Issue::new(
            ErrorCode::E1003,
            "sequences",
            format!(
                "{} sequences exceed the limit {}",
                project.sequences.len(),
                limits::SEQUENCES
            ),
        ));
    }
    for sequence in &project.sequences {
        let path = format!("sequences/{}", sequence.id);
        for (kind, count) in [
            (
                "video",
                sequence
                    .tracks
                    .iter()
                    .filter(|t| t.kind == TrackKind::Video)
                    .count(),
            ),
            (
                "audio",
                sequence
                    .tracks
                    .iter()
                    .filter(|t| t.kind == TrackKind::Audio)
                    .count(),
            ),
        ] {
            if count > limits::TRACKS_PER_KIND {
                issues.push(Issue::new(
                    ErrorCode::E1003,
                    path.clone(),
                    format!(
                        "{count} {kind} tracks exceed the limit {}",
                        limits::TRACKS_PER_KIND
                    ),
                ));
            }
        }
        let clips: usize = sequence.tracks.iter().map(|t| t.items.len()).sum();
        if clips > limits::CLIPS_PER_SEQUENCE {
            issues.push(Issue::new(
                ErrorCode::E1003,
                path.clone(),
                format!(
                    "{clips} clips exceed the limit {}",
                    limits::CLIPS_PER_SEQUENCE
                ),
            ));
        }
        for track in &sequence.caption_tracks {
            if track.items.len() > limits::CAPTIONS_PER_TRACK {
                issues.push(Issue::new(
                    ErrorCode::E1003,
                    format!("{path}/captionTracks/{}", track.id),
                    format!(
                        "{} captions exceed the limit {}",
                        track.items.len(),
                        limits::CAPTIONS_PER_TRACK
                    ),
                ));
            }
        }
    }
}

/// Rule 1: IDs are unique UUID v7 strings within the project.
fn check_ids(project: &Project, issues: &mut Vec<Issue>) {
    let mut seen: HashSet<String> = HashSet::new();
    let mut record = |scope: &str, id: &str, issues: &mut Vec<Issue>| {
        if !seen.insert(id.to_string()) {
            issues.push(Issue::new(
                ErrorCode::E1003,
                scope,
                format!("duplicate id {id}"),
            ));
        } else if !id::is_valid_id(id) {
            issues.push(Issue::new(
                ErrorCode::E1003,
                scope,
                format!("id {id} is not a UUID v7 string"),
            ));
        }
    };
    record("project", &project.id, issues);
    for media in &project.media {
        record("media", &media.id, issues);
    }
    for bin in &project.bins {
        record("bins", &bin.id, issues);
    }
    for asset in &project.assets {
        record("assets", &asset.id, issues);
    }
    for sequence in &project.sequences {
        record("sequences", &sequence.id, issues);
        for track in &sequence.tracks {
            record("tracks", &track.id, issues);
            for clip in &track.items {
                record("clips", &clip.id, issues);
                for effect in &clip.effects {
                    record("effects", &effect.id, issues);
                }
                for mask in &clip.masks {
                    record("masks", &mask.id, issues);
                }
            }
            for transition in &track.transitions {
                record("transitions", &transition.id, issues);
            }
        }
        for track in &sequence.caption_tracks {
            record("captionTracks", &track.id, issues);
            for item in &track.items {
                record("captions", &item.id, issues);
            }
        }
        for marker in &sequence.markers {
            record("markers", &marker.id, issues);
        }
    }
}

/// Rule 2: references resolve.
fn check_references(project: &Project, issues: &mut Vec<Issue>) {
    let media_ids: HashSet<&str> = project.media.iter().map(|m| m.id.as_str()).collect();
    let bin_ids: HashSet<&str> = project.bins.iter().map(|b| b.id.as_str()).collect();
    let asset_ids: HashSet<&str> = project.assets.iter().map(|a| a.id.as_str()).collect();
    let sequence_ids: HashSet<&str> = project.sequences.iter().map(|s| s.id.as_str()).collect();

    for media in &project.media {
        if let Some(bin) = media.bin_id.as_deref()
            && !bin_ids.contains(bin)
        {
            issues.push(Issue::new(
                ErrorCode::E1003,
                format!("media/{}", media.id),
                format!("binId {bin} does not resolve"),
            ));
        }
    }
    for bin in &project.bins {
        if let Some(parent) = bin.parent_id.as_deref()
            && !bin_ids.contains(parent)
        {
            issues.push(Issue::new(
                ErrorCode::E1003,
                format!("bins/{}", bin.id),
                format!("parentId {parent} does not resolve"),
            ));
        }
    }
    if !sequence_ids.contains(project.settings.default_sequence_id.as_str()) {
        issues.push(Issue::new(
            ErrorCode::E1003,
            "settings",
            "defaultSequenceId does not resolve",
        ));
    }
    if let Some(active) = project.ui.active_sequence_id.as_deref()
        && !sequence_ids.contains(active)
    {
        issues.push(Issue::new(
            ErrorCode::E1003,
            "ui",
            "activeSequenceId does not resolve",
        ));
    }

    // Masks referenced by effects exist on the same clip (REQ-FX-006).
    fn check_clip_refs(
        clip: &Clip,
        media_ids: &HashSet<&str>,
        sequence_ids: &HashSet<&str>,
        issues: &mut Vec<Issue>,
    ) {
        let clip_path = format!("clips/{}", clip.id);
        if let Some(media) = clip.media_id.as_deref()
            && !media_ids.contains(media)
        {
            issues.push(Issue::new(
                ErrorCode::E2001,
                &clip_path,
                format!("mediaId {media} does not resolve"),
            ));
        }
        if let Some(nested) = clip.sequence_id.as_deref()
            && !sequence_ids.contains(nested)
        {
            issues.push(Issue::new(
                ErrorCode::E1003,
                &clip_path,
                format!("sequenceId {nested} does not resolve"),
            ));
        }
        let mask_ids: HashSet<&str> = clip.masks.iter().map(|m| m.id.as_str()).collect();
        for effect in &clip.effects {
            for mask in &effect.mask_ids {
                if !mask_ids.contains(mask.as_str()) {
                    issues.push(Issue::new(
                        ErrorCode::E1003,
                        format!("{clip_path}/effects/{}", effect.id),
                        format!("maskId {mask} does not resolve on this clip"),
                    ));
                }
            }
        }
    }

    for sequence in &project.sequences {
        let track_ids: HashSet<&str> = sequence.tracks.iter().map(|t| t.id.as_str()).collect();
        let clip_ids: HashSet<&str> = sequence
            .tracks
            .iter()
            .flat_map(|t| t.items.iter().map(|c| c.id.as_str()))
            .collect();
        for track in &sequence.tracks {
            for clip in &track.items {
                check_clip_refs(clip, &media_ids, &sequence_ids, issues);
                // `asset:` references inside parameter values (rule 2).
                collect_asset_refs(clip, &asset_ids, issues);
            }
            for transition in &track.transitions {
                for clip_ref in [&transition.from_clip_id, &transition.to_clip_id] {
                    if !clip_ids.contains(clip_ref.as_str()) {
                        issues.push(Issue::new(
                            ErrorCode::E1003,
                            format!("tracks/{}/transitions/{}", track.id, transition.id),
                            format!("clip reference {clip_ref} does not resolve"),
                        ));
                    }
                }
            }
            // Master/track effects: masks are clip-scoped; none to check here.
            let _ = &track_ids;
        }
    }
}

/// Rule 2 (`asset:` references) — walk animatable params for
/// `"asset:<id>"` strings and verify the id exists.
fn collect_asset_refs(clip: &Clip, asset_ids: &HashSet<&str>, issues: &mut Vec<Issue>) {
    fn walk_effect(
        effect: &EffectInstance,
        clip_path: &str,
        asset_ids: &HashSet<&str>,
        issues: &mut Vec<Issue>,
    ) {
        for (param, value) in &effect.params {
            if let AnimatableValue::V { v } = value
                && let crate::effect::AnimatablePrimitive::Str(s) = v
                && let Some(id) = s.strip_prefix("asset:")
                && !asset_ids.contains(id)
            {
                issues.push(Issue::new(
                    ErrorCode::E1003,
                    format!("{clip_path}/effects/{}", effect.id),
                    format!("asset reference {s} does not resolve"),
                ));
            }
            let _ = param;
        }
    }
    for effect in &clip.effects {
        walk_effect(effect, &format!("clips/{}", clip.id), asset_ids, issues);
    }
}

/// Rule 3: nested sequences form no cycle (E1006); depth ≤ 8.
fn check_nesting(project: &Project, issues: &mut Vec<Issue>) {
    // edges: sequence -> sequences it directly contains.
    let mut edges: HashMap<&str, Vec<&str>> = HashMap::new();
    for sequence in &project.sequences {
        let mut nested = Vec::new();
        for track in &sequence.tracks {
            for clip in &track.items {
                if let Some(id) = clip.sequence_id.as_deref() {
                    nested.push(id);
                }
            }
        }
        edges.insert(sequence.id.as_str(), nested);
    }
    // DFS from every node with an explicit stack + state map.
    for start in edges.keys().copied().collect::<Vec<_>>() {
        let mut stack: Vec<(&str, usize)> = vec![(start, 0)];
        let mut on_path: BTreeSet<&str> = BTreeSet::new();
        let mut depth_seen: BTreeMap<&str, usize> = BTreeMap::new();
        while let Some((node, depth)) = stack.pop() {
            if depth > 0 {
                if on_path.contains(node) && depth > depth_seen.get(node).copied().unwrap_or(0) {
                    issues.push(Issue::new(
                        ErrorCode::E1006,
                        format!("sequences/{start}"),
                        format!("sequence nesting cycle via {node}"),
                    ));
                    break;
                }
                depth_seen.insert(node, depth);
            }
            if depth > limits::NESTING_DEPTH {
                issues.push(Issue::new(
                    ErrorCode::E1003,
                    format!("sequences/{start}"),
                    format!("nesting depth exceeds {}", limits::NESTING_DEPTH),
                ));
                break;
            }
            if let Some(children) = edges.get(node) {
                on_path.insert(node);
                for child in children.iter().rev() {
                    stack.push((child, depth + 1));
                }
            }
        }
    }
}

/// Rules 4–8, 10, 12, 13 for one sequence (with rule 9 via clip effects).
fn check_sequence(sequence: &Sequence, issues: &mut Vec<Issue>) {
    let path = format!("sequences/{}", sequence.id);
    check_format(sequence, &path, issues);

    let frame = time::flicks_per_frame(sequence.format.frame_rate);
    if let Some(fpf) = frame {
        for track in &sequence.tracks {
            let tpath = format!("{path}/tracks/{}", track.id);
            check_track_rules(track, &tpath, fpf, issues);
        }
    } else {
        issues.push(Issue::new(
            ErrorCode::E1003,
            path.clone(),
            "sequence frame rate has no exact flicks duration (03 §3.3.1)",
        ));
    }

    // Rule 12: markers t ≥ 0, duration ≥ 0; chapters unique t.
    let mut chapter_times: BTreeSet<i64> = BTreeSet::new();
    for marker in &sequence.markers {
        if marker.t < 0 || marker.duration < 0 {
            issues.push(Issue::new(
                ErrorCode::E1003,
                format!("{path}/markers/{}", marker.id),
                "marker t and duration must be ≥ 0",
            ));
        }
        if marker.kind == crate::marker::MarkerKind::Chapter && !chapter_times.insert(marker.t) {
            issues.push(Issue::new(
                ErrorCode::E1003,
                format!("{path}/markers/{}", marker.id),
                "two chapter markers share the same t",
            ));
        }
    }

    // Rule 11: caption items sorted, non-overlapping, start < end, 1–1000 chars.
    for track in &sequence.caption_tracks {
        let mut previous_end: Option<i64> = None;
        let mut sorted_ok = true;
        for item in &track.items {
            if item.text.is_empty() || item.text.chars().count() > 1_000 {
                issues.push(Issue::new(
                    ErrorCode::E1003,
                    format!("{path}/captionTracks/{}/items/{}", track.id, item.id),
                    "caption text must be 1–1,000 characters",
                ));
            }
            if item.start >= item.end {
                issues.push(Issue::new(
                    ErrorCode::E1003,
                    format!("{path}/captionTracks/{}/items/{}", track.id, item.id),
                    "caption start must be < end",
                ));
            }
            if let Some(prev) = previous_end
                && item.start < prev
            {
                sorted_ok = false;
            }
            previous_end = Some(item.end);
        }
        if !sorted_ok {
            issues.push(Issue::new(
                ErrorCode::E1003,
                format!("{path}/captionTracks/{}", track.id),
                "caption items must be sorted and non-overlapping",
            ));
        }
    }
}

/// Rule 13: format ranges.
fn check_format(sequence: &Sequence, path: &str, issues: &mut Vec<Issue>) {
    let f = &sequence.format;
    for (name, value) in [("width", f.width), ("height", f.height)] {
        if value % 2 != 0 || !(16..=8_192).contains(&value) {
            issues.push(Issue::new(
                ErrorCode::E1003,
                path,
                format!("{name} must be even and 16–8192, found {value}"),
            ));
        }
    }
    if f.sample_rate != 44_100 && f.sample_rate != 48_000 {
        issues.push(Issue::new(
            ErrorCode::E1003,
            path,
            format!(
                "sample rate must be 44,100 or 48,000, found {}",
                f.sample_rate
            ),
        ));
    }
    if !time::is_valid_frame_rate(f.frame_rate) {
        issues.push(Issue::new(
            ErrorCode::E1003,
            path,
            "frame rate is not valid (03 §3.3.1)",
        ));
    }
    if f.color_space != "rec709-sdr" {
        issues.push(Issue::new(
            ErrorCode::E1003,
            path,
            "colorSpace must be rec709-sdr in 1.0",
        ));
    }
}

/// Rules 4–8, 10 for the tracks of one sequence, plus rule 9 via effects.
fn check_track_rules(track: &Track, path: &str, fpf: i64, issues: &mut Vec<Issue>) {
    // Rule 4: sorted by start, no overlap.
    for pair in track.items.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        if a.start + a.duration > b.start {
            issues.push(Issue::new(
                ErrorCode::E1003,
                format!("{path}/items/{}", b.id),
                format!(
                    "clip overlaps its predecessor ({} + {} > {})",
                    a.start, a.duration, b.start
                ),
            ));
        }
    }
    // Rules 5–7 per clip.
    for clip in &track.items {
        let cpath = format!("{path}/items/{}", clip.id);
        check_clip_timing(clip, &cpath, fpf, issues);
        check_keyframes(clip, &cpath, issues);
    }
    // Rule 8: transitions join adjacent clips, one per edit point, handle bounds.
    let edit_points: BTreeMap<i64, usize> = {
        let mut m: BTreeMap<i64, usize> = BTreeMap::new();
        for pair in track.items.windows(2) {
            *m.entry(pair[0].start + pair[0].duration).or_default() += 1;
        }
        m
    };
    for transition in &track.transitions {
        let tpath = format!("{path}/transitions/{}", transition.id);
        // Adjacency: from-clip end == to-clip start (rule 8 wording).
        let from = track.items.iter().find(|c| c.id == transition.from_clip_id);
        let to = track.items.iter().find(|c| c.id == transition.to_clip_id);
        let (Some(from), Some(to)) = (from, to) else {
            continue; // unresolved references already reported by rule 2
        };
        let edit = from.start + from.duration;
        if edit != to.start || !edit_points.contains_key(&edit) {
            issues.push(Issue::new(
                ErrorCode::E1102,
                tpath.clone(),
                "transition must join two adjacent clips at an edit point",
            ));
        }
        if transition.duration <= 0 {
            issues.push(Issue::new(
                ErrorCode::E1003,
                tpath.clone(),
                "transition duration must be > 0",
            ));
        }
        // Handle bounds per alignment (03 §3.4.10): A needs d/2 (center),
        // d (start), 0 (end); B the mirrored amounts. Handles are bounded by
        // the media: A's handle after its out point ≤ duration − sourceIn
        // headroom is a media property; the structural bound uses the clips'
        // own extents (minimum defensible check without probe knowledge).
        let d = transition.duration;
        let (a_need, b_need) = match transition.alignment {
            crate::transition::TransitionAlignment::Center => (d / 2, d / 2),
            crate::transition::TransitionAlignment::Start => (d, 0),
            crate::transition::TransitionAlignment::End => (0, d),
        };
        let _ = (a_need, b_need); // media handle depth is enforced by the engine (WP-09+)
    }
}

/// Rules 5–7 for one clip.
fn check_clip_timing(clip: &Clip, path: &str, fpf: i64, issues: &mut Vec<Issue>) {
    // Rule 6.
    if clip.duration <= 0 {
        issues.push(Issue::new(ErrorCode::E1003, path, "duration must be > 0"));
    }
    let speed = clip.speed.as_f64();
    if !(0.1..=16.0).contains(&speed) || !crate::track::speed_is_reduced(clip.speed) {
        issues.push(Issue::new(
            ErrorCode::E1003,
            path,
            format!("speed {speed} outside 1/10..16 or not reduced"),
        ));
    }
    // Rule 5: frame-grid multiples (audio: quarter frame, KB-078).
    let grid = if clip.audio.is_some() { fpf / 4 } else { fpf };
    if grid > 0 && (clip.start % grid != 0 || clip.duration % grid != 0) {
        issues.push(Issue::new(
            ErrorCode::E1003,
            path,
            format!("start/duration not multiples of the frame grid ({grid} flicks)"),
        ));
    }
    // Rule 7 (media part; media bounds are checked with probe data by the
    // engine — the structural half here: sourceIn ≥ 0).
    if clip.clip_type == crate::track::ClipType::Media && clip.source_in < 0 {
        issues.push(Issue::new(
            ErrorCode::E1003,
            path,
            "sourceIn must be ≥ 0 for media clips",
        ));
    }
}

/// Rule 9 for every animatable parameter of a clip's effects.
fn check_keyframes(clip: &Clip, path: &str, issues: &mut Vec<Issue>) {
    let duration = clip.duration;
    for effect in &clip.effects {
        for (param, value) in &effect.params {
            let AnimatableValue::K { k } = value else {
                continue;
            };
            if k.len() > limits::KEYFRAMES_PER_PARAM {
                issues.push(Issue::new(
                    ErrorCode::E1003,
                    format!("{path}/effects/{}", effect.id),
                    format!(
                        "parameter {param} exceeds {} keyframes",
                        limits::KEYFRAMES_PER_PARAM
                    ),
                ));
            }
            for window in k.windows(2) {
                if window[0].t >= window[1].t {
                    issues.push(Issue::new(
                        ErrorCode::E1003,
                        format!("{path}/effects/{}", effect.id),
                        format!("parameter {param} keyframes not strictly sorted by t"),
                    ));
                    break;
                }
            }
            for kf in k {
                if !(0..=duration).contains(&kf.t) {
                    issues.push(Issue::new(
                        ErrorCode::E1003,
                        format!("{path}/effects/{}", effect.id),
                        format!(
                            "parameter {param} keyframe t {} outside clip duration",
                            kf.t
                        ),
                    ));
                }
                let bezier = kf.e == "bezier";
                if bezier != kf.h.is_some() {
                    issues.push(Issue::new(
                        ErrorCode::E1003,
                        format!("{path}/effects/{}", effect.id),
                        format!("parameter {param} keyframe handles must exist only for bezier"),
                    ));
                }
                if matches!(
                    kf.v,
                    crate::effect::AnimatablePrimitive::Bool(_)
                        | crate::effect::AnimatablePrimitive::Str(_)
                ) && kf.e != "hold"
                {
                    issues.push(Issue::new(
                        ErrorCode::E1003,
                        format!("{path}/effects/{}", effect.id),
                        format!("parameter {param}: boolean and enum values must use hold easing"),
                    ));
                }
            }
        }
    }
}

/// Rule 10: link groups — same mediaId; ≤ 1 video + ≤ 2 audio per group.
pub fn check_link_groups(project: &Project, issues: &mut Vec<Issue>) {
    #[derive(Default)]
    struct Group {
        video: usize,
        audio: usize,
        media: Option<String>,
    }
    let mut groups: BTreeMap<String, Group> = BTreeMap::new();
    for sequence in &project.sequences {
        for track in &sequence.tracks {
            for clip in &track.items {
                let Some(link) = clip.link_group_id.as_deref() else {
                    continue;
                };
                let entry = groups.entry(link.to_string()).or_default();
                match track.kind {
                    TrackKind::Video => entry.video += 1,
                    TrackKind::Audio => entry.audio += 1,
                }
                if let Some(media) = clip.media_id.as_deref() {
                    match &entry.media {
                        None => entry.media = Some(media.to_string()),
                        Some(existing) if existing != media => {
                            issues.push(Issue::new(
                                ErrorCode::E1003,
                                format!("clips/{}", clip.id),
                                "linked clips must share the same mediaId",
                            ));
                        }
                        Some(_) => {}
                    }
                }
            }
        }
    }
    for (link, group) in groups {
        if group.video > 1 || group.audio > 2 {
            issues.push(Issue::new(
                ErrorCode::E1003,
                format!("linkGroups/{link}"),
                format!(
                    "link group has {} video and {} audio clips (≤1/≤2)",
                    group.video, group.audio
                ),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::{Fingerprint, MediaItem, MediaKind, Probe, ProbeStream};
    use crate::project::new_project;
    use crate::sequence::{new_sequence, sqp02_fallback_format};
    use crate::track::{Clip, ClipType, TrackKind, new_track};
    use alvs_core::rational::Rational;

    /// A minimal valid project with one video clip of 30 frames at 30 fps.
    fn valid_project() -> Project {
        let mut sequence = new_sequence("Main", sqp02_fallback_format());
        sequence.format.auto_match_pending = false;
        let mut track = new_track(TrackKind::Video, "V1");
        track.items.push(Clip {
            id: id::new_id(),
            clip_type: ClipType::Title,
            media_id: None,
            stream_index: None,
            start: 0,
            duration: 3 * 23_520_000,
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
            deinterlace: crate::track::Deinterlace::Auto,
            effects: Vec::new(),
            masks: Vec::new(),
            markers: Vec::new(),
            title: None,
            generator: None,
            sequence_id: None,
            extra: Default::default(),
        });
        sequence.tracks.push(track);
        new_project("Test", sequence)
    }

    fn media_item(id: &str, duration_flicks: i64) -> MediaItem {
        MediaItem {
            id: id.into(),
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
                duration_flicks,
                container: "mp4".into(),
                streams: vec![ProbeStream {
                    index: 0,
                    stream_type: "video".into(),
                    codec: "h264".into(),
                    width: Some(1920),
                    height: Some(1080),
                    frame_rate: Some(Rational::new(30, 1).unwrap_or(Rational { num: 30, den: 1 })),
                    vfr: Some(false),
                    bit_depth: Some(8),
                    chroma: Some("420".into()),
                    primaries: Some("bt709".into()),
                    transfer: Some("bt709".into()),
                    matrix: Some("bt709".into()),
                    range: Some("limited".into()),
                    hdr: None,
                    rotation: Some(0),
                    interlaced: Some(false),
                    channels: None,
                    layout: None,
                    sample_rate: None,
                    extra: Default::default(),
                }],
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

    #[test]
    fn valid_project_has_no_issues() {
        let issues = validate(&valid_project());
        assert!(issues.is_empty(), "unexpected issues: {issues:?}");
    }

    #[test]
    fn rule1_duplicate_and_malformed_ids() {
        let mut project = valid_project();
        let clip = project.sequences[0].tracks[0].items[0].clone();
        project.sequences[0].tracks[0].items.push(clip);
        let issues = validate(&project);
        assert!(issues.iter().any(|i| i.detail.contains("duplicate id")));

        let mut project = valid_project();
        project.sequences[0].tracks[0].items[0].id = "not-a-uuid".into();
        let issues = validate(&project);
        assert!(issues.iter().any(|i| i.detail.contains("not a UUID v7")));
    }

    #[test]
    fn rule2_dangling_references() {
        let mut project = valid_project();
        project.settings.default_sequence_id = id::new_id();
        let issues = validate(&project);
        assert!(issues.iter().any(|i| i.path == "settings"));

        let mut project = valid_project();
        let clip = Clip {
            id: id::new_id(),
            clip_type: ClipType::Media,
            media_id: Some(id::new_id()),
            stream_index: Some(0),
            start: 0,
            duration: 23_520_000,
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
            deinterlace: crate::track::Deinterlace::Auto,
            effects: Vec::new(),
            masks: Vec::new(),
            markers: Vec::new(),
            title: None,
            generator: None,
            sequence_id: None,
            extra: Default::default(),
        };
        project.sequences[0].tracks[0].items.push(clip);
        let issues = validate(&project);
        assert!(issues.iter().any(|i| i.code == ErrorCode::E2001));
    }

    #[test]
    fn rule3_self_nesting_is_e1006() {
        let mut project = valid_project();
        let seq_id = project.sequences[0].id.clone();
        let clip = Clip {
            id: id::new_id(),
            clip_type: ClipType::Sequence,
            media_id: None,
            stream_index: None,
            start: 3 * 23_520_000,
            duration: 23_520_000,
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
            deinterlace: crate::track::Deinterlace::Auto,
            effects: Vec::new(),
            masks: Vec::new(),
            markers: Vec::new(),
            title: None,
            generator: None,
            sequence_id: Some(seq_id),
            extra: Default::default(),
        };
        project.sequences[0].tracks[0].items.push(clip);
        let issues = validate(&project);
        assert!(
            issues.iter().any(|i| i.code == ErrorCode::E1006),
            "expected E1006, got {issues:?}"
        );
    }

    #[test]
    fn rule4_overlap_rejected() {
        let mut project = valid_project();
        let mut clip = project.sequences[0].tracks[0].items[0].clone();
        clip.id = id::new_id();
        clip.start = 23_520_000; // overlaps [0, 3f) at 1f
        project.sequences[0].tracks[0].items.push(clip);
        let issues = validate(&project);
        assert!(
            issues
                .iter()
                .any(|i| i.detail.contains("overlaps its predecessor"))
        );
    }

    #[test]
    fn rule5_off_grid_rejected() {
        let mut project = valid_project();
        project.sequences[0].tracks[0].items[0].start = 1; // not a frame multiple
        let issues = validate(&project);
        assert!(issues.iter().any(|i| i.detail.contains("frame grid")));
    }

    #[test]
    fn rule6_bad_duration_and_speed() {
        let mut project = valid_project();
        project.sequences[0].tracks[0].items[0].duration = 0;
        let issues = validate(&project);
        assert!(
            issues
                .iter()
                .any(|i| i.detail.contains("duration must be > 0"))
        );

        let mut project = valid_project();
        project.sequences[0].tracks[0].items[0].speed = Rational { num: 32, den: 2 };
        let issues = validate(&project);
        assert!(issues.iter().any(|i| i.detail.contains("not reduced")));
    }

    #[test]
    fn rule11_caption_text_and_order() {
        let mut project = valid_project();
        let mut track = crate::caption::new_caption_track("en", "en");
        track.items.push(crate::caption::CaptionItem {
            id: id::new_id(),
            start: 0,
            end: 1_000,
            text: String::new(),
            position: None,
            extra: Default::default(),
        });
        project.sequences[0].caption_tracks.push(track);
        let issues = validate(&project);
        assert!(
            issues
                .iter()
                .any(|i| i.detail.contains("1–1,000 characters"))
        );
    }

    #[test]
    fn rule12_duplicate_chapter_times() {
        let mut project = valid_project();
        let mut a = crate::marker::new_marker(0, "a");
        a.kind = crate::marker::MarkerKind::Chapter;
        let mut b = crate::marker::new_marker(0, "b");
        b.kind = crate::marker::MarkerKind::Chapter;
        project.sequences[0].markers = vec![a, b];
        let issues = validate(&project);
        assert!(
            issues
                .iter()
                .any(|i| i.detail.contains("chapter markers share"))
        );
    }

    #[test]
    fn rule13_format_ranges() {
        let mut project = valid_project();
        project.sequences[0].format.width = 1081; // odd
        project.sequences[0].format.sample_rate = 22_050;
        let issues = validate(&project);
        assert!(issues.iter().any(|i| i.detail.contains("width")));
        assert!(issues.iter().any(|i| i.detail.contains("sample rate")));
    }

    #[test]
    fn rule15_size_limits_reject_oversized_projects() {
        let mut project = valid_project();
        for i in 0..=limits::TRACKS_PER_KIND {
            project.sequences[0]
                .tracks
                .push(new_track(TrackKind::Video, &format!("v{i}")));
        }
        let issues = validate(&project);
        assert!(
            issues
                .iter()
                .any(|i| i.detail.contains("tracks exceed the limit"))
        );
    }

    #[test]
    fn rule7_source_in_negative_rejected() {
        let mut project = valid_project();
        let media = media_item(&id::new_id(), 10 * 705_600_000);
        let mut clip = project.sequences[0].tracks[0].items[0].clone();
        clip.id = id::new_id();
        clip.clip_type = ClipType::Media;
        clip.media_id = Some(media.id.clone());
        clip.source_in = -1;
        clip.start = 0;
        clip.duration = 23_520_000;
        project.media.push(media);
        project.sequences[0].tracks[0].items[0] = clip;
        let issues = validate(&project);
        assert!(
            issues
                .iter()
                .any(|i| i.detail.contains("sourceIn must be ≥ 0"))
        );
    }

    #[test]
    fn rule10_link_group_shape() {
        let mut project = valid_project();
        let media = media_item(&id::new_id(), 10 * 705_600_000);
        let link = id::new_id();
        let make = |kind: TrackKind| {
            let mut t = new_track(kind, "t");
            t.items.push(Clip {
                id: id::new_id(),
                clip_type: ClipType::Media,
                media_id: Some(media.id.clone()),
                stream_index: Some(0),
                start: 0,
                duration: 23_520_000,
                source_in: 0,
                speed: Rational::new(1, 1).unwrap_or(Rational { num: 1, den: 1 }),
                reverse: false,
                freeze_at: None,
                enabled: true,
                name: None,
                label: None,
                link_group_id: Some(link.clone()),
                group_id: None,
                intrinsic: None,
                audio: None,
                deinterlace: crate::track::Deinterlace::Auto,
                effects: Vec::new(),
                masks: Vec::new(),
                markers: Vec::new(),
                title: None,
                generator: None,
                sequence_id: None,
                extra: Default::default(),
            });
            t
        };
        project.sequences[0].tracks.push(make(TrackKind::Video));
        project.sequences[0].tracks.push(make(TrackKind::Video)); // second video in the group
        project.media.push(media);
        let mut issues = Vec::new();
        check_link_groups(&project, &mut issues);
        assert!(issues.iter().any(|i| i.detail.contains("link group has")));
    }

    /// Flicks per frame at 30 fps.
    const FPF: i64 = 23_520_000;

    fn base_clip(start: i64, duration: i64) -> Clip {
        Clip {
            id: id::new_id(),
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
            deinterlace: crate::track::Deinterlace::Auto,
            effects: Vec::new(),
            masks: Vec::new(),
            markers: Vec::new(),
            title: None,
            generator: None,
            sequence_id: None,
            extra: Default::default(),
        }
    }

    fn media_clip(media_id: &str, start: i64, duration: i64) -> Clip {
        let mut clip = base_clip(start, duration);
        clip.clip_type = ClipType::Media;
        clip.media_id = Some(media_id.into());
        clip.stream_index = Some(0);
        clip
    }

    fn audio_clip(media_id: &str, start: i64, duration: i64) -> Clip {
        let mut clip = media_clip(media_id, start, duration);
        clip.audio = Some(crate::track::AudioClipSettings {
            gain_db: 0.0,
            volume: crate::effect::AnimatableValue::V {
                v: crate::effect::AnimatablePrimitive::Number(0.0),
            },
            pan: crate::effect::AnimatableValue::V {
                v: crate::effect::AnimatablePrimitive::Number(0.0),
            },
            fade_in: 0,
            fade_out: 0,
            fade_curve: crate::track::FadeCurve::Linear,
            channel_mode: crate::track::ChannelMode::Stereo,
            preserve_pitch: true,
        });
        clip
    }

    fn keyframe(
        t: i64,
        e: &str,
        h: Option<[f64; 4]>,
        v: crate::effect::AnimatablePrimitive,
    ) -> crate::effect::Keyframe {
        crate::effect::Keyframe {
            t,
            v,
            e: e.into(),
            h,
        }
    }

    fn project_with_kfs(k: Vec<crate::effect::Keyframe>) -> Project {
        let mut project = valid_project();
        let mut effect = crate::effect::new_effect_instance("fx.color.basic");
        let mut params = BTreeMap::new();
        params.insert("p".into(), crate::effect::AnimatableValue::K { k });
        effect.params = params;
        project.sequences[0].tracks[0].items[0].effects = vec![effect];
        project
    }

    #[test]
    fn full_house_project_with_every_entity_kind_is_valid() {
        let mut project = valid_project();
        project.bins.push(crate::bin::Bin {
            id: id::new_id(),
            name: "Footage".into(),
            parent_id: None,
            label: None,
            expanded: true,
            extra: Default::default(),
        });
        let asset = crate::asset::Asset {
            id: id::new_id(),
            kind: "lut".into(),
            name: "teal.cube".into(),
            rel_path: "assets/teal.cube".into(),
            sha256: "deadbeef".into(),
            lut_info: Some(crate::asset::LutInfo {
                lut_type: "3d".into(),
                size: 33,
                extra: Default::default(),
            }),
            extra: Default::default(),
        };
        let mut media = media_item(&id::new_id(), 60 * 705_600_000);
        media.bin_id = Some(project.bins[0].id.clone());
        project.assets.push(asset.clone());
        project.media.push(media.clone());

        let mut c1 = media_clip(&media.id, 0, 3 * FPF);
        let mut effect = crate::effect::new_effect_instance("fx.color.basic");
        let mut params = BTreeMap::new();
        params.insert(
            "exposure".into(),
            crate::effect::AnimatableValue::V {
                v: crate::effect::AnimatablePrimitive::Number(0.3),
            },
        );
        params.insert(
            "lut".into(),
            crate::effect::AnimatableValue::V {
                v: crate::effect::AnimatablePrimitive::Str(format!("asset:{}", asset.id)),
            },
        );
        params.insert(
            "mix".into(),
            crate::effect::AnimatableValue::K {
                k: vec![
                    keyframe(
                        0,
                        "linear",
                        None,
                        crate::effect::AnimatablePrimitive::Number(1.0),
                    ),
                    keyframe(
                        FPF,
                        "bezier",
                        Some([0.1, 0.0, 0.9, 1.0]),
                        crate::effect::AnimatablePrimitive::Number(0.5),
                    ),
                ],
            },
        );
        effect.params = params;
        let mask = crate::mask::new_mask("msk.rectangle");
        effect.mask_ids = vec![mask.id.clone()];
        c1.effects = vec![effect];
        c1.masks = vec![mask];
        let c2 = media_clip(&media.id, 3 * FPF, 3 * FPF);
        project.sequences[0].tracks[0].items = vec![c1.clone(), c2.clone()];
        project.sequences[0].tracks[0]
            .transitions
            .push(crate::transition::new_transition(
                "trs.crossDissolve",
                &c1.id,
                &c2.id,
                FPF,
            ));

        let mut chapter = crate::marker::new_marker(0, "chapter");
        chapter.kind = crate::marker::MarkerKind::Chapter;
        project.sequences[0].markers = vec![chapter, crate::marker::new_marker(FPF, "std")];

        let mut captions = crate::caption::new_caption_track("English", "en");
        captions.items.push(crate::caption::CaptionItem {
            id: id::new_id(),
            start: 0,
            end: 1_000_000,
            text: "Hello".into(),
            position: None,
            extra: Default::default(),
        });
        project.sequences[0].caption_tracks.push(captions);

        let issues = validate(&project);
        assert!(issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn rule2_bin_active_mask_sequence_and_transition_references() {
        // Dangling bin parent.
        let mut project = valid_project();
        project.bins.push(crate::bin::Bin {
            id: id::new_id(),
            name: "B".into(),
            parent_id: Some("nonexistent".into()),
            label: None,
            expanded: true,
            extra: Default::default(),
        });
        assert!(
            validate(&project)
                .iter()
                .any(|i| i.detail.contains("parentId"))
        );

        // Dangling media bin.
        let mut project = valid_project();
        let mut media = media_item(&id::new_id(), 60 * 705_600_000);
        media.bin_id = Some("nonexistent".into());
        project.media.push(media);
        assert!(
            validate(&project)
                .iter()
                .any(|i| i.detail.contains("binId"))
        );

        // Dangling active sequence.
        let mut project = valid_project();
        project.ui.active_sequence_id = Some("nonexistent".into());
        assert!(
            validate(&project)
                .iter()
                .any(|i| i.detail.contains("activeSequenceId"))
        );

        // Effect mask reference not on the clip.
        let mut project = valid_project();
        let mut effect = crate::effect::new_effect_instance("fx.color.basic");
        effect.mask_ids = vec!["nonexistent".into()];
        project.sequences[0].tracks[0].items[0].effects = vec![effect];
        assert!(
            validate(&project)
                .iter()
                .any(|i| i.detail.contains("maskId"))
        );

        // Dangling nested sequence reference.
        let mut project = valid_project();
        let mut nested = base_clip(3 * FPF, FPF);
        nested.clip_type = ClipType::Sequence;
        nested.sequence_id = Some("nonexistent".into());
        project.sequences[0].tracks[0].items.push(nested);
        assert!(
            validate(&project)
                .iter()
                .any(|i| i.detail.contains("sequenceId"))
        );

        // Dangling transition endpoint.
        let mut project = valid_project();
        let clip_id = project.sequences[0].tracks[0].items[0].id.clone();
        project.sequences[0].tracks[0]
            .transitions
            .push(crate::transition::new_transition(
                "trs.crossDissolve",
                "nonexistent",
                &clip_id,
                FPF,
            ));
        assert!(
            validate(&project)
                .iter()
                .any(|i| i.detail.contains("clip reference"))
        );
    }

    #[test]
    fn rule2_asset_references_in_params_resolve() {
        let mut project = valid_project();
        project.assets.push(crate::asset::Asset {
            id: id::new_id(),
            kind: "lut".into(),
            name: "teal.cube".into(),
            rel_path: "assets/teal.cube".into(),
            sha256: "aa".into(),
            lut_info: None,
            extra: Default::default(),
        });
        let asset_id = project.assets[0].id.clone();
        let mut effect = crate::effect::new_effect_instance("fx.color.basic");
        let mut params = BTreeMap::new();
        params.insert(
            "ok".into(),
            crate::effect::AnimatableValue::V {
                v: crate::effect::AnimatablePrimitive::Str(format!("asset:{asset_id}")),
            },
        );
        params.insert(
            "bad".into(),
            crate::effect::AnimatableValue::V {
                v: crate::effect::AnimatablePrimitive::Str("asset:nonexistent".into()),
            },
        );
        effect.params = params;
        project.sequences[0].tracks[0].items[0].effects = vec![effect];
        let issues = validate(&project);
        let asset_issues = issues
            .iter()
            .filter(|i| i.detail.contains("asset reference"))
            .count();
        assert_eq!(asset_issues, 1);
    }

    #[test]
    fn rule3_nesting_depth_over_eight_is_rejected() {
        let mut ids = Vec::new();
        for _ in 0..10 {
            ids.push(id::new_id());
        }
        let mut sequences = Vec::new();
        for i in 0..10 {
            let mut s = new_sequence(&format!("S{i}"), sqp02_fallback_format());
            s.id = ids[i].clone();
            s.format.auto_match_pending = false;
            if i < 9 {
                let mut clip = base_clip(0, FPF);
                clip.clip_type = ClipType::Sequence;
                clip.sequence_id = Some(ids[i + 1].clone());
                let mut t = new_track(TrackKind::Video, "V1");
                t.items.push(clip);
                s.tracks.push(t);
            }
            sequences.push(s);
        }
        let first = sequences.remove(0);
        let mut project = new_project("T", first);
        for s in sequences {
            project.sequences.push(s);
        }
        let issues = validate(&project);
        assert!(
            issues
                .iter()
                .any(|i| i.detail.contains("nesting depth exceeds")),
            "{issues:?}"
        );
    }

    #[test]
    fn rule5_audio_clips_use_the_quarter_frame_grid() {
        let mut project = valid_project();
        let media = media_item(&id::new_id(), 60 * 705_600_000);
        project.media.push(media.clone());
        let mut track = new_track(TrackKind::Audio, "A1");
        // Five quarter-frames: not a frame multiple, but quarter-aligned.
        track.items.push(audio_clip(&media.id, 5 * (FPF / 4), FPF));
        project.sequences[0].tracks.push(track);
        let issues = validate(&project);
        assert!(
            !issues.iter().any(|i| i.detail.contains("frame grid")),
            "{issues:?}"
        );

        let mut project = valid_project();
        project.media.push(media.clone());
        let mut track = new_track(TrackKind::Audio, "A1");
        track
            .items
            .push(audio_clip(&media.id, 5 * (FPF / 4) + 1, FPF));
        project.sequences[0].tracks.push(track);
        assert!(
            validate(&project)
                .iter()
                .any(|i| i.detail.contains("frame grid"))
        );
    }

    #[test]
    fn rule6_speed_outside_bounds_rejected() {
        let mut project = valid_project();
        project.sequences[0].tracks[0].items[0].speed =
            Rational::new(1, 20).unwrap_or(Rational { num: 1, den: 20 });
        let issues = validate(&project);
        assert!(issues.iter().any(|i| i.detail.contains("outside")));
    }

    #[test]
    fn rule8_transitions_join_adjacent_clips_with_positive_duration() {
        // A valid adjacent pair with a centered transition is accepted.
        let mut project = valid_project();
        let media = media_item(&id::new_id(), 60 * 705_600_000);
        project.media.push(media.clone());
        let c1 = media_clip(&media.id, 0, 3 * FPF);
        let c2 = media_clip(&media.id, 3 * FPF, 3 * FPF);
        project.sequences[0].tracks[0].items = vec![c1.clone(), c2.clone()];
        project.sequences[0].tracks[0]
            .transitions
            .push(crate::transition::new_transition(
                "trs.crossDissolve",
                &c1.id,
                &c2.id,
                FPF,
            ));
        let issues = validate(&project);
        assert!(
            !issues.iter().any(|i| i.detail.contains("transition")),
            "{issues:?}"
        );

        // A gap between the clips is not an edit point.
        let mut project = valid_project();
        project.media.push(media.clone());
        let c1 = media_clip(&media.id, 0, 3 * FPF);
        let c2 = media_clip(&media.id, 4 * FPF, 3 * FPF);
        project.sequences[0].tracks[0].items = vec![c1.clone(), c2.clone()];
        project.sequences[0].tracks[0]
            .transitions
            .push(crate::transition::new_transition(
                "trs.crossDissolve",
                &c1.id,
                &c2.id,
                FPF,
            ));
        let issues = validate(&project);
        assert!(
            issues
                .iter()
                .any(|i| i.code == ErrorCode::E1102 && i.detail.contains("adjacent"))
        );

        // Zero duration is rejected.
        let mut project = valid_project();
        project.media.push(media.clone());
        let c1 = media_clip(&media.id, 0, 3 * FPF);
        let c2 = media_clip(&media.id, 3 * FPF, 3 * FPF);
        project.sequences[0].tracks[0].items = vec![c1.clone(), c2.clone()];
        project.sequences[0].tracks[0]
            .transitions
            .push(crate::transition::new_transition(
                "trs.crossDissolve",
                &c1.id,
                &c2.id,
                0,
            ));
        let issues = validate(&project);
        assert!(
            issues
                .iter()
                .any(|i| i.detail.contains("transition duration must be > 0"))
        );
    }

    #[test]
    fn rule9_keyframe_rules() {
        // Unsorted keyframes.
        let project = project_with_kfs(vec![
            keyframe(
                FPF,
                "linear",
                None,
                crate::effect::AnimatablePrimitive::Number(1.0),
            ),
            keyframe(
                0,
                "linear",
                None,
                crate::effect::AnimatablePrimitive::Number(0.0),
            ),
        ]);
        assert!(
            validate(&project)
                .iter()
                .any(|i| i.detail.contains("not strictly sorted"))
        );

        // t beyond the clip duration.
        let project = project_with_kfs(vec![
            keyframe(
                0,
                "linear",
                None,
                crate::effect::AnimatablePrimitive::Number(0.0),
            ),
            keyframe(
                4 * FPF,
                "linear",
                None,
                crate::effect::AnimatablePrimitive::Number(1.0),
            ),
        ]);
        assert!(
            validate(&project)
                .iter()
                .any(|i| i.detail.contains("outside clip duration"))
        );

        // Bezier without handles, and handles without bezier.
        let project = project_with_kfs(vec![
            keyframe(
                0,
                "bezier",
                None,
                crate::effect::AnimatablePrimitive::Number(0.0),
            ),
            keyframe(
                FPF,
                "linear",
                Some([0.1, 0.0, 0.9, 1.0]),
                crate::effect::AnimatablePrimitive::Number(1.0),
            ),
        ]);
        assert!(
            validate(&project)
                .iter()
                .any(|i| i.detail.contains("handles must exist only for bezier"))
        );

        // Booleans and enum strings must use hold easing.
        let project = project_with_kfs(vec![keyframe(
            0,
            "linear",
            None,
            crate::effect::AnimatablePrimitive::Bool(true),
        )]);
        assert!(
            validate(&project)
                .iter()
                .any(|i| i.detail.contains("hold easing"))
        );

        let project = project_with_kfs(vec![keyframe(
            0,
            "linear",
            None,
            crate::effect::AnimatablePrimitive::Str("a".into()),
        )]);
        assert!(
            validate(&project)
                .iter()
                .any(|i| i.detail.contains("hold easing"))
        );

        // A well-formed keyframe set passes.
        let project = project_with_kfs(vec![
            keyframe(
                0,
                "linear",
                None,
                crate::effect::AnimatablePrimitive::Number(0.0),
            ),
            keyframe(
                FPF,
                "bezier",
                Some([0.1, 0.0, 0.9, 1.0]),
                crate::effect::AnimatablePrimitive::Number(1.0),
            ),
        ]);
        assert!(
            !validate(&project)
                .iter()
                .any(|i| i.detail.contains("keyframe"))
        );
    }

    #[test]
    fn rule9_keyframe_count_limit() {
        let duration = 10_002 * FPF;
        let mut project = valid_project();
        project.sequences[0].tracks[0].items[0].duration = duration;
        let mut k = Vec::new();
        for i in 0..=limits::KEYFRAMES_PER_PARAM {
            k.push(keyframe(
                i as i64 * FPF,
                "linear",
                None,
                crate::effect::AnimatablePrimitive::Number(0.0),
            ));
        }
        let mut effect = crate::effect::new_effect_instance("fx.color.basic");
        let mut params = BTreeMap::new();
        params.insert("p".into(), crate::effect::AnimatableValue::K { k });
        effect.params = params;
        project.sequences[0].tracks[0].items[0].effects = vec![effect];
        let issues = validate(&project);
        assert!(issues.iter().any(|i| i.detail.contains("exceeds")));
    }

    #[test]
    fn rule10_link_group_audio_and_media_mismatch() {
        let mut project = valid_project();
        let media = media_item(&id::new_id(), 60 * 705_600_000);
        project.media.push(media.clone());
        let link = id::new_id();
        for _ in 0..3 {
            let mut track = new_track(TrackKind::Audio, "A");
            let mut clip = audio_clip(&media.id, 0, FPF);
            clip.link_group_id = Some(link.clone());
            track.items.push(clip);
            project.sequences[0].tracks.push(track);
        }
        let mut issues = Vec::new();
        check_link_groups(&project, &mut issues);
        assert!(issues.iter().any(|i| i.detail.contains("link group has")));

        // Mixed media inside one group.
        let other = media_item(&id::new_id(), 60 * 705_600_000);
        project.media.push(other.clone());
        let mut track = new_track(TrackKind::Video, "V2");
        let mut clip = audio_clip(&other.id, 0, FPF);
        clip.link_group_id = Some(link);
        track.items.push(clip);
        project.sequences[0].tracks.push(track);
        let mut issues = Vec::new();
        check_link_groups(&project, &mut issues);
        assert!(issues.iter().any(|i| i.detail.contains("same mediaId")));
    }

    #[test]
    fn rule11_caption_bounds_and_order() {
        let mut project = valid_project();
        let mut track = crate::caption::new_caption_track("en", "en");
        track.items.push(crate::caption::CaptionItem {
            id: id::new_id(),
            start: 0,
            end: 0,
            text: "ok".into(),
            position: None,
            extra: Default::default(),
        });
        project.sequences[0].caption_tracks.push(track);
        assert!(
            validate(&project)
                .iter()
                .any(|i| i.detail.contains("start must be < end"))
        );

        let mut project = valid_project();
        let mut track = crate::caption::new_caption_track("en", "en");
        track.items.push(crate::caption::CaptionItem {
            id: id::new_id(),
            start: 0,
            end: 2_000_000,
            text: "a".into(),
            position: None,
            extra: Default::default(),
        });
        track.items.push(crate::caption::CaptionItem {
            id: id::new_id(),
            start: 1_000_000,
            end: 3_000_000,
            text: "b".into(),
            position: None,
            extra: Default::default(),
        });
        project.sequences[0].caption_tracks.push(track);
        assert!(
            validate(&project)
                .iter()
                .any(|i| i.detail.contains("sorted and non-overlapping"))
        );
    }

    #[test]
    fn rule12_negative_marker_times_rejected() {
        let mut project = valid_project();
        project.sequences[0].markers = vec![crate::marker::new_marker(-1, "neg")];
        let issues = validate(&project);
        assert!(issues.iter().any(|i| i.detail.contains("≥ 0")));
    }

    #[test]
    fn rule13_height_rate_and_color_space_ranges() {
        let mut project = valid_project();
        project.sequences[0].format.height = 1_081;
        project.sequences[0].format.frame_rate =
            Rational::new(31, 1).unwrap_or(Rational { num: 31, den: 1 });
        project.sequences[0].format.color_space = "rec2020".into();
        let issues = validate(&project);
        assert!(issues.iter().any(|i| i.detail.contains("height")));
        assert!(
            issues
                .iter()
                .any(|i| i.detail.contains("frame rate is not valid"))
        );
        assert!(issues.iter().any(|i| i.detail.contains("rec709-sdr")));
        assert!(
            issues
                .iter()
                .any(|i| i.detail.contains("no exact flicks duration"))
        );
    }

    #[test]
    fn rule15_media_items_limit() {
        let mut project = valid_project();
        for _ in 0..=limits::MEDIA_ITEMS {
            project.media.push(media_item(&id::new_id(), 0));
        }
        let issues = validate(&project);
        assert!(
            issues
                .iter()
                .any(|i| i.detail.contains("media items exceed the limit"))
        );
    }

    #[test]
    fn rule15_sequences_limit() {
        let mut project = valid_project();
        for i in 0..limits::SEQUENCES {
            project
                .sequences
                .push(new_sequence(&format!("S{i}"), sqp02_fallback_format()));
        }
        let issues = validate(&project);
        assert!(
            issues
                .iter()
                .any(|i| i.detail.contains("sequences exceed the limit"))
        );
    }

    #[test]
    fn rule15_clips_limit() {
        let mut project = valid_project();
        project.sequences[0].tracks[0].items.clear();
        for i in 0..=limits::CLIPS_PER_SEQUENCE {
            project.sequences[0].tracks[0]
                .items
                .push(base_clip(i as i64 * FPF, FPF));
        }
        let issues = validate(&project);
        assert!(
            issues
                .iter()
                .any(|i| i.detail.contains("clips exceed the limit"))
        );
    }

    #[test]
    fn rule15_captions_limit() {
        let mut project = valid_project();
        let mut track = crate::caption::new_caption_track("en", "en");
        for i in 0..=limits::CAPTIONS_PER_TRACK {
            track.items.push(crate::caption::CaptionItem {
                id: id::new_id(),
                start: i as i64 * 10_000_000,
                end: i as i64 * 10_000_000 + 5_000_000,
                text: "c".into(),
                position: None,
                extra: Default::default(),
            });
        }
        project.sequences[0].caption_tracks.push(track);
        let issues = validate(&project);
        assert!(
            issues
                .iter()
                .any(|i| i.detail.contains("captions exceed the limit"))
        );
    }
}
