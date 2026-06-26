# Changelog

## 1.0.1 - 2026-06-26

执行者：Codex

`1.0.1 - Release Hardening and Operational Certification` 是 `1.0.0` 之后的补丁版本。它不扩展产品功能，只补齐 v1 stable core 的发布证据、可复现性和运行可见性。

### Added

- 新增 `runtime/release-provenance.json`，记录 release version、tag、source commit、release URL、gate run、artifact manifest、certification artifact、release notes 和 tag signature 状态。
- 新增 `runtime/clean-room-test-proof.json`，证明 release gate 使用隔离 `CARGO_TARGET_DIR`，避免依赖人工 `cargo clean -p agentflow-pack`。
- 新增 `runtime/audit-sidecar-policy.json`，明确 Audit sidecar 不进入主发布链，严格模式与非严格模式的阻断语义分开表达。
- 新增 `runtime/provider-smoke-proof.json`，让 provider smoke 的 passed / failed / skipped / not configured 都有结构化 proof。
- 新增 `runtime/software-dev-pack-usage-baseline.json`，证明 Software Dev Pack 的真实使用链路仍以 Requirement、Spec、Issue、Run、Evidence、Acceptance、Delivery、Done 为主线。
- 新增 `runtime/trusted-governance-telemetry.json`，禁止 Runtime governance 从任意 request input 读取 provider capability 事实。
- 新增 `runtime/v101-release-certification.json`，作为 v1.0.1 最终发布认证 artifact。
- 新增 Message Bus no-go ADR 和 Software Dev Pack usage baseline 架构文档。

### Changed

- 版本元数据统一到 `1.0.1`，覆盖 Rust workspace、Desktop package、package-lock 和 Tauri 配置。
- 根目录 `AGENTS.md` 不再把 `docs/v0.9.1/README.md` 当作当前稳定入口，改为指向 v1 stable core、v1.0.1 hardening 和 release certification boundary。
- release gate 区分 PR / main / tag / release context：PR/main 可以记录 pending tag，tag/release 必须校验 tag commit 与 source commit 一致。
- provider smoke optional 不再被误读为 ready；只有真实 passed 才能写成 capability ready。

### Validation

- `cargo fmt --all --check`
- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `git diff --check`
- `bash scripts/verify_release_gate.sh --release-version v1.0.1 --release-tag v1.0.1`

## 1.0.0 - 2026-06-25

执行者：Codex

`1.0.0 - Project OS Stable Core` 将 AgentFlow 从 Runtime / Pack / Governance 分阶段基线推进到 v1 稳定核心认证。

### Added

- 新增 v1 Stable Contract Baseline、Runtime API / SDK Freeze、Filesystem Contract Freeze、Pack Contract Freeze、Projection / Read Model Freeze、Evidence / Acceptance Freeze、Executor Adapter Freeze、Replay / Migration / Upgrade Certification 和 Software Dev Pack Stable Baseline 的 release-gate 认证链。
- 新增 `runtime/v100-release-certification.json` 作为 v1.0.0 最终发布认证 artifact。
- release gate 输出 `v100Coverage`、`v100CoveragePassed`、`v1StableCore`、`v1StableCoreBlockers` 和 `v1SupportBoundary`。
- 明确 Software Dev Pack 是 v1 stable industry Pack，UI Design Pack 继续保持 experimental。

### Changed

- 版本元数据统一到 `1.0.0`，覆盖 Rust workspace、Desktop package、package-lock 和 Tauri 配置。
- release certification 将 Governance admission、Projection authority boundary、Acceptance Done decision、Audit sidecar separation 和 Executor truth boundary 纳入硬门禁。
- `docs/v1.0.0/` 从规划基线收口为 Project OS Stable Core release certification baseline。

### Validation

- `cargo fmt --all --check`
- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `git diff --check`
- `bash scripts/verify_release_gate.sh --release-version v1.0.0 --release-tag v1.0.0`

## 0.9.0 - 2026-06-25

执行者：Codex

`0.9.0 - Deployment Shape and Runtime Governance` 将 AgentFlow 从 Pack System remediation 基线推进到可部署、可重放、可迁移、可模拟、可治理的 Runtime 边界。

### Added

- 明确 Local Runtime Boundary 和 Cloud Runtime Boundary，固定 Runtime Core、API plane、Event Store、Projection、Pack、Connector 和行业 UI 的边界。
- 补齐 Runtime API / SDK contract hardening，确认所有运行时写动作必须经过 command admission 和 authority boundary。
- 增加 Event Replay / Projection Rebuild 证明，release gate 会输出 replay 与 projection rebuild 报告。
- 增加 Ontology / Pack Migration execution model，覆盖 migration preview、explicit apply、cancel、rollback 和 replay receipt。
- 增加 Simulation / Evaluation layer，确保 dry-run / simulation 只输出评估报告，不写 authority。
- 增加 Runtime Governance Policy，统一 role、capability、provider、sidecar、connector 和 audit boundary 的准入判断。
- 增加 Cross-process Scheduling Decision Gate，明确 Message Bus 当前结论为 `no-go`，不作为默认中心化架构。
- 增加 Deployment Evidence and Rollback Model，release gate 输出 deployment evidence、rollback readiness 和 external review surface。
- 增加 v0.9.0 release certification coverage，逐项证明 V090-001 到 V090-009，并给出 v1.0 planning readiness。

