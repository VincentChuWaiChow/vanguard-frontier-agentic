//! Verified model / reasoning-effort matrix (`catalog/model-registry.json`).
//!
//! The registry is the fail-closed allowlist `scripts/model-policy.mjs`
//! validates every policy value against. The TUI reads it for one purpose
//! only: to offer the operator the *same* choices the script would accept,
//! instead of a free-text box plus a hardcoded effort union that drifts.
//!
//! Read-first principle: this module never classifies an arbitrary value into
//! a namespace and never decides whether a pin is legal — that is the script's
//! job, and duplicating its regex evaluation here is exactly the kind of logic
//! fork the TUI is meant to avoid. It only walks the already-declared
//! structure, so a choice the operator *picks* carries its namespace with it
//! and the effort narrowing follows from that, with no matching required.
//! Free-typed values fall back to the harness vocabulary and are left for the
//! script to accept or reject.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Whether a namespace enumerates its models (`closed`) or validates shape
/// only because the catalog is too large/volatile to list (`open`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Membership {
    Closed,
    Open,
}

/// Provider lifecycle state of a registered model. Absent means available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelStatus {
    #[default]
    Available,
    Retiring,
    Retired,
}

/// One entry in a closed namespace's verified allowlist.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryModel {
    pub id: String,
    /// Effort values this model supports. `None` inherits the namespace (or
    /// harness) vocabulary; `Some([])` is a non-reasoning model.
    #[serde(default)]
    pub reasoning_efforts: Option<Vec<String>>,
    pub last_verified: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub status: Option<ModelStatus>,
    #[serde(default)]
    pub retirement_date: Option<String>,
    #[serde(default)]
    pub successor: Option<String>,
}

impl RegistryModel {
    pub fn status(&self) -> ModelStatus {
        self.status.unwrap_or_default()
    }
}

/// A namespace groups model values that share a routing target and vocabulary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryNamespace {
    pub id: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Anchored regex the script uses to classify a value into this namespace.
    /// Carried for display/provenance only — never evaluated here.
    #[serde(rename = "match")]
    pub match_pattern: String,
    pub membership: Membership,
    #[serde(default)]
    pub model_provider: Option<String>,
    #[serde(default)]
    pub requires_provider_table: Option<bool>,
    #[serde(default)]
    pub reasoning_efforts: Option<Vec<String>>,
    #[serde(default)]
    pub reasoning_note: Option<String>,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub examples: Vec<String>,
    #[serde(default)]
    pub models: Vec<RegistryModel>,
}

/// Per-harness section of the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryHarness {
    /// Config key the harness projects reasoning effort into; `None` means the
    /// harness has no reasoning field at all.
    pub reasoning_key: Option<String>,
    pub reasoning_efforts: Vec<String>,
    #[serde(default)]
    pub reasoning_note: Option<String>,
    pub namespaces: Vec<RegistryNamespace>,
}

impl RegistryHarness {
    /// Whether this harness can express a reasoning effort at all.
    pub fn supports_reasoning(&self) -> bool {
        self.reasoning_key.is_some() && !self.reasoning_efforts.is_empty()
    }
}

/// Top-level structure of `catalog/model-registry.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelRegistry {
    pub manifest_version: u32,
    #[serde(default)]
    pub description: Option<String>,
    pub last_refreshed: String,
    pub harnesses: HashMap<String, RegistryHarness>,
}

/// One selectable entry in the TUI's model picker, already resolved to the
/// effort vocabulary that applies to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelChoice {
    pub model: String,
    pub namespace: String,
    /// Effort values valid for this model: the model's own list when it
    /// declares one, else the namespace's, else the harness vocabulary.
    pub efforts: Vec<String>,
    pub status: ModelStatus,
    /// True when the entry came from an open namespace's advisory `examples`
    /// rather than a verified allowlist — pickable, but not an allowlist hit.
    pub is_example: bool,
    pub retirement_date: Option<String>,
    pub successor: Option<String>,
    pub note: Option<String>,
}

impl ModelChoice {
    /// Short suffix describing lifecycle / provenance for the picker line.
    pub fn label_suffix(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        match self.status {
            ModelStatus::Retiring => parts.push(match &self.retirement_date {
                Some(d) => format!("retiring {d}"),
                None => "retiring".to_string(),
            }),
            ModelStatus::Retired => parts.push(match &self.successor {
                Some(s) => format!("retired -> {s}"),
                None => "retired".to_string(),
            }),
            ModelStatus::Available => {}
        }
        if self.is_example {
            parts.push("example".to_string());
        }
        if self.efforts.is_empty() {
            parts.push("no effort".to_string());
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!(" ({})", parts.join(", "))
        }
    }
}

