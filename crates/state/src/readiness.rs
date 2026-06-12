use agentflow_execute::ExecuteRunIndexEntry;
use agentflow_input::{
    issue::{DisplayStatus, InputIssue, InputIssueStatus},
    relations::InputIssueRelationKind,
    InputSnapshot,
};
use agentflow_output::OutputSnapshot;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[derive(Debug, Clone)]
pub(crate) struct IssueReadinessBlocker {
    pub issue_id: String,
    pub action: String,
    pub reason: String,
    pub source_path: Option<String>,
}

pub(crate) fn issue_readiness_blockers(
    _root: &Path,
    input: Option<&InputSnapshot>,
    execute: Option<&agentflow_execute::ExecuteSnapshot>,
    output: Option<&OutputSnapshot>,
) -> Vec<IssueReadinessBlocker> {
    let Some(input) = input else {
        return Vec::new();
    };

    let latest_runs = latest_runs_by_issue(execute);
    let blocked_by = blocked_by_map(input);
    dependency_blockers(input, output, &latest_runs, &blocked_by)
}

pub(crate) fn issue_has_readiness_blocker(
    issue: &InputIssue,
    blockers: &[crate::model::WorkflowBlockedAction],
) -> bool {
    let issue_path = issue.issue_path.trim();
    if issue_path.is_empty() {
        return false;
    }
    blockers.iter().any(|blocker| {
        blocker.action.as_str() == "dependency-ready"
            && blocker.source_path.as_deref() == Some(issue_path)
    })
}

