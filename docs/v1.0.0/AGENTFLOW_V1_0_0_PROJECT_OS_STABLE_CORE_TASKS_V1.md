# AgentFlow v1.0.0 Project OS Stable Core Tasks V1

日期：2026-06-25
执行者：Codex

## Goal

`v1.0.0` 的目标不是继续扩功能，而是冻结 AgentFlow 的 Project OS 稳定核心。

主线是：

```text
Stable Contract Baseline
-> Runtime API / SDK Freeze
-> Filesystem Contract Freeze
-> Pack Contract Freeze
-> Projection Contract Freeze
-> Evidence / Acceptance Contract Freeze
-> Executor Adapter Contract Freeze
-> Replay / Migration / Upgrade Certification
-> Software Dev Pack Stable Baseline
-> v1.0.0 Release Certification
```

本版本要回答的是：

```text
AgentFlow 的底层 Project OS 合约是否已经稳定到可以支撑后续行业壳接入？
```

## Entry Gate

`v1.0.0` 有硬前置条件。

必须先完成 `v0.9.1` release certification，并得到：

```text
v1PlanningReadiness = ready
```

如果 `v0.9.1` 仍存在以下任一问题，`v1.0.0` 不允许进入执行：

- Governance 仍只是独立 report，没有进入 Runtime command admission；
- Deployment Evidence 仍只检查文件存在或 sha256；
- Pack migration receipt 仍伪装成 authority mutation；
- project `.agentflow/packs/**` path 仍没有 release gate proof；
- release source archive 仍没有自洽 Agent entry；
- negative semantic fixtures 仍不能阻断错误 happy path。

## Product Principle

`v1.0.0` 是稳定承诺，不是功能竞赛。

正确方向是：

```text
冻结核心合约。
明确兼容边界。
证明可复跑。
保留 Audit sidecar。
让 executor 被 AgentFlow 约束，而不是反过来拥有项目真相。
```

## Issues

| Issue | Title | Priority | Dependency | Status |
| --- | --- | --- | --- | --- |
| `V100-001` | Stable Contract Baseline | P0 | v0.9.1 certification ready | planned |
| `V100-002` | Runtime API / SDK Freeze | P0 | V100-001 | planned |
| `V100-003` | AgentFlow Filesystem Contract Freeze | P0 | V100-001 | planned |
| `V100-004` | Pack Contract Freeze | P0 | V100-001, V100-003 | planned |
| `V100-005` | Projection / Read Model Stable Contract | P0 | V100-002, V100-004 | planned |
| `V100-006` | Evidence + Acceptance Stable Contract | P0 | V100-002, V100-005 | planned |
| `V100-007` | Executor Adapter Stable Contract | P0 | V100-002, V100-006 | planned |
| `V100-008` | Replay / Migration / Upgrade Certification | P0 | V100-003, V100-004, V100-005, V100-006 | planned |
| `V100-009` | Software Dev Pack Stable Baseline | P1 | V100-004, V100-005, V100-006, V100-007 | planned |
| `V100-010` | v1.0.0 Release Certification | P0 | V100-001, V100-002, V100-003, V100-004, V100-005, V100-006, V100-007, V100-008, V100-009 | planned |

## V100-001 Stable Contract Baseline

### Scope

固化 v1.0 的稳定边界。

必须定义哪些对象在 v1 之后承诺兼容，哪些仍然是 internal implementation detail。

必须处理：

- stable public contract 清单；
- internal runtime implementation 清单；
- compatibility promise；
- breaking-change rule；
- deprecation rule；
- version field rule；
- release certification rule；
- v1 后新增能力不能破坏主链路。

### Acceptance

- 有一份 v1 stable contract baseline 文档；
- 文档明确 stable / internal / experimental 三类边界；
- 所有后续 V100 issue 都引用这份 baseline；
- release gate 能检查 stable contract version metadata；
- 如果缺 baseline，V100-002 到 V100-010 不允许进入 Done。

### Non-goals

- 不承诺所有历史草案兼容；
- 不把内部函数、临时 CLI 输出、debug fixture 全部变成 stable API。

## V100-002 Runtime API / SDK Freeze

### Scope

冻结 command / query / event API。

必须处理：

- command input contract；
- query input contract；
- event output contract；
- decision output contract；
- error model；
- version field；
- governance admission decision；
- accepted / rejected / deferred / failed 状态语义；
- Runtime API 与 CLI command 的关系；
- SDK 使用者能依赖的最小稳定面。

### Acceptance

- Runtime API / SDK contract 有稳定 schema；
- command path 不能绕过 Governance admission；
- rejected / deferred 不写 proposal 或 accepted event；
- error response 有稳定 code、stage、reason、evidence path；
- SDK 示例覆盖 command、query、event 三条路径；
- release gate 覆盖 API compatibility fixture。