/// Follow a `retired` model's `successor` chain within its namespace to the
/// entry the policy engine will actually project, so effort narrowing is read
/// off that model rather than the retired one.
///
/// Bounded by the namespace size and guarded against a cycle or a dangling
/// successor: either simply stops, returning the last entry reached, which is
/// the conservative choice — the script re-validates regardless.
fn resolve_successor<'a>(ns: &'a RegistryNamespace, model: &'a RegistryModel) -> &'a RegistryModel {
    let mut current = model;
    for _ in 0..ns.models.len() {
        if current.status() != ModelStatus::Retired {
            break;
        }
        let Some(successor) = current.successor.as_deref() else {
            break;
        };
        let Some(next) = ns.models.iter().find(|m| m.id == successor) else {
            break;
        };
        if next.id == current.id {
            break;
        }
        current = next;
    }
    current
}

impl ModelRegistry {
    pub fn harness(&self, harness: &str) -> Option<&RegistryHarness> {
        self.harnesses.get(harness)
    }

    /// Complete effort vocabulary a harness accepts, used when the operator
    /// free-types a model the registry does not enumerate.
    pub fn harness_efforts(&self, harness: &str) -> Vec<String> {
        self.harness(harness)
            .map(|h| h.reasoning_efforts.clone())
            .unwrap_or_default()
    }

    pub fn supports_reasoning(&self, harness: &str) -> bool {
        self.harness(harness)
            .is_some_and(|h| h.supports_reasoning())
    }

    /// Every model the operator can pick for `harness`, in registry order:
    /// each namespace's verified allowlist first, then the advisory examples
    /// of open namespaces. Duplicate ids keep their first (most specific)
    /// occurrence, mirroring the script's first-namespace-wins ordering.
    pub fn choices_for_harness(&self, harness: &str) -> Vec<ModelChoice> {
        let Some(hdef) = self.harness(harness) else {
            return Vec::new();
        };
        let mut out: Vec<ModelChoice> = Vec::new();
        let mut seen: Vec<String> = Vec::new();

        let push = |choice: ModelChoice, out: &mut Vec<ModelChoice>, seen: &mut Vec<String>| {
            if seen.iter().any(|m| m == &choice.model) {
                return;
            }
            seen.push(choice.model.clone());
            out.push(choice);
        };

        for ns in &hdef.namespaces {
            let ns_efforts = ns
                .reasoning_efforts
                .clone()
                .unwrap_or_else(|| hdef.reasoning_efforts.clone());

            for m in &ns.models {
                // A "retired" model is not what the engine validates against:
                // resolveAll in scripts/model-policy.mjs substitutes the
                // documented successor (chain-followed) and checks the effort
                // against *that* model. Mirror it, or the picker would offer
                // the retired entry's vocabulary for a pin the script resolves
                // elsewhere — offering a value the script then rejects, or
                // hiding one it would accept.
                let effective = resolve_successor(ns, m);
                let efforts = effective
                    .reasoning_efforts
                    .clone()
                    .unwrap_or_else(|| ns_efforts.clone());
                push(
                    ModelChoice {
                        model: m.id.clone(),
                        namespace: ns.id.clone(),
                        efforts,
                        status: m.status(),
                        is_example: false,
                        retirement_date: m.retirement_date.clone(),
                        successor: m.successor.clone(),
                        note: m.note.clone(),
                    },
                    &mut out,
                    &mut seen,
                );
            }

            // Open namespaces cannot enumerate their catalog; their `examples`
            // are advisory but they are exactly the values an operator wants
            // one keypress away (local Ollama tags, OpenRouter slugs).
            if ns.membership == Membership::Open {
                for ex in &ns.examples {
                    push(
                        ModelChoice {
                            model: ex.clone(),
                            namespace: ns.id.clone(),
                            efforts: ns_efforts.clone(),
                            status: ModelStatus::Available,
                            is_example: true,
                            retirement_date: None,
                            successor: None,
                            note: None,
                        },
                        &mut out,
                        &mut seen,
                    );
                }
            }
        }
        out
    }

    /// The choice matching an exact model id, if the registry enumerates it.
    /// Exact-match only — no namespace regex is evaluated, so a free-typed
    /// value simply returns `None` and the caller falls back to the harness
    /// vocabulary rather than guessing at a classification.
    pub fn choice_for(&self, harness: &str, model: &str) -> Option<ModelChoice> {
        self.choices_for_harness(harness)
            .into_iter()
            .find(|c| c.model == model)
    }

