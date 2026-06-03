# 008.1 - Agent Working Manual Health Polish

创建日期：2026-06-03
执行者：Codex

## 背景

PR #13 已完成 `008 - Agent Working Manual Bootstrap V1`：

- 新增 `crates/agent-manual`
- 接管项目根目录 `AGENT.MD`
- 写入 `.agentflow/define/agent/Agentflow.md`
- 写入 5 个内置 skills
- 写入 `.agentflow/define/agent/skills-lock.json`
- Project Workspace prepare 已接入 Agent Manual bootstrap
- Desktop 状态通道已显示 `工作手册`
- `AGENT.MD` 缺失 / 已存在 / Git tracked / 项目外 symlink / skill hash mismatch 等已处理

当前还需要补齐两个健康闭环小问题：

1. `validate_agent_working_manual` 需要明确检查：
   - `.agentflow/define/agent/state/bootstrap.json`
   - `.agentflow/define/agent/state/validation.json`
2. `AGENT.MD` 如果是 symlink 且目标仍在 project root 内，可以继续通过，但必须记录 warning。

## 用户目标

Agent 工作手册健康状态不能在 state 文件缺失时误判为 ready；项目内 `AGENT.MD` symlink 允许存在，但必须可观测。

## 范围

- 更新 `crates/agent-manual` validate 健康检查。
- 更新 `load_agent_environment_status` 缓存读取边界，避免只剩旧 validation cache 时误读。
- 增加单元测试覆盖：
  - 缺失 `bootstrap.json`。
  - 缺失 `validation.json`。
  - `AGENT.MD` 是项目内 symlink。

## 非目标

- 不新增 OpenSpec / Goal Tree / AgentRun 代码。
- 不改 Desktop 页面结构。
- 不新增执行命令能力。
- 不调用模型。
- 不写用户项目源码。
- 不改 `.agentflow/` runtime 目录结构。

## 页面 / 功能

- `agentflow-agent-manual`
- Project Workspace prepare 间接使用的 Agent Manual health status

## 数据来源

- `AGENT.MD`
- `.agentflow/define/agent/Agentflow.md`
- `.agentflow/define/agent/skills-lock.json`
- `.agentflow/define/agent/skills/**/SKILL.md`
- `.agentflow/define/agent/state/bootstrap.json`
- `.agentflow/define/agent/state/validation.json`

## 交互边界

- validate 只读检查。
- repair / prepare 仍按 008 已授权路径修复 Agent Manual 管理文件。
- 项目内 symlink 只 warning，不 blocker。
- 项目外 symlink 仍 blocker。

## 验收标准

- [ ] 删除 `bootstrap.json` 后，`validate_agent_working_manual` 不再 ready，并报告缺失。
- [ ] 删除 `validation.json` 后，`validate_agent_working_manual` 不再 ready，并报告缺失。
- [ ] `AGENT.MD` 是项目内 symlink 时，状态 ready/degraded 可继续，但 warnings 中记录该事实。
- [ ] `AGENT.MD` 是项目外 symlink 时仍 blocked。
- [ ] 不引入 OpenSpec / Goal Tree / AgentRun 新代码。

## 验证命令

- `cargo fmt --check`
- `cargo test -p agentflow-agent-manual`
- `cargo test`
- `npm --prefix apps/desktop run build`
- `git diff --check`