### Non-goals

- 不冻结每个内部 Rust 函数签名；
- 不做多语言 SDK 全量实现；
- 不绑定特定云 API 网关。

## V100-003 AgentFlow Filesystem Contract Freeze

### Scope

固化 `.agentflow/` 文件系统协议。

必须处理：

- project facts；
- packs；
- spec projects；
- spec issues；
- tasks；
- runs；
- events；
- evidence；
- reports；
- local-only tmp；
- ignored runtime artifacts；
- source archive 必须包含的 Agent entry；
- 哪些路径是 authority，哪些路径是 projection，哪些路径是 local cache。

### Acceptance

- `.agentflow/` stable path contract 完整列出；
- 每个路径都有 owner、read/write rule、authority level、version rule；
- release source archive 与 local runtime state 的边界清楚；
- 禁止 retired path 被重新写入；
- release gate 覆盖 filesystem contract fixture。

### Non-goals

- 不把所有本地运行事实提交到 git；
- 不恢复 retired `.agentflow/input/**`、`.agentflow/output/**`、`.agentflow/goal-tree/**`。

## V100-004 Pack Contract Freeze

### Scope

冻结 Domain Pack / Surface Pack / Connector Pack 的 schema、version、capability、migration 规则。

必须处理：

- Pack manifest；
- Domain object / link / action definitions；
- Surface read model / view model definitions；
- Connector capability definitions；
- capability status；
- provider smoke binding；
- Pack fingerprint；
- Pack migration metadata；
- compatibility and migration rule；
- invalid / deferred Pack 状态。

### Acceptance

- Pack schema 有 stable version；
- project `.agentflow/packs/**` 是可验证 path；
- invalid Pack command 不能进入 Runtime proposal；
- disabled capability 不能被 command resolver 当成 available；
- Pack migration 不混淆 receipt-only 和 authority-applied；
- release gate 覆盖 Software Dev Pack 和 UI Design Pack fixtures。

### Non-goals

- 不做 Pack marketplace；
- 不承诺所有未来行业 Pack 已完成；
- 不把 UI Design Pack 提升为 v1 默认稳定行业壳。

## V100-005 Projection / Read Model Stable Contract

### Scope

固化 Projection API、Read Model、View Model。

原则：

```text
行业客户端只读 Projection。
Projection 不拥有 authority。
UI 不直接读 Event Store 写路径。
```

必须处理：

- Projection API；
- read model schema；
- view model schema；
- projection rebuild rule；
- stale / invalid / deferred 状态；
- Pack-specific projection loading；
- evidence graph read model；
- audit sidecar read model；
- delivery read model。

### Acceptance

- Projection schema 有 stable version；
- Projection 可以从 Event Store 重建；
- Projection missing Pack definition 时显示 invalid / deferred，不静默回退 Software Dev；
- Industry Surface 只能消费 Projection / Read Model；
- release gate 覆盖 projection rebuild compatibility fixture。

### Non-goals

- 不做全新 UI 大改版；
- 不把 Projection 变成写入 authority；
- 不要求每个行业壳都完成。

## V100-006 Evidence + Acceptance Stable Contract

### Scope

把验证、证据、验收、完成写入做成稳定闭环。

主链：

```text
Confirmed Work
-> Admission
-> Execution
-> Verification
-> Evidence Pack
-> Acceptance Gate
-> Completion Commit
-> Done
```

必须处理：

- verification result；
- evidence pack；
- acceptance criteria；
- acceptance decision；
- failure reasons；
- completion commit；
- event append；
- issue / run status writeback；
- delivery record；
- Audit trigger evaluation 只作为 sidecar 判断，不阻断默认 Done。

### Acceptance

- Acceptance Gate 汇总 Verification Gate、Evidence Gate、Contract Gate、State Gate；
- Done 只能由 Acceptance Decision passed 后进入；
- Completion Commit 是唯一完成写入边界；
- failed acceptance 有稳定 reason 和 evidence path；
- Audit 不进入主业务链；
- release gate 覆盖 pass / fail / missing evidence / state blocked fixtures。

### Non-goals

- 不把 Audit Agent 变成默认阻断流程；
- 不把人工审查当成唯一验收依据；
- 不把 CI 当成唯一验证 authority。

## V100-007 Executor Adapter Stable Contract

### Scope

固化 Codex / Claude Code 等执行器适配合同。

AgentFlow 管：

- 当前 issue；
- role；
- allowed surface；
- non-goals；
- acceptance criteria；
- expected outputs；
- evidence policy；
- completion writeback。

Executor 管：

- model call；
- tool call；
- shell / file edit；
- local session；
- context window；
- provider behavior。

必须处理：

- work handoff schema；
- allowed path / denied path；
- expected outputs；
- evidence return；
- diff boundary check；
- session isolation guidance；
- executor result normalization；
- executor runtime 不能成为 project truth。

