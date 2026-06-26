# Latest Verification Summary

创建日期：2026-05-21
更新日期：2026-05-26
执行者：Codex

## 当前状态

| 项目 | 状态 |
| --- | --- |
| Project Definition | established |
| Flow 0.1 / 0.2 / 0.3 | established |
| AEP + Linear 参考蓝图 | established |
| Root docs archive | established |
| Docs compression pass | established |
| Goal Execution Spine | established |
| Goal Compiler + Core/CLI Bootstrap v0 | implemented |
| Context Collector + Planner v0 | implemented |
| Codex Runtime Adapter + Validation / Evidence v0 | implemented |
| SQLite Index + Saved Views v0 | implemented |
| Review / ProjectUpdate Generator v0 | implemented |
| Project Update Summary + Review Assistant v0 | implemented |
| Desktop Workbench MVP v0 Boundary | established |
| AEP Goal Initialization Protocol v0 | implemented |
| Goal Loop Orchestrator v0 | implemented |
| Desktop Workbench MVP v0 | implemented |
| Local Pro Experiments v0 Boundary | established |
| Local Metrics Snapshot v0 | implemented |
| Local Search v0 Boundary | established |
| Desktop Workbench 中文界面优化 v0 | implemented |
| Local Search Reader v0 | implemented |
| Saved Query v0 Boundary | established |
| Desktop Search Read-only View v0 Boundary | established |
| Desktop Search Read-only View v0 | implemented |
| Saved Query Writer v0 Boundary | established |
| Local Workspace / Team / Project Model v0 Boundary | established |
| Local Project Model v0 Read-only | implemented |
| Desktop Project View v0 Read-only | implemented |
| Desktop Workspace Overview v0 Read-only | implemented |
| Local Project Seed v0 Boundary | established |
| Local Project Seed v0 | implemented |
| Issue Project Link v0 Boundary | established |
| Issue Project Link Writer v0 | implemented |
| Project-aware GoalLoop v0 | implemented |
| Desktop GoalLoop Trace v0 | implemented |
| Desktop Issue Lifecycle Trace v0 | implemented |
| Desktop Project Update Timeline v0 | implemented |
| Desktop MVP Navigation Scope Reduction v0 | implemented |
| Desktop Team Hierarchy v0 | implemented |
| Desktop Team Parent Child Columns v0 | implemented |
| Desktop Workspace Sidebar Tree v0 | implemented |
| Desktop Teams Add Button v0 | implemented |
| MVP Productization Project v0 | implemented |
| Project Seed Fact Source v0 | implemented |
| Milestone-aware Issue Planning v0 | implemented |
| MVP Execution Loop v0 | implemented |
| MVP Minimal Workflow v0 | implemented |
| Desktop MVP Productization v0 | implemented |
| AgentFlow AI Delivery Workflow Contract v1 | implemented |
| Workflow State Machine v0 | implemented |
| Workflow Control Core v0 | implemented |
| Project Audit / Docs Refresh v0 Boundary | established |
| Project Closure State v0 | implemented |
| Project Code Audit Snapshot v0 | implemented |
| Root Docs Refresh Snapshot v0 | implemented |
| Product Feature Creation Flow v0 | implemented |
| Product Feature Execution Flow v0 | implemented |
| Product Feature Controlled Run v0 | implemented |
| Goal + Criteria Driven MVP | defined |
| Project / Issue Status Model v0 | implemented |
| Team / Project / Milestone / Issue Writers v0 | implemented |
| 技术方案 | GoalCompiler, Rust Core / CLI, Tauri 2, React/TS, SQLite, SavedView, ProjectUpdate, GoalLoop, GoalLoopSelection, ActiveMilestoneQueue, LocalWorkspace, LocalTeam, LocalProject, ProjectStatus, IssueStatus, MilestoneDerivedProgress, LocalProjectIssueRef, TeamDraft, ProjectDraft, MilestoneDraft, IssueDraft, CreationPreview, CreationWriteSummary, ProductFeatureCreation, ProductFeatureExecution, ProductFeatureControlledRun, GoalCriteriaDrivenMvp, ControlledRunPlan, ExecutionRunTrace, WorkflowStateMachine, EligibilityEngine, LeaseLock, ExecutionEvidence, MilestoneReview, ProjectAudit, ProjectCodeAuditSnapshot, ProjectDocsRefreshSnapshot, DesktopWorkbenchSnapshot, LocalProBoundary, LocalMetricsSnapshot, LocalSearchBoundary, LocalSearchReader, SavedQueryBoundary, SavedQueryWriterBoundary, DesktopSearchBoundary, DesktopSearchReadOnlyView, DesktopGoalLoopTrace, DesktopIssueLifecycleTrace, DesktopProjectUpdateTimeline, DesktopMvpNavigation, DesktopTeamHierarchy, DesktopTeamParentChildColumns, DesktopWorkspaceSidebarTree, DesktopTeamsAddButton, DesktopChineseUI |
| 下一步执行授权 | `Desktop read-only creation results polish` 或 `Goal + Criteria MVP product copy` |

## 最近验证

