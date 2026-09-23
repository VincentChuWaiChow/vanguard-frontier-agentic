//! Audit log data models — entries and event types.

use serde::{Deserialize, Serialize};

/// A single append-only entry in the audit log.
///
/// Entries form a SHA-256 hash chain for tamper detection. The recipe for each
/// row is recorded in `audit_log.hash_version`; see
/// `AuditLogger::compute_hash` (v1) and `AuditLogger::compute_hash_v2` (v2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Auto-increment primary key from SQLite.
    pub id: i64,
    /// ISO 8601 timestamp with millisecond precision (e.g. `"2025-01-01T00:00:00.000Z"`).
    pub timestamp: String,
    /// Discriminator for the kind of event recorded.
    pub event_type: AuditEventType,
    /// Asset or workspace identifier — what the event is about.
    pub subject: String,
    /// Structured detail blob; arbitrary JSON payload.
    pub details: serde_json::Value,
    /// Who initiated the action: `"system"`, `"headless"`, or a user identifier.
    pub operator: String,
    /// SHA-256 chain link over this row's fields and `prev_hash`, computed by the
    /// recipe named in `audit_log.hash_version`.
    pub entry_hash: String,
    /// Hash of the immediately preceding entry (empty string for the first entry).
    pub prev_hash: String,
}

/// Discriminates the kind of event recorded in an [`AuditEntry`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    PolicyEvaluation,
    Promotion,
    InstallationDetected,
    DriftDetected,
    ViolationResolved,
    OperatorAction,
    GateExecution,
    ConfigChange,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry() -> AuditEntry {
        AuditEntry {
            id: 1,
            timestamp: "2025-01-01T00:00:00.000Z".to_string(),
            event_type: AuditEventType::PolicyEvaluation,
            subject: "prod-workspace".to_string(),
            details: serde_json::json!({"rule": "require-security-scanner", "passed": false}),
            operator: "headless".to_string(),
            entry_hash: "abc123".to_string(),
            prev_hash: "".to_string(),
        }
    }

    #[test]
    fn audit_entry_round_trip() {
        let entry = make_entry();
        let json = serde_json::to_string(&entry).expect("serialize");
        let decoded: AuditEntry = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.id, 1);
        assert_eq!(decoded.timestamp, "2025-01-01T00:00:00.000Z");
        assert_eq!(decoded.event_type, AuditEventType::PolicyEvaluation);
        assert_eq!(decoded.subject, "prod-workspace");
        assert_eq!(decoded.operator, "headless");
        assert_eq!(decoded.entry_hash, "abc123");
        assert_eq!(decoded.prev_hash, "");
    }

    #[test]
    fn audit_event_type_round_trip() {
        let variants = [
            AuditEventType::PolicyEvaluation,
            AuditEventType::Promotion,
            AuditEventType::InstallationDetected,
            AuditEventType::DriftDetected,
            AuditEventType::ViolationResolved,
            AuditEventType::OperatorAction,
            AuditEventType::GateExecution,
            AuditEventType::ConfigChange,
        ];
        for variant in &variants {
            let json = serde_json::to_string(variant).expect("serialize");
            let decoded: AuditEventType = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&decoded, variant);
        }
    }

    #[test]
    fn audit_event_type_serialized_strings() {
        assert_eq!(
            serde_json::to_string(&AuditEventType::PolicyEvaluation).unwrap(),
            "\"policy_evaluation\""
        );
        assert_eq!(
            serde_json::to_string(&AuditEventType::Promotion).unwrap(),
            "\"promotion\""
        );
        assert_eq!(
            serde_json::to_string(&AuditEventType::InstallationDetected).unwrap(),
            "\"installation_detected\""
        );
        assert_eq!(
            serde_json::to_string(&AuditEventType::DriftDetected).unwrap(),
            "\"drift_detected\""
        );
        assert_eq!(
            serde_json::to_string(&AuditEventType::ViolationResolved).unwrap(),
            "\"violation_resolved\""
        );
        assert_eq!(
            serde_json::to_string(&AuditEventType::OperatorAction).unwrap(),
            "\"operator_action\""
        );
        assert_eq!(
            serde_json::to_string(&AuditEventType::GateExecution).unwrap(),
            "\"gate_execution\""
        );
        assert_eq!(
            serde_json::to_string(&AuditEventType::ConfigChange).unwrap(),
            "\"config_change\""
        );
    }

    #[test]
    fn audit_entry_details_is_arbitrary_json() {
        let entry = AuditEntry {
            id: 42,
            timestamp: "2025-06-01T12:00:00.000Z".to_string(),
            event_type: AuditEventType::GateExecution,
            subject: "validate:lint".to_string(),
            details: serde_json::json!({
                "gate": "validate:lint",
                "exit_code": 0,
                "duration_ms": 1500,
            }),
            operator: "system".to_string(),
            entry_hash: "deadbeef".to_string(),
            prev_hash: "cafebabe".to_string(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let decoded: AuditEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.details["gate"], "validate:lint");
        assert_eq!(decoded.details["exit_code"], 0);
    }
}
