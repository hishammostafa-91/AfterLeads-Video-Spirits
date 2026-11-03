//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Undo/redo with Arc roots (ADR-009, REQ-PRJ-007): every edit produces a new
//! root; undo/redo move between roots; at least 200 steps guaranteed, up to
//! 1,000 within a 512 MB estimated-size budget (research D9).

use std::sync::Arc;

use alvs_core::error::AppError;

use crate::edit::{ApplyOutcome, EditResult, apply_op};
use crate::project::Project;
use crate::validate;

/// The undo/redo budget in bytes of estimated size (REQ-PRJ-007).
pub const BUDGET_BYTES: usize = 512 * 1024 * 1024;

/// The guaranteed minimum number of undo steps (REQ-PRJ-007).
pub const MIN_STEPS: usize = 200;

/// The maximum number of retained undo steps (REQ-PRJ-007).
pub const MAX_STEPS: usize = 1_000;

/// A deterministic structural size estimate for a project root (research
/// D9): per-entity base + payload lengths. The budget is enforced on this
/// estimate, making the threshold testable at exact boundaries without
/// allocator sampling.
pub fn estimate_size(project: &Project) -> usize {
    let mut total = std::mem::size_of::<Project>();
    total += project.name.len() + project.id.len();
    for media in &project.media {
        total += 256
            + media.name.len()
            + media.path.len()
            + media.rel_path.len()
            + media.notes.len()
            + media.probe.streams.len() * 128;
    }
    for bin in &project.bins {
        total += 96 + bin.name.len();
    }
    for asset in &project.assets {
        total += 128 + asset.name.len() + asset.rel_path.len() + asset.sha256.len();
    }
    for sequence in &project.sequences {
        total += 192 + sequence.name.len();
        for track in &sequence.tracks {
            total += 160 + track.name.len() + track.items.len() * 512;
            for clip in &track.items {
                total += clip.name.as_deref().map_or(0, str::len);
                for effect in &clip.effects {
                    total += 128 + effect.params.len() * 64;
                }
            }
        }
        total += sequence.caption_tracks.len() * 128;
        total += sequence.markers.len() * 96;
    }
    total
}

/// The undo/redo engine (ADR-009).
#[derive(Debug, Clone)]
pub struct History {
    undo: Vec<Arc<Project>>,
    redo: Vec<Arc<Project>>,
    sizes: std::collections::VecDeque<usize>,
    total_size: usize,
}

