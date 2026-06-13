## Agent Provider Bridge

创建日期：2026-06-13  
执行者：Codex

## 结论

`crates/mcp` 现在承担 AgentFlow 的外部 Agent Provider Bridge。

它不是任务编排层，也不是状态机本身。它只负责：

- 接外部 Agent provider；
- 统一 provider 能力探测；
- 统一 launch / session / poll / cancel / logs 的抽象；
- 把 provider 侧会话状态投影回 AgentFlow 可读事实。

`loop` 负责决定哪条任务该跑。  
`execute` 负责 run、preflight、delivery、writeback。  
`mcp` 负责“找谁跑、怎么接、怎么记外部会话”。

## 为什么不单独再起一个 agent-bridge crate

当前仓库已经有 `crates/mcp`，而且 `033-agentflow-v0.2.0-project-loop-issue-loop-mvp-v1.md`
已经把它定义成：

- GitHub
- GitLab
- Codex
- Browser Preview

等外部 provider 的统一适配层。

继续新建第二个 bridge crate，只会把边界拆散。

所以本轮选择：

- 保留 crate 名称：`agentflow-mcp`
- 明确业务角色：`Agent Provider Bridge`
- 后续把外部 coding agent 也并入这一层

## 模块边界

### crates/loop

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

### crates/execute

负责：

- create run
- runtime preflight
- lease
- checkpoint
- validation
- evidence
- release delivery
- done writeback

不负责：

- 选择 Codex / Claude / 其他 provider
- 管理 provider 会话生命周期

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

- provider bridge 创建会话成功后，正式 claim 当前 launch，并写 `build-agent.launch.claimed`；
- session 轮询第一次观察到运行态时，写 `build-agent.session.running`；
- `agentflow build-agent prepare-review` 写 `build-agent.session.review-ready`；
- `agentflow build-agent write-merge-proof --merged` 写 `build-agent.merge.confirmed`；
- `agentflow build-agent complete` 写 `build-agent.writeback.completed`。

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
