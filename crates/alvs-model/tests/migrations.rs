//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! Migration proofs against file fixtures (T-PRJ-007 suite, 03 §3.9.6;
//! research D12/F5): the v1 input/output pair round-trips through the driver
//! unchanged, deserializes as a `Project` with unknown fields kept in `extra`,
//! and a newer `schemaVersion` opens read-only (E1004 context) without data
//! loss. The synthetic-step machinery proofs live in `migrate.rs` (cfg(test)).

use alvs_model::migrate::{MigrationOutcome, migrate};

const V1_IN: &str = include_str!("fixtures/migrations/v1_in.json");
const V1_OUT: &str = include_str!("fixtures/migrations/v1_out.json");

#[test]
fn v1_fixture_pair_round_trips_through_the_driver() {
    let mut input: serde_json::Value =
        serde_json::from_str(V1_IN).unwrap_or_else(|e| panic!("fixture parse: {e}"));
    let outcome = migrate(&mut input, &[]).unwrap_or(MigrationOutcome::ReadOnlyNewer {
        found: 0,
        current: 0,
    });
    assert_eq!(outcome, MigrationOutcome::Migrated { from: 1, to: 1 });
    let expected: serde_json::Value =
        serde_json::from_str(V1_OUT).unwrap_or_else(|e| panic!("expected parse: {e}"));
    assert_eq!(
        input, expected,
        "the driver must leave a current-version file unchanged"
    );
}

#[test]
fn v1_fixture_deserializes_as_project_with_unknown_fields_kept() {
    let value: serde_json::Value =
        serde_json::from_str(V1_IN).unwrap_or_else(|e| panic!("fixture parse: {e}"));
    let project: alvs_model::project::Project =
        serde_json::from_value(value).unwrap_or_else(|e| panic!("project parse: {e}"));
    assert_eq!(project.name, "Migration Fixture");
    assert_eq!(project.sequences.len(), 1);
    // The unknown future field survives in the extra map (03 §3.2.3/§3.9.6).
    assert!(project.extra.contains_key("futureUnknownField"));
}

#[test]
fn newer_schema_version_fixture_is_read_only_with_fields_kept() {
    let mut value: serde_json::Value =
        serde_json::from_str(V1_IN).unwrap_or_else(|e| panic!("fixture parse: {e}"));
    if let Some(obj) = value.as_object_mut() {
        obj.insert("schemaVersion".into(), serde_json::json!(2));
    }
    let outcome = migrate(&mut value, &[]).unwrap_or(MigrationOutcome::Migrated { from: 0, to: 0 });
    assert_eq!(
        outcome,
        MigrationOutcome::ReadOnlyNewer {
            found: 2,
            current: 1
        }
    );
    assert!(value.get("futureUnknownField").is_some());
}
