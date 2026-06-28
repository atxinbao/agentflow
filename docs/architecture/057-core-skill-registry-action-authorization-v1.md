# 057 - Core Skill Registry / Action Authorization V1

日期：2026-06-28  
执行者：Codex

## 1. 目标

本文件定义 Core Skill Registry 和 Action Authorization 的通用合同。

它回答两个问题：

```text
每个 Agent role 拥有哪些 Skill？
每个 Skill 允许执行哪些 Core Action，并要求哪些输出和证据？
```

## 2. 权威边界

Core Skill Registry 的权威来源：

```text
crates/ontology/src/skill.rs
docs/architecture/057-core-skill-registry-action-authorization-v1.md
release-gate runtime/core-skill-registry.json
```

本合同不使用 Software Dev 专有对象名。Software Dev Pack 可以把 Skill 映射成 issue 执行、PR 检查、release 交付、repository patch 或 test log 等行业词，但这些映射不是 Core authority。

Machine-readable boundary phrase: reference mappings are not Core authority.

## 3. Core Skill 字段

每个 Skill 必须包含：

| Field | 说明 |
| --- | --- |
| skillId | 稳定 Skill ID |
| ownerRole | 拥有该 Skill 的 Agent role |
| allowedActions | 允许执行的 Core Action |
| allowedToolScopes | 允许访问的本地工具范围 |
| allowedConnectorScopes | 允许访问的连接器范围 |
| expectedOutputs | 该 Skill 应产出的 Core Object / Artifact |
| requiredEvidence | 执行该 Skill 所需证据 |
| forbiddenScope | 明确禁止的范围 |

## 4. Built-in Core Skills

| Skill | Owner Role | Allowed Actions |
| --- | --- | --- |
| goal-intake-skill | goal-agent | captureObject / normalizeObject / routeObject |
| spec-boundary-skill | spec-agent | acceptObject / attachEvidence |
| work-execution-skill | work-agent | startObject / attachEvidence / attachArtifact / submitForReview / blockObject |
| delivery-record-skill | delivery-agent | attachArtifact / completeObject |
| audit-review-skill | audit-agent | submitForReview / blockObject / cancelObject / supersedeObject |
| human-decision-skill | human-owner | acceptObject / completeObject / cancelObject / supersedeObject |

## 5. 授权规则

Core Action Authorization 必须遵守：

1. Agent role 只能调用自己拥有的 Skill；
2. Skill 只能调用 `allowedActions` 内的 Core Action；
3. Skill 只能使用 `allowedToolScopes` 和 `allowedConnectorScopes` 内的能力；
4. Skill 必须声明 `expectedOutputs`；
5. Skill 必须声明 `requiredEvidence`；
6. Skill 必须声明 `forbiddenScope`；
7. Reference App mapping 只能把 Skill 转译成行业任务，不得改变 Core Skill authority。

## 6. 禁止进入 Core 的行业词

Core Skill Registry 不得要求：

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

## 7. Release Gate 证明

release-gate 必须生成：

```text
runtime/core-skill-registry.json
runtime/core-skill-registry-rust-test.log
```

证明内容必须包含：

- `agentflow-core-skill-registry.v1`；
- 6 个 built-in Core skills；
- 每个 Skill 有 owner role；
- 每个 Skill 有 allowed actions；
- 每个 Skill 有 allowed tool / connector scopes；
- 每个 Skill 有 expected outputs、required evidence、forbidden scope；
- allowed actions 均指向已定义 Core Action；
- forbidden terms 未污染 Core Skill Registry。
