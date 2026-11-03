//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Serialization round-trip proof for every 03 §3.4 entity kind: value
//! equality through JSON, the 03 §3.2 conventions (camelCase keys, catalog
//! enum ids, 2-space JSON), and unknown-field survival through `extra`
//! (03 §3.2.3).

use std::collections::BTreeMap;

use alvs_core::rational::Rational;
use alvs_model::asset::{Asset, LutInfo};
use alvs_model::bin::Bin;
use alvs_model::caption::{CaptionItem, new_caption_track};
use alvs_model::effect::{AnimatablePrimitive, AnimatableValue, Keyframe, new_effect_instance};
use alvs_model::marker::{MarkerKind, new_marker};
use alvs_model::media::{Fingerprint, ImageSequence, MediaItem, MediaKind, Probe, ProbeStream};
use alvs_model::project::{Project, SourceViewer, ViewState, new_project};
use alvs_model::sequence::{MasterAudio, new_sequence};
use alvs_model::title::{TextAlign, TitleAnimation, TitleDocument, new_title_layer};
use alvs_model::track::{
    AudioClipSettings, ChannelMode, Clip, ClipType, Crop, Deinterlace, FadeCurve, Intrinsic,
    TrackKind, Transform, new_track,
};
use alvs_model::transition::{TransitionAlignment, new_transition};

fn v_num(v: f64) -> AnimatableValue {
    AnimatableValue::V {
        v: AnimatablePrimitive::Number(v),
    }
}
fn v_point(x: f64, y: f64) -> AnimatableValue {
    AnimatableValue::V {
        v: AnimatablePrimitive::Point([x, y]),
    }
}

