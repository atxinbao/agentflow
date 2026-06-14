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
    load_input_index, load_input_issue, load_input_manifest, load_input_project,
    load_input_snapshot, load_input_status, prepare_input_workspace,
    update_input_issue_branch_name, update_input_issue_latest_run, update_input_issue_status,
    update_input_project_status, validate_input_workspace,
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
            BUILD_AGENT_EXECUTION_PIPELINE_VERSION, BUILD_AGENT_PIPELINE_STAGE_IDS,
        },
        project::{InputProject, InputProjectStatus},
        relations::{
            InputDependencyGraph, InputIssueRelation, InputIssueRelationKind,
            InputIssueRelationsFile,
        },
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
        assert!(dir.path().join(".agentflow/spec").exists());
        assert!(dir.path().join(".agentflow/goal-tree").exists() == false);
        assert!(!dir.path().join(".agentflow/define/goals").exists());
        assert!(!dir.path().join(".agentflow/define/milestones").exists());
        assert!(!dir.path().join(".agentflow/define/issues").exists());
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
    fn priority_serializes_as_p0_to_p3_and_reads_legacy_values() {
        assert_eq!(serde_json::to_value(InputPriority::P0).unwrap(), "p0");
        assert_eq!(serde_json::to_value(InputPriority::P1).unwrap(), "p1");
        assert_eq!(serde_json::to_value(InputPriority::P2).unwrap(), "p2");
        assert_eq!(serde_json::to_value(InputPriority::P3).unwrap(), "p3");
        assert_eq!(
            serde_json::from_value::<InputPriority>(serde_json::json!("high")).unwrap(),
            InputPriority::P1
        );
        assert_eq!(
            serde_json::from_value::<InputPriority>(serde_json::json!("normal")).unwrap(),
            InputPriority::P2
        );
        assert_eq!(
            serde_json::from_value::<InputPriority>(serde_json::json!("low")).unwrap(),
            InputPriority::P3
        );
    }

    #[test]
    fn build_agent_pipeline_starts_with_issue_preflight() {
        let mut issue = InputIssue {
            issue_id: "iss-001".to_string(),
            issue_category: IssueCategory::Spec,
            required_agent_role: AgentRole::BuildAgent,
            ..InputIssue::default()
        };
        issue.normalize_execution_metadata();

        let pipeline = issue.execution_pipeline.unwrap();
        assert!(pipeline.git_providers.is_empty());
        assert_eq!(
            pipeline.stages.first().map(|stage| stage.stage_id.as_str()),
            Some("issue-preflight")
        );
    }

    #[test]
    fn prepare_migrates_legacy_priority_and_risk_field_names() {
        let dir = tempdir().unwrap();
        prepare_input_workspace(dir.path()).unwrap();
        let path = dir.path().join(".agentflow/input/issues/iss-legacy.json");
        let legacy_issue = InputIssue {
            issue_id: "iss-legacy".to_string(),
            issue_model: InputIssueModel::Direct,
            issue_category: IssueCategory::Spec,
            required_agent_role: AgentRole::BuildAgent,
            source_spec_id: "spec-001".to_string(),
            title: "Legacy priority".to_string(),
            summary: "Migrate old issue metadata.".to_string(),
            kind: crate::issue::InputIssueKind::Cleanup,
            priority: InputPriority::P1,
            execution_risk: InputRiskLevel::Medium,
            ..InputIssue::default()
        };
        let mut legacy_value = serde_json::to_value(legacy_issue).unwrap();
        legacy_value["priority"] = serde_json::json!("high");
        legacy_value["riskLevel"] = legacy_value["executionRisk"].clone();
        legacy_value
            .as_object_mut()
            .unwrap()
            .remove("executionRisk");
        fs::write(&path, serde_json::to_string_pretty(&legacy_value).unwrap()).unwrap();

        prepare_input_workspace(dir.path()).unwrap();

        let repaired: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(repaired["priority"], "p1");
        assert_eq!(repaired["executionRisk"], "medium");
        assert!(repaired.get("riskLevel").is_none());
    }

    #[test]
    fn prepare_rejects_legacy_issue_status_values() {
        let dir = tempdir().unwrap();
        prepare_input_workspace(dir.path()).unwrap();
        let path = dir
            .path()
            .join(".agentflow/input/issues/iss-legacy-status.json");
        fs::write(
            &path,
            serde_json::to_string_pretty(&serde_json::json!({
                "version": "input-issue.v1",
                "issueId": "iss-legacy-status",
                "issueModel": "direct",
                "issueCategory": "spec",
                "requiredAgentRole": "build-agent",
                "sourceSpecId": "spec-001",
                "projectId": null,
                "title": "Legacy status",
                "summary": "Reject old issue status values.",
                "kind": "feature",
                "priority": "p1",
                "status": "ready-for-execute",
                "displayStatus": "ready",
                "executionRisk": "medium",
                "scope": ["src/lib.rs"],
                "nonGoals": [],
                "acceptanceCriteria": [],
                "validationHints": ["cargo test"],
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
                    "path": ".agentflow/input/issues/iss-legacy-status.json",
                    "revision": 1
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let error = prepare_input_workspace(dir.path()).unwrap_err();
        assert!(error.to_string().contains("iss-legacy-status.json"));
    }

    #[test]
    fn direct_issue_requires_null_project_id_and_execution_risk() {
        let dir = tempdir().unwrap();
        prepare_input_workspace(dir.path()).unwrap();
        let issue = InputIssue {
            issue_id: "iss-001".to_string(),
            issue_model: InputIssueModel::Direct,
            source_spec_id: "spec-001".to_string(),
            project_id: Some("proj-001".to_string()),
            title: "Direct issue should not point to project".to_string(),
            execution_risk: InputRiskLevel::Medium,
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
            execution_risk: InputRiskLevel::Low,
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
            priority: InputPriority::P2,
            execution_risk: InputRiskLevel::Medium,
            ..InputIssue::default()
        };
        let issue_two = InputIssue {
            issue_id: "iss-002".to_string(),
            issue_model: InputIssueModel::Project,
            source_spec_id: "spec-001".to_string(),
            project_id: Some("proj-001".to_string()),
            title: "Second issue".to_string(),
            execution_risk: InputRiskLevel::High,
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
            execution_risk: InputRiskLevel::Low,
            ..InputIssue::default()
        })
        .unwrap();

        assert!(issue.get("executionRisk").is_some());
        assert!(issue.get("riskLevel").is_none());
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
    fn display_status_serializes_as_canonical_issue_status() {
        assert_eq!(
            serde_json::to_value(DisplayStatus::Backlog).unwrap(),
            "backlog"
        );
        assert_eq!(
            serde_json::to_value(DisplayStatus::Blocked).unwrap(),
            "blocked"
        );
        assert_eq!(serde_json::to_value(DisplayStatus::Todo).unwrap(), "todo");
        assert_eq!(
            serde_json::to_value(DisplayStatus::InProgress).unwrap(),
            "in_progress"
        );
        assert_eq!(
            serde_json::to_value(DisplayStatus::InReview).unwrap(),
            "in_review"
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
            status: InputIssueStatus::Todo,
            display_status: DisplayStatus::InReview,
            latest_run_id: Some("run-001".to_string()),
            ..InputIssue::default()
        })
        .unwrap();

        assert_eq!(issue["status"], "todo");
        assert_eq!(issue["displayStatus"], "in_review");
        assert_eq!(issue["latestRunId"], "run-001");
    }

    #[test]
    fn prepare_normalizes_spec_issue_target_metadata() {
        let dir = tempdir().unwrap();
        prepare_input_workspace(dir.path()).unwrap();
        let issue = InputIssue {
            issue_id: "iss-001".to_string(),
            source_spec_id: "spec-001".to_string(),
            title: "Ready issue".to_string(),
            status: InputIssueStatus::Todo,
            execution_risk: InputRiskLevel::Low,
            context_pack_path: ".agentflow/execute/runs/iss-001/context-pack.json".to_string(),
            scope: vec!["src/lib.rs".to_string()],
            validation_hints: vec!["cargo test".to_string()],
            ..InputIssue::default()
        };
        fs::write(
            dir.path().join(".agentflow/input/issues/iss-001.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();

        prepare_input_workspace(dir.path()).unwrap();

        let issue: InputIssue =
            crate::storage::read_json(&dir.path().join(".agentflow/input/issues/iss-001.json"))
                .unwrap();
        assert_eq!(issue.issue_category, IssueCategory::Spec);
        assert_eq!(issue.required_agent_role, AgentRole::BuildAgent);
        assert_eq!(issue.issue_path, ".agentflow/input/issues/iss-001.json");
        assert_eq!(
            issue.source_spec_path,
            ".agentflow/input/specs/approved/spec-001/spec.json"
        );
        assert_eq!(issue.handoff_id, "handoff-iss-001");
        assert_eq!(
            issue.context_pack_path,
            ".agentflow/panel/context-packs/iss-001.json"
        );
        assert_eq!(issue.panel.context_pack_id.as_deref(), Some("iss-001"));
        assert_eq!(issue.allowed_paths, vec!["src/lib.rs".to_string()]);
        assert_eq!(issue.validation_commands, vec!["cargo test".to_string()]);
        assert_eq!(
            issue
                .expected_outputs
                .get("executeRunDir")
                .map(String::as_str),
            Some(".agentflow/execute/runs/iss-001")
        );
        assert_eq!(
            issue
                .expected_outputs
                .get("evidencePath")
                .map(String::as_str),
            Some(".agentflow/output/evidence/iss-001.json")
        );
        assert_eq!(
            issue
                .expected_outputs
                .get("releaseDeliveryDir")
                .map(String::as_str),
            Some(".agentflow/output/release/iss-001")
        );
        let pipeline = issue.execution_pipeline.as_ref().unwrap();
        assert_eq!(pipeline.version, BUILD_AGENT_EXECUTION_PIPELINE_VERSION);
        assert_eq!(pipeline.agent_role, AgentRole::BuildAgent);
        assert!(pipeline.git_providers.is_empty());
        assert_eq!(
            pipeline.merge_modes,
            vec![
                "auto-merge-if-eligible".to_string(),
                "manual-merge".to_string()
            ]
        );
        for stage_id in BUILD_AGENT_PIPELINE_STAGE_IDS {
            assert!(
                pipeline
                    .stages
                    .iter()
                    .any(|stage| stage.stage_id == stage_id && stage.required),
                "missing required pipeline stage {stage_id}"
            );
        }
        let preflight_stage = pipeline
            .stages
            .iter()
            .find(|stage| stage.stage_id == "issue-preflight")
            .unwrap();
        assert_eq!(preflight_stage.label, "执行前置检测");
        assert!(preflight_stage
            .goal
            .contains("只认当前 AgentFlow input issue"));
        assert!(preflight_stage.goal.contains("仍在 backlog"));
        assert!(preflight_stage.goal.contains("依赖已完成"));
        assert!(preflight_stage.goal.contains("合同完整"));
        assert!(preflight_stage.goal.contains("Context Pack"));
        assert!(preflight_stage.goal.contains("切到 todo"));
        assert!(preflight_stage.goal.contains("创建当前 run"));
        assert!(preflight_stage.goal.contains("当前 run 进入 planned"));
        assert!(preflight_stage.goal.contains("issue 再进入 in_progress"));
        assert!(preflight_stage
            .goal
            .contains("不是禁止调用 AgentFlow 官方命令"));
        assert!(preflight_stage
            .goal
            .contains("GitHub/GitLab 不在这个阶段检测"));
        assert!(preflight_stage.evidence.contains(
            &"AgentFlow input issue is the only active task source; executionPipeline is read from that issue contract".to_string()
        ));
        assert!(preflight_stage.evidence.contains(
            &"no external issue/task/plan/queue/thread/tool state is used as task authority"
                .to_string()
        ));
        assert!(preflight_stage
            .evidence
            .contains(&"input issue status is backlog before scheduling".to_string()));
        assert!(preflight_stage
            .evidence
            .contains(&"Panel Context Pack exists or is generated".to_string()));
        assert!(preflight_stage
            .evidence
            .contains(&"input issue status changed to todo before runtime preflight".to_string()));
        assert!(preflight_stage.evidence.contains(
            &"current run is created by `agentflow build-agent start --issue-id <issue-id>` before source edits"
                .to_string()
        ));
        assert!(preflight_stage.evidence.contains(
            &"current run status changed to planned after runtime preflight".to_string()
        ));
        assert!(preflight_stage.evidence.contains(
            &"no `.agentflow/**` facts are handwritten; official AgentFlow loop commands are used instead"
                .to_string()
        ));
        assert!(preflight_stage.evidence.contains(
            &"working tree has no uncommitted user source changes before in_progress".to_string()
        ));
        assert!(preflight_stage.evidence.contains(
            &"input issue status changed to in_progress after runtime preflight".to_string()
        ));
        assert!(preflight_stage
            .evidence
            .iter()
            .all(|item| !item.contains("gh ") && !item.contains("glab ")));
        let test_design_stage = pipeline
            .stages
            .iter()
            .find(|stage| stage.stage_id == "test-design")
            .unwrap();
        assert_eq!(test_design_stage.label, "测试设计");
        assert!(test_design_stage.goal.contains("不能做 TDD"));
        assert!(test_design_stage
            .evidence
            .contains(&"failing test result or TDD-not-applicable reason".to_string()));
        let sandbox_stage = pipeline
            .stages
            .iter()
            .find(|stage| stage.stage_id == "sandbox-verify")
            .unwrap();
        assert_eq!(sandbox_stage.label, "沙箱验证");
        let create_pr_stage = pipeline
            .stages
            .iter()
            .find(|stage| stage.stage_id == "create-pr")
            .unwrap();
        assert!(create_pr_stage
            .goal
            .contains("AgentFlow Build Agent PR/MR 模板"));
        assert!(create_pr_stage
            .evidence
            .contains(&"AgentFlow Build Agent PR/MR template completed".to_string()));
        let merge_stage = pipeline
            .stages
            .iter()
            .find(|stage| stage.stage_id == "merge-pr")
            .unwrap();
        assert!(merge_stage.goal.contains("auto-merge-if-eligible"));
        assert!(merge_stage.goal.contains("manual-merge"));
        assert!(merge_stage.goal.contains("in_review"));
        assert!(merge_stage
            .evidence
            .iter()
            .any(|item| item.contains("gh pr merge --auto")));
        assert!(merge_stage
            .evidence
            .iter()
            .any(|item| item.contains("glab mr merge --auto-merge")));
        assert!(merge_stage.evidence.contains(
            &"auto-merge rejection reason when falling back to manual-merge".to_string()
        ));
        assert!(merge_stage
            .evidence
            .contains(&"in_review wait evidence when manual-merge fallback is active".to_string()));
        let writeback_stage = pipeline
            .stages
            .iter()
            .find(|stage| stage.stage_id == "writeback-done")
            .unwrap();
        assert!(writeback_stage.goal.contains("build-agent prepare-review"));
        assert!(writeback_stage
            .goal
            .contains("build-agent write-merge-proof"));
        assert!(writeback_stage.goal.contains("build-agent complete"));
        assert!(writeback_stage
            .evidence
            .iter()
            .any(|item| item.contains("target/release/agentflow build-agent prepare-review")));
        assert!(writeback_stage
            .evidence
            .iter()
            .any(|item| item.contains("target/release/agentflow build-agent write-merge-proof")));
        assert!(writeback_stage
            .evidence
            .iter()
            .any(|item| item.contains("cargo build --release --bin agentflow")));
        assert!(writeback_stage
            .evidence
            .iter()
            .any(|item| item.contains("target/debug/agentflow build-agent complete")));
    }

    #[test]
    fn prepare_publishes_ready_spec_issue_event() {
        let dir = tempdir().unwrap();
        prepare_input_workspace(dir.path()).unwrap();
        let issue = InputIssue {
            issue_id: "iss-ready-event".to_string(),
            source_spec_id: "spec-001".to_string(),
            title: "Ready issue event".to_string(),
            summary: "Publish ready event from input prepare.".to_string(),
            status: InputIssueStatus::Todo,
            display_status: DisplayStatus::Todo,
            execution_risk: InputRiskLevel::Low,
            scope: vec!["src/lib.rs".to_string()],
            validation_hints: vec!["cargo test".to_string()],
            ..InputIssue::default()
        };
        fs::write(
            dir.path()
                .join(".agentflow/input/issues/iss-ready-event.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();

        prepare_input_workspace(dir.path()).unwrap();

        let events = agentflow_workflow_events::load_events(dir.path()).unwrap();
        let ready_events = events
            .iter()
            .filter(|event| {
                event.event_type == agentflow_workflow_events::EVENT_TYPE_INPUT_ISSUE_READY
                    && event.subject_id == "iss-ready-event"
            })
            .collect::<Vec<_>>();
        assert_eq!(ready_events.len(), 1);
        let payload: agentflow_workflow_events::IssueReadyPayload =
            serde_json::from_value(ready_events[0].payload.clone()).unwrap();
        assert_eq!(payload.issue_id, "iss-ready-event");
        assert_eq!(
            payload.context_pack_path.as_deref(),
            Some(".agentflow/panel/context-packs/iss-ready-event.json")
        );
    }

    #[test]
    fn issue_without_display_status_defaults_to_backlog() {
        let issue: InputIssue = serde_json::from_value(serde_json::json!({
            "version": "input-issue.v1",
            "issueId": "iss-display-default",
            "issueModel": "direct",
            "sourceSpecId": "spec-001",
            "projectId": null,
            "title": "Issue without display status",
            "summary": "Issue without displayStatus",
            "kind": "feature",
            "priority": "p2",
            "status": "todo",
            "executionRisk": "low",
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
                "path": ".agentflow/input/issues/iss-display-default.json",
                "revision": 1
            }
        }))
        .unwrap();

        assert_eq!(issue.display_status, DisplayStatus::Backlog);
        assert_eq!(
            DisplayStatus::from_input_status(&issue.status),
            DisplayStatus::Todo
        );
        assert_eq!(issue.issue_category, IssueCategory::Spec);
        assert_eq!(issue.required_agent_role, AgentRole::BuildAgent);
        validate_agent_issue_permission(&issue, &AgentRole::BuildAgent).unwrap();
        assert!(validate_agent_issue_permission(&issue, &AgentRole::AuditAgent).is_err());
    }

    #[test]
    fn prepare_repairs_simplified_execution_pipeline_shape() {
        let dir = tempdir().unwrap();
        prepare_input_workspace(dir.path()).unwrap();
        fs::write(
            dir.path().join(".agentflow/input/issues/AF-0201.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "version": "input-issue.v1",
                "issueId": "AF-0201",
                "issueModel": "direct",
                "issueCategory": "spec",
                "requiredAgentRole": "build-agent",
                "sourceSpecId": "agentflow-v0.2.0-codex-bridge",
                "sourceSpecPath": ".agentflow/input/specs/approved/agentflow-v0.2.0-codex-bridge/spec.json",
                "issuePath": ".agentflow/input/issues/AF-0201.json",
                "handoffId": "handoff-AF-0201",
                "contextPackPath": ".agentflow/panel/context-packs/AF-0201.json",
                "projectId": null,
                "title": "落地 Codex 角色使用说明",
                "summary": "简化 executionPipeline 也不能让 input loader 失败。",
                "kind": "feature",
                "priority": "p1",
                "status": "todo",
                "displayStatus": "todo",
                "executionRisk": "medium",
                "scope": ["apps/desktop/src/**"],
                "nonGoals": [],
                "acceptanceCriteria": ["客户端能读取任务。"],
                "validationHints": ["npm --prefix apps/desktop run build"],
                "expectedOutputs": {
                    "executeRunDir": ".agentflow/execute/runs/AF-0201",
                    "evidencePath": ".agentflow/output/evidence/AF-0201.json",
                    "releaseDeliveryDir": ".agentflow/output/release/AF-0201"
                },
                "executionPipeline": {
                    "version": "build-agent-execution-pipeline.v1",
                    "mergeMode": "auto-merge-if-eligible",
                    "allowedMergeModes": ["auto-merge-if-eligible", "manual-merge"],
                    "prTemplateSource": "handoff/executionPipeline",
                    "auditTriggerPolicy": "done-and-task-delivery-do-not-create-audit-issue"
                },
                "relations": {
                    "blockedBy": [],
                    "blocks": [],
                    "related": [],
                    "duplicateOf": null
                },
                "panel": {
                    "snapshotId": null,
                    "contextPackId": "AF-0201"
                },
                "system": {
                    "createdBy": "fixture",
                    "createdAt": 1,
                    "updatedAt": 1,
                    "path": ".agentflow/input/issues/AF-0201.json",
                    "revision": 1
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let snapshot = prepare_input_workspace(dir.path()).unwrap();

        assert!(snapshot.ready, "{:?}", snapshot.status.errors);
        let issue: InputIssue =
            crate::storage::read_json(&dir.path().join(".agentflow/input/issues/AF-0201.json"))
                .unwrap();
        let pipeline = issue.execution_pipeline.as_ref().unwrap();
        assert_eq!(pipeline.agent_role, AgentRole::BuildAgent);
        assert!(pipeline.git_providers.is_empty());
        for stage_id in BUILD_AGENT_PIPELINE_STAGE_IDS {
            assert!(
                pipeline
                    .stages
                    .iter()
                    .any(|stage| stage.stage_id == stage_id && stage.required),
                "missing required pipeline stage {stage_id}"
            );
        }
    }

    #[test]
    fn audit_expected_outputs_array_is_supported() {
        let mut issue: InputIssue = serde_json::from_value(serde_json::json!({
            "version": "input-issue.v1",
            "issueId": "audit-release-v0.1.0",
            "issueModel": "direct",
            "issueCategory": "audit",
            "requiredAgentRole": "audit-agent",
            "sourceSpecId": "spec-001",
            "projectId": null,
            "title": "Audit release",
            "summary": "Audit release",
            "kind": "validation",
            "priority": "p1",
            "status": "todo",
            "displayStatus": "todo",
            "executionRisk": "high",
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
            "audit": {
                "auditId": "audit-001",
                "trigger": "release-auto",
                "sourceReleaseId": "release-v0.1.0",
                "sourceRunId": "release-v0.1.0",
                "sourceDeliveryPath": ".agentflow/output/release/release-v0.1.0/delivery.json",
                "auditOutputDir": ".agentflow/audit/audit-001",
                "expectedOutputs": [
                    "audit.json",
                    "audit-report.md",
                    "findings.json",
                    "evidence-map.json",
                    "traceability.json"
                ]
            },
            "system": {
                "createdBy": "fixture",
                "createdAt": 1,
                "updatedAt": 1,
                "path": ".agentflow/input/issues/audit-release-v0.1.0.json",
                "revision": 1
            }
        }))
        .unwrap();

        issue.normalize_execution_metadata();
        let audit = issue.audit.unwrap();
        assert_eq!(
            audit
                .expected_outputs
                .get("audit-report.md")
                .map(String::as_str),
            Some("audit-report.md")
        );
        assert!(audit.expected_outputs.contains_key("traceability.json"));
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
            status: InputIssueStatus::Todo,
            execution_risk: InputRiskLevel::Low,
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

        assert_eq!(issue_entry.status, "todo");
        assert_eq!(issue_entry.display_status, Some(DisplayStatus::Todo));
    }

    #[test]
    fn prepare_repairs_legacy_index_and_relation_field_names() {
        let dir = tempdir().unwrap();
        prepare_input_workspace(dir.path()).unwrap();
        let project = InputProject {
            project_id: "proj-001".to_string(),
            source_spec_id: "spec-001".to_string(),
            title: "Release audit project".to_string(),
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
            status: InputIssueStatus::Todo,
            execution_risk: InputRiskLevel::Medium,
            ..InputIssue::default()
        };
        let issue_two = InputIssue {
            issue_id: "iss-002".to_string(),
            issue_model: InputIssueModel::Project,
            source_spec_id: "spec-001".to_string(),
            project_id: Some("proj-001".to_string()),
            title: "Second issue".to_string(),
            status: InputIssueStatus::Todo,
            execution_risk: InputRiskLevel::Medium,
            ..InputIssue::default()
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
            dir.path().join(".agentflow/input/index.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "version": "input-index.v1",
                "updatedAt": 1,
                "specs": ["spec-001"],
                "projects": ["proj-001"],
                "issues": ["iss-001", "iss-002"]
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            dir.path()
                .join(".agentflow/input/relations/issue-relations.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "version": "input-issue-relations.v1",
                "relations": [
                    { "from": "iss-001", "to": "iss-002", "type": "blocks" }
                ]
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            dir.path()
                .join(".agentflow/input/relations/dependency-graph.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "version": "input-dependency-graph.v1",
                "nodes": ["iss-001", "iss-002"],
                "edges": [
                    { "from": "iss-001", "to": "iss-002", "type": "blocks" }
                ]
            }))
            .unwrap(),
        )
        .unwrap();

        let snapshot = prepare_input_workspace(dir.path()).unwrap();

        assert!(snapshot.ready, "{:?}", snapshot.status.errors);
        assert_eq!(snapshot.issues.len(), 2);
        let index = load_input_index(dir.path()).unwrap();
        assert_eq!(index.issues.len(), 2);
        assert_eq!(index.issues[0].id, "iss-001");
        assert_eq!(index.issues[0].display_status, Some(DisplayStatus::Todo));
        let relations: InputIssueRelationsFile = crate::storage::read_json(
            &dir.path()
                .join(".agentflow/input/relations/issue-relations.json"),
        )
        .unwrap();
        assert_eq!(relations.relations[0].from_issue_id, "iss-001");
        assert_eq!(relations.relations[0].to_issue_id, "iss-002");
        let dependency_graph: InputDependencyGraph = crate::storage::read_json(
            &dir.path()
                .join(".agentflow/input/relations/dependency-graph.json"),
        )
        .unwrap();
        assert_eq!(dependency_graph.edges[0].from_issue_id, "iss-001");
        assert_eq!(dependency_graph.edges[0].to_issue_id, "iss-002");
    }
}
