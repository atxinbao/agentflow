# Changelog

## Unreleased

执行者：Codex

下一版修复目标暂定为 `0.6.1 - Release Closeout & Acceptance Gate Refinement`。

当前状态：计划中，尚未发布。

### Included

- 计划修复 `v0.6.0` 发布后暴露的 release metadata 漂移。
- 计划补齐 `0.6.0` CHANGELOG 与 `docs/v0.6.0/**` release closeout。
- 计划修复 release-gate 默认版本和 E2E fixture 仍指向 `v0.5.1` 的问题。
- 计划将 Evidence Gate 升级为 Acceptance Gate，覆盖 verification / evidence / contract / state。
- 计划定义 Completion Commit，明确 Event Store、status writeback、Projection refresh、Delivery record 的权威顺序。
- 计划保留 Done 后 optional audit trigger evaluation，但不自动触发 Audit。
- 计划生成 `v0.6.1` release audit certification。

### Architecture

- `v0.6.0` 保留为 Work Loop Handoff & Controlled Execution 功能发布。
- `v0.6.0` 当前只应视为 functional release，不应表述为 clean stable closeout。
- `v0.6.1` 收口 release closeout、Acceptance Gate、Completion Commit 和 Audit separation。

### Validation

- 待实现后补齐。

## 0.5.0 - 2026-06-20

执行者：Codex

AgentFlow 0.5.0 将系统从 Runtime Foundation 继续推进到 Spec Loop Productization。这个版本的重点不是再扩展执行面，而是把需求理解链路正式文件化、可追踪化，并把它稳定接到 Runtime Action Proposal。

### Included

- Spec Loop Filesystem Contract 正式落地：`intake / classification / context / boundary / route / preview / confirmation / materialization` 8 个阶段都有文件合同。
- Requirement Intake Normalizer、Classifier、Context Resolver、Boundary Checker、Route Decider、Preview Generator、Confirmation Gate、Spec Materializer 全链打通。
- Spec Materialization 不再只是写 project / issue；现在会同步生成 Spec-to-Action Proposal Bridge，证明 confirmed preview 可以进入 Runtime Foundation。
- Projection / Query 面新增 Spec Loop 专用只读视图，可统一读取阶段状态、文件链路、traceability 和 runtime action proposal 摘要。
- Acceptance 覆盖主链闭环：`Raw Human Request -> Runtime Action Proposal`，并验证文件链路和 traceability，不依赖人工点击。

### Architecture

- `docs/v0.5.0/**` 定义本版 Spec Loop Productization 目标与任务收口。
- `.agentflow/spec/requirements/<requirement-id>/**` 是 Spec Loop 阶段事实源。
- `.agentflow/projections/spec-loops/<requirement-id>.json` 是 Spec Loop 只读 Projection。
- `runtime-api` 继续作为 command / query 正式边界。
- Runtime authority 仍然来自 Event Store 与 spec facts，Projection 只读，不回写 authority。

### Validation

- `cargo fmt --all --check`
- `cargo test -p agentflow-projection`
- `cargo test -p agentflow-runtime-api`
- `cargo test -p agentflow-workflow-acceptance`
- `cargo check --workspace`
- `release-gate` on `main`
- `git diff --check`

## 0.4.0 - 2026-06-20

执行者：Codex

AgentFlow 0.4.0 完成了 `Definition-driven Runtime Foundation` 的正式收口，把运行时 authority、写前裁决、事件事实源和只读投影边界压成同一条主链。

### Included

- Ontology Registry、Action Contract、Agent Role Policy、Object State Machine 四层定义模型全部落地。
- Action Arbitration 成为唯一写前裁决口，统一处理 command -> proposal -> accepted-event 的运行时入口。
- Event Store、Projection、Runtime Command / Query API 正式收口，形成 Runtime Foundation 主干。
- 运行时角色统一到 `work-agent` 主别名，`build-agent` 仅保留兼容映射。
- Runtime Foundation closeout baseline 和 release-gate 验证链路完成，成为 `v0.5.0` Spec Loop 的正式地基。

### Architecture

- `Contract` 是 authority，不是 Agent。
- `Arbitration` 是唯一写前裁决口。
- `Event Store` 是唯一事实源。
- `Projection` 只读，不回写 authority。
- `Runtime API` 是对外正式 command / query 边界。

### Validation

- `release-gate` on PR
- `release-gate` on `main`
- closeout baseline in `docs/architecture/009-runtime-foundation-closeout-baseline-v1.md`

## 0.3.0 - 2026-06-19

执行者：Codex

AgentFlow 0.3.0 将系统从任务驱动实现继续推进为项目级 Agent Operating System，可让 Project Brain、Task Runtime、Provider Session、Release Gate 和 External Review 进入同一条正式闭环。

