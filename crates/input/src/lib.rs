pub mod issue;
pub mod model;
pub mod project;
pub mod relations;
pub mod spec_gate;
pub mod views;

mod manager;
mod repair;
mod storage;
mod validate;

pub use manager::{
    load_input_index, load_input_manifest, load_input_snapshot, load_input_status,
    prepare_input_workspace, validate_input_workspace,
};
pub use model::{
    InputIndex, InputManifest, InputSnapshot, InputStatusSnapshot, INPUT_INDEX_VERSION,
    INPUT_MANIFEST_VERSION, INPUT_SNAPSHOT_VERSION, INPUT_STATUS_VERSION, INPUT_VERSION,
};
pub use repair::repair_input_workspace;
pub use validate::validate_input_snapshot;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        issue::{
            validate_agent_issue_permission, AgentRole, DisplayStatus, InputIssue, InputIssueModel,
            InputIssueStatus, InputPriority, InputRiskLevel, IssueCategory,
        },
        project::{InputProject, InputProjectStatus},
        relations::{InputIssueRelation, InputIssueRelationKind, InputIssueRelationsFile},
        spec_gate::{InputIssueGenerationMode, InputSpecApproval},
    };
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn prepare_creates_input_layout_manifest_index_and_relations() {
        let dir = tempdir().unwrap();

        let snapshot = prepare_input_workspace(dir.path()).unwrap();

        assert!(snapshot.ready);
        assert_eq!(snapshot.manifest.version, INPUT_MANIFEST_VERSION);
        assert_eq!(snapshot.index.version, INPUT_INDEX_VERSION);
        assert!(dir.path().join(".agentflow/input/manifest.json").is_file());
        assert!(dir.path().join(".agentflow/input/index.json").is_file());
        assert!(dir.path().join(".agentflow/input/intake").is_dir());
        assert!(dir.path().join(".agentflow/input/specs/drafts").is_dir());
        assert!(dir.path().join(".agentflow/input/specs/approved").is_dir());
        assert!(dir.path().join(".agentflow/input/specs/archive").is_dir());
        assert!(dir.path().join(".agentflow/input/projects").is_dir());
        assert!(dir.path().join(".agentflow/input/issues").is_dir());
        assert!(dir
            .path()
            .join(".agentflow/input/relations/issue-relations.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/input/relations/dependency-graph.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/input/views/active.json")
            .is_file());
        assert!(dir.path().join(".agentflow/spec").exists() == false);
        assert!(dir.path().join(".agentflow/goal-tree").exists() == false);
    }

    #[test]
    fn validate_fails_when_input_manifest_is_missing() {
        let dir = tempdir().unwrap();
        prepare_input_workspace(dir.path()).unwrap();
        fs::remove_file(dir.path().join(".agentflow/input/manifest.json")).unwrap();

        let snapshot = validate_input_workspace(dir.path()).unwrap();

        assert!(!snapshot.ready);
        assert!(snapshot
            .status
            .errors
            .iter()
            .any(|error| error.contains("manifest.json")));
    }

    #[test]
    fn high_risk_issue_requires_human_confirmation() {
        assert!(!InputRiskLevel::Low.requires_human_confirmation());
        assert!(!InputRiskLevel::Medium.requires_human_confirmation());
        assert!(InputRiskLevel::High.requires_human_confirmation());
    }

    #[test]
    fn direct_issue_requires_null_project_id_and_risk_level() {
        let dir = tempdir().unwrap();
        prepare_input_workspace(dir.path()).unwrap();
        let issue = InputIssue {
            issue_id: "iss-001".to_string(),
            issue_model: InputIssueModel::Direct,
            source_spec_id: "spec-001".to_string(),
            project_id: Some("proj-001".to_string()),
            title: "Direct issue should not point to project".to_string(),
            risk_level: InputRiskLevel::Medium,
            ..InputIssue::default()
        };
        fs::write(
            dir.path().join(".agentflow/input/issues/iss-001.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();

        let snapshot = validate_input_workspace(dir.path()).unwrap();

        assert!(!snapshot.ready);
        assert!(snapshot
            .status
            .errors
            .iter()
            .any(|error| { error.contains("direct issue iss-001 must use projectId = null") }));
    }

    #[test]
    fn project_issue_must_reference_existing_project() {
        let dir = tempdir().unwrap();
        prepare_input_workspace(dir.path()).unwrap();
        let issue = InputIssue {
            issue_id: "iss-001".to_string(),
            issue_model: InputIssueModel::Project,
            source_spec_id: "spec-001".to_string(),
            project_id: Some("missing-project".to_string()),
            title: "Project issue points to missing project".to_string(),
            risk_level: InputRiskLevel::Low,
            ..InputIssue::default()
        };
        fs::write(
            dir.path().join(".agentflow/input/issues/iss-001.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();

        let snapshot = validate_input_workspace(dir.path()).unwrap();

        assert!(!snapshot.ready);
        assert!(snapshot
            .status
            .errors
            .iter()
            .any(|error| error.contains("references missing project")));
    }

    #[test]
    fn project_issue_and_relations_validate_when_references_exist() {
        let dir = tempdir().unwrap();
        prepare_input_workspace(dir.path()).unwrap();
        let project = InputProject {
            project_id: "proj-001".to_string(),
            source_spec_id: "spec-001".to_string(),
            title: "Input model project".to_string(),
            status: InputProjectStatus::Planned,
            issue_ids: vec!["iss-001".to_string(), "iss-002".to_string()],
            ..InputProject::default()
        };
        let issue_one = InputIssue {
            issue_id: "iss-001".to_string(),
            issue_model: InputIssueModel::Project,
            source_spec_id: "spec-001".to_string(),
            project_id: Some("proj-001".to_string()),
            title: "First issue".to_string(),
            priority: InputPriority::Normal,
            risk_level: InputRiskLevel::Medium,
            ..InputIssue::default()
        };
        let issue_two = InputIssue {
            issue_id: "iss-002".to_string(),
            issue_model: InputIssueModel::Project,
            source_spec_id: "spec-001".to_string(),
            project_id: Some("proj-001".to_string()),
            title: "Second issue".to_string(),
            risk_level: InputRiskLevel::High,
            ..InputIssue::default()
        };
        let relations = InputIssueRelationsFile {
            relations: vec![InputIssueRelation {
                from_issue_id: "iss-002".to_string(),
                to_issue_id: "iss-001".to_string(),
                relation_type: InputIssueRelationKind::BlockedBy,
            }],
            ..InputIssueRelationsFile::default()
        };

        fs::write(
            dir.path().join(".agentflow/input/projects/proj-001.json"),
            serde_json::to_string_pretty(&project).unwrap(),
        )
        .unwrap();
        fs::write(
            dir.path().join(".agentflow/input/issues/iss-001.json"),
            serde_json::to_string_pretty(&issue_one).unwrap(),
        )
        .unwrap();
        fs::write(
            dir.path().join(".agentflow/input/issues/iss-002.json"),
            serde_json::to_string_pretty(&issue_two).unwrap(),
        )
        .unwrap();
        fs::write(
            dir.path()
                .join(".agentflow/input/relations/issue-relations.json"),
            serde_json::to_string_pretty(&relations).unwrap(),
        )
        .unwrap();

        let snapshot = validate_input_workspace(dir.path()).unwrap();

        assert!(snapshot.ready);
        assert_eq!(snapshot.status.summary.projects, 1);
        assert_eq!(snapshot.status.summary.issues, 2);
        assert_eq!(snapshot.status.summary.high_risk_issues, 1);
    }

    #[test]
    fn spec_approval_supports_direct_and_project_generation_modes() {
        let direct = InputSpecApproval {
            issue_generation_mode: InputIssueGenerationMode::Direct,
            ..InputSpecApproval::default()
        };
        let project = InputSpecApproval {
            issue_generation_mode: InputIssueGenerationMode::Project,
            ..InputSpecApproval::default()
        };

        assert_eq!(
            serde_json::to_value(direct).unwrap()["issueGenerationMode"],
            "direct"
        );
        assert_eq!(
            serde_json::to_value(project).unwrap()["issueGenerationMode"],
            "project"
        );
    }

    #[test]
    fn issue_model_does_not_expose_complex_automation_fields() {
        let issue = serde_json::to_value(InputIssue {
            issue_id: "iss-001".to_string(),
            source_spec_id: "spec-001".to_string(),
            title: "No automation fields".to_string(),
            risk_level: InputRiskLevel::Low,
            ..InputIssue::default()
        })
        .unwrap();

        assert!(issue.get("riskLevel").is_some());
        assert!(issue.get("automation").is_none());
        assert!(issue.get("humanGates").is_none());
        assert!(issue.get("prAutomation").is_none());
        assert!(issue.get("requiresHumanConfirmation").is_none());
        assert!(issue.get("allowedAgentActions").is_none());
        assert!(issue.get("blockedAgentActions").is_none());
        assert!(issue.get("riskReasons").is_none());
        assert!(issue.get("riskFactors").is_none());
    }

    #[test]
    fn display_status_serializes_as_kebab_case() {
        assert_eq!(
            serde_json::to_value(DisplayStatus::Backlog).unwrap(),
            "backlog"
        );
        assert_eq!(serde_json::to_value(DisplayStatus::Ready).unwrap(), "ready");
        assert_eq!(
            serde_json::to_value(DisplayStatus::InProgress).unwrap(),
            "in-progress"
        );
        assert_eq!(
            serde_json::to_value(DisplayStatus::Review).unwrap(),
            "review"
        );
        assert_eq!(serde_json::to_value(DisplayStatus::Done).unwrap(), "done");
        assert_eq!(
            serde_json::to_value(DisplayStatus::Cancel).unwrap(),
            "cancel"
        );
    }

    #[test]
    fn issue_serialization_preserves_display_status_field() {
        let issue = serde_json::to_value(InputIssue {
            issue_id: "iss-001".to_string(),
            source_spec_id: "spec-001".to_string(),
            title: "Ready issue".to_string(),
            status: InputIssueStatus::ReadyForExecute,
            display_status: DisplayStatus::Review,
            ..InputIssue::default()
        })
        .unwrap();

        assert_eq!(issue["status"], "ready-for-execute");
        assert_eq!(issue["displayStatus"], "review");
    }

    #[test]
    fn legacy_issue_without_display_status_defaults_to_backlog() {
        let issue: InputIssue = serde_json::from_value(serde_json::json!({
            "version": "input-issue.v1",
            "issueId": "iss-legacy",
            "issueModel": "direct",
            "sourceSpecId": "spec-001",
            "projectId": null,
            "title": "Legacy issue",
            "summary": "Legacy issue without displayStatus",
            "kind": "feature",
            "priority": "normal",
            "status": "ready-for-execute",
            "riskLevel": "low",
            "scope": [],
            "nonGoals": [],
            "acceptanceCriteria": [],
            "validationHints": [],
            "relations": {
                "blockedBy": [],
                "blocks": [],
                "related": [],
                "duplicateOf": null
            },
            "panel": {
                "snapshotId": null,
                "contextPackId": null
            },
            "system": {
                "createdBy": "fixture",
                "createdAt": 1,
                "updatedAt": 1,
                "path": ".agentflow/input/issues/iss-legacy.json",
                "revision": 1
            }
        }))
        .unwrap();

        assert_eq!(issue.display_status, DisplayStatus::Backlog);
        assert_eq!(
            DisplayStatus::from_input_status(&issue.status),
            DisplayStatus::Ready
        );
        assert_eq!(issue.issue_category, IssueCategory::Spec);
        assert_eq!(issue.required_agent_role, AgentRole::BuildAgent);
        validate_agent_issue_permission(&issue, &AgentRole::BuildAgent).unwrap();
        assert!(validate_agent_issue_permission(&issue, &AgentRole::AuditAgent).is_err());
    }

    #[test]
    fn audit_issue_requires_audit_agent() {
        let issue = InputIssue {
            issue_id: "audit-release-v0.1.0".to_string(),
            issue_category: IssueCategory::Audit,
            required_agent_role: AgentRole::AuditAgent,
            source_spec_id: "release-v0.1.0".to_string(),
            title: "Audit release".to_string(),
            ..InputIssue::default()
        };

        validate_agent_issue_permission(&issue, &AgentRole::AuditAgent).unwrap();
        assert!(validate_agent_issue_permission(&issue, &AgentRole::BuildAgent).is_err());
    }

    #[test]
    fn issue_category_role_mismatch_fails_validation() {
        let dir = tempdir().unwrap();
        prepare_input_workspace(dir.path()).unwrap();
        let issue = InputIssue {
            issue_id: "audit-release-v0.1.0".to_string(),
            issue_category: IssueCategory::Audit,
            required_agent_role: AgentRole::BuildAgent,
            source_spec_id: "release-v0.1.0".to_string(),
            title: "Audit release".to_string(),
            ..InputIssue::default()
        };
        fs::write(
            dir.path()
                .join(".agentflow/input/issues/audit-release-v0.1.0.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();

        let snapshot = validate_input_workspace(dir.path()).unwrap();

        assert!(!snapshot.ready);
        assert!(snapshot
            .status
            .errors
            .iter()
            .any(|error| error.contains("requires role audit-agent")));
    }

    #[test]
    fn input_index_records_issue_display_status() {
        let dir = tempdir().unwrap();
        prepare_input_workspace(dir.path()).unwrap();
        let issue = InputIssue {
            issue_id: "iss-001".to_string(),
            source_spec_id: "spec-001".to_string(),
            title: "Ready issue".to_string(),
            status: InputIssueStatus::ReadyForExecute,
            risk_level: InputRiskLevel::Low,
            ..InputIssue::default()
        };
        fs::write(
            dir.path().join(".agentflow/input/issues/iss-001.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();

        prepare_input_workspace(dir.path()).unwrap();
        let index = load_input_index(dir.path()).unwrap();
        let issue_entry = index
            .issues
            .iter()
            .find(|entry| entry.id == "iss-001")
            .unwrap();

        assert_eq!(issue_entry.status, "readyforexecute");
        assert_eq!(issue_entry.display_status, Some(DisplayStatus::Ready));
    }
}
