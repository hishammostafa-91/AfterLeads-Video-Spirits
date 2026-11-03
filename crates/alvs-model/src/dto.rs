//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! DTO surface (04 §4.2): the TS-exporting types re-exported under the
//! canonical `alvs_model::dto` path for `app/src/ipc/commands.ts` wrappers.

pub use crate::asset::{Asset, LutInfo};
pub use crate::bin::Bin;
pub use crate::caption::{CaptionBox, CaptionItem, CaptionStyle, CaptionTrack};
pub use crate::edit::{ParamIntrinsic, ParamTarget};
pub use crate::effect::{AnimatablePrimitive, AnimatableValue, EffectInstance, Keyframe};
pub use crate::marker::{Marker, MarkerKind};
pub use crate::media::{
    Derived, Fingerprint, ImageSequence, MediaItem, MediaKind, Optimized, Probe, ProbeStream,
    Stabilized,
};
pub use crate::patch::{DirtyRange, EntityMaps, EntityPatch, RemoveMaps};
pub use crate::project::{
    Project, ProjectSettings, SavedBy, SourceViewer, TimelineScroll, ViewState,
};
pub use crate::sequence::{MasterAudio, Sequence, SequenceFormat};
pub use crate::title::{
    TextAlign, TitleAnimation, TitleBox, TitleDocument, TitleLayer, TitleStroke,
};
pub use crate::track::{
    AudioClipSettings, ChannelMode, Clip, ClipType, Crop, Deinterlace, FadeCurve, GeneratorParams,
    Intrinsic, Track, TrackKind, Transform,
};
pub use crate::transition::{Transition, TransitionAlignment};
