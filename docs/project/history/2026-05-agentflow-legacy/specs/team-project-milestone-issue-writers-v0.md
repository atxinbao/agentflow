# Team / Project / Milestone / Issue Writers v0

创建日期：2026-05-27
执行者：Codex
状态：implemented / preview-first-local-writers

## Goal

本阶段把 Goal + Criteria Driven MVP 的核心创建闭环落到本地 CLI：

```text
Workspace
-> Team
-> Project
-> Milestone
-> Issue
-> .agentflow local fact source
```

所有创建命令默认只做 preview，不写 `.agentflow/`。只有用户显式传入 `--write --yes` 后才允许落盘。

## CLI

```bash
agentflow team create "<team name>"
agentflow project create "<project title>"
agentflow milestone create "<milestone title>"
agentflow issue create "<issue title>"
```

确认写入：

```bash
agentflow team create "<team name>" --write --yes
agentflow project create "<project title>" --write --yes
agentflow milestone create "<milestone title>" --write --yes
agentflow issue create "<issue title>" --write --yes
```

可选目标参数：

```bash
agentflow project create "<title>" --team-id <team-id> --status draft
agentflow milestone create "<title>" --project-id <project-id>
agentflow issue create "<title>" --team-id <team-id> --project-id <project-id> --milestone-id <milestone-id>
```

## Data Objects

新增创建对象：

```text
TeamDraft
ProjectDraft
MilestoneDraft
IssueDraft
CreationPreview
CreationPreviewFile
CreationV1ContractPreview
TeamCreationV1Preview
ProjectCharterV1Preview
MilestoneGateV1Preview
IssueContractV1Preview
CreationWriteSummary
```

`CreationPreview` 统一输出：

```text
mode
kind
entityId
title
action
writesRequired
files[]
v1Contract
confirmationGates
recommendedCommand
sources
boundary
```

## Write Rules

Team writer：

```text
.agentflow/teams/{team-id}.json
.agentflow/workspace.json teamIds append
```

Project writer：

```text
.agentflow/projects/{project-id}.json
.agentflow/workspace.json projectIds append
.agentflow/teams/{team-id}.json projectIds append
```

Milestone writer：

```text
append into .agentflow/projects/{project-id}.json milestones[]
```

Issue writer：

```text
.agentflow/issues/ISSUE-XXXX.json
.agentflow/issues/ISSUE-XXXX.md
.agentflow/projects/{project-id}.json issueIds append
.agentflow/projects/{project-id}.json milestones[].issueIds append
.agentflow/teams/{team-id}.json issueIds append
.agentflow/index.json issues append / nextIssueNumber increment
```

## v1 Product Model Alignment

当前 writer preview 已向 `Project / Milestone / Issue / View Model v1` 收敛：

- Team preview 通过 `v1Contract.team` 展示 team id、name、projectIds、issueIds、queue rule 和 boundary。
- Project preview 通过 `v1Contract.projectCharter` 展示 Project charter：goal、scope、non-goals、success criteria、milestones、issue order、validation gate、evidence requirements、queue rule、closure gate 和 boundary。
- Milestone preview 通过 `v1Contract.milestoneGate` 展示阶段门：goal、entry criteria、scope、non-goals、issues、exit criteria、validation、evidence required 和 next milestone gate。
- Issue preview 通过 `v1Contract.issueContract` 展示执行合同：goal、scope、non-goals、dependencies、Codex instructions、acceptance criteria、validation commands、evidence required、allowed / forbidden files 和 boundary。
- View writer 后置；View 只能保存 filter / sort / layout，不能写业务状态，不能保存结果，不能执行命令。

该对齐只改变 preview 输出，不立即改变落盘 schema；写入仍遵守当前 v0 本地事实源格式、canonical status 和确认门。

## Status Rules

Project 新写入 status：

```text
draft / active / paused / completed / canceled
```

默认：

```text
Project status = draft
```

Project 可以通过 `--status active` 明确写成 active，但 writer 不会隐式覆盖 `.agentflow/workspace.json.activeProjectId`。

Issue 新写入 status：

```text
todo
```

Milestone 不写产品状态，只写：

```text
id
name / title
description
sortOrder
target
issueIds
```

## Confirmation Gates

写入必须通过：

```text
preview-default
explicit-write-flag
explicit-yes-confirmation
refuse-existing-team-or-project
canonical-project-status
canonical-issue-status
no-milestone-status
local-facts-only
```

## Non-Goals

- 不自动执行 run / verify / review。
- 不调用模型。
- 不创建远程 PR / GitHub issue / Linear issue。
- 不接入 SaaS、账号、支付、云同步。
- 不覆盖已有 Team / Project。
- 不绕过 preview / confirmation gate。
- 不新增 Desktop 写入 UI。

## Desktop Boundary

Desktop 继续只读取 `LocalProjectModelSnapshot`。新创建的 Team / Project / Milestone / Issue 会在 Desktop 中展示，但 Desktop 不负责创建、编辑或执行。

## Verification

```bash
cargo fmt --check
cargo test
npm --prefix apps/desktop run build
cargo run -p agentflow-cli -- team create "Demo Team"
cargo run -p agentflow-cli -- project create "Demo Project"
cargo run -p agentflow-cli -- milestone create "Demo Milestone"
cargo run -p agentflow-cli -- issue create "Demo Issue"
cargo run -p agentflow-cli -- projects
cargo run -p agentflow-cli -- feature status
bash checks/agentflow-readiness.sh
git diff --check
```
