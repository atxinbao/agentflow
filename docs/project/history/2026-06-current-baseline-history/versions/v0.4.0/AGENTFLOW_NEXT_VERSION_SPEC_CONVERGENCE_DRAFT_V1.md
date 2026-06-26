# AgentFlow Next Version SPEC Convergence Draft V1

日期：2026-06-19
执行者：Codex
用途：Next Version Development Preflight / 正式 SPEC 前收敛草案
状态：Architecture Draft / 非执行需求 / 不进入当前 v0.3.0 审计 / 不授权 Build Agent 执行

关联架构基线：

- [AGENTFLOW_AGENT_PROJECT_OS_ARCHITECTURE_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AGENT_PROJECT_OS_ARCHITECTURE_V1.md)
- [AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md)
- [AGENTFLOW_ACTION_CONTRACT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ACTION_CONTRACT_DRAFT_V1.md)
- [AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md)
- [AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md)
- [AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md)
- [AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md)

说明：本文件用于把下一版本 Agent Project OS 架构收敛成正式 SPEC Draft Preview 前的统一口径。它不创建 `.agentflow/spec/**` 任务事实，不授权执行，也不替代正式 SPEC。

## 1. Conclusion

下一版本不应该尝试“一次性实现完整跨行业 Agent Project OS”。

应该先实现一个可落地的 Runtime Foundation：

```text
Project Ontology
→ Action Contract
→ Agent Role Policy
→ Object State Machine
→ Event Store
→ Projection Surface
```

这 6 个模块形成最小闭环。
行业客户端、云端部署、多行业 Pack、复杂 Message Bus、多 Agent 并发优化都应该放到后续版本。

## 2. Source Documents

本次收敛读取并对齐以下文档：

| document | role |
| --- | --- |
| `AGENTFLOW_AGENT_PROJECT_OS_ARCHITECTURE_V1.md` | OS 总架构 |
| `AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md` | 定义层 schema |
| `AGENTFLOW_ACTION_CONTRACT_DRAFT_V1.md` | Action 合约 |
| `AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md` | Agent 能力边界 |
| `AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md` | 对象生命周期 |
| `AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md` | 不可摇摆的架构决策 |
| `docs/architecture/001-project-operating-system-v1.md` | 旧版 OS 蓝图 |
| `docs/architecture/002-agent-capability-matrix-v1.md` | 旧版能力矩阵 |
| `docs/architecture/003-workflow-schema-v1.md` | 旧版 workflow schema |
| `docs/architecture/004-event-and-projection-model-v1.md` | 旧版事件和投影模型 |

## 3. Canonical Terminology

下一版本 SPEC 应统一使用以下术语。

| canonical term | meaning |
| --- | --- |
| `Agent Project OS` | AgentFlow 的系统定位，不是普通 workflow tool |
| `Runtime Core` | 稳定内部主链路 |
| `Project Ontology Layer` | 项目世界定义层 |
| `Object Type` | 项目对象类型，如 Requirement / Spec / Issue |
| `Link Type` | 对象之间的合法关系 |
| `Action Type` | Runtime 可识别的动作类型 |
| `Action Contract` | Action 的输入、前置条件、效果、证据、回滚规则 |
| `Action Proposal` | Agent 或 UI 提交的待仲裁动作 |
| `Agent Role Policy` | Agent 角色能力合约 |
| `Object State Machine` | 单个对象的生命周期定义 |
| `Event Store` | 唯一事实权威 |
| `Projection Surface` | 面向 UI / CLI / 行业客户端的只读投影和命令入口 |
| `Runtime API` | Command / Query / Event 的标准入口 |
| `Domain Pack` | 行业对象、术语、规则 |
| `Surface Pack` | 行业页面和交互语义 |
| `Connector Pack` | 外部工具、系统、数据源适配 |

## 4. Terminology Mapping

旧文档中的术语不直接删除，但进入下一版本 SPEC 时需要映射。

| old / mixed term | canonical term | decision |
| --- | --- | --- |
| `Workflow State` | `Object State Machine` + `Projection State` | workflow 不再是唯一状态来源 |
| `Work Flow` | `Issue State Machine` + `Run State Machine` | work 拆成可执行契约和执行尝试 |
| `Work Agent` | `BuildAgent` | 统一为 Build Agent |
| `AuditFinding` | `Finding` | Finding 可带 `source=audit` |
| `Goal` | `Requirement` / `Decision` | Goal 不作为下一版核心对象名 |
| `Delivery Flow` | `Delivery Package` / `Projection Surface` | 交付优先作为投影和打包结果 |
| `Capability Matrix` | `Agent Role Policy` | 能力矩阵升级为可仲裁 policy |
| `Workflow Action` | `Action Type` / `Action Contract` | action 必须有合约 |
| `Event Model` | `Event Store` + `Event Envelope` | 事件模型成为事实层 |
| `Task Projection` | `Projection Read Model` | 页面读模型来自 Projection |

