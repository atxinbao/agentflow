use crate::product_workspace::{
    load_product_workspace_projection, ProductWorkspaceProjection, ProductWorkspaceStatus,
};
use agentflow_mcp::{McpProviderSmokeArtifact, McpProviderSmokeOutcome};
use agentflow_task_artifacts::{
    create_task_run, prepare_task_artifact_workspace, task_run_dir, update_task_run_status,
    write_task_acceptance_gate_decision, write_task_command_record, write_task_evidence,
    write_task_validation, TaskAcceptanceGateDecision, TaskAcceptanceGateKind,
    TaskAcceptanceOutcome, TaskAcceptanceSubGateDecision, TaskAcceptanceTraceability,
    TaskCommandInput, TaskRunStatus, TASK_ACCEPTANCE_GATE_VERSION,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const PRODUCT_FIRST_RUN_ONBOARDING_CONTRACT_VERSION: &str =
    "agentflow-product-first-run-onboarding.v1";
pub const PRODUCT_ONBOARDING_READINESS_VERSION: &str = "agentflow-product-onboarding-readiness.v1";
pub const PRODUCT_GUIDED_SAMPLE_RUN_PLAN_VERSION: &str =
    "agentflow-product-guided-sample-run-plan.v1";
pub const PRODUCT_GUIDED_SAMPLE_RUN_RECEIPT_VERSION: &str =
    "agentflow-product-guided-sample-run-receipt.v1";
const GUIDED_SAMPLE_ISSUE_ID: &str = "AF-GUIDED-SAMPLE-001";
const GUIDED_SAMPLE_RUN_ID: &str = "run-001";
const GUIDED_SAMPLE_WORKFLOW_REF: &str = "workflow://agentflow/product-onboarding-guided-sample";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProductOnboardingStatus {
    Start,
    Blocked,
    Repairable,
    Deferred,
    Ready,
    Completed,
    Retry,
}

impl ProductOnboardingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Blocked => "blocked",
            Self::Repairable => "repairable",
            Self::Deferred => "deferred",
            Self::Ready => "ready",
            Self::Completed => "completed",
            Self::Retry => "retry",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProductReadinessStatus {
    Ready,
    Missing,
    Stale,
    Failed,
    Unknown,
}

impl ProductReadinessStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Missing => "missing",
            Self::Stale => "stale",
            Self::Failed => "failed",
            Self::Unknown => "unknown",
        }
    }

    fn ready(&self) -> bool {
        matches!(self, Self::Ready)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductOnboardingStateContract {
    pub status: ProductOnboardingStatus,
    pub user_label: String,
    pub runtime_meaning: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductFirstRunOnboardingContract {
    pub version: String,
    pub selected_product_id: String,
    pub user_goal: String,
    pub stages: Vec<String>,
    pub states: Vec<ProductOnboardingStateContract>,
    pub required_inputs: Vec<String>,
    pub runtime_writes: Vec<String>,
    pub user_hidden_paths: Vec<String>,
    pub diagnostic_paths: Vec<String>,
    pub command_entries: Vec<String>,
    pub authority_boundary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductReadinessItem {
    pub id: String,
    pub label: String,
    pub status: ProductReadinessStatus,
    pub user_summary: String,
    pub diagnostic_ref: Option<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductOnboardingReadinessReport {
    pub version: String,
    pub selected_product_id: String,
    pub workspace_root_ref: String,
    pub status: ProductOnboardingStatus,
    pub items: Vec<ProductReadinessItem>,
    pub next_actions: Vec<String>,
    pub blockers: Vec<String>,
    pub repairable: bool,
    pub projection: ProductWorkspaceProjection,
    pub user_hidden_agentflow_boundary: bool,
    pub diagnostics_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductGuidedSampleStage {
    pub id: String,
    pub label: String,
    pub owner: String,
    pub expected_output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductGuidedSampleRunPlan {
    pub version: String,
    pub selected_product_id: String,
    pub workspace_root_ref: String,
    pub status: ProductOnboardingStatus,
    pub stages: Vec<ProductGuidedSampleStage>,
    pub expected_trace: Vec<String>,
    pub delivery_summary: Vec<String>,
    pub failure_next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductGuidedSampleRunReceipt {
    pub version: String,
    pub selected_product_id: String,
    pub workspace_root_ref: String,
    pub issue_id: String,
    pub run_id: String,
    pub command: String,
    pub execution_mode: String,
    pub status: ProductOnboardingStatus,
    pub result: String,
    pub run_path: String,
    pub receipt_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivery_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deferred_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_attempt_path: Option<String>,
    pub retryable: bool,
    pub decision_result: String,
    pub next_actions: Vec<String>,
    pub delivery_summary: Vec<String>,
    pub created_paths: Vec<String>,
    pub trace: Vec<String>,
}

pub fn first_run_onboarding_contract(
    selected_product_id: impl Into<String>,
) -> ProductFirstRunOnboardingContract {
    let selected_product_id = selected_product_id.into();
    ProductFirstRunOnboardingContract {
        version: PRODUCT_FIRST_RUN_ONBOARDING_CONTRACT_VERSION.to_string(),
        selected_product_id,
        user_goal: "帮助新用户选择产品、准备工作区、确认环境可用，并跑通第一个安全样例。".to_string(),
        stages: vec![
            "choose-product".to_string(),
            "bootstrap-workspace".to_string(),
            "check-readiness".to_string(),
            "run-guided-sample".to_string(),
            "view-delivery".to_string(),
        ],
        states: vec![
            state_contract(
                ProductOnboardingStatus::Start,
                "选择产品",
                "等待用户选择产品和本地项目目录。",
                "选择产品并创建工作区",
            ),
            state_contract(
                ProductOnboardingStatus::Blocked,
                "无法继续",
                "产品、工作区或必需事实源缺失，Runtime 不允许进入样例执行。",
                "修复缺失项后重新检测",
            ),
            state_contract(
                ProductOnboardingStatus::Repairable,
                "需要补齐",
                "工作区已存在，但 provider / connector / skill readiness 证据不完整。",
                "按提示补齐环境证据",
            ),
            state_contract(
                ProductOnboardingStatus::Deferred,
                "暂缓执行",
                "工作区证据存在但有 stale / skipped 状态，Runtime 暂不进入样例执行。",
                "刷新 readiness 证据后重新检测",
            ),
            state_contract(
                ProductOnboardingStatus::Ready,
                "可以开始",
                "产品工作区、投影、provider、connector 和 skill readiness 都可用。",
                "运行引导样例",
            ),
            state_contract(
                ProductOnboardingStatus::Completed,
                "已完成",
                "样例链路已留下 intake、task、executor、evidence、decision 和 delivery 事实。",
                "查看交付摘要",
            ),
            state_contract(
                ProductOnboardingStatus::Retry,
                "需要重试",
                "样例执行失败但有可修复下一步，不能静默标记完成。",
                "根据修复建议重试",
            ),
        ],
        required_inputs: vec![
            "selectedProductId".to_string(),
            "workspaceRoot".to_string(),
            "projectName".to_string(),
            "initialGoal".to_string(),
        ],
        runtime_writes: vec![
            "docs/project/**".to_string(),
            "docs/requirements/**".to_string(),
            ".agentflow/workspace.json".to_string(),
            ".agentflow/spec/projects/**".to_string(),
            ".agentflow/spec/issues/**".to_string(),
            ".agentflow/events/**".to_string(),
            ".agentflow/tasks/**".to_string(),
            ".agentflow/projections/**".to_string(),
        ],
        user_hidden_paths: vec![".agentflow/**".to_string()],
        diagnostic_paths: vec![
            ".agentflow/workspace.json".to_string(),
            ".agentflow/projections/workspace-state.json".to_string(),
            ".agentflow/state/mcp/provider-smoke/**".to_string(),
            ".agentflow/tasks/<issue-id>/runs/<run-id>/smoke/**".to_string(),
        ],
        command_entries: vec![
            "create_product_workspace".to_string(),
            "load_product_workspace_projection".to_string(),
            "check_product_onboarding_readiness".to_string(),
            "guided_sample_run_plan".to_string(),
            "run_guided_sample".to_string(),
        ],
        authority_boundary:
            "Desktop shows projections and Runtime command results; users do not edit .agentflow facts."
                .to_string(),
    }
}

pub fn check_product_onboarding_readiness(
    product_source_root: impl AsRef<Path>,
    workspace_root: impl AsRef<Path>,
    selected_product_id: impl Into<String>,
) -> ProductOnboardingReadinessReport {
    let product_source_root = product_source_root.as_ref();
    let workspace_root = workspace_root.as_ref();
    let selected_product_id = selected_product_id.into();
    let projection = load_product_workspace_projection(workspace_root);

    let product_item = product_readiness_item(product_source_root, &selected_product_id);
    let workspace_item = workspace_readiness_item(&projection);
    let provider_item = smoke_file_item(
        workspace_root.join(".agentflow/state/mcp/provider-smoke"),
        "provider",
        "Provider",
        "provider smoke 证据",
        ".agentflow/state/mcp/provider-smoke/**",
    );
    let connector_item = status_file_item(
        workspace_root.join(format!(
            ".agentflow/state/mcp/connectors/{selected_product_id}.json"
        )),
        "connector",
        "Connector",
        "Product connector readiness",
        ".agentflow/state/mcp/connectors/<product-id>.json",
    );
    let skill_item = status_file_item(
        workspace_root.join(".agentflow/state/mcp/skills/build-agent.json"),
        "skill",
        "Skill",
        "Build Agent skill readiness",
        ".agentflow/state/mcp/skills/build-agent.json",
    );
    let projection_item = ProductReadinessItem {
        id: "projection".to_string(),
        label: "Projection".to_string(),
        status: if projection.readiness == "ready"
            && projection.docs_ready
            && projection.fact_source_ready
        {
            ProductReadinessStatus::Ready
        } else if projection.blockers.is_empty() {
            ProductReadinessStatus::Unknown
        } else {
            ProductReadinessStatus::Failed
        },
        user_summary: if projection.readiness == "ready" {
            "工作区投影可读取。".to_string()
        } else {
            "工作区投影还不可用。".to_string()
        },
        diagnostic_ref: Some(".agentflow/projections/workspace-state.json".to_string()),
        next_action: "刷新工作区投影".to_string(),
    };

    let items = vec![
        product_item,
        workspace_item,
        projection_item,
        provider_item,
        connector_item,
        skill_item,
    ];
    let blockers = items
        .iter()
        .filter(|item| matches!(item.status, ProductReadinessStatus::Failed))
        .map(|item| format!("{}: {}", item.id, item.user_summary))
        .collect::<Vec<_>>();
    let missing = items
        .iter()
        .filter(|item| {
            matches!(
                item.status,
                ProductReadinessStatus::Missing
                    | ProductReadinessStatus::Unknown
                    | ProductReadinessStatus::Stale
            )
        })
        .map(|item| item.next_action.clone())
        .collect::<Vec<_>>();
    let ready = items.iter().all(|item| item.status.ready());
    let has_deferred_evidence = items
        .iter()
        .any(|item| matches!(item.status, ProductReadinessStatus::Stale));
    let has_repairable_gap = items.iter().any(|item| {
        matches!(
            item.status,
            ProductReadinessStatus::Missing | ProductReadinessStatus::Unknown
        )
    });
    let status = if ready {
        ProductOnboardingStatus::Ready
    } else if !blockers.is_empty() {
        ProductOnboardingStatus::Blocked
    } else if has_deferred_evidence && !has_repairable_gap {
        ProductOnboardingStatus::Deferred
    } else {
        ProductOnboardingStatus::Repairable
    };

    ProductOnboardingReadinessReport {
        version: PRODUCT_ONBOARDING_READINESS_VERSION.to_string(),
        selected_product_id,
        workspace_root_ref: "workspace://root".to_string(),
        status,
        items,
        next_actions: if ready {
            vec!["运行引导样例".to_string()]
        } else {
            missing
        },
        blockers,
        repairable: !ready,
        projection,
        user_hidden_agentflow_boundary: true,
        diagnostics_available: true,
    }
}

pub fn guided_sample_run_plan(
    workspace_root: impl AsRef<Path>,
    selected_product_id: impl Into<String>,
) -> ProductGuidedSampleRunPlan {
    let workspace_root = workspace_root.as_ref();
    let selected_product_id = selected_product_id.into();
    let projection = load_product_workspace_projection(workspace_root);
    let status = if projection.status == ProductWorkspaceStatus::Ready && projection.docs_ready {
        ProductOnboardingStatus::Ready
    } else {
        ProductOnboardingStatus::Blocked
    };
    ProductGuidedSampleRunPlan {
        version: PRODUCT_GUIDED_SAMPLE_RUN_PLAN_VERSION.to_string(),
        selected_product_id,
        workspace_root_ref: "workspace://root".to_string(),
        status,
        stages: vec![
            sample_stage(
                "intake",
                "整理样例需求",
                "Spec Agent",
                "preview + confirmation",
            ),
            sample_stage(
                "tasks",
                "生成样例任务",
                "Spec Agent",
                ".agentflow/spec/issues/**",
            ),
            sample_stage(
                "execute",
                "执行受控改动",
                "Build Agent",
                ".agentflow/tasks/<issue-id>/runs/**",
            ),
            sample_stage(
                "evidence",
                "采集验证证据",
                "Build Agent",
                ".agentflow/tasks/<issue-id>/evidence/**",
            ),
            sample_stage(
                "delivery",
                "形成交付摘要",
                "Build Agent",
                "docs/delivery 或 PR/MR body",
            ),
            sample_stage(
                "feedback",
                "失败时给出修复下一步",
                "Runtime",
                "repairable next action",
            ),
        ],
        expected_trace: vec![
            "Project -> Intake -> Tasks".to_string(),
            "Executor Run -> Evidence -> Decision -> Delivery".to_string(),
            "Failure -> Retry -> Feedback".to_string(),
        ],
        delivery_summary: vec![
            "样例任务完成状态".to_string(),
            "验证命令结果".to_string(),
            "交付摘要和可诊断 refs".to_string(),
        ],
        failure_next_action: "失败时保持 repairable/retry，不静默 Done。".to_string(),
    }
}

pub fn run_guided_sample(
    workspace_root: impl AsRef<Path>,
    selected_product_id: impl Into<String>,
    execution_mode: impl Into<String>,
) -> Result<ProductGuidedSampleRunReceipt> {
    let workspace_root = workspace_root.as_ref();
    let selected_product_id = selected_product_id.into();
    let execution_mode = execution_mode.into();
    let plan = guided_sample_run_plan(workspace_root, selected_product_id.clone());
    let projection = load_product_workspace_projection(workspace_root);
    let active_product_matches = projection
        .active_product
        .as_ref()
        .is_some_and(|product| product.product_id == selected_product_id);
    let execution_mode_allowed = matches!(
        execution_mode.as_str(),
        "deterministic-dry-run" | "deterministic-fail"
    );
    let deferred_reason = if !active_product_matches {
        Some(format!(
            "selected product {selected_product_id} does not match the active workspace product"
        ))
    } else if plan.status != ProductOnboardingStatus::Ready {
        Some("workspace projection is not ready for guided sample execution".to_string())
    } else if !execution_mode_allowed {
        Some(format!(
            "execution mode {execution_mode} is not allowed for guided sample"
        ))
    } else {
        None
    };

    prepare_task_artifact_workspace(workspace_root, GUIDED_SAMPLE_ISSUE_ID)?;
    let run_directory = task_run_dir(workspace_root, GUIDED_SAMPLE_ISSUE_ID, GUIDED_SAMPLE_RUN_ID)?;
    if run_directory.exists() {
        fs::remove_dir_all(&run_directory)
            .with_context(|| format!("reset guided sample run dir {}", run_directory.display()))?;
    }
    create_task_run(
        workspace_root,
        GUIDED_SAMPLE_ISSUE_ID,
        GUIDED_SAMPLE_RUN_ID,
        GUIDED_SAMPLE_WORKFLOW_REF,
        None,
    )?;

    let run_path = guided_sample_run_path();
    let receipt_path = guided_sample_receipt_path();
    let mut created_paths = vec![run_path.clone()];
    let trace;
    let (
        status,
        result,
        evidence_path,
        decision_path,
        delivery_path,
        deferred_reason,
        failure_reason,
        decision_result,
        next_actions,
        delivery_summary,
    ) = if let Some(reason) = deferred_reason {
        update_task_run_status(
            workspace_root,
            GUIDED_SAMPLE_ISSUE_ID,
            GUIDED_SAMPLE_RUN_ID,
            TaskRunStatus::Cancelled,
        )?;
        trace = vec![
            "guided-sample:received".to_string(),
            "guided-sample:deferred".to_string(),
        ];
        (
            ProductOnboardingStatus::Deferred,
            "deferred".to_string(),
            None,
            None,
            None,
            Some(reason),
            None,
            "deferred".to_string(),
            vec!["刷新 readiness 证据后重试引导样例。".to_string()],
            Vec::new(),
        )
    } else {
        let should_fail = execution_mode == "deterministic-fail";
        update_task_run_status(
            workspace_root,
            GUIDED_SAMPLE_ISSUE_ID,
            GUIDED_SAMPLE_RUN_ID,
            TaskRunStatus::InProgress,
        )?;
        let command = write_task_command_record(
            workspace_root,
            GUIDED_SAMPLE_ISSUE_ID,
            GUIDED_SAMPLE_RUN_ID,
            TaskCommandInput {
                label: "Run guided sample dry-run".to_string(),
                program: "agentflow-runtime".to_string(),
                args: vec![
                    "guided-sample".to_string(),
                    selected_product_id.clone(),
                    execution_mode.clone(),
                ],
                exit_code: if should_fail { Some(1) } else { Some(0) },
                stdout: if should_fail {
                    String::new()
                } else {
                    format!(
                        "Guided sample deterministic dry-run completed for {selected_product_id}."
                    )
                },
                stderr: if should_fail {
                    "Guided sample deterministic failure for release proof.".to_string()
                } else {
                    String::new()
                },
            },
        )?;
        update_task_run_status(
            workspace_root,
            GUIDED_SAMPLE_ISSUE_ID,
            GUIDED_SAMPLE_RUN_ID,
            TaskRunStatus::Validating,
        )?;
        let validation =
            write_task_validation(workspace_root, GUIDED_SAMPLE_ISSUE_ID, GUIDED_SAMPLE_RUN_ID)?;
        let _evidence = write_task_evidence(
            workspace_root,
            GUIDED_SAMPLE_ISSUE_ID,
            GUIDED_SAMPLE_RUN_ID,
            if validation.passed {
                "Guided sample run completed with deterministic dry-run evidence."
            } else {
                "Guided sample run failed with structured validation evidence."
            },
        )?;
        let decision =
            write_guided_sample_decision(workspace_root, validation.passed, &selected_product_id)?;
        let final_status = if validation.passed {
            TaskRunStatus::Completed
        } else {
            TaskRunStatus::Failed
        };
        update_task_run_status(
            workspace_root,
            GUIDED_SAMPLE_ISSUE_ID,
            GUIDED_SAMPLE_RUN_ID,
            final_status,
        )?;
        let evidence_path = Some(guided_sample_evidence_path());
        let decision_path = Some(guided_sample_decision_path());
        let delivery_path = if validation.passed {
            let delivery_path = write_guided_sample_delivery_record(
                workspace_root,
                &selected_product_id,
                &receipt_path,
                evidence_path.as_deref().unwrap_or(""),
                decision_path.as_deref().unwrap_or(""),
            )?;
            Some(delivery_path)
        } else {
            None
        };
        created_paths.extend([
            command.stdout_path,
            command.stderr_path,
            ".agentflow/tasks/AF-GUIDED-SAMPLE-001/runs/run-001/validation.json".to_string(),
            evidence_path.clone().unwrap_or_else(|| validation_path()),
            decision_path
                .clone()
                .unwrap_or_else(|| guided_sample_decision_path()),
        ]);
        if let Some(path) = delivery_path.clone() {
            created_paths.push(path);
        }
        trace = vec![
            "guided-sample:received".to_string(),
            "guided-sample:run-created".to_string(),
            format!("guided-sample:command-recorded:{}", command.command_id),
            format!("guided-sample:validation:{}", validation.passed),
            "guided-sample:evidence-written".to_string(),
            format!("guided-sample:decision:{}", decision.outcome.as_str()),
            if validation.passed {
                "guided-sample:delivery-written".to_string()
            } else {
                "guided-sample:delivery-skipped".to_string()
            },
            if validation.passed {
                "guided-sample:completed".to_string()
            } else {
                "guided-sample:failed".to_string()
            },
        ];
        (
            if validation.passed {
                ProductOnboardingStatus::Completed
            } else {
                ProductOnboardingStatus::Retry
            },
            if validation.passed {
                "passed".to_string()
            } else {
                "failed".to_string()
            },
            evidence_path,
            decision_path,
            delivery_path,
            None,
            if validation.passed {
                None
            } else {
                Some("guided sample validation command failed".to_string())
            },
            decision.outcome.as_str().to_string(),
            if validation.passed {
                vec!["查看样例交付。".to_string()]
            } else {
                vec![
                    "查看失败原因。".to_string(),
                    "修复配置后重新运行引导样例。".to_string(),
                ]
            },
            if validation.passed {
                vec![
                    "Guided sample evidence accepted.".to_string(),
                    "Delivery record written for first-run review.".to_string(),
                ]
            } else {
                vec![
                    "No delivery record written because acceptance decision rejected the sample."
                        .to_string(),
                ]
            },
        )
    };

    let retryable = matches!(
        status,
        ProductOnboardingStatus::Retry
            | ProductOnboardingStatus::Repairable
            | ProductOnboardingStatus::Deferred
            | ProductOnboardingStatus::Blocked
    );
    let retry_attempt_path = if retryable {
        Some(guided_sample_retry_attempt_path(
            status.as_str(),
            unix_timestamp_seconds(),
        ))
    } else {
        None
    };
    if let Some(path) = retry_attempt_path.clone() {
        created_paths.push(path);
    }
    created_paths.push(receipt_path.clone());
    let receipt = ProductGuidedSampleRunReceipt {
        version: PRODUCT_GUIDED_SAMPLE_RUN_RECEIPT_VERSION.to_string(),
        selected_product_id,
        workspace_root_ref: "workspace://root".to_string(),
        issue_id: GUIDED_SAMPLE_ISSUE_ID.to_string(),
        run_id: GUIDED_SAMPLE_RUN_ID.to_string(),
        command: "run_guided_sample".to_string(),
        execution_mode,
        status,
        result,
        run_path,
        receipt_path: receipt_path.clone(),
        evidence_path,
        decision_path,
        delivery_path,
        deferred_reason,
        failure_reason,
        retry_attempt_path,
        retryable,
        decision_result,
        next_actions,
        delivery_summary,
        created_paths,
        trace,
    };
    write_json(workspace_root.join(&receipt_path), &receipt)?;
    if let Some(path) = receipt.retry_attempt_path.as_deref() {
        write_guided_sample_retry_attempt(workspace_root, path, &receipt)?;
    }
    Ok(receipt)
}

fn state_contract(
    status: ProductOnboardingStatus,
    user_label: &str,
    runtime_meaning: &str,
    next_action: &str,
) -> ProductOnboardingStateContract {
    ProductOnboardingStateContract {
        status,
        user_label: user_label.to_string(),
        runtime_meaning: runtime_meaning.to_string(),
        next_action: next_action.to_string(),
    }
}

fn guided_sample_run_path() -> String {
    ".agentflow/tasks/AF-GUIDED-SAMPLE-001/runs/run-001/run.json".to_string()
}

fn guided_sample_receipt_path() -> String {
    ".agentflow/tasks/AF-GUIDED-SAMPLE-001/runs/run-001/guided-sample-receipt.json".to_string()
}

fn validation_path() -> String {
    ".agentflow/tasks/AF-GUIDED-SAMPLE-001/runs/run-001/validation.json".to_string()
}

fn guided_sample_evidence_path() -> String {
    ".agentflow/tasks/AF-GUIDED-SAMPLE-001/evidence/evidence.json".to_string()
}

fn guided_sample_decision_path() -> String {
    ".agentflow/tasks/AF-GUIDED-SAMPLE-001/acceptance-gate.json".to_string()
}

fn guided_sample_retry_attempt_path(status: &str, checked_at: u64) -> String {
    format!(".agentflow/tasks/AF-GUIDED-SAMPLE-001/retry-attempts/{checked_at}-{status}.json")
}

fn write_guided_sample_retry_attempt(
    workspace_root: &Path,
    path: &str,
    receipt: &ProductGuidedSampleRunReceipt,
) -> Result<()> {
    write_json(
        workspace_root.join(path),
        &json!({
            "version": "agentflow-guided-sample-retry-attempt.v1",
            "issueId": receipt.issue_id,
            "runId": receipt.run_id,
            "status": receipt.status.as_str(),
            "result": receipt.result,
            "receiptPath": receipt.receipt_path,
            "failureReason": receipt.failure_reason,
            "deferredReason": receipt.deferred_reason,
            "decisionResult": receipt.decision_result,
            "nextActions": receipt.next_actions.clone(),
            "createdAt": unix_timestamp_seconds(),
        }),
    )
}

fn write_guided_sample_decision(
    workspace_root: &Path,
    passed: bool,
    selected_product_id: &str,
) -> Result<TaskAcceptanceGateDecision> {
    let evidence_path = guided_sample_evidence_path();
    let validation_path = validation_path();
    let decision = TaskAcceptanceGateDecision {
        version: TASK_ACCEPTANCE_GATE_VERSION.to_string(),
        issue_id: GUIDED_SAMPLE_ISSUE_ID.to_string(),
        run_id: GUIDED_SAMPLE_RUN_ID.to_string(),
        passed,
        outcome: if passed {
            TaskAcceptanceOutcome::Accepted
        } else {
            TaskAcceptanceOutcome::Rejected
        },
        summary: if passed {
            format!("Guided sample evidence accepted for {selected_product_id}.")
        } else {
            format!("Guided sample evidence rejected for {selected_product_id}.")
        },
        sub_gates: vec![
            guided_sample_sub_gate(
                TaskAcceptanceGateKind::Verification,
                passed,
                vec![validation_path.clone()],
                if passed {
                    Vec::new()
                } else {
                    vec!["validation command exited non-zero".to_string()]
                },
            ),
            guided_sample_sub_gate(
                TaskAcceptanceGateKind::Evidence,
                passed,
                vec![evidence_path.clone()],
                if passed {
                    Vec::new()
                } else {
                    vec!["evidence status is failed".to_string()]
                },
            ),
        ],
        required_evidence_types: vec!["command-output".to_string(), "validation".to_string()],
        evidence_entries: Vec::new(),
        blockers: if passed {
            Vec::new()
        } else {
            vec!["guided sample validation failed".to_string()]
        },
        failure_reasons: if passed {
            Vec::new()
        } else {
            vec!["guided sample validation command failed".to_string()]
        },
        next_steps: if passed {
            vec!["查看 guided sample delivery record".to_string()]
        } else {
            vec!["修复 guided sample 配置后重试".to_string()]
        },
        traceability: TaskAcceptanceTraceability {
            issue_id: GUIDED_SAMPLE_ISSUE_ID.to_string(),
            run_id: GUIDED_SAMPLE_RUN_ID.to_string(),
            session_id: None,
            session_owner: Some("Runtime".to_string()),
            provider: Some("agentflow-runtime".to_string()),
            branch_name: None,
            acceptance_decision_path: guided_sample_decision_path(),
            evidence_path,
            validation_path,
            changed_files_path: None,
            closeout_proof_path: guided_sample_receipt_path(),
            pr_url: None,
            merge_commit_sha: None,
        },
        checked_at: unix_timestamp_seconds(),
    };
    write_task_acceptance_gate_decision(workspace_root, GUIDED_SAMPLE_ISSUE_ID, &decision)
}

fn guided_sample_sub_gate(
    gate: TaskAcceptanceGateKind,
    passed: bool,
    inputs: Vec<String>,
    failure_reasons: Vec<String>,
) -> TaskAcceptanceSubGateDecision {
    TaskAcceptanceSubGateDecision {
        gate,
        passed,
        inputs,
        outputs: Vec::new(),
        failure_reasons,
        repair_suggestion: if passed {
            "无需修复。".to_string()
        } else {
            "重新生成 readiness 证据并重跑 guided sample。".to_string()
        },
    }
}

fn write_guided_sample_delivery_record(
    workspace_root: &Path,
    selected_product_id: &str,
    receipt_path: &str,
    evidence_path: &str,
    decision_path: &str,
) -> Result<String> {
    let delivery_path = format!("docs/delivery/guided-sample/{selected_product_id}.md");
    let body = format!(
        "# Guided Sample Delivery\n\n- Product: `{selected_product_id}`\n- Issue: `{GUIDED_SAMPLE_ISSUE_ID}`\n- Run: `{GUIDED_SAMPLE_RUN_ID}`\n- Receipt: `{receipt_path}`\n- Evidence: `{evidence_path}`\n- Decision: `{decision_path}`\n- Result: accepted\n"
    );
    write_text(workspace_root.join(&delivery_path), &body)?;
    Ok(delivery_path)
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn write_json(path: impl AsRef<Path>, value: &impl Serialize) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)
        .with_context(|| format!("write {}", path.display()))
}

fn write_text(path: impl AsRef<Path>, value: &str) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, value).with_context(|| format!("write {}", path.display()))
}

fn sample_stage(
    id: &str,
    label: &str,
    owner: &str,
    expected_output: &str,
) -> ProductGuidedSampleStage {
    ProductGuidedSampleStage {
        id: id.to_string(),
        label: label.to_string(),
        owner: owner.to_string(),
        expected_output: expected_output.to_string(),
    }
}

fn product_readiness_item(
    product_source_root: &Path,
    selected_product_id: &str,
) -> ProductReadinessItem {
    let result = (|| -> Result<()> {
        let registry = agentflow_pack::load_product_registry(product_source_root)?;
        let entry = registry
            .product(selected_product_id)
            .cloned()
            .with_context(|| format!("product `{selected_product_id}` is not registered"))?;
        if !entry.valid {
            anyhow::bail!("product `{selected_product_id}` is invalid");
        }
        let definition = agentflow_pack::load_product_definition_from_entry(&entry)?;
        if !definition.valid {
            anyhow::bail!("product `{selected_product_id}` definition is invalid");
        }
        Ok(())
    })();
    match result {
        Ok(()) => ProductReadinessItem {
            id: "product".to_string(),
            label: "Product".to_string(),
            status: ProductReadinessStatus::Ready,
            user_summary: "产品定义可用。".to_string(),
            diagnostic_ref: Some("products/<product-id>/product.json".to_string()),
            next_action: "继续准备工作区".to_string(),
        },
        Err(error) => ProductReadinessItem {
            id: "product".to_string(),
            label: "Product".to_string(),
            status: ProductReadinessStatus::Failed,
            user_summary: format!("产品定义不可用：{error}"),
            diagnostic_ref: Some("products/**".to_string()),
            next_action: "选择有效产品".to_string(),
        },
    }
}

fn workspace_readiness_item(projection: &ProductWorkspaceProjection) -> ProductReadinessItem {
    let ready = projection.status == ProductWorkspaceStatus::Ready
        && projection.docs_ready
        && projection.fact_source_ready;
    ProductReadinessItem {
        id: "workspace".to_string(),
        label: "Workspace".to_string(),
        status: if ready {
            ProductReadinessStatus::Ready
        } else if projection.status == ProductWorkspaceStatus::Partial {
            ProductReadinessStatus::Failed
        } else {
            ProductReadinessStatus::Missing
        },
        user_summary: if ready {
            "项目文档和 Runtime 事实源已准备好。".to_string()
        } else {
            "项目工作区还没有准备完整。".to_string()
        },
        diagnostic_ref: Some(".agentflow/workspace.json".to_string()),
        next_action: "创建或修复项目工作区".to_string(),
    }
}

fn smoke_file_item(
    dir: PathBuf,
    id: &str,
    label: &str,
    summary: &str,
    diagnostic_ref: &str,
) -> ProductReadinessItem {
    let mut latest: Option<McpProviderSmokeArtifact> = None;
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            if let Ok(payload) = fs::read_to_string(&path) {
                if let Ok(artifact) = serde_json::from_str::<McpProviderSmokeArtifact>(&payload) {
                    if latest
                        .as_ref()
                        .map(|current| artifact.created_at > current.created_at)
                        .unwrap_or(true)
                    {
                        latest = Some(artifact);
                    }
                }
            }
        }
    }
    let status = match latest.as_ref().map(|artifact| &artifact.outcome) {
        Some(McpProviderSmokeOutcome::Passed) => ProductReadinessStatus::Ready,
        Some(McpProviderSmokeOutcome::Failed) => ProductReadinessStatus::Failed,
        Some(McpProviderSmokeOutcome::Skipped) => ProductReadinessStatus::Stale,
        None => ProductReadinessStatus::Missing,
    };
    ProductReadinessItem {
        id: id.to_string(),
        label: label.to_string(),
        status,
        user_summary: if latest.is_some() {
            format!("{summary}已生成。")
        } else {
            format!("{summary}缺失。")
        },
        diagnostic_ref: Some(diagnostic_ref.to_string()),
        next_action: format!("刷新 {label} readiness"),
    }
}

fn status_file_item(
    path: PathBuf,
    id: &str,
    label: &str,
    summary: &str,
    diagnostic_ref: &str,
) -> ProductReadinessItem {
    let status = fs::read_to_string(&path)
        .ok()
        .and_then(|payload| serde_json::from_str::<Value>(&payload).ok())
        .and_then(|value| {
            value
                .get("status")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .map(|status| match status.as_str() {
            "ready" | "passed" => ProductReadinessStatus::Ready,
            "failed" => ProductReadinessStatus::Failed,
            "stale" => ProductReadinessStatus::Stale,
            _ => ProductReadinessStatus::Unknown,
        })
        .unwrap_or(ProductReadinessStatus::Missing);
    ProductReadinessItem {
        id: id.to_string(),
        label: label.to_string(),
        user_summary: if status.ready() {
            format!("{summary}可用。")
        } else {
            format!("{summary}还不可用。")
        },
        status,
        diagnostic_ref: Some(diagnostic_ref.to_string()),
        next_action: format!("刷新 {label} readiness"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::product_workspace::{
        create_product_workspace, ProductWorkspaceCreationMode, ProductWorkspaceCreationRequest,
    };
    use agentflow_mcp::{
        McpCapability, McpProviderKind, McpProviderSmokeArtifact, McpProviderSmokeOutcome,
        McpProviderStatus, McpProviderStatusCode, McpSessionStatus,
        MCP_PROVIDER_SMOKE_ARTIFACT_VERSION,
    };
    use std::path::PathBuf;

    #[test]
    fn first_run_contract_defines_required_states_and_hidden_boundary() {
        let product_id = test_product_id();
        let contract = first_run_onboarding_contract(product_id);
        let states = contract
            .states
            .iter()
            .map(|state| state.status.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            contract.version,
            PRODUCT_FIRST_RUN_ONBOARDING_CONTRACT_VERSION
        );
        assert!(states.contains(&"start"));
        assert!(states.contains(&"blocked"));
        assert!(states.contains(&"deferred"));
        assert!(states.contains(&"ready"));
        assert!(states.contains(&"completed"));
        assert!(states.contains(&"retry"));
        assert!(contract
            .user_hidden_paths
            .contains(&".agentflow/**".to_string()));
        assert!(contract
            .command_entries
            .contains(&"create_product_workspace".to_string()));
    }

    #[test]
    fn readiness_requires_workspace_provider_connector_and_skill_evidence() {
        let source = workspace_root();
        let product_id = test_product_id();
        let dir = tempfile::tempdir().unwrap();
        let workspace = dir.path().join("workspace");
        let receipt = create_product_workspace(
            &source,
            ProductWorkspaceCreationRequest {
                project_name: "V120 Onboarding".to_string(),
                workspace_root: workspace.to_string_lossy().to_string(),
                selected_product_id: product_id.clone(),
                initial_goal: "Prepare first run.".to_string(),
                creation_mode: ProductWorkspaceCreationMode::Create,
            },
        );
        assert_eq!(receipt.status, ProductWorkspaceStatus::Created);

        let missing = check_product_onboarding_readiness(&source, &workspace, product_id.clone());
        assert_eq!(missing.status, ProductOnboardingStatus::Repairable);
        assert!(missing
            .items
            .iter()
            .any(|item| item.id == "provider" && item.status == ProductReadinessStatus::Missing));

        write_provider_smoke(&workspace, 1);
        write_status(&workspace.join(connector_status_ref(&product_id)), "ready");
        write_status(
            &workspace.join(".agentflow/state/mcp/skills/build-agent.json"),
            "ready",
        );
        let ready = check_product_onboarding_readiness(&source, &workspace, product_id);
        assert_eq!(ready.status, ProductOnboardingStatus::Ready);
        assert!(ready.user_hidden_agentflow_boundary);
    }

    #[test]
    fn readiness_reports_deferred_for_stale_runtime_evidence() {
        let source = workspace_root();
        let product_id = test_product_id();
        let dir = tempfile::tempdir().unwrap();
        let workspace = dir.path().join("workspace");
        create_product_workspace(
            &source,
            ProductWorkspaceCreationRequest {
                project_name: "V121 Deferred".to_string(),
                workspace_root: workspace.to_string_lossy().to_string(),
                selected_product_id: product_id.clone(),
                initial_goal: "Check deferred readiness.".to_string(),
                creation_mode: ProductWorkspaceCreationMode::Create,
            },
        );

        write_provider_smoke_with_outcome(&workspace, 2, McpProviderSmokeOutcome::Skipped);
        write_status(&workspace.join(connector_status_ref(&product_id)), "ready");
        write_status(
            &workspace.join(".agentflow/state/mcp/skills/build-agent.json"),
            "ready",
        );

        let deferred = check_product_onboarding_readiness(&source, &workspace, product_id);
        assert_eq!(deferred.status, ProductOnboardingStatus::Deferred);
        assert!(deferred
            .items
            .iter()
            .any(|item| item.id == "provider" && item.status == ProductReadinessStatus::Stale));
        assert!(deferred
            .next_actions
            .iter()
            .any(|action| action.contains("Provider")));
    }

    #[test]
    fn guided_sample_plan_is_ready_only_for_ready_workspace() {
        let source = workspace_root();
        let product_id = test_product_id();
        let dir = tempfile::tempdir().unwrap();
        let workspace = dir.path().join("workspace");
        let blocked = guided_sample_run_plan(&workspace, product_id.clone());
        assert_eq!(blocked.status, ProductOnboardingStatus::Blocked);

        create_product_workspace(
            &source,
            ProductWorkspaceCreationRequest {
                project_name: "V120 Sample".to_string(),
                workspace_root: workspace.to_string_lossy().to_string(),
                selected_product_id: product_id.clone(),
                initial_goal: "Run sample.".to_string(),
                creation_mode: ProductWorkspaceCreationMode::Create,
            },
        );
        let ready = guided_sample_run_plan(&workspace, product_id);
        assert_eq!(ready.status, ProductOnboardingStatus::Ready);
        assert!(ready
            .expected_trace
            .iter()
            .any(|item| item.contains("Executor")));
    }

    #[test]
    fn guided_sample_run_writes_actual_receipt_and_task_evidence() {
        let source = workspace_root();
        let product_id = test_product_id();
        let dir = tempfile::tempdir().unwrap();
        let workspace = dir.path().join("workspace");
        create_product_workspace(
            &source,
            ProductWorkspaceCreationRequest {
                project_name: "V121 Guided Run".to_string(),
                workspace_root: workspace.to_string_lossy().to_string(),
                selected_product_id: product_id.clone(),
                initial_goal: "Run guided sample.".to_string(),
                creation_mode: ProductWorkspaceCreationMode::Create,
            },
        );

        let receipt =
            run_guided_sample(&workspace, product_id, "deterministic-dry-run").expect("sample run");
        assert_eq!(receipt.status, ProductOnboardingStatus::Completed);
        assert_eq!(receipt.result, "passed");
        assert_eq!(receipt.issue_id, GUIDED_SAMPLE_ISSUE_ID);
        assert_eq!(receipt.run_id, GUIDED_SAMPLE_RUN_ID);
        assert!(workspace.join(&receipt.run_path).is_file());
        assert!(workspace.join(&receipt.receipt_path).is_file());
        assert!(receipt
            .evidence_path
            .as_ref()
            .is_some_and(|path| workspace.join(path).is_file()));
        assert!(receipt
            .decision_path
            .as_ref()
            .is_some_and(|path| workspace.join(path).is_file()));
        assert!(receipt
            .delivery_path
            .as_ref()
            .is_some_and(|path| workspace.join(path).is_file()));
        assert_eq!(receipt.decision_result, "accepted");
        assert!(!receipt.retryable);
        assert!(receipt.retry_attempt_path.is_none());
        assert!(receipt
            .next_actions
            .iter()
            .any(|item| item.contains("交付")));
        assert!(receipt
            .trace
            .iter()
            .any(|item| item == "guided-sample:completed"));
    }

    #[test]
    fn guided_sample_run_defers_invalid_sample_configuration() {
        let source = workspace_root();
        let dir = tempfile::tempdir().unwrap();
        let workspace = dir.path().join("workspace");
        create_product_workspace(
            &source,
            ProductWorkspaceCreationRequest {
                project_name: "V121 Invalid Guided Run".to_string(),
                workspace_root: workspace.to_string_lossy().to_string(),
                selected_product_id: test_product_id(),
                initial_goal: "Reject mismatched guided sample.".to_string(),
                creation_mode: ProductWorkspaceCreationMode::Create,
            },
        );

        let receipt = run_guided_sample(&workspace, "unknown-product", "deterministic-dry-run")
            .expect("deferred sample run");
        assert_eq!(receipt.status, ProductOnboardingStatus::Deferred);
        assert_eq!(receipt.result, "deferred");
        assert!(receipt.deferred_reason.is_some());
        assert!(workspace.join(&receipt.receipt_path).is_file());
        assert!(receipt.evidence_path.is_none());
        assert!(receipt.decision_path.is_none());
        assert!(receipt.delivery_path.is_none());
        assert!(receipt.retryable);
        assert!(receipt
            .retry_attempt_path
            .as_ref()
            .is_some_and(|path| workspace.join(path).is_file()));
        assert!(receipt
            .next_actions
            .iter()
            .any(|item| item.contains("重试")));
    }

    #[test]
    fn guided_sample_run_rejects_failed_evidence_without_delivery() {
        let source = workspace_root();
        let product_id = test_product_id();
        let dir = tempfile::tempdir().unwrap();
        let workspace = dir.path().join("workspace");
        create_product_workspace(
            &source,
            ProductWorkspaceCreationRequest {
                project_name: "V121 Failed Guided Run".to_string(),
                workspace_root: workspace.to_string_lossy().to_string(),
                selected_product_id: product_id.clone(),
                initial_goal: "Reject failed guided sample evidence.".to_string(),
                creation_mode: ProductWorkspaceCreationMode::Create,
            },
        );

        let receipt =
            run_guided_sample(&workspace, product_id, "deterministic-fail").expect("failed sample");
        assert_eq!(receipt.status, ProductOnboardingStatus::Retry);
        assert_eq!(receipt.result, "failed");
        assert_eq!(receipt.decision_result, "rejected");
        assert!(receipt.failure_reason.is_some());
        assert!(receipt
            .evidence_path
            .as_ref()
            .is_some_and(|path| workspace.join(path).is_file()));
        assert!(receipt
            .decision_path
            .as_ref()
            .is_some_and(|path| workspace.join(path).is_file()));
        assert!(receipt.delivery_path.is_none());
        assert!(receipt.retryable);
        assert!(receipt
            .retry_attempt_path
            .as_ref()
            .is_some_and(|path| workspace.join(path).is_file()));
        assert!(receipt
            .next_actions
            .iter()
            .any(|item| item.contains("失败原因")));
        assert!(receipt
            .trace
            .iter()
            .any(|item| item == "guided-sample:delivery-skipped"));
    }

    fn write_status(path: &Path, status: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, format!("{{\"status\":\"{status}\"}}\n")).unwrap();
    }

    fn write_provider_smoke(root: &Path, created_at: u64) {
        write_provider_smoke_with_outcome(root, created_at, McpProviderSmokeOutcome::Passed);
    }

    fn write_provider_smoke_with_outcome(
        root: &Path,
        created_at: u64,
        outcome: McpProviderSmokeOutcome,
    ) {
        let mut health = McpProviderStatus::new(McpProviderKind::Codex, created_at);
        let passed = outcome == McpProviderSmokeOutcome::Passed;
        health.status = if passed {
            McpProviderStatusCode::Ready
        } else {
            McpProviderStatusCode::Unavailable
        };
        health.installed = passed;
        health.authenticated = Some(passed);
        health.capabilities = vec![McpCapability::new("provider.codex.launch", passed)];
        let artifact = McpProviderSmokeArtifact {
            version: MCP_PROVIDER_SMOKE_ARTIFACT_VERSION.to_string(),
            provider: "codex".to_string(),
            outcome,
            reason: if passed {
                "test smoke passed".to_string()
            } else {
                "test smoke skipped".to_string()
            },
            health,
            launch_request_path: None,
            session_id: Some("session-v120".to_string()),
            session_snapshot_path: None,
            session_snapshot_readable: true,
            terminal_status: Some(McpSessionStatus::Done),
            terminal_provider_state_projectable: true,
            artifact_path: format!(".agentflow/state/mcp/provider-smoke/codex-{created_at}.json"),
            created_at,
        };
        let path = root.join(&artifact.artifact_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_string_pretty(&artifact).unwrap()).unwrap();
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
            .to_path_buf()
    }

    fn test_product_id() -> String {
        ["software", "dev"].join("-")
    }

    fn connector_status_ref(product_id: &str) -> String {
        format!(".agentflow/state/mcp/connectors/{product_id}.json")
    }
}
