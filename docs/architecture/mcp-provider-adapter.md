## MCP Provider Adapter

创建日期：2026-06-13  
执行者：Codex

## 结论

`crates/mcp` 现在承担 AgentFlow 的外部 provider adapter。

它不是任务编排层，也不是状态机本身。它只负责：

- 接外部 Agent provider；
- 统一 provider 能力探测；
- 统一 launch / session / poll / cancel / logs 的抽象；
- 把 provider 侧会话状态投影回 AgentFlow 可读事实。

`task-loop` 负责决定哪条任务该跑。
`agent-dispatcher` 负责消费 launch event，并把任务启动请求派发给 provider adapter。
`mcp` 负责“provider 怎么接、怎么启动、怎么轮询、怎么读取日志”。

## 为什么保留 agent-dispatcher

`agent-dispatcher` 和 `mcp` 不承担同一层职责。

`agent-dispatcher` 是 AgentFlow 内部事件消费层：

- 只消费 `agent.launch.requested`；
- 把 `task-loop` 生成的 launch payload 转成 `McpLaunchRequest`；
- 调用 `mcp` provider 创建 session；
- 写 `agent.session.*` 事件。

`mcp` 是外部 provider 适配层：

- GitHub
- GitLab
- Codex
- Browser Preview

等外部 provider 的统一抽象和存储。

两者分开后，`mcp` 不需要理解 Project Loop / Issue Loop，`agent-dispatcher` 也不需要知道每个 provider 的具体命令和日志格式。

## 命名规则

- `crates/agent-dispatcher`：AgentFlow launch event dispatcher。
- `crates/mcp`：provider adapter / session abstraction。
- `McpProviderBridge`：`mcp` 内部 provider registry，不代表 AgentFlow 任务调度层。

## 模块边界

### crates/task-loop

负责：

- project loop
- issue loop
- backlog -> todo -> in_progress -> in_review -> done
- 哪条 issue 可以进入 runtime
- 什么时候发 launch event

不负责：

- 直接调用外部 Agent CLI
- 管理 provider session
- 解释 provider 的日志格式

### crates/agent-dispatcher

负责：

- 消费 `agent.launch.requested`
- 领取尚未 claim 的 run
- 调用 `mcp` provider 创建 session
- 写 `agent.session.created` / `agent.session.running` 等事件

不负责：

- 选择 Codex / Claude / 其他 provider
- provider 健康检查
- provider 命令格式
- issue 状态调度

### crates/mcp

负责：

- provider health
- provider capability
- launch request normalization
- session snapshot persistence
- launch plan generation
- provider-specific command mapping

不负责：

- 推进 AgentFlow issue 状态机
- 改写 input issue 顺序
- 替代 execute 写回 run / delivery / done

## 事件流

当前外部执行链路，统一收口为：

1. `input.issue.ready`
2. `panel.context-pack.requested`
3. `panel.context-pack.ready`
4. `build-agent.launch.requested`
5. `build-agent.launch.claimed`
6. `build-agent.session.running`
7. `build-agent.session.review-ready`
8. `build-agent.merge.confirmed`
9. `build-agent.writeback.completed`
10. `project.issue.next`

说明：

- 前端刷新不是流程驱动器。
- `loop` 和 `workflow-events` 才是流程驱动器。
- `mcp` 只消费 launch 相关事件和 provider 状态。

## Provider 抽象

Provider 抽象只定义统一能力，不绑死某个外部 Agent：

- `check_health`
- `build_launch_plan`
- `create_session`
- `poll_session`
- `cancel_session`

第一版先不要求所有 provider 都支持真正的远程控制。

允许分层：

- health only
- launch plan only
- launch + session persistence
- full launch + poll + cancel + logs

## 第一版 session 事实

Provider session 事实写在：

```text
.agentflow/state/mcp/providers/*.json
.agentflow/state/mcp/plans/*.json
.agentflow/state/mcp/sessions/*.json
```

其中：

- `providers/*.json`：provider 健康与能力
- `plans/*.json`：某次 launch 的命令计划
- `sessions/*.json`：某次外部会话的当前快照

## Codex Provider 当前实现

当前 `codex` provider 已经做四件事：

1. 检测 `codex exec` 和 `agentflow build-agent complete` 是否可用；
2. 基于 AgentFlow launch request 生成一份可执行的 Codex CLI launch plan；
3. 真正拉起外部 `codex` 会话，并把 pid / log / session snapshot 写回 `.agentflow/state/mcp/`；
4. 轮询 input issue、launcher state、merge proof 和进程存活状态，把 session 生命周期投影成：
   - `queued`
   - `starting`
   - `running`
   - `in-review`
   - `done`
   - `failed`

同时支持：

- 会话日志读取；
- PR / merge 状态回写到 session snapshot；
- 前端任务页 / 执行页只读展示 session 状态和日志。

补充约束：

- provider 是否允许拉起外部会话，只看 `launch` capability；
- `build-agent complete` 缺失或本地 binary 过旧，只记 warning，不阻断会话创建；
- `codex exec` 默认使用隔离参数启动，避免读取本机用户配置或外部规则，把 AgentFlow 任务包保持为唯一执行源。

这轮再补了一层生命周期收口：

- `agent-dispatcher` 创建会话成功后，写 `agent.session.created`；
- session 轮询第一次观察到运行态时，写 `agent.session.running`；
- `agentflow build-agent prepare-review` 推进 issue 到 review；
- `agentflow build-agent write-merge-proof --merged` 写合并证明；
- `agentflow build-agent complete` 写 Done。

这样 `.agentflow/events`、`launcher/worker-state.json`、`state/mcp/sessions/*.json` 和
input issue / execute / output 的事实会同步收口到同一条链路：

```text
queued
→ launch.claimed
→ session.running
→ session.review-ready
→ merge.confirmed
→ writeback.completed
→ done
```

仍然不做：

- 代替 `execute` 写回 run / delivery / done；
- 代替 `loop` 决定哪条 issue 可以启动；
- 解析所有 Codex 内部细粒度事件并把它们升级成 AgentFlow 状态 authority。

## Claude / 其他 provider 如何进来

后续新增 provider 时，不改 `loop` 状态机。

只新增：

- `src/<provider>.rs`
- provider 注册
- provider-specific launch / session poll / cancel / logs

这样能保证：

- 任务 authority 始终是 `.agentflow/input/issues/**`
- 状态 authority 始终是 `loop + execute`
- 外部 Agent 只是执行承载，不反向决定 AgentFlow 工作流
