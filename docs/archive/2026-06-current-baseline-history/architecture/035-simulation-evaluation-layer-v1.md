# Simulation Evaluation Layer V1

创建日期：2026-06-24  
执行者：Codex

## 1. 目标

Simulation 不只是判断一个命令能不能 dry-run。

它必须在真实执行前回答：

```text
会影响哪些对象？
需要哪些证据？
会触发哪些状态变化？
会产生哪些后续事件？
有哪些冲突或阻断？
会影响哪些 gate？
```

## 2. 只读边界

Simulation 永远只读：

```text
writesAuthority = false
writesEventStore = false
executesProvider = false
```

它不能：

- 写 authority；
- append event；
- rebuild projection；
- 启动 provider；
- 把 simulation output 当成执行结果；
- 用生成内容替代 evidence。

## 3. 统一报告字段

`SimulationReport` 必须包含以下一等字段：

| 字段 | 说明 |
| --- | --- |
| `affectedObjects` | 将被当前动作影响的对象 |
| `affectedProjections` | 会被刷新或读取的 projection |
| `requiredEvidence` | 执行或完成前需要的证据 |
| `stateTransitions` | 预期状态变化 |
| `downstreamTriggers` | 可能触发的后续事件或流程 |
| `conflicts` | 冲突 scope、锁、阻断预览 |
| `gateImpact` | 会通过、阻断或需要人工判断的 gate |

## 4. Pack Command Simulation

Pack command simulation 是 release gate 的最小覆盖点。

每个 Pack command dry-run 至少要证明：

- 不写 authority；
- 不写 event store；
- 不启动 provider；
- 能说明 target object；
- 能说明 required evidence；
- 能说明 state transition；
- 能说明 downstream trigger；
- 能说明 conflict preview；
- 能说明 gate impact。

## 5. Issue / Completion Simulation

Issue simulation 必须能说明：

- 当前 issue 是否能从 `todo` 进入 `in_progress`；
- context pack 是否可用；
- workspace 是否干净；
- Build Agent launch 是否会被触发。

Completion simulation 必须能说明：

- issue 是否能从 `in_review` 进入 `done`；
- validation evidence 是否存在；
- delivery artifact 是否存在；
- merge proof 是否存在；
- completion 后是否需要 projection rebuild；
- audit trigger 是否只做独立评估。

## 6. Release Gate 要求

Release gate 不能只检查 `simulation.status = passed`。

它还必须检查至少一个 Pack command simulation report 同时具备：

```text
affectedObjects
requiredEvidence
stateTransitions
downstreamTriggers
conflicts
gateImpact
```

否则 release gate 必须失败。

## 7. 实现位置

```text
crates/simulation/src/lib.rs
crates/cli/src/main.rs
scripts/verify_release_gate.sh
```

## 8. 非目标

本层不做：

- 真实执行；
- provider launch；
- projection rebuild；
- authority write；
- 生成 evidence；
- 自动修复冲突。
