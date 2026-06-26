# 001 - Goal Agent / Project Brain V1

创建日期：2026-06-17
执行者：Codex

## Source Product Baseline

本文从以下产品设计基线派生：

- [../product/README.md](../product/README.md)
- [../product/001-project-agent-role-model-v1.md](../product/001-project-agent-role-model-v1.md)
- [../product/002-project-lifecycle-v1.md](../product/002-project-lifecycle-v1.md)
- [../product/003-goal-plan-document-model-v1.md](../product/003-goal-plan-document-model-v1.md)
- [../product/004-requirement-to-goal-flow-v1.md](../product/004-requirement-to-goal-flow-v1.md)
- [../product/005-goal-to-project-loop-flow-v1.md](../product/005-goal-to-project-loop-flow-v1.md)

## Purpose

Goal Agent / Project Brain V1 是 AgentFlow 下一代项目模型的第一刀基础能力。

它的目标不是执行任务，而是建立项目大脑层：

```text
读取项目目标
读取项目计划
读取项目决策
判断项目方向状态
生成下一步建议草稿
等待用户确认
```

## User Goal

用户添加一个 Project 后，需要有一个 Agent 作为项目第二大脑，持续从项目大局判断：

- 当前项目要达成什么？
- 当前计划是否服务于目标？
- 是否需要补充目标或计划？
- 是否出现范围变化？
- 下一步应该进入规划、执行、审计、交付，还是继续澄清？

## Scope

V1 只定义和实现 Project Brain 的基础边界。

### In Scope

- 定义 Project Brain 文档集合。
- 定义 GOAL.md / PLAN.md / DECISIONS.md 的本地位置。
- 定义 Project Brain 状态读取。
- 定义缺失文档时的草稿预览。
- 定义 Project Brain Snapshot。
- 定义只读状态输出。
- 定义用户确认前不写入的边界。
- 为后续 Desktop Project Brain 展示预留数据结构。

### Out of Scope

- 不调用模型。
- 不自动生成 Issue。
- 不进入 Work Loop。
- 不执行任务。
- 不审计结果。
- 不生成 Delivery Report。
- 不创建远程 PR。
- 不接入 GitHub / Linear。
- 不写 `.agentflow/` 运行态数据。
- 不修改现有 `docs/requirements/` 队列。

## Project Brain Documents

建议项目大脑文档位置：

```text
docs/projects/<project-id>/GOAL.md
docs/projects/<project-id>/PLAN.md
docs/projects/<project-id>/DECISIONS.md
```

如果未来需要机器可读摘要，可派生：

```text
.agentflow/spec/projects/<project-id>.json
```

但 V1 不要求写入 `.agentflow/`。

## Document Responsibilities

### GOAL.md

负责项目方向：

- 项目目标
- 用户 / 受众
- 预期结果
- 范围
- 非目标
- 成功标准
- 当前目标判断

### PLAN.md

负责项目路径：

- 阶段计划
- 当前阶段
- 下一步建议
- 验证策略
- 证据策略
- 风险和阻塞
- 当前计划判断

### DECISIONS.md

负责项目确认记录：

- Goal 确认
- Plan 确认
- Scope change
- Risk acceptance
- Pause / resume
- Delivery acceptance

## Project Brain Snapshot

V1 建议定义只读快照：

```text
ProjectBrainSnapshot
```

字段：

```text
projectId
projectTitle
projectPath
goalDocument
planDocument
decisionsDocument
goalStatus
planStatus
decisionStatus
brainStatus
missingDocuments
openQuestions
nextRecommendedAction
readonly
```

### Status Values

```text
missing
draft
needs-confirmation
confirmed
stale
blocked
```

### Brain Status

```text
not-initialized
needs-goal
needs-plan
needs-confirmation
ready-for-project-loop
needs-recheck
blocked
```

## Preview-first Rule

如果 GOAL.md / PLAN.md / DECISIONS.md 缺失：

- 系统可以生成草稿预览。
- 用户确认前不写入。
- 草稿不能直接生成 Issue。
- 草稿不能进入 Work Loop。

确认后才允许写入项目文档。

## Suggested Local Commands

V1 可考虑后续提供只读命令：

```text
agentflow project brain status
agentflow project brain preview
```

但本文不要求立即实现 CLI。

## Desktop Boundary

未来 Desktop 可以展示 Project Brain：

- 当前 Goal 状态
- 当前 Plan 状态
- 缺失文档
- 待确认问题
- 下一步建议

Desktop 不允许：

- 不执行任务。
- 不直接写 `.agentflow/`。
- 不绕过确认写 GOAL.md / PLAN.md。
- 不生成 Issue。

## Acceptance Criteria

- [ ] Project Brain 文档位置明确。
- [ ] GOAL.md / PLAN.md / DECISIONS.md 职责明确。
- [ ] ProjectBrainSnapshot 字段明确。
- [ ] 缺失文档时的 preview-first 规则明确。
- [ ] 用户确认前不写入。
- [ ] 不进入 Work Loop。
- [ ] 不生成 Issue。
- [ ] 不调用模型。
- [ ] 不修改 `docs/requirements/` 当前迭代队列。

## Future Slices

后续可继续拆：

```text
002-project-brain-document-store-v1
003-requirement-to-goal-draft-v1
004-plan-draft-preview-v1
005-project-brain-desktop-view-v1
006-project-brain-confirmation-gate-v1
```
