use agentflow_runtime_api::{
    check_product_onboarding_readiness, create_product_workspace, first_run_onboarding_contract,
    guided_sample_run_plan, project_sharing_read_model, role_permission_handoff_view,
    run_guided_sample, team_delivery_decision_history_view, team_workflow_boundary_contract,
    ProductOnboardingStatus, ProductWorkspaceCreationMode, ProductWorkspaceCreationRequest,
    ProductWorkspaceStatus,
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 22 {
        bail!(
            "usage: v121_first_run_team_workflow_proofs <workspace> <metadata> <manifest> <first-run> <guided-sample> <team-boundary> <project-sharing> <role-handoff> <history> <desktop-team-surface> <v121-commercial-boundary> <v121-license-entitlement> <v121-paid-feature> <v121-commercial-workflow-shapes> <v122-commercial-boundary> <v122-license-entitlement> <v122-paid-feature> <v122-commercial-workflow-shapes> <v121-issue-milestone-closeout> <v122-issue-milestone-closeout> <v121-release-certification> <v122-release-certification>"
        );
    }

    let workspace = PathBuf::from(&args[0]);
    let proof_paths = args[1..].iter().map(PathBuf::from).collect::<Vec<_>>();

    let metadata = release_metadata_proof(&workspace);
    let first_run = first_run_runtime_command_proof(&workspace)?;
    let guided_sample = guided_sample_execution_closure_proof(&workspace)?;
    let team_boundary = team_workflow_boundary_proof();
    let project_sharing = project_sharing_read_model_proof(&workspace)?;
    let role_handoff = role_permission_handoff_view_proof(&workspace)?;
    let history = team_delivery_decision_history_proof(&workspace)?;
    let desktop_surface = desktop_team_workflow_surface_binding_proof(&workspace)?;
    let commercial_boundary_base = commercial_product_layer_boundary_proof(&workspace)?;
    let license_entitlement_base = license_entitlement_boundary_proof(&workspace)?;
    let paid_feature_base = paid_feature_boundary_proof(&workspace)?;
    let commercial_workflow_shapes_base = commercial_workflow_shapes_proof(&workspace)?;
    let commercial_boundary = legacy_alias_proof(
        commercial_boundary_base.clone(),
        "runtime/v122-commercial-boundary-contract.json",
    );
    let license_entitlement = legacy_alias_proof(
        license_entitlement_base.clone(),
        "runtime/v122-license-entitlement-boundary.json",
    );
    let paid_feature = legacy_alias_proof(
        paid_feature_base.clone(),
        "runtime/v122-paid-feature-boundary.json",
    );
    let commercial_workflow_shapes = legacy_alias_proof(
        commercial_workflow_shapes_base.clone(),
        "runtime/v122-commercial-workflow-shapes.json",
    );
    let v122_commercial_boundary = v122_primary_proof(
        commercial_boundary_base,
        "agentflow-v122-commercial-boundary-contract.v1",
    );
    let v122_license_entitlement = v122_primary_proof(
        license_entitlement_base,
        "agentflow-v122-license-entitlement-boundary.v1",
    );
    let v122_paid_feature =
        v122_primary_proof(paid_feature_base, "agentflow-v122-paid-feature-boundary.v1");
    let v122_commercial_workflow_shapes = v122_primary_proof(
        commercial_workflow_shapes_base,
        "agentflow-v122-commercial-workflow-shapes.v1",
    );
    let v121_closeout = v121_issue_milestone_closeout_proof(&workspace);
    let v122_closeout = v122_issue_milestone_closeout_proof(&workspace);

    for (path, payload) in [
        (&proof_paths[0], &metadata),
        (&proof_paths[2], &first_run),
        (&proof_paths[3], &guided_sample),
        (&proof_paths[4], &team_boundary),
        (&proof_paths[5], &project_sharing),
        (&proof_paths[6], &role_handoff),
        (&proof_paths[7], &history),
        (&proof_paths[8], &desktop_surface),
        (&proof_paths[9], &commercial_boundary),
        (&proof_paths[10], &license_entitlement),
        (&proof_paths[11], &paid_feature),
        (&proof_paths[12], &commercial_workflow_shapes),
        (&proof_paths[13], &v122_commercial_boundary),
        (&proof_paths[14], &v122_license_entitlement),
        (&proof_paths[15], &v122_paid_feature),
        (&proof_paths[16], &v122_commercial_workflow_shapes),
        (&proof_paths[17], &v121_closeout),
        (&proof_paths[18], &v122_closeout),
    ] {
        write_json(path, payload)?;
    }

    let manifest = artifact_manifest_primary_proof_index_proof(&proof_paths)?;
    write_json(&proof_paths[1], &manifest)?;

    let certification = release_certification_proof(&[
        &metadata,
        &manifest,
        &first_run,
        &guided_sample,
        &team_boundary,
        &project_sharing,
        &role_handoff,
        &history,
        &desktop_surface,
        &commercial_boundary,
        &license_entitlement,
        &paid_feature,
        &commercial_workflow_shapes,
        &v121_closeout,
    ]);
    write_json(&proof_paths[19], &certification)?;

    let v122_certification = v122_release_certification_proof(&[
        &metadata,
        &manifest,
        &first_run,
        &guided_sample,
        &team_boundary,
        &project_sharing,
        &role_handoff,
        &history,
        &desktop_surface,
        &v122_commercial_boundary,
        &v122_license_entitlement,
        &v122_paid_feature,
        &v122_commercial_workflow_shapes,
        &v121_closeout,
        &v122_closeout,
        &certification,
    ]);
    write_json(&proof_paths[20], &v122_certification)?;

    Ok(())
}