### Acceptance

- Executor adapter contract 有 stable schema；
- Codex / Claude Code handoff 都能映射到同一 AgentFlow task contract；
- executor 越界修改会被 post-run validation 拒绝推进状态；
- executor session / memory 不被当成 AgentFlow authority；
- release gate 覆盖 accepted / rejected executor result fixtures。

### Non-goals

- 不重写 Codex / Claude Code 内部 runtime；
- 不保证所有第三方 executor 行为完全一致；
- 不把 executor 的 chat history 当成项目事实。

## V100-008 Replay / Migration / Upgrade Certification

### Scope

证明 event replay、projection rebuild、Pack migration、旧版本升级路径可复跑。

必须处理：

- event replay；
- projection rebuild；
- Pack migration apply / rollback；
- filesystem contract migration；
- upgrade guide；
- rollback guide；
- semantic fixture；
- negative fixture；
- deterministic report。

### Acceptance

- 至少覆盖 v0.9.x 到 v1.0.0 的 upgrade path；
- replay 后 Projection 与 expected read model 一致；
- migration receipt 和 authority-applied 状态可区分；
- rollback target 语义可验证；
- negative upgrade fixture 能在正确 stage 失败；
- certification report 可复跑。

### Non-goals

- 不承诺所有早期实验版本自动升级；
- 不做复杂数据库迁移平台；
- 不隐藏破坏性变更。

## V100-009 Software Dev Pack Stable Baseline

### Scope

把 Software Dev Pack 作为 v1.0 默认稳定行业壳。

必须证明软件开发现场可闭环：

```text
Requirement
-> Spec
-> Issue
-> Run
-> Evidence
-> Acceptance
-> Delivery
-> Optional Audit sidecar
```

必须处理：

- Software Dev Domain Pack；
- Software Dev Surface Pack；
- Software Dev Connector Pack；
- Git / GitHub / executor connector baseline；
- Project Home / Task Workbench / Delivery / Audit sidecar read model；
- Release flow；
- Audit 仍然独立。

### Acceptance

- Software Dev Pack 有 stable manifest；
- Software Dev read models 不直接读 authority write path；
- GitHub issue 仍只是临时协作镜像，不成为 AgentFlow authority；
- Delivery 和 Audit sidecar 的边界可验证；
- 至少一条 Software Dev fixture 从 intake 到 Done 可复跑。

### Non-goals

- 不把 UI Design Pack 作为 v1 stable 要求；
- 不做所有行业壳；
- 不把 Audit 并回主业务链。

## V100-010 v1.0.0 Release Certification

### Scope

最终发布认证。

必须输出：

- stable contract baseline proof；
- Runtime API / SDK compatibility proof；
- filesystem contract proof；
- Pack contract proof；
- Projection / Read Model proof；
- Evidence / Acceptance proof；
- Executor Adapter proof；
- replay / migration / upgrade proof；
- Software Dev Pack stable proof；
- negative fixture coverage；
- remaining risk / deferred list；
- v1 support boundary。

### Acceptance

- V100-001 到 V100-009 都有 release gate coverage；
- certification 明确 `v1StableCore = ready | blocked`；
- 如果 Governance admission 不在主链，必须 blocked；
- 如果 Projection 仍能绕过 authority 边界，必须 blocked；
- 如果 Acceptance 不能决定 Done，必须 blocked；
- 如果 Audit 被放回主业务链，必须 blocked；
- 如果 executor runtime 被当成 project truth，必须 blocked；
- 如果 v1 compatibility boundary 不清楚，必须 blocked。

### Non-goals

- 不替代独立 Audit Agent 流程；
- 不承诺长期商业 SLA；
- 不承诺所有 future Pack 兼容；
- 不把 v1.0.0 当成行业市场发布。

## Execution Order

建议执行顺序：

```text
V100-001
-> V100-002
-> V100-003
-> V100-004
-> V100-005
-> V100-006
-> V100-007
-> V100-008
-> V100-009
-> V100-010
```

`V100-010` 必须最后执行。

## Stable Core Completion Target

`v1.0.0` 完成后，AgentFlow 底层能力应达到：

```text
Core Project OS capability: about 80%
```

这个 80% 表示：

- 软件开发 AgentFlow 可以稳定运行；
- 第二、第三个行业壳可以基于 Pack contract 接入试点；
- Runtime / Pack / Projection / Evidence / Acceptance / Executor 的边界稳定；
- 后续能力扩展不能破坏 v1 stable core。

剩余能力进入 v1.x：

- 多行业规模化；
- 云端 Runtime 产品化；
- 跨进程调度和 Message Bus 决策；
- OS Console 深化；
- 第三方 Pack / Skill / Connector 生态；
- 长期兼容与 deprecation 流程。