## 5. Concept Deduplication

### 5.1 Workflow vs Object State Machine

保留两者，但职责不同。

```text
Object State Machine = 对象能处于什么状态、如何合法转移
Workflow = Runtime 如何编排阶段、handoff、pause/resume/retry/cancel
Projection State = UI 看到的组合状态
```

下一版本 MVP 优先实现 Object State Machine。
Workflow Schema 作为后续增强，不进入第一批核心实现。

### 5.2 Event vs Message

必须区分：

```text
Event = 已发生事实
Message = 调度通知
Command = 请求系统做事
Action Proposal = 等待 Runtime 仲裁的动作
```

下一版本 MVP 必须实现 Event Store。
Message Bus 可以先用同步调用或轻量队列替代，不进入核心范围。

### 5.3 Issue vs Run

必须拆开：

```text
Issue = 可执行工作契约
Run = Agent 执行 Issue 的一次尝试
```

`Run.completed` 不等于 `Issue.done`。
`Issue.done` 必须通过 Action Contract 和 State Machine 进入。

### 5.4 Audit vs Build

必须拆开：

```text
Build = 按 Issue Contract 交付
Audit = 独立审计 evidence 和交付事实
Finding = 审计或评审发现
Fix Issue = Finding 派生的修复工作契约
```

Build Done 不自动触发 Audit。
Audit Finding 不直接改写 Build 事实。

### 5.5 Role vs Prompt

必须拆开：

```text
Prompt = 行为引导
Agent Role Policy = Runtime 可执行的能力边界
```

下一版本 MVP 只接受 Role Policy 作为权限输入。
Prompt 不能覆盖或绕过 Role Policy。

## 6. Next Version MVP Scope

下一版本 MVP 的目标是建立 Runtime Foundation，而不是完整行业商业化产品。

### 6.1 In Scope

MVP 包含：

1. Project Ontology Registry draft implementation
2. Core Object Type definitions
3. Core Link Type definitions
4. Core Action Type and Action Contract definitions
5. Agent Role Policy definitions
6. Object State Machine definitions
7. Action Proposal data model
8. Basic Action Arbitration
9. Append-only Event Store model
10. Projection Read Models
11. Runtime API command/query boundary
12. Compatibility mapping from existing workflow/event concepts

### 6.2 Core Objects

MVP 核心对象：

| object | include | note |
| --- | --- | --- |
| `Requirement` | yes | 标准化后的需求对象 |
| `Spec` | yes | 已确认边界和验收 |
| `Issue` | yes | 可执行工作契约 |
| `Run` | yes | Agent 执行尝试 |
| `Evidence` | yes | 验证、日志、截图、输出 |
| `Artifact` | yes | 代码、文档、交付物引用 |
| `Decision` | yes | 人类确认和治理决策 |
| `Audit` | yes | 独立审计流程 |
| `Finding` | yes | 审计或评审发现 |
| `Project` | yes | 聚合根和项目视图 |
| `WorkPackage` | later | MVP 中可由 Spec 直接派生 Issue |

### 6.3 Core State Machines

MVP 状态机：

```text
requirement.state-machine
spec.state-machine
issue.state-machine
run.state-machine
audit.state-machine
finding.state-machine
```

`Project` 在 MVP 中优先作为聚合 Projection，不强行定义完整状态机。
`Delivery` 在 MVP 中优先作为 Delivery Package，不强行定义完整状态机。

### 6.4 Core Agent Roles

MVP 角色：

| role | include | note |
| --- | --- | --- |
| `SpecAgent` | yes | 处理需求、草案、预览 |
| `BuildAgent` | yes | 执行单个 Issue Contract |
| `AuditAgent` | yes | 独立审计 |
| `ReviewAgent` | yes | 可作为轻量检查者 |
| `CoordinatorAgent` | limited | MVP 只做调度和冲突解释，不做事实写入 |
| `HumanOwner` | yes | 确认、拒绝、重开、裁决 |

### 6.5 Core Projections

MVP 读模型：

| projection | purpose |
| --- | --- |
| `RequirementIntakeView` | 显示需求分类、歧义、边界 |
| `SpecPreviewView` | 显示 Spec Draft 和确认状态 |
| `ProjectHomeView` | 显示项目聚合、issue、阻塞、下一步 |
| `TaskWorkbenchView` | 显示 active Issue、Run、Evidence、Artifact |
| `AuditSurfaceView` | 显示独立 Audit 和 Finding |
| `DeliveryPackageView` | 显示交付摘要和证据包 |