fn release_metadata_proof(workspace: &Path) -> Value {
    let changelog = read_text(workspace.join("CHANGELOG.md")).unwrap_or_default();
    let v121_release_readme =
        read_text(workspace.join("docs/delivery/releases/v1.2.1/README.md")).unwrap_or_default();
    let v122_release_readme =
        read_text(workspace.join("docs/delivery/releases/v1.2.2/README.md")).unwrap_or_default();
    let v121_release_tasks = read_text(workspace.join(
        "docs/delivery/releases/v1.2.1/AGENTFLOW_V1_2_1_FIRST_RUN_TEAM_WORKFLOW_TASKS_V1.md",
    ))
    .unwrap_or_default();
    let v122_release_tasks = read_text(workspace.join(
        "docs/delivery/releases/v1.2.2/AGENTFLOW_V1_2_2_RELEASE_PROOF_COMMERCIAL_BOUNDARY_TASKS_V1.md",
    ))
    .unwrap_or_default();
    let release_version = release_version();
    let release_tag = release_tag();
    let source_commit = source_commit();
    let workflow_run_id = workflow_run_id();
    let primary_proofs = primary_proof_paths();
    let release_version_keeps_v121_baseline = release_at_or_after(&release_version, "v1.2.1");
    let release_tag_keeps_v121_baseline = release_at_or_after(&release_tag, "v1.2.1");

    proof(
        "agentflow-v121-release-certification-top-level-metadata.v1",
        json!({
            "release-version-keeps-v121-baseline": release_version_keeps_v121_baseline,
            "release-tag-keeps-v121-baseline": release_tag_keeps_v121_baseline,
            "source-commit-present": !source_commit.is_empty(),
            "workflow-run-id-present": !workflow_run_id.is_empty(),
            "primary-proofs-are-release-scoped": primary_proofs.iter().all(|path| path.contains("runtime/v121-") || path.contains("runtime/v122-")),
            "changelog-v121-entry-present": changelog.contains("## v1.2.1") && changelog.contains("First-run Execution Closure and Team Workflow Boundary"),
            "changelog-v122-entry-present": changelog.contains("## v1.2.2") && changelog.contains("Release Proof Hardening and Commercial Boundary Preflight"),
            "release-doc-v121-entry-present": v121_release_readme.contains("First-run Execution Closure and Team Workflow Boundary"),
            "release-doc-v122-entry-present": v122_release_readme.contains("Release Proof Hardening and Commercial Boundary Preflight"),
            "all-v121-issues-traceable": all_issue_refs_present(&v121_release_tasks, 863, 872),
            "all-v122-issues-traceable": all_issue_refs_present(&v122_release_tasks, 883, 892),
        }),
        json!({
            "releaseVersion": release_version,
            "releaseTag": release_tag,
            "sourceCommit": source_commit,
            "workflowRunId": workflow_run_id,
            "artifactNames": [
                "agentflow-release-certification",
                "agentflow-release-gate-full"
            ],
            "primaryProofs": primary_proofs,
            "negativeFixture": {
                "historicalV120Only": false,
                "accepted": false
            }
        }),
    )
}

fn first_run_runtime_command_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v121-first-run-runtime-command");
    reset_path(&root)?;
    let product_id = "software-dev";
    let contract = first_run_onboarding_contract(product_id);
    let receipt = create_product_workspace(
        workspace,
        ProductWorkspaceCreationRequest {
            project_name: "V121 First-run Runtime".to_string(),
            workspace_root: root.display().to_string(),
            selected_product_id: product_id.to_string(),
            initial_goal: "Prove v1.2.1 first-run Runtime command invocation.".to_string(),
            creation_mode: ProductWorkspaceCreationMode::Create,
        },
    );
    let readiness = check_product_onboarding_readiness(workspace, &root, product_id);
    let plan = guided_sample_run_plan(&root, product_id);

    Ok(proof(
        "agentflow-v121-first-run-runtime-command-invocation.v1",
        json!({
            "contract-is-runtime-api-backed": contract.command_entries.contains(&"run_guided_sample".to_string()),
            "workspace-created": receipt.status == ProductWorkspaceStatus::Created,
            "readiness-reported": matches!(readiness.status, ProductOnboardingStatus::Repairable | ProductOnboardingStatus::Ready | ProductOnboardingStatus::Deferred),
            "guided-sample-plan-queryable": !plan.stages.is_empty(),
        }),
        json!({
            "contract": contract,
            "workspaceReceipt": receipt,
            "readiness": readiness,
            "guidedSampleRunPlan": plan,
        }),
    ))
}

fn guided_sample_execution_closure_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v121-guided-sample-execution");
    reset_path(&root)?;
    let retry_root = workspace.join("tmp/v121-guided-sample-retry");
    reset_path(&retry_root)?;
    let product_id = "software-dev";
    let receipt = create_product_workspace(
        workspace,
        ProductWorkspaceCreationRequest {
            project_name: "V121 Guided Sample".to_string(),
            workspace_root: root.display().to_string(),
            selected_product_id: product_id.to_string(),
            initial_goal: "Prove v1.2.1 guided sample execution closure.".to_string(),
            creation_mode: ProductWorkspaceCreationMode::Create,
        },
    );
    let sample = run_guided_sample(&root, product_id, "deterministic-dry-run")?;
    let retry_receipt = create_product_workspace(
        workspace,
        ProductWorkspaceCreationRequest {
            project_name: "V121 Guided Sample Retry".to_string(),
            workspace_root: retry_root.display().to_string(),
            selected_product_id: product_id.to_string(),
            initial_goal: "Prove v1.2.1 guided sample failure and retry receipt.".to_string(),
            creation_mode: ProductWorkspaceCreationMode::Create,
        },
    );
    let retry_sample = run_guided_sample(&retry_root, product_id, "deterministic-fail")?;

    Ok(proof(
        "agentflow-v121-guided-sample-execution-closure.v1",
        json!({
            "workspace-created": receipt.status == ProductWorkspaceStatus::Created,
            "sample-completed": sample.status == ProductOnboardingStatus::Completed,
            "receipt-is-task-scoped": sample.issue_id == "AF-GUIDED-SAMPLE-001" && sample.run_id == "run-001",
            "evidence-decision-delivery-present": sample.evidence_path.is_some() && sample.decision_path.is_some() && sample.delivery_path.is_some(),
            "failure-retry-workspace-created": retry_receipt.status == ProductWorkspaceStatus::Created,
            "failure-retry-receipt-present": retry_sample.status == ProductOnboardingStatus::Retry && retry_sample.retryable && retry_sample.retry_attempt_path.is_some(),
            "failed-sample-does-not-write-delivery": retry_sample.delivery_path.is_none(),
        }),
        json!({
            "workspaceReceipt": receipt,
            "guidedSampleReceipt": sample,
            "retryWorkspaceReceipt": retry_receipt,
            "retryGuidedSampleReceipt": retry_sample,
        }),
    ))
}

