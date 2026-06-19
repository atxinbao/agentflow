# AF-OS-001 Ontology Registry Technical Design Draft V1

日期：2026-06-19
执行者：Codex
用途：Next Version Development Preflight / AF-OS-001 技术设计草案
状态：Architecture Draft / 非执行需求 / 不进入当前 v0.3.0 审计 / 不授权 Build Agent 执行

关联文档：

- [AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md)
- [AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md)
- [AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md)
- [AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md)

说明：本文件只展开 `AF-OS-001` 的技术设计。它不创建 `.agentflow/spec/**` 任务事实，不写 `docs/requirements/**`，不修改源码，也不授权 Build Agent 执行。

## 1. Conclusion

`AF-OS-001` 的目标不是实现完整 Runtime。

它只做一件事：

```text
定义 AgentFlow 项目世界里有哪些对象、对象之间有哪些合法关系，并让这些定义可以被 Runtime 读取和校验。
```

这一步完成后，后续 `Action Contract / Role Policy / State Machine / Arbitration / Projection` 才有共同语言。

## 2. Problem

当前 AgentFlow 已有：

- `crates/spec` 管理 spec project / issue；
- `crates/event-store` 管理事件；
- `crates/projection` 重建读模型；
- `crates/workflow-runtime` 处理 workflow transition。

但它们缺少一个统一定义层：

```text
Requirement 是什么？
Issue 和 Run 的关系是什么？
Evidence 可以证明什么？
Audit 和 Finding 如何关联？
哪些 link 是合法的？
定义如何版本化？
```

如果没有 Ontology Registry，后续 Action Arbitration 只能靠散落的 if/else 和字符串判断。

## 3. Scope

### 3.1 In Scope

`AF-OS-001` 应覆盖：

- `crates/ontology` crate 设计；
- Ontology Bundle schema；
- Ontology Registry record schema；
- Object Type definition schema；
- Link Type definition schema；
- core object definitions；
- core link definitions；
- validation rules；
- 与现有 `crates/spec` 的映射；
- 后续 action/state/role 引用方式。

### 3.2 Out Of Scope

`AF-OS-001` 不做：

- Action Contract；
- Agent Role Policy；
- Object State Machine 具体 transition；
- Action Arbitration；
- Event Store append；
- Projection read model；
- Runtime Command API；
- Desktop UI；
- Build Agent 执行；
- Audit 执行。

## 4. Proposed Crate

建议新增：

```text
crates/ontology
```

建议模块：

```text
crates/ontology/src/lib.rs
crates/ontology/src/model.rs
crates/ontology/src/registry.rs
crates/ontology/src/validation.rs
crates/ontology/src/core.rs
crates/ontology/src/storage.rs
```

职责划分：

| module | responsibility |
| --- | --- |
| `model.rs` | schema structs and enums |
| `registry.rs` | in-memory registry lookup and indexing |
| `validation.rs` | bundle/object/link validation |
| `core.rs` | built-in `agentflow.core@v1-draft` definitions |
| `storage.rs` | future file read/write boundary |
| `lib.rs` | public exports |

## 5. Core Model

### 5.1 OntologyBundle

建议字段：

```text
ontologyId
namespace
version
status
objectTypes
linkTypes
definitionRecords
compatibility
migration
```

MVP 不在 bundle 内嵌 Action / Role / State 完整定义，只保留引用字段。

原因：

- `AF-OS-001` 只建立对象和关系；
- Action / Role / State 会由后续 issue 独立定义；
- 提前内嵌会扩大第一条 issue 范围。

### 5.2 OntologyDefinitionRecord

建议字段：

```text
id
namespace
kind
version
status
owner
createdAt
updatedAt
compatibility
deprecation
```

`kind` MVP 先支持：

```text
objectType
linkType
```

保留但不强制实现：

```text
stateMachine
actionType
agentRolePolicy
projectionModel
```

### 5.3 ObjectTypeDefinition

建议字段：