| Command | Result |
| --- | --- |
| `cargo test` after Product Feature Creation Flow v0 | pass, 53 tests |
| `npm --prefix apps/desktop run build` after Product Feature Creation Flow v0 | pass |
| `cargo run -p agentflow-cli -- feature create "示例产品功能"` | pass, preview only / no facts written |
| `cargo run -p agentflow-cli -- feature create "示例产品功能" --write --yes` | pass, wrote new feature project, milestones, issue contracts, workspace/team/index updates |
| `cargo run -p agentflow-cli -- feature status` | pass, active feature project `feature-0043`, current issue `ISSUE-0043` |
| `cargo run -p agentflow-cli -- feature next` | pass, recommends `agentflow run ISSUE-0043 --dry-run` |
| `cargo test` after Product Feature Execution Flow v0 | pass, 55 tests |
| `npm --prefix apps/desktop run build` after Product Feature Execution Flow v0 | pass |
| `bash checks/agentflow-readiness.sh` after Product Feature Execution Flow v0 | pass |
| `cargo fmt --check` after Product Feature Controlled Run v0 | pass |
| `cargo test` after Product Feature Controlled Run v0 | pass, 56 tests |
| `npm --prefix apps/desktop run build` after Product Feature Controlled Run v0 | pass |
| `cargo test -p agentflow-core controlled_run_records_plan_and_updates_feature_status -- --nocapture` | pass |
| `cargo run -p agentflow-cli -- run ISSUE-0043 --dry-run` | pass, RUN-0041 with project / milestone / lease / runPlan |
| `cargo run -p agentflow-cli -- feature status` after controlled run | pass, dry-run recorded true / latest run plan visible |
| `cargo run -p agentflow-cli -- feature next` after controlled run | pass, recommends `agentflow verify ISSUE-0043` |
| `cargo run -p agentflow-cli -- goal next` after controlled run | pass, active issue recommends verify |
| `bash checks/agentflow-readiness.sh` after Product Feature Controlled Run v0 | pass |
| `git diff --check` after Product Feature Controlled Run v0 | pass |
| `docs/specs/goal-criteria-driven-mvp.md` | added, locks current MVP Goal and 24 Criteria |
| `.agentflow/goal.json` / `.agentflow/goal.md` | updated to Goal + Criteria Driven MVP |
| `cargo fmt --check` after Goal + Criteria Driven MVP | pass |
| `cargo test` after Goal + Criteria Driven MVP | pass, 56 tests |
| `npm --prefix apps/desktop run build` after Goal + Criteria Driven MVP | pass |
| `cargo run -p agentflow-cli -- goal check` after Goal + Criteria Driven MVP | pass |
| `cargo run -p agentflow-cli -- goal next` after Goal + Criteria Driven MVP | pass, still reports existing active issue from prior dry-run |
| `cargo run -p agentflow-cli -- feature next` after Goal + Criteria Driven MVP | pass, reports stale lease requires human recovery |
| `bash checks/agentflow-readiness.sh` after Goal + Criteria Driven MVP | pass, no longer starts execution dry-run as MVP readiness |
| `git diff --check` after Goal + Criteria Driven MVP | pass |
| `docs/specs/project-issue-status-model-v0.md` | added, locks canonical Project / Issue status and milestone derived progress |
| `cargo fmt --check` after Project / Issue Status Model v0 | pass |
| `cargo test` after Project / Issue Status Model v0 | pass, 57 tests |
| `npm --prefix apps/desktop run build` after Project / Issue Status Model v0 | pass |
| `cargo run -p agentflow-cli -- projects` after Project / Issue Status Model v0 | pass, shows canonical project / issue status and milestone progress |
| `cargo run -p agentflow-cli -- feature status` after Project / Issue Status Model v0 | pass, shows canonical project / issue status |
| `cargo run -p agentflow-cli -- feature create "状态模型验证功能"` | pass, preview only / issue defaults to `todo` in generated contracts |
| `bash checks/agentflow-readiness.sh` after Project / Issue Status Model v0 | pass |
| `docs/specs/team-project-milestone-issue-writers-v0.md` | added, locks preview-first local writers |
| `cargo fmt --check` after Team / Project / Milestone / Issue Writers v0 | pass |
| `cargo test` after Team / Project / Milestone / Issue Writers v0 | pass, 59 tests |
| `npm --prefix apps/desktop run build` after Team / Project / Milestone / Issue Writers v0 | pass |
| `cargo run -p agentflow-cli -- team create "Demo Team"` | pass, preview only / no facts written |
| `cargo run -p agentflow-cli -- project create "Demo Project"` | pass, preview only / default Project status draft |
| `cargo run -p agentflow-cli -- milestone create "Demo Milestone"` | pass, preview only / updates target Project milestones only after confirmation |
| `cargo run -p agentflow-cli -- issue create "Demo Issue"` | pass, preview only / generated Issue status todo |
| `cargo run -p agentflow-cli -- projects` after Writers v0 | pass, reads canonical status and existing hierarchy |
| `cargo run -p agentflow-cli -- feature status` after Writers v0 | pass, Desktop-facing read model remains read-only |
| `bash checks/agentflow-readiness.sh` after Team / Project / Milestone / Issue Writers v0 | pass, includes preview commands and temp write unit tests |
| `cargo run -p agentflow-cli -- goal next` after Product Feature write | pass, recommended first active project issue |
| `cargo run -p agentflow-cli -- eligibility` after Product Feature write | pass, first feature issue eligible or explicit reasons |
| `cargo run -p agentflow-cli -- metrics` after Product Feature write | pass, 46 issues / 42 completed / 4 planned |
| `cargo run -p agentflow-cli -- search "Product Feature Creation"` | pass, returns feature creation summary |
| `bash checks/agentflow-readiness.sh` after Product Feature Creation Flow v0 | pass |
| `git diff --check` after Product Feature Creation Flow v0 | pass |
| `cargo run -p agentflow-cli -- project-seed --write --yes` | pass, wrote `.agentflow/workspace.json`, `.agentflow/teams/core.json`, `.agentflow/projects/agentflow-local-execution.json` |
| `cargo run -p agentflow-cli -- plan "Project Seed Fact Source v0 实现"` | pass, ISSUE-0037 |
| `cargo run -p agentflow-cli -- issue-link ISSUE-0037 --write --yes` | pass, milestone `mvp-project-foundation` |
| `cargo test -p agentflow-core local_project_model_snapshot_prefers_seed_files_when_present -- --nocapture` | pass |
| `cargo run -p agentflow-cli -- run ISSUE-0037 --dry-run` | pass, RUN-0035 |
| `cargo run -p agentflow-cli -- verify ISSUE-0037` | pass, 2 commands |
| `cargo run -p agentflow-cli -- review ISSUE-0037` | pass, evidence / review / update generated |
| `cargo run -p agentflow-cli -- goal next` after ISSUE-0037 | pass, next action plan `Milestone-aware Issue Planning v0 实现` |
| `cargo run -p agentflow-cli -- plan "Milestone-aware Issue Planning v0 实现"` | pass, ISSUE-0038 |
| `cargo run -p agentflow-cli -- issue-link ISSUE-0038 --write --yes` | pass, milestone `mvp-issue-planning` |
| `cargo run -p agentflow-cli -- projects` | pass, active milestone `mvp-issue-planning`, current issue `ISSUE-0038` |
| `cargo run -p agentflow-cli -- goal next` | pass, next action run `ISSUE-0038` |
| `cargo test -p agentflow-core plan_issue_links_active_project_milestone_when_seed_exists -- --nocapture` | pass |
| `cargo test -p agentflow-core issue_project_link_write_updates_only_target_issue -- --nocapture` | pass |
| `cargo run -p agentflow-cli -- run ISSUE-0038 --dry-run` | pass, RUN-0036 |
| `cargo run -p agentflow-cli -- verify ISSUE-0038` | pass, 5 commands |
| `cargo run -p agentflow-cli -- review ISSUE-0038` | pass, evidence / review / update generated |
| `cargo run -p agentflow-cli -- projects` after ISSUE-0038 | pass, active milestone `mvp-execution-loop`, next intent `MVP Execution Loop v0 收敛` |
| `cargo run -p agentflow-cli -- plan "MVP Execution Loop v0 收敛"` | pass, ISSUE-0039 auto-linked to `mvp-execution-loop` |
| `cargo run -p agentflow-cli -- goal next` after ISSUE-0039 | pass, next action run `ISSUE-0039` |
| `cargo run -p agentflow-cli -- update summary` | pass, 39 issues / 38 completed / 36 runs / 36 updates |
| `cargo fmt --check` | pass |
| `cargo test -p agentflow-core local_project_model_snapshot_reads_current_facts_without_writing -- --nocapture` | pass |
| `cargo run -p agentflow-cli -- run ISSUE-0039 --dry-run` | pass, RUN-0037 |
| `cargo run -p agentflow-cli -- verify ISSUE-0039` | pass, 6 commands |
| `cargo run -p agentflow-cli -- review ISSUE-0039` | pass, evidence / review / update generated |
| `cargo run -p agentflow-cli -- projects` after ISSUE-0039 | pass, milestone execution trace shows evidence/update paths |
| `cargo run -p agentflow-cli -- plan "Desktop MVP Productization v0 收敛"` | pass, ISSUE-0040 auto-linked to `mvp-desktop-polish` |
| `cargo run -p agentflow-cli -- goal next` after ISSUE-0040 | pass, next action run `ISSUE-0040` |
| `cargo run -p agentflow-cli -- update summary` after ISSUE-0040 | pass, 40 issues / 39 completed / 37 runs / 37 updates |
| `cargo test goal_next_uses_active_milestone_queue_before_outside_backlog` | pass, active milestone queue preflight wins over outside backlog |
| `cargo test review_completes_milestone_and_writes_summary_when_seed_linked` | pass, milestone completion writes `MILESTONE-*-evidence-summary.md` and activates next milestone |
| `cargo run -p agentflow-cli -- verify ISSUE-0040` | pass, RUN-0038 / 6 commands |
| `cargo run -p agentflow-cli -- review ISSUE-0040` | pass, evidence / review / project update generated |
| `test -f .agentflow/evidence/MILESTONE-mvp-desktop-polish-evidence-summary.md` | pass, milestone summary generated |
| `cargo run -p agentflow-cli -- goal next` after ISSUE-0040 review | pass, next action plan `MVP Release Readiness v0 验收` |
| `cargo run -p agentflow-cli -- plan "AgentFlow AI Delivery Workflow Contract v1 边界定义"` | pass, ISSUE-0041 auto-linked to `mvp-release-readiness` |
| `cargo run -p agentflow-cli -- run ISSUE-0041 --dry-run` | pass, RUN-0039 |
| `test -f docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md` | pass, canonical PRD contract exists |
| contract completeness anchors | pass, canonical contract now includes command boundary, local file mapping, event log contract, first executable vertical slice, verification matrix |
| `cargo run -p agentflow-cli -- verify ISSUE-0041` | pass, RUN-0039 / 8 commands |
| `cargo run -p agentflow-cli -- review ISSUE-0041` | pass, evidence / review / project update generated |
| `test -f .agentflow/evidence/MILESTONE-mvp-release-readiness-evidence-summary.md` | pass, milestone summary generated |
| `cargo run -p agentflow-cli -- goal next` after ISSUE-0041 review | pass, next action plan `Workflow State Machine v0 边界定义` |
| `cargo run -p agentflow-cli -- plan "Workflow State Machine v0 边界定义"` | pass, ISSUE-0042 auto-linked to `workflow-core-state-machine` |
| `cargo test workflow_state` | pass, workflow state snapshot tests |
| `cargo run -p agentflow-cli -- state check` | pass, ready true, wrote `.agentflow/state/workflow-state.json` and `.agentflow/updates/WORKFLOW-STATE-SUMMARY.md` |
| `cargo run -p agentflow-cli -- run ISSUE-0042 --dry-run` | pass, RUN-0040 |
| `cargo run -p agentflow-cli -- verify ISSUE-0042` | pass, RUN-0040 / 2 commands |
| `cargo run -p agentflow-cli -- review ISSUE-0042` | pass, evidence / review / project update generated |
| `test -f .agentflow/evidence/MILESTONE-workflow-core-state-machine-evidence-summary.md` | pass, milestone summary generated |
| `cargo run -p agentflow-cli -- goal next` after ISSUE-0042 review | pass, next action plan `Eligibility Engine v0 边界定义` |
| `cargo fmt --check` final | pass |
| `cargo test` | pass, 41 tests |
| `npm --prefix apps/desktop run build` final | pass |
| `cargo run -p agentflow-cli -- update summary` | pass, 42 issues / 42 completed / 40 runs / 40 updates |
| `cargo run -p agentflow-cli -- state check` final | pass, ready true, 0 errors |
| `cargo run -p agentflow-cli -- goal check` final | pass |
| `cargo run -p agentflow-cli -- goal next` final | pass, next action plan `Eligibility Engine v0 边界定义` |
| `cargo run -p agentflow-cli -- projects` final | pass, active milestone `workflow-core-eligibility-engine` |
| `cargo run -p agentflow-cli -- metrics` final | pass, latest evidence `MILESTONE-workflow-core-state-machine-evidence-summary.md` |
| `cargo run -p agentflow-cli -- search "Workflow State"` | pass, returns ISSUE-0042 / workflow state summary / milestone evidence |
| `cargo run -p agentflow-cli -- search "Eligibility Engine"` | pass, returns next intent and contract traces |
| `cargo fmt --check` after Workflow Control Core v0 | pass |
| `cargo test` after Workflow Control Core v0 | pass, 44 tests |
| `npm --prefix apps/desktop run build` after Workflow Control Core v0 | pass |
| `cargo run -p agentflow-cli -- state check` after Workflow Control Core v0 | pass, ready true / 1 project / 9 milestones / 42 issues / 0 errors / 36 warnings |
| `cargo run -p agentflow-cli -- eligibility` after Workflow Control Core v0 | pass, active milestone has no open issue, recommends `Project Audit / Docs Refresh v0 边界定义` |
| `cargo run -p agentflow-cli -- lease` after Workflow Control Core v0 | pass, 0 active leases / 0 stale leases |
| `cargo run -p agentflow-cli -- goal check` after Workflow Control Core v0 | pass, ready true |
| `cargo run -p agentflow-cli -- goal next` after Workflow Control Core v0 | pass, next action plan `Project Audit / Docs Refresh v0 边界定义` |
| `cargo run -p agentflow-cli -- projects` after Workflow Control Core v0 | pass, active milestone `workflow-core-closure-gates` |
| `cargo run -p agentflow-cli -- metrics` after Workflow Control Core v0 | pass, 42 issues / 40 runs / next action plan |
| `cargo run -p agentflow-cli -- search "Workflow Control Core"` | pass, traceable roadmap result |
| `cargo run -p agentflow-cli -- search "Project Audit"` | pass, traceable next-stage results |
| `bash checks/agentflow-readiness.sh` after Workflow Control Core v0 | pass |
| `git diff --check` after Workflow Control Core v0 | pass |
| `test -f docs/specs/project-audit-docs-refresh-boundary.md` | pass |
| `rg "Project Audit / Docs Refresh v0" docs/specs/project-audit-docs-refresh-boundary.md README.md ROADMAP.md docs/specs/mvp-spec.md docs/specs/workflow-control-core-v0.md docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md` | pass |
| `cargo run -p agentflow-cli -- goal next` after Project Audit / Docs Refresh boundary | pass, next action plan `Project Closure State v0 实现` |
| `cargo fmt --check` after Project Audit / Docs Refresh boundary | pass |
| `cargo test` after Project Audit / Docs Refresh boundary | pass, 44 tests |
| `npm --prefix apps/desktop run build` after Project Audit / Docs Refresh boundary | pass |
| `cargo run -p agentflow-cli -- state check` after Project Audit / Docs Refresh boundary | pass, ready true / 1 project / 9 milestones / 42 issues / 0 errors / 36 warnings |
| `cargo run -p agentflow-cli -- eligibility` after Project Audit / Docs Refresh boundary | pass, recommends `Project Closure State v0 实现` |
| `cargo run -p agentflow-cli -- lease` after Project Audit / Docs Refresh boundary | pass, 0 active leases / 0 stale leases |
| `cargo run -p agentflow-cli -- goal check` after Project Audit / Docs Refresh boundary | pass, ready true |
| `cargo run -p agentflow-cli -- projects` after Project Audit / Docs Refresh boundary | pass, active milestone `workflow-core-closure-gates` |
| `cargo run -p agentflow-cli -- metrics` after Project Audit / Docs Refresh boundary | pass, next action plan `Project Closure State v0 实现` |
| `cargo run -p agentflow-cli -- search "Project Audit"` after Project Audit / Docs Refresh boundary | pass, traceable `.agentflow/` results |
| `bash checks/agentflow-readiness.sh` after Project Audit / Docs Refresh boundary | pass |
| `test ! -d .agentflow/audits` after Project Audit / Docs Refresh boundary | pass |
| `git diff --check` after Project Audit / Docs Refresh boundary | pass |
| `cargo test project_closure -- --nocapture` | pass, 2 focused tests |
| `cargo run -p agentflow-cli -- project closure` | pass, closure state `audit-ready`, `can_mark_done=false`, wrote project closure state and summary |
| `cargo run -p agentflow-cli -- goal next` after Project Closure State v0 | pass, next action `project-closure`, recommended command `agentflow project closure` |
| `cargo fmt --check` after Project Closure State v0 | pass |
| `cargo test` after Project Closure State v0 | pass, 46 tests |
| `npm --prefix apps/desktop run build` after Project Closure State v0 | pass |
| `cargo run -p agentflow-cli -- state check` after Project Closure State v0 | pass, ready true / 1 project / 9 milestones / 42 issues / 0 errors / 36 warnings |
| `cargo run -p agentflow-cli -- eligibility` after Project Closure State v0 | pass, recommends `Project Code Audit Snapshot v0 只读实现` |
| `cargo run -p agentflow-cli -- lease` after Project Closure State v0 | pass, 0 active leases / 0 stale leases |
| `cargo run -p agentflow-cli -- goal check` after Project Closure State v0 | pass, ready true |
| `cargo run -p agentflow-cli -- projects` after Project Closure State v0 | pass, recommended command `agentflow project closure` |
| `cargo run -p agentflow-cli -- metrics` after Project Closure State v0 | pass, next action `project-closure` |
| `cargo run -p agentflow-cli -- search "Project Closure"` after Project Closure State v0 | pass, traceable `.agentflow/` results |
| `bash checks/agentflow-readiness.sh` after Project Closure State v0 | pass |
| `test ! -d .agentflow/audits` after Project Closure State v0 | pass |
| `git diff --check` after Project Closure State v0 | pass |
| `cargo test project_code_audit -- --nocapture` | pass, 2 focused tests |
| `cargo fmt --check` after Project Code Audit Snapshot v0 | pass |
| `cargo test` after Project Code Audit Snapshot v0 | pass, 47 tests |
| `npm --prefix apps/desktop run build` after Project Code Audit Snapshot v0 | pass |
| `cargo run -p agentflow-cli -- state check` after Project Code Audit Snapshot v0 | pass, ready true / 1 project / 9 milestones / 42 issues / 0 errors / 36 warnings |
| `cargo run -p agentflow-cli -- eligibility` after Project Code Audit Snapshot v0 | pass, recommends `Root Docs Refresh Snapshot v0 只读实现` |
| `cargo run -p agentflow-cli -- lease` after Project Code Audit Snapshot v0 | pass, 0 active leases / 0 stale leases |
| `cargo run -p agentflow-cli -- project closure` after Project Code Audit Snapshot v0 | pass, closure state `audit`, code audit gate `snapshot-ready`, `can_mark_done=false` |
| `cargo run -p agentflow-cli -- project code-audit` | pass, wrote `.agentflow/state/project-code-audit.json` and `.agentflow/updates/PROJECT-CODE-AUDIT-SUMMARY.md` |
| `cargo run -p agentflow-cli -- goal check` after Project Code Audit Snapshot v0 | pass, ready true |
| `cargo run -p agentflow-cli -- goal next` after Project Code Audit Snapshot v0 | pass, next action `project-closure`, recommended command `agentflow project closure` |
| `cargo run -p agentflow-cli -- projects` after Project Code Audit Snapshot v0 | pass, active milestone `workflow-core-closure-gates` |
| `cargo run -p agentflow-cli -- metrics` after Project Code Audit Snapshot v0 | pass, next action `project-closure` |
| `cargo run -p agentflow-cli -- search "Project Code Audit"` | pass, returns snapshot and closure summary traces |
| `bash checks/agentflow-readiness.sh` after Project Code Audit Snapshot v0 | pass |
| `test ! -d .agentflow/audits` after Project Code Audit Snapshot v0 | pass |
| `cargo test` after Root Docs Refresh Snapshot v0 | pass, 50 tests |
| `npm --prefix apps/desktop run build` after Root Docs Refresh Snapshot v0 | pass |
| `cargo run -p agentflow-cli -- project docs-refresh` | pass, wrote `.agentflow/state/project-docs-refresh.json` and `.agentflow/updates/PROJECT-DOCS-REFRESH-SUMMARY.md`, checked docs 14, update-needed 0, missing 0 |
| `cargo run -p agentflow-cli -- project closure` after Root Docs Refresh Snapshot v0 | pass, closure state `audit`, code audit / docs refresh gates `snapshot-ready`, `can_mark_done=false` |
| `cargo run -p agentflow-cli -- goal next` after Root Docs Refresh Snapshot v0 | pass, next action `plan`, recommended command `agentflow plan "Product Feature Creation Flow v0"` |
| `cargo run -p agentflow-cli -- eligibility` after Root Docs Refresh Snapshot v0 | pass, recommends `Product Feature Creation Flow v0` |
| `cargo run -p agentflow-cli -- metrics` after Root Docs Refresh Snapshot v0 | pass, next action `plan`, recommended command `agentflow plan "Product Feature Creation Flow v0"` |
| `cargo run -p agentflow-cli -- search "Root Docs Refresh"` | pass, returns docs refresh and closure summary traces |
| `bash checks/agentflow-readiness.sh` after Root Docs Refresh Snapshot v0 | pass |
| `test ! -d .agentflow/audits` after Root Docs Refresh Snapshot v0 | pass |
| `git diff --check` after Project Code Audit Snapshot v0 | pass |
| `npm --prefix apps/desktop run build` | pass |
| `cargo run -p agentflow-cli -- goal check` | pass |
| `cargo run -p agentflow-cli -- projects` | pass, active milestone `mvp-execution-loop`, issue `ISSUE-0039` |
| `bash checks/agentflow-readiness.sh` | pass, seed-backed MVP checks |
| `git diff --check` | pass |
| `cargo run -p agentflow-cli -- plan "Desktop Teams Add Button v0"` | pass, ISSUE-0036 |
| `cargo run -p agentflow-cli -- run ISSUE-0036 --dry-run` | pass, RUN-0034 |
| `npm --prefix apps/desktop run build` | pass, Teams add button UI |
| browser smoke | pass, TEAMS has one `+` button and clicking it opens 新增团队 / 初始化创建入口 |
| `cargo fmt --check` | pass |
| `cargo test` | pass, 35 tests |
| `cargo run -p agentflow-cli -- goal check` | pass, ready true |
| `cargo run -p agentflow-cli -- goal next` active issue | pass, next action verify ISSUE-0036 |
| `cargo run -p agentflow-cli -- search "Teams Add Button"` | pass, traceable `.agentflow/` results |
| `bash checks/agentflow-readiness.sh` | pass, includes Desktop Teams Add Button anchors |
| no live workspace/team/project seed proof | pass, no `.agentflow/workspace.json`、`.agentflow/teams/`、`.agentflow/projects/` |
| no issue projectLink proof | pass, 36 issue JSON files without top-level projectLink/teamId/projectId/milestoneId/linkSource |
| `cargo run -p agentflow-cli -- verify ISSUE-0036` | pass, 8 commands |
| `cargo run -p agentflow-cli -- review ISSUE-0036` | pass, evidence / review / update generated |
| `cargo run -p agentflow-cli -- update summary` | pass, 36 issues / 34 runs / 34 updates / 1 saved view |
| `cargo run -p agentflow-cli -- review-assistant ISSUE-0036` | pass, 15 checks / ready |
| `cargo run -p agentflow-cli -- goal next` after review | pass, next action plan `Desktop MVP Task Detail v0 收敛` |
| `cargo run -p agentflow-cli -- plan "Desktop Workspace Sidebar Tree v0"` | pass, ISSUE-0035 |
| `cargo run -p agentflow-cli -- run ISSUE-0035 --dry-run` | pass, RUN-0033 |
| `npm --prefix apps/desktop run build` | pass, workspace sidebar tree UI |
| browser smoke | pass, sidebar shows Workspace / Teams parent nodes and project / issues children; no trace nav |
| `cargo fmt --check` | pass |
| `cargo test` | pass, 35 tests |
| `cargo run -p agentflow-cli -- goal check` | pass, ready true |
| `cargo run -p agentflow-cli -- goal next` active issue | pass, next action verify ISSUE-0035 |
| `cargo run -p agentflow-cli -- projects` | pass, LocalProjectModelSnapshot read-only；recommended command 指向 `agentflow verify ISSUE-0035` |
| `cargo run -p agentflow-cli -- search "Workspace Sidebar Tree"` | pass, traceable `.agentflow/` results |
| `bash checks/agentflow-readiness.sh` | pass, includes Desktop Workspace Sidebar Tree anchors |
| no live workspace/team/project seed proof | pass, no `.agentflow/workspace.json`、`.agentflow/teams/`、`.agentflow/projects/` |
| no issue projectLink proof | pass, 35 issue JSON files without top-level projectLink/teamId/projectId/milestoneId/linkSource |
| `cargo run -p agentflow-cli -- verify ISSUE-0035` | pass, 9 commands |
| `cargo run -p agentflow-cli -- review ISSUE-0035` | pass, evidence / review / update generated |
| `cargo run -p agentflow-cli -- update summary` | pass, 35 issues / 33 runs / 33 updates / 1 saved view |
| `cargo run -p agentflow-cli -- review-assistant ISSUE-0035` | pass, 15 checks / ready |
| `cargo run -p agentflow-cli -- goal next` after review | pass, next action plan `Desktop MVP Task Detail v0 收敛` |
| `cargo run -p agentflow-cli -- plan "Desktop Team Parent Child Columns v0"` | pass, ISSUE-0034 |
| `cargo run -p agentflow-cli -- run ISSUE-0034 --dry-run` | pass, RUN-0032 |
| `npm --prefix apps/desktop run build` | pass, parent-child column UI |
| browser smoke | pass, team page shows 3 columns with 父级栏目 / 子级栏目 / 团队 / 项目 / 任务 and no trace nav |
| `cargo fmt --check` | pass |
| `cargo test` | pass, 35 tests |
| `cargo run -p agentflow-cli -- goal check` | pass, ready true |
| `cargo run -p agentflow-cli -- goal next` active issue | pass, next action verify ISSUE-0034 |
| `cargo run -p agentflow-cli -- projects` | pass, LocalProjectModelSnapshot read-only；recommended command 指向 `agentflow verify ISSUE-0034` |
| `cargo run -p agentflow-cli -- search "Parent Child Columns"` | pass, traceable `.agentflow/` results |
| `bash checks/agentflow-readiness.sh` | pass, includes Desktop Team Parent Child Columns anchors |
| no live workspace/team/project seed proof | pass, no `.agentflow/workspace.json`、`.agentflow/teams/`、`.agentflow/projects/` |
| no issue projectLink proof | pass, 34 issue JSON files without top-level projectLink/teamId/projectId/milestoneId/linkSource |
| `cargo run -p agentflow-cli -- verify ISSUE-0034` | pass, 9 commands |
| `cargo run -p agentflow-cli -- review ISSUE-0034` | pass, evidence / review / update generated |
| `cargo run -p agentflow-cli -- update summary` | pass, 34 issues / 32 runs / 32 updates / 1 saved view |
| `cargo run -p agentflow-cli -- review-assistant ISSUE-0034` | pass, 15 checks / ready |
| `cargo run -p agentflow-cli -- goal next` after review | pass, next action plan `Desktop MVP Task Detail v0 收敛` |
| `cargo run -p agentflow-cli -- plan "Desktop Team Hierarchy v0 收敛"` | pass, ISSUE-0033 |
| `cargo run -p agentflow-cli -- run ISSUE-0033 --dry-run` | pass, RUN-0031 |
| `npm --prefix apps/desktop run build` | pass, Desktop Team Hierarchy UI |
| browser smoke | pass, 团队页显示 Your teams、团队入口、项目、任务和多团队说明；主导航未恢复决策/生命周期/更新时间线 |
| `cargo fmt --check` | pass |
| `cargo test` | pass, 35 tests |
| `cargo run -p agentflow-cli -- goal check` | pass, ready true |
| `cargo run -p agentflow-cli -- goal next` active issue | pass, next action verify ISSUE-0033 |
| `cargo run -p agentflow-cli -- projects` | pass, LocalProjectModelSnapshot read-only；recommended command 指向 `agentflow verify ISSUE-0033` |
| `cargo run -p agentflow-cli -- search "Team Hierarchy"` | pass, traceable `.agentflow/` results |
| `bash checks/agentflow-readiness.sh` | pass, includes Desktop Team Hierarchy anchors |
| no live workspace/team/project seed proof | pass, no `.agentflow/workspace.json`、`.agentflow/teams/`、`.agentflow/projects/` |
| no issue projectLink proof | pass, 33 issue JSON files without top-level projectLink/teamId/projectId/milestoneId/linkSource |
| `cargo run -p agentflow-cli -- verify ISSUE-0033` | pass, 9 commands |
| `cargo run -p agentflow-cli -- review ISSUE-0033` | pass, evidence / review / update generated |
| `cargo run -p agentflow-cli -- update summary` | pass, 33 issues / 31 runs / 31 updates / 1 saved view |
| `cargo run -p agentflow-cli -- review-assistant ISSUE-0033` | pass after sequential retry, 15 checks / ready |
| `cargo run -p agentflow-cli -- goal next` after review | pass, next action plan `Desktop MVP Task Detail v0 收敛` |
| `git diff --check` | pass |
| Linear / SavedView / ProjectUpdate anchor `rg` | pass |
| stale MVP range `rg` | pass |
| trailing whitespace `rg` | pass |
| root file check | pass |
| docs line count | pass |
| Goal spine anchor `rg` | pass |
| `cargo fmt --check` | pass |
| `cargo test` | pass, 19 tests |
| `agentflow init --from-goal GOAL.md` | pass |
| `agentflow goal check` | pass |
| `agentflow plan "实现 Goal Compiler + Core/CLI Bootstrap v0"` | pass |
| `agentflow context` | pass, 69 files |
| `agentflow plan "实现 Codex Runtime Adapter + Validation / Evidence v0"` | pass, ISSUE-0003 |
| `agentflow run ISSUE-0003 --dry-run` | pass, RUN-0001 |
| `agentflow verify ISSUE-0003` | pass, 2 commands |
| `agentflow review ISSUE-0003` | pass, evidence / review / update generated |
| `agentflow plan "实现 SQLite Index + Saved Views v0"` | pass, ISSUE-0004 |
| `agentflow index rebuild` | pass, 4 issues / 2 runs / 2 updates / 1 saved view |
| `agentflow view save completed --issue-status completed --run-status completed --validation-status passed` | pass |
| `agentflow view show completed` | pass, 4 issues / 2 runs |
| `agentflow run ISSUE-0004 --dry-run` | pass, RUN-0002 |
| `agentflow verify ISSUE-0004` | pass, 2 commands |
| `agentflow review ISSUE-0004` | pass, evidence / review / update generated |
| `agentflow plan "实现 Project Update Summary + Review Assistant v0"` | pass, ISSUE-0005 |
| `agentflow run ISSUE-0005 --dry-run` | pass, RUN-0003 |
| `agentflow verify ISSUE-0005` | pass, 2 commands |
| `agentflow review ISSUE-0005` | pass, evidence / review / update generated |
| `agentflow review-assistant ISSUE-0005` | pass, 9 checks / ready |
| `agentflow update summary` | pass, 5 issues / 3 runs / 3 updates / 1 saved view |
| `agentflow view show completed` | pass, 5 issues / 3 runs |
| `agentflow plan "定义 Desktop Workbench MVP v0 边界"` | pass, ISSUE-0006 |
| `agentflow run ISSUE-0006 --dry-run` | pass, RUN-0004 |
| `agentflow verify ISSUE-0006` | pass, 2 commands |
| `agentflow review ISSUE-0006` | pass, evidence / review / update generated |
| `agentflow review-assistant ISSUE-0006` | pass, 9 checks / ready |
| `agentflow update summary` | pass, 6 issues / 4 runs / 4 updates / 1 saved view |
| `agentflow view show completed` | pass, 6 issues / 4 runs |
| `agentflow plan "实现 AEP Goal Initialization Protocol v0"` | pass, ISSUE-0007 |
| `agentflow goal bootstrap` | pass, wrote AEP initialization artifacts |
| `agentflow goal check` | pass, ready true |
| `agentflow run ISSUE-0007 --dry-run` | pass, RUN-0005 |
| `agentflow verify ISSUE-0007` | pass, 2 commands |
| `agentflow review ISSUE-0007` | pass, evidence / review / update generated |
| `agentflow review-assistant ISSUE-0007` | pass, 14 checks / ready |
| `agentflow update summary` | pass, 7 issues / 5 runs / 5 updates / 1 saved view |
| `agentflow plan "实现 Goal Loop Orchestrator v0"` | pass, ISSUE-0008 |
| `agentflow goal next` before run | pass, next action run ISSUE-0008 |
| `agentflow run ISSUE-0008 --dry-run` | pass, RUN-0006 |
| `agentflow goal next` with active issue | pass, next action verify ISSUE-0008 |
| `agentflow verify ISSUE-0008` | pass, 2 commands |
| `agentflow review ISSUE-0008` | pass, evidence / review / update generated |
| `agentflow goal next` after review | pass, next action plan Desktop Workbench MVP v0 只读壳实现 |
| `agentflow review-assistant ISSUE-0008` | pass, 15 checks / ready |
| `agentflow update summary` | pass, 8 issues / 6 runs / 6 updates / 1 saved view |
| `bash checks/agentflow-readiness.sh` | pass |
| `agentflow plan "Desktop Workbench MVP v0 只读壳实现"` | pass, ISSUE-0009 |
| `agentflow goal next` before run | pass, next action run ISSUE-0009 |
| `agentflow run ISSUE-0009 --dry-run` | pass, RUN-0007 |
| `agentflow goal next` with active issue | pass, next action verify ISSUE-0009 |
| `agentflow verify ISSUE-0009` | pass, 7 commands |
| `agentflow review ISSUE-0009` | pass, evidence / review / update generated |
| `agentflow review-assistant ISSUE-0009` | pass, 15 checks / ready |
| `agentflow update summary` | pass, 9 issues / 7 runs / 7 updates / 1 saved view |
| `agentflow goal next` after review | pass, next action plan Local Pro Experiments v0 边界定义 |
| `npm --prefix apps/desktop run build` | pass |
| `npm --prefix apps/desktop run tauri -- dev` | pass, desktop process launched and was stopped manually |
| `bash checks/agentflow-readiness.sh` | pass, includes Desktop Workbench anchors |
| `agentflow plan "Local Pro Experiments v0 边界定义"` | pass, ISSUE-0010 |
| `agentflow goal next` before run | pass, next action run ISSUE-0010 |
| `agentflow run ISSUE-0010 --dry-run` | pass, RUN-0008 |
| `agentflow goal next` with active issue | pass, next action verify ISSUE-0010 |
| `agentflow verify ISSUE-0010` | pass, 7 commands |
| `agentflow review ISSUE-0010` | pass, evidence / review / update generated |
| `agentflow review-assistant ISSUE-0010` | pass, 15 checks / ready |
| `agentflow update summary` | pass, 10 issues / 8 runs / 8 updates / 1 saved view |
| `agentflow goal next` after review | pass, next action plan Local Metrics Snapshot v0 只读实现 |
| `bash checks/agentflow-readiness.sh` | pass, includes Local Pro boundary anchors |
| `agentflow plan "Local Metrics Snapshot v0 只读实现"` | pass, ISSUE-0011 |
| `agentflow goal next` before run | pass, next action run ISSUE-0011 |
| `agentflow run ISSUE-0011 --dry-run` | pass, RUN-0009 |
| `agentflow goal next` with active issue | pass, next action verify ISSUE-0011 |
| `agentflow verify ISSUE-0011` | pass, 8 commands |
| `agentflow review ISSUE-0011` | pass, evidence / review / update generated |
| `agentflow review-assistant ISSUE-0011` | pass, 15 checks / ready |
| `agentflow update summary` | pass, 11 issues / 9 runs / 9 updates / 1 saved view |
| `agentflow metrics` | pass, 11 issues / 9 runs / read-only true |
| `agentflow goal next` after review | pass, next action plan Local Search v0 边界定义 |
| `bash checks/agentflow-readiness.sh` | pass, includes Local Metrics anchors and metrics command |
| `agentflow plan "Local Search v0 边界定义"` | pass, ISSUE-0012 |
| `agentflow goal next` before run | pass, next action run ISSUE-0012 |
| `agentflow run ISSUE-0012 --dry-run` | pass, RUN-0010 |
| `agentflow goal next` with active issue | pass, next action verify ISSUE-0012 |
| `agentflow verify ISSUE-0012` | pass, 8 commands |
| `agentflow review ISSUE-0012` | pass, evidence / review / update generated |
| `agentflow review-assistant ISSUE-0012` | pass, 15 checks / ready |
| `agentflow update summary` | pass, 12 issues / 10 runs / 10 updates / 1 saved view |
| `agentflow goal next` after review | pass, next action plan Local Search Reader v0 只读实现 |
| `agentflow metrics` | pass, 12 issues / 10 runs / read-only true |
| `bash checks/agentflow-readiness.sh` | pass, includes Local Search anchors and no `.agentflow/search` / `.agentflow/queries` directories |
| `agentflow plan "Desktop Workbench 中文界面优化 v0"` | pass, ISSUE-0013 |
| `npm --prefix apps/desktop run build` | pass, Chinese UI build |
| `agentflow goal next` before run | pass, next action run ISSUE-0013 |
| `agentflow run ISSUE-0013 --dry-run` | pass, RUN-0011 |
| `agentflow goal next` with active issue | pass, next action verify ISSUE-0013 |
| `agentflow verify ISSUE-0013` | pass, 8 commands |
| `agentflow review ISSUE-0013` | pass, evidence / review / update generated |
| `agentflow review-assistant ISSUE-0013` | pass, 15 checks / ready |
| `agentflow update summary` | pass, 13 issues / 11 runs / 11 updates / 1 saved view |
| `agentflow goal next` after review | pass, next action plan Local Search Reader v0 只读实现 |
| `agentflow metrics` | pass, 13 issues / 11 runs / read-only true |
| `agentflow plan "Local Search Reader v0 只读实现"` | pass, ISSUE-0014 |
| `cargo test` | pass, 21 tests |
| `agentflow goal next` before run | pass, next action run ISSUE-0014 |
| `agentflow run ISSUE-0014 --dry-run` | pass, RUN-0012 |
| `agentflow goal next` with active issue | pass, next action verify ISSUE-0014 |
| `agentflow verify ISSUE-0014` | pass, 9 commands |
| `agentflow search "Local Search"` | pass, traceable `.agentflow/` results |
| `agentflow review ISSUE-0014` | pass, evidence / review / update generated |
| `agentflow review-assistant ISSUE-0014` | pass, 15 checks / ready |
| `agentflow update summary` | pass, 14 issues / 12 runs / 12 updates / 1 saved view |
| `agentflow goal next` after review | pass, next action plan Saved Query v0 边界定义 |
| `agentflow metrics` | pass, 14 issues / 12 runs / read-only true |
| `bash checks/agentflow-readiness.sh` | pass, includes Local Search Reader and search command |
| `agentflow plan "Saved Query v0 边界定义"` | pass, ISSUE-0015 |
| `agentflow goal next` before run | pass, next action run ISSUE-0015 |
| `agentflow run ISSUE-0015 --dry-run` | pass, RUN-0013 |
| `agentflow goal next` with active issue | pass, next action verify ISSUE-0015 |
| `agentflow verify ISSUE-0015` | pass, 10 commands |
| `agentflow search "Saved Query"` | pass, traceable `.agentflow/` results |
| `agentflow review ISSUE-0015` | pass, evidence / review / update generated |
| `agentflow update summary` | pass, 15 issues / 13 runs / 13 updates / 1 saved view |
| `agentflow goal next` after review | pass, next action plan Desktop Search Read-only View v0 边界定义 |
| `agentflow review-assistant ISSUE-0015` | pass, 15 checks / ready |
| `agentflow metrics` | pass, 15 issues / 13 runs / read-only true |
| `test ! -d .agentflow/queries` | pass |
| `agentflow plan "Desktop Search Read-only View v0 边界定义"` | pass, ISSUE-0016 |
| `agentflow goal next` before run | pass, next action run ISSUE-0016 |
| `agentflow run ISSUE-0016 --dry-run` | pass, RUN-0014 |
| `agentflow goal next` with active issue | pass, next action verify ISSUE-0016 |
| `agentflow verify ISSUE-0016` | pass, 10 commands |
| `agentflow search "Desktop Search"` | pass, traceable `.agentflow/` results |
| `agentflow review ISSUE-0016` | pass, evidence / review / update generated |
| `agentflow update summary` | pass, 16 issues / 14 runs / 14 updates / 1 saved view |
| `agentflow goal next` after review | pass, next action plan Desktop Search Read-only View v0 实现 |
| `agentflow review-assistant ISSUE-0016` | pass, 15 checks / ready |
| `agentflow metrics` | pass, 16 issues / 14 runs / read-only true |
| `test ! -d .agentflow/search` | pass |
| `agentflow plan "Desktop Search Read-only View v0 实现"` | pass, ISSUE-0017 |
| `agentflow goal next` before run | pass, next action run ISSUE-0017 |
| `agentflow run ISSUE-0017 --dry-run` | pass, RUN-0015 |
| `agentflow goal next` with active issue | pass, next action verify ISSUE-0017 |
| `agentflow verify ISSUE-0017` | pass, 11 commands |
| `agentflow search "Desktop Search"` | pass, traceable `.agentflow/` results |
| `agentflow review ISSUE-0017` | pass, evidence / review / update generated |
| `agentflow update summary` | pass, 17 issues / 15 runs / 15 updates / 1 saved view |
| `agentflow goal next` after review | pass, next action plan Saved Query Writer v0 边界定义 |
| `agentflow review-assistant ISSUE-0017` | pass, 15 checks / ready |
| `agentflow metrics` | pass, 17 issues / 15 runs / read-only true |
| `agentflow plan "Saved Query Writer v0 边界定义"` | pass, ISSUE-0018 |
| `agentflow goal next` before run | pass, next action run ISSUE-0018 |
| `agentflow run ISSUE-0018 --dry-run` | pass, RUN-0016 |
| `agentflow goal next` with active issue | pass, next action verify ISSUE-0018 |
| `agentflow verify ISSUE-0018` | pass, 11 commands |
| `agentflow search "Saved Query Writer"` | pass, traceable `.agentflow/` results |
| `agentflow review ISSUE-0018` | pass, evidence / review / update generated |
| `agentflow update summary` | pass, 18 issues / 16 runs / 16 updates / 1 saved view |
| `agentflow goal next` after review | pass, next action plan Saved Query Writer v0 实现 |
| `agentflow review-assistant ISSUE-0018` | pass, 15 checks / ready |
| `agentflow metrics` | pass, 18 issues / 16 runs / read-only true |
| `agentflow plan "Local Workspace / Team / Project Model v0 边界定义"` | pass, ISSUE-0019 |
| `agentflow goal next` before run | pass, next action run ISSUE-0019 |
| `agentflow run ISSUE-0019 --dry-run` | pass, RUN-0017 |
| `agentflow goal next` with active issue | pass, next action verify ISSUE-0019 |
| `agentflow verify ISSUE-0019` | pass, 13 commands |
| `agentflow search "Local Workspace"` | pass, traceable `.agentflow/` results |
| `agentflow search "Local Project"` | pass, traceable `.agentflow/` results |
| `agentflow review ISSUE-0019` | pass, evidence / review / update generated |
| `agentflow update summary` | pass, 19 issues / 17 runs / 17 updates / 1 saved view |
| `agentflow goal next` after review | pass, next action plan Local Project Model v0 只读实现 |
| `agentflow review-assistant ISSUE-0019` | pass, 15 checks / ready |
| `agentflow metrics` | pass, 19 issues / 17 runs / read-only true |
| `test ! -f .agentflow/workspace.json` | pass |
| `test ! -d .agentflow/teams` | pass |
| `test ! -d .agentflow/projects` | pass |
| `agentflow plan "Local Project Model v0 只读实现"` | pass, ISSUE-0020 |
| `agentflow goal next` before run | pass, next action run ISSUE-0020 |
| `agentflow run ISSUE-0020 --dry-run` | pass, RUN-0018 |
| `agentflow goal next` with active issue | pass, next action verify ISSUE-0020 |
| `agentflow verify ISSUE-0020` | pass, 13 commands |
| `agentflow projects` | pass, LocalProjectModelSnapshot read-only output |
| `agentflow search "Local Project Model"` | pass, traceable `.agentflow/` results |
| `agentflow review ISSUE-0020` | pass, evidence / review / update generated |
| `agentflow update summary` | pass, 20 issues / 18 runs / 18 updates / 1 saved view |
| `agentflow goal next` after review | pass, next action plan Local Project Seed v0 边界定义 |
| `agentflow review-assistant ISSUE-0020` | pass, 15 checks / ready |
| `agentflow metrics` | pass, 20 issues / 18 runs / read-only true |
| `test ! -f .agentflow/workspace.json` | pass |
| `test ! -d .agentflow/teams` | pass |
| `test ! -d .agentflow/projects` | pass |
| `agentflow plan "Desktop Project View v0 只读实现"` | pass, ISSUE-0021 |
| `agentflow goal next` before run | pass, next action run ISSUE-0021 |
| `agentflow run ISSUE-0021 --dry-run` | pass, RUN-0019 |
| `agentflow goal next` with active issue | pass, next action verify ISSUE-0021 |
| `agentflow verify ISSUE-0021` | pass, 13 commands |
| `agentflow projects` | pass, active issue ISSUE-0021 remains read-only over LocalProjectModelSnapshot |
| `agentflow search "Desktop Project View"` | pass, traceable `.agentflow/` results |
| `agentflow review ISSUE-0021` | pass, evidence / review / update generated |
| `agentflow update summary` | pass, 21 issues / 19 runs / 19 updates / 1 saved view |
| `agentflow review-assistant ISSUE-0021` | pass, 15 checks / ready |
| `agentflow goal next` after review | pass, next action plan Local Project Seed v0 边界定义 |
| `agentflow plan "Desktop Workspace Overview v0 只读入口优化"` | pass, ISSUE-0022 |
| `agentflow goal next` before run | pass, next action run ISSUE-0022 |
| `agentflow run ISSUE-0022 --dry-run` | pass, RUN-0020 |
| `agentflow goal next` with active issue | pass, next action verify ISSUE-0022 |
| `agentflow verify ISSUE-0022` | pass, 13 commands |
| `agentflow projects` | pass, workspace / team / project read-only output |
| `agentflow search "Workspace Overview"` | pass, traceable `.agentflow/` results |
| `agentflow review ISSUE-0022` | pass, evidence / review / update generated |
| `agentflow update summary` | pass, 22 issues / 20 runs / 20 updates / 1 saved view |
| `agentflow review-assistant ISSUE-0022` | pass, 15 checks / ready |
| `agentflow goal next` after overview review | pass, next action plan Local Project Seed v0 边界定义 |
| Browser preview Workspace Overview | pass, 总览显示 Workspace Projects / Teams 与 Team Issues / Projects |
| `agentflow plan "Local Project Seed v0 边界定义"` | pass, ISSUE-0023 |
| `agentflow goal next` before seed boundary run | pass, next action run ISSUE-0023 |
| `agentflow run ISSUE-0023 --dry-run` | pass, RUN-0021 |
| `agentflow goal next` with seed boundary active issue | pass, next action verify ISSUE-0023 |
| `agentflow verify ISSUE-0023` | pass, 13 commands |
| `agentflow search "Local Project Seed"` | pass, traceable `.agentflow/` results |
| `agentflow review ISSUE-0023` | pass, evidence / review / update generated |
| `agentflow update summary` | pass, 23 issues / 21 runs / 21 updates / 1 saved view |
| `agentflow review-assistant ISSUE-0023` | pass, 15 checks / ready |
| `agentflow goal next` after seed boundary review | pass, next action plan Local Project Seed v0 实现 |
| `agentflow metrics` after seed boundary review | pass, 23 issues / 21 runs / read-only true |
| `agentflow projects` after seed boundary review | pass, next intent Local Project Seed v0 实现 |
| no seed files after seed boundary | pass, no workspace/team/project fact files |
| `agentflow plan "Local Project Seed v0 实现"` | pass, ISSUE-0024 |
| `agentflow run ISSUE-0024 --dry-run` | pass, RUN-0022 |
| `cargo test local_project_seed` | pass, 4 focused seed tests |
| `agentflow project-seed` | pass, read-only preview for workspace/team/project seed files |
| `agentflow verify ISSUE-0024` | pass, 14 commands |
| `cargo test` for seed implementation | pass, 27 tests |
| `npm --prefix apps/desktop run build` for seed implementation | pass |
| no live seed files after `agentflow project-seed` preview | pass |
| `agentflow review ISSUE-0024` | pass, evidence / review / update generated |
| `agentflow update summary` | pass, 24 issues / 22 runs / 22 updates / 1 saved view |
| `agentflow review-assistant ISSUE-0024` | pass, 15 checks / ready |
| `agentflow goal next` after seed implementation review | pass, next action plan Issue Project Link v0 边界定义 |
| `bash checks/agentflow-readiness.sh` after review | pass |
| no live seed files after review | pass |
| `agentflow plan "Issue Project Link v0 边界定义"` | pass, ISSUE-0025 |
| `agentflow run ISSUE-0025 --dry-run` | pass, RUN-0023 |
| `agentflow verify ISSUE-0025` | pass, 14 commands |
| `agentflow search "Issue Project Link"` | pass, traceable `.agentflow/` results |
| no issue project link migration | pass, no `projectLink` / team / project / milestone / linkSource properties in issue JSON |
| no seed files during issue link boundary | pass |
| `agentflow review ISSUE-0025` | pass, evidence / review / update generated |
| `agentflow update summary` | pass, 25 issues / 23 runs / 23 updates / 1 saved view |
| `agentflow review-assistant ISSUE-0025` | pass, 15 checks / ready |
| `agentflow goal next` after issue link boundary review | pass, next action plan Issue Project Link Writer v0 实现 |
| `bash checks/agentflow-readiness.sh` after issue link boundary review | pass |
| no issue project link migration after review | pass |
| no seed files after issue link boundary review | pass |
| `agentflow plan "Issue Project Link Writer v0 实现"` | pass, ISSUE-0026 |
| `agentflow run ISSUE-0026 --dry-run` | pass, RUN-0024 |
| `cargo test issue_project_link` | pass, 4 focused issue-link tests |
| `cargo test` for issue-link writer | pass, 31 tests |
| `npm --prefix apps/desktop run build` for issue-link writer | pass |
| `agentflow issue-link ISSUE-0025` | pass, read-only preview / no live write |
| no live issue projectLink after preview | pass |
| `agentflow search "Issue Project Link"` after writer | pass, traceable `.agentflow/` results |
| `bash checks/agentflow-readiness.sh` after writer implementation | pass, includes `agentflow issue-link ISSUE-0025` |
| `agentflow verify ISSUE-0026` | pass, 9 commands |
| `agentflow review ISSUE-0026` | pass, evidence / review / update generated |
| `agentflow update summary` | pass after sequential rerun, 26 issues / 24 runs / 24 updates / 1 saved view |
| `agentflow review-assistant ISSUE-0026` | pass, 15 checks / ready |
| `agentflow goal next` after issue-link writer review | pass after sequential rerun, next action plan Project-aware GoalLoop v0 边界定义 |
| `agentflow plan "Project-aware GoalLoop v0 边界定义"` | pass, ISSUE-0027 |
| `agentflow run ISSUE-0027 --dry-run` | pass, RUN-0025 |
| `cargo test` for Project-aware GoalLoop boundary | pass, 31 tests |
| `npm --prefix apps/desktop run build` for Project-aware GoalLoop boundary | pass |
| `agentflow projects` | pass, active issue ISSUE-0027 remains WIP=1 / verify |
| `agentflow issue-link ISSUE-0025` | pass, read-only preview remains unchanged |
| `agentflow search "Project-aware GoalLoop"` | pass, traceable `.agentflow/` results |
| `bash checks/agentflow-readiness.sh` after Project-aware GoalLoop boundary | pass |
| no live issue projectLink during Project-aware GoalLoop boundary | pass |
| `agentflow verify ISSUE-0027` | pass, 10 commands |
| `agentflow review ISSUE-0027` | pass, evidence / review / update generated |
| `agentflow update summary` | pass, 27 issues / 25 runs / 25 updates / 1 saved view |
| `agentflow review-assistant ISSUE-0027` | pass, 15 checks / ready |
| `agentflow goal next` after Project-aware GoalLoop boundary review | pass, next action plan Project-aware GoalLoop v0 实现 |
| `agentflow plan "Project-aware GoalLoop v0 实现"` | pass, ISSUE-0028 |
| `agentflow run ISSUE-0028 --dry-run` | pass, RUN-0026 |
| `cargo test project_aware_goal_loop` | pass, 4 focused project-aware tests |
| `cargo test` for Project-aware GoalLoop implementation | pass, 35 tests |
| `npm --prefix apps/desktop run build` for Project-aware GoalLoop implementation | pass |
| `agentflow goal next` with ISSUE-0028 active | pass, next action verify ISSUE-0028 |
| `agentflow projects` with ISSUE-0028 active | pass, read-only snapshot / recommended command verify ISSUE-0028 |
| `agentflow search "Project-aware GoalLoop"` after implementation | pass, traceable `.agentflow/` results |
| `bash checks/agentflow-readiness.sh` after Project-aware GoalLoop implementation | pass |
| no live issue projectLink after Project-aware GoalLoop implementation | pass |
| no live workspace/project seed after Project-aware GoalLoop implementation | pass |
| `agentflow verify ISSUE-0028` | pass, 9 commands |
| `agentflow review ISSUE-0028` | pass, evidence / review / update generated |
| `agentflow update summary` after Project-aware GoalLoop implementation | pass, 28 issues / 26 runs / 26 updates / 1 saved view |
| `agentflow review-assistant ISSUE-0028` | pass, 15 checks / ready |
| `agentflow goal next` after Project-aware GoalLoop implementation review | pass, next action plan Desktop GoalLoop Trace v0 只读展示 |
| final `bash checks/agentflow-readiness.sh` after Project-aware GoalLoop implementation | pass |
| final no live issue projectLink proof | pass |
| final no live workspace/project seed proof | pass |
| final `git diff --check` after Project-aware GoalLoop implementation | pass |
| `agentflow plan "Desktop GoalLoop Trace v0 只读展示"` | pass, ISSUE-0029 |
| `agentflow run ISSUE-0029 --dry-run` | pass, RUN-0027 |
| `npm --prefix apps/desktop run build` for Desktop GoalLoop Trace | pass |
| browser smoke for Desktop GoalLoop Trace | pass, Decision nav / trace heading / priority / recommended command / no execute text visible |
| `cargo fmt --check` for Desktop GoalLoop Trace | pass |
| `cargo test` for Desktop GoalLoop Trace | pass, 35 tests |
| `agentflow goal check` for Desktop GoalLoop Trace | pass, ready true |
| `agentflow goal next` with ISSUE-0029 active | pass, next action verify ISSUE-0029 |
| `agentflow projects` with ISSUE-0029 active | pass, read-only snapshot / recommended command verify ISSUE-0029 |
| `agentflow search "Desktop GoalLoop Trace"` | pass, traceable `.agentflow/` results |
| `bash checks/agentflow-readiness.sh` for Desktop GoalLoop Trace | pass |
| `agentflow verify ISSUE-0029` | pass, 9 commands |
| `agentflow review ISSUE-0029` | pass, evidence / review / update generated |
| `agentflow update summary` after Desktop GoalLoop Trace | pass, 29 issues / 27 runs / 27 updates / 1 saved view |
| `agentflow review-assistant ISSUE-0029` | pass, 15 checks / ready |
| `agentflow goal next` after Desktop GoalLoop Trace review | pass, next action plan Desktop Issue Lifecycle Trace v0 只读展示 |
| final `bash checks/agentflow-readiness.sh` after Desktop GoalLoop Trace | pass |
| final no live issue projectLink proof after Desktop GoalLoop Trace | pass |
| final no live workspace/project seed proof after Desktop GoalLoop Trace | pass |
| final `agentflow goal next` after Desktop GoalLoop Trace | pass, next action plan Desktop Issue Lifecycle Trace v0 只读展示 |
| final `git diff --check` after Desktop GoalLoop Trace | pass |
| `agentflow plan "Desktop Issue Lifecycle Trace v0 只读展示"` | pass, ISSUE-0030 |
| `agentflow run ISSUE-0030 --dry-run` | pass, RUN-0028 |
| `npm --prefix apps/desktop run build` for Desktop Issue Lifecycle Trace | pass |
| browser smoke for Desktop Issue Lifecycle Trace | pass, 生命周期入口 / current step / Contract / Validation / Evidence / Project Update / no execute text visible |
| no execution controls in Desktop Issue Lifecycle Trace | pass, no run / verify action buttons |
| no live issue projectLink proof after Desktop Issue Lifecycle Trace | pass, 30 issue JSON files |
| no live workspace/project seed proof after Desktop Issue Lifecycle Trace | pass |
| no search/query write directory proof after Desktop Issue Lifecycle Trace | pass |
| `cargo run -p agentflow-cli -- verify ISSUE-0030` | pass, 9 commands |
| `cargo run -p agentflow-cli -- review ISSUE-0030` | pass, evidence / review / update generated |
| `cargo run -p agentflow-cli -- update summary` after Desktop Issue Lifecycle Trace | pass, 30 issues / 28 runs / 28 updates / 1 saved view |
| `cargo run -p agentflow-cli -- review-assistant ISSUE-0030` | pass, 15 checks / ready |
| `cargo run -p agentflow-cli -- goal next` after Desktop Issue Lifecycle Trace review | pass, next action plan Desktop Project Update Timeline v0 只读展示 |
| final `cargo fmt --check` after Desktop Issue Lifecycle Trace | pass |
| final `cargo test` after Desktop Issue Lifecycle Trace | pass, 35 tests |
| final `npm --prefix apps/desktop run build` after Desktop Issue Lifecycle Trace | pass |
| final `cargo run -p agentflow-cli -- goal check` after Desktop Issue Lifecycle Trace | pass, ready true |
| final `cargo run -p agentflow-cli -- goal next` after Desktop Issue Lifecycle Trace | pass, next action plan Desktop Project Update Timeline v0 只读展示 |
| final `cargo run -p agentflow-cli -- projects` after Desktop Issue Lifecycle Trace | pass, read-only project snapshot / next intent Desktop Project Update Timeline |
| final `cargo run -p agentflow-cli -- search "Issue Lifecycle Trace"` | pass, 30 traceable results |
| final `bash checks/agentflow-readiness.sh` after Desktop Issue Lifecycle Trace | pass |
| final no live issue projectLink proof after Desktop Issue Lifecycle Trace | pass |
| final no live workspace/project seed proof after Desktop Issue Lifecycle Trace | pass |
| final no search/query write directory proof after Desktop Issue Lifecycle Trace | pass |
| final `git diff --check` after Desktop Issue Lifecycle Trace | pass |
| `agentflow plan "Desktop Project Update Timeline v0 只读展示"` | pass, ISSUE-0031 |
| `agentflow run ISSUE-0031 --dry-run` | pass, RUN-0029 |
| `npm --prefix apps/desktop run build` for Desktop Project Update Timeline | pass |
| browser smoke for Desktop Project Update Timeline | pass, 更新时间线入口 / chain / issue / run / validation / evidence / review visible |
| no execution controls in Desktop Project Update Timeline | pass, no run / verify / review action buttons |
| `cargo fmt --check` for Desktop Project Update Timeline | pass |
| `cargo test` for Desktop Project Update Timeline | pass, 35 tests |
| final `npm --prefix apps/desktop run build` for Desktop Project Update Timeline | pass |
| `cargo run -p agentflow-cli -- goal check` for Desktop Project Update Timeline | pass, ready true |
| `cargo run -p agentflow-cli -- goal next` with ISSUE-0031 active | pass, next action verify ISSUE-0031 |
| `cargo run -p agentflow-cli -- projects` with ISSUE-0031 active | pass, read-only snapshot / recommended command verify ISSUE-0031 |
| `cargo run -p agentflow-cli -- search "Project Update Timeline"` | pass, 23 traceable results before review / 36 final results |
| `bash checks/agentflow-readiness.sh` for Desktop Project Update Timeline | pass |
| no live issue projectLink proof after Desktop Project Update Timeline | pass, 31 issue JSON files |
| no live workspace/project seed proof after Desktop Project Update Timeline | pass |
| no search/query write directory proof after Desktop Project Update Timeline | pass |
| `cargo run -p agentflow-cli -- verify ISSUE-0031` | pass, 9 commands |
| `cargo run -p agentflow-cli -- review ISSUE-0031` | pass, evidence / review / update generated |
| `cargo run -p agentflow-cli -- update summary` after Desktop Project Update Timeline | pass, 31 issues / 29 runs / 29 updates / 1 saved view |
| `cargo run -p agentflow-cli -- review-assistant ISSUE-0031` | pass, 15 checks / ready |
| `cargo run -p agentflow-cli -- goal next` after Desktop Project Update Timeline review | pass, next action plan Desktop Run Validation Trace v0 只读展示 |
| final `cargo fmt --check` after Desktop Project Update Timeline | pass |
| final `cargo test` after Desktop Project Update Timeline | pass, 35 tests |
| final `npm --prefix apps/desktop run build` after Desktop Project Update Timeline | pass |
| final `cargo run -p agentflow-cli -- goal check` after Desktop Project Update Timeline | pass, ready true |
| final `cargo run -p agentflow-cli -- goal next` after Desktop Project Update Timeline | pass, next action plan Desktop Run Validation Trace v0 只读展示 |
| final `cargo run -p agentflow-cli -- projects` after Desktop Project Update Timeline | pass, read-only project snapshot / next intent Desktop Run Validation Trace |
| final `cargo run -p agentflow-cli -- search "Project Update Timeline"` | pass, 36 traceable results |
| final `bash checks/agentflow-readiness.sh` after Desktop Project Update Timeline | pass |
| final no live issue projectLink proof after Desktop Project Update Timeline | pass |
| final no live workspace/project seed proof after Desktop Project Update Timeline | pass |
| final no search/query write directory proof after Desktop Project Update Timeline | pass |
| final `git diff --check` after Desktop Project Update Timeline | pass |
| `agentflow plan "Desktop MVP Navigation Scope Reduction v0"` | pass, ISSUE-0032 |
| `agentflow run ISSUE-0032 --dry-run` | pass, RUN-0030 |
| `npm --prefix apps/desktop run build` for Desktop MVP Navigation Scope Reduction | pass |
| browser smoke for Desktop MVP Navigation Scope Reduction | pass, main nav only 总览 / 团队 / 项目 / 任务; debug nav hidden |
| `cargo fmt --check` for Desktop MVP Navigation Scope Reduction | pass |
| `cargo test` for Desktop MVP Navigation Scope Reduction | pass, 35 tests |
| final `npm --prefix apps/desktop run build` for Desktop MVP Navigation Scope Reduction | pass |
| `cargo run -p agentflow-cli -- goal check` for Desktop MVP Navigation Scope Reduction | pass, ready true |
| `cargo run -p agentflow-cli -- goal next` with ISSUE-0032 active | pass, next action verify ISSUE-0032 |
| `cargo run -p agentflow-cli -- projects` with ISSUE-0032 active | pass, read-only snapshot / recommended command verify ISSUE-0032 |
| `cargo run -p agentflow-cli -- search "Desktop MVP Navigation Scope Reduction"` | pass, 17 traceable results |
| `bash checks/agentflow-readiness.sh` for Desktop MVP Navigation Scope Reduction | pass |
| no live issue projectLink proof after Desktop MVP Navigation Scope Reduction | pass, 32 issue JSON files |
| no live workspace/project seed proof after Desktop MVP Navigation Scope Reduction | pass |
| no search/query write directory proof after Desktop MVP Navigation Scope Reduction | pass |
| `cargo run -p agentflow-cli -- verify ISSUE-0032` | pass, 8 commands |
| `cargo run -p agentflow-cli -- review ISSUE-0032` | pass, evidence / review / update generated |
| `cargo run -p agentflow-cli -- update summary` after Desktop MVP Navigation Scope Reduction | pass, 32 issues / 30 runs / 30 updates / 1 saved view |
| `cargo run -p agentflow-cli -- review-assistant ISSUE-0032` | pass, 15 checks / ready |
| `cargo run -p agentflow-cli -- goal next` after Desktop MVP Navigation Scope Reduction review | pass, next action plan Desktop MVP Task Detail v0 收敛 |
| final `bash checks/agentflow-readiness.sh` after Desktop MVP Navigation Scope Reduction | pass |
| final `cargo run -p agentflow-cli -- goal next` after Desktop MVP Navigation Scope Reduction | pass, next action plan Desktop MVP Task Detail v0 收敛 |
| final `git diff --check` after Desktop MVP Navigation Scope Reduction | pass |
| JSON parse | pass |
| `.DS_Store` scan | pass |
| `npm --prefix apps/desktop run build` for Desktop Project View | pass |