/// A project covering every entity kind of 03 §3.4.
fn kitchen_sink() -> Project {
    let rate = || Rational::new(30, 1).unwrap_or(Rational { num: 30, den: 1 });

    let media = MediaItem {
        id: alvs_core::id::new_id(),
        kind: MediaKind::Video,
        name: "interview.mp4".into(),
        path: r"D:\Footage\interview.mp4".into(),
        rel_path: r"..\Footage\interview.mp4".into(),
        fingerprint: Fingerprint {
            size: 734_003_200,
            mtime_utc: "2026-10-30T08:12:00Z".into(),
            head_tail_sha256: "9a1b".into(),
            extra: BTreeMap::new(),
        },
        probe: Probe {
            duration_flicks: 1_270_080_000_000,
            container: "mov,mp4,m4a,3gp,3g2,mj2".into(),
            streams: vec![
                ProbeStream {
                    index: 0,
                    stream_type: "video".into(),
                    codec: "hevc".into(),
                    width: Some(3840),
                    height: Some(2160),
                    frame_rate: Some(Rational::new(30_000, 1001).unwrap_or(rate())),
                    vfr: Some(true),
                    bit_depth: Some(10),
                    chroma: Some("420".into()),
                    primaries: Some("bt2020".into()),
                    transfer: Some("arib-std-b67".into()),
                    matrix: Some("bt2020nc".into()),
                    range: Some("limited".into()),
                    hdr: Some("hlg".into()),
                    rotation: Some(90),
                    interlaced: Some(false),
                    channels: None,
                    layout: None,
                    sample_rate: None,
                    extra: BTreeMap::new(),
                },
                ProbeStream {
                    index: 1,
                    stream_type: "audio".into(),
                    codec: "aac".into(),
                    width: None,
                    height: None,
                    frame_rate: None,
                    vfr: None,
                    bit_depth: None,
                    chroma: None,
                    primaries: None,
                    transfer: None,
                    matrix: None,
                    range: None,
                    hdr: None,
                    rotation: None,
                    interlaced: None,
                    channels: Some(2),
                    layout: Some("stereo".into()),
                    sample_rate: Some(48_000),
                    extra: BTreeMap::new(),
                },
            ],
            extra: BTreeMap::new(),
        },
        image_sequence: Some(ImageSequence {
            pattern: "shot_%04d.png".into(),
            first: 1,
            count: 240,
            frame_rate: rate(),
            extra: BTreeMap::new(),
        }),
        bin_id: None,
        label: Some("CLR-01".into()),
        notes: "primary interview".into(),
        derived: Default::default(),
        added_utc: "2026-11-01T10:05:00Z".into(),
        extra: BTreeMap::new(),
    };

    let bin = Bin {
        id: alvs_core::id::new_id(),
        name: "Day 1".into(),
        parent_id: None,
        label: None,
        expanded: true,
        extra: BTreeMap::new(),
    };

    let asset = Asset {
        id: alvs_core::id::new_id(),
        kind: "lut".into(),
        name: "Film.cube".into(),
        rel_path: "assets/luts/abc.cube".into(),
        sha256: "cafe".into(),
        lut_info: Some(LutInfo {
            lut_type: "3d".into(),
            size: 33,
            extra: BTreeMap::new(),
        }),
        extra: BTreeMap::new(),
    };

    let clip_id = alvs_core::id::new_id();
    let second_clip_id = alvs_core::id::new_id();
    let link_group = alvs_core::id::new_id();
    let mut effect = new_effect_instance("fx.color.basic");
    effect.params.insert("exposure".into(), v_num(0.3));
    effect.params.insert(
        "saturation".into(),
        AnimatableValue::K {
            k: vec![
                Keyframe {
                    t: 0,
                    v: AnimatablePrimitive::Number(100.0),
                    e: "linear".into(),
                    h: None,
                },
                Keyframe {
                    t: 705_600_000,
                    v: AnimatablePrimitive::Number(130.0),
                    e: "bezier".into(),
                    h: Some([0.33, 0.0, 0.67, 1.0]),
                },
            ],
        },
    );
    effect.params.insert(
        "wheel".into(),
        AnimatableValue::V {
            v: AnimatablePrimitive::Wheel {
                x: 0.0,
                y: 0.1,
                luma: -0.05,
            },
        },
    );
    effect.params.insert(
        "lut".into(),
        AnimatableValue::V {
            v: AnimatablePrimitive::Str(format!("asset:{}", asset.id)),
        },
    );

    let clip = Clip {
        id: clip_id.clone(),
        clip_type: ClipType::Media,
        media_id: Some(media.id.clone()),
        stream_index: Some(0),
        start: 0,
        duration: 352_800_000,
        source_in: 70_560_000,
        speed: Rational::new(1, 1).unwrap_or(rate()),
        reverse: false,
        freeze_at: None,
        enabled: true,
        name: None,
        label: None,
        link_group_id: Some(link_group.clone()),
        group_id: None,
        intrinsic: Some(Intrinsic {
            transform: Transform {
                position: v_point(0.5, 0.5),
                scale: v_num(100.0),
                scale_x: v_num(100.0),
                scale_y: v_num(100.0),
                uniform_scale: true,
                rotation: v_num(0.0),
                anchor: v_point(0.5, 0.5),
                flip_h: false,
                flip_v: false,
            },
            crop: Crop {
                left: v_num(0.0),
                top: v_num(0.0),
                right: v_num(0.0),
                bottom: v_num(0.0),
                feather: v_num(0.0),
            },
            opacity: v_num(100.0),
            blend_mode: "normal".into(),
        }),
        audio: Some(AudioClipSettings {
            gain_db: 0.0,
            volume: v_num(0.0),
            pan: v_num(0.0),
            fade_in: 0,
            fade_out: 0,
            fade_curve: FadeCurve::ConstantPower,
            channel_mode: ChannelMode::Stereo,
            preserve_pitch: true,
        }),
        deinterlace: Deinterlace::Auto,
        effects: vec![effect],
        masks: vec![alvs_model::mask::new_mask("mask.rect")],
        markers: vec![new_marker(70_560_000, "good take")],
        title: None,
        generator: None,
        sequence_id: None,
        extra: BTreeMap::new(),
    };

    let title_clip = Clip {
        id: second_clip_id.clone(),
        clip_type: ClipType::Title,
        media_id: None,
        stream_index: None,
        start: 352_800_000,
        duration: 705_600_000,
        source_in: 0,
        speed: Rational::new(1, 1).unwrap_or(rate()),
        reverse: false,
        freeze_at: None,
        enabled: true,
        name: Some("Lower third".into()),
        label: None,
        link_group_id: None,
        group_id: None,
        intrinsic: None,
        audio: None,
        deinterlace: Deinterlace::Auto,
        effects: Vec::new(),
        masks: Vec::new(),
        markers: Vec::new(),
        title: Some(TitleDocument {
            layers: vec![{
                let mut layer = new_title_layer("مرحبا Hello", "Cairo");
                layer.align = TextAlign::Center;
                layer
            }],
            anim_in: Some(TitleAnimation {
                animation_id: "tan.fade".into(),
                duration: 352_800_000,
                easing: "cubicOut".into(),
            }),
            anim_out: None,
            template_id: Some("tpl.lowerThird.clean".into()),
            extra: BTreeMap::new(),
        }),
        generator: None,
        sequence_id: None,
        extra: BTreeMap::new(),
    };

    let mut track = new_track(TrackKind::Video, "V1");
    track.items = vec![clip, title_clip];
    track.transitions.push(new_transition(
        "tr.dissolve.cross",
        &clip_id,
        &second_clip_id,
        705_600_000,
    ));
    track.transitions[0].alignment = TransitionAlignment::Center;

    let mut audio_track = new_track(TrackKind::Audio, "A1");
    audio_track.items.push(Clip {
        id: alvs_core::id::new_id(),
        clip_type: ClipType::Media,
        media_id: Some(media.id.clone()),
        stream_index: Some(1),
        start: 0,
        duration: 352_800_000,
        source_in: 0,
        speed: Rational::new(1, 1).unwrap_or(rate()),
        reverse: false,
        freeze_at: None,
        enabled: true,
        name: None,
        label: None,
        link_group_id: Some(link_group),
        group_id: None,
        intrinsic: None,
        audio: Some(AudioClipSettings {
            gain_db: 0.0,
            volume: v_num(0.0),
            pan: v_num(0.0),
            fade_in: 0,
            fade_out: 0,
            fade_curve: FadeCurve::Linear,
            channel_mode: ChannelMode::LeftToBoth,
            preserve_pitch: true,
        }),
        deinterlace: Deinterlace::Auto,
        effects: Vec::new(),
        masks: Vec::new(),
        markers: Vec::new(),
        title: None,
        generator: None,
        sequence_id: None,
        extra: BTreeMap::new(),
    });

    let mut sequence = new_sequence("Main", alvs_model::sequence::sqp02_fallback_format());
    sequence.format.auto_match_pending = false;
    sequence.tracks = vec![track, audio_track];
    let mut chapter = new_marker(0, "Intro");
    chapter.kind = MarkerKind::Chapter;
    sequence.markers.push(chapter);
    let mut captions = new_caption_track("Arabic", "ar");
    captions.items.push(CaptionItem {
        id: alvs_core::id::new_id(),
        start: 0,
        end: 1_411_200_000,
        text: "مرحبا بكم".into(),
        position: None,
        extra: BTreeMap::new(),
    });
    sequence.caption_tracks.push(captions);
    sequence.in_point = Some(0);
    sequence.out_point = Some(1_411_200_000);
    sequence.master_audio = MasterAudio {
        volume_db: 0.0,
        effects: Vec::new(),
        extra: BTreeMap::new(),
    };

    let mut project = new_project("Interview Cut", sequence);
    project.media.push(media);
    project.bins.push(bin);
    project.assets.push(asset);
    project.ui = ViewState {
        active_sequence_id: Some(project.sequences[0].id.clone()),
        playheads: BTreeMap::from([(project.sequences[0].id.clone(), 70_560_000)]),
        timeline_zoom: BTreeMap::from([(project.sequences[0].id.clone(), 42.5)]),
        timeline_scroll: BTreeMap::new(),
        source_viewer: SourceViewer {
            media_id: None,
            r#in: None,
            out: None,
            playhead: 0,
        },
        selected_track_targets: BTreeMap::new(),
        extra: BTreeMap::new(),
    };
    project
}