```text
id
name
description
properties
requiredProperties
stateMachineRef
allowedLinkTypes
allowedActionTypes
projectionHints
```

MVP 的 `properties` 可以先是结构化 metadata，不需要完整 JSON Schema 引擎。

建议格式：

```text
PropertyDefinition {
  name
  valueType
  required
  description
}
```

### 5.4 LinkTypeDefinition

建议字段：

```text
id
sourceObjectType
targetObjectType
cardinality
description
allowedActions
projectionHints
```

`cardinality` 候选：

```text
oneToOne
oneToMany
manyToOne
manyToMany
```

## 6. Core Object Types

`AF-OS-001` 第一版应定义 10 个核心对象。

| object type | purpose | existing mapping |
| --- | --- | --- |
| `Requirement` | 标准化后的人类需求 | `RequirementPreviewRuntime` / public requirement record |
| `Spec` | 被确认的需求边界和验收 | `docs/requirements/**` + spec package |
| `Project` | 项目聚合根 | `SpecProject` |
| `Issue` | 可执行工作契约 | `SpecIssue` |
| `Run` | Agent 执行 Issue 的一次尝试 | `.agentflow/tasks/<issue-id>/runs/**` |
| `Evidence` | 验证、日志、截图、输出证明 | `.agentflow/tasks/<issue-id>/evidence/**` |
| `Artifact` | 代码、文档、交付物引用 | task artifacts / public delivery refs |
| `Decision` | 人类确认、拒绝、裁决、重开 | requirement confirmation / completion decision |
| `Audit` | 独立审计流程 | `.agentflow/audit/**` / audit issue |
| `Finding` | 审计或评审发现 | audit findings |

明确不在 MVP 核心对象中：

| object type | decision |
| --- | --- |
| `WorkPackage` | 暂缓；MVP 可以从 Spec 直接派生 Issue |
| `AgentRole` | 放到 `AF-OS-003` |
| `Delivery` | 暂作为 Projection / DeliveryPackage，不建完整对象 |

## 7. Core Link Types

第一版 link definitions：

| link type | source | target | cardinality | purpose |
| --- | --- | --- | --- | --- |
| `derivesFrom` | `Spec` | `Requirement` | manyToOne | Spec 来自需求 |
| `contains` | `Project` | `Issue` | oneToMany | Project 包含 Issue |
| `blocks` | `Issue` | `Issue` | manyToMany | Issue 依赖阻塞 |
| `executes` | `Run` | `Issue` | manyToOne | Run 执行 Issue |
| `produces` | `Run` | `Artifact` | oneToMany | Run 产生产物 |
| `proves` | `Evidence` | `Issue` | manyToOne | Evidence 证明 Issue |
| `supports` | `Evidence` | `Run` | manyToOne | Evidence 支撑某次 Run |
| `reviews` | `Finding` | `Evidence` | manyToMany | Finding 审查 Evidence |
| `requiresFix` | `Finding` | `Issue` | oneToMany | Finding 派生修复 Issue |
| `decides` | `Decision` | `Requirement` | manyToOne | Decision 影响 Requirement |
| `accepts` | `Decision` | `Spec` | manyToOne | Decision 确认 Spec |

注意：`decomposesTo` 暂缓。
原因是 MVP 不引入 `WorkPackage` 作为核心对象。

## 8. Registry Behavior

`OntologyRegistry` 需要支持：

```text
loadBundle
listObjectTypes
listLinkTypes
getObjectType
getLinkType
validateBundle
validateLinkEndpoint
validateObjectRef
```

不需要支持：

```text
dynamic hot reload
remote registry
multi-tenant registry
runtime mutation by UI
```

## 9. Validation Rules

MVP validation 必须检查：

- `ontologyId` 非空；
- `namespace` 非空；
- `version` 非空；
- object type id 唯一；
- link type id 唯一；
- link source object type 存在；
- link target object type 存在；
- definition record 指向的 definition 存在；
- deprecated definition 不被 active link 默认引用；
- object 的 `allowedLinkTypes` 必须存在；
- object 的 `stateMachineRef` 可以为空，但如果存在必须是合法 ref 格式。

