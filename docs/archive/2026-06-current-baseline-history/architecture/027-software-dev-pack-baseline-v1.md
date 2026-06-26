# Software Dev Pack Baseline V1

创建日期：2026-06-23
执行者：Codex

## 目标

Software Dev Pack 是 AgentFlow 的第一个正式行业壳。

它把当前软件开发 Project OS 现场表达成 Pack 定义：

```text
Requirement
-> Spec
-> Issue
-> Run
-> Acceptance
-> Delivery
-> Release
```

Audit 不是主链阻塞步骤，只作为独立 sidecar：

```text
Delivery / Done
-> Optional Audit Request
-> Audit Report
-> Finding
-> Follow-up Proposal
```

## Pack 边界

Software Dev Pack 只能定义：

- 软件开发领域对象；
- 对象关系；
- 状态机；
- action semantics；
- acceptance semantics；
- evidence policy；
- surface pages；
- command entries；
- connector capabilities；
- readiness artifact。

Software Dev Pack 不能：

- 写 `.agentflow/spec/**`；
- 写 `.agentflow/events/**`；
- 写 `.agentflow/tasks/**`；
- 直接创建 PR/MR；
- 直接执行 provider；
- 把 Audit 放入主业务阻塞链。

## Domain Baseline

Software Dev Domain 至少包含：

```text
Requirement
Spec
Issue
Run
Acceptance
Delivery
PullRequest
Release
Evidence
Audit
Finding
FollowUpProposal
```

主链关系：

```text
Requirement -> Spec
Spec -> Issue
Issue -> Run
Run -> Acceptance
Acceptance -> Delivery
Delivery -> Release
```

sidecar 关系：

```text
Delivery -> Audit
Audit -> Finding
Finding -> FollowUpProposal
```

## Surface Baseline

主 Surface：

```text
Project Home
Spec Workbench
Task Workbench
Acceptance
Delivery
Event Timeline
Evidence Graph
```

sidecar Surface：

```text
Audit Surface
Finding Review
Follow-up Proposal
```

命令入口必须挂到对应页面：

| Surface | Command |
| --- | --- |
| Spec Workbench | `spec.intake.start` |
| Task Workbench | `work.issue.start` |
| Acceptance | `acceptance.evaluate` |
| Delivery | `delivery.open` |
| Audit Surface | `audit.request.sidecar` |

## Connector Baseline

Software Dev Pack 初始 connector：

```text
GitHub
Git
Codex
Claude
Browser Preview
```

这些 connector 只能提供 capability 和 evidence output，不得写 Runtime authority。

## Readiness Artifact

Software Dev Pack readiness artifact 必须证明：

- `canLoad = true`；
- `canValidate = true`；
- `canProject = true`；
- `writesAuthority = false`；
- 主链完整；
- Audit sidecar 不阻塞主链；
- Finding 只能生成 follow-up proposal；
- projection source refs 可追溯到 `projection.pack-industry-workbench`。

## 实现位置

- `crates/pack/src/domain.rs`
- `crates/pack/src/surface.rs`
- `crates/pack/src/connector.rs`
- `crates/pack/src/validation.rs`
- `crates/projection/src/query.rs`

## 验收

- Software Dev Pack 能表达现有软件开发 Project OS 现场；
- Audit Surface 标记为 sidecar；
- Finding 只能生成 Follow-up Proposal；
- readiness artifact 能证明 Pack 可加载、可验证、可投影；
- Projection 能看到 Acceptance / Delivery / Audit sidecar。