    /// Effort values to offer for `model` under `harness`. A registered model
    /// narrows to its own vocabulary; anything else gets the full harness
    /// vocabulary and is left for `scripts/model-policy.mjs` to adjudicate.
    pub fn efforts_for(&self, harness: &str, model: &str) -> Vec<String> {
        match self.choice_for(harness, model) {
            Some(choice) => choice.efforts,
            None => self.harness_efforts(harness),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> ModelRegistry {
        let json = r#"{
          "manifest_version": 1,
          "description": "test",
          "last_refreshed": "2026-09-05",
          "harnesses": {
            "codex": {
              "reasoning_key": "model_reasoning_effort",
              "reasoning_efforts": ["none", "minimal", "low", "medium", "high", "xhigh", "max"],
              "namespaces": [
                {
                  "id": "openai",
                  "match": "^(gpt-|o[0-9])[a-z0-9.-]*$",
                  "membership": "closed",
                  "model_provider": null,
                  "models": [
                    { "id": "gpt-6-astra", "reasoning_efforts": ["low", "high", "max"], "last_verified": "2026-09-05" },
                    { "id": "gpt-4.1-mini", "reasoning_efforts": [], "last_verified": "2026-09-05" },
                    { "id": "gpt-old", "last_verified": "2026-09-05", "status": "retiring", "retirement_date": "2026-12-11", "successor": "gpt-6-astra" }
                  ]
                },
                {
                  "id": "ollama",
                  "match": "^[a-z0-9][a-z0-9._-]*:[a-z0-9][a-z0-9._-]*$",
                  "membership": "open",
                  "model_provider": "ollama",
                  "requires_provider_table": true,
                  "reasoning_efforts": ["none", "low", "medium", "high", "max"],
                  "examples": ["qwen3:32b", "glm-5.3:cloud"]
                }
              ]
            },
            "cursor": {
              "reasoning_key": null,
              "reasoning_efforts": [],
              "namespaces": [
                { "id": "named", "match": "^[a-z0-9][a-z0-9.-]*$", "membership": "closed",
                  "models": [{ "id": "composer-2.5", "last_verified": "2026-09-05" }] }
              ]
            }
          }
        }"#;
        serde_json::from_str(json).expect("fixture parses")
    }

    #[test]
    fn parses_registry_shape() {
        let reg = fixture();
        assert_eq!(reg.manifest_version, 1);
        assert_eq!(reg.last_refreshed, "2026-09-05");
        assert_eq!(reg.harness("codex").unwrap().namespaces.len(), 2);
    }

    #[test]
    fn choices_include_closed_models_and_open_examples() {
        let ids: Vec<String> = fixture()
            .choices_for_harness("codex")
            .into_iter()
            .map(|c| c.model)
            .collect();
        assert_eq!(
            ids,
            vec![
                "gpt-6-astra",
                "gpt-4.1-mini",
                "gpt-old",
                "qwen3:32b",
                "glm-5.3:cloud"
            ]
        );
    }

    #[test]
    fn open_namespace_entries_are_marked_as_examples() {
        let reg = fixture();
        assert!(!reg.choice_for("codex", "gpt-6-astra").unwrap().is_example);
        let tag = reg.choice_for("codex", "qwen3:32b").unwrap();
        assert!(tag.is_example);
        assert_eq!(tag.namespace, "ollama");
    }

    #[test]
    fn effort_narrowing_prefers_model_then_namespace_then_harness() {
        let reg = fixture();
        // model-level list wins
        assert_eq!(
            reg.efforts_for("codex", "gpt-6-astra"),
            vec!["low", "high", "max"]
        );
        // namespace-level list wins where the model declares none
        assert_eq!(
            reg.efforts_for("codex", "qwen3:32b"),
            vec!["none", "low", "medium", "high", "max"]
        );
        // model with no list inherits the harness vocabulary
        assert_eq!(reg.efforts_for("codex", "gpt-old").len(), 7);
    }

    #[test]
    fn non_reasoning_model_offers_no_efforts() {
        assert!(fixture().efforts_for("codex", "gpt-4.1-mini").is_empty());
    }

    #[test]
    fn free_typed_model_falls_back_to_harness_vocabulary() {
        let reg = fixture();
        // Deliberately never classified against a namespace regex.
        assert!(reg.choice_for("codex", "deepseek-r1:70b").is_none());
        assert_eq!(reg.efforts_for("codex", "deepseek-r1:70b").len(), 7);
    }

    #[test]
    fn harness_without_reasoning_key_reports_unsupported() {
        let reg = fixture();
        assert!(reg.supports_reasoning("codex"));
        assert!(!reg.supports_reasoning("cursor"));
        assert!(!reg.supports_reasoning("nonexistent"));
        assert!(reg.choices_for_harness("nonexistent").is_empty());
    }

    #[test]
    fn lifecycle_surfaces_in_the_picker_label() {
        let reg = fixture();
        let retiring = reg.choice_for("codex", "gpt-old").unwrap();
        assert_eq!(retiring.status, ModelStatus::Retiring);
        assert!(retiring.label_suffix().contains("retiring 2026-12-11"));
        assert!(reg
            .choice_for("codex", "gpt-4.1-mini")
            .unwrap()
            .label_suffix()
            .contains("no effort"));
        assert!(reg
            .choice_for("codex", "qwen3:32b")
            .unwrap()
            .label_suffix()
            .contains("example"));
        assert_eq!(
            reg.choice_for("codex", "gpt-6-astra")
                .unwrap()
                .label_suffix(),
            ""
        );
    }

    #[test]
    fn retired_model_takes_its_successors_efforts() {
        // The engine projects the successor for a retired pin and validates the
        // effort against it, so the picker must narrow the same way.
        let json = r#"{
          "manifest_version": 1,
          "last_refreshed": "2026-09-05",
          "harnesses": {
            "codex": {
              "reasoning_key": "model_reasoning_effort",
              "reasoning_efforts": ["none", "low", "medium", "high", "xhigh", "max"],
              "namespaces": [
                {
                  "id": "openai",
                  "match": "^gpt-[a-z0-9.-]*$",
                  "membership": "closed",
                  "models": [
                    { "id": "gpt-dead", "reasoning_efforts": ["low"], "last_verified": "2026-09-05", "status": "retired", "successor": "gpt-mid" },
                    { "id": "gpt-mid", "reasoning_efforts": ["medium"], "last_verified": "2026-09-05", "status": "retired", "successor": "gpt-live" },
                    { "id": "gpt-live", "reasoning_efforts": ["none", "high", "max"], "last_verified": "2026-09-05" },
                    { "id": "gpt-loop", "reasoning_efforts": ["low"], "last_verified": "2026-09-05", "status": "retired", "successor": "gpt-loop" },
                    { "id": "gpt-dangling", "reasoning_efforts": ["low"], "last_verified": "2026-09-05", "status": "retired", "successor": "gpt-missing" }
                  ]
                }
              ]
            },
            "claude-code": { "reasoning_key": "effort", "reasoning_efforts": ["high"], "namespaces": [
              { "id": "alias", "match": "^opus$", "membership": "closed", "models": [{ "id": "opus", "last_verified": "2026-09-05" }] } ] },
            "cursor": { "reasoning_key": null, "reasoning_efforts": [], "namespaces": [
              { "id": "named", "match": "^[a-z0-9-]+$", "membership": "closed", "models": [{ "id": "composer-2", "last_verified": "2026-09-05" }] } ] }
          }
        }"#;
        let reg: ModelRegistry = serde_json::from_str(json).expect("parse");
        // Chain is followed to the end, not one hop.
        assert_eq!(
            reg.efforts_for("codex", "gpt-dead"),
            vec!["none", "high", "max"]
        );
        assert_eq!(
            reg.efforts_for("codex", "gpt-mid"),
            vec!["none", "high", "max"]
        );
        // A live model is untouched.
        assert_eq!(
            reg.efforts_for("codex", "gpt-live"),
            vec!["none", "high", "max"]
        );
        // A self-referential or dangling successor must terminate, not hang.
        assert_eq!(reg.efforts_for("codex", "gpt-loop"), vec!["low"]);
        assert_eq!(reg.efforts_for("codex", "gpt-dangling"), vec!["low"]);
        // The retired entry still renders as retired in the picker.
        assert!(reg
            .choice_for("codex", "gpt-dead")
            .unwrap()
            .label_suffix()
            .contains("retired -> gpt-mid"));
    }

    #[test]
    fn unknown_fields_are_rejected() {
        // deny_unknown_fields keeps the TUI honest when the generator adds a key.
        let json = r#"{ "id": "x", "last_verified": "2026-09-05", "surprise": true }"#;
        assert!(serde_json::from_str::<RegistryModel>(json).is_err());
    }

    #[test]
    fn parses_the_committed_registry() {
        // Guards against a registry change that the TUI's strict structs
        // would reject at runtime (the catalog would silently stop loading).
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../catalog/model-registry.json"
        );
        let data = std::fs::read_to_string(path).expect("committed registry is readable");
        let reg: ModelRegistry = serde_json::from_str(&data).expect("committed registry parses");
        for harness in ["codex", "claude-code", "cursor"] {
            assert!(
                !reg.choices_for_harness(harness).is_empty(),
                "{harness} should offer at least one model choice"
            );
        }
    }
}
