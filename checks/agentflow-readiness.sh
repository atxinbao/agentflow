#!/usr/bin/env bash
set -euo pipefail

test -f docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md
test -f docs/specs/workflow-state-machine-v0.md
test -f docs/specs/workflow-control-core-v0.md
test -f docs/specs/project-audit-docs-refresh-boundary.md
test -f docs/specs/product-feature-creation-flow-v0.md
test -f docs/specs/product-feature-execution-flow-v0.md
test -f docs/specs/product-feature-controlled-run-v0.md
test -f docs/specs/goal-criteria-driven-mvp.md
test -f docs/specs/project-issue-status-model-v0.md
test -f docs/specs/team-project-milestone-issue-writers-v0.md
rg -n "Product Thesis|Core Entity Model|Project State Machine|Milestone State Machine|Issue State Machine|Eligibility Engine|Single Code-Changing Issue Rule|Lease / Lock Contract|Execution Run Contract|PR / Checks Contract|Evidence Contract|Milestone Review Gate|Project Closure Contract|Human Gates|Command Boundary|Local File Mapping|Event Log Contract|Minimal Data Model|PRD Acceptance Criteria for v1|ARC Review Checklist|AIE Implementation Brief|Verification Matrix" docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md >/dev/null
rg -n "Workflow State Machine v0|agentflow state check|workflow-state.json|WORKFLOW-STATE-SUMMARY.md|WorkflowStateSnapshot|WorkflowTransitionGuard|write_workflow_state_check|Eligibility Engine v0 边界定义" docs/specs/workflow-state-machine-v0.md README.md ROADMAP.md docs/specs/mvp-spec.md docs/planning/construction-plan.md docs/validation/latest-verification-summary.md crates/agentflow-core/src/lib.rs crates/agentflow-cli/src/main.rs >/dev/null
rg -n "Workflow Control Core v0|agentflow eligibility|agentflow lease|eligibility.json|leases.json|ELIGIBILITY-SUMMARY.md|LEASE-SUMMARY.md|WorkflowEligibilitySnapshot|WorkflowLeaseRecord|WorkflowLeaseSnapshot|acquire_workflow_lease|release_workflow_lease_after_review|Project Audit / Docs Refresh v0" docs/specs/workflow-control-core-v0.md README.md ROADMAP.md docs/specs/mvp-spec.md docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md docs/planning/construction-plan.md docs/validation/latest-verification-summary.md crates/agentflow-core/src/lib.rs crates/agentflow-cli/src/main.rs >/dev/null
rg -n "Project Audit / Docs Refresh v0|Code Audit|Root Docs Refresh|Final Evidence Summary|Human Final Approval|active -> audit -> docs-refresh -> final-review -> done|Project Closure State v0 实现|\\.agentflow/audits|不能从 active 直接 done" docs/specs/project-audit-docs-refresh-boundary.md README.md ROADMAP.md docs/specs/mvp-spec.md docs/specs/workflow-control-core-v0.md docs/specs/agentflow-ai-delivery-workflow-contract-v1.md docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md docs/planning/construction-plan.md docs/validation/latest-verification-summary.md >/dev/null
rg -n "Project Closure State v0|agentflow project closure|project-closure.json|PROJECT-CLOSURE-SUMMARY.md|ProjectClosureStateSnapshot|ProjectClosureGate|ProjectClosureCounts|write_project_closure_state|audit-ready|can_mark_done=false|Project Code Audit Snapshot v0" README.md ROADMAP.md docs/specs/mvp-spec.md docs/specs/project-audit-docs-refresh-boundary.md docs/specs/workflow-control-core-v0.md docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md docs/planning/construction-plan.md docs/validation/latest-verification-summary.md crates/agentflow-core/src/lib.rs crates/agentflow-cli/src/main.rs >/dev/null
rg -n "Project Code Audit Snapshot v0|agentflow project code-audit|project-code-audit.json|PROJECT-CODE-AUDIT-SUMMARY.md|ProjectCodeAuditSnapshot|ProjectCodeAuditCheck|ProjectCodeAuditFinding|write_project_code_audit_snapshot|snapshot-ready|Root Docs Refresh Snapshot v0" README.md ROADMAP.md docs/specs/mvp-spec.md docs/specs/project-audit-docs-refresh-boundary.md docs/planning/construction-plan.md docs/planning/mvp-productization-project.md docs/validation/latest-verification-summary.md crates/agentflow-core/src/lib.rs crates/agentflow-cli/src/main.rs >/dev/null
rg -n "Root Docs Refresh Snapshot v0|agentflow project docs-refresh|project-docs-refresh.json|PROJECT-DOCS-REFRESH-SUMMARY.md|ProjectDocsRefreshSnapshot|ProjectDocsRefreshCheckedDoc|ProjectDocsRefreshRequiredUpdate|write_project_docs_refresh_snapshot|Product Feature Creation Flow v0" README.md ROADMAP.md docs/specs/mvp-spec.md docs/specs/project-audit-docs-refresh-boundary.md docs/planning/construction-plan.md docs/planning/mvp-productization-project.md docs/validation/latest-verification-summary.md crates/agentflow-core/src/lib.rs crates/agentflow-cli/src/main.rs >/dev/null
rg -n "Product Feature Creation Flow v0|agentflow feature create|ProductFeatureDraft|ProductFeatureProject|ProductFeatureMilestoneDraft|ProductFeatureIssueDraft|ProductFeatureCreationSnapshot|FEATURE-CREATION-SUMMARY.md|product-feature-creation-flow-v0|riskLevel|rollbackPlan" docs/specs/product-feature-creation-flow-v0.md README.md ROADMAP.md docs/specs/mvp-spec.md docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md docs/specs/workflow-control-core-v0.md docs/validation/latest-verification-summary.md verification.md crates/agentflow-core/src/lib.rs crates/agentflow-cli/src/main.rs >/dev/null
rg -n "Product Feature Execution Flow v0|agentflow feature status|agentflow feature next|ProductFeatureExecutionSnapshot|ProductFeatureExecutionMilestone|ProductFeatureExecutionIssue|read_product_feature_execution_status|read_product_feature_execution_next|feature-0043|ISSUE-0043" docs/specs/product-feature-execution-flow-v0.md README.md ROADMAP.md docs/specs/mvp-spec.md docs/specs/product-feature-creation-flow-v0.md docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md docs/specs/agentflow-ai-delivery-workflow-contract-v1.md docs/specs/workflow-control-core-v0.md docs/validation/latest-verification-summary.md verification.md crates/agentflow-core/src/lib.rs crates/agentflow-cli/src/main.rs >/dev/null
rg -n "Product Feature Controlled Run v0|ControlledRunPlan|runPlan|dry-run recorded|latest run plan|blocked files|expected files|evidence requirements|agentflow run ISSUE-0043 --dry-run|product-feature-controlled-run-v0" docs/specs/product-feature-controlled-run-v0.md README.md ROADMAP.md docs/specs/mvp-spec.md docs/specs/product-feature-execution-flow-v0.md docs/specs/workflow-control-core-v0.md docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md docs/validation/latest-verification-summary.md verification.md crates/agentflow-core/src/lib.rs crates/agentflow-cli/src/main.rs >/dev/null
rg -n "Goal \\+ Criteria Driven MVP|goal-criteria-driven-mvp|Project / Issue Status Model v0|draft / active / paused / completed / canceled|backlog / todo / in_progress / in_review / done / canceled|Milestone 不维护独立状态|Team / Project / Milestone / Issue|当前 MVP 不把 Agent 自动执行流程作为主产品目标|用户和 Agent 可以共同" docs/specs/goal-criteria-driven-mvp.md GOAL.md .agentflow/goal.md README.md ROADMAP.md docs/specs/mvp-spec.md docs/planning/construction-plan.md docs/validation/latest-verification-summary.md verification.md >/dev/null
rg -n "Project / Issue Status Model v0|ProjectStatus|IssueStatus|MilestoneDerivedProgress|canonicalStatus|legacy status|Product Feature Project 默认 status = active|Product Feature Issue 默认 status = todo|agentflow plan 新建 Issue 默认 status = todo|Milestone 不作为产品状态机|derived progress" docs/specs/project-issue-status-model-v0.md README.md ROADMAP.md docs/specs/mvp-spec.md docs/specs/goal-criteria-driven-mvp.md docs/validation/latest-verification-summary.md verification.md crates/agentflow-core/src/lib.rs crates/agentflow-cli/src/main.rs apps/desktop/src/App.tsx apps/desktop/src/types.ts >/dev/null
rg -n "Team / Project / Milestone / Issue Writers v0|TeamDraft|ProjectDraft|MilestoneDraft|IssueDraft|CreationPreview|CreationWriteSummary|agentflow team create|agentflow project create|agentflow milestone create|agentflow issue create|preview-default|explicit-write-flag|Project default status = draft|Issue default status = todo|Milestone status = 不写入|team_project_milestone_issue_writers" docs/specs/team-project-milestone-issue-writers-v0.md README.md ROADMAP.md docs/specs/mvp-spec.md docs/specs/goal-criteria-driven-mvp.md docs/validation/latest-verification-summary.md verification.md crates/agentflow-core/src/lib.rs crates/agentflow-cli/src/main.rs >/dev/null
cargo test -p agentflow-core team_project_milestone_issue_writers -- --nocapture >/dev/null

