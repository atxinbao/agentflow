# Product Feature Execution Flow v0

创建日期：2026-05-26
执行者：Codex
状态：implemented / local-feature-execution

## 目标

Product Feature Execution Flow v0 接在 Product Feature Creation Flow v0 后面。

它让用户创建 feature project 后，可以清楚看到当前产品功能应该推进哪条 issue，以及下一步应该执行哪个本地命令。

该阶段不新增 UI，不做 Linear clone，不新建执行引擎，只复用现有 workflow control：

```text
IssueContract
-> eligibility
-> lease
-> run
-> verify
-> review
-> evidence
-> milestone summary
-> next milestone
```

## CLI

```bash
agentflow feature status
agentflow feature next
```

`feature status` 展示当前 active Product Feature Project 的完整状态。

`feature next` 只展示下一步决策，不执行命令。

## Status 输出

`feature status` 读取：

- `.agentflow/workspace.json`
- `.agentflow/projects/{active-project-id}.json`
- `.agentflow/issues/*.json`
- `.agentflow/runs/*/run.json`
- `.agentflow/state/eligibility.json`

展示：

- project id / title / goal
- project canonical status
- active milestone
- milestone 列表和 derived progress
- 当前 issue
- issue canonical status
- ready / eligible / leased 状态
- latest run / validation / evidence / review
- recommended command
- dry-run recorded
- latest run plan
- expected files / blocked files
- validation commands / evidence requirements

## Next 决策

`feature next` 只做本地决策：

```text
active Product Feature Project ready?
-> active milestone exists?
-> active milestone has exactly one open issue?
-> current issue eligible?
-> issue_next_step:
     no run -> run
     run without validation -> verify
     validation passed without evidence/review -> review
     completed -> next milestone
```

它不自动执行 `run`、`verify` 或 `review`。

## Milestone 推进

Milestone 推进仍由现有 `agentflow review ISSUE-XXXX` 完成：

```text
review
-> evidence
-> review artifact
-> project update
-> issue completed
-> milestone summary if complete
-> activate next planned milestone
```

Product Feature Execution Flow v0 只读取这个结果并推荐下一步。

## Controlled Run 接入

Product Feature Controlled Run v0 强化 `agentflow run ISSUE-XXXX --dry-run`：

- run 前检查 active project / active milestone / eligibility / lease。
- dry-run 生成 `AgentRun.runPlan`。
- run plan 包含 goal、expected files、blocked files / areas、planned steps、validation commands、evidence requirements 和 rollback plan。
- `feature status` 展示 latest run plan。
- `feature next` 在 dry-run 后推荐 verify。

该能力仍不执行真实代码修改，不调用模型，不创建远程对象。

## 禁止项

当前阶段不允许：

- 自动执行 run / verify / review。
- 调用模型。
- 创建远程 PR / GitHub issue / Linear issue。
- 从 Desktop 执行命令。
- 绕过 IssueContract。
- 标记 Project done。
- 创建 `.agentflow/audits/`。

## 验证矩阵

```bash
cargo fmt --check
cargo test
npm --prefix apps/desktop run build
cargo run -p agentflow-cli -- feature status
cargo run -p agentflow-cli -- feature next
cargo run -p agentflow-cli -- run ISSUE-0043 --dry-run
cargo run -p agentflow-cli -- goal check
cargo run -p agentflow-cli -- goal next
cargo run -p agentflow-cli -- eligibility
cargo run -p agentflow-cli -- projects
cargo run -p agentflow-cli -- metrics
cargo run -p agentflow-cli -- search "Product Feature Execution"
bash checks/agentflow-readiness.sh
test ! -d .agentflow/audits
git diff --check
```

## 验收

- 当前 `feature-0043` 下，`feature next` 推荐 `agentflow run ISSUE-0043 --dry-run`。
- issue 已 run 时，推荐 `agentflow verify ISSUE-XXXX`。
- verify 通过后，推荐 `agentflow review ISSUE-XXXX`。
- review 后，进入下一个 milestone 的第一条 issue。
- 全流程只推荐，不自动执行，不调用模型，不创建远程对象。
