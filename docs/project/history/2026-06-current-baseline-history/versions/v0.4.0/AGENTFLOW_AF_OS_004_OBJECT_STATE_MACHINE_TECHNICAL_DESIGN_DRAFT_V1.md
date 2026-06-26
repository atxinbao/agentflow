# AF-OS-004 Object State Machine Technical Design Draft V1

日期：2026-06-20
执行者：Codex
用途：Next Version Development Preflight / AF-OS-004 技术设计草案
状态：Architecture Draft / 非执行需求 / 不进入当前 v0.3.0 审计 / 不授权 Build Agent 执行

关联文档：

- [AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md)
- [AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md)
- [AGENTFLOW_AF_OS_001_ONTOLOGY_REGISTRY_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_001_ONTOLOGY_REGISTRY_TECHNICAL_DESIGN_DRAFT_V1.md)
- [AGENTFLOW_AF_OS_002_ACTION_CONTRACT_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_002_ACTION_CONTRACT_TECHNICAL_DESIGN_DRAFT_V1.md)
- [AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md)

说明：本文件只展开 `AF-OS-004` 的技术设计。它不创建 `.agentflow/spec/**` 任务事实，不写 `docs/requirements/**`，不修改源码，也不授权 Build Agent 执行。

## 1. Conclusion

`AF-OS-004` 的目标是把项目状态拆成对象生命周期，而不是继续用一个 issue status 表达所有事实。

核心规则：

```text
Requirement 有 Requirement 的生命周期
Spec 有 Spec 的生命周期
Issue 有 Issue 的生命周期
Run 有 Run 的生命周期
Audit 有 Audit 的生命周期
Finding 有 Finding 的生命周期
状态变化必须由 Action + State Machine + Event 共同证明
```

`Run.completed` 不等于 `Issue.done`。
`Issue.done` 不自动创建 `Audit`。
`Finding.fixRequired` 不改写旧 Build 事实，只能派生或链接修复 Issue。

## 2. Scope

### 2.1 In Scope

`AF-OS-004` 应覆盖：

- object state machine schema；
- 6 个核心状态机定义；
- transition schema；
- terminal state rule；
- actionType reference rule；
- required evidence reference rule；
- cross-object state rule；
- validation report；
- 与 `crates/workflow-core` / `crates/workflow-runtime` 的兼容边界。

### 2.2 Out Of Scope

`AF-OS-004` 不做：

- 扩大 workflow schema；
- Event Store append；
- Projection rebuild；
- Build Agent 执行；
- Audit Agent 执行；
- 状态迁移脚本；
- Desktop UI 状态改造。

## 3. Proposed Crate

建议新增：

```text
crates/object-state
```

原因：

- 状态机是 Runtime Core 的独立概念；
- 不应该塞进 Ontology Registry；
- 不应该塞进 Action Contract；
- Arbitration 后续需要单独读取状态机定义。

建议模块：

```text
crates/object-state/src/lib.rs
crates/object-state/src/model.rs
crates/object-state/src/registry.rs
crates/object-state/src/validation.rs
crates/object-state/src/core.rs
crates/object-state/src/report.rs
```

## 4. Dependencies

依赖：

```text
AF-OS-001 Ontology Registry
AF-OS-002 Action Contract
```

原因：

- 每个 state machine 绑定一个 Ontology object type；
- 每个 transition 引用已注册 action type；
- 每个 required evidence 引用 Action Contract evidence type；
- 后续 Arbitration 读取 live state 后用状态机判断 transition 是否允许。

## 5. Core Model

### 5.1 ObjectStateMachine

建议字段：

```text
stateMachineId
objectType
namespace
version
status
initialState
states
terminalStates
transitions
invariants
projectionHints
```

### 5.2 ObjectState

建议字段：

```text
stateId
name
description
isTerminal
isErrorState
```

### 5.3 StateTransition

建议字段：

```text
transitionId
fromState
toState
actionType
guards
requiredEvidence
emittedEvents
linkEffects
roleHints
```

注意：

- `roleHints` 不是权限判断；
- 权限判断属于 `AF-OS-003` 和 `AF-OS-005`；
- state machine 只声明 transition 期望。

### 5.4 StateMachineValidationReport

建议字段：

```text
stateMachineId
objectType
status
errors
warnings
normalizedDefinition
```

## 6. Core State Machines

MVP 必须定义：

```text
requirement.state-machine
spec.state-machine
issue.state-machine
run.state-machine
audit.state-machine
finding.state-machine
```

不在 MVP 定义：

```text
Project full lifecycle
Evidence full lifecycle
Artifact full lifecycle
Decision full lifecycle
```

原因：

- Project/Evidence/Artifact/Decision 在第一版主要作为聚合、证明和记录对象；
- 状态复杂度先集中在主链路对象；
- 后续可以通过 ontology versioning 添加。

## 7. Required Lifecycle Rules

### 7.1 Requirement

最小状态：

```text
captured
normalized
classified
needsClarification
accepted
rejected
superseded
```

关键规则：