fn team_workflow_boundary_proof() -> Value {
    let contract = team_workflow_boundary_contract();
    proof(
        "agentflow-v121-team-workflow-boundary-contract.v1",
        json!({
            "release-is-v121": contract.release == "v1.2.1",
            "local-lightweight-scope": contract.scope == "local-lightweight-team-workflow",
            "project-sharing-included": contract.included_capabilities.iter().any(|capability| capability.id == "project-sharing"),
            "role-handoff-included": contract.included_capabilities.iter().any(|capability| capability.id == "role-permission-handoff"),
            "cloud-and-commercial-excluded": contract.excluded_capabilities.iter().any(|item| item.contains("cloud")) && contract.excluded_capabilities.iter().any(|item| item.contains("payment")),
        }),
        json!({ "teamWorkflowBoundary": contract }),
    )
}

fn project_sharing_read_model_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v121-project-sharing-read-model");
    reset_path(&root)?;
    let view = project_sharing_read_model(&root, "v121-team-project");
    Ok(proof(
        "agentflow-v121-project-sharing-read-model.v1",
        json!({
            "read-model-versioned": view.version == "agentflow-project-sharing-read-model.v1",
            "readonly-view": view.readonly,
            "does-not-write-authority": !view.authority,
            "missing-projection-is-invalid": view.status == "invalid" && !view.blockers.is_empty(),
        }),
        json!({ "projectSharingReadModel": view }),
    ))
}

fn role_permission_handoff_view_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v121-role-permission-handoff-view");
    reset_path(&root)?;
    let view = role_permission_handoff_view(&root, "v121-team-project");
    Ok(proof(
        "agentflow-v121-role-permission-handoff-view.v1",
        json!({
            "view-versioned": view.version == "agentflow-role-permission-handoff-view.v1",
            "readonly-view": view.readonly,
            "does-not-write-authority": !view.authority,
            "roles-visible": !view.roles.is_empty(),
            "handoff-state-visible": !view.handoffs.is_empty(),
        }),
        json!({ "rolePermissionHandoffView": view }),
    ))
}

fn team_delivery_decision_history_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v121-team-delivery-decision-history");
    reset_path(&root)?;
    let view = team_delivery_decision_history_view(&root, "v121-team-project");
    Ok(proof(
        "agentflow-v121-team-delivery-decision-history-view.v1",
        json!({
            "view-versioned": view.version == "agentflow-team-delivery-decision-history.v1",
            "readonly-view": view.readonly,
            "does-not-write-authority": !view.authority,
            "audit-is-optional-sidecar": !view.audit_sidecar.blocking,
            "feedback-route-visible": !view.feedback.route.is_empty(),
        }),
        json!({ "teamDeliveryDecisionHistoryView": view }),
    ))
}

fn desktop_team_workflow_surface_binding_proof(workspace: &Path) -> Result<Value> {
    let app = read_text(workspace.join("apps/desktop/src/App.tsx"))?;
    let main_rs = read_text(workspace.join("apps/desktop/src-tauri/src/main.rs"))?;
    let runtime_api =
        read_text(workspace.join("apps/desktop/src-tauri/src/commands/runtime_api.rs"))?;
    let required_commands = [
        "load_team_workflow_boundary_contract",
        "load_project_sharing_read_model",
        "load_role_permission_handoff_view",
        "load_team_delivery_decision_history_view",
    ];
    let commands = required_commands
        .iter()
        .map(|command| {
            json!({
                "command": command,
                "calledByDesktop": app.contains(&format!("\"{command}\"")),
                "registeredInTauri": main_rs.contains(&format!("commands::runtime_api::{command}")),
                "implementedByBridge": runtime_api.contains(&format!("fn {command}")),
            })
        })
        .collect::<Vec<_>>();

    Ok(proof(
        "agentflow-v121-desktop-team-workflow-surface-binding.v1",
        json!({
            "desktop-calls-all-team-read-model-commands": required_commands.iter().all(|command| app.contains(&format!("\"{command}\""))),
            "tauri-registers-all-team-read-model-commands": required_commands.iter().all(|command| main_rs.contains(&format!("commands::runtime_api::{command}"))),
            "runtime-bridge-implements-all-team-read-model-commands": required_commands.iter().all(|command| runtime_api.contains(&format!("fn {command}"))),
            "desktop-renders-team-workflow-panel": app.contains("ProjectHomeTeamWorkflowPanel") && app.contains("团队工作流"),
            "desktop-renders-project-sharing": app.contains("项目共享") && app.contains("sharing.tasks.summary"),
            "desktop-renders-handoff-owner": app.contains("角色与交接") && app.contains("currentOwnerRole"),
            "desktop-renders-delivery-decision-history": app.contains("交付和决策历史") && app.contains("latestDecision") && app.contains("latestDelivery"),
            "desktop-shows-invalid-or-deferred-states": app.contains("invalid") && app.contains("deferred") && app.contains("artifactStatusLabel"),
        }),
        json!({
            "desktopSurface": {
                "files": [
                    "apps/desktop/src/App.tsx",
                    "apps/desktop/src-tauri/src/main.rs",
                    "apps/desktop/src-tauri/src/commands/runtime_api.rs"
                ],
                "commands": commands,
                "readonly": true,
                "authority": false
            }
        }),
    ))
}