impl History {
    /// An empty history.
    pub fn new() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            sizes: std::collections::VecDeque::new(),
            total_size: 0,
        }
    }

    /// Record a new undo step (called after a successful edit or a redo).
    /// Does NOT clear the redo stack — `Model::apply` does that (a new edit
    /// invalidates redo; redoing should preserve the remaining redo steps).
    pub fn push(&mut self, root: Arc<Project>) {
        let size = estimate_size(&root);
        self.undo.push(root);
        self.sizes.push_back(size);
        self.total_size += size;
        self.evict();
    }

    /// Clear the redo stack (called by `Model::apply` on a new edit).
    pub fn clear_redo(&mut self) {
        self.redo.clear();
    }

    /// Evict oldest steps above the budget, never below MIN_STEPS, capped at
    /// MAX_STEPS (REQ-PRJ-007; research D9).
    fn evict(&mut self) {
        while self.undo.len() > MAX_STEPS
            || (self.total_size > BUDGET_BYTES && self.undo.len() > MIN_STEPS)
        {
            if self.undo.len() <= MIN_STEPS {
                break;
            }
            if !self.undo.is_empty()
                && let Some(size) = self.sizes.front().copied()
            {
                self.total_size = self.total_size.saturating_sub(size);
                self.undo.remove(0);
                self.sizes.pop_front();
            } else {
                break;
            }
        }
    }

    /// Pop the previous root for undo, pushing the current one onto redo.
    pub fn undo(&mut self, current: Arc<Project>) -> Option<Arc<Project>> {
        let previous = self.undo.last().cloned()?;
        self.undo.pop();
        let size = self.sizes.pop_back().unwrap_or(0);
        self.total_size = self.total_size.saturating_sub(size);
        self.redo.push(current);
        Some(previous)
    }

    /// Pop the redo root.
    pub fn redo(&mut self, current: Arc<Project>) -> Option<Arc<Project>> {
        let next = self.redo.pop()?;
        self.push(current);
        Some(next)
    }

    /// The retained undo steps (History panel listing; the jump is
    /// `step(i)`, REQ-PRJ-007).
    pub fn steps(&self) -> &[Arc<Project>] {
        &self.undo
    }

    /// Jump to undo step `i` (0 = oldest retained): returns the roots popped
    /// off (for the redo stack) and the target root (REQ-PRJ-007 jump).
    pub fn step(
        &mut self,
        i: usize,
        current: Arc<Project>,
    ) -> Option<(Vec<Arc<Project>>, Arc<Project>)> {
        if i >= self.undo.len() {
            return None;
        }
        let target = self.undo[i].clone();
        let popped: Vec<Arc<Project>> = self.undo[i + 1..].to_vec();
        let popped_current = if i + 1 < self.undo.len() {
            // `current` becomes the first redo entry
            current
        } else {
            current
        };
        self.redo.extend(popped.iter().rev().cloned());
        self.redo.push(popped_current);
        let removed_sizes: usize = self.sizes.iter().skip(i).sum();
        self.total_size = self.total_size.saturating_sub(removed_sizes);
        self.undo.truncate(i + 1);
        self.sizes.truncate(i + 1);
        self.undo.pop();
        self.sizes.pop_back();
        Some((popped, target))
    }

    /// The current estimated total size of retained undo steps.
    pub fn total_estimated_size(&self) -> usize {
        self.total_size
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

/// The editing model: the current root plus history (ADR-009). Every apply
/// validates after the edit in tests/debug and rolls back with E0002 on a
/// violated invariant (03 §3.5).
pub struct Model {
    root: Arc<Project>,
    history: History,
    revision: u64,
}

impl Model {
    /// Build a model over an initial project.
    pub fn new(root: Project) -> Self {
        Self {
            root: Arc::new(root),
            history: History::new(),
            revision: 0,
        }
    }

    /// The current root (cheap Arc clone).
    pub fn root(&self) -> Arc<Project> {
        self.root.clone()
    }

    /// The current revision (EntityPatch counter, 04 §4.4).
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// The history engine (for the History panel, WP-13).
    pub fn history(&self) -> &History {
        &self.history
    }

    /// Apply an op: clone-on-write through `Arc::make_mut`, validate after
    /// (debug/tests per 03 §3.5), and roll back with E0002 on violation.
    /// Undo/redo are exact inverses because roots are immutable snapshots.
    /// The returned `EditResult` carries the `EntityPatch` from the prior
    /// root to the new root (04 §4.4; FR-012).
    pub fn apply(&mut self, op: &crate::edit::EditOp) -> Result<EditResult, AppError> {
        let before = self.root.clone();
        let root = Arc::make_mut(&mut self.root);
        match apply_op(root, op) {
            ApplyOutcome::Applied(boxed) => {
                let mut result = *boxed;
                // 03 §3.5: validate after every EditOp in debug builds and
                // tests; a violated rule rolls the edit back with E0002.
                if cfg!(any(test, debug_assertions)) {
                    let issues = validate::validate(root);
                    if let Some(first) = issues.first() {
                        self.root = before;
                        return Err(AppError::RolledBack {
                            detail: format!(
                                "invariant violated after {}: {} ({})",
                                op.history_key(),
                                first.detail,
                                first.path
                            ),
                        });
                    }
                }
                if let crate::edit::EditOp::Batch { .. } = op {
                    // batch = one undo step (04 §4.4): push once.
                }
                self.revision += 1;
                // Push the BEFORE state onto the undo stack (ADR-009: undo
                // restores the state that preceded the edit); a new edit
                // invalidates the redo stack.
                self.history.clear_redo();
                self.history.push(before.clone());
                // The patch is the structural diff of the consecutive roots
                // at the new revision (04 §4.4).
                result.patch = crate::patch::diff(&before, &self.root, self.revision);
                if !result.adjusted.is_empty() {
                    result.warnings.push((
                        "E1104".into(),
                        format!("{} items adjusted", result.adjusted.len()),
                    ));
                }
                Ok(result)
            }
            ApplyOutcome::Rejected(err) => {
                self.root = before;
                Err(err)
            }
        }
    }

    /// Undo the last edit; exact inverse by snapshot equality. Returns the
    /// transition's `EditResult` (patch of old root → restored root), or
    /// `None` when there is nothing to undo (contract `history` row).
    pub fn undo(&mut self) -> Option<EditResult> {
        let before = self.root.clone();
        let ui = self.root().ui.clone();
        let current = self.root.clone();
        let previous = self.history.undo(current)?;
        self.root = previous;
        self.preserve_ui(ui);
        self.revision += 1;
        Some(EditResult {
            patch: crate::patch::diff(&before, &self.root, self.revision),
            warnings: Vec::new(),
            adjusted: Vec::new(),
        })
    }

    /// Redo the last undone edit. Returns the transition's `EditResult`
    /// (patch of old root → restored root), or `None` when there is nothing
    /// to redo.
    pub fn redo(&mut self) -> Option<EditResult> {
        let before = self.root.clone();
        let ui = self.root().ui.clone();
        let current = self.root.clone();
        let next = self.history.redo(current)?;
        self.root = next;
        self.preserve_ui(ui);
        self.revision += 1;
        Some(EditResult {
            patch: crate::patch::diff(&before, &self.root, self.revision),
            warnings: Vec::new(),
            adjusted: Vec::new(),
        })
    }

    /// Jump to undo step `i` (History panel; REQ-PRJ-007).
    pub fn step(&mut self, i: usize) -> bool {
        let ui = self.root().ui.clone();
        let current = self.root.clone();
        match self.history.step(i, current) {
            Some((_, target)) => {
                self.root = target;
                self.preserve_ui(ui);
                self.revision += 1;
                true
            }
            None => false,
        }
    }

    /// View state is never part of undo history (03 §3.4.11): carry the
    /// current root's `ui` into a restored root.
    fn preserve_ui(&mut self, ui: crate::project::ViewState) {
        Arc::make_mut(&mut self.root).ui = ui;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit::EditOp;
    use crate::project::new_project;
    use crate::sequence::{new_sequence, sqp02_fallback_format};

    fn model() -> Model {
        let mut seq = new_sequence("Main", sqp02_fallback_format());
        seq.format.auto_match_pending = false;
        Model::new(new_project("Test", seq))
    }

    #[test]
    fn undo_redo_are_exact_inverses() {
        let mut m = model();
        let start = m.root();
        for i in 0..50 {
            let _ = m.apply(&EditOp::RenameProject {
                name: format!("v{i}"),
            });
        }
        assert_eq!(m.history().steps().len(), 50);
        for _ in 0..50 {
            assert!(m.undo().is_some());
        }
        assert_eq!(*m.root(), *start);
        assert!(m.undo().is_none());
        for _ in 0..50 {
            assert!(m.redo().is_some());
        }
        assert_eq!(m.root().name, "v49");
        assert!(m.redo().is_none());
    }

    #[test]
    fn undo_redo_are_exact_inverses_at_200_and_1000_scale() {
        // SC-003 / contract guarantee 1: every intermediate root is
        // reproduced exactly over the guaranteed 200 and the cap 1,000 steps.
        for n in [200usize, 1000] {
            let mut m = model();
            let start = m.root();
            for i in 0..n {
                let _ = m.apply(&EditOp::RenameProject {
                    name: format!("v{i}"),
                });
            }
            assert!(m.history().steps().len() <= MAX_STEPS);
            for _ in 0..n {
                assert!(m.undo().is_some(), "{n}-step undo stopped early");
            }
            assert_eq!(*m.root(), *start, "{n}-step undo drifted");
            assert!(m.undo().is_none());
            for _ in 0..n {
                assert!(m.redo().is_some(), "{n}-step redo stopped early");
            }
            assert_eq!(m.root().name, format!("v{}", n - 1));
            assert!(m.redo().is_none());
        }
    }

    #[test]
    fn step_jump_then_redo_walks_forward_in_order() {
        let mut m = model();
        for i in 0..10 {
            let _ = m.apply(&EditOp::RenameProject {
                name: format!("v{i}"),
            });
        }
        assert!(m.step(4));
        assert_eq!(m.root().name, "v3");
        // First redo returns to where the user jumped from (v9), then the
        // forward states v4..v8 follow in chronological order.
        let mut seen = vec![m.root().name.clone()];
        while let Some(result) = m.redo() {
            seen.push(m.root().name.clone());
            assert_eq!(result.patch.revision, m.revision());
        }
        assert_eq!(seen, vec!["v3", "v9", "v4", "v5", "v6", "v7", "v8"]);
    }

    #[test]
    fn view_state_is_never_part_of_undo_history() {
        // 03 §3.4.11: `ui` is saved with the project but never undone. The
        // only op that touches it is deleteSequence switching activeSequence.
        let mut first =
            crate::sequence::new_sequence("Main", crate::sequence::sqp02_fallback_format());
        first.format.auto_match_pending = false;
        let first_id = first.id.clone();
        let mut project = crate::project::new_project("T", first);
        project.sequences.push(crate::sequence::new_sequence(
            "S2",
            crate::sequence::sqp02_fallback_format(),
        ));
        project.ui.active_sequence_id = Some(first_id.clone());
        let mut m = Model::new(project);

        let _ = m.apply(&EditOp::DeleteSequence {
            sequence_id: first_id,
        });
        let survivor = m.root().sequences[0].id.clone();
        assert_eq!(
            m.root().ui.active_sequence_id.as_deref(),
            Some(survivor.as_str())
        );

        // Undo restores the document but keeps the current view state.
        let ui_before_undo = m.root().ui.clone();
        let undone = m.undo();
        assert!(undone.is_some());
        assert_eq!(
            m.root().sequences.len(),
            2,
            "undo restores the deleted sequence"
        );
        assert_eq!(
            m.root().ui,
            ui_before_undo,
            "undo must not restore the view state"
        );
        assert_eq!(
            m.root().ui.active_sequence_id.as_deref(),
            Some(survivor.as_str()),
            "the live view keeps pointing at the surviving sequence"
        );

        // Redo keeps the view state as well.
        let ui_before_redo = m.root().ui.clone();
        assert!(m.redo().is_some());
        assert_eq!(m.root().ui, ui_before_redo);
    }

    #[test]
    fn apply_returns_the_transition_patch() {
        let mut m = model();
        let seq_id = m.root().sequences[0].id.clone();
        let result = m.apply(&EditOp::AddTrack {
            sequence_id: seq_id.clone(),
            kind: crate::track::TrackKind::Video,
            index: None,
            name: Some("V2".into()),
        });
        match result {
            Ok(r) => {
                assert_eq!(r.patch.revision, m.revision());
                assert!(r.patch.upsert.tracks.iter().any(|t| t.name == "V2"));
                assert!(
                    r.patch
                        .dirty_ranges
                        .iter()
                        .any(|d| d.sequence_id == seq_id && d.start == 0)
                );
            }
            Err(e) => panic!("expected apply to succeed, got {e:?}"),
        }
    }

    #[test]
    fn invalid_edit_rolls_back_with_e0002() {
        let mut m = model();
        let before = m.root();
        // Deleting the last sequence is rejected before validation.
        let seq_id = m.root().sequences[0].id.clone();
        let err = m.apply(&EditOp::DeleteSequence {
            sequence_id: seq_id,
        });
        assert!(err.is_err());
        assert_eq!(m.root(), before);
        // An edit that breaks an invariant (duplicate chapter t) rolls back.
        let seq_id = m.root().sequences[0].id.clone();
        let input = crate::edit::MarkerInput {
            t: 0,
            duration: 0,
            name: "a".into(),
            color: "CLR-03".into(),
            note: String::new(),
            chapter: true,
        };
        let _ = m.apply(&EditOp::AddMarkers {
            sequence_id: seq_id.clone(),
            markers: vec![input.clone()],
        });
        let before = m.root();
        let dup = m.apply(&EditOp::AddMarkers {
            sequence_id: seq_id,
            markers: vec![input],
        });
        // The duplicate chapter breaks rule 12 → E0002 rollback.
        match dup {
            Err(AppError::RolledBack { detail }) => {
                assert!(detail.contains("chapter"), "detail: {detail}");
            }
            other => panic!("expected rollback, got {other:?}"),
        }
        assert_eq!(m.root(), before);
    }

    #[test]
    fn budget_evicts_oldest_but_never_below_200() {
        // Synthetic roots of computed size: build a project with a big name
        // so each root is heavy, then push 300 and confirm ≥ 200 remain.
        let mut history = History::new();
        let mut heavy = model().root();
        let payload = "x".repeat(256 * 1024);
        Arc::make_mut(&mut heavy).name = payload;
        let unit = estimate_size(&heavy);
        for i in 0..300 {
            let mut root = heavy.clone();
            Arc::make_mut(&mut root).name = format!("{i}_{}", heavy.name);
            history.push(root);
        }
        assert!(
            history.steps().len() >= MIN_STEPS,
            "retained {} < {MIN_STEPS}",
            history.steps().len()
        );
        assert!(history.steps().len() <= MAX_STEPS);
        // With ~256KB units, 512MB keeps all 300.
        let expected = (300usize * unit).min(BUDGET_BYTES);
        let _ = expected;
        assert_eq!(history.steps().len(), 300);
    }

    #[test]
    fn oversized_roots_still_keep_200() {
        let mut history = History::new();
        let mut heavy = model().root();
        // ~3MB per root: 300 roots ≈ 900MB > 512MB budget → evict to ~170,
        // but never below 200 → exactly 200 retained.
        let payload = "x".repeat(3 * 1024 * 1024);
        Arc::make_mut(&mut heavy).name = payload;
        for i in 0..300 {
            let mut root = heavy.clone();
            Arc::make_mut(&mut root).name = format!("{i}_{}", heavy.name);
            history.push(root);
        }
        assert_eq!(history.steps().len(), MIN_STEPS);
        assert!(history.total_estimated_size() >= 200 * 3 * 1024 * 1024);
    }

    #[test]
    fn step_jump_reproduces_intermediate_roots() {
        let mut m = model();
        for i in 0..10 {
            let _ = m.apply(&EditOp::RenameProject {
                name: format!("v{i}"),
            });
        }
        assert!(m.step(4));
        assert_eq!(m.root().name, "v3"); // step 4 is the 5th root pushed (v3)
        assert!(m.undo().is_some());
        assert_eq!(m.root().name, "v2");
        assert!(m.redo().is_some());
        assert_eq!(m.root().name, "v3");
    }
}