## 2026-05-28 Project / Milestone / Issue / View Model v1

执行者：Codex

目标：

- 将 AgentFlow MVP 产品主干收敛为 `Workspace / Team -> Project -> Milestone -> Issue -> View`，并落地只读 schema adapter。
- 固定 Project、Milestone、Issue、View 的职责边界、页面展示边界和 adapter 输出格式。
- 明确 View 只是 saved filter，不承载业务状态。
- 明确 Queue Preflight 是 `Backlog -> Todo` 的唯一授权门。

结果：

- 新增规格文档：`docs/specs/project-milestone-issue-view-model-v1.md`。
- 新增 core schema / adapter：
  - `ProjectMilestoneIssueViewModelSnapshot`
  - `V1WorkspaceRef`
  - `V1TeamRef`
  - `V1Project`
  - `V1Milestone`
  - `V1Issue`
  - `V1View`
  - `V1ViewFilter`
  - `V1ViewSort`
- 新增只读 reader：`read_project_milestone_issue_view_model_snapshot`。
- 更新 Desktop TypeScript 类型，保持前端类型可对齐 v1 schema。
- 更新 README、ROADMAP、MVP Spec、Goal + Criteria Driven MVP、Project / Issue Status Model、Team / Project / Milestone / Issue Writers、Desktop Workbench Boundary。
- 固定产品不变量：
  - Project 不执行。
  - Milestone 不执行。
  - Issue 执行。
  - View 只展示。
  - Queue Preflight 决定谁能执行。
  - Evidence 决定是否 Done。