## 7. Explicitly Out Of Scope

下一版本 MVP 不做：

- 多行业客户端同时落地；
- 云端多租户部署；
- 完整 Message Bus；
- 完整 Connector Pack 市场；
- 完整 Domain Pack SDK；
- 完整 Surface Pack 设计系统；
- 多 Agent 同时写同一 Issue；
- 自动审计触发；
- Build Agent 写审计报告；
- `.agentflow/spec/**` 直接落盘执行；
- 当前 v0.3.0 审计改造。

## 8. Basic Arbitration Scope

MVP 的 Action Arbitration 只实现基础能力：

```text
validate actionType
validate actorRole
validate object state
validate required evidence
validate object lock
accept / reject / requireHumanDecision
append accepted event
```

MVP 不实现复杂能力：

```text
distributed lock
multi-agent parallel write
priority preemption
cross-project transaction
automatic conflict merge
```

## 9. Runtime API Boundary

MVP 需要定义两个入口：

```text
Command API
Query API
```

Command API 只接收命令并生成 Action Proposal：

```text
submitRequirement
approveSpec
createIssue
startRun
submitEvidence
submitDelivery
markIssueDone
requestAudit
createFinding
linkFixIssue
```

Query API 只读 Projection：

```text
getRequirementIntakeView
getSpecPreviewView
getProjectHomeView
getTaskWorkbenchView
getAuditSurfaceView
getDeliveryPackageView
```

## 10. Requirement Intake Result

结论：`ready-for-spec`。

### Requirement Summary

用户要把前面形成的 Agent Project OS 架构草案收敛成下一版本正式 SPEC 前的统一输入，包括合并重复概念、统一术语、裁剪 MVP 范围，并准备 SPEC Draft Preview。

### Known Facts

- 当前工作处于下一版本开发前置阶段；
- 当前 v0.3.0 审计不应被影响；
- 本阶段不授权 Build Agent 执行；
- 不应直接写 `.agentflow/spec/**`；
- 已有 6 份根目录架构前置草案；
- 已有旧版 OS、capability、workflow、event/projection 架构文档可作为兼容输入；
- Audit 必须保持独立，不并入 Build Done；
- SPEC 需要先预览，等人类确认后才能进入正式 fact source。

### Missing Facts

没有阻断 SPEC Draft Preview 的缺失事实。

后续进入正式 SPEC 时，可以再确认一件事：

```text
下一版本是否只做 Runtime Foundation，不做客户端 UI 产品化。
```

本草案默认答案是：只做 Runtime Foundation。

### Suggested Scope

进入正式 SPEC 的范围应限定为：

- Project Ontology；
- Action Contract；
- Agent Role Policy；
- Object State Machine；
- Event Store；
- Projection Surface；
- Runtime API command/query 边界；
- 旧 workflow/event/capability 术语迁移说明。

### Non-goals

不进入范围：

- 当前 v0.3.0 审计；
- Build Agent 执行；
- 源码实现；
- `.agentflow/spec/**` 写入；
- `docs/requirements/**` 写入；
- 云端部署；
- 多行业客户端产品化；
- 完整 Message Bus；
- 自动审计触发。

### Acceptance Direction

SPEC Draft Preview 可接受的方向是：

- 能明确下一版本 project 目标；
- 能拆出有依赖关系的 issue preview；
- 能说明第一条可执行 issue；
- 能明确哪些文件在确认后才允许写入；
- 能保留 Audit 独立边界；
- 能避免直接进入 Build Agent 执行。

### Boundary Risks

主要风险：

- 把 preview 当成 `.agentflow/spec/**` 事实；
- 在未确认前写 `docs/requirements/**`；
- 把架构草案误当成 Build Agent handoff；
- 把 Audit 合并进 Build Done；
- 把 Runtime Foundation 做成过大的跨行业产品化项目。

### Recommended Next Step

推荐下一步：保留本文件为预览，等待人类确认是否进入正式 SPEC Gate。

确认后，才生成：

```text
docs/requirements/<requirement-id>.md
.agentflow/spec/projects/<project-id>.json
.agentflow/spec/issues/<issue-id>.json
```

## 11. SPEC Draft Preview

以下是正式 SPEC 前的 preview，不写入 `.agentflow/spec/**`。

### 11.1 SPEC Title

```text
AgentFlow Agent Project OS Runtime Foundation v1
```

### 11.2 Problem

当前 AgentFlow 已经有 Spec / Build / Audit 的流程边界，但底层对象、动作、状态、事件、角色权限和投影还没有统一收敛成可执行 Runtime Foundation。

