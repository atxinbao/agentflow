# Product Feature Creation Flow v0

创建日期：2026-05-26
执行者：Codex
状态：implemented / local-product-entry

## 目标

Product Feature Creation Flow v0 是 AgentFlow 的第一个真正产品功能入口。

它不是 Linear clone，也不是完整项目管理系统。它只解决一个最小闭环：

```text
Human 输入产品功能目标
-> agentflow feature create preview
-> agentflow feature create --write --yes
-> Project
-> Milestones
-> IssueContracts
-> goal next
-> eligibility
-> run / verify / review
```

## CLI

```bash
agentflow feature create "<feature goal>"
agentflow feature create "<feature goal>" --write --yes
```

默认只 preview，不写 `.agentflow/`。只有同时传入 `--write --yes` 才会写事实源。

支持输入：

- `feature goal`
- `--team-id`，默认 `core`
- `--project-title`
- `--non-goal`
- `--success-criterion`
- `--risk-level`，默认 `medium`
- `--scope-boundary`

## 数据对象

新增对象：

- `ProductFeatureDraft`
- `ProductFeatureProject`
- `ProductFeatureMilestoneDraft`
- `ProductFeatureIssueDraft`
- `ProductFeatureCreationSnapshot`

这些对象只描述本地 feature creation 的草案、预览和写入结果，不调用模型，不连接远程系统。

## 默认拆分

v0 使用确定性默认拆分，不做模型自动拆解：

```text
Project Charter
Milestone Plan
Issue Contracts
Validation / Evidence
```

每个 milestone 对应一条 IssueContract。Project 写入后默认为 `active`，因为它会立即成为 `workspace.activeProjectId`。Issue 默认状态为 `todo`。Milestone 不作为产品状态机展示，只作为阶段分组和完成度派生来源。

## 写入产物

确认写入后生成或更新：

```text
.agentflow/projects/{feature-project-id}.json
.agentflow/issues/ISSUE-XXXX.json
.agentflow/issues/ISSUE-XXXX.md
.agentflow/teams/{team-id}.json
.agentflow/workspace.json
.agentflow/index.json
.agentflow/updates/FEATURE-CREATION-SUMMARY.md
```

写入后：

- 新 Project 成为 `workspace.activeProjectId`。
- 新 Project 写入 `status = active`。
- Project 包含 4 个 milestones。
- Project active milestone 为 `project-charter`。
- 每个 issue 都写入 `projectLink`。
- 每个 issue 写入 `status = todo`。
- 每个 issue 都包含 `scope`、`nonGoals`、`validation`、`evidenceRequirements`、`rollbackPlan`、`riskLevel`。

## GoalLoop / Eligibility 接入

写入后：

```bash
agentflow goal next
agentflow eligibility
```

应能读取新 active project / active milestone。

规则：

- `goal next` 只推荐第一条 issue 的下一步命令，不自动执行。
- `eligibility` 只计算 ready / eligible，不允许手写 eligible 状态。
- WIP=1 继续生效。
- `run` 仍必须先通过 eligibility 和 lease。

## 禁止项

当前阶段不允许：

- 调用模型自动拆解功能。
- 创建远程 PR / GitHub issue / Linear issue。
- 从 Desktop 执行创建。
- 绕过 IssueContract。
- 批量迁移历史 issue。
- 修改 closure final approval。
- 标记 Project done。
- 写 `.agentflow/audits/`。

## 验证矩阵

```bash
cargo fmt --check
cargo test
npm --prefix apps/desktop run build
cargo run -p agentflow-cli -- feature create "示例产品功能"
cargo run -p agentflow-cli -- feature create "示例产品功能" --write --yes
cargo run -p agentflow-cli -- goal check
cargo run -p agentflow-cli -- goal next
cargo run -p agentflow-cli -- eligibility
cargo run -p agentflow-cli -- projects
cargo run -p agentflow-cli -- metrics
cargo run -p agentflow-cli -- search "Product Feature Creation"
bash checks/agentflow-readiness.sh
test ! -d .agentflow/audits
git diff --check
```

验收通过时，AgentFlow 具备第一个可用产品功能入口：Human 可以从 CLI 创建产品功能目标，并把它落成本地 Project -> Milestones -> IssueContracts，再进入现有 workflow control core。

## 后续执行入口

Product Feature Execution Flow v0 已接在本入口之后：

```bash
agentflow feature status
agentflow feature next
```

写入 feature project 后，用户不需要直接读 JSON；可以用 `feature status` 查看当前 active Product Feature Project、active milestone、当前 issue、eligibility、latest run / validation / evidence / review，用 `feature next` 查看下一条推荐命令。该入口仍只推荐，不执行。