#[test]
fn every_entity_round_trips_through_json() {
    let project = kitchen_sink();
    let json = serde_json::to_string(&project).unwrap_or_default();
    let back: Project = serde_json::from_str(&json).unwrap_or_else(|e| {
        panic!("deserialize failed: {e}");
    });
    assert_eq!(back, project);
}

#[test]
fn conventions_hold_camel_case_and_catalog_ids() {
    let json = serde_json::to_string(&kitchen_sink()).unwrap_or_default();
    for key in [
        "\"schemaVersion\"",
        "\"createdUtc\"",
        "\"savedBy\"",
        "\"defaultSequenceId\"",
        "\"relPath\"",
        "\"headTailSha256\"",
        "\"durationFlicks\"",
        "\"frameRate\"",
        "\"captionTracks\"",
        "\"linkGroupId\"",
        "\"blendMode\":\"normal\"",
        "\"colorSpace\":\"rec709-sdr\"",
        "\"effectId\":\"fx.color.basic\"",
        "\"transitionId\":\"tr.dissolve.cross\"",
        "\"label\":\"CLR-01\"",
        "\"color\":\"CLR-03\"",
        "\"channelMode\":\"leftToBoth\"",
        "\"fadeCurve\":\"constantPower\"",
    ] {
        assert!(
            json.contains(key),
            "missing convention key {key} in:\n{json}"
        );
    }
    // Pretty form uses 2-space indent (03 §3.2.1).
    let pretty = serde_json::to_string_pretty(&kitchen_sink()).unwrap_or_default();
    assert!(pretty.contains("\n  \"format\""));
}