### Included

- Project Brain 正式接入 runtime，支持 Requirement -> Goal Preview -> Plan Preview -> Confirm -> Project / Issue materialization。
- Task / Project 工作台完成产品化，任务页右侧统一展示状态时间线、事件流、实时会话信息和公开交付摘要。
- Work / Audit / Delivery / Completion 四条 runtime 链路完成收口，任务完成后通过 completion / release 入口进入项目级关闭判断。
- Provider 执行面进入统一 `agent-dispatcher + mcp + session governance`，支持 Codex / Claude provider 能力注册、会话投影、打断恢复和 closeout attestation。
- `event-store / workflow-core / workflow-runtime / task-loop / task-artifacts / projection / release / audit` 组成新的底层主干，旧 `input / execute / output / workflow-events / core legacy` 活跃依赖已经移除。
- Release gate、trusted validation、atomic claim、closeout proof、workflow saga separation 等稳定性修补全部并入主线。

### Architecture

- `docs/product/**` 定义产品基线。
- `docs/foundation/**` 定义 Project / Goal / Plan / Loop / Audit / Completion 领域模型。
- `docs/architecture/**` 定义模块边界、事件、投影、workflow schema 和 release runtime。
- `docs/requirements/**` 记录当前版本需求切片。
- `.agentflow/spec/**` 仍是本地 project / issue 合同事实源。
- `.agentflow/events/**` 是唯一任务事件流。
- `.agentflow/tasks/<issue-id>/**` 只保留 run 与 evidence 本地产物。
- 公开交付记录通过 PR/MR body、`CHANGELOG.md` 和 release notes 对外暴露。

### Validation

- `cargo fmt --all --check`
- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `bash scripts/verify_release_gate.sh --artifact-dir /tmp/agentflow-v030-release-gate-fix`
- `git diff --check`

## 0.2.0 - 2026-06-17

执行者：Codex

AgentFlow 0.2.0 将工作流主线从旧 `input / execute / output` 分层收口到任务驱动架构。

### Included

- 新增 `spec / task-loop / task-artifacts / event-store / projection / release / agent-dispatcher / mcp` 等底层模块边界。
- 清理旧 `input / execute / output / core legacy / workflow-events / degraded fallback` 活跃依赖。
- 任务页成为主工作台，右侧展示状态时间线、事件流、实时会话信息和最终交付摘要。
- Build Agent loop 支持 `start / claim-launch / prepare-review / write-merge-proof / complete` 官方命令链路。
- Build Agent session 支持 `interrupted / resumed` 生命周期，并通过事件流写回投影。
- Project Loop 支持手动触发、按依赖顺序推进 issue，并在当前 issue Done 后尝试拉起下一条 issue。
- 公开交付记录写入 PR/MR、CHANGELOG 或 release notes，不再依赖旧 `.agentflow/output/**`。
- Browser Preview mock 和 smoke 覆盖任务状态流、公开交付和投影读取。

### Architecture

- `docs/requirements/**` 是公开需求记录。
- `.agentflow/spec/**` 是本地项目和 issue 合同事实源。
- `.agentflow/events/**` 是任务状态事件流。
- `.agentflow/tasks/<issue-id>/**` 是本地 run 与验证证据事实源。
- `.agentflow/projections/**` 和 `.agentflow/indexes/**` 是 Desktop 只读展示投影。

### Validation

- `npm --prefix apps/desktop run build`
- `cargo test --workspace`
- `git diff --check`

## 0.1.0 - 2026-06-07

执行者：Codex

AgentFlow 0.1.0 是第一个可交付本地版本。

### Included

- 本地项目接入和空项目工作台。
- Agent working manual、locale、voice style 和 workspace ownership guard。
- Project Panel、项目文件只读阅读器、任务流转、交付、审计和高级详情页面。
- Input / Execute / Output / State 工作流基础层。
- Browser Preview mock 和 smoke 验证。
- Desktop Design System V1 和统一 V16 UX。
- Base Release 初始化。
- Release Audit Trigger Rules V1。

### Release Audit

- Release Delivery 生成后，AgentFlow 自动登记 `release-auto` audit request。
- 同一个 Release Delivery 不重复创建 `release-auto` audit request。
- Desktop 普通 UI 只展示审计状态、触发来源、报告、发现项、证据链和追溯关系。
- Desktop 普通 UI 不创建审计请求。

### Validation

- `cargo fmt --check`
- `cargo check --workspace`
- `cargo test`
- `npm --prefix apps/desktop run build`
- `npm --prefix apps/desktop run preview:smoke`
- `cargo build --release -p agentflow-cli`
- `npm --prefix apps/desktop run tauri -- build`
- `git diff --check`