### Architecture

- Runtime Core owns command / event / state transitions。
- Industry clients own surface and interaction。
- Pack owns definitions。
- Connector owns external integration。
- Projection owns read-only views。
- Governance owns policy and admission decisions。
- Projection、Connector 和 industry UI 都不能升级为 authority。

### Validation

- `cargo fmt --all --check`
- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `git diff --check`
- `bash scripts/verify_release_gate.sh --release-version v0.9.0 --release-tag v0.9.0`
- GitHub `release-gate` on V090 closeout PRs

## 0.8.1 - 2026-06-23

执行者：Codex

`0.8.1 - Pack System File-backed Remediation` 修复 `0.8.0` 发布后暴露的 Pack System 事实源、命令解析、投影、能力状态和 release certification 缺口。

### Changed

- Pack Registry 改为 file-backed / fixture-backed source of truth，release gate 会证明 registry 不来自 built-in fallback。
- Runtime Pack Command Resolver 改为从 registry、domain、surface、connector 和 capability requirement 解析命令。
- Projection 改为加载 Pack-specific definitions，避免 custom Pack 回退成 Software Dev baseline。
- Pack command availability 接入 capability registry / provider smoke 状态，disabled capability 会给出不可用原因。
- Release summary 中 Audit sidecar 与 release gate 主结论分离表达，避免把旁路审计失败误读成 release failed。

### Fixed

- invalid Pack command 会在 submit 前被拒绝，并输出包含 stage / reason / Pack / command 的 rejected validation report。
- release gate 增加 Pack negative fixtures，覆盖 invalid Pack、missing read model、missing connector、disabled capability、invalid command submit 和 unexpected Software Dev fallback。
- release certification artifact 覆盖 Pack registry source、resolver、projection、capability/provider smoke、invalid submit rejection、negative fixtures 和 audit sidecar wording。

### Validation

- `cargo test --workspace`
- `cargo fmt --all --check`
- `npm --prefix apps/desktop run build`
- `git diff --check`
- `bash scripts/verify_release_gate.sh --release-version v0.8.1 --release-tag v0.8.1`
- GitHub `release-gate` on V081 remediation PRs

## 0.8.0 - 2026-06-23

执行者：Codex

`0.8.0 - Pack System and First Industry Shells` 将 AgentFlow 从单一软件开发项目控制台推进到 Pack 驱动的多行业项目运行基线。

### Added

- 新增 Pack filesystem contract、manifest schema、registry、validation、versioning 和 migration preview。
- 新增 Domain Pack、Surface Pack、Connector Pack 分层，明确 Pack 只定义行业现场，不直接写 authority。
- 新增 Pack simulation / dry-run，支持预览 Pack command 的影响，不写 `.agentflow/**` authority。
- 新增 Pack-aware Projection read models、Pack-aware Command Surface 和 Runtime API Plane entry。
- 新增 Software Dev Pack baseline，主链保持 Requirement -> Spec -> Issue -> Run -> Acceptance -> Delivery -> Release。
- 新增 UI Design Pack baseline，主链覆盖 Product Brief -> Direction -> Wireframe -> HiFi -> Design System -> Handoff。
- release-gate 增加 Pack System readiness certification，输出 pack registry、validation、simulation、projection readiness、API plane manifest、Software Dev readiness 和 UI Design readiness artifacts。

### Architecture

- Runtime Core 仍然是 Spec Loop、Work Loop、Arbitration、Event Store、Projection 的通用主链。
- Pack 只能通过 Runtime API / Command Surface 接入，不能直接写 `.agentflow/spec/**`、`.agentflow/events/**` 或 `.agentflow/tasks/**`。
- Software Dev Pack 中 Audit 仍是 sidecar，`Finding` 不阻断主链 release，只能回流为 follow-up proposal。
- UI Design Pack 是 baseline readiness，不伪装成软件任务链，也不触发代码执行。

### Validation

- `cargo test --workspace`
- `cargo fmt --all --check`
- `npm --prefix apps/desktop run build`
- `git diff --check`
- `bash scripts/verify_release_gate.sh --artifact-dir /tmp/af-release-gate-test`
- GitHub `release-gate` on V080 closeout PRs

