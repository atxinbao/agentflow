# Event Replay and Projection Rebuild V1

创建日期：2026-06-24
执行者：Codex

## 1. 目标

`Event Store` 是任务运行事实源，`Projection` 是 Desktop / CLI / SDK 的只读 read model。

本文件定义 `v0.9.0` 的 replay / rebuild 边界：

```text
.agentflow/events/**
-> replay
-> .agentflow/projections/**
-> .agentflow/indexes/**
-> Desktop / CLI / SDK readonly query
```

Projection 不能成为 authority。Replay / rebuild 只能从现有事实重新生成 read model，不能修复历史事件，不能写入 spec、task authority、release authority 或 audit authority。

## 2. 输入

Replay 输入只允许来自：

- `.agentflow/events/task-events.jsonl`
- `.agentflow/events/task-events/*.json`
- `.agentflow/spec/issues/*.json`
- `.agentflow/spec/projects/*.json`
- `.agentflow/spec/requirements/**`
- `.agentflow/audit/**` 的只读 summary / index
- `.agentflow/release/**` 的只读 facts

其中 Event Store 决定状态变化和时间线，Spec 决定任务 / 项目合同，Projection 只做合成。

## 3. 输出

Projection rebuild 可以写：

- `.agentflow/projections/tasks/<issue-id>.json`
- `.agentflow/projections/projects/<project-id>.json`
- `.agentflow/projections/requirements/<requirement-id>.json`
- `.agentflow/projections/spec-loops/<requirement-id>.json`
- `.agentflow/projections/completions/<project-id>.json`
- `.agentflow/indexes/*.json`
- `.agentflow/projections/replay-report.json`

Projection rebuild 不允许写：

- `.agentflow/spec/**`
- `.agentflow/events/**`
- `.agentflow/tasks/**`
- `.agentflow/release/**`
- `.agentflow/audit/**`
- `docs/**`

## 4. Replay Report

每次 release gate 级 replay / rebuild 都必须生成机器可读报告：

```json
{
  "version": "projection-replay-report.v1",
  "status": "passed",
  "eventCount": 12,
  "taskCount": 2,
  "projectCount": 1,
  "rebuiltPaths": [
    ".agentflow/projections/tasks/AF-001.json",
    ".agentflow/indexes/issue-status.json"
  ],
  "failures": [],
  "writesAuthority": false,
  "projectionAuthority": false,
  "generatedAt": 1780000000
}
```

失败时仍要写 report：

```json
{
  "version": "projection-replay-report.v1",
  "status": "failed",
  "failures": [
    {
      "stage": "event-replay-projection-rebuild",
      "message": "parse task event line 1"
    }
  ],
  "writesAuthority": false,
  "projectionAuthority": false
}
```

这个报告的重点不是替用户修复事件，而是明确告诉 release gate：

- replay 是否能读取事件；
- projection 是否能从事件和合同稳定重建；
- 失败发生在哪个阶段；
- Projection 没有写 authority；
- Projection 不是 authority。

## 5. Release Gate

Release gate 必须覆盖两条路径：

1. Happy path  
   在真实 runtime fixture 完成后执行 replay / rebuild。报告必须：
   - `status = passed`
   - `eventCount > 0`
   - `taskCount > 0`
   - `rebuiltPaths` 非空
   - `writesAuthority = false`
   - `projectionAuthority = false`

2. Failure path  
   构造损坏事件 fixture。报告必须：
   - `status = failed`
   - `failures` 非空
   - failure message 保留解析原因
   - `writesAuthority = false`
   - `projectionAuthority = false`

## 6. CLI Contract

CLI 提供：

```bash
agentflow projection replay-report --output <path>
```

该命令执行 replay / rebuild 并输出 `projection-replay-report.v1`。

失败报告不等于命令异常。只要报告能写出，命令可以返回 0；release gate 根据 report 内容判断 happy / failure 是否符合预期。

## 7. 非目标

- 不把 Projection 当 authority。
- 不在 replay 中修复历史事件。
- 不改写 Event Store。
- 不在 Projection 里执行 provider。
- 不把 replay report 当作审计报告。