rg -n "AEP Goal Initialization Protocol|Goal Loop Orchestrator|Desktop Workbench MVP v0|Local Pro Experiments v0|Local Metrics Snapshot v0|Local Search v0|Local Search Reader v0|Saved Query v0|Desktop Search Read-only View v0|Saved Query Writer v0|Local Workspace / Team / Project Model v0|Local Project Model v0|Desktop Project View v0|Desktop Workspace Overview v0|Local Project Seed v0|Issue Project Link v0|Issue Project Link Writer v0|Project-aware GoalLoop v0|Desktop GoalLoop Trace v0|Desktop Issue Lifecycle Trace v0|Desktop Project Update Timeline v0|Desktop MVP Navigation Scope Reduction v0|Desktop Team Hierarchy v0|Desktop Team Parent Child Columns v0|Desktop Workspace Sidebar Tree v0|Desktop Teams Add Button v0|MVP Productization Project v0|Milestone-aware Issue Planning v0|MVP Execution Loop v0|MVP Minimal Workflow v0|AgentFlow AI Delivery Workflow Contract v1|Workflow State Machine|Workflow Control Core|Eligibility Engine|Lease / Lock|Execution Evidence|Milestone / Project Closure|Project Audit / Docs Refresh v0|Project Closure State v0|Product Feature Creation Flow v0|Product Feature Execution Flow v0" README.md ROADMAP.md docs/specs/mvp-spec.md docs/specs/agentflow-ai-delivery-workflow-contract-v1.md docs/specs/workflow-control-core-v0.md docs/specs/project-audit-docs-refresh-boundary.md docs/specs/product-feature-creation-flow-v0.md docs/specs/product-feature-execution-flow-v0.md docs/specs/desktop-workbench-mvp-boundary.md docs/specs/local-pro-experiments-boundary.md docs/specs/local-search-boundary.md docs/specs/saved-query-boundary.md docs/specs/saved-query-writer-boundary.md docs/specs/desktop-search-readonly-boundary.md docs/specs/local-workspace-project-model-boundary.md docs/specs/local-project-seed-boundary.md docs/specs/issue-project-link-boundary.md docs/specs/project-aware-goalloop-boundary.md docs/validation/latest-verification-summary.md >/dev/null
rg -n "project-definition.json|scope-state.json|bootstrap/project-bootstrap-sequence.md|goal-loop.json|GOAL-LOOP-SUMMARY.md|DesktopWorkbenchSnapshot|read_desktop_workbench_snapshot|projectUpdates|LocalMetricsSnapshot|read_local_metrics_snapshot|LocalSearchSnapshot|read_local_search_snapshot|LocalProjectModelSnapshot|read_local_project_model_snapshot|LocalProjectIssueRef|executionState|latestRunStatus|evidencePath|reviewPath|projectUpdatePath|LocalProjectSeedPreview|read_local_project_seed_preview|write_local_project_seed|IssueProjectLinkPreview|read_issue_project_link_preview|write_issue_project_link|prepare_plan_project_seed_update|PlanProjectSeedUpdate|milestone-aware-issue-planning-v0|project_aware_candidate_intent|active_project_id_from_seed|milestone_next_issue_intent|ActiveMilestoneQueue|active_milestone_queue|write_milestone_summary_if_complete|Milestone Evidence Summary|queue preflight|AgentFlow AI Delivery Workflow Contract v1|ProjectDeliveryContract|MilestoneDeliveryContract|IssueDeliveryContract|EligibilitySnapshot|ExecutionEvidence|Human Gates|@003 / PRD|@005 / ARC|@000 / AIE|agentflow projects|agentflow project-seed|agentflow issue-link|SavedQueryDefinition|Saved Query v0|Desktop Search Read-only View v0|Saved Query Writer v0|Desktop Project View v0|Desktop Workspace Overview v0|Desktop GoalLoop Trace v0|Desktop Issue Lifecycle Trace v0|LocalProjectSeed|LocalWorkspace|LocalTeam|LocalProject|Milestone|GoalLoopSelection|IssueProjectLink|projectLink|teamId|projectId|milestoneId|linkSource|confirmationGates|resultPersistence|read-only badge|source trace|\\.agentflow/search|\\.agentflow/queries|\\.agentflow/workspace.json|\\.agentflow/teams|\\.agentflow/projects|Issue Project Link v0|Issue Project Link Writer v0|Project-aware GoalLoop v0|activeProjectId|activeMilestoneId|nextIssueIntent|roadmap candidate|wait-human|project update" docs/specs/mvp-spec.md docs/specs/agentflow-ai-delivery-workflow-contract-v1.md docs/specs/local-pro-experiments-boundary.md docs/specs/local-search-boundary.md docs/specs/saved-query-boundary.md docs/specs/saved-query-writer-boundary.md docs/specs/desktop-search-readonly-boundary.md docs/specs/local-workspace-project-model-boundary.md docs/specs/local-project-seed-boundary.md docs/specs/issue-project-link-boundary.md docs/specs/project-aware-goalloop-boundary.md docs/specs/desktop-workbench-mvp-boundary.md crates/agentflow-core/src/lib.rs crates/agentflow-cli/src/main.rs apps/desktop/src/types.ts >/dev/null
rg -n "load_workbench_snapshot|load_metrics_snapshot|load_search_snapshot|load_project_model_snapshot|GoalLoopTraceView|IssueLifecycleTraceView|ProjectUpdateTimelineView|TeamView|WorkspaceTreeNav|workspace-tree-nav|tree-parent|tree-child|tree-add-button|新增团队|初始化创建入口|Desktop GoalLoop Trace v0|Desktop Issue Lifecycle Trace v0|Desktop Project Update Timeline v0|团队入口|Workspace 入口|team-relation-grid|父级栏目|子级栏目|团队是父级栏目|workspace|总览|团队|项目|任务|队列预检|项目 / 里程碑|任务合同|只读|推荐命令" apps/desktop/src-tauri/src/main.rs apps/desktop/src/App.tsx >/dev/null
test ! -d .agentflow/search
test ! -d .agentflow/queries
test ! -d .agentflow/audits
test -f .agentflow/workspace.json
test -f .agentflow/teams/core.json
test -f .agentflow/projects/agentflow-local-execution.json
node -e 'const fs=require("fs"); const linked=[]; for (const f of fs.readdirSync(".agentflow/issues").filter((x)=>x.endsWith(".json"))) { const j=JSON.parse(fs.readFileSync(`.agentflow/issues/${f}`,"utf8")); for (const k of ["teamId","projectId","milestoneId","linkSource"]) { if (Object.prototype.hasOwnProperty.call(j,k)) throw new Error(`${f} has top-level ${k}`); } if (j.projectLink) { for (const k of ["teamId","projectId","milestoneId","linkSource"]) { if (!j.projectLink[k]) throw new Error(`${f} projectLink missing ${k}`); } linked.push(j.id); } } for (const id of ["ISSUE-0037","ISSUE-0038","ISSUE-0039"]) { if (!linked.includes(id)) throw new Error(`${id} missing projectLink`); }'
cargo run -p agentflow-cli -- goal check >/dev/null
cargo run -p agentflow-cli -- goal next >/dev/null
cargo run -p agentflow-cli -- state check >/dev/null
test -f .agentflow/state/workflow-state.json
test -f .agentflow/updates/WORKFLOW-STATE-SUMMARY.md
cargo run -p agentflow-cli -- eligibility >/dev/null
test -f .agentflow/state/eligibility.json
test -f .agentflow/updates/ELIGIBILITY-SUMMARY.md
cargo run -p agentflow-cli -- lease >/dev/null
test -f .agentflow/state/leases.json
test -f .agentflow/updates/LEASE-SUMMARY.md
cargo run -p agentflow-cli -- project closure >/dev/null
test -f .agentflow/state/project-closure.json
test -f .agentflow/updates/PROJECT-CLOSURE-SUMMARY.md
cargo run -p agentflow-cli -- project code-audit >/dev/null
test -f .agentflow/state/project-code-audit.json
test -f .agentflow/updates/PROJECT-CODE-AUDIT-SUMMARY.md
cargo run -p agentflow-cli -- project docs-refresh >/dev/null
test -f .agentflow/state/project-docs-refresh.json
test -f .agentflow/updates/PROJECT-DOCS-REFRESH-SUMMARY.md
cargo run -p agentflow-cli -- feature create "Readiness Product Feature" | rg -n "status=todo" >/dev/null
cargo run -p agentflow-cli -- team create "Readiness Demo Team" >/dev/null
cargo run -p agentflow-cli -- project create "Readiness Demo Project" >/dev/null
cargo run -p agentflow-cli -- milestone create "Readiness Demo Milestone" >/dev/null
cargo run -p agentflow-cli -- issue create "Readiness Demo Issue" >/dev/null
cargo run -p agentflow-cli -- feature status >/dev/null
cargo run -p agentflow-cli -- feature next >/dev/null
cargo run -p agentflow-cli -- metrics >/dev/null
cargo run -p agentflow-cli -- projects >/dev/null
cargo run -p agentflow-cli -- project-seed >/dev/null
cargo run -p agentflow-cli -- issue-link ISSUE-0025 >/dev/null
cargo run -p agentflow-cli -- search "Local Search" >/dev/null
cargo run -p agentflow-cli -- search "Saved Query" >/dev/null
cargo run -p agentflow-cli -- search "Desktop Search" >/dev/null
cargo run -p agentflow-cli -- search "Saved Query Writer" >/dev/null
cargo run -p agentflow-cli -- search "Local Workspace" >/dev/null
cargo run -p agentflow-cli -- search "Local Project" >/dev/null
cargo run -p agentflow-cli -- search "Local Project Model" >/dev/null
cargo run -p agentflow-cli -- search "Desktop Project View" >/dev/null
cargo run -p agentflow-cli -- search "Workspace Overview" >/dev/null
cargo run -p agentflow-cli -- search "Local Project Seed" >/dev/null
cargo run -p agentflow-cli -- search "Issue Project Link" >/dev/null
cargo run -p agentflow-cli -- search "Project-aware GoalLoop" >/dev/null
cargo run -p agentflow-cli -- search "Desktop GoalLoop Trace" >/dev/null
cargo run -p agentflow-cli -- search "Issue Lifecycle Trace" >/dev/null
cargo run -p agentflow-cli -- search "Project Update Timeline" >/dev/null
cargo run -p agentflow-cli -- search "Desktop MVP Navigation Scope Reduction" >/dev/null
cargo run -p agentflow-cli -- search "Team Hierarchy" >/dev/null
cargo run -p agentflow-cli -- search "Parent Child Columns" >/dev/null
cargo run -p agentflow-cli -- search "Workspace Sidebar Tree" >/dev/null
cargo run -p agentflow-cli -- search "Teams Add Button" >/dev/null
cargo run -p agentflow-cli -- search "Project Audit" >/dev/null
cargo run -p agentflow-cli -- search "Project Code Audit" >/dev/null
cargo run -p agentflow-cli -- search "Root Docs Refresh" >/dev/null
cargo run -p agentflow-cli -- search "Product Feature Creation" >/dev/null
cargo run -p agentflow-cli -- search "Product Feature Execution" >/dev/null
cargo run -p agentflow-cli -- search "Team / Project / Milestone / Issue Writers" >/dev/null