MVP 不检查：

- full JSON Schema；
- action contract existence；
- role policy existence；
- state transition legality；
- projection renderability。

## 10. Storage Strategy

`AF-OS-001` 可以先提供 built-in registry，不必立即落 `.agentflow/ontology/**`。

推荐阶段：

```text
Phase 1: built-in core ontology in Rust
Phase 2: export/read JSON bundle
Phase 3: future .agentflow/ontology/** persisted definitions
```

原因：

- 第一版需要稳定类型和验证；
- 过早引入文件存储会扩大写路径；
- 当前用户明确不要求写 `.agentflow/**`。

## 11. Existing Model Mapping

### 11.1 SpecProject

`SpecProject` 映射为：

```text
ObjectType: Project
links:
  Project contains Issue
```

### 11.2 SpecIssue

`SpecIssue` 映射为：

```text
ObjectType: Issue
properties:
  issueId
  title
  status
  priority
  requiredAgentRole
  workflowRef
```

### 11.3 SpecIssue blockedBy

`blockedBy` 映射为：

```text
LinkType: blocks
source: Issue
target: Issue
```

### 11.4 Audit Issue

`issueCategory=audit` 映射为：

```text
ObjectType: Audit
source link:
  Audit reviews Evidence
```

但注意：Audit Issue 本身仍然是 `.agentflow/spec/issues/**` 的任务入口。Ontology 只定义语义，不改变当前事实源。

## 12. Public API Sketch

后续实现可以提供这些函数：

```text
core_ontology_bundle() -> OntologyBundle
core_ontology_registry() -> OntologyRegistry
validate_ontology_bundle(bundle) -> OntologyValidationReport
get_object_type(registry, object_type_id) -> Option<ObjectTypeDefinition>
get_link_type(registry, link_type_id) -> Option<LinkTypeDefinition>
validate_link(registry, link_type_id, source_type, target_type) -> Result
```

这些 API 不应接触：

```text
Event Store
Projection
Task Loop
Agent Dispatcher
Audit Storage
Desktop UI
```

## 13. Test Plan

后续实现时建议测试：

1. core ontology bundle validates；
2. duplicate object type id fails；
3. duplicate link type id fails；
4. link with missing source object fails；
5. link with missing target object fails；
6. object allowedLinkTypes references missing link fails；
7. `Project contains Issue` validates；
8. `Run executes Issue` validates；
9. `Finding requiresFix Issue` validates；
10. `WorkPackage` absence does not fail MVP bundle。

## 14. Acceptance Criteria

`AF-OS-001` 完成标准：

- `crates/ontology` 边界清楚；
- 核心对象定义完整；
- 核心关系定义完整；
- registry 可按 id 查找 object/link；
- validation report 能表达 pass/fail 和错误列表；
- 与 `SpecProject / SpecIssue / blockedBy / audit issue` 的映射清楚；
- 不依赖 Action Contract；
- 不依赖 Arbitration；
- 不写 `.agentflow/spec/**`；
- 不影响当前 v0.3.0 审计。

## 15. Risks

主要风险：

- 把 Ontology 做成过重的数据库 schema；
- 在第一条 issue 中提前实现 Action / Role / State；
- 过早写 `.agentflow/ontology/**` 并扩大 fact source；
- 把 `AuditFinding` 和 `Finding` 两套命名混用；
- 让 Ontology 直接决定执行，而不是只提供定义。

规避方式：

- 第一版只做 Object / Link Registry；
- `Finding` 作为 canonical term；
- 文件存储放到后续阶段；
- Action / Role / State 只保留 ref，不实现。

## 16. Next

如果继续推进下一步，应展开：

```text
AF-OS-002 Action Contract Technical Design Draft
```

但前提是 `AF-OS-001` 的对象和关系边界已经被接受。
否则 `AF-OS-002` 会没有稳定 target object type。
