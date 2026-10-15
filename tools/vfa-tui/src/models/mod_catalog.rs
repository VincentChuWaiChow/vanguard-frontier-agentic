use serde::{Deserialize, Serialize};

/// Declared capabilities copied from the canonical mod manifest, not containment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModFootprint {
    pub env_reads: Vec<String>,
    pub file_writes: Vec<String>,
    pub spawns: Vec<String>,
    pub egress_hosts: Vec<String>,
}

/// A generated mod catalog entry. Legacy catalogs may omit optional metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mod {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    /// Stored as u8; the loader enforces the narrower 0..=3 domain.
    pub reach_level: u8,
    pub events: Vec<String>,
    pub harnesses: Vec<String>,
    pub path: String,
    #[serde(default)]
    pub security_notes: Option<String>,
    /// None means not recorded, not an empty capability declaration.
    #[serde(default)]
    pub footprint: Option<ModFootprint>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn legacy_entry() -> serde_json::Value {
        json!({
            "id": "example-mod", "name": "Example Mod", "version": "1.0.0",
            "description": "A catalog test entry", "reach_level": 0,
            "events": ["PreToolUse"], "harnesses": ["codex"], "path": "mods/example-mod"
        })
    }

    #[test]
    fn mod_model_accepts_legacy_and_recorded_footprints() {
        let mut entry = legacy_entry();
        let legacy: Mod = serde_json::from_value(entry.clone()).unwrap();
        assert_eq!(legacy.footprint, None);
        assert_eq!(legacy.security_notes, None);
        entry["footprint"] = json!({
            "env_reads": ["EXAMPLE"], "file_writes": [], "spawns": [], "egress_hosts": []
        });
        entry["security_notes"] = json!("Declared footprint only");
        let recorded: Mod = serde_json::from_value(entry.clone()).unwrap();
        assert_eq!(recorded.footprint.as_ref().unwrap().env_reads, ["EXAMPLE"]);
        assert!(recorded.footprint.as_ref().unwrap().file_writes.is_empty());
        assert_eq!(serde_json::to_value(recorded).unwrap(), entry);
        entry["footprint"] = serde_json::Value::Null;
        entry["security_notes"] = serde_json::Value::Null;
        assert_eq!(serde_json::from_value::<Mod>(entry).unwrap(), legacy);
    }

    #[test]
    fn mod_model_rejects_missing_required_and_unknown_fields() {
        for field in [
            "id",
            "name",
            "version",
            "description",
            "reach_level",
            "events",
            "harnesses",
            "path",
        ] {
            let mut entry = legacy_entry();
            entry.as_object_mut().unwrap().remove(field);
            assert!(
                serde_json::from_value::<Mod>(entry).is_err(),
                "missing {field}"
            );
        }
        for field in ["type", "provider", "unknown"] {
            let mut entry = legacy_entry();
            entry[field] = json!("unexpected");
            assert!(
                serde_json::from_value::<Mod>(entry).is_err(),
                "unknown {field}"
            );
        }
        for footprint in [
            json!({"env_reads": []}),
            json!({"env_reads": [], "file_writes": [], "spawns": [], "egress_hosts": [], "unknown": []}),
            json!({"env_reads": [], "file_writes": "not-an-array", "spawns": [], "egress_hosts": []}),
        ] {
            let mut entry = legacy_entry();
            entry["footprint"] = footprint;
            assert!(serde_json::from_value::<Mod>(entry).is_err());
        }
    }

    #[test]
    fn mod_model_rejects_wrong_json_types() {
        for (field, value) in [
            ("id", json!(false)),
            ("reach_level", json!(-1)),
            ("reach_level", json!(1.5)),
            ("reach_level", json!(256)),
            ("events", json!("PreToolUse")),
            ("harnesses", json!([1])),
        ] {
            let mut entry = legacy_entry();
            entry[field] = value;
            assert!(
                serde_json::from_value::<Mod>(entry).is_err(),
                "wrong type {field}"
            );
        }
    }
}