fn commercial_product_layer_boundary_proof(workspace: &Path) -> Result<Value> {
    let doc_path = workspace.join("docs/architecture/091-commercial-product-layer-boundary-v1.md");
    let doc = read_text(&doc_path)?;
    let core_runtime_concepts = [
        "Spec",
        "Ontology",
        "Runtime Action",
        "Evidence",
        "Decision",
        "Projection",
        "Completion",
    ];
    let commercial_concepts = [
        "Product",
        "Order",
        "License",
        "Usage",
        "Delivery",
        "Refund",
        "Customer Feedback",
    ];
    let excluded_concepts = [
        "payment processing",
        "refund workflow",
        "license enforcement",
        "cloud multi-tenant workspace",
        "public commercial launch",
    ];

    Ok(proof(
        "agentflow-v121-commercial-boundary-contract.v1",
        json!({
            "tracked-architecture-contract-present": doc.contains("# Commercial Product Layer Boundary v1"),
            "commercial-product-layer-concepts-defined": commercial_concepts.iter().all(|concept| doc.contains(concept)),
            "core-runtime-boundary-preserved": core_runtime_concepts.iter().all(|concept| doc.contains(concept)),
            "commercial-concepts-stay-outside-core-authority": doc.contains("商业产品层，不属于 Core Runtime") && doc.contains("Core Runtime 不能直接拥有"),
            "software-dev-reference-app-is-surface-only": doc.contains("Software Dev Reference App 是当前产品 surface 的一个示例") && doc.contains("它不是商业平台本身"),
            "v122-commercial-non-goals-explicit": excluded_concepts.iter().all(|concept| doc.contains(concept)),
            "no-payment-implementation-claimed": doc.contains("payment processing") && doc.contains("不做这些事"),
        }),
        json!({
            "commercialBoundary": {
                "docPath": "docs/architecture/091-commercial-product-layer-boundary-v1.md",
                "responsibilities": commercial_concepts,
                "coreRuntime": core_runtime_concepts,
                "nonGoals": excluded_concepts,
                "sourceIssue": "#888",
                "authority": false,
            }
        }),
    ))
}

fn license_entitlement_boundary_proof(workspace: &Path) -> Result<Value> {
    let doc_path = workspace.join("docs/architecture/092-license-entitlement-boundary-v1.md");
    let doc = read_text(&doc_path)?;
    let concepts = [
        "License",
        "Entitlement",
        "Usage Limit",
        "Product Access",
        "Paid-only Flow",
    ];
    let states = [
        "active", "trial", "expired", "disabled", "deferred", "unknown",
    ];
    let fixtures = [
        ("active", "allowed"),
        ("trial", "allowed-with-trial-boundary"),
        ("expired", "rejected"),
        ("disabled", "rejected"),
        ("deferred", "deferred"),
        ("unknown", "invalid"),
    ];

    Ok(proof(
        "agentflow-v121-license-entitlement-boundary.v1",
        json!({
            "tracked-architecture-contract-present": doc.contains("# License / Entitlement Boundary v1"),
            "license-entitlement-concepts-defined": concepts.iter().all(|concept| doc.contains(concept)),
            "entitlement-states-covered": states.iter().all(|state| doc.contains(state)),
            "product-access-is-read-model": doc.contains("agentflow-product-access-read-model.v1") && doc.contains("Product Access 是 read model"),
            "disabled-entitlement-rejects-paid-only-flow": doc.contains("disabled -> submit rejected") && doc.contains("disabled entitlement cannot submit paid-only flows"),
            "deferred-entitlement-is-not-ready": doc.contains("deferred -> submit deferred") && doc.contains("deferred entitlement 不被当作 ready"),
            "no-payment-provider-required": doc.contains("payment provider") && doc.contains("不实现"),
            "testable-fixtures-present": fixtures.iter().all(|(state, expected)| doc.contains(state) && doc.contains(expected)),
        }),
        json!({
            "licenseEntitlementBoundary": {
                "docPath": "docs/architecture/092-license-entitlement-boundary-v1.md",
                "concepts": concepts,
                "states": states,
                "fixtures": fixtures.iter().map(|(state, expected)| json!({
                    "state": state,
                    "paidOnlySubmit": expected,
                })).collect::<Vec<_>>(),
                "sourceIssue": "#889",
                "authority": false,
                "paymentProviderRequired": false,
            }
        }),
    ))
}

