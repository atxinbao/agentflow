# AF-OS-008 Runtime Command API Technical Design Draft V1

日期：2026-06-20
执行者：Codex
用途：Next Version Development Preflight / AF-OS-008 技术设计草案
状态：Architecture Draft / 非执行需求 / 不进入当前 v0.3.0 审计 / 不授权 Build Agent 执行

关联文档：

- [AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md)
- [AGENTFLOW_AF_OS_005_ACTION_ARBITRATION_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_005_ACTION_ARBITRATION_TECHNICAL_DESIGN_DRAFT_V1.md)
- [AGENTFLOW_AF_OS_007_PROJECTION_READ_MODELS_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_007_PROJECTION_READ_MODELS_TECHNICAL_DESIGN_DRAFT_V1.md)

说明：本文件只展开 `AF-OS-008` 的技术设计。它不创建 `.agentflow/spec/**` 任务事实，不写 `docs/requirements/**`，不修改源码，也不授权 Build Agent 执行。

## 1. Conclusion

`AF-OS-008` 的目标是把 UI / CLI / Agent 的写意图统一转成 `Action Proposal`。

核心规则：

```text
Command API 不直接改状态
Command API 不直接写 Event Store
Command API 只构造 Action Proposal
Action Proposal 必须进入 Arbitration
Command response 返回 accepted / rejected / humanDecisionRequired
```

这就是行业客户端接入 Runtime Core 的写入口。

## 2. Scope

### 2.1 In Scope

`AF-OS-008` 应覆盖：

- Runtime Command API boundary；
- command request schema；
- command response schema；
- command to ActionProposal mapping；
- Desktop / CLI / SDK adapter boundary；
- forbidden direct-write list；
- response error taxonomy。

### 2.2 Out Of Scope

`AF-OS-008` 不做：

- 重写所有 Tauri commands；
- 迁移全部 CLI；
- 启动 Build Agent；
- 自动创建 Audit；
- Event Store append；
- Projection rebuild；
- 行业客户端 UI 设计。

## 3. Proposed Crate

建议新增：

```text
crates/runtime-api
```

建议模块：

```text
crates/runtime-api/src/lib.rs
crates/runtime-api/src/commands.rs
crates/runtime-api/src/responses.rs
crates/runtime-api/src/mapping.rs
crates/runtime-api/src/errors.rs
```

适配层：

```text
apps/desktop/src-tauri/src/commands/runtime.rs
crates/cli
```

## 4. Dependencies

依赖：

```text
AF-OS-005 Action Arbitration
AF-OS-007 Projection Read Models
```

原因：

- Command API 写侧必须调用 Arbitration；
- UI 读侧必须来自 Projection；
- command response 需要返回 latest projection cursor / next query hint。

## 5. Command Request Model

建议字段：

```text
commandId
commandType
sourceSurface
actorRole
targetObjectRef
input
evidenceRefs
artifactRefs
idempotencyKey
createdAt
```

`sourceSurface` 候选：

```text
desktop
cli
sdk
agent
conversation
system
```

## 6. Command Response Model

建议字段：

```text
commandId
proposalId
status
decision
acceptedActionId
rejectedReasons
humanDecisionRequest
nextQueryHint
correlationId
```

`status` 候选：

```text
accepted
rejected
humanDecisionRequired
queued
invalidCommand
```

MVP 建议先支持：

```text
accepted
rejected
humanDecisionRequired
invalidCommand
```

## 7. Core Commands

MVP commands：

```text
submitRequirement
approveSpec
createProject
createIssue
activateIssue
startRun
submitEvidence
submitArtifact
markIssueDone
recordDecision
requestAudit
createFinding
linkFixIssue
```

注意：

- `submitDelivery` 在 MVP 中不作为独立对象写命令；
- 交付展示由 `DeliveryPackageView` 投影；
- Build completion 仍使用 `markIssueDone` + evidence / artifact refs。

## 8. Command To Action Mapping

示例映射：

| command | actionType |
| --- | --- |
| `submitRequirement` | `submitRequirement` |
| `approveSpec` | `approveSpec` |
| `createIssue` | `createIssue` |
| `startRun` | `startRun` |
| `submitEvidence` | `submitEvidence` |
| `submitArtifact` | `submitArtifact` |
| `markIssueDone` | `markIssueDone` |
| `requestAudit` | `requestAudit` |
| `createFinding` | `createFinding` |
| `linkFixIssue` | `linkFixIssue` |

Mapping 输出：

```text
ActionProposal
```

然后：

```text
ActionProposal
→ Arbitration
→ AcceptedAction
→ Event Store append
```

## 9. Forbidden Direct Writes

UI / CLI / Agent 禁止直接写：

```text
.agentflow/spec/**
.agentflow/events/**
.agentflow/tasks/*/runs/**
.agentflow/tasks/*/evidence/**
Event Store files
Projection storage
```

允许：

```text
read Projection
submit Runtime Command
display Runtime response
```

## 10. Adapter Boundary

### 10.1 Desktop

Desktop Tauri command 应只做：

```text
parse request
call runtime-api command
return response
```

不应该：

```text
directly mutate spec files
directly append event
directly modify projection
```

### 10.2 CLI

CLI 应只做：

```text
parse flags
build command request
call runtime-api
print response
```

### 10.3 Agent

Agent 工具调用应只做：

```text
submit Action-oriented command
attach evidence refs
wait for accepted/rejected response
```

Agent 不直接标记事实成立。

## 11. Public API Sketch

后续实现可以提供：

```text
submit_runtime_command(command, runtime_context) -> RuntimeCommandResponse
map_command_to_action_proposal(command) -> ActionProposal
validate_runtime_command(command) -> RuntimeCommandValidationReport
execute_command_via_arbitration(command, context) -> RuntimeCommandResponse
```

这些 API 不应接触：

```text
Provider Session startup
Desktop UI rendering
Projection writeback
GitHub / GitLab
```

## 12. Test Plan

后续实现时建议测试：

1. valid command maps to ActionProposal；
2. invalid command returns invalidCommand；
3. command does not append event directly；
4. command response propagates rejected reasons；
5. command response propagates humanDecisionRequired；
6. `markIssueDone` command maps to `markIssueDone` action；
7. `markIssueDone` command does not create audit；
8. UI adapter cannot write `.agentflow/spec/**` directly；
9. CLI adapter cannot bypass Runtime API；
10. idempotencyKey passes through to ActionProposal。

## 13. Acceptance Criteria

`AF-OS-008` 完成时应满足：

- Command API 只生成 Action Proposal；
- Action Proposal 必须进入 Arbitration；
- response 可以表达 accepted / rejected / humanDecisionRequired；
- UI 不直接改 `.agentflow/spec/**` 或 `.agentflow/events/**`；
- CLI 不绕过 Runtime；
- command side 和 query side 分离；
- 不自动创建 Audit。

## 14. Risks

| risk | mitigation |
| --- | --- |
| Runtime API 变成新写事实入口 | API 只构造 proposal，append 在 Event Store 层 |
| UI 继续直接写文件 | forbidden direct-write list 固化 |
| CLI 绕过 Runtime | CLI adapter 只调用 runtime-api |
| Build Done 自动触发 Audit | `markIssueDone` 不映射 `requestAudit` |

## 15. Next

`AF-OS-008` 完成后需要做术语和旧文档迁移：

```text
AF-OS-009 Migration Alignment
```

否则下一版会同时存在 Workflow / Capability / Event / Projection 两套语言。
