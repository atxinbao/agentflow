# Verification

创建日期：2026-05-21
执行者：Codex

本文档是 AgentFlow 的 append-only 历史验证记录。2026-06-01 之后的新需求验证应追加到本文档末尾，并以 `docs/requirements/` 中的新需求为准；旧 `docs/validation/` 内容已归档，不再作为默认开发入口。

## 2026-05-21 - AEP project document pass

执行者：Codex

范围：

- 按 AEP 编号流程重新梳理 AgentFlow 项目文档。
- 新增 Project Definition 初始化合同。
- 新增 Construction Plan / Local Draft。
- 新增 latest verification summary。

计划验证：

- `git diff --check`
- `rg` 一致性检查

结果：

- `git diff --check`：pass。

## 2026-06-05 Execute Patch / Checkpoint V1

执行者：Codex

目标：

- 执行 `docs/requirements/010-execute-patch-checkpoint-v1.md`。
- 新增受控执行层 `execute/`。
- execute 只能从 `.agentflow/input/issues/<issue-id>.json` 启动。
- 每次 run 记录 preflight、lease、plan、checkpoint、patch / command、validation、result 和 evidence。

结果：

- 新增 `crates/execute`，Cargo package 为 `agentflow-execute`。
- 新增 Execute canonical layout：
  - `.agentflow/execute/manifest.json`
  - `.agentflow/execute/index.json`
  - `.agentflow/execute/runs/`
  - `.agentflow/execute/leases/`
  - `.agentflow/execute/queue/`
  - `.agentflow/output/evidence/`
- 新增 Execute public API：
  - `prepare_execute_workspace`
  - `validate_execute_workspace`
  - `load_execute_status`
  - `load_execute_manifest`
  - `load_execute_index`
  - `load_execute_snapshot`
  - `create_execute_run`
  - `execute_run_preflight`
  - `confirm_high_risk_execute_run`
  - `acquire_execute_lease`
  - `release_execute_lease`
  - `write_execute_plan`
  - `create_execute_checkpoint`
  - `apply_execute_patch`
  - `run_execute_command`
  - `validate_execute_run`
  - `complete_execute_run`
  - `cancel_execute_run`
- Preflight 已检查 ownership、define、panel、input、issue、Approved SPEC、risk、lease、working tree 和 validation hints。
- High risk issue 必须写入 run-scoped human confirmation 后才可通过 preflight。
- Low / medium risk issue 不需要 human confirmation。
- Lease 防止同一个 issue 同时拥有多个 active run。
- Patch 必须在 checkpoint 后执行，并且只能修改 run plan 的 `allowedWritePaths`。
- Command record 结构化记录 program、args、cwd、exitCode、stdout、stderr。
- Dangerous command 已阻断，包括 shell freeform、git push / commit / checkout / reset / clean、rm -rf、deploy、release、curl。
- Result 和 evidence 已绑定，completed / failed run 均会释放 lease。
- Desktop Tauri 新增 execute commands；Desktop human UI 只读展示 execute 状态。
- Desktop 状态通道新增“执行流水线”事件。
- Browser Preview mock 已同步 execute status。
- README / GOAL / ROADMAP / docs index / requirements index 已更新到 010。

边界：

- 未修改 `.agentflow/input/**` facts。
- 未修改 Approved SPEC。
- 未创建 PR。
- 未 merge。
- 未 release。
- 未 deploy。
- 未调用模型。
- 未把 Desktop human UI 改成执行入口。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-execute`：pass，12 tests。
- `cargo test -p agentflow-desktop`：pass，17 tests。
- `cargo test`：pass，agent-manual 21 tests + CLI 2 tests + core 61 tests + desktop 17 tests + execute 12 tests + goal-tree 3 tests + input 8 tests + panel 27 tests。
- `npm --prefix apps/desktop run build`：pass。
- `git diff --check`：pass。

## 2026-06-05 Execute Lease Preflight Polish

执行者：Codex

目标：

- 修复 Released lease 误阻断同 issue 后续 execute run preflight。
- 保持 Active lease 对并发 run 的阻断。
- 将 corrupted / unreadable lease 作为 blocked preflight 返回。
- 修复 ExecutePatchOutcome 的 patch artifact 返回路径。

结果：

- `execute_run_preflight` 不再仅根据 `.agentflow/execute/leases/<issue-id>.json` 是否存在判断阻断。
- lease 文件不存在时，lease check passed。
- `lease.status = Active` 时，lease check blocked。
- `lease.status = Released` 时，lease check passed。
- lease 文件损坏或不可解析时，lease check blocked，并返回 `Lease state unreadable`。
- `ExecutePatchOutcome.proposedPatchPath` 返回 `.agentflow/execute/runs/<run-id>/patches/proposed.patch`。
- `ExecutePatchOutcome.appliedPatchPath` 返回 `.agentflow/execute/runs/<run-id>/patches/applied.patch`。
- `ExecutePatchOutcome.worktreeDiffPath` 返回 `.agentflow/execute/runs/<run-id>/patches/worktree.diff`。
- 新增 `released_lease_does_not_block_second_run_preflight` 测试。
- 新增 `corrupted_lease_blocks_preflight` 测试。

边界：

- 未改变 run / preflight / lease / checkpoint / patch / command / result / evidence 主流程。
- 未写 input issue。
- 未写 Approved SPEC。
- 未创建 PR、merge、release 或 deploy。
- 未调用模型。
- 未修改 Desktop UI 交互。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-execute`：pass，14 tests。
- `cargo test`：pass，agent-manual 21 tests + CLI 2 tests + core 61 tests + desktop 17 tests + execute 14 tests + goal-tree 3 tests + input 8 tests + panel 27 tests。
- `npm --prefix apps/desktop run build`：pass。
- `git diff --check`：pass。

## 2026-06-05 Agent Role Consolidation V2

执行者：Codex

目标：

- 执行 `docs/requirements/010-2-agent-role-consolidation-v2.md`。
- 将 V1 顶层 Agent 角色收敛为 Spec / Build / Audit。
- 删除独立 Release Agent 顶层角色，但保留 release delivery 能力并归入 Build Agent。
- 新增 Build Agent release delivery 输出模型和 API。

结果：

- `Agentflow.md` 模板的 Agent Roles 只保留：
  - Spec Agent / 需求规格 Agent
  - Build Agent / 实现交付 Agent
  - Audit Agent / 代码审计 Agent
- Build Agent 明确负责：
  - `.agentflow/execute/runs/<run-id>/`
  - `.agentflow/output/evidence/<run-id>.json`
  - `.agentflow/output/release/<run-id>/`
- RELEASE.md 模板改为 Build Agent 的 release delivery manual。
- `.agentflow/output/release/` 纳入 Agent Manual layout 和 Execute paths。
- `ExecuteResultNext` 从 `readyForRelease` 改为 `readyForDelivery` + `needsAudit`。
- 新增 `OutputReleaseDelivery` / `OutputReleaseDeliveryArtifacts`。
- 新增 `prepare_release_delivery` / `load_release_delivery`。
- `prepare_release_delivery` 写入：
  - `.agentflow/output/release/<run-id>/delivery.json`
  - `.agentflow/output/release/<run-id>/pr-draft.md`
  - `.agentflow/output/release/<run-id>/pr-metadata.json`
  - `.agentflow/output/release/<run-id>/review-checklist.md`
  - `.agentflow/output/release/<run-id>/changelog.md`
  - `.agentflow/output/release/<run-id>/release-note.md`
- Browser Preview mock layout 增加 `.agentflow/output/release`。
- README / GOAL / ROADMAP / requirements index 已更新到 010.2。

边界：

- 未恢复独立 Release Agent。
- 未改 input 模型。
- 未改 panel 模型。
- 未改 riskLevel 规则。
- 未绕过 execute preflight、checkpoint、lease 或 evidence。
- 未 merge。
- 未 deploy。
- 未直接发布生产。
- 未调用模型。
- 未启动 Audit Agent。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-agent-manual`：pass，23 tests。
- `cargo test -p agentflow-execute`：pass，17 tests。
- `cargo test -p agentflow-desktop`：pass，17 tests。
- `cargo test`：pass，agent-manual 23 tests + CLI 2 tests + core 61 tests + desktop 17 tests + execute 17 tests + goal-tree 3 tests + input 8 tests + panel 27 tests。
- `npm --prefix apps/desktop run build`：pass。
- `git diff --check`：pass。
- `rg -n "Project Definition|CONSTRUCTION_PLAN|Construction Plan|latest-verification|verification.md|0\\. New Project Initialization|1\\. Human Project Planning|2\\. Construction Plan|3\\. Linear execution contract|施工材料|未授权执行|不授权" ...`：pass。

结论：

- AEP Flow 0、Human Planning、Construction Plan、local issue contract、Root Docs Refresh 的项目文档落点已建立。
- 当前只完成文档梳理，未授权代码实现。

## 2026-06-05 Human-triggered Audit Report V1

执行者：Codex

目标：

- 执行 `docs/requirements/012-human-triggered-audit-report-v1.md`。
- 将 Audit 明确为人类主动触发的完整审计报告，而不是每次 execute / output 后的自动流程。
- 统一写入 `.agentflow/output/audit/<audit-id>/`，不按 run / project / batch 拆三套目录模型。

结果：

- 新增 `docs/requirements/012-human-triggered-audit-report-v1.md` 并更新 requirements index。
- `prepare_output_workspace` 只创建 output/audit root、`manifest.json` 和 `index.json`，不会创建 `<audit-id>` 报告目录。
- 新增 `request_human_audit`，人类触发后写入：
  - `.agentflow/output/audit/<audit-id>/audit-request.json`
  - `.agentflow/output/audit/<audit-id>/audit.json`
  - `.agentflow/output/audit/<audit-id>/audit-report.md`
  - `.agentflow/output/audit/<audit-id>/findings.json`
  - `.agentflow/output/audit/<audit-id>/checklist.md`
  - `.agentflow/output/audit/<audit-id>/evidence-map.json`
  - `.agentflow/output/audit/<audit-id>/traceability.json`
- 新增 `load_audit_report`、`load_audit_index`、`load_audit_status`。
- Audit V1 固定检查 7 项：
  - checkpoint exists
  - changed files recorded
  - allowedWritePaths only
  - command records complete
  - high risk confirmation if needed
  - evidence complete
  - release delivery complete
- Desktop Tauri 新增 human audit trigger 和 audit read commands。

边界：

- Audit 不自动随 execute / output 生成。
- Audit 不修改 input facts。
- Audit 不修改 execute facts。
- Audit 不修改 output evidence。
- Audit 不修改 output release delivery。
- Audit 不写用户源码。
- Audit 不执行命令。
- Audit 不创建 PR、merge 或 deploy。
- Audit 不调用模型。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-output`：pass，18 tests。
- `cargo test -p agentflow-desktop`：pass，17 tests。
- `cargo test`：pass，agent-manual 23 tests + CLI 2 tests + core 61 tests + desktop 17 tests + execute 17 tests + goal-tree 3 tests + input 8 tests + output 18 tests + panel 27 tests。
- `npm --prefix apps/desktop run build`：pass。
- `git diff --check`：pass。

## 2026-05-22 - Startup flow archive pass

执行者：Codex

范围：

- 将根目录文档收敛为入口层。
- 将产品、设计、架构、规划、规格、参考文档归档到 `docs/`。
- 新增 `docs/startup/0.1-project-initialization-questions.md`，保存项目初始化 15 问。
- 新增 `docs/startup/0.2-reference-reading-blueprint.md`，保存参考项目阅读和蓝图初始化。
- 新增 `docs/startup/0.3-project-map-and-archive.md`，保存项目地图和归档规则。
- 更新 `README.md`、`AGENTS.md`、`docs/startup/project-definition.md`、`docs/planning/construction-plan.md` 和验证摘要。

验证：

- `git diff --check`：pass。
- 旧根目录大写文档路径 `rg` 检查：pass，未发现残留。
- `find . -maxdepth 1 -type f`：pass，根目录仅保留 `.gitignore`、`AGENTS.md`、`GOAL.md`、`README.md`、`ROADMAP.md`、`verification.md`。
- 0.1 / 0.2 / 0.3 启动流程锚点 `rg` 检查：pass。

结论：

- 根目录文件过多的问题已通过归档层解决。
- 0.1 / 0.2 / 0.3 项目启动流程已补齐。
- 当前仍未授权代码实现。

## 2026-05-22 - Linear reference blueprint pass

执行者：Codex

范围：

- 阅读 Linear 官方入口、docs、conceptual model、project updates 和 developers GraphQL 文档。
- 按 0.2 参考阅读流程更新 AgentFlow 产品蓝图、设计蓝图和架构蓝图。
- 明确采用 Linear 的对象模型、动态 Views、Project Updates、Agent 可观察性和 future API 边界意识。
- 明确不采用 Linear 的 SaaS workspace/team 层级、cycles、initiatives、enterprise roadmap、远程 GraphQL API、webhook 平台和云同步。
- 同步更新 README、ROADMAP、MVP Spec、Construction Plan、Product Requirements、Design Spec、Architecture、ADR 和 latest verification summary。

参考来源：

- `https://linear.app/homepage`
- `https://linear.app/docs/start-guide`
- `https://linear.app/docs/conceptual-model`
- `https://linear.app/docs/initiative-and-project-updates`
- `https://linear.app/developers/graphql`

验证：

- `git diff --check`：pass。
- `rg -n "Linear homepage|Linear 参考项目提炼|SavedView|ProjectUpdate|Local View Engine|Project Update Summary|远程 GraphQL API|Stage 1 到 Stage 11|views/|updates/" ...`：pass。
- stale MVP range `rg`：pass，未发现旧范围残留。
- `rg -n "[ \t]+$" README.md GOAL.md ROADMAP.md AGENTS.md docs verification.md`：pass，未发现尾随空白。
- `find . -maxdepth 1 -type f -print`：pass，根目录仍只保留 `.gitignore`、`AGENTS.md`、`GOAL.md`、`README.md`、`ROADMAP.md`、`verification.md`。

结论：

- 0.2 参考蓝图已经从 AEP-only 扩展为 AEP + Linear 双参考，但 AgentFlow 定位仍是本地 AI 执行工作台。
- MVP 范围调整为 Flow 0 + Stage 1 到 Stage 11，新增 Saved Views 和 Project Update Summary，仍不授权代码实现。

## 2026-05-22 - Full documentation compression pass

执行者：Codex

范围：

- 重新压缩根目录入口文档和 `docs/` 下所有项目文档。
- 保留 Flow 0、AEP/Linear 参考边界、MVP Stage、核心对象、CLI 规格、验证入口和未授权边界。
- 将重复叙述压缩为事实表、边界表和最小规则。
- 保持 `verification.md` append-only，不重写历史记录。

压缩结果：

- Markdown 总行数从 2607 行降到 1272 行。
- `docs/specs/mvp-spec.md` 从 441 行降到 180 行。
- `docs/architecture/architecture-decisions.md` 从 251 行降到 52 行。
- `docs/design/design-spec.md` 从 206 行降到 63 行。

验证：

- `git diff --check`：pass。
- `rg` Linear / SavedView / ProjectUpdate / Flow 0 / not authorized anchors：pass。
- stale MVP range `rg`：pass。
- trailing whitespace `rg`：pass。
- root file check：pass。

结论：

- 当前文档已完成第二轮压缩。
- 项目仍处于文档蓝图阶段，未授权代码实现。

## 2026-05-22 - Goal execution spine hardening pass

执行者：Codex

范围：

- 将 `/goal -> 可验证工程完成` 固化为 `Goal Execution Spine`。
- 把底层链条写入 README、ROADMAP、Architecture、MVP Spec、Construction Plan、Project Definition、Product Requirements、ADR 和 latest summary。
- 第一候选施工包从 `AgentFlow Core + CLI Bootstrap v0` 收紧为 `Goal Compiler + Core/CLI Bootstrap v0`。
- `.agentflow/` 事实源新增 `goal.{md,json}`，并将 `agentflow init --from-goal GOAL.md` 定为第一入口。
- 在新增底层链条后继续压缩文档，总 Markdown 行数从 1272 行降到 1228 行。

验证：

- `git diff --check`：pass。
- Goal spine anchor `rg`：pass。
- stale bootstrap wording `rg`：pass。
- trailing whitespace `rg`：pass。
- docs line count：pass。

结论：

- 当前基础已从“Core/CLI 泛化启动”收紧为“GoalCompiler 优先启动”。
- 项目仍处于文档蓝图阶段，未授权代码实现。

## 2026-05-22 - Goal Compiler + Core/CLI Bootstrap v0

执行者：Codex

范围：

- 建立 Rust workspace。
- 新增 `crates/agentflow-core`。
- 新增 `crates/agentflow-cli`，二进制命令名为 `agentflow`。
- 实现 `agentflow init --from-goal GOAL.md`。
- 实现 `agentflow goal check`。
- 实现离线模板版 `agentflow plan "..."`。
- 生成 `.agentflow/goal.{md,json}`、settings、index、初始化 evidence 和 `ISSUE-0001` issue contract。

验证：

- `brew install rust`：pass。
- `cargo fmt --check`：pass。
- `cargo test`：pass，3 tests。
- `cargo run -p agentflow-cli -- init --from-goal GOAL.md`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass。
- `cargo run -p agentflow-cli -- plan "实现 Goal Compiler + Core/CLI Bootstrap v0"`：pass。
- JSON parse for `.agentflow/goal.json`、settings、index、`ISSUE-0001.json`：pass。
- `git diff --check`：pass。

结论：

- Goal Compiler、Local Project Store、Issue Contract Builder 已具备可运行雏形。
- Context Collector / Planner 仍是模板级能力。
- Codex Runtime Adapter、Validation Runner、Evidence Chain、Review / ProjectUpdate Generator 尚未产品化实现。

## 2026-05-22 - Context Collector + Planner v0

执行者：Codex

范围：

- 新增 `ProjectContext` 和 `ContextFile`。
- 实现 repo 文件扫描，跳过 `.git`、`target`、`node_modules`、`.agentflow/runs`、`.agentflow/tmp`、`.env*` 和 `.DS_Store`。
- 实现 `agentflow context`，输出 `.agentflow/context.json` 和 `.agentflow/context.md`。
- `agentflow plan` 在存在 `.agentflow/context.json` 时使用上下文文件填充 issue contract。
- 补充 context collection 和 context-aware planning 单元测试。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，5 tests。
- `cargo run -p agentflow-cli -- context`：pass，44 files。

结论：

- Context Collector 已具备可运行雏形。
- Planner 已能读取上下文，但仍是 deterministic template，不调用模型。
- 下一步应进入 `Codex Runtime Adapter + Validation / Evidence v0`。

## 2026-05-22 - Codex Runtime Adapter + Validation / Evidence v0

执行者：Codex

范围：

- 新增 `AgentRun`、`RunOutputs`、`CommandRecord`。
- 实现 `agentflow run ISSUE-0003 --dry-run`，生成 `.agentflow/runs/RUN-0001/` 下的 `run.json`、`transcript.md`、`commands.jsonl`、`diff-summary.md`。
- 实现 `agentflow verify ISSUE-0003`，执行 issue contract 中的本地 validation commands，并将 stdout / stderr / exit code 写入 `commands.jsonl` 和 `run.json`。
- 实现 `agentflow review ISSUE-0003`，生成 `.agentflow/evidence/ISSUE-0003-evidence.md`、`.agentflow/reviews/ISSUE-0003-review.md`、`.agentflow/updates/PROJECT-UPDATE-0001.md`，并在验证通过后把 issue 标记为 completed。
- 补充 Runtime Adapter、Validation Runner、Evidence Chain、Review / ProjectUpdate Generator 的核心单元测试。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，8 tests。
- `cargo run -p agentflow-cli -- plan "实现 Codex Runtime Adapter + Validation / Evidence v0"`：pass，生成 `ISSUE-0003`。
- `cargo run -p agentflow-cli -- run ISSUE-0003 --dry-run`：pass，生成 `RUN-0001`。
- `cargo run -p agentflow-cli -- verify ISSUE-0003`：pass，2 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0003`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- context`：pass，49 files。
- JSON parse for `.agentflow/index.json`、`.agentflow/issues/ISSUE-0003.json`、`.agentflow/runs/RUN-0001/run.json`、`.agentflow/context.json`：pass。
- `find . -name .DS_Store -print`：pass，已清理 macOS 自动生成文件。

结论：

- CLI MVP 已从 `/goal -> init -> context -> plan -> issue` 推进到 `run -> verify -> evidence -> review -> update`。
- Runtime Adapter v0 仍是 dry-run only，不调用外部模型、不上传代码、不自动改文件。
- 下一候选应进入 `SQLite Index + Saved Views v0`。

## 2026-05-22 - SQLite Index + Saved Views v0

执行者：Codex

范围：

