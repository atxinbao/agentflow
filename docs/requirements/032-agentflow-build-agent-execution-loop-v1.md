# 032 - AgentFlow Build Agent Execution Loop V1

日期：2026-06-09
执行者：Codex

## 背景

Build Agent 现在要从“只负责写代码并创建 PR”收口成完整的执行闭环。

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

1. GitHub 自动化预检
2. 测试设计
3. Agent 执行 issue
4. 沙箱验证
5. 创建 PR
6. 合并 PR
7. 写回 Done

## 阶段定义

### 1. GitHub 自动化预检

检查工具和权限是否满足后续自动化：

- GitHub CLI 可用
- GitHub 认证可用
- 远端仓库正确
- 当前分支和工作区状态安全
- PR 创建能力可用
- `mergeMode` 可判断
- AgentFlow CLI 支持 `build-agent complete`

这一步也必须确认：

- AgentFlow input issue 和 `executionPipeline` 是唯一任务源
- 不使用外部 issue、任务、计划、队列、线程或工具状态作为任务权威

### 2. 测试设计

从 SPEC 和当前 issue 推导测试点。

如果任务适合 TDD：

- 先新增或修改测试
- 记录失败测试结果
- 再进入实现

如果任务不适合 TDD：

- 记录不适合原因
- 明确替代验证方式，例如构建、浏览器 smoke、截图、DOM snapshot 或命令验证

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

在本地受控环境运行验证命令，并核对结果。

必须记录：

- stdout
- stderr
- exit code
- 测试或构建结果
- 浏览器 smoke / 截图证据，当任务需要时
- `git diff --check`

沙箱验证是交付门禁。即使前面做了 TDD，也不能跳过。

### 5. 创建 PR

推送任务分支并创建 PR。

PR 描述必须使用 AgentFlow Build Agent PR 模板。

标准模板文件：

```text
.github/pull_request_template.md
```

复制任务包也必须带同一套 PR 描述模板，避免 Build Agent 使用 `gh pr create --body` 时绕过模板。

PR 描述必须包含：

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

Draft PR 只是中间状态，不是完成状态。

### 6. 合并 PR

按 `mergeMode` 处理：

#### manual-merge

Build Agent 将 PR 转 Ready 后进入 `waiting-for-merge`。

人类在 GitHub 合并 PR。

AgentFlow 本地检测器检测到 PR merged 后，继续写回 Done。

#### auto-merge-if-eligible

Build Agent 执行：

```bash
gh pr ready
gh pr merge --auto
```

然后轮询 GitHub，直到 PR 进入 merged 状态。

如果 GitHub 拒绝 auto-merge，Build Agent 必须报告原因并停在 PR ready。

### 7. 写回 Done

只有确认 PR merged 后，才能写回 Done。

写回内容包括：

- execute run
- evidence
- release delivery
- issue Done 状态

如果这是 project 下最后一个未完成 issue，project 派生状态可以变为完成。

## 验收标准

- 新 Spec Issue 默认包含 7 个 required execution stages。
- `test-design` 是 required stage。
- `sandbox-verify` 的显示名称为“沙箱验证”。
- 复制任务包展示 7 步 Build Agent 执行流程。
- 复制任务包包含 AgentFlow Build Agent PR 描述模板。
- `.github/pull_request_template.md` 存在并使用 AgentFlow 任务语义。
- 创建 PR 前必须要求完成 GitHub 自动化预检、测试设计和沙箱验证。
- `manual-merge` 不直接 Done，而是进入 `waiting-for-merge`，等待检测 PR merged。
- `auto-merge-if-eligible` 不停在 Draft PR。
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
