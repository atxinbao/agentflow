# 054 - Core Ontology Kernel Contract V1

日期：2026-06-28  
执行者：Codex

## 1. 目标

本文件冻结 AgentFlow Core Ontology Kernel 的 v1 合同边界。

Core Ontology Kernel 只定义跨行业都成立的项目世界基本元素：

- Object
- Link
- Action
- State
- Skill
- Evidence
- Decision
- Artifact
- Route
- Spec Bundle
- Projection

这些元素是 AgentFlow Project OS 的底层语义地基。它们不能直接等同于软件研发行业里的 issue、PR、release、repository、test log 等对象。

## 2. 权威边界

Core authority 只来自：

```text
crates/ontology/src/kernel.rs
docs/architecture/054-core-ontology-kernel-contract-v1.md
release-gate runtime/core-ontology-kernel.json
```

Reference App 可以把 Core 元素翻译成行业词：

```text
Object -> Issue / Screen / Shot
Action -> Implement / Design / Edit
Evidence -> Test Log / Preview / Render
Decision -> Accept / Request Changes / Recut
```

但这些行业词只能出现在 mapping fixture 或行业 Pack 里，不能成为 Core authority。

Machine-readable boundary phrase: reference mappings are not Core authority.

## 3. Core 元素定义

| Element | 说明 |
| --- | --- |
| Object | 可被 Action、State、Projection 引用的工作、知识、输出或审查单元 |
| Link | Object 之间的类型化关系 |
| Action | 被角色授权后可执行的操作提案 |
| State | Object 或 Action Execution 的生命周期值 |
| Skill | 约束 Agent 角色能力的能力包 |
| Evidence | 支撑状态变化、决策或完成声明的证据引用 |
| Decision | 改变权威边界的判断或确认记录 |
| Artifact | 工作流阶段产生的持久输出引用 |
| Route | workflow policy 选择出的下一步路径 |
| Spec Bundle | 经确认后可 materialize runtime work 的合同集合 |
| Projection | 从 events 和 contracts 派生给 UI/API 的读模型 |

## 4. 禁止进入 Core 的行业词

Core authority 不得要求以下软件研发行业词：

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

这些词可以作为 Software Dev Reference App 的映射输出，但不能出现在 Core element description、authority boundary 或 release certification 的 Core authority 里。

## 5. Release Gate 证明

release-gate 必须生成：

```text
runtime/core-ontology-kernel.json
runtime/core-ontology-kernel-rust-test.log
```

证明内容必须包含：

- `agentflow-core-ontology-kernel.v1`；
- 11 个 Core elements 全部存在；
- forbidden Core terms 未污染 Core authority；
- reference mappings 不具备 Core authority；
- Rust contract tests 通过。

## 6. 非目标

本文件不定义：

- Software Dev issue schema；
- PR/MR schema；
- release delivery schema；
- GitHub / GitLab provider 行为；
- 行业 Pack 的完整 ontology；
- UI 页面布局。

这些属于 Reference App、Provider、Projection 或 Desktop surface。
