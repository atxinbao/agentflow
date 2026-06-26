# AF-OS-010 Runtime Foundation Closeout Technical Design Draft V1

日期：2026-06-20
执行者：Codex
用途：Next Version Development Preflight / AF-OS-010 技术设计草案
状态：Architecture Draft / 非执行需求 / 不进入当前 v0.3.0 审计 / 不授权 Build Agent 执行

关联文档：

- [AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md)
- [AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md)
- [AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md)
- [AGENTFLOW_AF_OS_008_RUNTIME_COMMAND_API_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_008_RUNTIME_COMMAND_API_TECHNICAL_DESIGN_DRAFT_V1.md)
- [AGENTFLOW_AF_OS_009_MIGRATION_ALIGNMENT_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_009_MIGRATION_ALIGNMENT_TECHNICAL_DESIGN_DRAFT_V1.md)

说明：本文件只展开 `AF-OS-010` 的技术设计。它不创建 `.agentflow/spec/**` 任务事实，不写 `docs/requirements/**`，不修改源码，也不授权 Build Agent 执行。

## 1. Conclusion

`AF-OS-010` 的目标不是实现 Runtime Foundation，而是在正式进入 Build Agent 前确认设计闭合。

核心规则：

```text
Closeout 不是完成实现
Closeout 只确认 issue 设计、依赖、边界、验收闭合
Build Agent entry 仍需正式 SPEC 和人类确认
```

## 2. Scope

### 2.1 In Scope

`AF-OS-010` 应覆盖：

- integration checklist；
- dependency sanity check；
- issue boundary check；
- MVP scope check；
- Build / Audit separation check；
- validation plan；
- SPEC closeout summary；
- first executable issue recommendation。

### 2.2 Out Of Scope

`AF-OS-010` 不做：

- Build Agent 执行；
- 项目构建；
- PR 创建；
- 关闭当前 v0.3.0 审计；
- 写 `.agentflow/spec/**`；
- 写 `docs/requirements/**`；
- 宣称 Runtime Foundation 已实现。

## 3. Integration Checklist

必须检查这些对象是否闭合：

```text
Ontology definitions
Action contracts
Role policies
State machines
Arbitration rules
Event envelope
Projection read models
Runtime API boundary
Migration notes
```

## 4. Dependency Sanity Check

目标依赖：

```text
AF-OS-001
→ AF-OS-002
→ AF-OS-003
→ AF-OS-004
→ AF-OS-005
→ AF-OS-006
→ AF-OS-007
→ AF-OS-008
→ AF-OS-009
→ AF-OS-010
```

允许的局部并行：

```text
AF-OS-002 and AF-OS-003 after AF-OS-001
AF-OS-006 and AF-OS-007 design review can overlap after AF-OS-005 envelope is stable
```

不允许：

```text
AF-OS-005 before Action / Role / State are defined
AF-OS-006 before accepted action is defined
AF-OS-008 before command response can express arbitration result
```

## 5. Boundary Check

必须确认：

- `docs/requirements/**` 仍未被写入，除非人类正式确认；
- `.agentflow/spec/**` 仍未被写入，除非人类正式确认；
- Build Agent 没有被授权执行；
- Audit Agent 没有被混入 Build Done；
- Projection 没有成为事实源；
- Runtime API 没有绕过 Arbitration；
- Event Store 仍是事实权威；
- Message Bus 没有被提升成事实源。

## 6. MVP Scope Check

MVP 包含：

```text
Project Ontology Registry
Action Contract
Agent Role Policy
Object State Machine
Action Arbitration
Event Store Integration
Projection Read Models
Runtime Command API
Migration Alignment
Closeout
```

MVP 不包含：

```text
multi-industry product shell
cloud deployment
distributed message bus
distributed lock
full Domain Pack marketplace
WorkPackage core object
Delivery core object
automatic audit
```

## 7. Validation Plan

Closeout 应使用文档验证，不跑项目构建。

建议检查：

```text
rg "AF-OS-00[1-9]|AF-OS-010" AGENTFLOW_*.md
rg "不授权 Build Agent|不进入当前 v0.3.0 审计" AGENTFLOW_AF_OS_*.md
rg "WorkPackage|DeliveryPackage|AuditFinding" AGENTFLOW_AF_OS_*.md
rg "Issue.done.*Audit|Run.completed.*Issue.done" AGENTFLOW_AF_OS_*.md
```

检查目标：

- 每个 issue 有技术设计草案；
- 每个草案都有边界声明；
- 已知 deferred concept 没有进入 MVP action target；
- Build Done auto Audit 没有被引入；
- Run / Issue 状态没有混用。

## 8. SPEC Closeout Summary Shape

正式 closeout 应输出：

```text
Runtime Foundation SPEC readiness summary
issue dependency table
MVP scope table
boundary confirmation
first executable issue recommendation
known deferred concepts
human confirmation gate
```

## 9. First Executable Issue Recommendation

第一条真正可执行 issue 仍应是：

```text
AF-OS-001 Ontology Registry
```

原因：

- Action Contract 需要 Object Type；
- Role Policy 需要 Object Type；
- State Machine 需要 Object Type；
- Projection 需要 Object Type；
- Migration 需要统一术语根。

不能从 Arbitration 开始。
没有 Ontology，Arbitration 只会变成字符串 if/else。

## 10. Build Agent Entry Gate

Build Agent entry 必须满足：

```text
1. 人类确认正式 SPEC
2. 写入 docs/requirements/**
3. 写入 .agentflow/spec/projects/**
4. 写入 .agentflow/spec/issues/**
5. 当前 issue handoff 明确授权
6. executionPipeline 明确允许对应阶段
```

在这之前，所有文档都只是 Architecture Draft / Preflight Draft。

## 11. Acceptance Criteria

`AF-OS-010` 完成时应满足：

- 所有 issue 依赖闭合；
- 所有 issue 有明确边界；
- 第一条可执行 issue 明确；
- Build/Audit 边界没有混入；
- `.agentflow/spec/**` 写入仍需人类确认；
- Runtime Foundation MVP 没有膨胀到行业客户端；
- Closeout 不被当作实现完成。

## 12. Risks

| risk | mitigation |
| --- | --- |
| closeout 被误认为已经实现 | 明确 closeout 只做 readiness |
| 直接跳到 Build Agent | 固定 Build Agent Entry Gate |
| MVP 范围继续膨胀 | 明确 deferred concepts |
| v0.3.0 审计被混入 | 每个草案保留审计边界声明 |

## 13. Next

AF-OS-001 到 AF-OS-010 全部技术设计草案齐备后，下一步不是继续新增 issue。

下一步应是：

```text
Runtime Foundation SPEC Draft Preview
```

仍然必须先由人类确认，再写：

```text
docs/requirements/**
.agentflow/spec/projects/**
.agentflow/spec/issues/**
```
