# Product Feature Controlled Run v0

创建日期：2026-05-26
执行者：Codex
状态：implemented / local-controlled-run

## 目标

Product Feature Controlled Run v0 把 Product Feature Creation / Execution 之后的第一条 issue 推进到受控执行入口。

它不新增 UI，不调用模型，不创建远程 PR。它只让当前 active feature project 的唯一 eligible issue 可以通过本地 dry-run 形成可追溯执行记录：

```text
Feature Project
-> Active Milestone
-> Eligible Issue
-> Lease
-> Dry Run
-> Execution Run Record
-> Validation Readiness
-> Evidence Requirement
-> Next Action
```

## CLI

```bash
agentflow run ISSUE-XXXX --dry-run
agentflow feature status
agentflow feature next
```

`agentflow run` 在 v0 只支持 `--dry-run`。不带 `--dry-run` 会失败，避免用户误以为 AgentFlow 已经进入真实代码执行器。

## Run Gate

`agentflow run ISSUE-XXXX --dry-run` 必须先通过：

- active project 检查。
- active milestone 检查。
- eligibility 检查。
- lease 检查或 acquire。
- WIP=1 / single code-changing issue 检查。

通过后写入：

- `.agentflow/leases/LEASE-*.json`
- `.agentflow/runs/RUN-XXXX/run.json`
- `.agentflow/runs/RUN-XXXX/transcript.md`
- `.agentflow/runs/RUN-XXXX/commands.jsonl`
- `.agentflow/runs/RUN-XXXX/diff-summary.md`
- `.agentflow/scope-state.json`

## Controlled Run Plan

`AgentRun.runPlan` 记录：

- goal
- non-goals
- expected files
- blocked files / areas
- planned steps
- validation commands
- evidence requirements
- rollback plan

CLI 输出同一份 run plan，便于用户在进入 verify 前确认边界。

## Feature Status / Next

dry-run 后：

- `feature status` 显示 `dry-run recorded=true`。
- `feature status` 显示 latest run plan、expected files、blocked files、validation commands、evidence requirements。
- `feature next` 从 `run` 推荐推进到 `verify`。

后续仍沿用既有链路：

```text
verify
-> review
-> evidence / review / project update
-> milestone summary
-> next milestone
```

## 禁止项

当前阶段不允许：

- 自动执行真实代码修改。
- 自动调用模型。
- 自动创建 PR / GitHub issue / Linear issue。
- 从 Desktop 执行 run。
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
cargo run -p agentflow-cli -- eligibility
cargo run -p agentflow-cli -- goal check
cargo run -p agentflow-cli -- goal next
cargo run -p agentflow-cli -- projects
cargo run -p agentflow-cli -- metrics
bash checks/agentflow-readiness.sh
test ! -d .agentflow/audits
git diff --check
```

## 验收

- `agentflow run ISSUE-0043 --dry-run` 通过 eligibility + lease gate。
- dry-run 输出明确 run plan，但不修改源码。
- run record 关联 project / milestone / issue / lease。
- `feature status` 显示 latest run 状态和 run plan。
- `feature next` 从 run 推荐到 verify。
- 全流程保持本地、受控、不自动执行远程动作。
