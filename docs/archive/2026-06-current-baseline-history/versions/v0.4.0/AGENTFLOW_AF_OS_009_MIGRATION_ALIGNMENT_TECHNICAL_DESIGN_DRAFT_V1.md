# AF-OS-009 Migration Alignment Technical Design Draft V1

日期：2026-06-20
执行者：Codex
用途：Next Version Development Preflight / AF-OS-009 技术设计草案
状态：Architecture Draft / 非执行需求 / 不进入当前 v0.3.0 审计 / 不授权 Build Agent 执行

关联文档：

- [AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md)
- [AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md)
- [AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md)
- [AGENTFLOW_AF_OS_006_EVENT_STORE_INTEGRATION_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_006_EVENT_STORE_INTEGRATION_TECHNICAL_DESIGN_DRAFT_V1.md)
- [AGENTFLOW_AF_OS_007_PROJECTION_READ_MODELS_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_007_PROJECTION_READ_MODELS_TECHNICAL_DESIGN_DRAFT_V1.md)

说明：本文件只展开 `AF-OS-009` 的技术设计。它不创建 `.agentflow/spec/**` 任务事实，不写 `docs/requirements/**`，不修改源码，也不授权 Build Agent 执行。

## 1. Conclusion

`AF-OS-009` 的目标是把旧 Workflow / Capability / Event / Projection 语言迁移到 Runtime Foundation 语言，避免下一版本出现两套模型。

核心规则：

```text
旧文档不删除
旧概念不强行推翻
旧概念需要明确映射到新 Runtime 语言
下一版实现只能以 Runtime Foundation 术语为主
```

## 2. Scope

### 2.1 In Scope

`AF-OS-009` 应覆盖：

- old-to-new terminology map；
- module responsibility migration map；
- deprecated concept list；
- compatibility notes；
- next-version documentation update plan；
- old event / projection / workflow mapping。

### 2.2 Out Of Scope

`AF-OS-009` 不做：

- 删除旧文档；
- 修改源码；
- 写正式 requirements；
- 写 `.agentflow/spec/**`；
- 执行迁移脚本；
- 关闭当前 v0.3.0 审计。

## 3. Documents To Align

需要对齐：

```text
docs/architecture/001-project-operating-system-v1.md
docs/architecture/002-agent-capability-matrix-v1.md
docs/architecture/003-workflow-schema-v1.md
docs/architecture/004-event-and-projection-model-v1.md
docs/architecture/current-module-boundaries.md
```

本草案只定义迁移说明，不直接修改这些文档。

## 4. Terminology Map

| old term | new term | rule |
| --- | --- | --- |
| Project Operating System | Agent Project OS / Runtime Foundation | 保留方向，收敛底层术语 |
| BuildAgent / `build-agent` | WorkAgent / `work-agent` | `WorkAgent` 是 runtime 主命名，`BuildAgent` 只保留为兼容别名 |
| Capability Matrix | Agent Role Policy | 权限以 Role Policy 为准 |
| Workflow State | Object State Machine / Projection State | 写侧状态和读侧状态拆开 |
| Workflow Action | Action Type / Action Contract | action 必须有 contract |
| Task Event | Runtime Event Envelope | 兼容读取旧事件 |
| Event Projection | Projection Read Model | Projection 是读模型 |
| AuditFinding | Finding | 统一对象命名 |
| Delivery | DeliveryPackageView | MVP 不建 Delivery 对象 |
| WorkPackage | Project / Issue decomposition | MVP 暂缓 WorkPackage 对象 |

## 5. Module Responsibility Map

| current module | next responsibility |
| --- | --- |
| `crates/spec` | spec/project/issue contract compatibility |
| `crates/workflow-core` | workflow schema compatibility and guard patterns |
| `crates/workflow-runtime` | transition execution experience, not final OS authority |
| `crates/event-store` | Runtime Event append-only fact source |
| `crates/projection` | Projection Read Models and query output |
| `crates/task-loop` | issue/run scheduling compatibility |
| `crates/agent-dispatcher` | provider role binding compatibility |
| `crates/audit` | Audit / Finding compatibility |
| `crates/state` | indexes, gates, blockers, locks compatibility |

