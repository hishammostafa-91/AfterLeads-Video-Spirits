//! SPDX-License-Identifier: AGPL-3.0-or-later
//! Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
//! JSON Schema generation for the project file (ADR-008).

use schemars::Schema;
use schemars::schema_for;

use crate::project::Project;

/// Generate the JSON Schema of the `.alvs` project document from the types
/// (ADR-008: project files are JSON validated by a generated JSON Schema).
pub fn project_json_schema() -> Schema {
    schema_for!(Project)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_generates_and_describes_the_project() {
        let schema = project_json_schema();
        let root = schema
            .as_object()
            .unwrap_or_else(|| panic!("root schema must be an object"));
        let props = root
            .get("properties")
            .and_then(|p| p.as_object())
            .map(|m| m.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        for expected in [
            "format",
            "schemaVersion",
            "id",
            "name",
            "media",
            "bins",
            "assets",
            "sequences",
            "ui",
        ] {
            assert!(
                props.iter().any(|k| k == expected),
                "schema missing property {expected}; has {props:?}"
            );
        }
        // The schema is itself serializable JSON (ADR-008) and its object
        // graph names the entity types via $defs/$ref.
        let json = serde_json::to_value(&schema).unwrap_or(serde_json::Value::Null);
        assert!(json.is_object());
    }
}