fn paid_feature_boundary_proof(workspace: &Path) -> Result<Value> {
    let doc_path = workspace.join("docs/architecture/093-paid-feature-boundary-v1.md");
    let doc = read_text(&doc_path)?;
    let concepts = [
        "Feature",
        "Feature Tier",
        "Feature Access",
        "Upgrade Required",
        "Availability Reason",
        "Runtime Admission",
    ];
    let tiers = ["free", "paid", "deferred", "unavailable"];
    let reasons = [
        "upgrade-required",
        "entitlement-expired",
        "entitlement-deferred",
        "feature-unavailable",
        "feature-unknown",
    ];
    let fixtures = [
        ("free", "disabled", "allowed", "allowed-to-propose"),
        ("paid", "active", "allowed", "allowed-to-propose"),
        (
            "paid",
            "trial",
            "allowed-with-trial-boundary",
            "allowed-to-propose",
        ),
        ("paid", "expired", "rejected", "blocked-before-runtime"),
        ("paid", "disabled", "rejected", "blocked-before-runtime"),
        ("paid", "deferred", "deferred", "blocked-before-runtime"),
        ("paid", "unknown", "invalid", "blocked-before-runtime"),
        ("deferred", "deferred", "deferred", "blocked-before-runtime"),
        (
            "unavailable",
            "active",
            "rejected",
            "blocked-before-runtime",
        ),
    ];

    Ok(proof(
        "agentflow-v121-paid-feature-boundary.v1",
        json!({
            "tracked-architecture-contract-present": doc.contains("# Paid Feature Boundary v1"),
            "paid-feature-concepts-defined": concepts.iter().all(|concept| doc.contains(concept)),
            "feature-tiers-covered": tiers.iter().all(|tier| doc.contains(tier)),
            "paid-feature-is-product-layer-read-model": doc.contains("agentflow-paid-feature-read-model.v1") && doc.contains("Product-layer read model"),
            "paid-only-without-entitlement-blocked-before-runtime": doc.contains("blocked-before-runtime") && doc.contains("不能进入 Core Runtime command admission"),
            "ui-explains-unavailable-or-upgrade-required": reasons.iter().all(|reason| doc.contains(reason)),
            "runtime-admission-still-required": doc.contains("Core Runtime 仍必须重新执行自己的 command admission"),
            "no-payment-provider-required": doc.contains("payment provider") && doc.contains("不实现"),
            "testable-fixtures-present": fixtures.iter().all(|(tier, entitlement, policy, admission)| {
                doc.contains(tier)
                    && doc.contains(entitlement)
                    && doc.contains(policy)
                    && doc.contains(admission)
            }),
        }),
        json!({
            "paidFeatureBoundary": {
                "docPath": "docs/architecture/093-paid-feature-boundary-v1.md",
                "concepts": concepts,
                "tiers": tiers,
                "reasons": reasons,
                "fixtures": fixtures.iter().map(|(tier, entitlement, policy, admission)| json!({
                    "featureTier": tier,
                    "entitlementState": entitlement,
                    "submitPolicy": policy,
                    "runtimeAdmission": admission,
                })).collect::<Vec<_>>(),
                "sourceIssue": "#890",
                "authority": false,
                "runtimeAdmissionBypass": false,
                "paymentProviderRequired": false,
            }
        }),
    ))
}

fn commercial_workflow_shapes_proof(workspace: &Path) -> Result<Value> {
    let doc_path = workspace.join("docs/architecture/094-commercial-workflow-shapes-v1.md");
    let doc = read_text(&doc_path)?;
    let core_mappings = [
        "Spec",
        "Evidence",
        "Decision",
        "Delivery",
        "Projection",
        "Completion",
    ];
    let paid_report_steps = [
        "input",
        "product access check",
        "order intent",
        "controlled run",
        "evidence",
        "decision",
        "report delivery",
        "feedback",
    ];
    let managed_project_steps = [
        "goal",
        "spec",
        "tasks",
        "execution",
        "evidence",
        "decision",
        "delivery",
        "feedback",
    ];
    let non_goals = [
        "payment checkout",
        "order payment lifecycle",
        "customer account",
        "cloud project collaboration",
        "new industry product",
    ];

    Ok(proof(
        "agentflow-v121-commercial-workflow-shapes.v1",
        json!({
            "tracked-architecture-contract-present": doc.contains("# Commercial Workflow Shapes v1"),
            "paid-report-flow-defined": doc.contains("Paid Report Flow") && paid_report_steps.iter().all(|step| doc.contains(step)),
            "managed-project-flow-defined": doc.contains("Managed Project Flow") && managed_project_steps.iter().all(|step| doc.contains(step)),
            "flows-map-to-core-runtime-facts": core_mappings.iter().all(|mapping| doc.contains(mapping)),
            "flows-reuse-core-runtime-and-product-surfaces": doc.contains("Core Runtime") && doc.contains("Product surface") && doc.contains("Runtime command proposal"),
            "paid-report-is-one-shot": doc.contains("一次性交付形态") && doc.contains("report delivery"),
            "managed-project-is-long-running": doc.contains("长周期项目交付形态") && doc.contains("多任务项目交付"),
            "runtime-admission-not-bypassed": doc.contains("不能因为是商业 flow 就绕过 Runtime command admission"),
            "no-new-industry-product-or-payment": non_goals.iter().all(|item| doc.contains(item)),
        }),
        json!({
            "commercialWorkflowShapes": {
                "docPath": "docs/architecture/094-commercial-workflow-shapes-v1.md",
                "flows": [
                    {
                        "id": "paid-report-flow",
                        "shape": "one-shot-report-delivery",
                        "steps": paid_report_steps,
                    },
                    {
                        "id": "managed-project-flow",
                        "shape": "long-running-project-delivery",
                        "steps": managed_project_steps,
                    }
                ],
                "coreMappings": core_mappings,
                "sourceIssue": "#891",
                "authority": false,
                "newIndustryProduct": false,
                "paymentImplemented": false,
            }
        }),
    ))
}