#[test]
fn unknown_fields_survive_the_round_trip() {
    let mut value = serde_json::to_value(kitchen_sink()).unwrap_or(serde_json::Value::Null);
    // A field a newer version might write on the project and on a clip.
    value["futureTopLevel"] = serde_json::json!({ "v2": true });
    value["sequences"][0]["tracks"][0]["items"][0]["futureClipField"] = serde_json::json!(7);
    let back: Project = serde_json::from_value(value).unwrap_or_else(|e| panic!("load: {e}"));
    let resaved = serde_json::to_value(&back).unwrap_or(serde_json::Value::Null);
    assert_eq!(resaved["futureTopLevel"]["v2"], true, "project extra lost");
    assert_eq!(
        resaved["sequences"][0]["tracks"][0]["items"][0]["futureClipField"], 7,
        "clip extra lost"
    );
    // And the typed view still round-trips.
    let again: Project =
        serde_json::from_value(resaved.clone()).unwrap_or_else(|e| panic!("reload: {e}"));
    let _ = serde_json::to_value(&again).unwrap_or_default();
}

#[test]
fn keyframe_and_value_shapes_serialize_per_doc() {
    let project = kitchen_sink();
    let json = serde_json::to_value(&project).unwrap_or_default();
    let effect = &json["sequences"][0]["tracks"][0]["items"][0]["effects"][0];
    assert_eq!(effect["effectId"], "fx.color.basic");
    assert_eq!(effect["params"]["exposure"]["v"], 0.3);
    let kfs = &effect["params"]["saturation"]["k"];
    assert_eq!(kfs[0]["t"], 0);
    assert_eq!(kfs[0]["e"], "linear");
    assert_eq!(kfs[1]["h"][0], 0.33);
    assert_eq!(effect["params"]["wheel"]["v"]["luma"], -0.05);
    assert_eq!(
        effect["params"]["lut"]["v"],
        format!("asset:{}", project.assets[0].id)
    );
}
