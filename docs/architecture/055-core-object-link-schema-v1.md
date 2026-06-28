# 055 - Core Object / Link Schema V1

日期：2026-06-28  
执行者：Codex

## 1. 目标

本文件定义 Core Ontology Kernel 之上的 Object / Link schema 合同。

它回答一个问题：

```text
AgentFlow Core 里有哪些通用对象？
这些对象之间可以怎样建立关系？
```

## 2. 权威边界

Core Object / Link schema 的权威来源：

```text
crates/ontology/src/schema.rs
docs/architecture/055-core-object-link-schema-v1.md
release-gate runtime/core-object-link-schema.json
```

本合同不使用 Software Dev 专有对象名。Software Dev Pack 可以把这些 Core 对象映射成 issue、PR、release、test log 等行业词，但这些映射不是 Core authority。

Machine-readable boundary phrase: reference mappings are not Core authority.

## 3. Core Objects

| Object | 说明 |
| --- | --- |
| RequestObject | 人类原始意图进入确认前的请求对象 |
| IntentObject | 规范化后的意图与上下文对象 |
| GoalObject | 目标和验收边界对象 |
| PlanObject | 产生一个或多个工作单元的顺序路径对象 |
| WorkObject | 可执行的项目工作单元 |
| ExecutionObject | 针对 WorkObject 的受控执行尝试 |
| EvidenceObject | 支撑状态变化和决策的证据引用 |
| ArtifactObject | 执行或审查产生的持久输出引用 |
| DecisionObject | 人类或系统记录的判断和确认 |
| ReviewObject | 独立审查工作流对象 |
| ProjectionObject | 给 UI/API 使用的派生读模型对象 |

所有 Core Object 必须至少包含：

```text
objectId
status
```

## 4. Core Links

| Link | Source | Target | 说明 |
| --- | --- | --- | --- |
| derivesFrom | IntentObject | RequestObject | 从上游权威对象派生 |
| contains | GoalObject | PlanObject | 父对象包含子对象 |
| blocks | WorkObject | WorkObject | 工作对象之间的阻塞关系 |
| executes | ExecutionObject | WorkObject | 执行对象尝试完成工作对象 |
| produces | ExecutionObject | ArtifactObject | 执行对象产生产物 |
| proves | EvidenceObject | WorkObject | 证据证明工作完成或验收 |
| supports | EvidenceObject | ExecutionObject | 证据支撑某次执行 |
| reviews | ReviewObject | EvidenceObject | 审查对象检查证据 |
| requiresFollowUp | ReviewObject | WorkObject | 审查要求后续工作 |
| decides | DecisionObject | RequestObject | 决策影响请求处理 |
| accepts | DecisionObject | GoalObject | 决策接受目标边界 |
| routesTo | IntentObject | GoalObject | 意图路由到目标对象 |

## 5. 禁止进入 Core 的行业词

Core Object / Link schema 不得要求：

```text
bug
feature
issue
pr
pull-request
release
repository
repository-patch
test-log
github-issue
```

这些词只能作为 Reference App mapping 或行业 Pack 的输出词。

## 6. Release Gate 证明

release-gate 必须生成：

```text
runtime/core-object-link-schema.json
runtime/core-object-link-schema-rust-test.log
```

证明内容必须包含：

- `agentflow-core-object-link-schema.v1`；
- 11 个 Core objects；
- 12 个 Core links；
- Link source / target 均指向已定义 Core Object；
- Object allowed links 均指向已定义 Core Link；
- forbidden terms 未污染 Core Object / Link schema。