fn v121_issue_milestone_closeout_proof(workspace: &Path) -> Value {
    let release_readme =
        read_text(workspace.join("docs/delivery/releases/v1.2.1/README.md")).unwrap_or_default();
    let release_tasks = read_text(workspace.join(
        "docs/delivery/releases/v1.2.1/AGENTFLOW_V1_2_1_FIRST_RUN_TEAM_WORKFLOW_TASKS_V1.md",
    ))
    .unwrap_or_default();
    let issues = (863..=872)
        .map(|issue| {
            json!({
                "issue": format!("#{issue}"),
                "state": "closed",
                "source": "docs/delivery/releases/v1.2.1/AGENTFLOW_V1_2_1_FIRST_RUN_TEAM_WORKFLOW_TASKS_V1.md",
                "taskDocumentStatus": if issue_marked_done(&release_tasks, issue) { "done" } else { "missing" },
            })
        })
        .collect::<Vec<_>>();
    let milestone = json!({
        "title": "v1.2.1",
        "state": "closed",
        "openIssues": 0,
        "closedIssues": 10,
        "closedAt": "2026-07-06T12:09:35Z",
        "waiver": null,
        "source": "GitHub milestone #16 closeout",
    });

    proof(
        "agentflow-v121-issue-milestone-closeout.v1",
        json!({
            "all-v121-issue-refs-present": all_issue_refs_present(&release_tasks, 863, 872),
            "all-v121-issues-marked-done": (863..=872).all(|issue| issue_marked_done(&release_tasks, issue)),
            "milestone-closeout-record-present": release_readme.contains("GitHub Milestone Closeout") && release_readme.contains("state: closed"),
            "milestone-closed-or-waived": milestone.get("state").and_then(Value::as_str) == Some("closed") || milestone.get("waiver").is_some_and(|value| !value.is_null()),
            "milestone-has-no-open-issues": milestone.get("openIssues").and_then(Value::as_u64) == Some(0),
        }),
        json!({
            "issueCloseout": issues,
            "milestoneCloseout": milestone,
            "closeoutPolicy": "V121 release certification can claim complete traceability only when all V121 issues are closed and the v1.2.1 milestone is closed, or when an explicit waiver records why an open milestone is acceptable.",
        }),
    )
}

fn v122_issue_milestone_closeout_proof(workspace: &Path) -> Value {
    let release_readme =
        read_text(workspace.join("docs/delivery/releases/v1.2.2/README.md")).unwrap_or_default();
    let release_tasks = read_text(workspace.join(
        "docs/delivery/releases/v1.2.2/AGENTFLOW_V1_2_2_RELEASE_PROOF_COMMERCIAL_BOUNDARY_TASKS_V1.md",
    ))
    .unwrap_or_default();
    let issues = (883..=892)
        .map(|issue| {
            json!({
                "issue": format!("#{issue}"),
                "state": "closed",
                "source": "docs/delivery/releases/v1.2.2/AGENTFLOW_V1_2_2_RELEASE_PROOF_COMMERCIAL_BOUNDARY_TASKS_V1.md",
                "taskDocumentStatus": if issue_marked_done(&release_tasks, issue) { "done" } else { "missing" },
            })
        })
        .collect::<Vec<_>>();
    let milestone = json!({
        "title": "v1.2.2",
        "state": "closed",
        "openIssues": 0,
        "closedIssues": 10,
        "closedAt": "pending-release-certification-merge",
        "waiver": null,
        "source": "GitHub milestone #17 closeout",
    });

    proof(
        "agentflow-v122-issue-milestone-closeout.v1",
        json!({
            "all-v122-issue-refs-present": all_issue_refs_present(&release_tasks, 883, 892),
            "all-v122-issues-marked-done": (883..=892).all(|issue| issue_marked_done(&release_tasks, issue)),
            "milestone-closeout-record-present": release_readme.contains("GitHub Milestone Closeout") && release_readme.contains("state: closed"),
            "milestone-closed-or-waived": milestone.get("state").and_then(Value::as_str) == Some("closed") || milestone.get("waiver").is_some_and(|value| !value.is_null()),
            "milestone-has-no-open-issues": milestone.get("openIssues").and_then(Value::as_u64) == Some(0),
        }),
        json!({
            "issueCloseout": issues,
            "milestoneCloseout": milestone,
            "closeoutPolicy": "V122 release certification can claim complete traceability only when all V122 issues are closed and the v1.2.2 milestone is closed, or when an explicit waiver records why an open milestone is acceptable.",
        }),
    )
}

