//! Port of the TypeScript `minimize artifacts` command
//! (frontend/packages/cli/src/commands/minimize.ts), so `.min.json` output is
//! the same resources whichever tool produced it.

use serde_json::{Map, Value};

/// Top-level `StructureDefinition` fields kept in the minimized form.
const STRUCTURE_DEFINITION_FIELDS: &[&str] = &[
    "id",
    "resourceType",
    "name",
    "title",
    "abstract",
    "url",
    "version",
    "context",
    "type",
    "baseDefinition",
    "kind",
    "derivation",
    "status",
];

/// Strips a `StructureDefinition` down to its snapshot (without element
/// mappings) and identifying fields. Other resources are left unchanged.
pub(crate) fn minimize_resource(resource: &mut Value) {
    if resource.get("resourceType").and_then(Value::as_str) != Some("StructureDefinition") {
        return;
    }
    let Value::Object(fields) = resource else {
        return;
    };

    let mut elements = fields
        .get_mut("snapshot")
        .and_then(|snapshot| snapshot.get_mut("element"))
        .map_or_else(|| Value::Array(Vec::new()), Value::take);
    for element in elements.as_array_mut().into_iter().flatten() {
        if let Value::Object(element) = element {
            element.remove("mapping");
        }
    }

    let mut minimized = Map::new();
    for name in STRUCTURE_DEFINITION_FIELDS {
        if let Some(value) = fields.remove(*name) {
            minimized.insert((*name).to_string(), value);
        }
    }
    let mut snapshot = Map::new();
    snapshot.insert("element".to_string(), elements);
    minimized.insert("snapshot".to_string(), Value::Object(snapshot));

    *fields = minimized;
}