新增模块建议：

```text
crates/ontology
crates/action-contract
crates/role-policy
crates/object-state
crates/action-arbitration
crates/runtime-api
```

## 6. Deprecated Concepts

下一版本应标记为 deprecated 或 compatibility-only：

```text
BuildAgent as primary role name
AuditFinding as object name
Workflow State as write-side final state
Delivery as core write object
WorkPackage as required MVP object
UI direct state mutation
Agent direct done write
Build Done auto Audit
```

注意：deprecated 不等于立即删除。
它表示新 SPEC 和新实现不再以它们为主语言。

## 7. Compatibility Notes

### 7.1 Existing Spec Files

现有 `.agentflow/spec/**` 仍是当前任务契约事实源。
Runtime Foundation 设计不会绕过它。

### 7.2 Existing Task Events

旧 task events 兼容读取。
不要求批量改写历史事件。

### 7.3 Existing Projection

现有 projection 可以逐步映射到新的 read model。
不要求一次重做 Desktop UI。

### 7.4 Existing Agent Handoff

现有 handoff 文案继续可读，但后续能力边界以 Role Policy 为准。

## 8. Documentation Update Plan

正式进入 SPEC 后，建议更新顺序：

```text
1. current-module-boundaries.md
2. 002-agent-capability-matrix-v1.md
3. 003-workflow-schema-v1.md
4. 004-event-and-projection-model-v1.md
5. 001-project-operating-system-v1.md
```

原因：

- 先改模块边界，防止代码实现无归属；
- 再改 capability，固定角色权限；
- 再改 workflow，拆出 object state；
- 再改 event/projection，稳定事实和读模型；
- 最后回收 OS 总图。

## 9. Validation Checklist

迁移说明必须检查：

- `BuildAgent` 已映射到 `WorkAgent`；
- `AuditFinding` 已映射到 `Finding`；
- `Workflow State` 不再作为写侧唯一状态；
- Event model 与 Runtime Event Envelope 不冲突；
- Projection 不被描述成事实源；
- Build Done 不自动触发 Audit；
- current module boundaries 没有被推翻，只被升级；
- 新增 crate 的职责没有和旧 crate 冲突。

## 10. Deliverable Shape

后续正式执行时应产出：

```text
Runtime Foundation Migration Notes
Old-to-new Terminology Map
Module Responsibility Map
Deprecated Concept List
Documentation Update Plan
Compatibility Risk Register
```

## 11. Test / Review Plan

这条 issue 是文档和架构对齐，不跑项目构建。

建议验证：

1. `rg "Work Agent|AuditFinding|Workflow State|Delivery|WorkPackage"` 定位旧术语；
2. 检查每个旧术语是否有新术语映射；
3. 检查 docs 不再把 Projection 描述为事实源；
4. 检查 docs 不再描述 Build Done auto Audit；
5. 检查 new crate responsibilities 不互相覆盖；
6. 检查迁移说明不要求立即删除旧文档。

## 12. Acceptance Criteria

`AF-OS-009` 完成时应满足：

- old-to-new terminology map 完整；
- module responsibility map 完整；
- deprecated concept list 明确；
- compatibility notes 明确；
- next-version documentation update plan 明确；
- 不删除旧文档；
- 不修改源码；
- 不写正式 requirements；
- 不写 `.agentflow/spec/**`。

## 13. Risks

| risk | mitigation |
| --- | --- |
| 两套术语长期并存 | 新 SPEC 只采用 Runtime Foundation 术语 |
| 迁移被误解为删除旧文档 | 明确 compatibility-only，不直接删除 |
| 旧 projection 被误认为事实源 | docs 中固定 Event Store authority |
| WorkPackage 重新膨胀 MVP | 标记为 future object, not MVP |

## 14. Next

`AF-OS-009` 之后进入：

```text
AF-OS-010 Runtime Foundation Integration Closeout
```

它负责检查全部 issue、依赖、边界、验收和正式 SPEC 入口是否闭合。