## 0.7.2 - 2026-06-23

执行者：Codex

`0.7.2 - Runtime Foundation Hardening` 收口 `v0.7.x` Console 之后暴露的底层运行时边界，为后续 Pack / Industry Shell 提供可验证底座。

### Added

- 新增 `docs/v0.7.2/**`，固定 Runtime Foundation hardening 任务基线和 V072 issue 顺序。
- 新增 `crates/message-bus` 和 Local Message Bus 架构文档，固定本地 runtime / projection / command / worker / audit channel，明确 bus 只做 fanout / refresh signal，Event Store 仍是 durable authority。
- 新增 `crates/capability-registry` 和 Worker / Tool Capability Registry 架构文档，提供 worker/tool 列表、health、capability、requiresAuth 和 Command Surface disabled reason 判断。
- 新增 `agentflow-mcp` provider smoke gate，支持最小 provider health / launch request / session snapshot / terminal projection 证明，并通过 `agentflow provider-smoke` 暴露 CLI 入口。
- 新增 Connector / MCP boundary baseline，在 provider profile 和 capability registry 中显式暴露 read/write capability、authority write 禁止、Runtime Command 必经和输出落点。
- 新增 Runtime / Projection / Command API Plane manifest，覆盖 runtime_commands、projection_queries、command_surface_actions、connector_actions、provider_actions、audit_actions 和 release_actions，并接入 Desktop Advanced 与 release-gate 检查。
- 新增 v0.7.2 foundation readiness report，明确 Runtime Foundation 的 completed / baseline / deferred / v0.8.0 carryover 状态。
- release-gate 增加 v0.7.2 foundation coverage，输出 foundation-readiness-report、foundation-coverage、api-plane-manifest 和 capability-registry runtime artifact。
- 新增 `crates/simulation` 和 Simulation Dry-run Runtime 架构文档，提供 command / issue / completion 的只读 dry-run 报告、expected events、rejected reasons、affected projections、risk / conflict 和 gate impact。
- 新增 `crates/schema-registry` 和 schema migration 架构文档，提供当前 schema version 清单、legacy 检测、migration preview 和显式 apply 边界。
- 新增 `docs/v0.7.1/**`，固定 `v0.7.0` release certification evidence，包括 PR / main / tag / release event gate run、artifact、release URL 和 source commit。
- release-gate 增加明确的 Browser Preview smoke 和 Console readiness 步骤，并把两个步骤的 status JSON 写入 gate artifact。
- Desktop Tauri projection command 增加真实临时 workspace readiness 测试，覆盖 `.agentflow/spec/**`、`.agentflow/events/**`、`.agentflow/tasks/**` 和 `.agentflow/projections/**` 的真实读路径。
- release-gate certification 增加 `runtime-fixture-gate` / `provider-smoke-gate` 边界，避免把本地 fixture E2E 误表述为真实 provider production E2E。

### Changed

- `docs/v0.7.0/**` 从 planning draft 口径收口为 released implementation record。
- `v0.7.0` 验证说明改为显式列出 `preview:smoke`、`console:readiness` 和 release-gate 版本认证命令。
- `scripts/verify_release_gate.sh` 的 summary / certification payload 改用 `runtime-fixture-gate` 命名，并把 `provider-smoke-gate` 接入为默认 clear skip、显式执行的独立门禁。
- Project canonical workflow 不再把 Audit 放在 Work Done 和 Delivery 之间；Audit 改为独立 Sidecar Loop，finding 只能回流为 Follow-up Proposal。

## 0.7.0 - 2026-06-22

执行者：Codex

`0.7.0 - Projection Surface & Project OS Console` 将 `v0.4.0` 到 `v0.6.1` 形成的 Runtime / Spec Loop / Work Loop / Acceptance / Delivery / Audit 事实链，整理成可读、只读、可回归验证的项目控制台。

当前状态：Project OS Console release readiness 已完成，`v0.8.0` 可以基于该 Console 基线进入 Pack System。

### Included

- 定义 Projection Surface contract 和 Console information architecture，明确 Projection / View Model / Console 都不是 authority。
- 增加 Projection Query API 和统一 read model，供 Desktop 读取 Project / Spec / Task / Audit / Delivery 状态。
- 重构 Project Home、Spec Workbench、Task Workbench，形成项目阶段、需求链路、任务状态流和下一步入口。
- 将 Event Timeline、Evidence Graph、Acceptance / Delivery、Audit read-only、Command Surface 纳入任务工作面。
- 增加 Advanced Runtime Diagnostics，把 projection freshness、runtime status、event replay、provider sessions、role policy、missing facts、stale projection 和 conflict diagnostics 收进只读高级页面。
- 增加 Desktop Projection View Models 和 Browser Preview 回归覆盖。
- 增加 Project OS Console readiness evidence，证明软件开发场景可从 Project -> Spec -> Task -> Work -> Acceptance -> Delivery -> Audit read-only -> Command Surface 完成可读闭环。

