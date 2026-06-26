# AF-OS-007 Projection Read Models Technical Design Draft V1

日期：2026-06-20
执行者：Codex
用途：Next Version Development Preflight / AF-OS-007 技术设计草案
状态：Architecture Draft / 非执行需求 / 不进入当前 v0.3.0 审计 / 不授权 Build Agent 执行

关联文档：

- [AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md)
- [AGENTFLOW_AF_OS_006_EVENT_STORE_INTEGRATION_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_006_EVENT_STORE_INTEGRATION_TECHNICAL_DESIGN_DRAFT_V1.md)
- [AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md)

说明：本文件只展开 `AF-OS-007` 的技术设计。它不创建 `.agentflow/spec/**` 任务事实，不写 `docs/requirements/**`，不修改源码，也不授权 Build Agent 执行。

## 1. Conclusion

`AF-OS-007` 的目标是让 UI / CLI / 行业客户端只读 Projection，不直接读写 Runtime 事实。

核心规则：

```text
Event Store 是事实
Projection 是读模型
UI 读 Projection
UI 命令回到 Runtime API
Projection 不能写 Event Store
Projection 不能成为事实权威
```

## 2. Scope

### 2.1 In Scope

`AF-OS-007` 应覆盖：

- Projection read model schema；
- Runtime Query API shape；
- ontology-aware projection mapping；
- state-aware projection mapping；
- compatibility mapping to current Desktop views；
- read model rebuild boundary；
- read model freshness metadata。

### 2.2 Out Of Scope

`AF-OS-007` 不做：

- 重做 Desktop UI；
- command side；
- Action Proposal creation；
- Event Store append；
- Build Agent 执行；
- Audit Agent 执行；
- 行业客户端具体页面设计。

## 3. Existing Base

建议扩展：

```text
crates/projection
apps/desktop/src-tauri/src/commands/projection.rs
```

复用：

```text
task projection
project projection
requirement preview projection
audit summary projection
delivery summary projection
```

## 4. Core Read Models

MVP read models：

```text
RequirementIntakeView
SpecPreviewView
ProjectHomeView
TaskWorkbenchView
AuditSurfaceView
DeliveryPackageView
RuntimeHealthView
```

### 4.1 RequirementIntakeView

用途：

```text
展示 Requirement 分类、歧义、边界、下一步 route
```

核心字段：

```text
requirementId
state
classification
ambiguities
boundaryNotes
allowedActions
lastEventId
```

### 4.2 SpecPreviewView

用途：

```text
展示 SPEC 草案、确认状态、变更请求和 issue preview
```

核心字段：

```text
specId
state
requirementRef
previewSummary
acceptanceCriteria
issuePreview
confirmationState
allowedActions
```

### 4.3 ProjectHomeView

用途：

```text
展示 Project 聚合、Issue 状态、依赖、风险和最近活动
```

核心字段：

```text
projectId
stateSummary
issueGroups
dependencyGraph
activeRuns
blockedItems
recentEvents
```

### 4.4 TaskWorkbenchView

用途：

```text
展示当前 Issue、Run、Evidence、Artifact 和可执行动作
```

核心字段：

```text
issueId
issueState
activeRun
evidenceRefs
artifactRefs
acceptanceMapping
allowedActions
blockedReasons
```

### 4.5 AuditSurfaceView

用途：

```text
展示独立 Audit、Finding、Evidence graph 和 traceability
```

核心字段：

```text
auditId
auditState
scope
evidenceMap
findings
traceability
allowedActions
```

硬规则：

```text
AuditSurfaceView 不修改 Build delivery fact
AuditSurfaceView 不由 Issue.done 自动生成
```

### 4.6 DeliveryPackageView

用途：

```text
展示 Issue done 相关产物、证据和验收映射
```

核心字段：

```text
issueId
deliveryState
artifactRefs
verificationLogs
acceptanceMapping
buildAgentSummary
```

硬规则：

```text
DeliveryPackageView 不触发 Audit
DeliveryPackageView 不是 Delivery 对象事实源
```

## 5. Query API Shape

建议查询：

```text
getRequirementIntakeView(requirementId)
getSpecPreviewView(specId)
getProjectHomeView(projectId)
getTaskWorkbenchView(issueId)
getAuditSurfaceView(auditId)
getDeliveryPackageView(issueId)
getRuntimeHealthView(projectId)
```

统一 response metadata：

```text
projectionVersion
lastEventId
lastRebuiltAt
definitionVersions
staleness
warnings
```

## 6. Projection Mapping Rules

Projection 输入：

```text
RuntimeEventEnvelope
Ontology Registry
Action Contract Registry
Role Policy Registry
Object State Machine Registry
```

Projection 输出：

```text
read models
allowed view actions
blocked reasons
traceability links
```

注意：

- `allowedActions` 是可展示候选动作，不是最终授权；
- 用户点击后必须走 Runtime Command API；
- 最终 accepted/rejected 仍由 Arbitration 决定。

## 7. Compatibility Rules

当前 Desktop views 可以继续读取兼容投影。

映射建议：

| current surface | new read model |
| --- | --- |
| requirement preview | `RequirementIntakeView` / `SpecPreviewView` |
| project dashboard | `ProjectHomeView` |
| task panel | `TaskWorkbenchView` |
| audit display | `AuditSurfaceView` |
| delivery display | `DeliveryPackageView` |

## 8. Public API Sketch

后续实现可以提供：

```text
rebuild_projection(events, definition_registries) -> ProjectionBuildResult
get_project_home_view(project_id) -> ProjectHomeView
get_task_workbench_view(issue_id) -> TaskWorkbenchView
get_audit_surface_view(audit_id) -> AuditSurfaceView
explain_projection_staleness(view) -> ProjectionFreshness
```

这些 API 不应接触：

```text
Event Store append
Action Arbitration mutation
Provider Session
GitHub / GitLab
```

## 9. Test Plan

后续实现时建议测试：

1. Projection rebuild reads Runtime Event only；
2. Projection does not append events；
3. `Issue.done` appears in DeliveryPackageView；
4. `Issue.done` does not create AuditSurfaceView；
5. AuditSurfaceView shows Finding without mutating Build facts；
6. TaskWorkbenchView separates Issue state and Run state；
7. old task events can still project compatibility view；
8. allowedActions are hints and not final authorization；
9. staleness metadata changes when event cursor changes；
10. read model includes definition versions。

## 10. Acceptance Criteria

`AF-OS-007` 完成时应满足：

- Projection 只读 Event Store 和定义层；
- UI / CLI / 行业客户端只读 Projection；
- read model 能表达 current / past / future / exception；
- AuditSurfaceView 保持独立；
- DeliveryPackageView 不触发 Audit；
- Projection 不成为事实源；
- Query API 有 freshness metadata。

## 11. Risks

| risk | mitigation |
| --- | --- |
| Projection 被当作事实源 | 明确 read-only，禁止写回 Event Store |
| allowedActions 被误认为授权 | allowedActions 只是 UI hint，命令仍走 Arbitration |
| UI 改造范围失控 | 本 issue 只定义 read model，不重做 UI |
| Audit 和 Delivery 混合 | 分开 AuditSurfaceView 和 DeliveryPackageView |

## 12. Next

`AF-OS-007` 之后进入：

```text
AF-OS-008 Runtime Command API
```

Command API 把 UI / CLI / Agent 的写意图统一转换成 Action Proposal。