```text
Requirement.accepted 才能进入 draftSpec
Requirement.rejected 不得派生 Spec
Requirement.superseded 必须保留替代关系
```

### 7.2 Spec

最小状态：

```text
drafting
draftReady
awaitingConfirmation
changeRequested
approved
cancelled
superseded
```

关键规则：

```text
Spec.approved 才能派生 Project / Issue
未确认 Spec Preview 不授权 Build Agent
Spec.cancelled 不得继续派生 Issue
```

### 7.3 Issue

最小状态：

```text
proposed
ready
blocked
running
reviewReady
done
reopened
cancelled
superseded
```

关键规则：

```text
Issue.ready 才能 startRun
Issue.running 表示存在 active Run
Issue.done 不自动创建 Audit
Issue.reopened 必须由 HumanOwner 或 Finding 触发
```

### 7.4 Run

最小状态：

```text
queued
started
awaitingInput
paused
blocked
failed
completed
cancelled
```

关键规则：

```text
Run.completed 不等于 Issue.done
Run.failed 必须保留失败 evidence
Run.cancelled 必须释放执行锁
```

### 7.5 Audit

最小状态：

```text
requested
accepted
running
reportReady
closed
cancelled
```

关键规则：

```text
Audit.requested 只能来自显式 requestAudit
Build Done 不能自动创建 Audit
Audit 不修改 Build 交付事实
```

### 7.6 Finding

最小状态：

```text
open
triaged
fixRequired
notActionable
fixLinked
resolved
verified
closed
reopened
```

关键规则：

```text
Finding.fixRequired 可以派生或链接修复 Issue
Finding 不直接修改 Issue.done
Finding.closed 必须有关闭证据或决策
```

## 8. Cross-object Rules

必须支持这些跨对象规则：

| source | target rule |
| --- | --- |
| `Requirement.accepted` | can propose `draftSpec` |
| `Spec.approved` | can propose `createProject` / `createIssue` |
| `Issue.ready` | can propose `startRun` |
| `Run.completed` | can support `Issue.reviewReady` |
| `Issue.done` | can be audit input but cannot create audit automatically |
| `Audit.reportReady` | can create `Finding.open` |
| `Finding.fixRequired` | can link or create repair `Issue.proposed` |

跨对象变化必须通过事件和 link 表达，不能隐式级联写状态。

## 9. Validation Pipeline

建议检查：

```text
1. state machine id is unique
2. object type exists in Ontology
3. initial state exists
4. terminal states exist
5. transition from/to states exist
6. action type exists in Action Contract registry
7. required evidence type exists
8. emitted event type declared
9. link effects reference known link types
10. no forbidden implicit transition exists
```

禁止出现：

```text
Issue.done -> Audit.requested implicit transition
Run.completed -> Issue.done implicit transition
Finding.fixRequired -> Issue.done mutation
```

## 10. Public API Sketch

后续实现可以提供：

```text
core_state_machine_bundle() -> ObjectStateMachineBundle
core_state_machine_registry() -> ObjectStateMachineRegistry
validate_state_machine_bundle(bundle, ontology_registry, action_registry) -> StateMachineValidationReport
get_state_machine(object_type) -> Option<ObjectStateMachine>
is_transition_defined(object_type, from_state, action_type) -> TransitionDecision
```

这些 API 不应接触：

```text
Event Store
Projection
Task Loop
Provider Session
Desktop UI
```

## 11. Test Plan

后续实现时建议测试：

1. core state machine bundle validates；
2. unknown object type fails；
3. unknown action type fails；
4. transition with missing state fails；
5. `Run.completed` does not imply `Issue.done`；
6. `Issue.done` does not imply `Audit.requested`；
7. `Finding.fixRequired` can link fix Issue；
8. `Finding.fixRequired` cannot mutate old Issue done state；
9. terminal states reject undefined outgoing transitions unless explicitly allowed；
10. all emitted events include object type and state fields。

## 12. Acceptance Criteria

`AF-OS-004` 完成时应满足：

- 定义 `crates/object-state` 的模型边界；
- 定义 6 个核心对象状态机；
- transition 引用已注册 Action Type；
- 状态机引用已注册 Object Type；
- `Run.completed` 与 `Issue.done` 分离；
- `Issue.done` 与 `Audit.requested` 分离；
- Finding 修复通过 Issue 回流；
- 可被 `AF-OS-005` Arbitration 读取。

## 13. Risks

| risk | mitigation |
| --- | --- |
| 状态机扩大成工作流引擎重写 | 只定义 object lifecycle，不替代 workflow-core |
| 隐式级联状态造成事实污染 | 所有跨对象变化通过事件和 link 表达 |
| Audit 被并回 Build 流 | 明确 Audit 独立状态机 |
| Run / Issue 状态混淆 | 使用测试固定 `Run.completed != Issue.done` |

## 14. Next

`AF-OS-004` 之后，Action、Role、State 三个输入都具备了。下一步进入：

```text
AF-OS-005 Action Arbitration
```

它负责把结构合法性、角色权限、状态机、证据和对象锁合成最终写入决策。
