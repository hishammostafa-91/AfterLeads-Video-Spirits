// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! T029 (b): the SEC-04 containment guarantee (contracts G7) — every
//! write target of every service is inside an allowed root: application
//! writers stay inside the §3.8 roots, and project-folder artifacts
//! (sidecar, lock, backups, migration backups) are by construction
//! siblings of the user-chosen project file (05 §5.7: the project folder
//! and user-picked folders are per-operation roots).

use std::path::{Path, PathBuf};

use alvs_core::path::{PathDecision, PathOrigin, check_path};
use alvs_store::backup;
use alvs_store::lock;
use alvs_store::paths::AppPaths;
use alvs_store::project;

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "alvs-sec04-{tag}-{}",
        uuid::Uuid::now_v7().simple()
    ));
    std::fs::create_dir_all(&dir).unwrap_or(());
    dir
}

#[test]
fn application_writers_stay_inside_the_3_8_roots() {
    let base = scratch("approots");
    let (roaming, local, temp, home) = (
        base.join("roaming"),
        base.join("local"),
        base.join("tmp"),
        base.join("home"),
    );
    for d in [&roaming, &local, &temp, &home] {
        std::fs::create_dir_all(d).unwrap_or(());
    }
    let paths = match AppPaths::installed(&roaming, &local, &temp, &home) {
        Ok(p) => p,
        Err(e) => panic!("paths: {e}"),
    };
    // Materialize each §3.8 writer target and prove containment.
    let targets = [
        paths.settings_file(),
        paths.settings_backup(),
        paths.cache_db(),
        paths.logs_dir().join("app.log"),
        paths.session_marker(),
        paths.library_subfolder("effect-presets").join("x.json"),
    ];
    let roots = paths.containment_roots();
    for target in &targets {
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).unwrap_or(());
        }
        std::fs::write(target, b"x").unwrap_or(());
        assert!(
            matches!(
                check_path(target, PathOrigin::UserPicked, &roots),
                PathDecision::Allowed
            ),
            "{target:?} must be inside the containment roots"
        );
    }
    std::fs::remove_dir_all(&base).unwrap_or(());
}

#[test]
fn project_artifacts_are_siblings_of_the_user_chosen_project() {
    let project_dir = scratch("projfolder").join("deep").join("project.alvs");
    let parent = project_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    // Every artifact this package writes next to a project lives in the
    // project's own folder (03 §3.8 layout), so the user-picked folder is
    // the containment root by construction.
    for artifact in [
        project::autosave_path(&project_dir),
        lock::lock_path(&project_dir),
        backup::backups_dir(&project_dir),
        project_dir.with_extension("alvs.v0.bak"),
    ] {
        assert!(
            artifact.starts_with(&parent),
            "{artifact:?} must stay in the project folder"
        );
        // And concretely, with the folder materialized, SEC-04 allows it.
        std::fs::create_dir_all(&parent).unwrap_or(());
        if let Some(dir) = artifact.parent() {
            std::fs::create_dir_all(dir).unwrap_or(());
        }
        if artifact.extension().is_some() {
            std::fs::write(&artifact, b"x").unwrap_or(());
        } else {
            std::fs::create_dir_all(&artifact).unwrap_or(()); // Backups\
        }
        assert!(matches!(
            check_path(
                &artifact,
                PathOrigin::UserPicked,
                std::slice::from_ref(&parent)
            ),
            PathDecision::Allowed
        ));
    }
    std::fs::remove_dir_all(parent.parent().unwrap_or(&parent)).unwrap_or(());
}