- v1 状态模型作为后续产品目标，不立即破坏当前 canonical status 或 `.agentflow/` 事实源。
- Adapter 保留 `rawStatus`，同时输出 v1 派生 `status`；Milestone status 只派生，不写回事实源；View 只从 SavedView 派生 filter / sort / layout。

验证：

- `cargo test -p agentflow-core project_milestone_issue_view_model_v1 -- --nocapture`：pass，2 tests。
- `cargo fmt --check`：pass。
- `cargo test`：pass，61 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- projects`：pass。
- `git diff --check`：pass。

## 2026-05-29 Project / Milestone / Issue / View Model v1 Writer Preview Alignment

执行者：Codex

目标：

- 将“开发文档任务”推进到 100%：让 v1 产品模型不只停留在文档和只读 adapter，而是在创建命令 preview 中输出可检查的结构化合同。
- 保持 preview-first，不改变 `.agentflow/` 落盘 schema。

结果：

- `CreationPreview` 新增 `v1Contract`。
- 新增预览对象：
  - `CreationV1ContractPreview`
  - `TeamCreationV1Preview`
  - `ProjectCharterV1Preview`
  - `MilestoneGateV1Preview`
  - `IssueContractV1Preview`
  - `ViewFilterV1Preview`
- `agentflow team create` preview 输出 Team relation。
- `agentflow project create` preview 输出 Project charter。
- `agentflow milestone create` preview 输出 Milestone gate。
- `agentflow issue create` preview 输出 Issue execution contract。
- CLI preview 会打印 v1 model / relation / 关键合同摘要。
- 更新 Team / Project / Milestone / Issue Writers、Project / Milestone / Issue / View Model v1、README、ROADMAP、MVP Spec。

边界：

- 不改变 `.agentflow/` 写入 schema。
- 不执行 run / verify / review。
- 不调用模型。
- 不创建远程 PR / GitHub issue / Linear issue。

验证：

- `cargo test -p agentflow-core team_project_milestone_issue_writers_preview_without_writes -- --nocapture`：pass。
- `cargo fmt --check`：pass。
- `cargo test`：pass，61 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- project create "开发文档任务"`：pass，preview only，输出 v1 Project charter。
- `cargo run -p agentflow-cli -- team create "开发文档团队"`：pass，preview only，输出 v1 Team relation。
- `cargo run -p agentflow-cli -- milestone create "开发文档阶段"`：pass，preview only，输出 v1 Milestone gate。
- `cargo run -p agentflow-cli -- issue create "开发文档任务"`：pass，preview only，输出 v1 Issue execution contract。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next`：pass，保持 WIP=1，推荐 `agentflow verify ISSUE-0043`。
- `cargo run -p agentflow-cli -- projects`：pass。
- `bash checks/agentflow-readiness.sh`：pass。
- `git diff --check`：pass。
- touched-file trailing whitespace scan：pass。

## 2026-05-29 Desktop Project / Milestone / Issue 页面职责收敛 v0

执行者：Codex

目标：

- 把 `Project / Milestone / Issue / View Model v1` 从 Core / CLI / 文档层呈现到 Desktop Workbench。
- Desktop 继续只读，不提供创建 UI，不执行 run / verify / review。

结果：

- 新增 Tauri command：`load_project_milestone_issue_view_model_snapshot`。
- Desktop load flow 同时读取 `ProjectMilestoneIssueViewModelSnapshot`。
- Project 页面收敛为 Project charter、milestones、issue progress、queue status、closure gate。
- Milestone 区块收敛为 milestone goal、entry criteria、issues、exit criteria、derived progress。
- Issue 页面收敛为 issue contract：goal、scope、non-goals、validation、evidence、boundary、status。
- View 页面只展示 saved filter / sort / layout，不承载业务状态。
- Project 排序按 active -> draft -> completed -> canceled；Issue 排序按 in_progress -> in_review -> todo -> backlog -> done -> canceled。
- 更新 README、ROADMAP、MVP Spec、Desktop Workbench Boundary、Project / Milestone / Issue / View Model v1。

边界：

- 不写 `.agentflow/`。
- 不创建 Team / Project / Milestone / Issue。
- 不执行 run / verify / review。
- 不调用模型。
- 不创建远程 PR / GitHub issue / Linear issue。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，61 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- projects`：pass。
- `bash checks/agentflow-readiness.sh`：pass。
- `git diff --check`：pass。
- Browser verification at `http://127.0.0.1:1420/`：pass，Project / Issue / View 页面均可打开，页面非空，console warn/error 为 0。
- Browser screenshots saved：
  - `/tmp/agentflow-desktop-project-page.png`
  - `/tmp/agentflow-desktop-issue-page.png`

## 当前下一步

`Project / Milestone / Issue / View Model v1` 已完成产品合同、只读 schema adapter、writer preview 对齐和 Desktop 页面职责收敛。下一候选实现切片应从二选一开始：

1. Queue Preflight v1：把 `Backlog -> Todo` 从人工状态切换改为本地计算 + 显式确认。
2. Saved View v1：把“当前 Todo / 高风险 / 缺证据 / Ready for closure”做成 filter，而不是新业务层级。

当前仍不进入自动 run / verify / review。