### Architecture

- Console 只消费 Event Store / Spec Facts / Task Facts / Audit Facts 形成的 projection 和 view model。
- UI 不直接写 `.agentflow/**`。
- Command Surface 只进入 Runtime API / Action Proposal，不绕过 arbitration 或 authority boundary。
- Audit Surface 只读独立审计事实，不默认参与任务 Done。
- `docs/v0.7.0/**` 是 Project OS Console 的 release readiness baseline。

### Validation

- `cargo fmt --all --check`
- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `npm --prefix apps/desktop run preview:smoke`
- `npm --prefix apps/desktop run console:readiness`
- `bash scripts/verify_release_gate.sh --release-version v0.7.0 --release-tag v0.7.0`
- `release-gate` on PR / main for V070 closeout PRs

## 0.6.1 - 2026-06-22

执行者：Codex

`0.6.1 - Release Closeout & Acceptance Gate Refinement` 修复 `v0.6.0` 发布后暴露的 release hygiene 和验收闭环问题。

当前状态：修复链已实现，release audit certification 已补齐，作为 `v0.6.0` 之后的 clean remediation release。

### Included

- 修复 `v0.6.0` 发布后暴露的 release metadata 漂移。
- 补齐 `0.6.0` CHANGELOG 与 `docs/v0.6.0/**` release closeout。
- 修复 release-gate 默认版本和 E2E fixture 仍指向 `v0.5.1` 的问题。
- 将 Evidence Gate 升级为 Acceptance Gate，覆盖 verification / evidence / contract / state。
- 定义 Completion Commit，明确 Event Store、status writeback、Projection refresh、Delivery record 的权威顺序。
- 保留 Done 后 optional audit trigger evaluation，但不自动触发 Audit。
- 生成 `v0.6.1` release audit certification。

### Architecture

- `v0.6.0` 保留为 Work Loop Handoff & Controlled Execution 功能发布。
- `v0.6.0` 当前只应视为 functional release，不应表述为 clean stable closeout。
- `v0.6.1` 已收口 release closeout、Acceptance Gate、Completion Commit 和 Audit separation。

### Validation

- `cargo fmt --all --check`
- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `git diff --check`
- `release-gate` on PR / main for V061 closeout PRs

## 0.6.0 - 2026-06-21

执行者：Codex

AgentFlow 0.6.0 将 `Spec Loop -> Work Loop` 的交接链路推进为受 Runtime 管控的执行闭环。这个版本保留为功能发布基线，但不作为 clean stable closeout；发布后发现的版本元数据、文档状态、release-gate 默认版本和 Acceptance Gate 收口问题进入 `v0.6.1` 修复链。

### Included

- Work Loop 文件合同和 CodeFlow 边界落地，确认后的 spec issue 可以被转换为 Work Command。
- Work / Build Agent 的关键写动作进入 Action Proposal 和 Arbitration，不再直接写状态。
- Issue preflight、lock / lease、dependency queue、work session、verification、evidence、Acceptance Gate 和 Completion Commit 主链完成。
- Work Loop 事件和 Projection 可以重建执行状态，Projection 仍保持只读。
- Done 写回和 optional audit trigger evaluation 分离，Done 不自动创建 Audit issue。
- release-gate E2E 覆盖 requirement -> project -> task loop -> completion -> release -> audit request 的 runtime 链路。

### Architecture

- `docs/v0.6.0/**` 记录 Work Loop Handoff & Controlled Execution 的功能发布基线。
- `.agentflow/spec/**` 继续作为 spec issue 和 project 合同事实源。
- `.agentflow/events/**`、`.agentflow/tasks/**`、`.agentflow/projections/**` 承载执行事件、任务证据和只读投影。
- Acceptance Gate 当前已经形成主链，但还需要在 `v0.6.1` 中细化为 verification / evidence / contract / state 四个子 gate。
- Completion Commit 已进入主链，但 authority write order 和 release closeout 证明仍在 `v0.6.1` 中继续收紧。

### Validation

- `cargo fmt --all --check`
- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `bash scripts/verify_release_gate.sh --release-version v0.6.0 --release-tag v0.6.0`
- `release-gate` on `main`
- `release-gate` on tag `v0.6.0`
- GitHub Release `v0.6.0`

### Carry-over to 0.6.1

- Release metadata drift must be fixed across Cargo, Desktop package metadata, Tauri config, package lock and release-gate defaults.
- `docs/v0.5.1/**` and `docs/v0.6.0/**` must no longer contradict the published `v0.6.0` fact.
- release-gate must certify version consistency and release facts before future releases.
- Acceptance Gate and Completion Commit must become explicit authority records.

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