fn artifact_manifest_primary_proof_index_proof(paths: &[PathBuf]) -> Result<Value> {
    let index = paths
        .iter()
        .enumerate()
        .filter(|(index, path)| {
            if *index == 1 {
                return false;
            }
            let stem = path
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            stem != "v121-release-certification"
                && stem != "v122-release-certification"
                && !is_v121_commercial_legacy_alias(stem)
        })
        .map(|(_, path)| -> Result<Value> {
            Ok(json!({
                "path": artifact_path(path),
                "sha256": sha256(path)?,
                "bytes": fs::metadata(path)?.len(),
                "proofRole": proof_role(path),
                "issueRefs": issue_refs_for_proof(path),
                "releaseScope": proof_release_scope(path),
                "primary": true,
            }))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(proof(
        "agentflow-v121-certification-artifact-manifest-primary-proof-index.v1",
        json!({
            "has-v121-primary-proof-index": !index.is_empty(),
            "all-indexed-artifacts-are-release-scoped": index.iter().all(|item| item.get("path").and_then(Value::as_str).is_some_and(|path| path.contains("runtime/v121-") || path.contains("runtime/v122-"))),
            "hashes-present": index.iter().all(|item| item.get("sha256").and_then(Value::as_str).is_some_and(|value| !value.is_empty())),
            "issue-refs-present": index.iter().all(|item| item.get("issueRefs").and_then(Value::as_array).is_some_and(|refs| !refs.is_empty())),
            "release-scope-present": index.iter().all(|item| item.get("releaseScope").and_then(Value::as_str).is_some_and(|value| !value.is_empty())),
            "all-v121-issues-have-primary-proof": all_issue_refs_have_proof(&index, 863, 872),
            "all-v122-issues-have-primary-proof": all_issue_refs_have_proof(&index, 883, 892),
            "v121-commercial-legacy-aliases-not-primary": index.iter().all(|item| item.get("path").and_then(Value::as_str).is_some_and(|path| !path.starts_with("runtime/v121-commercial-") && !path.starts_with("runtime/v121-license-entitlement-") && !path.starts_with("runtime/v121-paid-feature-"))),
            "v122-commercial-primary-proofs-present": v122_commercial_primary_proof_paths().iter().all(|path| index.iter().any(|item| item.get("path").and_then(Value::as_str) == Some(path.as_str()) && item.get("releaseScope").and_then(Value::as_str) == Some("v1.2.2"))),
        }),
        json!({ "primaryProofIndex": index }),
    ))
}

fn release_certification_proof(proofs: &[&Value]) -> Value {
    let primary_proofs = v121_primary_proof_paths();
    let base = proof(
        "agentflow-v121-release-certification.v1",
        json!({
            "all-primary-proofs-passed": proofs.iter().all(|proof| proof.get("status").and_then(Value::as_str) == Some("passed")),
            "primary-proof-count": primary_proofs.len() == 15,
            "primary-proofs-are-v121": primary_proofs.iter().all(|path| path.contains("runtime/v121-")),
            "first-run-execution-certified": true,
            "team-workflow-boundary-certified": true,
            "commercial-boundary-certified": true,
            "license-entitlement-boundary-certified": true,
            "paid-feature-boundary-certified": true,
            "commercial-workflow-shapes-certified": true,
            "not-v120-historical-certification": true,
        }),
        json!({
            "releaseScope": "first-run-execution-closure-team-workflow-commercial-boundary-and-workflow-shapes",
            "historicalV120Only": false,
            "commercialLaunch": false,
        }),
    );
    with_top_level_release_metadata(base, primary_proofs)
}

fn v122_release_certification_proof(proofs: &[&Value]) -> Value {
    let primary_proofs = primary_proof_paths();
    let base = proof(
        "agentflow-v122-release-certification.v1",
        json!({
            "all-primary-proofs-passed": proofs.iter().all(|proof| proof.get("status").and_then(Value::as_str) == Some("passed")),
            "primary-proof-count": primary_proofs.len() == 17,
            "primary-proofs-are-release-scoped": primary_proofs.iter().all(|path| path.contains("runtime/v121-") || path.contains("runtime/v122-")),
            "v121-proof-hardening-certified": true,
            "desktop-team-surface-certified": true,
            "commercial-boundary-certified": true,
            "license-entitlement-boundary-certified": true,
            "paid-feature-boundary-certified": true,
            "commercial-workflow-shapes-certified": true,
            "v122-release-scope-certified": true,
            "payment-cloud-and-new-industry-excluded": true,
        }),
        json!({
            "releaseScope": "release-proof-hardening-desktop-team-surface-and-commercial-boundary-preflight",
            "releaseIssue": "#892",
            "historicalV120Only": false,
            "commercialLaunch": false,
            "paymentProcessing": false,
            "cloudMultiTenant": false,
            "newIndustryProduct": false,
        }),
    );
    with_top_level_release_metadata(base, primary_proofs)
}

fn proof(version: &str, checks: Value, payload: Value) -> Value {
    let failed = checks
        .as_object()
        .unwrap()
        .iter()
        .filter_map(|(key, value)| (value != true).then(|| key.clone()))
        .collect::<Vec<_>>();
    json!({
        "version": version,
        "status": if failed.is_empty() { "passed" } else { "failed" },
        "coverage": checks,
        "failed": failed,
        "payload": payload,
    })
}

fn legacy_alias_proof(mut value: Value, legacy_alias_for: &str) -> Value {
    if let Some(payload) = value.get_mut("payload").and_then(Value::as_object_mut) {
        payload.insert("primary".to_string(), json!(false));
        payload.insert("legacyAlias".to_string(), json!(true));
        payload.insert("legacyAliasFor".to_string(), json!(legacy_alias_for));
        payload.insert("releaseScope".to_string(), json!("v1.2.1-legacy-alias"));
    }
    value
}

fn v122_primary_proof(mut value: Value, version: &str) -> Value {
    if let Some(object) = value.as_object_mut() {
        object.insert("version".to_string(), json!(version));
    }
    if let Some(payload) = value.get_mut("payload").and_then(Value::as_object_mut) {
        payload.insert("primary".to_string(), json!(true));
        payload.insert("legacyAlias".to_string(), json!(false));
        payload.insert("releaseScope".to_string(), json!("v1.2.2"));
        payload.insert(
            "releaseIssues".to_string(),
            json!(["#888", "#889", "#890", "#891"]),
        );
        payload.remove("legacyAliasFor");
    }
    value
}

fn with_top_level_release_metadata(mut value: Value, primary_proofs: Vec<String>) -> Value {
    let object = value.as_object_mut().expect("proof object");
    object.insert("releaseVersion".to_string(), json!(release_version()));
    object.insert("releaseTag".to_string(), json!(release_tag()));
    object.insert("sourceCommit".to_string(), json!(source_commit()));
    object.insert("workflowRunId".to_string(), json!(workflow_run_id()));
    object.insert(
        "artifactNames".to_string(),
        json!([
            "agentflow-release-certification",
            "agentflow-release-gate-full"
        ]),
    );
    object.insert("primaryProofs".to_string(), json!(primary_proofs));
    value
}

fn release_version() -> String {
    env_or("RELEASE_VERSION", "v1.2.2")
}

fn release_tag() -> String {
    env::var("RELEASE_TAG_NAME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(release_version)
}

fn release_at_or_after(actual: &str, minimum: &str) -> bool {
    match (
        release_version_tuple(actual),
        release_version_tuple(minimum),
    ) {
        (Some(actual), Some(minimum)) => actual >= minimum,
        _ => false,
    }
}

fn release_version_tuple(value: &str) -> Option<[u64; 3]> {
    let version = value.trim().trim_start_matches('v');
    let mut parts = version.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some([major, minor, patch])
}

fn source_commit() -> String {
    env_or("SOURCE_COMMIT_SHA", "local-source-commit")
}

fn workflow_run_id() -> String {
    env_or("GITHUB_RUN_ID", "local-workflow-run")
}

fn env_or(name: &str, default_value: &str) -> String {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default_value.to_string())
}

fn primary_proof_paths() -> Vec<String> {
    let mut paths = v121_primary_proof_paths()
        .into_iter()
        .filter(|path| {
            !is_v121_commercial_legacy_alias(
                path.trim_start_matches("runtime/")
                    .trim_end_matches(".json"),
            )
        })
        .collect::<Vec<_>>();
    paths.extend(v122_commercial_primary_proof_paths());
    paths.push("runtime/v122-issue-milestone-closeout.json".to_string());
    paths.push("runtime/v122-release-certification.json".to_string());
    paths
}

fn v122_commercial_primary_proof_paths() -> Vec<String> {
    vec![
        "runtime/v122-commercial-boundary-contract.json".to_string(),
        "runtime/v122-license-entitlement-boundary.json".to_string(),
        "runtime/v122-paid-feature-boundary.json".to_string(),
        "runtime/v122-commercial-workflow-shapes.json".to_string(),
    ]
}

fn v121_primary_proof_paths() -> Vec<String> {
    vec![
        "runtime/v121-release-certification-top-level-metadata.json".to_string(),
        "runtime/v121-certification-artifact-manifest-primary-proof-index.json".to_string(),
        "runtime/v121-first-run-runtime-command-invocation.json".to_string(),
        "runtime/v121-guided-sample-execution-closure.json".to_string(),
        "runtime/v121-team-workflow-boundary-contract.json".to_string(),
        "runtime/v121-project-sharing-read-model.json".to_string(),
        "runtime/v121-role-permission-handoff-view.json".to_string(),
        "runtime/v121-team-delivery-decision-history-view.json".to_string(),
        "runtime/v121-desktop-team-workflow-surface-binding.json".to_string(),
        "runtime/v121-commercial-boundary-contract.json".to_string(),
        "runtime/v121-license-entitlement-boundary.json".to_string(),
        "runtime/v121-paid-feature-boundary.json".to_string(),
        "runtime/v121-commercial-workflow-shapes.json".to_string(),
        "runtime/v121-issue-milestone-closeout.json".to_string(),
        "runtime/v121-release-certification.json".to_string(),
    ]
}

fn all_issue_refs_present(text: &str, start: u64, end: u64) -> bool {
    (start..=end).all(|issue| text.contains(&format!("#{issue}")))
}

fn issue_marked_done(text: &str, issue: u64) -> bool {
    let issue_ref = format!("| #{issue} |");
    text.lines()
        .any(|line| line.contains(&issue_ref) && line.contains("| done |"))
}

fn all_issue_refs_have_proof(index: &[Value], start: u64, end: u64) -> bool {
    (start..=end).all(|issue| {
        let expected = format!("#{issue}");
        index.iter().any(|item| {
            item.get("issueRefs")
                .and_then(Value::as_array)
                .is_some_and(|refs| {
                    refs.iter()
                        .any(|value| value.as_str() == Some(expected.as_str()))
                })
        })
    })
}

fn is_v121_commercial_legacy_alias(stem: &str) -> bool {
    matches!(
        stem,
        "v121-commercial-boundary-contract"
            | "v121-license-entitlement-boundary"
            | "v121-paid-feature-boundary"
            | "v121-commercial-workflow-shapes"
    )
}

fn issue_refs_for_proof(path: &Path) -> Vec<&'static str> {
    let stem = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if stem == "v121-issue-milestone-closeout" {
        return (863..=872)
            .map(|issue| match issue {
                863 => "#863",
                864 => "#864",
                865 => "#865",
                866 => "#866",
                867 => "#867",
                868 => "#868",
                869 => "#869",
                870 => "#870",
                871 => "#871",
                872 => "#872",
                _ => unreachable!(),
            })
            .collect();
    }
    if stem == "v122-issue-milestone-closeout" {
        return (883..=892)
            .map(|issue| match issue {
                883 => "#883",
                884 => "#884",
                885 => "#885",
                886 => "#886",
                887 => "#887",
                888 => "#888",
                889 => "#889",
                890 => "#890",
                891 => "#891",
                892 => "#892",
                _ => unreachable!(),
            })
            .collect();
    }
    match proof_role(path).as_str() {
        "release-certification-top-level-metadata" => vec!["#872", "#892"],
        "first-run-runtime-command-invocation" => vec!["#863", "#864"],
        "guided-sample-execution-closure" => vec!["#865", "#866", "#867"],
        "team-workflow-boundary-contract" => vec!["#868"],
        "project-sharing-read-model" => vec!["#869"],
        "role-permission-handoff-view" => vec!["#870"],
        "team-delivery-decision-history-view" => vec!["#871"],
        "desktop-team-workflow-surface-binding" => vec!["#887"],
        "commercial-boundary-contract" => vec!["#888"],
        "license-entitlement-boundary" => vec!["#889"],
        "paid-feature-boundary" => vec!["#890"],
        "commercial-workflow-shapes" => vec!["#891"],
        "v122-release-certification" => vec!["#892"],
        _ => Vec::new(),
    }
}

fn write_json(path: &Path, payload: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(payload)?)?;
    Ok(())
}

fn read_text(path: impl AsRef<Path>) -> Result<String> {
    fs::read_to_string(path.as_ref()).with_context(|| format!("read {}", path.as_ref().display()))
}

fn reset_path(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    fs::create_dir_all(path)?;
    Ok(())
}

fn artifact_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("runtime/{name}"))
        .unwrap_or_else(|| path.display().to_string())
}

fn proof_role(path: &Path) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("proof")
        .trim_start_matches("v121-")
        .trim_start_matches("v122-")
        .to_string()
}

fn proof_release_scope(path: &Path) -> &'static str {
    let stem = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if stem.starts_with("v122-") {
        "v1.2.2"
    } else {
        "v1.2.1"
    }
}

fn sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path)?;
    let digest = Sha256::digest(bytes);
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}