- 新增 `rusqlite` bundled SQLite 依赖。
- 新增 `SavedView`、`SavedViewFilter`、`IndexedIssue`、`IndexedRun`、`IndexedUpdate`。
- 实现 `agentflow index rebuild`，从 `.agentflow/issues`、`.agentflow/runs`、`.agentflow/updates`、`.agentflow/views` 重建 `.agentflow/index.sqlite`。
- 实现 `agentflow view save`，保存 filter-only SavedView JSON，不保存查询结果。
- 实现 `agentflow view show`，从 SQLite 索引读取 SavedView 查询结果。
- 将 `.agentflow/index.sqlite` 和 WAL/SHM 文件加入 ignore，并从 context 扫描中排除，保持 JSON / Markdown 为事实源。
- 生成 `ISSUE-0004`、`RUN-0002`、`ISSUE-0004` evidence / review 和 `PROJECT-UPDATE-0002`。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，10 tests。
- `cargo run -p agentflow-cli -- context`：pass，55 files。
- `cargo run -p agentflow-cli -- plan "实现 SQLite Index + Saved Views v0"`：pass，生成 `ISSUE-0004`。
- `cargo run -p agentflow-cli -- index rebuild`：pass，4 issues / 2 runs / 2 updates / 1 saved view。
- `cargo run -p agentflow-cli -- view save completed --issue-status completed --run-status completed --validation-status passed`：pass。
- `cargo run -p agentflow-cli -- view show completed`：pass，4 issues / 2 runs。
- `cargo run -p agentflow-cli -- run ISSUE-0004 --dry-run`：pass，生成 `RUN-0002`。
- `cargo run -p agentflow-cli -- verify ISSUE-0004`：pass，2 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0004`：pass，生成 evidence / review / update。

结论：

- SQLite Index + Saved Views v0 已具备可运行雏形。
- `.agentflow/index.sqlite` 是可重建查询索引，不是事实源。
- SavedView 只保存 filter，不保存结果、不授权执行。
- 下一候选应进入 `Project Update Summary + Review Assistant v0`。

## 2026-05-22 - Project Update Summary + Review Assistant v0

执行者：Codex

范围：

- 新增 `ProjectSummaryResult`、`ReviewAssistantSummary`、`ReviewAssistantCheck`。
- 实现 `agentflow update summary`，生成 `.agentflow/updates/PROJECT-SUMMARY.md`，汇总 issue / run / update / saved view 当前状态。
- 实现 `agentflow review-assistant ISSUE-0005`，生成 `.agentflow/reviews/ISSUE-0005-assistant.md`，检查 issue contract、scope、validation、run、evidence、review、project update 和 SQLite index。
- 生成 `ISSUE-0005`、`RUN-0003`、`ISSUE-0005` evidence / review / review assistant 和 `PROJECT-UPDATE-0003`。
- `completed` SavedView 已能读取 5 个 completed issue 和 3 个 passed run。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，12 tests。
- `cargo run -p agentflow-cli -- context`：pass，62 files。
- `cargo run -p agentflow-cli -- plan "实现 Project Update Summary + Review Assistant v0"`：pass，生成 `ISSUE-0005`。
- `cargo run -p agentflow-cli -- run ISSUE-0005 --dry-run`：pass，生成 `RUN-0003`。
- `cargo run -p agentflow-cli -- verify ISSUE-0005`：pass，2 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0005`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0005`：pass，9 checks / ready。
- `cargo run -p agentflow-cli -- update summary`：pass，5 issues / 3 runs / 3 updates / 1 saved view。
- `cargo run -p agentflow-cli -- view show completed`：pass，5 issues / 3 runs。

结论：

- Project Update Summary + Review Assistant v0 已具备可运行雏形。
- Review Assistant 只做本地审查清单，不执行远程 PR 操作，不变更团队 workspace。
- 下一候选应进入 `Desktop Workbench MVP v0`。

## 2026-05-22 - Desktop Workbench MVP v0 Boundary

执行者：Codex

范围：

- 新增 `docs/specs/desktop-workbench-mvp-boundary.md`。
- 明确 Desktop Workbench MVP v0 是只读桌面工作台，不是新的执行引擎。
- 定义输入事实源：goal、index、context、Project Summary、SavedView、issue、run、evidence、review、review assistant、可重建 SQLite index。
- 定义 MVP 内功能：Project Overview、Issue List、Issue Detail、Run / Validation、Evidence / Review、Saved Views、Refresh。
- 明确不做：创建/编辑 issue、执行 run/verify/review、调用模型、写入 `.agentflow/` 事实源、创建 PR / 远程 issue、账号/云同步、完整 PM 看板。
- 同步 README、ROADMAP、MVP Spec、Design Spec、Product Requirements、Architecture、Construction Plan 和 `.agentflow/roadmap.md`。
- 生成 `ISSUE-0006`、`RUN-0004`、`ISSUE-0006` evidence / review / review assistant 和 `PROJECT-UPDATE-0004`。

验证：

- `cargo run -p agentflow-cli -- context`：pass，69 files。
- `cargo run -p agentflow-cli -- plan "定义 Desktop Workbench MVP v0 边界"`：pass，生成 `ISSUE-0006`。
- `cargo run -p agentflow-cli -- run ISSUE-0006 --dry-run`：pass，生成 `RUN-0004`。
- `cargo run -p agentflow-cli -- verify ISSUE-0006`：pass，2 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0006`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0006`：pass，9 checks / ready。
- `cargo run -p agentflow-cli -- update summary`：pass，6 issues / 4 runs / 4 updates / 1 saved view。
- `cargo run -p agentflow-cli -- view show completed`：pass，6 issues / 4 runs。
- `cargo fmt --check`：pass。
- `cargo test`：pass，12 tests。
- `git diff --check`：pass。
- JSON parse for `.agentflow/index.json`、`.agentflow/issues/ISSUE-0006.json`、`.agentflow/runs/RUN-0004/run.json`、`.agentflow/context.json`、`.agentflow/views/completed.json`：pass。
- `find . -name .DS_Store -print`：pass。
- Desktop boundary anchor `rg`：pass。

结论：

- Desktop Workbench MVP v0 边界已确立。
- 下一候选可以进入 `Desktop Workbench MVP v0 只读壳实现`。
- 实现阶段仍必须保持只读：不触发执行、不写入 `.agentflow/` 事实源、不调用模型、不接远程 PR / issue 平台。

## 2026-05-22 - AEP Goal Initialization Protocol v0

执行者：Codex

范围：

- 将 Codex `/goal` 固化为 AEP 第一阶段新项目初始化入口。
- 补齐 `ProjectDefinition`、`AgentScopeState`、AEP bootstrap artifacts、goal readiness check。
- 扩展 `IssueContract`，加入 stop condition、fastest feedback loop、vertical slice、tracer bullet、diagnose、Graphify status、docs claim trace、boundary confirmation、PR handoff requirements。
- 扩展 Review Assistant，把 AEP issue protocol、boundary、docs claim trace、Graphify context 和 scope state 纳入本地 review gate。

变更：

- 新增 `agentflow goal bootstrap`，可在既有 `.agentflow/` 中补齐 AEP 第一阶段初始化包，不重置 issue / run / update 历史。
- `agentflow init --from-goal` 新项目初始化时直接生成 `.agentflow/project-definition.json`、`.agentflow/scope-state.json` 和 `.agentflow/bootstrap/*`。
- `agentflow goal check` 改为检查 ProjectGoal、ProjectDefinition、ScopeState、environment、architecture、roadmap、初始化 evidence 和 bootstrap 产物。
- `agentflow run` 会通过 scope state 维护 WIP=1 active issue；review 通过后释放 active issue。
- 新增 `docs/specs/aep-goal-initialization-protocol.md` 和 `checks/agentflow-readiness.sh`。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，13 tests。
- `cargo run -p agentflow-cli -- goal bootstrap`：pass，补齐当前 `.agentflow/` AEP 初始化包。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- run ISSUE-0007 --dry-run`：pass，RUN-0005。
- `cargo run -p agentflow-cli -- verify ISSUE-0007`：pass，2 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0007`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0007`：pass，14 checks / ready。
- `cargo run -p agentflow-cli -- update summary`：pass，7 issues / 5 runs / 5 updates / 1 saved view。
- `bash checks/agentflow-readiness.sh`：pass。
- `git diff --check`：pass。

结论：

- AEP 第一阶段已经产品化为 `/goal` 初始化协议。
- 当前 8 个底层能力不再只是可跑通链路，而是带有 AEP 启动合同、scope state、机械 readiness 和 review gate。
- 下一候选恢复为 `Desktop Workbench MVP v0 只读壳实现`，仍需 Human 明确授权。

## 2026-05-22 - Goal Loop Orchestrator v0

执行者：Codex

范围：

- 新增 `GoalLoopState`、`GoalLoopIssueRef`、`GoalLoopCounts`、`GoalLoopSources`。
- 新增 `agentflow goal next`，读取 goal、project definition、scope state、index、roadmap、issues、runs、evidence、reviews 和 project summary。
- `goal next` 写出 `.agentflow/goal-loop.json` 和 `.agentflow/updates/GOAL-LOOP-SUMMARY.md`。
- 决策结果限制为 `plan`、`run`、`verify`、`review`、`update`、`wait-human`。
- active issue 存在时优先完成当前 issue，保持 WIP=1，不推荐新 plan。
- Review Assistant 新增 Goal Loop readiness 检查。

变更：

- `agentflow goal next` 在 `ISSUE-0008` planned 状态下返回 `run ISSUE-0008 --dry-run`。
- `agentflow goal next` 在 `ISSUE-0008` active 状态下返回 `verify ISSUE-0008`。
- `agentflow goal next` 在 `ISSUE-0008` 完成后返回 `plan "Desktop Workbench MVP v0 只读壳实现"`。
- `checks/agentflow-readiness.sh` 增加 Goal Loop Orchestrator 锚点和 `agentflow goal next`。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，15 tests。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next`：pass，写出 goal-loop / summary。
- `cargo run -p agentflow-cli -- run ISSUE-0008 --dry-run`：pass，RUN-0006。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify。
- `cargo run -p agentflow-cli -- verify ISSUE-0008`：pass，2 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0008`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0008`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- update summary`：pass，8 issues / 6 runs / 6 updates / 1 saved view。
- `bash checks/agentflow-readiness.sh`：pass。
- `git diff --check`：pass。

结论：

- `/goal` 初始化后的项目推进大脑已具备本地 v0。
- 当前系统已经能回答“下一步应该做什么”，但仍不自动执行、不创建远程 issue、不调用模型、不绕过 IssueContract。
- 下一候选仍是 `Desktop Workbench MVP v0 只读壳实现`。

## 2026-05-22 - Desktop Workbench MVP v0 Read-only Shell

执行者：Codex

范围：

- 新增 `DesktopWorkbenchSnapshot`、`WorkbenchCounts`、`WorkbenchTextArtifact`、`WorkbenchBoundary`。
- 新增 `agentflow-core::read_desktop_workbench_snapshot`，从 `.agentflow/` 只读读取 Project Summary、Goal Loop Summary、issues、runs、saved views、evidence 和 reviews。
- 新增 Tauri 2 桌面壳 `apps/desktop/src-tauri`，通过 `load_workbench_snapshot` 暴露只读快照。
- 新增 React + TypeScript UI：Overview、Issues、Issue Detail、Evidence、Review、Saved Views。
- UI 明确展示 next action 和 recommended command，并标注只读，不创建 issue、不执行 run / verify / review、不调用模型、不写 `.agentflow/`。
- 更新 README、ROADMAP、MVP Spec、Desktop Workbench Boundary、latest verification summary 和 readiness anchors。
- 修复 Review Assistant docs claim trace 校验，使其同时支持 `.agentflow/` 路径和 repo-relative 路径。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，17 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next`：pass，next action plan，recommended command `agentflow plan "Local Pro Experiments v0 边界定义"`。
- `cargo run -p agentflow-cli -- run ISSUE-0009 --dry-run`：pass，RUN-0007。
- `cargo run -p agentflow-cli -- verify ISSUE-0009`：pass，7 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0009`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0009`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- update summary`：pass，9 issues / 7 runs / 7 updates / 1 saved view。
- `bash checks/agentflow-readiness.sh`：pass。
- `git diff --check`：pass。
- `npm --prefix apps/desktop run tauri -- dev`：pass，Tauri dev process 启动并运行，无 runtime error，已手动停止。

结论：

- Desktop Workbench MVP v0 只读壳已完成本地可启动版本。
- `.agentflow/` 仍是事实源；桌面 UI 只读快照，不拥有执行和写入能力。
- 下一候选为 `Local Pro Experiments v0 边界定义`，未授权前不进入。

## 2026-05-22 - Local Pro Experiments v0 Boundary

执行者：Codex

范围：

- 新增 `docs/specs/local-pro-experiments-boundary.md`。
- 明确 Local Pro Experiments 包含本地 analytics / metrics、DuckDB 后置分析、local project intelligence、本地搜索 / saved query、多项目 workspace 和 Desktop Workbench 后续交互能力。
- 明确当前阶段禁止云同步、账号 / 支付、团队协作、远程 PR / Linear issue、Desktop UI 执行 run / verify / review、自动模型调用和绕过 IssueContract 写 `.agentflow/`。
- 为每个候选实验定义只读性、`.agentflow/` 写入、IssueContract、人类确认、验证命令和 evidence 授权门。
- 更新 README、ROADMAP、MVP Spec、Construction Plan、latest verification summary 和 readiness anchors。
- 将下一候选推进为 `Local Metrics Snapshot v0 只读实现`。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，17 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` before run：pass，next action run ISSUE-0010。
- `cargo run -p agentflow-cli -- run ISSUE-0010 --dry-run`：pass，RUN-0008。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0010。
- `cargo run -p agentflow-cli -- verify ISSUE-0010`：pass，7 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0010`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0010`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- update summary`：pass，10 issues / 8 runs / 8 updates / 1 saved view。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Local Metrics Snapshot v0 只读实现`。
- `bash checks/agentflow-readiness.sh`：pass。
- `git diff --check`：pass。

结论：

- Local Pro Experiments v0 已完成边界定义，没有实现具体高级功能。
- Desktop Workbench 仍保持只读，不新增执行按钮、不写 `.agentflow/`。
- 下一候选为 `Local Metrics Snapshot v0 只读实现`，未授权前不进入。

## 2026-05-22 - Local Metrics Snapshot v0 Read-only Implementation

执行者：Codex

范围：

- 新增 `LocalMetricsSnapshot`、`LocalIssueMetrics`、`LocalRunMetrics`、`LocalArtifactMetrics`、latest run / artifact 引用对象。
- 新增 `read_local_metrics_snapshot`，复用 `read_desktop_workbench_snapshot` 派生只读 metrics，不写 `.agentflow/`，不创建 DuckDB 或 analytics cache。
- 新增 CLI 命令 `agentflow metrics`。
- Desktop Workbench 新增 Metrics 只读视图，并通过 `load_metrics_snapshot` Tauri command 读取本地 metrics。
- 更新 README、ROADMAP、MVP Spec、Local Pro Boundary、latest verification summary、readiness script。
- 将下一候选推进为 `Local Search v0 边界定义`。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，19 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` before run：pass，next action run ISSUE-0011。
- `cargo run -p agentflow-cli -- run ISSUE-0011 --dry-run`：pass，RUN-0009。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0011。
- `cargo run -p agentflow-cli -- verify ISSUE-0011`：pass，8 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0011`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0011`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- update summary`：pass，11 issues / 9 runs / 9 updates / 1 saved view。
- `cargo run -p agentflow-cli -- metrics`：pass，11 issues / 9 runs / read-only true。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Local Search v0 边界定义`。
- `bash checks/agentflow-readiness.sh`：pass。
- `git diff --check`：pass。

结论：

- Local Metrics Snapshot v0 已完成只读实现。
- 指标全部来自 `.agentflow/` 和现有 snapshot，没有新增 DuckDB，没有写 `.agentflow/analytics`。
- Desktop Metrics 视图只读展示，不执行命令。
- 下一候选为 `Local Search v0 边界定义`，未授权前不进入。

## 2026-05-22 - Local Search v0 Boundary Definition

执行者：Codex

范围：

- 收紧 `ISSUE-0012` contract，明确本阶段只定义 Local Search / saved query 边界。
- 新增 `docs/specs/local-search-boundary.md`，定义候选能力、禁止项、可搜索路径、排除路径、结果字段、literal query、saved query 后置规则、索引边界和 Desktop 只读边界。
- 更新 README、ROADMAP、MVP Spec、Local Pro Boundary、construction plan、readiness script 和 latest verification summary。
- 将下一候选推进为 `Local Search Reader v0 只读实现`。
- 未实现搜索引擎，未引入 Tantivy / SQLite FTS / DuckDB FTS，未新增 Desktop 搜索 UI，未写 `.agentflow/search` 或 `.agentflow/queries`。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，19 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` before run：pass，next action run ISSUE-0012。
- `cargo run -p agentflow-cli -- run ISSUE-0012 --dry-run`：pass，RUN-0010。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0012。
- `cargo run -p agentflow-cli -- verify ISSUE-0012`：pass，8 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0012`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0012`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- update summary`：pass，12 issues / 10 runs / 10 updates / 1 saved view。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Local Search Reader v0 只读实现`。
- `cargo run -p agentflow-cli -- metrics`：pass，12 issues / 10 runs / read-only true。
- `bash checks/agentflow-readiness.sh`：pass。
- `git diff --check`：pass。

结论：

- Local Search v0 已完成边界定义。
- 可搜索路径和排除路径已经明确，saved query 明确后置。
- Desktop Workbench 未新增搜索 UI。
- 下一候选为 `Local Search Reader v0 只读实现`，未授权前不进入。

## 2026-05-22 - Desktop Workbench 中文界面优化 v0

执行者：Codex

范围：

- 创建并收紧 `ISSUE-0013` contract，限定为 Desktop Workbench 中文界面优化。
- 将导航、加载态、顶部状态、Overview、Metrics、Issues、Issue Detail、Evidence、Review、Saved Views、Boundary Panel、空状态和 tooltip 改为中文。
- 保留 issue id、run id、命令、路径和事实源原始值的可追溯性。
- 同步浏览器 mock/fallback 的中文项目摘要、目标循环摘要和指标示例。
- 更新 HTML `lang`、页面标题、Tauri 窗口标题和中文字体栈。
- 未新增执行按钮、issue 写入入口、搜索 UI、搜索索引或模型调用。

验证：

- `npm --prefix apps/desktop run build`：pass。
- Tauri dev：pass，Vite HMR 更新，Tauri 因配置变更自动 rebuild 并重新运行。
- `cargo run -p agentflow-cli -- goal next` before run：pass，next action run ISSUE-0013。
- `cargo run -p agentflow-cli -- run ISSUE-0013 --dry-run`：pass，RUN-0011。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0013。
- `cargo run -p agentflow-cli -- verify ISSUE-0013`：pass，8 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0013`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0013`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- update summary`：pass，13 issues / 11 runs / 11 updates / 1 saved view。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Local Search Reader v0 只读实现`。
- `cargo run -p agentflow-cli -- metrics`：pass，13 issues / 11 runs / read-only true。

结论：

- Desktop Workbench 已完成中文界面优化。
- 浏览器预览和 Tauri 桌面入口的文案风格已统一为中文；真实数据仍以 Tauri 桌面入口为准。
- Desktop Workbench 继续保持只读边界。

## 2026-05-22 - Local Search Reader v0 Read-only Implementation

执行者：Codex

范围：

- 新增 `LocalSearchQuery`、`LocalSearchResult`、`LocalSearchSnapshot`。
- 新增 `read_local_search_snapshot`，只读扫描 `docs/specs/local-search-boundary.md` 授权的 `.agentflow/` JSON / JSONL / Markdown 路径。
- 新增 CLI 命令 `agentflow search "<query>"`。
- 搜索结果包含 `sourceType`、`entityKind`、`entityId`、`path`、`title`、`field`、`line`、`snippet`、`score`。
- 新增测试覆盖 traceable result、排除 `.agentflow/search` / `.agentflow/queries` / `index.sqlite`、不创建 query/search 目录。
- 更新 README、ROADMAP、MVP Spec、Local Search Boundary、Local Pro Boundary、construction plan、readiness script 和 latest verification summary。
- 将下一候选推进为 `Saved Query v0 边界定义`。
- 未新增 Desktop 搜索 UI，未创建搜索索引，未写 `.agentflow/search` 或 `.agentflow/queries`，未调用模型。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，21 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` before run：pass，next action run ISSUE-0014。
- `cargo run -p agentflow-cli -- run ISSUE-0014 --dry-run`：pass，RUN-0012。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0014。
- `cargo run -p agentflow-cli -- verify ISSUE-0014`：pass，9 commands。
- `cargo run -p agentflow-cli -- search "Local Search"`：pass，返回 `.agentflow/` 内可追溯结果，包含 path / line / snippet / entityKind / entityId。
- `cargo run -p agentflow-cli -- review ISSUE-0014`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0014`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- update summary`：pass，14 issues / 12 runs / 12 updates / 1 saved view。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Saved Query v0 边界定义`。
- `cargo run -p agentflow-cli -- metrics`：pass，14 issues / 12 runs / read-only true。
- `bash checks/agentflow-readiness.sh`：pass。
- `git diff --check`：pass。

结论：

- Local Search Reader v0 已完成只读实现。
- `agentflow search "Local Search"` 可以返回带 path、line、snippet、entityKind、entityId 的可追溯结果。
- 搜索只读，不生成索引、不写 query 文件、不修改事实源。
- Desktop Workbench 未新增搜索 UI。
- 下一候选为 `Saved Query v0 边界定义`，未授权前不进入。

## 2026-05-22 - Saved Query v0 Boundary Definition

执行者：Codex

范围：

- 创建并收紧 `ISSUE-0015` contract，限定为 Saved Query v0 边界定义。
- 新建 `docs/specs/saved-query-boundary.md`。
- 明确 Saved Query 与现有 SavedView 分离：SavedView 继续保存 issue/run filter，Saved Query 后续首选 `.agentflow/queries/*.json`。
- 定义 saved query schema 候选、路径规则、用户确认门、验证方式和 evidence 要求。
- 更新 README、ROADMAP、Construction Plan、MVP Spec、Local Pro Boundary、Local Search Boundary、readiness script 和 latest verification summary。
- 将下一候选推进为 `Desktop Search Read-only View v0 边界定义`。
- 未创建 `.agentflow/queries`，未写 saved query 文件，未保存搜索结果，未新增 CLI / Desktop 搜索 UI，未调用模型。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，21 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` before run：pass，next action run ISSUE-0015。
- `cargo run -p agentflow-cli -- run ISSUE-0015 --dry-run`：pass，RUN-0013。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0015。
- `cargo run -p agentflow-cli -- verify ISSUE-0015`：pass，10 commands。
- `cargo run -p agentflow-cli -- search "Saved Query"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass。
- `test ! -d .agentflow/queries`：pass。
- `cargo run -p agentflow-cli -- review ISSUE-0015`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0015`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- update summary`：pass，15 issues / 13 runs / 13 updates / 1 saved view。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Desktop Search Read-only View v0 边界定义`。
- `cargo run -p agentflow-cli -- metrics`：pass，15 issues / 13 runs / read-only true。

结论：

- Saved Query v0 已完成边界定义。
- 后续 saved query 写入必须另建 IssueContract，并需要用户确认 `.agentflow/queries/*.json` 路径和内容。
- 当前阶段没有创建 query 目录、没有保存结果、没有新增 Desktop 搜索 UI。
- 下一候选为 `Desktop Search Read-only View v0 边界定义`，未授权前不进入。

## 2026-05-22 - Desktop Search Read-only View v0 Boundary Definition

执行者：Codex

范围：

- 创建并收紧 `ISSUE-0016` contract，限定为 Desktop Search Read-only View v0 边界定义。
- 新建 `docs/specs/desktop-search-readonly-boundary.md`。
- 定义 Desktop 搜索入口只允许调用 Local Search Reader 的只读能力。
- 定义后续 UI 契约：query 输入框、result list、source trace、empty / loading / error 状态、read-only badge 和 recommended command 只展示。
- 更新 README、ROADMAP、Construction Plan、MVP Spec、Local Pro Boundary、Local Search Boundary、Saved Query Boundary、readiness script 和 latest verification summary。
- 将下一候选推进为 `Desktop Search Read-only View v0 实现`。
- 未新增 Desktop 搜索 UI 实现，未写 `.agentflow/search` 或 `.agentflow/queries`，未保存搜索结果，未执行 run / verify / review，未创建 issue，未调用模型，未上传远程。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，21 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` before run：pass，next action run ISSUE-0016。
- `cargo run -p agentflow-cli -- run ISSUE-0016 --dry-run`：pass，RUN-0014。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0016。
- `cargo run -p agentflow-cli -- verify ISSUE-0016`：pass，10 commands。
- `cargo run -p agentflow-cli -- search "Desktop Search"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass。
- `test ! -d .agentflow/queries`：pass。
- `test ! -d .agentflow/search`：pass。
- `cargo run -p agentflow-cli -- review ISSUE-0016`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0016`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- update summary`：pass，16 issues / 14 runs / 14 updates / 1 saved view。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Desktop Search Read-only View v0 实现`。
- `cargo run -p agentflow-cli -- metrics`：pass，16 issues / 14 runs / read-only true。

结论：

- Desktop Search Read-only View v0 已完成边界定义。
- 后续 Desktop 搜索 UI 实现必须只调用 Local Search Reader，只读展示结果和 source trace。
- 当前阶段没有新增 Desktop 搜索 UI、没有写 search/query 目录、没有保存搜索结果、没有执行命令。
- 下一候选为 `Desktop Search Read-only View v0 实现`，未授权前不进入。

## 2026-05-22 - Desktop Search Read-only View v0 Implementation

执行者：Codex

范围：

- 创建并收紧 `ISSUE-0017` contract，限定为 Desktop Search Read-only View v0 实现。
- 新增 Tauri command `load_search_snapshot`，直接调用 `agentflow_core::read_local_search_snapshot`。
- Desktop Workbench 新增 Search 导航和只读 Search 视图。
- Search 视图展示 query 输入框、result list、source trace、empty / loading / error 状态、read-only badge 和 recommended command 文本。
- 搜索结果展示 path、line、snippet、entityKind、entityId、score。
- 浏览器预览增加只读 mock search snapshot；真实数据仍以 Tauri 桌面窗口为准。
- 更新 README、ROADMAP、Construction Plan、MVP Spec、Local Pro Boundary、Local Search Boundary、Saved Query Boundary、Desktop Search Boundary、readiness script 和 latest verification summary。
- 将下一候选推进为 `Saved Query Writer v0 边界定义`。
- 未写 `.agentflow/search` 或 `.agentflow/queries`，未保存搜索结果，未创建 issue，未执行 run / verify / review，未调用模型，未上传远程，未新增 saved query writer。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，21 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` before run：pass，next action run ISSUE-0017。
- `cargo run -p agentflow-cli -- run ISSUE-0017 --dry-run`：pass，RUN-0015。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0017。
- `cargo run -p agentflow-cli -- verify ISSUE-0017`：pass，11 commands。
- `cargo run -p agentflow-cli -- search "Desktop Search"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass。
- `test ! -d .agentflow/queries`：pass。
- `test ! -d .agentflow/search`：pass。
- `cargo run -p agentflow-cli -- review ISSUE-0017`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0017`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- update summary`：pass，17 issues / 15 runs / 15 updates / 1 saved view。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Saved Query Writer v0 边界定义`。
- `cargo run -p agentflow-cli -- metrics`：pass，17 issues / 15 runs / read-only true。

结论：

- Desktop Search Read-only View v0 已完成只读实现。
- 搜索结果来自 Local Search Reader，并通过 Tauri command 读取本地 `.agentflow/` 事实源。
- UI 不写 `.agentflow/`，recommended command 只展示不执行。
- 下一候选为 `Saved Query Writer v0 边界定义`，未授权前不进入。

## 2026-05-22 - Saved Query Writer v0 Boundary

执行者：Codex

范围：

- 重启 Desktop Workbench Tauri dev client。
- 创建并收紧 `ISSUE-0018` contract，限定为 Saved Query Writer v0 边界定义。
- 新建 `docs/specs/saved-query-writer-boundary.md`。
- 定义 `.agentflow/queries/{query-id}.json` 写入路径、`SavedQueryDefinition` schema、用户确认门、验证矩阵和 evidence 要求。
- 更新 README、ROADMAP、Construction Plan、MVP Spec、Local Pro Boundary、Local Search Boundary、Saved Query Boundary、Desktop Search Boundary、readiness script 和 latest verification summary。
- 将下一候选推进为 `Saved Query Writer v0 实现`。
- 未创建 `.agentflow/queries`，未写 saved query JSON 文件，未保存搜索结果，未创建 `.agentflow/search`、索引或 cache，未实现 writer。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，21 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` before run：pass，next action run ISSUE-0018。
- `cargo run -p agentflow-cli -- run ISSUE-0018 --dry-run`：pass，RUN-0016。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0018。
- `cargo run -p agentflow-cli -- verify ISSUE-0018`：pass，11 commands。
- `cargo run -p agentflow-cli -- search "Saved Query Writer"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass。
- `test ! -d .agentflow/queries`：pass。
- `test ! -d .agentflow/search`：pass。
- `cargo run -p agentflow-cli -- review ISSUE-0018`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0018`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- update summary`：pass，18 issues / 16 runs / 16 updates / 1 saved view。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Saved Query Writer v0 实现`。
- `cargo run -p agentflow-cli -- metrics`：pass，18 issues / 16 runs / read-only true。

结论：

- Saved Query Writer v0 已完成边界定义。
- 后续 Writer 实现必须在用户确认点下创建 `.agentflow/queries/*.json` query definition。
- 当前阶段没有创建 query 目录、没有写 query 文件、没有保存搜索结果、没有放宽 Desktop 只读边界。
- 下一候选为 `Saved Query Writer v0 实现`，未授权前不进入。

## 2026-05-22 - Local Workspace / Team / Project Model v0 Boundary

执行者：Codex

范围：

- 基于 Linear 的 workspace / team / project / issue 关系，锁定 AgentFlow 本地最小关系版本。
- 创建并收紧 `ISSUE-0019` contract，限定为 Local Workspace / Team / Project Model v0 边界定义。
- 新建 `docs/specs/local-workspace-project-model-boundary.md`。
- 明确本地关系为 `LocalWorkspace -> LocalTeams -> IssueContracts`、`LocalWorkspace -> LocalProjects -> Milestones -> IssueContracts`、`GoalLoop -> 从 active project 里选择下一条 issue`。
- 定义 `LocalWorkspace`、`LocalTeam`、`LocalProject`、`Milestone`、`GoalLoopSelection` 的职责和后续 schema 候选。
- 更新 README、ROADMAP、Construction Plan、MVP Spec、Local Pro Boundary、Architecture Decisions、readiness script 和 latest verification summary。
- 将下一候选推进为 `Local Project Model v0 只读实现`。
- 未创建 `.agentflow/workspace.json`、`.agentflow/teams` 或 `.agentflow/projects`，未迁移既有 issue，未实现 project-aware GoalLoop。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，21 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` before run：pass，next action run ISSUE-0019。
- `cargo run -p agentflow-cli -- run ISSUE-0019 --dry-run`：pass，RUN-0017。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0019。
- `cargo run -p agentflow-cli -- verify ISSUE-0019`：pass，13 commands。
- `cargo run -p agentflow-cli -- search "Local Workspace"`：pass，返回 `.agentflow/` 内可追溯结果。
- `cargo run -p agentflow-cli -- search "Local Project"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass。
- `test ! -f .agentflow/workspace.json`：pass。
- `test ! -d .agentflow/teams`：pass。
- `test ! -d .agentflow/projects`：pass。
- `cargo run -p agentflow-cli -- review ISSUE-0019`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0019`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- update summary`：pass，19 issues / 17 runs / 17 updates / 1 saved view。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Local Project Model v0 只读实现`。
- `cargo run -p agentflow-cli -- metrics`：pass，19 issues / 17 runs / read-only true。
- `git diff --check`：pass。

结论：

- Local Workspace / Team / Project Model v0 已完成边界定义。
- 当前最小本地组织模型已经锁定，但仍停留在文档和 readiness 层。
- 后续 `Local Project Model v0 只读实现` 才允许新增只读 reader 和派生对象。
- Project-aware GoalLoop 仍是后续阶段，不在本阶段实现。

## 2026-05-22 - Local Project Model v0 Read-only Implementation

执行者：Codex

范围：

- 创建并收紧 `ISSUE-0020` contract，限定为 Local Project Model v0 只读实现。
- 新增 `LocalProjectModelSnapshot`、`LocalWorkspace`、`LocalTeam`、`LocalProject`、`LocalMilestone`、`LocalProjectIssueRef`、`GoalLoopSelection` 数据对象。
- 新增 core reader `read_local_project_model_snapshot`，只从现有 `.agentflow/` 事实源派生默认 workspace、core team、active project、current milestone 和 issue refs。
- 新增 CLI 命令 `agentflow projects`，只展示本地项目模型和 GoalLoop selection，不执行命令。
- 更新 README、ROADMAP、Construction Plan、MVP Spec、Local Pro Boundary、Local Workspace / Project Boundary、Architecture Decisions、readiness script 和 latest verification summary。
- 将下一候选推进为 `Local Project Seed v0 边界定义`。
- 未创建 `.agentflow/workspace.json`、`.agentflow/teams` 或 `.agentflow/projects`，未迁移 issue，未实现 Project-aware GoalLoop，未新增 Desktop Project View。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，23 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` before run：pass，next action run ISSUE-0020。
- `cargo run -p agentflow-cli -- run ISSUE-0020 --dry-run`：pass，RUN-0018。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0020。
- `cargo run -p agentflow-cli -- verify ISSUE-0020`：pass，13 commands。
- `cargo run -p agentflow-cli -- projects`：pass，输出 LocalProjectModelSnapshot 只读摘要。
- `cargo run -p agentflow-cli -- metrics`：pass，20 issues / 18 runs / read-only true。
- `cargo run -p agentflow-cli -- search "Local Project Model"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass。
- `test ! -f .agentflow/workspace.json`：pass。
- `test ! -d .agentflow/teams`：pass。
- `test ! -d .agentflow/projects`：pass。
- `cargo run -p agentflow-cli -- review ISSUE-0020`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0020`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- update summary`：pass，20 issues / 18 runs / 18 updates / 1 saved view。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Local Project Seed v0 边界定义`。
- `git diff --check`：pass。

结论：

- Local Project Model v0 已完成只读实现。
- `agentflow projects` 现在可以展示本地 workspace/team/project/milestone/read-only issue refs。
- 当前仍没有落盘 workspace/team/project seed，Project-aware GoalLoop 也仍未实现。
- 下一候选为 `Local Project Seed v0 边界定义`，未授权前不进入写入阶段。

## 2026-05-22 - Desktop Project View v0 Read-only Implementation

执行者：Codex

范围：

- 创建并收紧 `ISSUE-0021` contract，限定为 Desktop Project View v0 只读实现。
- 新增 Tauri command `load_project_model_snapshot`，直接复用 Rust core `read_local_project_model_snapshot`。
- Desktop Workbench 新增 Project 视图，展示 LocalWorkspace、LocalTeams、active LocalProject、milestones、issue refs、GoalLoopSelection、source trace 和 recommended command 文本。
- 浏览器预览增加只读 `LocalProjectModelSnapshot` mock；真实层级数据仍以 Tauri 桌面窗口为准。
- 更新 README、ROADMAP、Construction Plan、MVP Spec、Local Pro Boundary、Local Workspace / Project Boundary、Architecture Decisions、readiness script 和 latest verification summary。
- 保持下一候选为 `Local Project Seed v0 边界定义`。
- 未创建 `.agentflow/workspace.json`、`.agentflow/teams` 或 `.agentflow/projects`，未实现 Project-aware GoalLoop，未增加 Desktop 执行按钮。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，23 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` before run：pass，next action run ISSUE-0021。
- `cargo run -p agentflow-cli -- run ISSUE-0021 --dry-run`：pass，RUN-0019。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0021。
- `cargo run -p agentflow-cli -- verify ISSUE-0021`：pass，13 commands。
- `cargo run -p agentflow-cli -- projects`：pass，输出 active issue 与只读 LocalProjectModelSnapshot。
- `cargo run -p agentflow-cli -- metrics`：pass，RUN-0019 validation 已进入只读 metrics。
- `cargo run -p agentflow-cli -- search "Desktop Project View"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass，包含 Desktop Project View 和 `load_project_model_snapshot` anchors。
- `test ! -f .agentflow/workspace.json`：pass。
- `test ! -d .agentflow/teams`：pass。
- `test ! -d .agentflow/projects`：pass。
- `cargo run -p agentflow-cli -- review ISSUE-0021`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- update summary`：pass，21 issues / 19 runs / 19 updates / 1 saved view。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0021`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Local Project Seed v0 边界定义`。
- `git diff --check`：pass。

观察：

- 将 `update summary`、`review-assistant` 和 `goal next` 临时并行执行时，SQLite index 出现一次建表 / readonly 写入冲突；按 CLI 正常顺序重跑后 `review-assistant` 和 `goal next` 均通过。

结论：

- Desktop Project View v0 已完成只读实现，本地 workspace/team/project/milestone 关系现在可在桌面工作台查看。
- Project 视图只展示 read model 和 recommended command，不会创建 project seed，也不会绕过 IssueContract。
- 下一候选仍为 `Local Project Seed v0 边界定义`。

## 2026-05-23 - Desktop Workspace Overview v0 Read-only Entry

执行者：Codex

范围：

- 依据 Workspace / Team / Project 层级反馈创建并收紧 `ISSUE-0022`。
- Desktop Overview 复用现有 `LocalProjectModelSnapshot`，把 Workspace 入口放到总览顶部。
- Workspace 入口展示 workspace 下的 Projects 和 Teams。
- Team 摘要展示 Issues 计数和关联 Projects，保留 Team WIP 只读提示。
- 更新 README、MVP Spec、Local Workspace / Project Boundary、readiness script 和 latest verification summary。
- 保持下一候选为 `Local Project Seed v0 边界定义`。
- 未创建 `.agentflow/workspace.json`、`.agentflow/teams` 或 `.agentflow/projects`，未新增 Desktop 写入或执行入口。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，23 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` before run：pass，next action run ISSUE-0022。
- `cargo run -p agentflow-cli -- run ISSUE-0022 --dry-run`：pass，RUN-0020。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0022。
- `cargo run -p agentflow-cli -- verify ISSUE-0022`：pass，13 commands。
- `cargo run -p agentflow-cli -- projects`：pass，输出 workspace / team / project 只读摘要。
- `cargo run -p agentflow-cli -- metrics`：pass，22 issues / 20 runs / read-only true。
- `cargo run -p agentflow-cli -- search "Workspace Overview"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass，包含 Workspace Overview 文档、UI 和 search anchors。
- `test ! -f .agentflow/workspace.json`：pass。
- `test ! -d .agentflow/teams`：pass。
- `test ! -d .agentflow/projects`：pass。
- `cargo run -p agentflow-cli -- review ISSUE-0022`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- update summary`：pass，22 issues / 20 runs / 20 updates / 1 saved view。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0022`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Local Project Seed v0 边界定义`。
- Browser preview：pass，总览顶部显示 Workspace Projects / Teams，Team 下显示 Issues / Projects。
- `git diff --check`：pass。

结论：

- 总览页现在承担 Workspace 入口职责，Workspace / Team / Project 的层级关系不用先进入 Project 视图才能看到。
- 当前 UI 仍只读复用 `LocalProjectModelSnapshot`，不会提前落盘 Workspace / Team / Project seed。
- 下一候选仍为 `Local Project Seed v0 边界定义`。

## 2026-05-23 - Local Project Seed v0 Boundary

执行者：Codex

范围：

- 创建并收紧 `ISSUE-0023` contract，限定为 Local Project Seed v0 边界定义。
- 新建 `docs/specs/local-project-seed-boundary.md`。
- 定义 `.agentflow/workspace.json`、`.agentflow/teams/*.json`、`.agentflow/projects/*.json` 的 seed 写入合同。
- 定义 seed source、schema 候选、路径规则、用户确认门、覆盖 / 回滚规则、后续实现边界和 evidence 要求。
- 更新 README、ROADMAP、MVP Spec、Local Pro Boundary、Local Workspace / Project Boundary、Construction Plan、Architecture Decisions、readiness script 和 latest verification summary。
- 将下一候选推进为 `Local Project Seed v0 实现`。
- 未创建 `.agentflow/workspace.json`、`.agentflow/teams` 或 `.agentflow/projects`，未实现 seed writer，未迁移 issue，未实现 Project-aware GoalLoop。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，23 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` before run：pass，next action run ISSUE-0023。
- `cargo run -p agentflow-cli -- run ISSUE-0023 --dry-run`：pass，RUN-0021。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0023。
- `cargo run -p agentflow-cli -- verify ISSUE-0023`：pass，13 commands。
- `cargo run -p agentflow-cli -- projects`：pass，输出 workspace / team / project 只读摘要。
- `cargo run -p agentflow-cli -- metrics`：pass，23 issues / 21 runs / read-only true。
- `cargo run -p agentflow-cli -- search "Local Project Seed"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass，包含 Local Project Seed Boundary 和 search anchors。
- `test ! -f .agentflow/workspace.json`：pass。
- `test ! -d .agentflow/teams`：pass。
- `test ! -d .agentflow/projects`：pass。
- `cargo run -p agentflow-cli -- review ISSUE-0023`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- update summary`：pass，23 issues / 21 runs / 21 updates / 1 saved view。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0023`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Local Project Seed v0 实现`。
- `git diff --check`：pass。

观察：

- 一次并行读取中 `metrics` 先于 `goal next` 完成时读到了旧 active issue；按顺序重跑后 active issue 为 none，下一命令为 `agentflow plan "Local Project Seed v0 实现"`。

结论：

- Local Project Seed v0 已完成边界定义。
- 后续 seed writer 的路径、schema、source 和用户确认门已明确。
- 当前仍未创建 workspace/team/project seed 文件。
- 下一候选为 `Local Project Seed v0 实现`。

## 2026-05-23 - Local Project Seed v0 Implementation

执行者：Codex

范围：

- 创建并收紧 `ISSUE-0024` contract，限定为 Local Project Seed v0 实现。
- 新增 `LocalProjectSeedPreview`、`LocalProjectSeedFile`、`LocalProjectSeedWriteSummary`。
- 新增 `read_local_project_seed_preview` 和 `write_local_project_seed`。
- 新增 CLI 命令 `agentflow project-seed`。
- 默认 `agentflow project-seed` 只输出 preview，不写 `.agentflow/`。
- 显式 `agentflow project-seed --write --yes` 才创建 `.agentflow/workspace.json`、`.agentflow/teams/core.json`、`.agentflow/projects/agentflow-local-execution.json`。
- writer 默认拒绝覆盖已有 seed 文件。
- seed writer 不迁移 issue，不写 `teamId` / `projectId` 到 issue contract，不实现 Project-aware GoalLoop，不新增 Desktop 写入 UI。
- 更新 README、ROADMAP、MVP Spec、Local Project Seed Boundary、Local Workspace Boundary、Local Pro Boundary、Construction Plan、Architecture Decisions、readiness script 和 latest verification summary。
- 将下一候选推进为 `Issue Project Link v0 边界定义`。

验证：

- `cargo fmt --check`：pass。
- `cargo test local_project_seed`：pass，4 focused seed tests。
- `cargo test`：pass，27 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0024。
- `cargo run -p agentflow-cli -- projects`：pass，输出 LocalProjectModelSnapshot 只读摘要。
- `cargo run -p agentflow-cli -- project-seed`：pass，输出 3 个待创建 seed 文件和 9 个确认门，read-only preview true。
- `test ! -f .agentflow/workspace.json`：pass。
- `test ! -d .agentflow/teams`：pass。
- `test ! -d .agentflow/projects`：pass。
- `cargo run -p agentflow-cli -- metrics`：pass。
- `cargo run -p agentflow-cli -- search "Local Project Seed"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass，包含 `agentflow project-seed`、LocalProjectSeedPreview、writer 和 Issue Project Link anchors。
- `cargo run -p agentflow-cli -- verify ISSUE-0024`：pass，14 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0024`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- update summary`：pass，24 issues / 22 runs / 22 updates / 1 saved view。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0024`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Issue Project Link v0 边界定义`。
- `bash checks/agentflow-readiness.sh` after review：pass。
- no live seed files after review：pass。
- `git diff --check`：pass。

结论：

- Local Project Seed v0 已实现默认只读 preview 和显式确认 writer。
- 当前 live repo 仍未创建 workspace/team/project seed 文件。
- 显式写入路径由 tempdir 单元测试覆盖，避免在当前 repo 未授权落盘 seed。
- 下一候选为 `Issue Project Link v0 边界定义`。

## 2026-05-25 - Issue Project Link v0 Boundary

执行者：Codex

范围：

- 创建并收紧 `ISSUE-0025` contract，限定为 Issue Project Link v0 边界定义。
- 新建 `docs/specs/issue-project-link-boundary.md`。
- 定义候选 `IssueContract.projectLink` 对象，字段为 `teamId`、`projectId`、`milestoneId`、`linkSource`。
- 定义 link source、写入路径、用户确认门、后续实现顺序、验证矩阵和 evidence 要求。
- 明确当前阶段不迁移现有 issue，不写 `projectLink` 或 team/project/milestone/linkSource 真实字段。
- 明确当前阶段不改写 GoalLoop，不新增 Desktop 写入口，不绕过 IssueContract。
- 更新 README、ROADMAP、MVP Spec、Local Workspace Boundary、Local Project Seed Boundary、Local Pro Boundary、Construction Plan、Architecture Decisions、readiness script 和 latest verification summary。
- 将下一候选推进为 `Issue Project Link Writer v0 实现`。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，27 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0025。
- `cargo run -p agentflow-cli -- projects`：pass，LocalProjectModelSnapshot 仍只读。
- `cargo run -p agentflow-cli -- project-seed`：pass，仍为 read-only preview。
- `cargo run -p agentflow-cli -- search "Issue Project Link"`：pass，返回 `.agentflow/` 内可追溯结果。
- no issue project link migration proof：pass，`.agentflow/issues/*.json` 没有真实 `projectLink`、`teamId`、`projectId`、`milestoneId`、`linkSource` 属性。
- `test ! -f .agentflow/workspace.json`：pass。
- `test ! -d .agentflow/teams`：pass。
- `test ! -d .agentflow/projects`：pass。
- `bash checks/agentflow-readiness.sh`：pass，包含 Issue Project Link Boundary / Writer anchors 和 no-migration proof。
- `cargo run -p agentflow-cli -- verify ISSUE-0025`：pass，14 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0025`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- update summary`：pass，25 issues / 23 runs / 23 updates / 1 saved view。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0025`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Issue Project Link Writer v0 实现`。
- `bash checks/agentflow-readiness.sh` after review：pass。
- no issue project link migration after review：pass。
- no seed files after review：pass。
- `git diff --check`：pass。

结论：

- Issue Project Link v0 已完成边界定义。
- 候选字段、写入路径、用户确认门和 no-migration proof 已明确。
- 当前没有迁移任何现有 issue，也没有写 project/team/milestone/linkSource 真实字段。
- 下一候选为 `Issue Project Link Writer v0 实现`。

## 2026-05-25 - Issue Project Link Writer v0

执行者：Codex

范围：

- 在 `ISSUE-0026` contract 下实现 `IssueProjectLink` 和 `IssueProjectLinkPreview`。
- 新增 `agentflow issue-link ISSUE-XXXX` CLI preview，默认只读，不写 `.agentflow/`。
- 新增显式 writer：只有 `agentflow issue-link ISSUE-XXXX --write --yes` 才写指定 issue。
- writer 只更新 `.agentflow/issues/{issue-id}.json` 和 `.agentflow/issues/{issue-id}.md`。
- Markdown 渲染新增 `## Project Link` 可读摘要，但仅在 issue 已有 `projectLink` 时输出。
- writer 拒绝覆盖已有 `projectLink`，不批量迁移历史 issue，不改写 GoalLoop，不新增 Desktop 写入口。
- 更新 README、ROADMAP、MVP Spec、Issue Project Link Boundary、Local Workspace Boundary、Local Pro Boundary、Construction Plan、Architecture Decisions、readiness script 和 latest verification summary。
- 将下一候选推进为 `Project-aware GoalLoop v0 边界定义`。

验证：

- `cargo fmt --check`：pass。
- `cargo test issue_project_link`：pass，4 focused tests。
- `cargo test`：pass，31 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0026。
- `cargo run -p agentflow-cli -- issue-link ISSUE-0025`：pass，read-only preview，action write，recommended `--write --yes`。
- no live issue projectLink proof：pass，`.agentflow/issues/*.json` 没有真实 top-level `projectLink`。
- `cargo run -p agentflow-cli -- search "Issue Project Link"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass，包含 `agentflow issue-link ISSUE-0025`。
- `cargo run -p agentflow-cli -- verify ISSUE-0026`：pass，9 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0026`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- update summary`：pass，26 issues / 24 runs / 24 updates / 1 saved view。首次与 `goal next` 并发执行时触发 SQLite lock/table exists，顺序重跑通过。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0026`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Project-aware GoalLoop v0 边界定义`。首次并发执行时触发 readonly database，顺序重跑通过。
- `git diff --check`：pass。

结论：

- Issue Project Link Writer v0 已实现默认 preview 和显式确认 writer。
- 当前 live repo 没有执行 `--write --yes`，所以没有迁移任何历史 issue。
- 显式写入行为由 tempdir 单元测试覆盖，避免在当前 repo 未授权写入现有 issue link。
- 下一候选为 `Project-aware GoalLoop v0 边界定义`。

## 2026-05-25 - Project-aware GoalLoop v0 Boundary

执行者：Codex

范围：

- 在 `ISSUE-0027` contract 下新增 `docs/specs/project-aware-goalloop-boundary.md`。
- 明确最小关系：`LocalWorkspace.activeProjectId`、`LocalProject.activeMilestoneId`、`Milestone.issueIds` / `nextIssueIntent`、`IssueContract.projectLink`、`GoalLoopSelection`。
- 定义后续 Project-aware GoalLoop 决策优先级：goal readiness、active issue、incomplete issue、active project / active milestone candidate、roadmap fallback、wait-human。
- 定义缺失 workspace seed、project seed、issue projectLink、active project next issue 时的 fallback。
- 明确当前阶段不改写 `goal_loop_decision`，不自动执行 plan / run / verify / review，不迁移历史 issue，不批量写 projectLink，不新增 Desktop 执行入口。
- 更新 README、ROADMAP、MVP Spec、Local Workspace Boundary、Issue Project Link Boundary、Local Pro Boundary、Construction Plan、readiness script 和 latest verification summary。
- 将下一候选推进为 `Project-aware GoalLoop v0 实现`。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，31 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0027，证明 WIP=1 / active issue 优先级仍有效。
- `cargo run -p agentflow-cli -- projects`：pass，LocalProjectModelSnapshot 只读，recommended command 指向 `agentflow verify ISSUE-0027`。
- `cargo run -p agentflow-cli -- issue-link ISSUE-0025`：pass，仍为 read-only preview。
- `cargo run -p agentflow-cli -- search "Project-aware GoalLoop"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass，包含 Project-aware GoalLoop boundary anchors。
- no live issue projectLink proof：pass，`.agentflow/issues/*.json` 没有真实 top-level `projectLink`。
- no `goal_loop_decision` implementation change proof：pass，本阶段只新增/更新边界文档和 readiness anchors，没有实现 Project-aware GoalLoop 决策代码。
- `cargo run -p agentflow-cli -- verify ISSUE-0027`：pass，10 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0027`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- update summary`：pass，27 issues / 25 runs / 25 updates / 1 saved view。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0027`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Project-aware GoalLoop v0 实现`。
- `git diff --check`：pass。

结论：

- Project-aware GoalLoop v0 已完成边界定义。
- WIP=1、active issue、incomplete issue 优先级已被明确锁定。
- 当前没有迁移历史 issue，没有批量写 projectLink，也没有自动执行任何 recommended command。
- 下一候选为 `Project-aware GoalLoop v0 实现`。

## 2026-05-25 - Project-aware GoalLoop v0 Implementation

执行者：Codex

范围：

- 在 `ISSUE-0028` contract 下实现 Project-aware GoalLoop v0。
- 更新 `goal_loop_decision`：readiness 未通过仍 `wait-human`；active issue 仍优先；incomplete issue 仍优先；只有无 active / incomplete issue 时才读取 active project / active milestone candidate。
- 新增只读 project candidate helper：读取默认 active project 或 workspace seed 指定的 active project，再读取 project seed 中 active milestone / project-level `nextIssueIntent`。
- 缺失 workspace seed、project seed、projectLink 或 `nextIssueIntent` 时回退 root roadmap candidate。
- 不写 workspace / project seed，不写 issue projectLink，不迁移历史 issue，不执行 recommended command。
- 增加 focused tests 覆盖 active issue 优先、incomplete issue 优先、project candidate、roadmap fallback 和只推荐不执行。
- 更新 README、ROADMAP、MVP Spec、Project-aware GoalLoop Boundary、Local Workspace Boundary、Local Pro Boundary、Architecture Decisions、readiness script 和 latest verification summary。
- 将下一候选推进为 `Desktop GoalLoop Trace v0 只读展示`。

验证：

- `cargo fmt --check`：pass。
- `cargo test project_aware_goal_loop`：pass，4 focused tests。
- `cargo test`：pass，35 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0028，证明 WIP=1 / active issue 优先级仍有效。
- `cargo run -p agentflow-cli -- projects`：pass，LocalProjectModelSnapshot 只读，recommended command 指向 `agentflow verify ISSUE-0028`。
- `cargo run -p agentflow-cli -- search "Project-aware GoalLoop"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass，包含 Project-aware GoalLoop implementation anchors 和 Desktop GoalLoop Trace next-candidate anchors。
- no live issue projectLink proof：pass，`.agentflow/issues/*.json` 没有真实 top-level `projectLink`。
- no live workspace/project seed proof：pass，当前 live repo 没有 `.agentflow/workspace.json`、`.agentflow/teams/` 或 `.agentflow/projects/`。
- `cargo run -p agentflow-cli -- verify ISSUE-0028`：pass，9 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0028`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- update summary`：pass，28 issues / 26 runs / 26 updates / 1 saved view。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0028`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Desktop GoalLoop Trace v0 只读展示`。
- final `bash checks/agentflow-readiness.sh`：pass。
- final no live issue projectLink proof：pass。
- final no live workspace/project seed proof：pass。
- `git diff --check`：pass。

结论：

- Project-aware GoalLoop v0 已实现本地只推荐决策。
- active issue / incomplete issue 仍优先于 project candidate。
- project candidate 缺失时回退 roadmap candidate。
- 当前没有迁移历史 issue，没有写 projectLink，也没有自动执行任何 recommended command。
- 下一候选为 `Desktop GoalLoop Trace v0 只读展示`。

## 2026-05-25 - Desktop GoalLoop Trace v0 Read-only View

执行者：Codex

范围：

- 在 `ISSUE-0029` contract 下实现 Desktop GoalLoop Trace v0 只读展示。
- Desktop Workbench 新增“决策”视图，复用现有 `DesktopWorkbenchSnapshot.goalLoop`、`.agentflow/goal-loop.json` 和 `.agentflow/updates/GOAL-LOOP-SUMMARY.md`。
- UI 展示 goal readiness、active issue、incomplete issue、project candidate、roadmap fallback、next action、recommended issue intent 和 recommended command。
- UI 明确展示 GoalLoop 决策优先级：readiness、active issue / WIP=1、incomplete issue、active project / active milestone candidate、roadmap fallback、wait-human。
- recommended command 只展示文本，不新增桌面执行按钮，不创建 issue，不执行 plan / run / verify / review，不调用模型，不写 `.agentflow/`。
- 更新 README、ROADMAP、MVP Spec、Project-aware GoalLoop Boundary、Desktop Workbench Boundary、Local Pro Boundary、Local Workspace Boundary、Construction Plan、readiness script 和 latest verification summary。
- 将下一候选推进为 `Desktop Issue Lifecycle Trace v0 只读展示`。

验证：

- `cargo run -p agentflow-cli -- plan "Desktop GoalLoop Trace v0 只读展示"`：pass，ISSUE-0029。
- `cargo run -p agentflow-cli -- run ISSUE-0029 --dry-run`：pass，RUN-0027。
- `cargo fmt --check`：pass。
- `cargo test`：pass，35 tests。
- `npm --prefix apps/desktop run build`：pass。
- browser smoke：pass，能看到“决策”入口、“目标循环决策追踪”、“决策优先级”、“推荐命令”和“这里没有执行按钮”。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0029，证明 WIP=1 / active issue 优先级仍有效。
- `cargo run -p agentflow-cli -- projects`：pass，LocalProjectModelSnapshot 只读，recommended command 指向 `agentflow verify ISSUE-0029`。
- `cargo run -p agentflow-cli -- search "Desktop GoalLoop Trace"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass，包含 Desktop GoalLoop Trace UI anchors。
- no live issue projectLink proof：pass，`.agentflow/issues/*.json` 没有真实 top-level `projectLink` / `teamId` / `projectId` / `milestoneId` / `linkSource`。
- no live workspace/project seed proof：pass，当前 live repo 没有 `.agentflow/workspace.json`、`.agentflow/teams/` 或 `.agentflow/projects/`。
- `cargo run -p agentflow-cli -- verify ISSUE-0029`：pass，9 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0029`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- update summary`：pass，29 issues / 27 runs / 27 updates / 1 saved view。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0029`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Desktop Issue Lifecycle Trace v0 只读展示`。
- final `bash checks/agentflow-readiness.sh`：pass。
- final no live issue projectLink proof：pass。
- final no live workspace/project seed proof：pass。
- final `cargo run -p agentflow-cli -- goal next`：pass，next action plan `Desktop Issue Lifecycle Trace v0 只读展示`。
- `git diff --check`：pass。

结论：

- Desktop GoalLoop Trace v0 已实现只读展示。
- UI 可以解释当前 next action 为什么被推荐。
- recommended command 只展示，不执行。
- 当前没有写 `.agentflow/` 事实源，没有迁移 issue，没有写 projectLink，也没有破坏 Project-aware GoalLoop 的 WIP=1 / fallback 优先级。
- 下一候选为 `Desktop Issue Lifecycle Trace v0 只读展示`。

## 2026-05-25 - Desktop Issue Lifecycle Trace v0 Read-only View

执行者：Codex

范围：

- 在 `ISSUE-0030` contract 下实现 Desktop Issue Lifecycle Trace v0 只读展示。
- 扩展 `DesktopWorkbenchSnapshot`，只读读取 `.agentflow/updates/PROJECT-UPDATE-*.md`，并向 Desktop 暴露 `projectUpdates` 计数和内容。
- Desktop Workbench 新增“生命周期”视图，复用 issues、runs、evidence、reviews、project updates 展示单个 issue 的本地链路。
- UI 展示 IssueContract 基本信息、scope、non-goals、latest run、validation command 状态、evidence / review / project update 链接和 lifecycle step 状态。
- 生命周期步骤固定为 contract、run、validation、evidence、review、project update、completed。
- evidence / review / update 只作为链接或文本展示，没有 run / verify / review 执行按钮，不创建或编辑 issue，不写 `.agentflow/`，不写 projectLink，不迁移历史 issue，不调用模型，不上传远程。
- 更新 README、ROADMAP、MVP Spec、Desktop Workbench Boundary、Local Pro Boundary、Construction Plan、readiness script 和 latest verification summary。
- 将下一候选推进为 `Desktop Project Update Timeline v0 只读展示`。

验证：

- `cargo run -p agentflow-cli -- plan "Desktop Issue Lifecycle Trace v0 只读展示"`：pass，ISSUE-0030。
- `cargo run -p agentflow-cli -- run ISSUE-0030 --dry-run`：pass，RUN-0028。
- `cargo fmt --check`：pass。
- `cargo test`：pass，35 tests。
- `npm --prefix apps/desktop run build`：pass。
- browser smoke：pass，能看到“生命周期”入口、“Issue 生命周期追踪”、“当前生命周期步骤”、Contract、Validation、Evidence、Project Update 和“没有执行按钮”。
- no execution controls：pass，生命周期视图没有 run / verify 执行按钮。
- no live issue projectLink proof：pass，30 个 `.agentflow/issues/*.json` 没有真实 top-level `projectLink` / `teamId` / `projectId` / `milestoneId` / `linkSource`。
- no live workspace/project seed proof：pass，当前 live repo 没有 `.agentflow/workspace.json`、`.agentflow/teams/` 或 `.agentflow/projects/`。
- no search/query write directory proof：pass，当前 live repo 没有 `.agentflow/search/` 或 `.agentflow/queries/`。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0030，证明 WIP=1 / active issue 优先级仍有效。
- `cargo run -p agentflow-cli -- projects`：pass，LocalProjectModelSnapshot 只读，recommended command 指向 `agentflow verify ISSUE-0030`。
- `cargo run -p agentflow-cli -- search "Issue Lifecycle Trace"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass，包含 Desktop Issue Lifecycle Trace UI anchors。
- `cargo run -p agentflow-cli -- verify ISSUE-0030`：pass，9 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0030`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- update summary`：pass，30 issues / 28 runs / 28 updates / 1 saved view。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0030`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Desktop Project Update Timeline v0 只读展示`。
- `git diff --check`：pass。

结论：

- Desktop Issue Lifecycle Trace v0 已实现只读展示。
- UI 可以解释一个 issue 当前卡在哪个生命周期步骤。
- evidence / review / update 只作为链接或文本展示。
- 当前没有桌面执行按钮，没有写 `.agentflow/` 事实源，没有迁移 issue，也没有写 projectLink。
- 下一候选为 `Desktop Project Update Timeline v0 只读展示`。

## 2026-05-25 - Desktop Project Update Timeline v0 Read-only View

执行者：Codex

范围：

- 在 `ISSUE-0031` contract 下实现 Desktop Project Update Timeline v0 只读展示。
- Desktop Workbench 新增“更新时间线”视图，复用 `DesktopWorkbenchSnapshot` 的 projectUpdates、issues、runs、evidence 和 reviews。
- UI 按最新优先展示 project update 列表、update id、path、title、snippet、关联 issue、关联 run、validation 状态、关联 evidence 和关联 review。
- UI 明确展示项目推进链路：issue contract -> run -> validation -> evidence -> review -> project update。
- project update 只作为链接或文本展示；没有 run / verify / review 执行按钮，不创建或编辑 issue，不保存 timeline filter，不写 `.agentflow/`，不写 projectLink，不迁移历史 issue，不调用模型，不上传远程。
- 更新 README、ROADMAP、MVP Spec、Desktop Workbench Boundary、Local Pro Boundary、Construction Plan、readiness script 和 latest verification summary。
- 将下一候选推进为 `Desktop Run Validation Trace v0 只读展示`。

验证：

- `cargo run -p agentflow-cli -- plan "Desktop Project Update Timeline v0 只读展示"`：pass，ISSUE-0031。
- `cargo run -p agentflow-cli -- run ISSUE-0031 --dry-run`：pass，RUN-0029。
- `cargo fmt --check`：pass。
- `cargo test`：pass，35 tests。
- `npm --prefix apps/desktop run build`：pass。
- browser smoke：pass，能看到“更新时间线”入口、“项目更新时间线”、项目推进链路、Issue Contract、Run、Validation、Evidence、Review 和“没有执行按钮”。
- no execution controls：pass，更新时间线视图没有 run / verify / review 执行按钮。
- no live issue projectLink proof：pass，31 个 `.agentflow/issues/*.json` 没有真实 top-level `projectLink` / `teamId` / `projectId` / `milestoneId` / `linkSource`。
- no live workspace/project seed proof：pass，当前 live repo 没有 `.agentflow/workspace.json`、`.agentflow/teams/` 或 `.agentflow/projects/`。
- no search/query write directory proof：pass，当前 live repo 没有 `.agentflow/search/` 或 `.agentflow/queries/`。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0031，证明 WIP=1 / active issue 优先级仍有效。
- `cargo run -p agentflow-cli -- projects`：pass，LocalProjectModelSnapshot 只读，recommended command 指向 `agentflow verify ISSUE-0031`。
- `cargo run -p agentflow-cli -- search "Project Update Timeline"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass，包含 Desktop Project Update Timeline UI anchors。
- `cargo run -p agentflow-cli -- verify ISSUE-0031`：pass，9 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0031`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- update summary`：pass，31 issues / 29 runs / 29 updates / 1 saved view。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0031`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Desktop Run Validation Trace v0 只读展示`。
- `git diff --check`：pass。

结论：

- Desktop Project Update Timeline v0 已实现只读展示。
- UI 可以解释项目更新来自哪个 issue / run / evidence / review。
- project update 只作为链接或文本展示。
- 当前没有桌面执行按钮，没有写 `.agentflow/` 事实源，没有保存 timeline filter，没有迁移 issue，也没有写 projectLink。
- 下一候选为 `Desktop Run Validation Trace v0 只读展示`。

## 2026-05-25 - Desktop MVP Navigation Scope Reduction v0

执行者：Codex

范围：

- 在 `ISSUE-0032` contract 下收敛 Desktop Workbench MVP 主导航。
- 主导航只保留总览、团队、项目、任务。
- 新增“团队”只读视图，展示 LocalWorkspace / LocalTeams、团队关联项目和团队任务。
- 项目视图继续展示 LocalProject / Milestone / GoalLoopSelection。
- 任务视图继续展示 IssueContract、latest run、validation commands 和证据链接。
- 保留 GoalLoop Trace、Issue Lifecycle Trace、Project Update Timeline、Search、Metrics 等底层只读能力，但不作为 MVP 主导航入口展示。
- 不新增 trace 视图，不删除本地事实源或 reader，不执行 run / verify / review，不写 `.agentflow/`，不写 projectLink，不创建 workspace/team/project seed。
- 更新 README、ROADMAP、MVP Spec、Desktop Workbench Boundary、Local Pro Boundary、Construction Plan、readiness script 和 latest verification summary。
- 将下一候选推进为 `Desktop MVP Task Detail v0 收敛`。

验证：

- `cargo run -p agentflow-cli -- plan "Desktop MVP Navigation Scope Reduction v0"`：pass，ISSUE-0032。
- `cargo run -p agentflow-cli -- run ISSUE-0032 --dry-run`：pass，RUN-0030。
- `npm --prefix apps/desktop run build`：pass。
- browser smoke：pass，主导航只剩“总览 / 团队 / 项目 / 任务”，决策、生命周期、更新时间线、指标、搜索、证据、审查、视图入口均未出现在主导航。
- browser smoke：pass，“团队”页能看到团队入口、项目和任务关系。
- `cargo fmt --check`：pass。
- `cargo test`：pass，35 tests。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0032。
- `cargo run -p agentflow-cli -- projects`：pass，LocalProjectModelSnapshot 只读，recommended command 指向 `agentflow verify ISSUE-0032`。
- `cargo run -p agentflow-cli -- search "Desktop MVP Navigation Scope Reduction"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass，包含 Desktop MVP Navigation anchors。
- no live issue projectLink proof：pass，32 个 `.agentflow/issues/*.json` 没有真实 top-level `projectLink` / `teamId` / `projectId` / `milestoneId` / `linkSource`。
- no live workspace/project seed proof：pass，当前 live repo 没有 `.agentflow/workspace.json`、`.agentflow/teams/` 或 `.agentflow/projects/`。
- no search/query write directory proof：pass，当前 live repo 没有 `.agentflow/search/` 或 `.agentflow/queries/`。
- `cargo run -p agentflow-cli -- verify ISSUE-0032`：pass，8 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0032`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- update summary`：pass，32 issues / 30 runs / 30 updates / 1 saved view。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0032`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Desktop MVP Task Detail v0 收敛`。
- `git diff --check`：pass。

结论：

- Desktop Workbench MVP 已从 trace 扩张状态收敛回最小产品骨架。
- 当前主导航只围绕总览、团队、项目、任务构建。
- 决策、生命周期、更新时间线等能力降级为内部只读 trace/debug 能力，不再作为 MVP 主入口。
- 下一候选为 `Desktop MVP Task Detail v0 收敛`。

## 2026-05-25 - Desktop Team Hierarchy v0 收敛

执行者：Codex

范围：

- 在 `ISSUE-0033` contract 下收敛 Desktop 团队入口。
- 团队页改为 workspace -> teams -> projects / tasks 的只读层级展示。
- 一个 workspace 可以包含多个团队；每个团队下面展示关联项目和任务。
- 总览页同步把 Projects / Teams 文案收敛为中文“项目 / 团队”。
- 保持主导航只包含总览、团队、项目、任务。
- 不新增 Desktop 新建/编辑/删除 team / project / task。
- 不写 `.agentflow/workspace.json`、`.agentflow/teams/`、`.agentflow/projects/` 或 issue projectLink。

验证：

- `cargo run -p agentflow-cli -- plan "Desktop Team Hierarchy v0 收敛"`：pass，ISSUE-0033。
- `cargo run -p agentflow-cli -- run ISSUE-0033 --dry-run`：pass，RUN-0031。
- `npm --prefix apps/desktop run build`：pass。
- browser smoke：pass，“团队”页能看到 Your teams、团队入口、项目、任务和多团队说明；主导航未恢复决策、生命周期、更新时间线。
- `cargo fmt --check`：pass。
- `cargo test`：pass，35 tests。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0033。
- `cargo run -p agentflow-cli -- projects`：pass，LocalProjectModelSnapshot 只读。
- `cargo run -p agentflow-cli -- search "Team Hierarchy"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass，包含 Desktop Team Hierarchy anchors。
- no live workspace/team/project seed proof：pass，当前 live repo 没有 `.agentflow/workspace.json`、`.agentflow/teams/` 或 `.agentflow/projects/`。
- no issue projectLink proof：pass，33 个 `.agentflow/issues/*.json` 没有真实 top-level `projectLink` / `teamId` / `projectId` / `milestoneId` / `linkSource`。
- `cargo run -p agentflow-cli -- verify ISSUE-0033`：pass，9 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0033`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- update summary`：pass，33 issues / 31 runs / 31 updates / 1 saved view。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0033`：pass，15 checks / ready。第一次与 `update summary` 并发时触发 SQLite readonly error，串行重跑通过。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Desktop MVP Task Detail v0 收敛`。

结论：

- Desktop MVP 的团队入口已对齐“多个团队，每个团队下有项目和任务”的产品结构。
- 当前实现仍是只读展示，不提供创建按钮，不写 `.agentflow/` 组织模型 seed。
- 下一候选为 `Desktop MVP Task Detail v0 收敛`。

## 2026-05-25 - Desktop Team Parent Child Columns v0

执行者：Codex

范围：

- 在 `ISSUE-0034` contract 下收敛 Desktop 团队页栏目。
- 将团队页改为三栏父子关系：团队是父栏目，项目和任务是选中团队下面的子栏目。
- 团队栏目负责选择父级 team；项目栏目只展示该 team 关联 projects；任务栏目只展示该 team 关联 issue refs。
- 保持 Desktop 主导航只包含总览、团队、项目、任务。
- 不新增 Desktop 新建/编辑/删除 team / project / task。
- 不写 `.agentflow/workspace.json`、`.agentflow/teams/`、`.agentflow/projects/` 或 issue projectLink。

验证：

- `cargo run -p agentflow-cli -- plan "Desktop Team Parent Child Columns v0"`：pass，ISSUE-0034。
- `cargo run -p agentflow-cli -- run ISSUE-0034 --dry-run`：pass，RUN-0032。
- `npm --prefix apps/desktop run build`：pass。
- browser smoke：pass，团队页显示 3 个栏目，包含父级栏目、子级栏目、团队、项目、任务；主导航未恢复决策、生命周期、更新时间线。
- `cargo fmt --check`：pass。
- `cargo test`：pass，35 tests。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0034。
- `cargo run -p agentflow-cli -- projects`：pass，LocalProjectModelSnapshot 只读。
- `cargo run -p agentflow-cli -- search "Parent Child Columns"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass，包含 Desktop Team Parent Child Columns anchors。
- no live workspace/team/project seed proof：pass，当前 live repo 没有 `.agentflow/workspace.json`、`.agentflow/teams/` 或 `.agentflow/projects/`。
- no issue projectLink proof：pass，34 个 `.agentflow/issues/*.json` 没有真实 top-level `projectLink` / `teamId` / `projectId` / `milestoneId` / `linkSource`。
- `cargo run -p agentflow-cli -- verify ISSUE-0034`：pass，9 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0034`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- update summary`：pass，34 issues / 32 runs / 32 updates / 1 saved view。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0034`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Desktop MVP Task Detail v0 收敛`。

结论：

- 团队页已在栏目结构上表达父子关系。
- 团队是父级；项目和任务是团队下面的子级。
- 当前仍是只读 UI，不提供创建按钮，不写 `.agentflow/` 组织模型 seed。
- 下一候选为 `Desktop MVP Task Detail v0 收敛`。

## 2026-05-26 - Desktop Workspace Sidebar Tree v0

执行者：Codex

范围：

- 在 `ISSUE-0035` contract 下收敛 Desktop 左侧栏目。
- 左侧栏目改为本地工作区树：
  - Workspace
    - project
    - issues
  - Team
    - project
    - issues
- Workspace 节点进入总览；workspace project / issues 子项进入项目 / 任务。
- Team 节点进入团队页；team project / issues 子项进入项目 / 任务。
- 所有入口只切换视图，不执行命令，不写 `.agentflow/`。
- 不新增 Desktop 新建/编辑/删除 workspace / team / project / issue。

验证：

- `cargo run -p agentflow-cli -- plan "Desktop Workspace Sidebar Tree v0"`：pass，ISSUE-0035。
- `cargo run -p agentflow-cli -- run ISSUE-0035 --dry-run`：pass，RUN-0033。
- `npm --prefix apps/desktop run build`：pass。
- browser smoke：pass，左侧栏目显示 Workspace / Teams 父节点和 project / issues 子项；未恢复决策、生命周期、更新时间线。
- `cargo fmt --check`：pass。
- `cargo test`：pass，35 tests。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0035。
- `cargo run -p agentflow-cli -- projects`：pass，LocalProjectModelSnapshot 只读。
- `cargo run -p agentflow-cli -- search "Workspace Sidebar Tree"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass，包含 Desktop Workspace Sidebar Tree anchors。
- no live workspace/team/project seed proof：pass，当前 live repo 没有 `.agentflow/workspace.json`、`.agentflow/teams/` 或 `.agentflow/projects/`。
- no issue projectLink proof：pass，35 个 `.agentflow/issues/*.json` 没有真实 top-level `projectLink` / `teamId` / `projectId` / `milestoneId` / `linkSource`。
- `cargo run -p agentflow-cli -- verify ISSUE-0035`：pass，9 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0035`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- update summary`：pass，35 issues / 33 runs / 33 updates / 1 saved view。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0035`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Desktop MVP Task Detail v0 收敛`。

结论：

- Desktop 左侧栏目已按用户指定的 workspace/team 父子结构呈现。
- Workspace 和 Team 都是父节点；project 和 issues 是各自下面的子项。
- 当前仍是只读导航，不提供创建按钮，不写 `.agentflow/` 组织模型 seed。
- 下一候选为 `Desktop MVP Task Detail v0 收敛`。

## 2026-05-26 - Desktop Teams Add Button v0

执行者：Codex

范围：

- 在 `ISSUE-0036` contract 下给 Desktop 左侧工作区树的 TEAMS 标题增加 `+` 入口。
- `+` 点击后切换到团队页并显示“新增团队 / 初始化创建入口”面板。
- 当前入口只展示创建入口和后续写入边界，不真正保存 team。
- 不写 `.agentflow/workspace.json`、`.agentflow/teams/`、`.agentflow/projects/` 或 issue projectLink。

验证：

- `cargo run -p agentflow-cli -- plan "Desktop Teams Add Button v0"`：pass，ISSUE-0036。
- `cargo run -p agentflow-cli -- run ISSUE-0036 --dry-run`：pass，RUN-0034。
- `npm --prefix apps/desktop run build`：pass。
- browser smoke：pass，TEAMS 有且只有一个 `+` 按钮；点击后显示“新增团队 / 初始化创建入口”。
- `cargo fmt --check`：pass。
- `cargo test`：pass，35 tests。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next` active issue：pass，next action verify ISSUE-0036。
- `cargo run -p agentflow-cli -- search "Teams Add Button"`：pass，返回 `.agentflow/` 内可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass，包含 Desktop Teams Add Button anchors。
- no live workspace/team/project seed proof：pass，当前 live repo 没有 `.agentflow/workspace.json`、`.agentflow/teams/` 或 `.agentflow/projects/`。
- no issue projectLink proof：pass，36 个 `.agentflow/issues/*.json` 没有真实 top-level `projectLink` / `teamId` / `projectId` / `milestoneId` / `linkSource`。
- `cargo run -p agentflow-cli -- verify ISSUE-0036`：pass，8 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0036`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- update summary`：pass，36 issues / 34 runs / 34 updates / 1 saved view。
- `cargo run -p agentflow-cli -- review-assistant ISSUE-0036`：pass，15 checks / ready。
- `cargo run -p agentflow-cli -- goal next` after review：pass，next action plan `Desktop MVP Task Detail v0 收敛`。

结论：

- TEAMS 右侧已新增 `+` 入口。
- 点击 `+` 可以进入新增团队面板。
- 当前仍不保存 team，不写 `.agentflow/teams/`；真正创建能力后置到 Team Writer。
- 下一候选为 `Desktop MVP Task Detail v0 收敛`。

## 2026-05-26 - MVP Productization Project / Milestone-aware Issue Planning v0

执行者：Codex

范围：

- 参考 Linear 的 workspace / team / project / milestone / issue 最小关系，把 AgentFlow 当前工作收敛为本地 MVP project。
- 创建并使用 `.agentflow/workspace.json`、`.agentflow/teams/core.json`、`.agentflow/projects/agentflow-local-execution.json` 作为本地 project seed。
- 将 project 拆成 `mvp-project-foundation`、`mvp-issue-planning`、`mvp-execution-loop`、`mvp-desktop-polish`、`mvp-release-readiness`。
- 完成 `ISSUE-0037 Project Seed Fact Source v0 实现` 和 `ISSUE-0038 Milestone-aware Issue Planning v0 实现`。
- `agentflow plan` 现在会在 seed 存在时自动写入 issue `projectLink`，并同步更新 active team / project / milestone 的 `issueIds`。
- 基于 active milestone `mvp-execution-loop` 创建 `ISSUE-0039 MVP Execution Loop v0 收敛`。

验证：

- `cargo run -p agentflow-cli -- project-seed --write --yes`：pass，创建 workspace/team/project seed。
- `cargo run -p agentflow-cli -- plan "Project Seed Fact Source v0 实现"`：pass，ISSUE-0037。
- `cargo run -p agentflow-cli -- issue-link ISSUE-0037 --write --yes`：pass，关联 `mvp-project-foundation`。
- `cargo test -p agentflow-core local_project_model_snapshot_prefers_seed_files_when_present -- --nocapture`：pass。
- `cargo run -p agentflow-cli -- run ISSUE-0037 --dry-run`：pass，RUN-0035。
- `cargo run -p agentflow-cli -- verify ISSUE-0037`：pass。
- `cargo run -p agentflow-cli -- review ISSUE-0037`：pass，生成 evidence / review / update。
- `cargo test -p agentflow-core plan_issue_links_active_project_milestone_when_seed_exists -- --nocapture`：pass。
- `cargo test -p agentflow-core issue_project_link_write_updates_only_target_issue -- --nocapture`：pass。
- `cargo run -p agentflow-cli -- run ISSUE-0038 --dry-run`：pass，RUN-0036。
- `cargo run -p agentflow-cli -- verify ISSUE-0038`：pass，5 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0038`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- projects`：pass，active milestone 进入 `mvp-execution-loop`。
- `cargo run -p agentflow-cli -- plan "MVP Execution Loop v0 收敛"`：pass，ISSUE-0039 自动关联 `mvp-execution-loop`。
- `cargo run -p agentflow-cli -- goal next`：pass，next action run ISSUE-0039。
- `cargo run -p agentflow-cli -- update summary`：pass，39 issues / 38 completed / 36 runs / 36 updates。
- `cargo fmt --check`：pass。
- `cargo test`：pass，37 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass。
- `cargo run -p agentflow-cli -- projects`：pass，active milestone `mvp-execution-loop`，issue `ISSUE-0039`。
- `bash checks/agentflow-readiness.sh`：pass，seed-backed MVP checks。
- `git diff --check`：pass。

结论：

- AgentFlow 现在具备 MVP 产品化的最小项目骨架：workspace -> team -> project -> milestones -> issue contracts。
- 后续 issue 可以从 active milestone 自动生成并归属，不再需要手工补 `issue-link` 和 seed `issueIds`。
- 当前下一步是执行 `ISSUE-0039`：`agentflow run ISSUE-0039 --dry-run`。

## 2026-05-26 - MVP Execution Loop v0

执行者：Codex

范围：

- 在 `ISSUE-0039` contract 下把 MVP 执行链路固定为 `Project -> Milestone -> Issue -> ExecutionRun -> VerificationEvidence -> ReviewEvidence -> ProjectUpdate`。
- `LocalProjectIssueRef` 增加 latest run、run status、validation status、execution state、evidence path、review path 和 project update path。
- `agentflow projects` 在每个 milestone 下展示 issue execution trace。
- Desktop Project 视图在 milestone 下只读展示 issue 的执行状态、运行、验证、证据和项目更新。
- Milestone 仍然只做阶段归属和证据收口，不参与复杂调度；调度继续由 IssueContract、WIP=1 和 `goal next` 决定。

验证：

- `cargo run -p agentflow-cli -- run ISSUE-0039 --dry-run`：pass，RUN-0037。
- `cargo fmt --check`：pass。
- `cargo test -p agentflow-core local_project_model_snapshot_reads_current_facts_without_writing -- --nocapture`：pass。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- projects`：pass，milestone 下展示 issue execution trace。
- `cargo run -p agentflow-cli -- verify ISSUE-0039`：pass，6 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0039`：pass，生成 evidence / review / update。
- `cargo run -p agentflow-cli -- goal next`：pass，推荐 `Desktop MVP Productization v0 收敛`。
- `cargo run -p agentflow-cli -- plan "Desktop MVP Productization v0 收敛"`：pass，ISSUE-0040 自动关联 `mvp-desktop-polish`。
- `cargo run -p agentflow-cli -- update summary`：pass，40 issues / 39 completed / 37 runs / 37 updates。
- `cargo test`：pass，37 tests。
- `cargo run -p agentflow-cli -- goal check`：pass。
- `cargo run -p agentflow-cli -- goal next` after ISSUE-0040：pass，next action run ISSUE-0040。
- `bash checks/agentflow-readiness.sh`：pass。
- `git diff --check`：pass。

结论：

- MVP 的执行链路已从扁平 issue queue 收敛为阶段化链路。
- 当前 active milestone 已推进到 `mvp-desktop-polish`。
- 当前下一步是执行 `ISSUE-0040`：`agentflow run ISSUE-0040 --dry-run`。

## 2026-05-26 - MVP Minimal Workflow v0 / Desktop MVP Productization v0

执行者：Codex

范围：

- 在 `ISSUE-0040` contract 下把 MVP 最小产品链路锁定为 `Project -> Milestone -> Issue -> ExecutionRun -> VerificationEvidence -> ReviewEvidence -> ProjectUpdate -> Milestone Evidence Summary`。
- `goal_loop_decision` 增加 active milestone queue preflight：有 active milestone issue 队列时，只推进当前 milestone 下唯一 eligible issue；同一 milestone 多个未完成 issue 时返回 `wait-human`。
- `review_issue` 在 linked milestone 全部 issue completed 后自动写 `.agentflow/evidence/MILESTONE-<milestone-id>-evidence-summary.md`，并把 project seed 推进到下一个 planned milestone。
- Desktop Project 视图将选择区文案收敛为“队列预检”，项目层级文案收敛为“项目 / 里程碑”和“任务合同”。
- `ISSUE-0040` 已完成，`mvp-desktop-polish` 已完成，当前 active milestone 已推进到 `mvp-release-readiness`。

验证：

- `cargo test goal_next_uses_active_milestone_queue_before_outside_backlog`：pass，active milestone queue preflight 优先于外部未完成 backlog。
- `cargo test review_completes_milestone_and_writes_summary_when_seed_linked`：pass，milestone 完成后写 summary 并激活下一 milestone。
- `cargo fmt --check`：pass。
- `cargo test`：pass，39 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- verify ISSUE-0040`：pass，RUN-0038 / 6 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0040`：pass，生成 evidence / review / project update。
- `test -f .agentflow/evidence/MILESTONE-mvp-desktop-polish-evidence-summary.md`：pass。
- `cargo run -p agentflow-cli -- update summary`：pass，40 issues / 40 completed / 38 runs / 38 updates。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next`：pass，推荐 `agentflow plan "MVP Release Readiness v0 验收"`。
- `cargo run -p agentflow-cli -- projects`：pass，active milestone `mvp-release-readiness`，`mvp-desktop-polish` completed。
- `cargo run -p agentflow-cli -- metrics`：pass，latest evidence 指向 milestone summary。
- `cargo run -p agentflow-cli -- search "mvp-desktop-polish"`：pass，返回 milestone summary、ISSUE-0040 和 RUN-0038 结果。
- `bash checks/agentflow-readiness.sh`：pass。
- `git diff --check`：pass。

结论：

- AgentFlow MVP 的主链路已从“扁平 issue queue”收敛为 `Project -> Milestone -> Issue -> Evidence`。
- `goal next` 当前不会越过 active milestone 直接处理全局 backlog。
- 当前下一步是进入发布验收：`agentflow plan "MVP Release Readiness v0 验收"`。

## 2026-05-26 - AgentFlow AI Delivery Workflow Contract v1

执行者：Codex

范围：

- 在 `ISSUE-0041` contract 下新增 `docs/specs/agentflow-ai-delivery-workflow-contract-v1.md`，并将 PRD 正式合同落到 `docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md`。
- 将 AgentFlow MVP 主干定义为 `Workspace / Team -> Project -> Milestone -> Issue -> Lease -> Execution Run -> PR / Checks -> Evidence -> Milestone Review -> Project Audit / Docs Refresh`。
- 明确五个能力面：Workflow State Machine、Eligibility Engine、Lease / Lock、Execution Evidence、Milestone / Project Closure。
- 固定 PRD / ARC / AIE 顺序：`@003 / PRD` 先输出合同，`@005 / ARC` 审查状态机、eligible、lease、evidence 和数据模型，`@000 / AIE` 再落仓执行。
- 当前阶段只定义产品 / 架构合同，不实现 UI、不实现 lease writer、不接入远程 PR provider、不改写 `goal_loop_decision`。

验证：

- `cargo run -p agentflow-cli -- plan "AgentFlow AI Delivery Workflow Contract v1 边界定义"`：pass，ISSUE-0041 自动关联 `mvp-release-readiness`。
- `cargo run -p agentflow-cli -- run ISSUE-0041 --dry-run`：pass，RUN-0039。
- `cargo fmt --check`：pass。
- `cargo test`：pass，39 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass。
- `cargo run -p agentflow-cli -- goal next`：pass，review 后推荐 `agentflow plan "Workflow State Machine v0 边界定义"`。
- `cargo run -p agentflow-cli -- projects`：pass。
- `cargo run -p agentflow-cli -- search "AI Delivery Workflow Contract"`：pass。
- `test -f docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md`：pass。
- `cargo run -p agentflow-cli -- verify ISSUE-0041`：pass，RUN-0039 / 8 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0041`：pass，生成 evidence / review / project update。
- `test -f .agentflow/evidence/MILESTONE-mvp-release-readiness-evidence-summary.md`：pass。
- `cargo run -p agentflow-cli -- update summary`：pass，41 issues / 41 completed / 39 runs / 39 updates。
- `bash checks/agentflow-readiness.sh`：pass。
- `git diff --check`：pass。

结论：

- `AgentFlow AI Delivery Workflow Contract v1` 已完成并通过 review，`mvp-release-readiness` milestone 已生成 evidence summary。
- AgentFlow 当前 MVP 主干已从 Linear-like issue runner 收敛为受控 AI coding agent 交付系统。
- 当前下一步是 `agentflow plan "Workflow State Machine v0 边界定义"`。

## 2026-05-26 - AgentFlow AI Delivery Workflow Contract v1 可执行化

执行者：Codex

范围：

- 将 `docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md` 从 PRD 草案升级为可直接供 ARC / AIE 使用的 canonical 合同。
- 补齐 Core Entity Model 的必填字段、状态枚举、状态迁移守卫、禁止迁移、Eligibility failure reason vocabulary、Lease race conditions、Command Boundary、Local File Mapping、Event Log Contract、Minimal Data Model、First Executable Vertical Slice 和 Verification Matrix。
- 保持当前阶段为文档合同工作：不实现 UI、不实现状态机代码、不实现 lease writer、不接入远程 PR provider、不改写 `goal_loop_decision`。

验证：

- `test -f docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md`：pass。
- `rg -n "Command Boundary|Local File Mapping|Event Log Contract|Verification Matrix" docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md`：pass。
- `cargo fmt --check`：pass。
- `cargo test`：pass，39 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass。
- `cargo run -p agentflow-cli -- goal next`：pass，仍推荐 `agentflow plan "Workflow State Machine v0 边界定义"`。
- `bash checks/agentflow-readiness.sh`：pass。
- `git diff --check`：pass。

结论：

- `AgentFlow AI Delivery Workflow Contract v1` 现在是可执行版本，可以作为 `Workflow State Machine v0 边界定义` 的直接输入。
- 下一步仍然是 `agentflow plan "Workflow State Machine v0 边界定义"`。

## 2026-05-26 - Workflow State Machine v0 可用切片

执行者：Codex

范围：

- 在 `ISSUE-0042` contract 下实现 Workflow State Machine v0。
- 新增 `agentflow state check`，读取 `.agentflow/projects/*.json`、`.agentflow/issues/*.json`、evidence / review 本地事实源。
- 新增 `WorkflowStateSnapshot`、`WorkflowStateCounts`、`WorkflowStateCheck`、`WorkflowTransitionGuard`、`WorkflowStateCheckSummary` 和 `write_workflow_state_check`。
- 写出 `.agentflow/state/workflow-state.json` 和 `.agentflow/updates/WORKFLOW-STATE-SUMMARY.md`。
- 增加 `docs/specs/workflow-state-machine-v0.md`，并更新 README、ROADMAP、MVP Spec、construction plan、latest verification summary 和 readiness script。
- 将 project seed 推进到 `workflow-core-eligibility-engine` active milestone，下一候选为 `Eligibility Engine v0 边界定义`。
- 当前阶段不新增 UI、不实现 lease writer、不调用模型、不创建远程 issue / PR、不改写 `goal_loop_decision`。

验证：

- `cargo run -p agentflow-cli -- plan "Workflow State Machine v0 边界定义"`：pass，ISSUE-0042 自动关联 `workflow-core-state-machine`。
- `cargo test workflow_state`：pass，2 个 workflow state focused tests。
- `cargo run -p agentflow-cli -- state check`：pass，ready true，1 project / 8 milestones / 42 issues / 0 errors / 36 warnings。
- `cargo run -p agentflow-cli -- run ISSUE-0042 --dry-run`：pass，RUN-0040。
- `cargo run -p agentflow-cli -- verify ISSUE-0042`：pass，RUN-0040 / 2 commands。
- `cargo run -p agentflow-cli -- review ISSUE-0042`：pass，生成 evidence / review / project update。
- `test -f .agentflow/evidence/MILESTONE-workflow-core-state-machine-evidence-summary.md`：pass。
- `cargo fmt --check`：pass。
- `cargo test`：pass，41 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- update summary`：pass，42 issues / 42 completed / 40 runs / 40 updates。
- `cargo run -p agentflow-cli -- goal check`：pass。
- `cargo run -p agentflow-cli -- goal next`：pass，推荐 `agentflow plan "Eligibility Engine v0 边界定义"`。
- `cargo run -p agentflow-cli -- projects`：pass，active milestone `workflow-core-eligibility-engine`。
- `cargo run -p agentflow-cli -- metrics`：pass，latest evidence 指向 `MILESTONE-workflow-core-state-machine-evidence-summary.md`。
- `cargo run -p agentflow-cli -- search "Workflow State"`：pass。
- `cargo run -p agentflow-cli -- search "Eligibility Engine"`：pass。
- `bash checks/agentflow-readiness.sh`：pass。
- `git diff --check`：pass。

结论：

- Workflow State Machine v0 已从合同变成可运行的本地状态检查能力。
- AgentFlow 当前已经具备 Project / Milestone / Issue 状态不变量检查、transition guard 输出和 readiness 硬门槛。
- 历史 `ISSUE-0001` 到 `ISSUE-0036` 缺少 projectLink 会以 warning 呈现，不阻断当前 active milestone；后续如需清理，应单独走 issue 迁移合同。
- 下一步进入 `agentflow plan "Eligibility Engine v0 边界定义"`。

## 2026-05-26 - Workflow Control Core v0 完整可用闭环

执行者：Codex

范围：

- 按 goal 级别直接推进，不新增单个 issue 作为本轮授权载体。
- 在 `Workflow State Machine v0` 基础上实现 Workflow Control Core v0。
- 新增 Eligibility Engine v0：`WorkflowEligibilitySnapshot`、`WorkflowEligibilityCandidate`、`WorkflowEligibilitySummary`、`agentflow eligibility`。
- 新增 Lease / Lock v0：`WorkflowLeaseRecord`、`WorkflowLeaseSnapshot`、`agentflow lease`、`.agentflow/leases/LEASE-*.json`。
- 改造 `agentflow run`：run 前必须通过 eligibility，并自动 acquire local lease；`AgentRun` 记录 `projectId`、`milestoneId`、`leaseId`。
- 改造 `agentflow review`：evidence / review / project update 完成后 release lease，并继续在 milestone 完成时生成 evidence summary。
- 更新 `goal next`：active milestone queue 推荐 run 前先看 eligibility；无 eligible issue 时推荐 `agentflow eligibility` 或 active milestone 的下一条 plan。
- 更新 README、ROADMAP、MVP Spec、Workflow State Machine Spec、AI Delivery Workflow Contract、construction plan、latest verification summary 和 readiness script。
- 当前不新增 Desktop UI，不接入远程 PR / GitHub / Linear，不调用模型，不做 SaaS、账号、支付、云同步。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，44 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- state check`：pass，ready true，1 project / 9 milestones / 42 issues / 0 errors / 36 warnings。
- `cargo run -p agentflow-cli -- eligibility`：pass，当前 active milestone 没有 open issue，推荐 `agentflow plan "Project Audit / Docs Refresh v0 边界定义"`。
- `cargo run -p agentflow-cli -- lease`：pass，0 active leases / 0 stale leases。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next`：pass，推荐 `agentflow plan "Project Audit / Docs Refresh v0 边界定义"`。
- `cargo run -p agentflow-cli -- projects`：pass，active milestone `workflow-core-closure-gates`。
- `cargo run -p agentflow-cli -- metrics`：pass，42 issues / 40 runs，下一步仍为 `Project Audit / Docs Refresh v0 边界定义`。
- `cargo run -p agentflow-cli -- search "Workflow Control Core"`：pass，返回可追溯 roadmap 结果。
- `cargo run -p agentflow-cli -- search "Project Audit"`：pass，返回 next-stage、goal loop 和 eligibility summary 结果。
- `bash checks/agentflow-readiness.sh`：pass。
- `git diff --check`：pass。
- `cargo test` focused coverage includes：
  - `eligibility_finds_unique_active_milestone_issue`
  - `eligibility_reports_failure_reasons_when_issue_is_not_ready`
  - `run_acquires_lease_and_review_releases_it`

结论：

- AgentFlow 现在具备本地 workflow control core：state -> eligibility -> lease -> run -> verify -> review -> evidence -> milestone summary -> next milestone。
- Eligible 是计算结果，不能通过手动 issue 状态绕过。
- run 前必须通过 eligibility + lease。
- review 后释放 lease，避免同一 project code-changing WIP 泄漏。
- 当前 active milestone 已切到 `workflow-core-closure-gates`，下一步是 `agentflow plan "Project Audit / Docs Refresh v0 边界定义"`。

## 2026-05-26 - Project Audit / Docs Refresh v0 边界定义

执行者：Codex

范围：

- 新增 `docs/specs/project-audit-docs-refresh-boundary.md`。
- 明确 Project closure 不能从 `active` 直接进入 `done`，必须经过 `audit -> docs-refresh -> final-review -> done`。
- 定义 Code Audit 检查范围：duplicate code、temporary code、unused code、TODO / FIXME、security / auth / permission risk、performance risk、architecture drift、test gaps、unexpected public API changes。
- 定义 Root Docs Refresh 检查范围：README、ROADMAP、MVP Spec、architecture docs、contracts、validation docs、runbook / known limitations。
- 定义 Final Evidence Summary 必须包含 project goal、completed milestones、completed issues、runs / validations、evidence / reviews、known gaps、deferred work 和 final recommendation。
- 明确 Human Final Approval 是 Project Done 前最后 gate，Agent 不能替用户批准。
- 更新 README、ROADMAP、MVP Spec、Workflow Control Core Spec、AI Delivery Workflow Contract、construction plan、latest verification summary 和 readiness script。
- 将 active milestone `workflow-core-closure-gates` 的下一候选推进为 `Project Closure State v0 实现`。
- 当前阶段只定义边界，不实现自动审计器，不修改 Desktop UI，不创建 `.agentflow/audits/`，不接入远程 PR / GitHub / Linear，不调用模型。

验证：

- `test -f docs/specs/project-audit-docs-refresh-boundary.md`：pass。
- `rg "Project Audit / Docs Refresh v0" docs/specs/project-audit-docs-refresh-boundary.md README.md ROADMAP.md docs/specs/mvp-spec.md docs/specs/workflow-control-core-v0.md docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md`：pass。
- `cargo fmt --check`：pass。
- `cargo test`：pass，44 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- state check`：pass，ready true，1 project / 9 milestones / 42 issues / 0 errors / 36 warnings。
- `cargo run -p agentflow-cli -- eligibility`：pass，推荐 `agentflow plan "Project Closure State v0 实现"`。
- `cargo run -p agentflow-cli -- lease`：pass，0 active leases / 0 stale leases。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next`：pass，推荐 `agentflow plan "Project Closure State v0 实现"`。
- `cargo run -p agentflow-cli -- projects`：pass，active milestone `workflow-core-closure-gates`。
- `cargo run -p agentflow-cli -- metrics`：pass，下一步为 `Project Closure State v0 实现`。
- `cargo run -p agentflow-cli -- search "Project Audit"`：pass，返回 `.agentflow/` 可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass。
- `test ! -d .agentflow/audits`：pass。
- `git diff --check`：pass。

结论：

- Project closure 边界已清楚：Code Audit、Root Docs Refresh、Final Evidence Summary 和 Human Final Approval 都是 Project Done 前的 gate。
- Project 不能绕过 audit / docs refresh 直接 done。
- 本阶段没有实现自动审计器，没有新增 Desktop UI，也没有创建 `.agentflow/audits/`。
- 当前下一步是 `agentflow plan "Project Closure State v0 实现"`。

## 2026-05-26 - Project Closure State v0 实现

执行者：Codex

范围：

- 新增 `ProjectClosureStateSnapshot`、`ProjectClosureCounts`、`ProjectClosureGate` 和 `ProjectClosureStateSummary`。
- 新增 `agentflow project closure` CLI。
- `agentflow project closure` 读取 `.agentflow/projects/{project-id}.json`、milestones、issues、runs、evidence、reviews、updates。
- 输出 `.agentflow/state/project-closure.json` 和 `.agentflow/updates/PROJECT-CLOSURE-SUMMARY.md`。
- 判断 Project closure state：`active`、`audit-ready`、`audit`、`docs-refresh`、`final-review`、`done-blocked`、`done`。
- 明确 Project done 前必须满足：all milestones completed、milestone evidence summaries exist、code audit exists、docs refresh exists、final evidence summary exists、human final approval exists。
- 更新 `goal next`：当无 active issue 且进入 closure state 时，推荐 `agentflow project closure`，不自动执行 audit / docs refresh / final approval。
- 更新 README、ROADMAP、MVP Spec、Project Audit / Docs Refresh Boundary、Workflow Control Core Spec、AI Delivery Workflow Contract、construction plan、latest verification summary 和 readiness script。
- 当前阶段不实现自动审计器、不执行 docs refresh、不自动标记 Project done、不创建 `.agentflow/audits/`、不调用模型、不接入远程 PR / GitHub / Linear、不修改 Desktop UI。

验证：

- `cargo test project_closure -- --nocapture`：pass，2 focused tests。
- `cargo fmt --check`：pass。
- `cargo test`：pass，46 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- state check`：pass，ready true，1 project / 9 milestones / 42 issues / 0 errors / 36 warnings。
- `cargo run -p agentflow-cli -- eligibility`：pass，推荐 `agentflow plan "Project Code Audit Snapshot v0 只读实现"`。
- `cargo run -p agentflow-cli -- lease`：pass，0 active leases / 0 stale leases。
- `cargo run -p agentflow-cli -- project closure`：pass，closure state `audit-ready`，`can_mark_done=false`，写出 project closure state 和 summary。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next`：pass，next action `project-closure`，推荐 `agentflow project closure`。
- `cargo run -p agentflow-cli -- projects`：pass，recommended command `agentflow project closure`。
- `cargo run -p agentflow-cli -- metrics`：pass，next action `project-closure`。
- `cargo run -p agentflow-cli -- search "Project Closure"`：pass，返回 `.agentflow/` 可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass。
- `test ! -d .agentflow/audits`：pass。
- `git diff --check`：pass。

结论：

- Project closure state 已能明确说明当前 Project 为什么不能 done。
- 当前 Project 进入 `audit-ready`，但 `can_mark_done=false`。
- `goal next` 推荐 `agentflow project closure`，不会自动执行 audit、docs refresh 或 final approval。
- 本阶段没有创建 `.agentflow/audits/`，没有实现自动审计器，没有修改 Desktop UI。
- 下一候选实现切片是 `Project Code Audit Snapshot v0 只读实现`。

## 2026-05-26 - Project Code Audit Snapshot v0 只读实现

执行者：Codex

范围：

- 新增 `ProjectCodeAuditSnapshot`、`ProjectCodeAuditCounts`、`ProjectCodeAuditCheck`、`ProjectCodeAuditFinding` 和 `ProjectCodeAuditSummary`。
- 新增 `agentflow project code-audit` CLI。
- `agentflow project code-audit` 读取 `.agentflow/state/project-closure.json`、project / milestone / issue / run / evidence / review / update 事实源和源码树。
- 输出 `.agentflow/state/project-code-audit.json` 和 `.agentflow/updates/PROJECT-CODE-AUDIT-SUMMARY.md`。
- 只读汇总 duplicate code、temporary code、unused / dead code、TODO / FIXME、security / auth / permission risk、performance risk、architecture drift、test gap 和 unexpected public API change 候选项。
- `agentflow project closure` 在 snapshot 存在时把 code audit gate 显示为 `snapshot-ready`，但仍不视为 final Code Audit passed。
- `goal next` 在 closure state 为 `audit-ready` 且 snapshot 缺失时推荐 `agentflow project code-audit`；snapshot 存在后继续推荐本地 closure state 检查，不自动执行修复或 docs refresh。
- 更新 README、ROADMAP、MVP Spec、Project Audit / Docs Refresh Boundary、construction plan、MVP productization plan、latest verification summary 和 readiness script。
- 当前阶段不创建 `.agentflow/audits/`，不自动修复 audit findings，不修改代码或文档，不调用模型，不创建远程 PR / GitHub issue / Linear issue，不修改 Desktop UI，不标记 Project done。

验证：

- `cargo test project_code_audit -- --nocapture`：pass，2 focused tests。
- `cargo fmt --check`：pass。
- `cargo test`：pass，47 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- state check`：pass，ready true，1 project / 9 milestones / 42 issues / 0 errors / 36 warnings。
- `cargo run -p agentflow-cli -- eligibility`：pass，推荐 `agentflow plan "Root Docs Refresh Snapshot v0 只读实现"`。
- `cargo run -p agentflow-cli -- lease`：pass，0 active leases / 0 stale leases。
- `cargo run -p agentflow-cli -- project closure`：pass，closure state `audit`，code audit gate `snapshot-ready`，`can_mark_done=false`。
- `cargo run -p agentflow-cli -- project code-audit`：pass，生成 `.agentflow/state/project-code-audit.json` 和 `.agentflow/updates/PROJECT-CODE-AUDIT-SUMMARY.md`。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next`：pass，next action `project-closure`，推荐 `agentflow project closure`。
- `cargo run -p agentflow-cli -- projects`：pass，active milestone `workflow-core-closure-gates`。
- `cargo run -p agentflow-cli -- metrics`：pass，next action `project-closure`。
- `cargo run -p agentflow-cli -- search "Project Code Audit"`：pass，返回 snapshot 和 closure summary 可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass。
- `test ! -d .agentflow/audits`：pass。
- `git diff --check`：pass。

结论：

- Project Code Audit Snapshot v0 已能生成本地只读 audit input package。
- Snapshot 能说明当前 Project 的审计输入、风险候选和 closure blockers。
- `agentflow project closure` 已能区分 `snapshot-ready` 和 final Code Audit passed，Project 仍不能 done。
- 本阶段没有创建 `.agentflow/audits/`，没有修改 Desktop UI，没有自动修复或执行 docs refresh。
- 下一候选实现切片是 `Root Docs Refresh Snapshot v0 只读实现`。

## 2026-05-26 Root Docs Refresh Snapshot v0

执行者：Codex

目标：

- 实现 `agentflow project docs-refresh`，生成 Root Docs Refresh Snapshot v0 只读文档刷新输入包。
- 读取 closure state、code audit snapshot、README、ROADMAP、MVP Spec、architecture docs、contracts、validation docs 和 `verification.md`。
- 输出 `.agentflow/state/project-docs-refresh.json` 和 `.agentflow/updates/PROJECT-DOCS-REFRESH-SUMMARY.md`。
- 不创建 `.agentflow/audits/`，不修改文档内容作为 refresh 动作，不调用模型，不标记 Project done。
- 处理完成后，把下一候选切到产品功能：`Product Feature Creation Flow v0`。

结果：

- 新增 `ProjectDocsRefreshSnapshot`、`ProjectDocsRefreshCheckedDoc`、`ProjectDocsRefreshRequiredUpdate` 和 `ProjectDocsRefreshSummary`。
- 新增 CLI：`agentflow project docs-refresh`。
- `agentflow project closure` 现在能把 docs refresh gate 显示为 `snapshot-ready`，但仍不视为 final Root Docs Refresh passed。
- `goal next` 在 code audit / docs refresh snapshots 都存在后，回到 active project candidate，推荐 `agentflow plan "Product Feature Creation Flow v0"`。
- 更新 README、ROADMAP、MVP Spec、Project Audit / Docs Refresh Boundary、Workflow Control Core Spec、Architecture Decisions、construction plan、MVP productization plan、latest verification summary 和 readiness script。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，50 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- state check`：pass，ready true，1 project / 9 milestones / 42 issues / 0 errors / 36 warnings。
- `cargo run -p agentflow-cli -- eligibility`：pass，推荐 `agentflow plan "Product Feature Creation Flow v0"`。
- `cargo run -p agentflow-cli -- lease`：pass，0 active leases / 0 stale leases。
- `cargo run -p agentflow-cli -- project code-audit`：pass，生成 `.agentflow/state/project-code-audit.json` 和 `.agentflow/updates/PROJECT-CODE-AUDIT-SUMMARY.md`。
- `cargo run -p agentflow-cli -- project docs-refresh`：pass，生成 `.agentflow/state/project-docs-refresh.json` 和 `.agentflow/updates/PROJECT-DOCS-REFRESH-SUMMARY.md`；checked docs 14，update-needed 0，missing 0。
- `cargo run -p agentflow-cli -- project closure`：pass，closure state `audit`，code audit / docs refresh gates 均为 `snapshot-ready`，`can_mark_done=false`。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next`：pass，next action `plan`，推荐 `agentflow plan "Product Feature Creation Flow v0"`。
- `cargo run -p agentflow-cli -- projects`：pass，active milestone `workflow-core-closure-gates`，recommended command `agentflow plan "Product Feature Creation Flow v0"`。
- `cargo run -p agentflow-cli -- metrics`：pass，next action `plan`，recommended command `agentflow plan "Product Feature Creation Flow v0"`。
- `cargo run -p agentflow-cli -- search "Root Docs Refresh"`：pass，返回 docs refresh 和 closure summary 可追溯结果。
- `bash checks/agentflow-readiness.sh`：pass。
- `test ! -d .agentflow/audits`：pass。
- `git diff --check`：pass。

结论：

- Root Docs Refresh Snapshot v0 已完成本地只读 docs refresh input package。
- 当前 Project 仍不能 done，因为 final Code Audit、final Root Docs Refresh、Final Evidence Summary、Human Final Approval 和部分历史 milestone summaries 尚未完成。
- 下一阶段已切到产品功能入口：`Product Feature Creation Flow v0`。

## 2026-05-26 Product Feature Creation Flow v0

执行者：Codex

目标：

- 实现 `agentflow feature create "<feature goal>"`，把产品功能目标落成本地 Project -> Milestones -> IssueContracts。
- 默认 preview，不写事实源；只有 `--write --yes` 才写 `.agentflow/`。
- 写入后新 Project 成为 active project，并接入 `goal next / eligibility / lease / run` 受控执行链路。
- 不调用模型、不创建远程 PR / GitHub issue / Linear issue、不从 Desktop 执行创建、不标记 Project done。

结果：

- 新增 `ProductFeatureDraft`、`ProductFeatureProject`、`ProductFeatureMilestoneDraft`、`ProductFeatureIssueDraft`、`ProductFeatureCreationSnapshot`。
- 新增 CLI：`agentflow feature create`。
- 写入产物包括 `.agentflow/projects/{feature-project-id}.json`、`.agentflow/issues/ISSUE-XXXX.{json,md}`、`.agentflow/teams/{team-id}.json`、`.agentflow/workspace.json`、`.agentflow/index.json`、`.agentflow/updates/FEATURE-CREATION-SUMMARY.md`。
- IssueContract 已补齐 `riskLevel` 和 `rollbackPlan` 字段，eligibility 会检查这些执行必备字段。
- 新增规格文档：`docs/specs/product-feature-creation-flow-v0.md`。

验证：

- `cargo test`：pass，53 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- feature create "示例产品功能"`：pass，preview only。
- `cargo run -p agentflow-cli -- feature create "示例产品功能" --write --yes`：pass，创建新 Project / Milestones / IssueContracts 并更新 active project。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next`：pass，推荐新 active project 下第一条 issue。
- `cargo run -p agentflow-cli -- eligibility`：pass，第一条 feature issue 有明确 ready / eligible 状态。
- `cargo run -p agentflow-cli -- projects`：pass，workspace active project 为 `feature-0043`。
- `cargo run -p agentflow-cli -- metrics`：pass，46 issues / 42 completed / 4 planned。
- `cargo run -p agentflow-cli -- search "Product Feature Creation"`：pass，返回 `FEATURE-CREATION-SUMMARY`。
- `bash checks/agentflow-readiness.sh`：pass。
- `test ! -d .agentflow/audits`：pass。
- `git diff --check`：pass。

结论：

- AgentFlow 已具备第一个可用产品功能入口。
- 新功能创建仍然服从 IssueContract、WIP=1、Eligibility、Lease、Evidence gate。
- Desktop 仍保持只读，不参与创建或执行。

## 2026-05-26 Product Feature Execution Flow v0

执行者：Codex

目标：

- 实现 `agentflow feature status` 和 `agentflow feature next`。
- 读取当前 active Product Feature Project，展示 project、active milestone、milestone 列表、当前 issue、eligibility、latest run / validation / evidence / review 和推荐命令。
- 只推荐下一步，不执行 run / verify / review，不调用模型，不创建远程对象，不标记 Project done。

结果：

- 新增 `ProductFeatureExecutionSnapshot`、`ProductFeatureExecutionMilestone`、`ProductFeatureExecutionIssue`。
- 新增 CLI：`agentflow feature status`、`agentflow feature next`。
- `feature next` 复用现有 `issue_next_step` 语义：无 run 推荐 run，已有 run 未验证推荐 verify，验证通过未 review 推荐 review，review 完成后由既有 milestone summary writer 激活下一个 milestone。
- 新增规格文档：`docs/specs/product-feature-execution-flow-v0.md`。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，55 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- feature status`：pass，当前 active feature project 为 `feature-0043`，current issue 为 `ISSUE-0043`。
- `cargo run -p agentflow-cli -- feature next`：pass，推荐 `agentflow run ISSUE-0043 --dry-run`。
- `cargo run -p agentflow-cli -- goal check`：pass，ready true。
- `cargo run -p agentflow-cli -- goal next`：pass，推荐 `agentflow run ISSUE-0043 --dry-run`。
- `cargo run -p agentflow-cli -- eligibility`：pass，`ISSUE-0043` eligible。
- `cargo run -p agentflow-cli -- projects`：pass，active project `feature-0043`。
- `cargo run -p agentflow-cli -- metrics`：pass，next action `run`。
- `cargo run -p agentflow-cli -- search "Product Feature Execution"`：pass。
- `bash checks/agentflow-readiness.sh`：pass。
- `test ! -d .agentflow/audits`：pass。
- `git diff --check`：pass。

结论：

- 创建后的 Product Feature Project 已有可读执行入口。
- 当前下一步稳定指向 `ISSUE-0043` 的 run。
- 该层只读，不绕过 Workflow Control Core。

## 2026-05-26 Product Feature Controlled Run v0

执行者：Codex

目标：

- 强化 `agentflow run ISSUE-XXXX --dry-run`，使当前 active feature project 的唯一 eligible issue 进入受控 dry-run。
- run 前必须通过 active project、active milestone、eligibility、lease 和 WIP=1 gate。
- dry-run 必须输出 run plan、expected files、blocked files / areas、validation commands、evidence requirements 和 rollback plan。
- `feature status` 能显示 latest run plan，`feature next` 能在 dry-run 后推荐 verify。

结果：

- 新增 `ControlledRunPlan`，并写入 `AgentRun.runPlan`。
- `agentflow run` 在 v0 只接受 `--dry-run`，不带 dry-run 会失败。
- `run.json` 记录 project / milestone / issue / lease 关系和 run plan。
- `transcript.md` 与 `diff-summary.md` 展示受控执行计划和 no source edits 边界。
- `ProductFeatureExecutionIssue` 增加 dry-run recorded、latest run plan、expected files、blocked files、validation commands 和 evidence requirements。
- 新增规格文档：`docs/specs/product-feature-controlled-run-v0.md`。

验证：

- `cargo test -p agentflow-core product_feature_execution_next_moves_from_run_to_verify_to_review_to_next_milestone -- --nocapture`：pass。
- `cargo test -p agentflow-core controlled_run_records_plan_and_updates_feature_status -- --nocapture`：pass。
- `cargo fmt --check`：pass。
- `cargo test`：pass，56 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- feature status`：pass，显示 `dry-run recorded: true` 和 latest run plan。
- `cargo run -p agentflow-cli -- feature next`：pass，推荐 `agentflow verify ISSUE-0043`。
- `cargo run -p agentflow-cli -- run ISSUE-0043 --dry-run`：pass，复用 / 输出 `RUN-0041`，包含 project / milestone / lease / runPlan。
- `cargo run -p agentflow-cli -- eligibility`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass。
- `cargo run -p agentflow-cli -- goal next`：pass，active issue 推荐 verify。
- `cargo run -p agentflow-cli -- projects`：pass。
- `cargo run -p agentflow-cli -- metrics`：pass，latest run `RUN-0041 -> ISSUE-0043 [dry-run / not-run]`。
- `bash checks/agentflow-readiness.sh`：pass。
- `test ! -d .agentflow/audits`：pass。
- `git diff --check`：pass。

结论：

- Product Feature 已从创建和只读 next，推进到本地受控 dry-run。
- 当前仍不执行真实代码修改，不调用模型，不创建远程对象，不从 Desktop 执行 run。

## 2026-05-26 Goal + Criteria Driven MVP

执行者：Codex

目标：

- 将 AgentFlow 当前项目完成方式调整为 Goal + Criteria 驱动。
- 当前 MVP 主目标只保留两项：用户本地创建和管理 Team / Project / Milestone / Issue；用户和 Agent 共同把产品功能目标拆成 Project -> Milestones -> Issues 并保存为本地事实源。
- Agent 自动执行流程不再作为当前 MVP 主产品目标。

结果：

- 更新 `GOAL.md`、`.agentflow/goal.md`、`.agentflow/goal.json`。
- 新增 `docs/specs/goal-criteria-driven-mvp.md`，固定 24 条 Criteria。
- Project 状态锁定为 `draft / active / paused / completed / canceled`。
- Issue 状态锁定为 `backlog / todo / in_progress / in_review / done / canceled`。
- Milestone 不维护独立状态，只作为 Project 下的阶段分组。
- 更新 README、ROADMAP、MVP Spec、Construction Plan 和 latest verification summary。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，57 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass。
- `cargo run -p agentflow-cli -- goal next`：pass，仍报告上一轮 dry-run 留下的 active issue。
- `cargo run -p agentflow-cli -- projects`：pass。
- `cargo run -p agentflow-cli -- feature status`：pass。
- `cargo run -p agentflow-cli -- feature next`：pass，当前提示 stale lease requires human recovery。
- `bash checks/agentflow-readiness.sh`：pass。
- `git diff --check`：pass。

结论：

- 当前项目完成标准已从执行自动化闭环，切换为 Team / Project / Milestone / Issue 本地产品建模和协作创建。
- 下一实现切片应为 `Project / Issue Status Model v0`。

## 2026-05-26 Project / Issue Status Model v0

执行者：Codex

目标：

- 将 Project / Issue canonical status 落到 core、CLI、Desktop 和本地事实源读取。
- Project canonical status 固定为 `draft / active / paused / completed / canceled`。
- Issue canonical status 固定为 `backlog / todo / in_progress / in_review / done / canceled`。
- Milestone 不维护独立产品状态，只展示从 Issues 派生的完成度。

结果：

- 新增 `ProjectStatus` / `IssueStatus` canonical 定义和 legacy status 映射。
- `LocalProjectModelSnapshot` 输出 `LocalProject.canonicalStatus`、`LocalProjectIssueRef.canonicalStatus` 和 `LocalMilestone.progress`。
- `agentflow projects` 和 `agentflow feature status` 展示 canonical status。
- Desktop Project / Task 视图展示 canonical Project / Issue status，Milestone 展示 derived progress。
- Product Feature Creation 和 `agentflow plan` 新建 Issue 默认写 `todo`；review 完成 Issue 后写 `done`。
- SavedView issue status filter 支持 legacy / canonical 兼容匹配。
- 新增规格文档：`docs/specs/project-issue-status-model-v0.md`。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，56 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- goal check`：pass。
- `cargo run -p agentflow-cli -- projects`：pass。
- `cargo run -p agentflow-cli -- feature status`：pass。
- `cargo run -p agentflow-cli -- feature create "状态模型验证功能"`：pass，preview only。
- `bash checks/agentflow-readiness.sh`：pass。
- `git diff --check`：pass。

结论：

- Project / Issue canonical status 已成为本地 read model 和 UI 展示口径。
- 旧 `.agentflow/` 状态仍可兼容读取；Milestone 状态不再作为 MVP 产品状态展示。
- 下一实现切片应为 `Team / Project / Milestone / Issue Writers v0`。

## 2026-05-27 Team / Project / Milestone / Issue Writers v0

执行者：Codex

目标：

- 实现当前 MVP 的本地创建闭环：Team / Project / Milestone / Issue。
- 所有 create 命令默认 preview，不写 `.agentflow/`。
- 只有显式 `--write --yes` 后才允许写入本地事实源。

结果：

- 新增规格文档：`docs/specs/team-project-milestone-issue-writers-v0.md`。
- 新增 `TeamDraft`、`ProjectDraft`、`MilestoneDraft`、`IssueDraft`、`CreationPreview`、`CreationWriteSummary`。
- 新增 CLI：
  - `agentflow team create "<team name>"`
  - `agentflow project create "<project title>"`
  - `agentflow milestone create "<milestone title>"`
  - `agentflow issue create "<issue title>"`
- Team writer 写 `.agentflow/teams/{team-id}.json` 并更新 workspace teamIds。
- Project writer 写 `.agentflow/projects/{project-id}.json` 并更新 workspace projectIds / team projectIds，不隐式覆盖 activeProjectId。
- Milestone writer 只追加 project `milestones[]`，不写产品状态。
- Issue writer 写 `ISSUE-XXXX.{json,md}`，并同步 project / milestone / team / index 引用。
- Project 默认 status = `draft`；Issue 默认 status = `todo`。
- Desktop 继续只读展示，不新增写入口。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，59 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- team create "Demo Team"`：pass，preview only。
- `cargo run -p agentflow-cli -- project create "Demo Project"`：pass，preview only。
- `cargo run -p agentflow-cli -- milestone create "Demo Milestone"`：pass，preview only。
- `cargo run -p agentflow-cli -- issue create "Demo Issue"`：pass，preview only，Issue id preview 为 `ISSUE-0047`。
- `cargo run -p agentflow-cli -- projects`：pass。
- `cargo run -p agentflow-cli -- feature status`：pass。
- `bash checks/agentflow-readiness.sh`：pass，包含 writer preview commands 和 temp write unit tests。

结论：

- AgentFlow 当前已经具备第一个 MVP 产品创建闭环：用户可用 CLI preview / confirmed write 创建 Team、Project、Milestone、Issue。
- 执行链路仍未自动推进；run / verify / review 继续作为后续执行层能力，不进入当前创建闭环。

## 2026-05-28 Project / Milestone / Issue / View Model v1

执行者：Codex

目标：

- 将 AgentFlow MVP 产品主干收敛为 `Workspace / Team -> Project -> Milestone -> Issue -> View`，并落地只读 schema adapter。
- 固定 Project / Milestone / Issue / View 的职责、模板、状态目标、Queue Preflight、adapter 输出格式和 Desktop 页面边界。
- 明确 View 只是 saved filter，不承载业务状态。

结果：

- 新增规格文档：`docs/specs/project-milestone-issue-view-model-v1.md`。
- 新增 core schema / adapter：`ProjectMilestoneIssueViewModelSnapshot`、`V1WorkspaceRef`、`V1TeamRef`、`V1Project`、`V1Milestone`、`V1Issue`、`V1View`、`V1ViewFilter`、`V1ViewSort`。
- 新增只读 reader：`read_project_milestone_issue_view_model_snapshot`。
- 更新 Desktop TypeScript 类型，保持前端类型可对齐 v1 schema。
- 更新 README、ROADMAP、MVP Spec、Goal + Criteria Driven MVP、Project / Issue Status Model、Team / Project / Milestone / Issue Writers、Desktop Workbench Boundary、latest verification summary。
- 固定不变量：Project 不执行，Milestone 不执行，Issue 执行，View 只展示，Queue Preflight 决定谁能执行，Evidence 决定是否 Done。
- v1 状态模型先作为产品目标和后续迁移方向，不破坏当前 canonical status 和既有 `.agentflow/` 事实源。
- Adapter 保留 `rawStatus`，同时输出 v1 派生 `status`；Milestone status 只派生，不写回事实源；View 只从 SavedView 派生 filter / sort / layout。

验证：

- `cargo test -p agentflow-core project_milestone_issue_view_model_v1 -- --nocapture`：pass，2 tests。
- `cargo fmt --check`：pass。
- `cargo test`：pass，61 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- projects`：pass。
- `git diff --check`：pass。

结论：

- 当前 MVP 主干已从“功能堆叠”收敛为清晰的产品模型合同。
- 下一步应做 v1 writer preview 对齐或 Desktop 页面职责收敛，仍不进入自动 run / verify / review。

## 2026-05-29 Project / Milestone / Issue / View Model v1 Writer Preview Alignment

执行者：Codex

目标：

- 把开发文档任务推进至 100%：让 v1 产品模型合同落入 Team / Project / Milestone / Issue 创建预览。
- 保持 preview-first；不改变 `.agentflow/` 写入 schema，不执行 run / verify / review。

结果：

- `CreationPreview` 新增 `v1Contract`。
- `agentflow team create` preview 输出 Team relation。
- `agentflow project create` preview 输出 Project charter。
- `agentflow milestone create` preview 输出 Milestone gate。
- `agentflow issue create` preview 输出 Issue execution contract。
- CLI preview 增加 v1 model / relation / 关键合同摘要。
- 更新 README、ROADMAP、MVP Spec、Team / Project / Milestone / Issue Writers、Project / Milestone / Issue / View Model v1、latest verification summary。

验证：

- `cargo test -p agentflow-core team_project_milestone_issue_writers_preview_without_writes -- --nocapture`：pass，1 test。
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

边界：

- 不调用模型。
- 不创建远程 PR / GitHub issue / Linear issue。
- 不改变 `.agentflow/` 既有落盘格式。
- 不自动推进执行链路。

## 2026-05-29 Desktop Project / Milestone / Issue 页面职责收敛 v0

执行者：Codex

目标：

- 将 `Project / Milestone / Issue / View Model v1` 呈现到 Desktop Workbench。
- 每个详情页只展示自己负责的内容，避免 Project、Milestone、Issue、View 信息继续堆在同一页。
- Desktop 继续保持只读，不创建本地对象、不执行命令、不写 `.agentflow/`。

结果：

- `apps/desktop/src-tauri/src/main.rs` 新增只读 Tauri command：`load_project_milestone_issue_view_model_snapshot`。
- `apps/desktop/src/App.tsx` 的 load flow 同步读取 v1 snapshot。
- Project 页面只展示 Project charter、milestones、issue progress、queue status、closure gate。
- Milestone 区块只展示 milestone goal、entry criteria、issues、exit criteria、derived progress。
- Issue 页面只展示 issue contract 的 goal、scope、non-goals、validation、evidence、boundary、status。
- View 页面只展示 saved filter / sort / layout，不承载业务状态。
- Project / Issue 列表排序按本阶段规则收敛。
- 更新 README、ROADMAP、MVP Spec、Desktop Workbench Boundary、Project / Milestone / Issue / View Model v1、latest verification summary。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，61 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo run -p agentflow-cli -- projects`：pass，当前 active project 为 `feature-0043`，推荐命令保持 `agentflow verify ISSUE-0043`。
- `bash checks/agentflow-readiness.sh`：pass。
- `git diff --check`：pass。
- Browser verification at `http://127.0.0.1:1420/`：pass。
  - Project 页面包含 `项目职责视图`、`Project Charter`、`Queue Status`、`Closure Gate`、`Milestone 阶段门`。
  - Issue 页面包含 `Issue Contract`、`Contract`、`Execution Boundary`、`Validation`、`Evidence`、`只读边界`，且不展示 Project closure / audit。
  - View 页面包含 `高级视图` 和 saved filter 内容，不承载业务状态。
  - console warn/error：0。
  - screenshots：`/tmp/agentflow-desktop-project-page.png`、`/tmp/agentflow-desktop-issue-page.png`。

边界：

- 不写 `.agentflow/`。
- 不创建 Team / Project / Milestone / Issue。
- 不执行 run / verify / review。
- 不调用模型。
- 不创建远程 PR / GitHub issue / Linear issue。

## 2026-05-31 Project Files feature module 收口

执行者：Codex

目标：

- 将 Project 本地文件阅读器继续收口成独立 feature module。
- 减少 `App.tsx` 对 Project 文件阅读器内部实现的直接依赖。
- 保持浏览器/Vite 预览可用 mock，真实 Tauri 客户端不使用 mock fallback。

结果：

- 新增 `apps/desktop/src/features/project-files/index.ts` 作为模块入口。
- `App.tsx` 改为从 `features/project-files` 入口导入 Project 文件页、hook、类型和根路径工具。
- 删除 `App.tsx` 中已废弃的 Project Goal / Milestone / Architecture / Environment / Agent tab 模板组件，Project 页面继续只承载本地文件阅读器。
- `design.md` 更新 Project module boundary，明确 `App.tsx` 只通过模块入口使用 Project 文件模块。

验证：

- `npm --prefix apps/desktop run build`：pass。
- `cargo test`：pass，61 tests。
- `git diff --check`：pass。
- 静态搜索确认 `App.tsx` 不再直接导入 `features/project-files/*` 内部实现文件。
- 静态搜索确认 `App.tsx` 不再保留旧 Project template 组件。
- Browser verification at `http://127.0.0.1:1420/`：pass，Project 文件阅读器、右侧文件列表和 README 默认阅读状态正常渲染。

边界：

- 不写 `.agentflow/`。
- 不执行 run / verify / review。
- 不创建远程 PR / GitHub issue / Linear issue。
- 不调用模型。

## 2026-05-31 Project Files Tauri read-layer contract 补测

执行者：Codex

目标：

- 继续收口 Project 文件阅读器模块。
- 用 Tauri 侧单元测试固定真实客户端读取契约，而不是只依赖浏览器 mock。
- 验证真实 Tauri 客户端读取失败时由错误 / 空态承接，浏览器/Vite 预览才允许 mock fallback。

结果：

- `apps/desktop/src-tauri/src/project_files.rs` 承载 Project 文件读取 command、路径校验、metadata / preview 生成和本地文件夹选择 command。
- `apps/desktop/src-tauri/src/main.rs` 只保留 Tauri command 注册，不再承载 Project 文件读取实现。
- `apps/desktop/src-tauri/src/project_files.rs` 新增 6 个 Project file read-layer 单元测试：
  - 文本文件读取与 metadata。
  - 目录概览 children。
  - 未知二进制文件十六进制 fallback。
  - PNG / PDF / XLSX / DOCX 小文件 data URL 预览路径。
  - 大文本 512KB 截断标记。
  - 相对路径逃逸拒绝。
- `useProjectFiles.ts` 继续作为 Project 文件模块的加载 / 选择 / 刷新边界。
- `App.tsx` 继续只通过 `features/project-files/index.ts` 使用 Project 文件模块。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，core 61 tests + desktop Project file 6 tests。
- `npm --prefix apps/desktop run build`：pass。
- `git diff --check`：pass。

## 2026-06-04 Project Panel Finalization

执行者：Codex

目标：

- 执行 `docs/requirements/008-4-1-project-panel-finalization-and-graph-removal-v1.md`。
- 将 Project Panel 收敛为唯一项目现场模块。
- 移除旧代码地图兼容 API、命令、类型、路径和可见文案。
- 将 Agent 顶层角色收敛为 Spec / Build / Release / Audit。
- 补齐 Panel manifest / git / diagnostics / tests / snapshot / context pack 数据。

结果：

- `crates/graph` 迁移为 `crates/panel`，package 保持 `agentflow-panel`。
- Desktop Rust dependency 改为 `agentflow-panel`，不再使用旧 dependency alias。
- Tauri command surface 改为 `commands/panel.rs`：
  - `prepare_project_panel`
  - `load_project_panel_status`
  - `load_project_panel_manifest`
  - `search_project_panel`
  - `build_panel_context_pack`
  - `load_panel_context_pack`
  - `panel_preflight`
  - `analyze_panel_impact`
  - `check_panel_git_protection`
- Frontend 类型和 hook 改为 `types/panel.ts` 与 `useProjectPanel.ts`。
- Panel 新 manifest 使用 `panel-manifest.v1`，包含 status、backend、lastIndexedAt、activeSnapshotId、paths、summary、worktree、watcher、degradedReasons、warnings、errors。
- Panel 输出补齐：
  - `manifest.json`
  - `git.json`
  - `diagnostics.json`
  - `tests.json`
  - `snapshots/<id>.json`
  - `file-tree.json`
  - `languages.json`
  - `context-packs/<target>.json`
- Context Pack version 改为 `panel-context-pack.v1`。
- Goal Tree context pack 输出改到 `.agentflow/panel/context-packs/`。
- Agent 工作手册角色收敛为：
  - `Spec Agent / 规格定义 Agent`
  - `Build Agent / 实现执行 Agent`
  - `Release Agent / 发布交付 Agent`
  - `Audit Agent / 代码审计 Agent`
- README / GOAL / ROADMAP / docs index 更新到 008.4.1 当前路线。

边界：

- 未实现 OpenSpec Authoring。
- 未写 SPEC change、Approved SPEC、Goal Tree fact、AgentRun、Evidence、Audit report 或 Release record。
- 未运行用户项目 build / test 命令。
- 未写用户源码。
- 未创建远程 PR / issue。
- 未调用模型。
- 未保留旧兼容 commands / aliases / path writers。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-panel`：pass，26 tests。
- `cargo test -p agentflow-desktop`：pass，16 tests。
- `cargo test`：pass，agent-manual 11 tests + CLI 2 tests + core 61 tests + desktop 16 tests + goal-tree 3 tests + panel 26 tests。
- `npm --prefix apps/desktop run build`：pass。
- `git diff --check`：pass。
- active surface cleanup 检查：pass；active crates / apps / README / GOAL / ROADMAP / docs index 仅剩当前需求文件名中的历史目标词。
- 需求中的宽泛 cleanup `rg` 仍会命中历史 requirements / verification 旧记录和当前 008.4.1 需求正文；这些是历史审计记录，不是 active API / UI / runtime 兼容层。
- Browser smoke at `http://127.0.0.1:1420/`：pass。
  - `.project-file-page` 存在。
  - `.project-file-reader` 存在。
  - `.project-file-browser` 存在。
  - 文件列表渲染 18 行。
  - 默认阅读器 header 包含 `README.md`。
  - 只读说明文案存在。

边界：

- 不写 `.agentflow/`。
- 不执行 run / verify / review。
- 不创建远程 PR / GitHub issue / Linear issue。
- 不调用模型。

## 2026-06-01 项目文档重置与旧需求归档

执行者：Codex

目标：

- 清理 AgentFlow 当前项目文档入口。
- 将 2026-05 期间围绕 workflow control、feature flow、closure、desktop redesign 等旧需求形成的文档从默认开发入口移除。
- 为后续全新需求建立独立入口，避免旧需求继续影响后续开发判断。

结果：

- 旧文档目录已归档到 `docs/archive/2026-05-agentflow-legacy/`。
- 新增 `docs/requirements/README.md`，作为后续需求文档目录说明。
- 新增 `docs/requirements/next-requirements.md`，作为下一轮全新需求的空白入口。
- 重写 `README.md`、`GOAL.md`、`ROADMAP.md`、`AGENTS.md`、`docs/README.md`，明确当前开发只从 `docs/requirements/` 读取新需求。
- 旧文档只作为历史参考，不再作为默认开发依据。

验证：

- 静态搜索确认当前主动文档不再引用旧 `docs/specs`、`docs/contracts`、`docs/planning`、`docs/product`、`docs/validation` 等路径作为开发入口。
- `test -d docs/archive/2026-05-agentflow-legacy/specs`：pass。
- `test -f docs/requirements/next-requirements.md`：pass。
- `test ! -e docs/.DS_Store`：pass。
- `npm --prefix apps/desktop run build`：pass，存在 Vite chunk size warning。
- `cargo test`：pass，core 61 tests + desktop Project file 6 tests。
- `git diff --check`：pass。

边界：

- 不删除旧文档内容，只归档。
- 不修改 `.agentflow/` 运行态事实源。
- 不新增执行命令能力。
- 不调用模型。
- 不创建远程 PR / GitHub issue / Linear issue。

## 2026-06-01 Project Workspace Manager V0.2

执行者：Codex

目标：

- 将“添加项目 / 准备本地工作区”作为清理旧需求后的第一个新功能需求写入 `docs/requirements/`。
- 让 Desktop 可以把一个本地文件夹接入为 Project，并在真实桌面环境准备 `.agentflow/` 本地运行目录。
- 保持浏览器预览不写真实仓库。

结果：

- 新增 `docs/requirements/001-add-local-project.md`，定义 Project Workspace Manager V0.2。
- 更新 `docs/requirements/README.md`、`docs/requirements/next-requirements.md`、`GOAL.md`、`ROADMAP.md`，把当前第一功能指向 Project Workspace Manager。
- 新增 Tauri command `prepare_local_project_workspace`：
  - 创建或复用 `.agentflow/`。
  - 创建或复用 `workspace.yaml`、`config.yaml`。
  - 补齐 `define/`、`execute/`、`output/` 三阶段目录。
  - Git 项目写入 `.git/info/exclude`，排除 `.agentflow/`。
  - 不覆盖用户已有 `workspace.yaml` / `config.yaml`。
- Desktop 添加项目流程接入 workspace 准备：
  - Tauri 桌面环境选择文件夹后准备 `.agentflow/`。
  - 浏览器预览环境继续只通过路径输入模拟，不写文件。
  - 同一路径重复添加时不新增重复 Project，只切换到已有 Project。
  - 本地添加 Project 支持从侧栏列表移除；只移除客户端列表，不删除源码或 `.agentflow/`。
  - 无 `.agentflow/` 派生 Project 时，本地 Project 仍能进入文件阅读器。

验证：

- `cargo fmt --check`：pass。
- `cargo test`：pass，core 61 tests + desktop 9 tests。
- `npm --prefix apps/desktop run build`：pass，存在 Vite chunk size warning。
- `git diff --check`：pass。
- `test -e .agentflow; echo $?`：pass，返回 `1`，浏览器预览核对后真实仓库仍未重新创建 `.agentflow/`。
- Browser smoke at `http://127.0.0.1:1420/`：pass。
  - 添加项目面板可打开。
  - 浏览器预览可输入 `/Users/mac/Documents/AgentFlow` 模拟添加。
  - 重复添加同一路径不会新增重复项目。
  - 添加 / 切换后右侧显示项目文件阅读器。
  - 本地 Project 显示“从列表移除”入口。

边界：

- 只在真实 Tauri 添加项目时写 `.agentflow/` 和 `.git/info/exclude`。
- 浏览器预览不写 `.agentflow/`。
- 不创建 Goal / Milestone / Issue。
- 不检测技术栈。
- 不执行 run / verify / review。
- 不调用模型。
- 不创建远程 PR / GitHub issue / Linear issue。

## 2026-06-01 Desktop Mock Data Cleanup

执行者：Codex

说明：

- 该记录描述的是清理旧隐式 mock fallback 的阶段性结果。
- 当前规则见后续 `Browser Preview Mock Data Restore`：浏览器预览允许显式 mock；真实 Tauri 客户端不允许 mock fallback。

目标：

- 清理 Desktop 客户端的浏览器预览 mock 数据。
- 保持真实 Tauri 客户端只读取实际本地 Project 数据。
- 浏览器预览无法读取本地文件时显示空态 / 错误态，不再注入示例 Project、示例文件树或示例搜索结果。

结果：

- 删除 Desktop 旧 mock 数据源：
  - `apps/desktop/src/mockSnapshot.ts`
  - `apps/desktop/src/features/project-files/projectFileMock.ts`
- Desktop 初始化失败时不再回退到 mock snapshot，而是生成空的 Workbench / Metrics / Project Model / Project View Model snapshot。
- Project 文件阅读器失败时不再回退到 mock 文件树或 mock 文件内容。
- 浏览器预览只允许保留用户显式添加的本地 Project 入口；不能读取真实文件时显示提示：
  - `当前为浏览器预览，无法读取本地文件。请在桌面客户端打开项目。`
- 清理旧浏览器 localStorage 中的 legacy mock project：
  - `AgentFlow-Preview-Project`
- 更新 `design.md` 和 `docs/requirements/001-add-local-project.md`，明确浏览器 / Vite preview 不注入示例项目或示例文件树。

验证：

- `cargo test`：pass，core 61 tests + desktop 9 tests。
- `npm --prefix apps/desktop run build`：pass，存在 Vite chunk size warning。
- `git diff --check`：pass。
- Browser smoke at `http://127.0.0.1:1420/`：
  - sidebar 不再显示 `产品功能创建流`。
  - sidebar 不再显示 `AgentFlow-Preview-Project`。
  - 页面不再显示 `浏览器预览 · 示例数据`。
  - 右侧文件阅读器不再显示 mock 文件树。
  - 浏览器预览添加 `/Users/mac/Documents/AgentFlow` 后只保存项目入口，不注入示例文件内容。

边界：

- 不写 `.agentflow/`。
- 不创建 Goal / Milestone / Issue。
- 不执行 run / verify / review。
- 不调用模型。
- 不创建远程 PR / GitHub issue / Linear issue。

## 2026-06-01 Browser Preview Mock Data Restore

执行者：Codex

目标：

- 恢复浏览器预览所需的 mock 数据，保证 `http://127.0.0.1:1420/` 可以做 UI 测试。
- 保持真实 Tauri 桌面客户端只读取真实本地项目文件，不使用 mock fallback。

结果：

- 新增 `apps/desktop/src/browserPreviewData.ts`：
  - 提供浏览器预览专用 Workbench / Metrics / Project Model / Project View Model。
  - 提供浏览器预览专用文件树和文件内容。
  - 提供浏览器预览专用搜索结果。
- `loadSnapshot` 只有在 browser / Vite preview 中失败时才进入 `source = preview`。
- `useProjectFiles` 只有在 browser / Vite preview 中失败时才显示 mock 文件树和 mock 文件内容。
- 真实 Tauri 客户端调用失败仍显示真实错误 / 空态，不走 mock。

验证：

- `npm --prefix apps/desktop run build`：pass，存在 Vite chunk size warning。
- `cargo test`：pass，core 61 tests + desktop 9 tests。
- `git diff --check`：pass。
- `test ! -e .agentflow`：pass。
- Browser smoke at `http://127.0.0.1:1420/`：pass。
  - 浏览器预览显示 `AgentFlow` mock Project。
  - 文件列表显示 `.git`、`.DS_Store`、`Cargo.toml`、`README.md`、`apps`、`crates`、`docs`、`target`。
  - 阅读器显示 `README.md` mock 内容。
  - 不显示 raw `invoke` 错误。
  - 浏览器控制台无 app error / warning。

边界：

- 浏览器 mock 不写 `.agentflow/`。
- 浏览器 mock 不执行 run / verify / review。
- 浏览器 mock 不调用模型。
- 浏览器 mock 不创建远程 PR / GitHub issue / Linear issue。
- 真实 Tauri 客户端不允许使用 browser preview mock。

## 2026-06-01 Graph V1 Implementation

执行者：Codex

目标：

- 按 `docs/requirements/002-graph-v1.md` 实现 AgentFlow 本地代码现场地图服务。
- 在添加 / 打开本地 Project 后，系统可以准备 `.agentflow/output/graph/`，生成文件、符号、关系、搜索与 Context Pack 所需的本地索引。
- 保持 Desktop 只读边界，不新增执行命令、远程对象或模型调用能力。

结果：

- 新增 `crates/graph`：
  - `GraphStatusSnapshot` / `GraphManifestSnapshot` / `GraphSearchSnapshot` / `GraphContextPack` 等数据对象。
  - SQLite `graph.db` schema：`files`、`symbols`、`relations`、`chunks`、`context_packs`、`index_runs`。
  - 本地扫描器：跳过 `.git/`、`.agentflow/`、`node_modules/`、`target/`、`dist/`、`build/` 等运行态 / 构建目录。
  - 轻量符号抽取：源码结构、Markdown 标题、配置键。
  - 基础关系：contains、imports、test_of、configures、same_directory。
  - Search 与 Context Pack 写入：`.agentflow/output/graph/context-packs/`。
- 新增 Tauri commands：
  - `prepare_project_graph`
  - `load_project_graph_status`
  - `load_project_graph_manifest`
  - `search_project_graph`
  - `build_graph_context_pack`
  - `load_graph_context_pack`
- `prepare_local_project_workspace` 已创建：
  - `.agentflow/output/graph`
  - `.agentflow/output/graph/context-packs`
  - `.agentflow/output/graph/exports`
- Desktop Project 文件页新增轻量“代码地图”状态摘要，展示 Graph 状态、文件数、符号数、关系数和语言列表。
- 浏览器预览新增显式 Graph mock，不写真实 `.agentflow/output/graph`。
- `.gitignore` 增加 `.agentflow/output/graph/`，避免 Graph 输出进入 Git。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-graph`：pass，5 tests。
- `cargo test`：pass，core 61 tests + desktop 9 tests + graph 5 tests。
- `npm --prefix apps/desktop run build`：pass，存在 Vite chunk size warning。
- `git diff --check`：pass。
- Browser smoke at `http://127.0.0.1:1420/`：pass。
  - Project 页面显示 `代码地图`。
  - 浏览器预览显示 mock Graph 状态 `已就绪`。
  - 文件数、符号数、关系数、语言列表可见。
  - 浏览器控制台无 app error / warning。

边界：

- Graph V1 不执行项目命令。
- Graph V1 不调用模型。
- Graph V1 不创建远程 PR / GitHub issue / Linear issue。
- Graph V1 不写 `.codex/` 或 `graphify-out/`。
- Graph V1 输出仅写入 `.agentflow/output/graph/`。

## 2026-06-01 Graph V1 Completion

执行者：Codex

目标：

- 按 `docs/requirements/002-1-graph-v1-completion.md` 补齐 Graph V1 Completion。
- 将 Graph 从基础代码地图补到可作为 Agent 工作现场底座的稳定索引能力。
- 保持只读边界：不执行命令、不调用模型、不改源码、不创建远程对象。

结果：

- 已复制并登记补充需求文档：
  - `docs/requirements/002-1-graph-v1-completion.md`
  - `docs/requirements/README.md`
  - `docs/requirements/next-requirements.md`
- Graph Watcher：
  - 新增本地 watcher 和 debounce。
  - 文件变化后自动刷新 `.agentflow/output/graph/**`。
  - 忽略 `.git/`、`.agentflow/`、`node_modules/`、`target/` 等运行态 / 构建目录。
- Graph Preflight：
  - 新增 `GraphPreflightSnapshot`。
  - 新增 `graph_preflight` Tauri command。
  - missing / stale / failed / ready / degraded 状态均可返回明确结果。
  - ready / degraded 时可生成 Context Pack。
- Tree-sitter Registry：
  - 新增真实 Tree-sitter grammar 依赖和 parser registry。
  - L1 语言优先使用 Tree-sitter 解析：TypeScript / JavaScript / Python / Java / Kotlin / Swift / Go / Rust / C / C++ / C# / Dart。
  - Tree-sitter 不可用或未产出符号时降级到结构化 fallback。
- 符号索引：
  - 补齐 parent_symbol_id、start_line、end_line、visibility。
  - 补齐 Rust / TS / JS / Python / Java / Kotlin / Swift / Go / C / C++ / C# / Dart 核心符号。
  - 补齐移动端 Android / iOS / Flutter 入口、配置、组件和测试线索。
- 关系和影响分析：
  - 增加 parent_of、extends、implements、same_module、mentions、uses 等基础关系。
  - 新增 `analyze_graph_impact` API / Tauri command，返回 affected files / symbols / tests。
  - 新增测试推荐模块，覆盖 Rust / Node / Python / Go / Java / Android / iOS / Flutter / C# / PHP / Ruby。
- Git / PR 保护：
  - 新增 `check_graph_git_protection` API / Tauri command。
  - 检查 `.agentflow/` 或 `.agentflow/output/graph/` 是否被 Git 排除。
  - Graph status 会携带 watcher / preflight / protection 状态。
- Desktop / Browser preview：
  - 更新 Graph status 类型和浏览器 mock。
  - 状态通道可读取 watcher / preflight / protection 指标。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-graph`：pass，15 tests。
- `cargo test`：pass，core 61 tests + desktop 9 tests + graph 15 tests。
- `npm --prefix apps/desktop run build`：pass，存在 Vite chunk size warning。
- `git diff --check`：pass。

边界：

- Graph V1 Completion 不执行项目命令。
- Graph V1 Completion 不调用模型。
- Graph V1 Completion 不修改项目源码。
- Graph V1 Completion 不创建远程 PR / GitHub issue / Linear issue。
- Graph V1 Completion 输出仅写入 `.agentflow/output/graph/**`。

## 2026-06-02 Legacy Cleanup and New Module Split - Slice 1 / Tauri Split

执行者：Codex

目标：

- 按 `docs/requirements/004-legacy-cleanup-and-new-module-split.md` 建立新的主干清理需求入口。
- 先完成 legacy inventory 和 Tauri 后端模块边界拆分。
- 不新增产品功能，不改变 Desktop 只读边界，不改变 Tauri command 对外名称。

结果：

- 已复制并登记需求文档：
  - `docs/requirements/004-legacy-cleanup-and-new-module-split.md`
  - `docs/requirements/README.md`
  - `docs/requirements/next-requirements.md`
- 已新增架构边界文档：
  - `docs/architecture/legacy-code-map.md`
  - `docs/architecture/current-module-boundaries.md`
- Tauri command 层已拆出：
  - `apps/desktop/src-tauri/src/commands/mod.rs`
  - `apps/desktop/src-tauri/src/commands/legacy_core.rs`
  - `apps/desktop/src-tauri/src/commands/graph.rs`
  - `apps/desktop/src-tauri/src/commands/project_files.rs`
  - `apps/desktop/src-tauri/src/commands/project_workspace.rs`
- Project File Reader 后端已移动到模块目录：
  - `apps/desktop/src-tauri/src/project_files/mod.rs`
  - `apps/desktop/src-tauri/src/project_files/commands.rs`
- Project Workspace 后端已移动到模块目录：
  - `apps/desktop/src-tauri/src/project_workspace/mod.rs`
  - `apps/desktop/src-tauri/src/project_workspace/commands.rs`
- `agentflow-core` 已完成第一层 legacy quarantine：
  - `crates/agentflow-core/src/lib.rs` 变为薄入口。
  - 旧实现整体移动到 `crates/agentflow-core/src/legacy/mod.rs`。
  - 新增 `crates/agentflow-core/src/active/mod.rs`，只导出当前 Desktop read-only transitional read-model。
  - 新增 `crates/agentflow-core/src/shared/mod.rs`，保留为无业务立场的共享工具入口。
- `agentflow-cli` 已完成第一层 legacy isolation：
  - `crates/agentflow-cli/src/main.rs` 变为薄入口。
  - 旧命令实现整体移动到 `crates/agentflow-cli/src/legacy.rs`。
  - CLI 旧命令名称保持不变。

行为变化：

- 无预期行为变化。
- Tauri command 名称保持不变。
- Desktop 仍保持只读边界。

当前仍未完成：

- `agentflow-cli` 还未继续拆分 `args.rs` 和 `print.rs`。
- `agentflow-core` legacy 还未继续拆成 goal_protocol / product_feature / workflow_control 等细粒度文件。
- Project Files 后端进一步拆分为 model / path_guard / directory / content / search / range / mime。
- Project Workspace 后端进一步拆分为 model / prepare / dedupe / git / ignore / remove。
- Graph watcher native / fallback / filter / state / debounce 文件级拆分。
- Project File Reader 前端 browser / reader / hooks / model 拆分。
- Frontend `types/` 拆分。

已验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-core`：pass，61 tests。
- `cargo test -p agentflow-cli`：pass，0 tests。
- `cargo test -p agentflow-graph`：pass，26 tests。
- `cargo test -p agentflow-desktop`：pass，16 tests。
- `cargo test`：pass，core 61 tests + desktop 16 tests + graph 26 tests。
- `npm --prefix apps/desktop run build`：pass，存在 Vite chunk size warning。
- `git diff --check`：pass。

## 2026-06-02 Legacy Cleanup and New Module Split - Completed Split

执行者：Codex

目标：

- 完成 `docs/requirements/004-legacy-cleanup-and-new-module-split.md` 授权的 legacy 隔离和新模块拆分。
- 保持旧命令名、Tauri command 名称、Desktop 只读边界和现有用户行为不变。
- 不新增 Goal Tree、AgentRun、执行命令、模型调用或远程对象能力。

结果：

- `agentflow-core`：
  - `crates/agentflow-core/src/lib.rs` 已变为薄入口。
  - 旧 2026-05 workflow / product-feature 实现进入 `crates/agentflow-core/src/legacy/archive_2026_05.rs`。
  - `crates/agentflow-core/src/legacy/` 已按 goal protocol、product feature、Team/Project/Milestone/Issue、workflow control、run/verify/review、eligibility/lease、closure、audit/docs refresh、evidence、saved view、SQLite index 建立 legacy compatibility 出口。
  - 当前 Desktop 仍需的 read-only transitional read-model 通过 `crates/agentflow-core/src/active/` 的 `desktop_snapshot.rs`、`local_metrics.rs`、`local_project_model.rs`、`local_search.rs`、`boundary.rs` 导出。
  - `crates/agentflow-core/src/shared/` 已建立 `fs_paths.rs`、`json_io.rs`、`markdown.rs`、`ids.rs`、`time.rs` 无业务立场边界。
- `agentflow-cli`：
  - `crates/agentflow-cli/src/main.rs` 已变为薄入口。
  - 旧命令实现隔离到 `crates/agentflow-cli/src/legacy.rs`。
  - clap 参数定义拆到 `crates/agentflow-cli/src/args.rs`。
  - 输出 helper 拆到 `crates/agentflow-cli/src/print.rs`。
- Tauri Desktop：
  - command wrapper 拆到 `apps/desktop/src-tauri/src/commands/`。
  - legacy core read-model command 独立到 `commands/legacy_core.rs`。
  - graph / project files / project workspace command wrapper 分离。
- Project Files backend：
  - 拆为 `commands.rs`、`model.rs`、`path_guard.rs`、`directory.rs`、`content.rs`、`search.rs`、`range.rs`、`mime.rs`。
  - 保持路径逃逸、symlink、directory page、search、text range、binary fallback 行为不变。
- Project Workspace backend：
  - 拆为 `commands.rs`、`model.rs`、`prepare.rs`、`git.rs`、`ignore.rs`、`dedupe.rs`、`remove.rs`。
  - `dedupe.rs` 和 `remove.rs` 仅作为后续需求边界，不新增功能。
- Graph Watcher：
  - 拆为 `watcher/mod.rs`、`native.rs`、`fallback.rs`、`filter.rs`、`state.rs`、`debounce.rs`、`tests.rs`。
  - watcher 对外 API 和 native/fallback 行为保持不变。
- Project File Reader frontend：
  - 拆为 `browser/`、`reader/`、`reader/renderers/`、`hooks/`、`model/`。
  - `LargeTextReader`、file browser rows、localStorage reader state 已独立。
  - `useProjectFiles.ts` 保留为对外协调器。
  - `useProjectDirectoryPages.ts`、`useProjectFileSearch.ts`、`useProjectFileTextRange.ts`、`projectFileRuntime.ts` 已分别承接目录分页、搜索、文本分段读取和浏览器预览运行态边界。
- Frontend types：
  - 新增 `apps/desktop/src/types/` 领域类型目录。
  - `apps/desktop/src/types.ts` 保留 barrel export，现有 import 不破坏。
- 文档：
  - `docs/architecture/legacy-code-map.md` 记录旧代码兼容面。
  - `docs/architecture/current-module-boundaries.md` 记录当前模块边界。

行为变化：

- 无预期行为变化。
- CLI 旧命令仍保持可编译。
- 所有 Tauri command 名称保持不变。
- Desktop 仍保持只读边界。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-core`：pass，61 tests。
- `cargo test -p agentflow-cli`：pass，0 tests。
- `cargo test -p agentflow-graph`：pass，26 tests。
- `cargo test`：pass，core 61 tests + desktop 16 tests + graph 26 tests。
- `npm --prefix apps/desktop run build`：pass，存在 Vite chunk size warning。
- `git diff --check`：pass。

边界：

- 未新增产品功能。
- 未定义新的 Goal / Milestone / Issue / AgentRun 流程。
- 未调用模型。
- 未执行项目命令。
- 未修改用户项目源码。
- 未写入 `.agentflow/` 运行态数据。

## 2026-06-02 Goal Tree V1

执行者：Codex

目标：

- 执行 `docs/requirements/007-goal-tree-v1.md`。
- 新增本地 Goal Tree V1，用新的 Goal / Milestone / Issue 模型管理本地 Project Workspace 下的目标树。
- 为未来 AgentRun 提供稳定输入，但本阶段不启动 Agent、不执行命令、不调用模型。

结果：

- 需求文档已复制到 `docs/requirements/007-goal-tree-v1.md`。
- 新增 `crates/goal-tree`，Cargo package 为 `agentflow-goal-tree`。
- 新增 Goal / Milestone / Issue / GoalTreeIndex 模型，不复用旧 `IssueContract` / `GoalLoop` / `AgentRun`。
- 新增 `.agentflow/define/**` JSON storage：
  - `.agentflow/define/goal-tree.json`
  - `.agentflow/define/goals/*.json`
  - `.agentflow/define/milestones/*.json`
  - `.agentflow/define/issues/*.json`
- 新增 load/create/update/archive/reorder/validate API。
- 新增 integrity validation：
  - 缺失引用
  - ready Issue 必填字段
  - archived dependency
  - completed 状态冲突
  - Issue dependency cycle
  - validation/evidence/context pack warning
- 新增 Tauri Goal Tree commands。
- 新增 Desktop `Goal Tree` 页面：
  - 左侧 Goal / Milestone / Issue 树
  - 中间 Human editable contract 编辑器
  - 右侧 Graph context / 完整性提示 / 推荐文件入口
- 新增 browser preview Goal Tree mock，浏览器预览不写真实 `.agentflow/`。

边界：

- 不接 OpenSpec 工具链。
- 不依赖 `agentflow_core::legacy`。
- 不启动 Agent。
- 不执行项目命令。
- 不调用模型。
- 不写用户源码。
- 不写旧 `.agentflow/issues`、`runs`、`evidence`、`reviews`、`updates`、`views`。
- Graph Context Pack 失败时只返回 warning，不阻塞 Goal Tree 编辑。

验证：

- `cargo test -p agentflow-goal-tree`：pass，3 tests。
- `cargo test`：pass，CLI 2 tests + core 61 tests + desktop 16 tests + goal-tree 3 tests + graph 26 tests。
- `npm --prefix apps/desktop run build`：pass。

## 2026-06-02 Legacy Cleanup and New Module Split - Final Boundary Audit

执行者：Codex

补齐项：

- `agentflow-core` legacy 不再由 `legacy/mod.rs` 承载巨型实现。
  - 旧实现归档到 `crates/agentflow-core/src/legacy/archive_2026_05.rs`。
  - `legacy/mod.rs` 只负责声明 legacy compatibility 子模块和临时兼容导出。
  - 每个 legacy 子模块均保留 legacy compatibility 注释，并按旧领域暴露兼容符号。
- `agentflow-core` active/shared 目录补齐文件级边界。
  - active 仅包装当前 Desktop read-only transitional read-model。
  - shared 仅保留无业务立场工具边界，不承载旧 workflow 概念。
- Project File Reader frontend hook 继续拆分。
  - `useProjectFiles.ts` 保留对外协调器。
  - `useProjectDirectoryPages.ts` 承接目录分页。
  - `useProjectFileSearch.ts` 承接搜索。
  - `useProjectFileTextRange.ts` 承接文本分段读取。
  - `projectFileRuntime.ts` 承接浏览器预览运行态判断和错误文案。

最终验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-core`：pass，61 tests。
- `cargo test -p agentflow-cli`：pass，0 tests。
- `cargo test -p agentflow-graph`：pass，26 tests。
- `cargo test`：pass，core 61 tests + desktop 16 tests + graph 26 tests。
- `npm --prefix apps/desktop run build`：pass，存在既有 Vite chunk size warning。
- `git diff --check`：pass。

## 2026-06-02 Legacy and Degraded Code Removal

执行者：Codex

目标：

- 执行 `docs/requirements/005-legacy-and-degraded-code-removal.md`。
- 对 `crates/agentflow-core/src/legacy/archive_2026_05.rs` 和 legacy compatibility 出口做引用审计。
- 删除确认无 active / CLI / Desktop 引用的 legacy 公开出口。
- 收窄 legacy 暴露面，避免旧 2026-05 workflow 继续作为 crate root 产品 API。

结果：

- 新增 `docs/architecture/legacy-removal-audit.md`。
  - 记录 archive public symbol inventory。
  - 按 active-read-model / cli-legacy / test-only / unused / uncertain 分类。
  - 记录 degraded / fallback keep-delete 决策。
- `agentflow-core`：
  - 删除 crate root `pub use legacy::*`。
  - `legacy/archive_2026_05.rs` 改为 private module。
  - 删除 `legacy/mod.rs` 中的 `pub use archive_2026_05::*`。
  - 删除无 active / CLI / Desktop import 的 `legacy/evidence.rs` public compatibility module。
  - 保留 archive 内部 evidence/index DTO，因为旧索引逻辑和 archive tests 仍内部使用。
- `agentflow-cli`：
  - `legacy.rs` 改为显式 import `agentflow_core::active` 和命名 legacy compatibility modules。
  - `print.rs` 改为显式 import legacy print DTO。
- Desktop Tauri：
  - `commands/legacy_core.rs` 改为只从 `agentflow_core::active` 导入 transitional read-model。
  - Tauri command 名称未改变。
- 文档：
  - 更新 `docs/requirements/README.md`。
  - 更新 `docs/requirements/next-requirements.md`。
  - 更新 `docs/architecture/legacy-code-map.md`。
  - 更新 `docs/architecture/current-module-boundaries.md`。

保留：

- Desktop 必需 active read model。
- CLI legacy compatibility。
- Graph watcher native/fingerprint fallback。
- Project File Reader fallback 和 browser-preview mock data。

行为变化：

- 无预期用户行为变化。
- 旧 root-level legacy API 不再作为 public compatibility surface。
- 新需求必须继续通过 explicit `active` / named `legacy` module 访问仍授权的兼容面。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-core`：pass，61 tests。
- `cargo test -p agentflow-cli`：pass，0 tests。
- `cargo test -p agentflow-graph`：pass，26 tests。
- `cargo test`：pass，core 61 tests + desktop 16 tests + graph 26 tests。
- `npm --prefix apps/desktop run build`：pass，存在既有 Vite chunk size warning。
- `git diff --check`：pass。

边界：

- 未新增 Goal Tree。
- 未定义新的 Goal / Milestone / Issue / AgentRun。
- 未调用模型。
- 未执行项目命令。
- 未修改用户项目源码。
- 未改变 Desktop 只读边界。
- 未改变 Tauri command 对外名称。

## 2026-06-02 Legacy CLI Retirement and Archive Pruning

执行者：Codex

目标：

- 执行 `docs/requirements/006-legacy-cli-retirement-and-archive-pruning.md`。
- 将旧 2026-05 CLI writer / automation command surface 退役。
- 只临时保留 `metrics`、`projects`、`search` 三个只读 CLI 命令。
- 收窄 named legacy module re-export，移除旧 writer entrypoint 的公开兼容出口。

结果：

- 需求文档已复制到 `docs/requirements/006-legacy-cli-retirement-and-archive-pruning.md`。
- 新增 `docs/architecture/legacy-cli-retirement-plan.md`，记录每个旧 CLI 命令的分类。
- `agentflow-cli`：
  - 新增 `src/retirement.rs`，统一分类旧命令并输出退役消息。
  - 新增 `src/active.rs`，作为未来 active CLI 边界占位。
  - `src/legacy.rs` 只执行 `metrics`、`projects`、`search`。
  - 其他旧命令解析后只输出禁用说明，不执行旧流程。
- `agentflow-core`：
  - old writer re-export 已从 named legacy modules 移除。
  - active Desktop read model 需要的 read functions / DTOs 保留。
  - private `archive_2026_05.rs` 暂时保留，因为 archived tests 和 nested DTO shapes 仍依赖。
- 文档：
  - 更新 `docs/requirements/README.md`。
  - 更新 `docs/requirements/next-requirements.md`。
  - 更新 `docs/architecture/legacy-removal-audit.md`。
  - 更新 `docs/architecture/legacy-code-map.md`。
  - 更新 `docs/architecture/current-module-boundaries.md`。

CLI 退役 smoke：

```text
cargo run -p agentflow-cli -- run ISSUE-0001 --dry-run

legacy command: run
disposition: disable-with-message
reason: new AgentRun has not been defined yet
This command belongs to the archived 2026-05 AgentFlow workflow.
It is disabled in the new requirements track.
The new Goal Tree / AgentRun workflow has not been defined yet.
No files were written and no command was executed.
```

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-core`：pass，61 tests。
- `cargo test -p agentflow-cli`：pass，2 tests。
- `cargo test -p agentflow-graph`：pass，26 tests。
- `cargo test`：pass，CLI 2 tests + core 61 tests + desktop 16 tests + graph 26 tests。
- `npm --prefix apps/desktop run build`：pass。
- `npm --prefix apps/desktop audit`：pass，found 0 vulnerabilities。
- `cargo run -p agentflow-cli -- run ISSUE-0001 --dry-run`：pass，输出退役禁用消息，未执行旧 run。
- `cargo run -p agentflow-cli -- metrics`：pass，只读命令仍可执行。
- `git diff --check`：pass。

边界：

- 未新增 Goal Tree。
- 未定义新的 Goal / Milestone / Issue / AgentRun。
- 未调用模型。
- 未执行用户项目命令。
- 未修改用户项目源码。
- 未改变 Project Workspace / Graph / Project File Reader 行为。
- 未改变 Desktop UI。
- 未改变 Tauri command 名称。
- 未写入 `.agentflow/` 运行态数据。

## 2026-06-03 Goal Tree V1 Agent-Only Boundary Fix

执行者：Codex

目标：

- 执行 `docs/requirements/007-1-goal-tree-agent-only-boundary-fix.md`。
- 保留 Goal Tree V1 模型和 `.agentflow/define/**` 本地事实源。
- 将 Desktop Goal Tree 人类界面改为只读。
- 从 Desktop Tauri command surface 移除 Goal Tree 写命令和 Graph Context 准备命令。
- 将 `agentflow-goal-tree` 写 API 标注为 agent-only / system-only / internal tests。

结果：

- 需求文档已复制到 `docs/requirements/007-1-goal-tree-agent-only-boundary-fix.md`。
- Goal Tree 页面只保留刷新、选择、查看 Contract / Agent Draft / System State / Context / 完整性提示。
- Goal / Milestone / Issue 创建、编辑、保存、归档、排序入口已从 Desktop UI 移除。
- `GoalTreeContextPanel` 不再提供“准备 Graph Context”按钮，只显示已有 `graphContextPackPath`、Agent Draft 推荐文件和完整性提示。
- Desktop Tauri handler 只注册：
  - `load_goal_tree_snapshot`
  - `validate_goal_tree`
- `agentflow-goal-tree` crate 写 API 保留给未来 Agent planning flow、system migration 和 internal tests，不暴露给 Desktop human UI。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-goal-tree`：pass，3 tests。
- `cargo test -p agentflow-desktop`：pass，16 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo test`：pass，CLI 2 tests + core 61 tests + desktop 16 tests + goal-tree 3 tests + graph 26 tests。
- `git diff --check`：pass。
- Browser Preview Goal Tree 只读核对：pass。
  - 页面显示 `只读` 和 `Agent-only`。
  - 不显示 `创建 Goal` / `创建 Milestone` / `创建 Issue`。
  - 不显示 `保存合同` / `归档` / `准备 Graph Context`。

边界：

- 未启动 Agent。
- 未执行项目命令。
- 未调用模型。
- 未写用户源码。
- 人类 Desktop UI 不写 `.agentflow/define/**`。
- 人类 Desktop UI 不写 `.agentflow/output/graph/context-packs/**`。

## 2026-06-03 Agent Working Manual Bootstrap V1

执行者：Codex

目标：

- 执行 `docs/requirements/008-agent-working-manual-bootstrap-v1.md`。
- 新增 Agent 工作手册 bootstrap 能力，接管根目录 `AGENT.MD`。
- 写入 `.agentflow/define/agent/Agentflow.md`、5 个内置 skills、`skills-lock.json`。
- 在 Project Workspace prepare、App 打开 / 项目切换的 Desktop 状态通道中接入 Agent Manual 状态。

结果：

- 新增 `crates/agent-manual`，Cargo package 为 `agentflow-agent-manual`。
- 新增模型：
  - `AgentEnvironmentStatus`
  - `AgentMdStatus`
  - `ManualStatus`
  - `SkillsLockStatus`
  - `SkillStatus`
  - `SkillsLock`
- 新增 API：
  - `prepare_agent_working_manual`
  - `validate_agent_working_manual`
  - `repair_agent_working_manual`
  - `load_agent_environment_status`
  - `assert_agent_environment_ready`
- `prepare_local_project_workspace` 已接入 Agent Manual bootstrap。
- Desktop Tauri 新增 Agent Manual commands：
  - `prepare_agent_working_manual`
  - `load_agent_environment_status`
  - `repair_agent_working_manual`
  - `validate_agent_working_manual`
- Desktop 增加 browser preview mock Agent Manual 状态。
- Agent 状态通道增加 `工作手册` 通道，仍保持一次只显示一个通道事件 + 状态。

自动修复行为：

- 缺失 `AGENT.MD` 时创建。
- 已有 `AGENT.MD` 时备份到 `.agentflow/output/backup/agent-md/` 后重写。
- 缺失 / mismatch 的 `Agentflow.md`、`SKILL.md`、`skills-lock.json` 会自动修复。
- `AGENT.MD` 被 Git 跟踪时记录 warning，不阻断 ready。
- `AGENT.MD` 是指向项目外的 symlink 时进入 `blocked`。

边界：

- 本轮唯一允许写入项目根目录的是 `AGENT.MD`。
- 允许写 `.agentflow/define/agent/**`。
- 允许写 `.agentflow/output/backup/agent-md/**`。
- 允许写 `.agentflow/output/logs/**`。
- 未写 OpenSpec changes。
- 未写 Goal Tree。
- 未启动 AgentRun。
- 未执行用户项目命令。
- 未调用模型。
- 未创建远程 PR / Issue。
- 未写旧 `.agentflow/issues`、`runs`、`evidence`、`reviews`、`updates`、`views`。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-agent-manual`：pass，5 tests。
- `cargo test -p agentflow-desktop`：pass，16 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo test`：pass，agent-manual 5 tests + CLI 2 tests + core 61 tests + desktop 16 tests + goal-tree 3 tests + graph 26 tests。
- `git diff --check`：pass。
- Browser Preview 最新前端包核对：pass。
  - 临时打开 `http://127.0.0.1:1421/`。
  - 页面非空。
  - 状态栏显示 `工作手册 · 已就绪`。

## 2026-06-03 Agent Working Manual Health Polish

执行者：Codex

目标：

- 执行 `docs/requirements/008-1-agent-working-manual-health-polish.md`。
- 补齐 Agent Working Manual 健康闭环：
  - `validate_agent_working_manual` 检查 `.agentflow/define/agent/state/bootstrap.json`。
  - `validate_agent_working_manual` 检查 `.agentflow/define/agent/state/validation.json`。
  - `AGENT.MD` 是项目内 symlink 时记录 warning，但不阻断 ready。

结果：

- 缺失 `bootstrap.json` 或 `validation.json` 时，Agent Manual status 进入 `Missing`，`ready=false`。
- `load_agent_environment_status` 只有在 bootstrap / validation 两个 state 文件都存在时才复用 validation cache。
- 项目内 `AGENT.MD` symlink 进入 warnings，项目外 symlink 仍然 blocked。
- 新增 agent-manual 单元测试覆盖 state 文件缺失、缓存重验和项目内 symlink warning。

边界：

- 未新增 OpenSpec / Goal Tree / AgentRun 代码。
- 未改 Desktop 页面结构。
- 未新增执行命令能力。
- 未调用模型。
- 未写用户项目源码。
- 未改 `.agentflow/` runtime 目录结构。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-agent-manual`：pass，8 tests。
- `cargo test`：pass，agent-manual 8 tests + CLI 2 tests + core 61 tests + desktop 16 tests + goal-tree 3 tests + graph 26 tests。
- `npm --prefix apps/desktop run build`：pass。
- `git diff --check`：pass。

## 2026-06-03 Requirement Intake Filter Skill V1

执行者：Codex

目标：

- 执行 `docs/requirements/008-2-requirement-intake-filter-skill-v1.md`。
- 在 Agent Working Manual 中新增 AgentFlow 原生 `requirement-intake-filter` skill。
- 将 Agent 工作流调整为：
  - Conversation
  - Request triage
  - Requirement intake filter
  - OpenSpec Draft Preview
  - Human confirmation
  - Approved OpenSpec
  - Goal Tree materialization
  - Future AgentRun

结果：

- `skill_templates()` 从 5 个 skill 扩展为 6 个。
- 新增 `.agentflow/define/agent/skills/requirement-intake-filter/SKILL.md` 模板。
- `AGENT.MD` 模板增加 OpenSpec Draft 前必须运行 requirement-intake-filter 的硬规则。
- `Agentflow.md` 模板增加 Requirement Intake Result 准入规则。
- `skills-lock.json` expected template 自动包含第 6 个 skill 和 hash。
- `validate_agent_working_manual` 可检测第 6 个 skill 缺失或 hash mismatch。
- `repair_agent_working_manual` 可恢复第 6 个 skill。
- Browser Preview Agent Manual mock 更新为 `Skills 6/6`。
- 新增 agent-manual 单元测试覆盖 requirement-intake-filter 缺失后的 validate / repair。

边界：

- 未复制 Lyra 原文。
- 未使用 Lyra 名称。
- 未生成 OpenSpec 文件。
- 未写 Goal Tree。
- 未启动 AgentRun。
- 未执行用户项目命令。
- 未调用模型。
- 未写用户源码。
- 未写旧 `.agentflow/issues`、`runs`、`evidence`、`reviews`、`updates`、`views`。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-agent-manual`：pass，9 tests。
- `cargo test`：pass，agent-manual 9 tests + CLI 2 tests + core 61 tests + desktop 16 tests + goal-tree 3 tests + graph 26 tests。
- `npm --prefix apps/desktop run build`：pass。
- `git diff --check`：pass。

## 2026-06-04 Workflow Directory Blueprint V1

执行者：Codex

目标：

- 执行 `docs/requirements/008-3-agentflow-workflow-directory-blueprint-v1-final.md`。
- 将 AgentFlow managed root entry 从 `AGENT.MD` 切换为 canonical `AGENTS.md`。
- 保留 `AGENT.MD` 作为 legacy compatibility，不强制创建、不删除、不接管。
- 建立 `.agentflow/workspace-manifest.json` 和 008.3 workflow directory skeleton。
- 新增 Root Agent Entry Shadow Guard，只 warning 不重写 shadow files。

结果：

- 新增 008.3 需求文档并登记到 `docs/requirements/README.md` 与 `next-requirements.md`。
- `agent-manual` 新增 layout prepare / validation / manifest / shadow guard 状态。
- `repair_agent_working_manual` 写入 `AGENTS.md`、`Agentflow.md`、skills、`skills-lock.json`、五本工作手册和 workspace manifest。
- `.agentflow/define/` 收敛为 `agent/spec/tdd/release/audit` 工作手册区。
- `.agentflow/spec/`、`.agentflow/goal-tree/`、`.agentflow/graph/`、`.agentflow/execute/`、`.agentflow/output/`、`.agentflow/state/` 在 prepare / repair 中建立骨架。
- Desktop 状态栏 mock 和类型更新为 008.3 layout 状态。
- Project Workspace prepare 通过 agent-manual 接入 workflow layout prepare / repair，不再主动创建旧 `define/goals`、`define/milestones`、`define/issues`。

边界：

- 未迁移旧 Goal Tree 数据。
- 未迁移旧 Graph 数据。
- 未写 SPEC change、Approved SPEC、Goal、Milestone、Issue、AgentRun、Evidence、Audit report 或 Release record。
- 未接管 `.rules`、`.cursorrules`、`.windsurfrules`、`.clinerules`、`AGENT.md`、`CLAUDE.md`、`GEMINI.md`。
- 未创建 PR、远程 issue 或调用模型。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-agent-manual`：pass，11 tests。
- `cargo test -p agentflow-desktop`：pass，16 tests。
- `cargo test`：pass，agent-manual 11 tests + CLI 2 tests + core 61 tests + desktop 16 tests + goal-tree 3 tests + graph 26 tests。
- `npm --prefix apps/desktop run build`：pass。
- `git diff --check`：pass。

## 2026-06-04 Project Panel V1

执行者：Codex

目标：

- 执行 `docs/requirements/008-4-project-panel-v1.md`。
- 将 Graph 产品概念升级为 Project Panel。
- 将新的 canonical path 切换为 `.agentflow/panel/`。
- 保留 `.agentflow/output/graph/` 与 `.agentflow/graph/` 作为 legacy compatibility。

结果：

- 新增 008.4 需求文档并登记到 `docs/requirements/README.md` 与 `next-requirements.md`。
- `crates/graph` 的 package 名改为 `agentflow-panel`，desktop 依赖保留 `agentflow-graph` alias 以兼容既有 Rust import。
- 新增 Panel public API：`prepare_project_panel`、`load_project_panel_status`、`load_project_panel_manifest`、`search_project_panel`、`build_panel_context_pack`、`load_panel_context_pack`、`panel_preflight`、`analyze_panel_impact`、`check_panel_git_protection`、`ensure_panel_watcher`。
- Tauri 新增 Panel 命名 commands，旧 Graph commands 保留为 compatibility alias。
- Panel 新写入 `.agentflow/panel/**`：
  - `manifest.json`
  - `file-tree.json`
  - `languages.json`
  - `symbols.json`
  - `relations.json`
  - `diagnostics.json`
  - `git.json`
  - `tests.json`
  - `search/file-index.json`
  - `search/symbol-index.json`
  - `search/content-index.json`
  - `context-packs/`
  - `snapshots/`
  - `index/panel.db`
- `workspace-manifest.json` active layer 改为 `panel`，并新增 Panel paths：`panelManifest`、`panelFileTree`、`panelLanguages`、`panelSymbols`、`panelRelations`、`panelDiagnostics`、`panelGit`、`panelTests`、`panelSearch`、`panelContextPacks`、`panelSnapshots`、`panelIndex`。
- `workspace-manifest.json` compat 保留 `legacyGraphOutput = .agentflow/output/graph` 和 `legacyGraphCanonical = .agentflow/graph`。
- Desktop 优先调用 Panel commands，状态通道来源改为 `008.4 - Project Panel V1`。
- Browser Preview 和 Goal Tree context 可见文案从 Graph 收敛为 Panel / 项目现场。
- README / GOAL / ROADMAP 更新当前目标与 Panel canonical path。

边界：

- 未创建 `.agentflow/panel/output/`。
- 未创建 `.agentflow/inspect/`。
- 未删除旧 `.agentflow/output/graph/` 或 `.agentflow/graph/`。
- 未写 SPEC、Goal Tree、AgentRun、Evidence、Audit report 或 Release record。
- 未写用户源码。
- 未执行用户项目命令。
- 未调用模型。
- 未创建 PR、远程 issue 或 Linear issue。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-panel`：pass，26 tests。
- `cargo test -p agentflow-desktop`：pass，16 tests。
- `cargo test`：pass，agent-manual 11 tests + CLI 2 tests + core 61 tests + desktop 16 tests + goal-tree 3 tests + panel 26 tests。
- `npm --prefix apps/desktop run build`：pass。
- `git diff --check`：pass。

## 2026-06-04 AgentFlow Workspace Ownership Guard V1

执行者：Codex

目标：

- 执行 `docs/requirements/008-4-2-agentflow-workspace-ownership-guard-v1.md`。
- 在 Project Workspace prepare / Agent Manual prepare / Panel prepare 写入 `.agentflow/` 前检查归属权。
- foreign / blocked `.agentflow/` 不自动写入、不自动修复、不自动接管。

结果：

- 新增 AgentFlow workspace ownership 模型：
  - `none`
  - `managed-current`
  - `managed-legacy`
  - `foreign`
  - `corrupted`
  - `blocked`
- `.agentflow/workspace-manifest.json` 新增：
  - `managedBy = "AgentFlow"`
  - `ownership.status`
  - `ownership.createdBy`
  - `ownership.createdAt`
  - `ownership.lastValidatedAt`
  - `ownership.migratedFrom`
  - `ownership.migrationRecord`
- 新增 Rust API：
  - `check_agentflow_workspace_ownership`
  - `assert_agentflow_workspace_owned_or_creatable`
  - `take_over_agentflow_workspace`
- 新增 Tauri commands：
  - `load_agentflow_workspace_ownership`
  - `take_over_agentflow_workspace`
- Project Workspace prepare 改为：
  - 先检查 `.agentflow/` ownership。
  - 再准备 Agent Manual / workspace manifest。
  - 只有 ready 后才写 `workspace.yaml`、`config.yaml` 和 `.git/info/exclude`。
- Agent Manual validate / load / repair 改为：
  - validate 输出 ownership 状态。
  - cached validation 遇到 foreign / blocked 会重新 validate。
  - repair 遇到 foreign / blocked 直接返回 blocked，不写 `.agentflow/**` 或 `AGENTS.md`。
- Panel prepare / index / context pack build 改为：
  - 先检查 workspace ownership。
  - ready 后准备 Agent Manual。
  - foreign / blocked 时不写 `.agentflow/panel/**`。
- Desktop 类型和 Browser Preview mock 已接入 `ownership` 字段。
- 状态通道新增“工作区归属”事件，展示已接管 / 旧版接管 / 外部目录 / 已阻断等状态。
- 测试覆盖：
  - no `.agentflow` 可创建。
  - current managed manifest。
  - legacy marker migration / repair。
  - foreign `.agentflow` blocked 且不写入。
  - corrupted AgentFlow manifest 可修复。
  - corrupted foreign manifest blocked。
  - internal `.agentflow` symlink warning。
  - external `.agentflow` symlink blocked。
  - explicit takeover 备份 foreign `.agentflow` 并创建新 managed workspace。
  - Project Workspace foreign blocked。
  - Panel foreign blocked。

边界：

- 未在 UI 自动触发 takeover。
- 未自动接管 foreign `.agentflow/`。
- 未写用户源码。
- 未执行用户项目命令。
- 未调用模型。
- 未创建 PR、远程 issue 或 Linear issue。

验证：

- `cargo fmt`：pass。
- `cargo test`：pass，agent-manual 21 tests + CLI 2 tests + core 61 tests + desktop 17 tests + goal-tree 3 tests + panel 27 tests。
- `npm --prefix apps/desktop run build`：pass。

## 2026-06-04 Spec Agent Status Wording Polish

执行者：Codex

目标：

- 修正 Agentflow.md 模板中的 Spec Agent 状态文案。
- 避免 Agent 误解当前已经完整开放 Approved SPEC 写入和 Goal Tree materialization。

结果：

- `Spec Agent / 规格定义 Agent` 状态改为仅启用 requirement intake 与 SPEC Draft Preview。
- 模板明确写入：Approved SPEC writes 与 Goal Tree materialization 尚未由当前 manual 启用，必须等待后续需求和工具授权。
- 增加回归测试，防止文案退回为模糊的 `Status: enabled.`。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-agent-manual`：pass，21 tests。
- `cargo test`：pass，agent-manual 21 tests + CLI 2 tests + core 61 tests + desktop 17 tests + goal-tree 3 tests + panel 27 tests。
- `npm --prefix apps/desktop run build`：pass。
- `git diff --check`：pass。

## 2026-06-04 Input Model V1

执行者：Codex

目标：

- 执行 `docs/requirements/009-input-model-v1.md`。
- 将需求输入、SPEC Gate 和施工清单统一收敛到 `.agentflow/input/`。
- 旧 `.agentflow/spec/` 与 `.agentflow/goal-tree/` 仅作为 legacy marker，不再作为新写入路径。

结果：

- 新增 `crates/input`，Cargo package 为 `agentflow-input`。
- 新增 Input canonical layout：
  - `.agentflow/input/manifest.json`
  - `.agentflow/input/index.json`
  - `.agentflow/input/intake/`
  - `.agentflow/input/specs/drafts/`
  - `.agentflow/input/specs/approved/`
  - `.agentflow/input/specs/archive/`
  - `.agentflow/input/projects/`
  - `.agentflow/input/issues/`
  - `.agentflow/input/relations/issue-relations.json`
  - `.agentflow/input/relations/dependency-graph.json`
  - `.agentflow/input/views/active.json`
  - `.agentflow/input/views/blocked.json`
  - `.agentflow/input/views/by-spec.json`
  - `.agentflow/input/views/by-project.json`
- 新增 Input public API：
  - `prepare_input_workspace`
  - `validate_input_workspace`
  - `load_input_status`
  - `load_input_manifest`
  - `load_input_index`
  - `load_input_snapshot`
  - `repair_input_workspace`
  - `validate_input_snapshot`
- Input validation 已覆盖：
  - manifest / index / required paths 缺失。
  - Spec Gate draft / approved descriptor 文件完整性。
  - Approved SPEC 必须有 `approval.json`。
  - Issue 必须带 `sourceSpecId`。
  - direct issue 必须 `projectId = null`。
  - project issue 必须引用存在的 project。
  - project issueIds 和 issue relations 必须引用存在的 issue。
  - Issue model 不暴露 automation / humanGates / PR automation 等复杂自动化字段。
  - `riskLevel = high` 才需要人类确认。
- Agent Manual workspace layout 已接入 `.agentflow/input/**`，prepare 不再创建新的 `.agentflow/spec/` 或 `.agentflow/goal-tree/`。
- Agentflow.md 模板已更新为 Input Model V1 能力边界：
  - Spec Gate 使用 product.md + tech.md + approval.json。
  - Approved SPEC 后生成 direct issue 或 project issues。
  - 新写入只进入 `.agentflow/input/**`。
- Desktop Tauri 新增 Input commands：
  - `prepare_input_workspace`
  - `load_input_status`
  - `load_input_manifest`
  - `load_input_index`
  - `load_input_snapshot`
  - `validate_input`
- Project Workspace prepare 已接入 Input prepare，并将 input status 返回给 Desktop。
- Desktop 状态通道新增“需求输入”事件，展示 Intake、Draft SPEC、Approved SPEC、Projects、Issues 和 High Risk 汇总。
- Browser Preview mock 已同步 Input status 和新 skill 名称。
- README / GOAL / ROADMAP / docs index / requirements index 已更新到 009。

边界：

- 未迁移旧 `.agentflow/spec/` 或 `.agentflow/goal-tree/` 数据。
- 未强制删除旧 legacy 目录。
- 未启用 AgentRun。
- 未执行用户项目命令。
- 未写用户源码。
- 未调用模型。
- 未创建 PR、远程 issue 或 Linear issue。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-input`：pass，8 tests。
- `cargo test -p agentflow-desktop`：pass，17 tests。
- `cargo test`：pass，agent-manual 21 tests + CLI 2 tests + core 61 tests + desktop 17 tests + goal-tree 3 tests + input 8 tests + panel 27 tests。
- `npm --prefix apps/desktop run build`：pass。
- `git diff --check`：pass。

## 2026-06-05 Output Evidence / Delivery / Audit V1

执行者：Codex

目标：

- 执行 `docs/requirements/011-output-evidence-delivery-audit-v1.md`。
- 将 `.agentflow/output/` 正式收口为 AgentFlow 的交付与证据层。
- 明确 `output/evidence/` 属于 Build Agent 执行证明。
- 明确 `output/release/` 属于 Build Agent 本地交付材料。
- 明确 `output/audit/` 属于 Audit Agent 未来审计输出，本轮只创建 skeleton，不启用真实 Audit Agent。

结果：

- 新增 `crates/output`，Cargo package 为 `agentflow-output`。
- 新增 Output canonical layout：
  - `.agentflow/output/manifest.json`
  - `.agentflow/output/index.json`
  - `.agentflow/output/evidence/`
  - `.agentflow/output/release/`
  - `.agentflow/output/audit/`
  - `.agentflow/output/logs/`
  - `.agentflow/output/backup/`
  - `.agentflow/output/cache/`
  - `.agentflow/output/tmp/`
- 新增 Output public API：
  - `prepare_output_workspace`
  - `validate_output`
  - `load_output_status`
  - `load_output_manifest`
  - `load_output_index`
  - `load_output_snapshot`
  - `load_output_evidence`
  - `load_release_delivery`
  - `create_audit_skeleton`
  - `load_audit_output`
- 新增 Output 模型：
  - `OutputManifest`
  - `OutputIndex`
  - `OutputStatusSnapshot`
  - `OutputSnapshot`
  - `OutputEvidence`
  - `OutputReleaseDelivery`
  - `OutputPrMetadata`
  - `OutputAudit`
- Evidence schema 已扩展为引用型证据包：
  - 引用 input issue / approved spec。
  - 引用 panel snapshot / context pack。
  - 引用 execute run / preflight / plan / result / checkpoint / diff / changed-files / diff-summary。
  - 引用 command record / stdout path / stderr path，不复制大 stdout / stderr 内容。
  - 记录 validation summary 和 manual proof 占位。
- Release delivery schema 已扩展：
  - `delivery.json` 引用 evidence、execute result 和 diff summary。
  - `pr-metadata.json` 明确 `createdRemotePr = false`。
  - `createdBy = Build Agent`。
  - 继续生成 `pr-draft.md`、`review-checklist.md`、`changelog.md`、`release-note.md`。
- Audit skeleton 已实现：
  - 写入 `output/audit/<run-id>/audit.json`。
  - 写入 `audit-report.md`、`findings.json`、`checklist.md`。
  - `status = pending`，不运行真实 audit。
- Execute prepare 已接入 Output prepare。
- Execute evidence / release 写入已改用 `agentflow-output` schema。
- Desktop Tauri 新增只读 Output commands：
  - `load_output_status`
  - `load_output_manifest`
  - `load_output_index`
  - `load_output_snapshot`
  - `validate_output`
- Desktop 状态通道新增“交付输出”事件，展示 Evidence、Delivery、Audit 和 Incomplete。
- Browser Preview mock 已同步 Output status。
- README / GOAL / ROADMAP / requirements index 已更新到 011。

边界：

- 未修改 input facts。
- 未修改 execute run facts 主流程。
- 未写用户源码。
- 未创建远程 PR。
- 未 merge。
- 未 deploy。
- 未调用模型。
- 未启用真实 Audit Agent。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-output`：pass，10 tests。
- `cargo test -p agentflow-execute`：pass，17 tests。
- `cargo test -p agentflow-desktop`：pass，17 tests。
- `cargo test`：pass，agent-manual 23 tests + CLI 2 tests + core 61 tests + desktop 17 tests + execute 17 tests + goal-tree 3 tests + input 8 tests + output 10 tests + panel 27 tests。
- `npm --prefix apps/desktop run build`：pass。
- `git diff --check`：pass。

## 2026-06-05 Workflow State / Gate Orchestration V1

执行者：Codex

目标：

- 将 `/Users/mac/Downloads/013-workflow-state-gate-orchestration-v1.md` 复制到 `docs/requirements/`。
- 新增 `state/` 总控状态层，聚合 define / panel / input / execute / output / audit 的健康状态。
- 输出 workflow gates、next actions、blockers、sessions、locks、events 和 indexes。
- 接入 Project Workspace prepare、Desktop Tauri commands 和 Desktop 状态通道。

结果：

- 新增 `crates/state`，Cargo package 为 `agentflow-state`。
- 新增 `.agentflow/state/**` 派生状态布局：
  - `manifest.json`
  - `index.json`
  - `status.json`
  - `health/workflow.json`
  - `gates/workflow.json`
  - `gates/next-actions.json`
  - `gates/blockers.json`
  - `locks/active.json`
  - `locks/stale.json`
  - `locks/cleanup-candidates.json`
  - `events/timeline.jsonl`
  - `indexes/workspaces.json`
  - `indexes/issues.json`
  - `indexes/runs.json`
  - `indexes/outputs.json`
- 新增 State public API：
  - `prepare_state_workspace`
  - `refresh_state`
  - `load_state_status`
  - `load_state_manifest`
  - `load_state_index`
  - `load_workflow_gates`
  - `load_next_actions`
  - `load_blockers`
  - `load_state_timeline`
  - `append_state_event`
  - `load_state_session`
  - `update_state_session`
  - `load_state_locks`
- 新增 workflow stage / audit status / workspace status 模型。
- 新增 health 聚合，覆盖 workspace / define / panel / input / execute / output / audit。
- 新增 gate 推导：
  - workspace missing / blocked / ready
  - panel ready
  - input ready
  - issue ready
  - execute ready / running / blocked / completed
  - evidence ready
  - delivery ready
  - audit requested / running / completed
- 新增 blockers 与 next actions 推导，高风险 issue 会生成 human confirmation blocker。
- 新增 locks 聚合：
  - Active lease + non-terminal run => active
  - Active lease + missing / terminal run => stale
  - Released lease => ignored
  - unreadable / corrupted lease => cleanup candidate
- 新增 sessions 和 timeline event 写入 / 读取能力。
- 新增状态 indexes：
  - workspace status
  - issue status
  - run status
  - output status
- Project Workspace prepare 已接入 State prepare。
- Desktop Tauri 新增 State commands。
- Desktop 状态通道新增“工作流状态”事件，展示阶段、下一步、阻断、审计和运行状态。
- Browser Preview 新增 workflow state mock 数据。
- README / GOAL / ROADMAP / requirements index 已更新到 013。

边界：

- State 只写 `.agentflow/state/**`。
- 未修改 input facts。
- 未修改 execute run facts 主流程。
- 未修改 output evidence / delivery / audit facts 主流程。
- 未写用户源码。
- 未创建远程 PR。
- 未 merge。
- 未 deploy。
- 未调用模型。
- 未新增 Desktop 执行动作。

验证：

- `cargo test -p agentflow-state`：pass，9 tests。
- `cargo fmt --check`：pass。
- `cargo test`：pass，agent-manual 23 tests + CLI 2 tests + core 61 tests + desktop 17 tests + execute 17 tests + goal-tree 3 tests + input 8 tests + output 18 tests + panel 27 tests + state 9 tests。
- `npm --prefix apps/desktop run build`：pass。

## 2026-06-05 Desktop Human Audit Entry Polish

执行者：Codex

目标：

- 将 `Desktop Human Audit Entry Polish` 补充为 `docs/requirements/012-1-desktop-human-audit-entry-polish.md`。
- 在 Desktop 中新增人类可见的“人工审计”入口。
- 支持选择 release delivery、填写必填 reason，并由 Desktop 自动生成 audit scope refs。
- 成功触发后加载 `audit-report.md`，并可展开查看 findings / checklist / evidence-map / traceability。
- Browser Preview 不执行真实 audit 写入。

结果：

- 新增 `OutputAuditPanel`：
  - 读取 `load_output_index`。
  - 读取 `load_audit_index`。
  - 读取 `load_audit_report`。
  - 真实 Tauri 客户端中调用 `request_human_audit`。
  - 浏览器预览中禁用请求按钮并显示 preview-only 提示。
- 新增前端 audit / output index 类型：
  - `OutputIndex`
  - `OutputIndexEntry`
  - `AuditIndex`
  - `AuditIndexEntry`
  - `HumanAuditRequestDraft`
  - `HumanAuditReport`
- Project 页面新增“交付输出 / 人工审计”可见入口。
- reason 为空时请求按钮禁用。
- 无 release delivery 时显示“暂无可审计交付材料”并禁用按钮。
- 请求成功后刷新 Output status，并默认展示最新 audit report。
- Browser Preview mock 新增 output index / audit index / audit report 数据函数，且不调用 `request_human_audit`。
- README / GOAL / ROADMAP / requirements index 已补充 012.1。

边界：

- 未改 audit 核心模型。
- 未自动触发 audit。
- 未在 execute / output 完成后自动审计。
- 未写 input facts。
- 未写 execute facts。
- 未写 evidence。
- 未写 release delivery。
- 未写用户源码。
- 未执行命令。
- 未创建远程 PR / merge / deploy。
- 未调用模型。

验证：

- `cargo fmt --check`：pass。
- `cargo test -p agentflow-desktop`：pass，17 tests。
- `cargo test`：pass，agent-manual 23 tests + CLI 2 tests + core 61 tests + desktop 17 tests + execute 17 tests + goal-tree 3 tests + input 8 tests + output 18 tests + panel 27 tests + state 9 tests。
- `npm --prefix apps/desktop run build`：pass。
- `git diff --check`：pass。
- Browser Preview 核对：尝试刷新 `http://127.0.0.1:1421/` 时被浏览器侧拦截，未完成可视核对；本轮以前端 build 和 mock 禁写逻辑验证为准。

## 2026-06-05 Browser Preview Verification Polish

执行者：Codex

目标：

- 将 `Browser Preview Verification Polish` 补充为 `docs/requirements/013-1-browser-preview-verification-polish.md`。
- 修复 PR #27 遗留的 Browser Preview 可视核对缺口。
- 让 `http://127.0.0.1:1421/` 能展示 Desktop 人工审计入口的可核对 mock 状态。

结果：

- Browser Preview output status 从空状态补齐为：
  - `evidence = 1`
  - `releaseDeliveries = 1`
  - `audits = 1`
  - `incompleteEvidence = 0`
  - `incompleteDeliveries = 0`
- Browser Preview output index 新增 1 条 evidence、1 条 release delivery、1 条 audit report entry。
- Browser Preview audit index 新增 `audit-browser-preview-001`。
- Browser Preview human audit report 新增只读 mock 报告：
  - `# Human Audit Browser Preview`
  - findings / checklist / evidence map / traceability 均可展示。
- Browser Preview workflow state 的 `auditStatus` 更新为 `passed-with-warnings`。
- README / GOAL / ROADMAP / requirements index 已补充 013.1。

边界：

- 未修改 `request_human_audit` Tauri 命令。
- 未修改 Rust output / audit / state 模型。
- 未修改 `.agentflow/output/audit` 真实写入逻辑。
- 未自动触发 audit。
- 未写 input / execute / evidence / release delivery 真实 facts。
- Browser Preview 仍只展示 mock 数据，不写 `.agentflow/output/audit`。

验证：

- `npm --prefix apps/desktop run build`：pass。
- `cargo fmt --check`：pass。
- `cargo test -p agentflow-desktop`：pass，17 tests。
- `cargo test`：pass，agent-manual 23 tests + CLI 2 tests + core 61 tests + desktop 17 tests + execute 17 tests + goal-tree 3 tests + input 8 tests + output 18 tests + panel 27 tests + state 9 tests。
- `git diff --check`：pass。
- Browser Preview 核对：pass。
  - URL：`http://127.0.0.1:1421/`。
  - 页面标题：`AgentFlow 本地工作台`。
  - DOM snapshot 显示 `region "人工审计"`。
  - 统计显示 `证据 1`、`交付 1`、`审计 1`、`未完成 0`。
  - `交付材料` combobox 选中 `run-browser-preview-001 · iss-001 · delivered`。
  - `请求人工审计` button 在 Browser Preview 中保持 disabled。
  - `最新审计报告` 显示 `audit-browser-preview-001`、`通过，有警告` 和 `# Human Audit Browser Preview`。
  - preview-only 提示显示 `浏览器预览不写 .agentflow/output/audit；请在 Tauri Desktop 中触发人工审计。`
  - Browser logs 仅包含 Vite debug 和 React DevTools info，无应用 error / warning。
  - in-app Browser screenshot 命令尝试过，但当前后端 `Page.captureScreenshot` 超时且 `playwright_element_screenshot` 不支持；本轮以 DOM snapshot 和 console logs 作为 Browser Preview 核对证据。

## 2026-06-05 Browser Preview Smoke Script

执行者：Codex

目标：

- 将 `Browser Preview Smoke Script` 补充为 `docs/requirements/013-2-browser-preview-smoke-script.md`。
- 为 PR #28 遗留的 Browser Preview 人工核对证据补上可重复执行的本地 smoke 命令。
- 用自动断言覆盖 Browser Preview output / audit / state mock、人工审计禁用边界和 `.agentflow/output/audit` 禁写边界。

结果：

- `apps/desktop/package.json` 提供 `preview:smoke`：
  - `npm --prefix apps/desktop run preview:smoke`
- `apps/desktop/scripts/browser-preview-smoke.mjs` 通过 Vite SSR 加载 Browser Preview mock factory。
- smoke 断言 output status：
  - `ready = true`
  - `evidence = 1`
  - `releaseDeliveries = 1`
  - `audits = 1`
  - `incompleteEvidence = 0`
  - `incompleteDeliveries = 0`
- smoke 断言 output / audit indexes：
  - release delivery run id 为 `run-browser-preview-001`
  - audit id 为 `audit-browser-preview-001`
- smoke 断言 human audit report 包含 `Human Audit Browser Preview`。
- smoke 断言 workflow state：
  - `currentStage = workspace-ready`
  - `auditStatus = passed-with-warnings`
- smoke 断言 `OutputAuditPanel.tsx`：
  - Browser Preview runtime 分支存在。
  - preview 分支注入 `createBrowserPreviewHumanAuditReport()`。
  - preview 分支设置 `source = "preview"`。
  - request disabled 条件包含 `previewOnly`。
  - preview-only guard 位于真实 `request_human_audit` invoke 之前。
  - preview-only 提示包含 `浏览器预览不写 .agentflow/output/audit`。
- smoke 断言临时项目中没有 `.agentflow/output/audit`。

边界：

- 未新增真实浏览器 DOM 自动化。
- 未新增 CI。
- 未修改 Tauri `request_human_audit` 命令。
- 未修改 Rust output / audit / state 核心模型。
- 未在 Browser Preview 写 `.agentflow/output/audit`。

验证：

- `npm --prefix apps/desktop run preview:smoke`：pass。
  - 输出：`Browser Preview smoke passed: workflow state and human audit preview are read-only.`
- `npm --prefix apps/desktop run build`：pass。
- `cargo fmt --check`：pass。
- `cargo test -p agentflow-workflow-acceptance`：pass，6 tests。
- `git diff --check`：pass。

## 2026-06-05 AgentFlow End-to-End Workflow Acceptance V1

执行者：Codex

目标：

- 将 `014 - AgentFlow End-to-End Workflow Acceptance V1` 复制到 `docs/requirements/014-agentflow-end-to-end-workflow-acceptance-v1.md`。
- 新增系统级端到端验收，证明 define / panel / input / execute / output / state / human audit 闭环可达。
- 使用临时 fixture 项目验证写入边界，确认 AgentFlow 流程后用户源码 hash 不变。

结果：

- 新增 `agentflow-workflow-acceptance` crate：
  - `full_workflow_reaches_delivery_ready`
  - `human_audit_request_updates_state`
  - `write_boundary_keeps_user_source_unchanged`
  - `high_risk_issue_creates_blocker`
  - `stale_lease_is_reported`
  - `browser_preview_smoke_contract_is_registered`
- fixture 项目只在初始化阶段创建：
  - `README.md`
  - `Cargo.toml`
  - `src/lib.rs`
- 验收链路覆盖：
  - Agent working manual / define prepare。
  - Panel prepare。
  - Input prepare、approved SPEC fixture、issue fixture。
  - Execute run、preflight、lease、plan、checkpoint、`printf ok` command、validation。
  - 空 changed-files summary，用于证明本轮执行无源码改动。
  - Output evidence 和 local release delivery draft。
  - State refresh 推导 `delivery-ready`，且 `nextActions` 包含 `request-human-audit` 和 `start-new-input`。
  - 人类显式 `request_human_audit` 生成完整 audit package。
  - `load_audit_report` 可读取 report / findings / checklist / evidence map / traceability。
  - Audit 后 state refresh 推导 `audit-completed`，`auditStatus != not-requested`。
  - 高风险 issue preflight blocker。
  - stale lease 分类。
- Desktop command wrapper 测试新增覆盖：
  - `load_state_status`
  - `load_workflow_gates`
  - `load_next_actions`
  - `load_blockers`
  - `load_output_status`
  - `load_output_index`
  - `request_human_audit`
  - `load_audit_index`
  - `load_audit_report`
- Browser Preview smoke 复用 013.2：
  - `npm --prefix apps/desktop run preview:smoke`
  - 验证 workflow state / output / audit preview mock 可读。
  - 验证 Browser Preview smoke 不写 `.agentflow/output/audit`。
- README / GOAL / ROADMAP / requirements index 已补充 014。

边界：

- 未新增 OpenSpec Authoring。
- 未新增 SPEC 编辑器。
- 未新增 Goal Tree materializer。
- 未新增 Agent 自动执行。
- 未调用模型。
- 未创建远程 PR。
- 未 merge。
- 未 deploy。
- 未执行用户项目测试命令。
- 未修改真实用户源码。
- 未自动触发 audit。
- 未改 state / audit / execute / output 核心模型。
- 测试写入只发生在临时 fixture 项目内。

验证：

- `npm --prefix apps/desktop run preview:smoke`：pass。
- `cargo test -p agentflow-workflow-acceptance`：pass，6 tests。
- `cargo test -p agentflow-desktop`：pass，18 tests。
- `cargo test -p agentflow-state`：pass，9 tests。
- `npm --prefix apps/desktop run build`：pass。
- `cargo test`：pass，agent-manual 23 tests + CLI 2 tests + core 61 tests + desktop 18 tests + execute 17 tests + goal-tree 3 tests + input 8 tests + output 18 tests + panel 27 tests + state 9 tests + workflow-acceptance 6 tests。