fn dependency_blockers(
    input: &InputSnapshot,
    output: Option<&OutputSnapshot>,
    latest_runs: &BTreeMap<String, ExecuteRunIndexEntry>,
    blocked_by: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<IssueReadinessBlocker> {
    let issues_by_id = input
        .issues
        .iter()
        .map(|issue| (issue.issue_id.as_str(), issue))
        .collect::<BTreeMap<_, _>>();
    let mut blockers = Vec::new();

    for issue in &input.issues {
        if issue_terminal(issue, latest_runs.get(&issue.issue_id), output) {
            continue;
        }
        let Some(dependencies) = blocked_by.get(&issue.issue_id) else {
            continue;
        };

        for dependency_id in dependencies {
            let Some(dependency) = issues_by_id.get(dependency_id.as_str()) else {
                blockers.push(IssueReadinessBlocker {
                    issue_id: issue.issue_id.clone(),
                    action: "dependency-ready".to_string(),
                    reason: format!(
                        "任务 {} 的前置依赖 {} 不存在，不能开始执行。",
                        issue.issue_id, dependency_id
                    ),
                    source_path: issue_source_path(issue),
                });
                continue;
            };

            let dependency_status =
                issue_display_status(dependency, latest_runs.get(dependency_id), output);
            if dependency_status != DisplayStatus::Done {
                blockers.push(IssueReadinessBlocker {
                    issue_id: issue.issue_id.clone(),
                    action: "dependency-ready".to_string(),
                    reason: format!(
                        "任务 {} 的前置依赖 {} 还没有完成，不能开始执行。",
                        issue.issue_id, dependency_id
                    ),
                    source_path: issue_source_path(issue),
                });
            }
        }
    }

    blockers
}

fn blocked_by_map(input: &InputSnapshot) -> BTreeMap<String, BTreeSet<String>> {
    let mut map = input
        .issues
        .iter()
        .map(|issue| (issue.issue_id.clone(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();

    for issue in &input.issues {
        for dependency_id in &issue.relations.blocked_by {
            map.entry(issue.issue_id.clone())
                .or_default()
                .insert(dependency_id.clone());
        }
        for blocked_issue_id in &issue.relations.blocks {
            map.entry(blocked_issue_id.clone())
                .or_default()
                .insert(issue.issue_id.clone());
        }
    }

    for relation in &input.relations.relations {
        match relation.relation_type {
            InputIssueRelationKind::BlockedBy => {
                map.entry(relation.from_issue_id.clone())
                    .or_default()
                    .insert(relation.to_issue_id.clone());
            }
            InputIssueRelationKind::Blocks => {
                map.entry(relation.to_issue_id.clone())
                    .or_default()
                    .insert(relation.from_issue_id.clone());
            }
            InputIssueRelationKind::Related | InputIssueRelationKind::DuplicateOf => {}
        }
    }

    map
}

fn latest_runs_by_issue(
    execute: Option<&agentflow_execute::ExecuteSnapshot>,
) -> BTreeMap<String, ExecuteRunIndexEntry> {
    let mut latest_runs = BTreeMap::<String, ExecuteRunIndexEntry>::new();
    let Some(execute) = execute else {
        return latest_runs;
    };
    for run in &execute.index.runs {
        let replace = match latest_runs.get(&run.issue_id) {
            Some(existing) => {
                (run.updated_at, run.run_id.as_str())
                    > (existing.updated_at, existing.run_id.as_str())
            }
            None => true,
        };
        if replace {
            latest_runs.insert(run.issue_id.clone(), run.clone());
        }
    }
    latest_runs
}

pub(crate) fn issue_display_status(
    issue: &InputIssue,
    _latest_run: Option<&ExecuteRunIndexEntry>,
    _output: Option<&OutputSnapshot>,
) -> DisplayStatus {
    if matches!(issue.status, InputIssueStatus::Cancel) {
        return DisplayStatus::Cancel;
    }
    if matches!(issue.status, InputIssueStatus::Done) {
        return DisplayStatus::Done;
    }
    DisplayStatus::from_input_status(&issue.status)
}

fn issue_terminal(
    issue: &InputIssue,
    latest_run: Option<&ExecuteRunIndexEntry>,
    output: Option<&OutputSnapshot>,
) -> bool {
    matches!(
        issue_display_status(issue, latest_run, output),
        DisplayStatus::Done | DisplayStatus::Cancel
    )
}

fn issue_source_path(issue: &InputIssue) -> Option<String> {
    (!issue.issue_path.trim().is_empty()).then(|| issue.issue_path.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_input::{
        issue::{
            default_build_agent_execution_pipeline, AgentRole, InputIssueModel, InputRiskLevel,
            IssueCategory,
        },
        model::{InputIndex, InputManifest, InputStatusSnapshot, InputWorkspaceStatus},
        relations::InputIssueRelationsFile,
    };
    use agentflow_output::{
        OutputIndex, OutputIndexEntry, OutputManifest, OutputStatusSnapshot, OutputSummary,
        OutputWorkspaceStatus,
    };

    fn snapshot(issues: Vec<InputIssue>, relations: InputIssueRelationsFile) -> InputSnapshot {
        InputSnapshot {
            version: "input-snapshot.v1".to_string(),
            project_root: "/tmp/agentflow-test".to_string(),
            ready: true,
            status: InputStatusSnapshot {
                version: "input-status.v1".to_string(),
                project_root: "/tmp/agentflow-test".to_string(),
                status: InputWorkspaceStatus::Ready,
                ready: true,
                manifest_exists: true,
                index_exists: true,
                summary: Default::default(),
                missing_paths: Vec::new(),
                warnings: Vec::new(),
                errors: Vec::new(),
            },
            manifest: InputManifest::new("/tmp/agentflow-test", Default::default()),
            index: InputIndex::default(),
            intake: Vec::new(),
            specs: Vec::new(),
            projects: Vec::new(),
            issues,
            relations,
        }
    }

    fn issue(issue_id: &str, status: InputIssueStatus) -> InputIssue {
        InputIssue {
            issue_id: issue_id.to_string(),
            issue_model: InputIssueModel::Direct,
            issue_category: IssueCategory::Spec,
            required_agent_role: AgentRole::BuildAgent,
            source_spec_id: "spec-001".to_string(),
            issue_path: format!(".agentflow/input/issues/{issue_id}.json"),
            title: issue_id.to_string(),
            summary: issue_id.to_string(),
            status,
            execution_risk: InputRiskLevel::Low,
            ..InputIssue::default()
        }
    }

    #[test]
    fn dependency_gate_blocks_until_blocked_by_issue_is_done() {
        let mut blocked = issue("AF-002", InputIssueStatus::Todo);
        blocked.relations.blocked_by = vec!["AF-001".to_string()];
        let input = snapshot(
            vec![issue("AF-001", InputIssueStatus::Todo), blocked],
            InputIssueRelationsFile::default(),
        );

        let blockers = issue_readiness_blockers(Path::new("/tmp"), Some(&input), None, None);

        assert!(blockers.iter().any(|blocker| {
            blocker.issue_id == "AF-002" && blocker.action == "dependency-ready"
        }));
    }

    #[test]
    fn dependency_gate_allows_when_blocked_by_issue_is_done() {
        let mut blocked = issue("AF-002", InputIssueStatus::Todo);
        blocked.relations.blocked_by = vec!["AF-001".to_string()];
        let input = snapshot(
            vec![issue("AF-001", InputIssueStatus::Done), blocked],
            InputIssueRelationsFile::default(),
        );

        let blockers = issue_readiness_blockers(Path::new("/tmp"), Some(&input), None, None);

        assert!(blockers.is_empty());
    }

    #[test]
    fn readiness_blockers_only_include_dependency_checks() {
        let mut issue = issue("AF-001", InputIssueStatus::Todo);
        issue.execution_pipeline = Some(default_build_agent_execution_pipeline());
        let input = snapshot(vec![issue], InputIssueRelationsFile::default());

        let blockers =
            issue_readiness_blockers(Path::new("/tmp/not-a-git-repo"), Some(&input), None, None);

        assert!(blockers.is_empty());
    }

    #[test]
    fn issue_display_status_does_not_promote_todo_from_evidence_only() {
        let issue = issue("AF-001", InputIssueStatus::Todo);
        let run = ExecuteRunIndexEntry {
            run_id: "run-001".to_string(),
            issue_id: "AF-001".to_string(),
            ..ExecuteRunIndexEntry::default()
        };
        let output = OutputSnapshot {
            version: agentflow_output::OUTPUT_SNAPSHOT_VERSION.to_string(),
            project_root: "/tmp/agentflow-test".to_string(),
            ready: true,
            status: OutputStatusSnapshot {
                version: agentflow_output::OUTPUT_STATUS_VERSION.to_string(),
                project_root: "/tmp/agentflow-test".to_string(),
                status: OutputWorkspaceStatus::Ready,
                ready: true,
                manifest_exists: true,
                index_exists: true,
                summary: OutputSummary::default(),
                missing_paths: Vec::new(),
                warnings: Vec::new(),
                errors: Vec::new(),
            },
            manifest: OutputManifest {
                version: agentflow_output::OUTPUT_MANIFEST_VERSION.to_string(),
                project_root: "/tmp/agentflow-test".to_string(),
                status: OutputWorkspaceStatus::Ready,
                paths: std::collections::BTreeMap::new(),
                summary: OutputSummary::default(),
                updated_at: 1,
            },
            index: agentflow_output::OutputIndex {
                evidence: vec![OutputIndexEntry {
                    run_id: "run-001".to_string(),
                    issue_id: "AF-001".to_string(),
                    status: "ready".to_string(),
                    updated_at: 1,
                    ..OutputIndexEntry::default()
                }],
                ..OutputIndex::default()
            },
        };

        assert_eq!(
            issue_display_status(&issue, Some(&run), Some(&output)),
            DisplayStatus::Todo
        );
    }
}