如果继续只在页面、流程或单个 Agent handoff 上迭代，系统会遇到这些问题：

- Agent 权限边界散落在文档和 prompt；
- 状态推进容易被 UI、脚本或 Agent 直接写入；
- Audit 与 Build 的边界需要更强的底层约束；
- 多 Agent 执行缺少统一仲裁入口；
- 行业客户端无法复用稳定 Runtime；
- Projection 和事实源容易混在一起。

### 11.3 Goal

建立 AgentFlow 下一版本 Runtime Foundation，让系统具备：

- 标准 Project Ontology；
- 可验证 Action Contract；
- 可仲裁 Agent Role Policy；
- 可执行 Object State Machine；
- append-only Event Store；
- Projection Surface read model；
- 清晰 Runtime API command/query 边界。

### 11.4 Non-goals

不做：

- 当前 v0.3.0 审计修改；
- 直接生成 Build Agent issue；
- 直接写 `.agentflow/spec/**`；
- 多行业产品实现；
- 云端部署；
- 完整 Message Bus；
- 自动触发 Audit。

### 11.5 Acceptance Criteria

正式 SPEC 可接受的标准：

- 所有核心对象有 Object Type 定义；
- 所有核心关系有 Link Type 定义；
- 所有会写事实的动作有 Action Contract；
- 所有核心角色有 Role Policy；
- Requirement / Spec / Issue / Run / Audit / Finding 有状态机；
- Action Proposal 必须经过 Arbitration；
- accepted Action 才能写 Event Store；
- Projection 只读 Event Store 和 Ontology；
- UI / CLI / Agent 不能直接写最终状态；
- Build Done 不自动创建 Audit；
- Finding 修复通过 Issue 回流。

## 12. Project Preview

建议正式 SPEC 拆成一个 project：

```text
project-agentflow-agent-project-os-runtime-foundation-v1
```

Project 目标：

```text
把 AgentFlow 下一版本 Runtime Foundation 从架构草案收敛为可执行 SPEC，并按 issue 分阶段落地。
```

Project 边界：

- 只处理 Runtime Foundation；
- 不处理行业客户端产品化；
- 不处理当前 v0.3.0 审计；
- 不授权 Build Agent 执行，直到 SPEC 被确认并写入 `.agentflow/spec/**`。

## 13. Issues Preview

以下只是 preview，不是 `.agentflow/spec/**` 事实。

详细工程边界见：

- [AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md)

| issue | title | depends on |
| --- | --- | --- |
| `AF-OS-001` | 收敛 Project Ontology Registry 与核心对象/关系定义 | none |
| `AF-OS-002` | 定义 Action Type、Action Contract 与 Action Proposal schema | `AF-OS-001` |
| `AF-OS-003` | 定义 Agent Role Policy 与角色能力矩阵 | `AF-OS-001` |
| `AF-OS-004` | 定义 Requirement / Spec / Issue / Run / Audit / Finding 状态机 | `AF-OS-001`, `AF-OS-002` |
| `AF-OS-005` | 定义基础 Action Arbitration 与对象锁规则 | `AF-OS-002`, `AF-OS-003`, `AF-OS-004` |
| `AF-OS-006` | 定义 Event Store envelope、append 规则和 replay 边界 | `AF-OS-005` |
| `AF-OS-007` | 定义 Projection Read Models 与 Runtime Query API | `AF-OS-006` |
| `AF-OS-008` | 定义 Runtime Command API 与 UI/CLI 命令回流规则 | `AF-OS-005`, `AF-OS-007` |
| `AF-OS-009` | 对齐旧 workflow/event/capability 文档并形成迁移说明 | `AF-OS-006`, `AF-OS-007` |
| `AF-OS-010` | 完成 Runtime Foundation 集成验证与 SPEC closeout | `AF-OS-008`, `AF-OS-009` |

## 14. First Executable Candidate

如果后续确认进入正式 SPEC，第一条可执行 issue 应该是：

```text
AF-OS-001 收敛 Project Ontology Registry 与核心对象/关系定义
```

原因：

- Ontology 是后续 Action、Role、State、Projection 的共同语言；
- 没有对象和关系定义，后续状态机和事件无法稳定落地；
- 这条 issue 可以先做定义层，不触碰 Build/Audit 执行链路。

## 15. Confirmation Gate

进入正式 SPEC 前必须由人类确认：

```text
确认后，Spec Agent 才能把本 preview 转成正式 `.agentflow/spec/**`。
未确认前，本文件只是架构前置草案，不是任务事实。
```

建议下一步确认选项：

```text
1. 继续调整 preview
2. 确认进入正式 SPEC Draft
3. 暂停，不写 AgentFlow fact source
```
