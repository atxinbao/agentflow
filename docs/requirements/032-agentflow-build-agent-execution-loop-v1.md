# 032 - AgentFlow Build Agent Execution Loop V1

日期：2026-06-09
执行者：Codex

## 背景

Build Agent 现在要从“只负责写代码并创建 PR/MR”收口成完整的执行闭环。

它仍然只能执行 `issueCategory=spec` 且 `requiredAgentRole=build-agent` 的任务，不能执行审计任务，不能把外部 issue、任务、计划、队列、线程或工具状态当成任务源。

## 目标

把 Build Agent loop 固化为同一套源头规则：

1. input issue 默认 `executionPipeline`
2. Desktop 复制任务包
3. Agent manual
4. Browser Preview mock
5. 校验测试

## Build Agent Loop

Build Agent 必须按 7 个阶段执行：

1. 执行前置检测
2. 测试设计
3. Agent 执行 issue
4. 沙箱验证
5. 创建 PR/MR
6. 合并 PR/MR
7. 写回 Done

## 阶段定义

### 1. 执行前置检测

只检查当前 `input issue` 是否可以进入执行准备：

- 当前执行对象必须是 AgentFlow `.agentflow/input/issues/<issue-id>.json`
- 当前 issue 状态必须仍是 `backlog`
- 所有 `blockedBy` 前置 issue 必须已经 `done`
- 任务合同必须完整
- Panel Context Pack 必须可读，或者能由 Panel 生成
- 当前工作区不能有未提交的用户源码改动
- 必须先通过 AgentFlow 官方 run loop / runtime preflight 创建当前 run，之后才能改源码
- AgentFlow CLI 必须支持 `build-agent complete`

这一步也必须确认：

- 当前 `input issue` 是唯一任务源
- handoff package 只是这份 issue 的派生快照
- `executionPipeline` 只是这份 issue 合同里的字段，不是独立任务源
- 不使用外部 issue、任务、计划、队列、线程或工具状态作为任务权威
- GitHub/GitLab 不在这个 loop 阶段检测
- preflight 通过后，issue 先从 `backlog` 切到 `todo`，再准备进入 `in_progress`

### 2. 测试设计

从 SPEC 和当前 issue 推导测试点。

如果任务适合 TDD：

- 先新增或修改测试
- 记录失败测试结果
- 再进入实现

如果任务不适合 TDD：

- 记录不适合原因
- 明确替代验证方式，例如构建、浏览器 smoke、截图、DOM snapshot 或命令验证

状态说明：

- issue 已在 runtime preflight 通过后进入 `in_progress`
- 测试设计只是 `in_progress` 内的执行子阶段，不再单独改变 issue 对外状态

### 3. Agent 执行 issue

只按当前 issue 合同执行。

允许：

- 在 `allowedPaths` 内改代码、配置、测试或文档

禁止：

- 执行其他 issue
- 重排任务
- 写审计报告
- 写 audit findings / evidence map / traceability
- 越过任务边界

### 4. 沙箱验证

在本地受控沙箱中运行验证命令，并核对结果。

必须记录：

- stdout
- stderr
- exit code
- 测试或构建结果
- 浏览器 smoke / 截图证据，当任务需要时
- `git diff --check`

沙箱验证是交付门禁。即使前面做了 TDD，也不能跳过。

### 5. 创建 PR/MR

推送任务分支并创建 PR/MR。

PR/MR 描述必须使用 AgentFlow Build Agent PR/MR 模板。

PR/MR 描述是用户可见输出，必须按当前 `agentLocale` 输出。中文环境输出中文模板，英文环境输出英文模板。

模板来自当前 Handoff / executionPipeline，不要求 App 写入用户项目的 `.github/pull_request_template.md`。

复制任务包也必须带同一套 PR/MR 描述模板，避免 Build Agent 使用 `gh pr create --body` 或 `glab mr create --description` 时绕过模板。

PR/MR 描述必须包含：

- 任务 ID
- Source issue file
- Source SPEC
- Owner role
- Review role
- 改动范围
- 改动摘要
- 验证结果
- 风险或缺口
- Build Agent loop checklist
- 影响说明
- 回滚计划
- Review gate

Draft PR/MR 只是中间状态，不是完成状态。

### 6. 合并 PR/MR

默认先走 `auto-merge-if-eligible`，`manual-merge` 只作为自动合并不可用时的 fallback。

#### auto-merge-if-eligible

GitHub 执行：

```bash
gh pr ready
gh pr merge --auto
```

GitLab 执行：

```bash
glab mr update --ready
glab mr merge --auto-merge
```

然后轮询 PR/MR 是否 merged。

#### manual-merge fallback

如果 provider 拒绝自动合并，或仓库规则不允许自动合并：

1. 记录自动合并不可用原因。
2. 将 PR/MR 保持 ready。
3. issue 保持 `in_review`。
4. 等待人类在 GitHub 合并 PR，或在 GitLab 合并 MR。
5. AgentFlow 本地检测器检测到 PR/MR merged 后，继续写回 Done。

### 7. 写回 Done

只有确认 PR/MR merged 后，才能写回 Done。

写回内容包括：

- execute run
- evidence
- release delivery
- issue Done 状态

如果这是 project 下最后一个未完成 issue，Project Loop 继续生成 Delivery Audit Report；所有 Project issues done 且 Delivery Audits passed 后，Project Final Audit 才能把 project 状态切到 done。

## 验收标准

- 新 Spec Issue 默认包含 7 个 required execution stages。
- `test-design` 是 required stage。
- `sandbox-verify` 的显示名称为“沙箱验证”。
- Build Agent loop 从 `backlog` issue 开始，不允许跳过 preflight 直接执行。
- Preflight 通过后 issue 先进入 `todo`，runtime preflight 完成且 run 已创建后再进入 `in_progress`。
- Runtime preflight 必须确认 Context Pack 可读，并且当前工作区没有未提交的用户源码改动。
- `build-agent complete` 必须基于已有 `runId`，不能重新 create run。
- `build-agent complete` 必须校验 `agent-claim.json`、`branch.json` 和 `review/merge-proof.json`。
- 复制任务包展示 7 步 Build Agent 执行流程。
- 复制任务包包含 AgentFlow Build Agent PR 描述模板。
- PR/MR 描述模板按当前 `agentLocale` 输出，不默认固定英文。
- PR/MR 模板来自 Handoff / executionPipeline，不要求写入 `.github/pull_request_template.md`。
- 创建 PR/MR 前必须要求完成执行前置检测、测试设计和沙箱验证。
- 有未完成 `blockedBy` 依赖时，相关 issue 派生状态为 blocked，不能复制任务执行。
- GitHub/GitLab 不作为执行前置检测的 blocker。
- GitHub/GitLab 工具、认证、当前分支和工作区状态只在 PR/MR 创建或合并阶段记录。
- 默认先尝试 `auto-merge-if-eligible`，失败后才进入 `manual-merge` fallback。
- `manual-merge` fallback 不直接 Done，而是保持 `in_review`，等待检测 PR/MR merged。
- `manual-merge` fallback 同时适用于 GitHub PR 和 GitLab MR。
- `auto-merge-if-eligible` 不停在 Draft PR/MR。
- `auto-merge-if-eligible` 同时定义 GitHub `gh` 路径和 GitLab `glab` 路径。
- Agent manual 与 Desktop 任务包文案保持一致。
- Browser Preview mock 仍可构建。

## 验证命令

```bash
cargo fmt --check
cargo test -p agentflow-input
cargo test -p agentflow-agent-manual
npm --prefix apps/desktop run build
git diff --check
```
