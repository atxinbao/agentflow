# Architecture

创建日期：2026-05-21
最近压缩：2026-05-22
执行者：Codex

## Goal Execution Spine

AgentFlow 的底层不是 UI，而是把 `/goal` 编译成可验证工程闭环：

```text
GoalSource
-> GoalCompiler
-> ProjectDefinitionBuilder
-> ScopeStateManager
-> LocalProjectStore
-> ContextCollector
-> Planner
-> IssueContractBuilder
-> CodexRuntimeAdapter
-> ValidationRunner
-> EvidenceChain
-> ReviewAssistant
-> ProjectUpdateGenerator
-> GoalLoopOrchestrator
```

## 技术栈

| 层 | 技术 | 说明 |
| --- | --- | --- |
| Core | Rust | 对象、文件、命令、规则 |
| CLI | Rust | `init/goal/plan/run/verify/review` |
| Desktop | Tauri 2 | 后续体验壳 |
| UI | React + TypeScript | 状态、表单、evidence |
| Fact Source | Markdown + JSON | `.agentflow/` |
| Index | SQLite | 可重建索引 |

## 领域对象

| 对象 | 事实源 | 规则 |
| --- | --- | --- |
| ProjectGoal | `goal.{md,json}` | `/goal` 编译结果 |
| ProjectDefinition | `project-definition.json`, `bootstrap/*` | AEP 第一阶段初始化合同 |
| AgentScopeState | `scope-state.json` | WIP=1、active issue、执行授权状态 |
| ProjectEnvironment | `environment.md`, `settings.json` | 运行规则 |
| ProjectMap | `architecture.md` | repo / module map |
| CapabilityRoadmap | `roadmap.md`, `index.json` | 候选顺序，不授权执行 |
| IssueContract | `issues/*` | 执行唯一授权 |
| ExecutionRun | `runs/*` | dry-run / apply 记录 |
| ValidationResult | `runs/*` | exit code 和失败摘要 |
| EvidenceReport | `evidence/*` | 正式证据 |
| ReviewChecklist | `reviews/*` | 审查和 handoff |
| SavedView | `views/*` | filter，不是事实源 |
| ProjectUpdate | `updates/*` | evidence 派生摘要 |
| GoalLoopState | `goal-loop.json`, `updates/GOAL-LOOP-SUMMARY.md` | 下一步本地决策 |

## 模块

| 模块 | 职责 |
| --- | --- |
| Goal Compiler | `/goal` -> ProjectGoal / success / non-goals |
| Project Definition Builder | `/goal` -> AEP 第一阶段初始化包 |
| Scope State Manager | WIP=1 / active issue / execution authorization |
| Local Project Store | `.agentflow/` 读写 |
| Context Collector | 文件、命令、测试、模式收集 |
| Planner | goal + context -> roadmap / candidates |
| Issue Contract Builder | candidate -> executable contract |
| Codex Runtime Adapter | dry-run / apply / resume / human gate |
| Validation Runner | 本地验证命令 |
| Evidence Chain | transcript / commands / diff / evidence |
| Review Assistant | checklist / PR / handoff |
| Project Update Generator | evidence -> update |
| Goal Loop Orchestrator | goal / scope / issue / run / evidence -> next action |
| Local View Engine | SavedView filter |
| Desktop Workbench | 只读 `.agentflow/` 事实源 |

## 不变量

- `.agentflow/` 是事实源。
- `goal check` 必须通过 ProjectDefinition、ScopeState 和 bootstrap 产物检查。
- 客户端不得绕过 Core 改写执行事实。
- Roadmap / SavedView / ProjectUpdate 不授权执行。
- 没有 issue contract 不执行。
- 没有 validation / evidence 不完成。
- Goal Loop 只输出下一步建议，不直接执行。
- 默认不上传代码。
- Desktop v0 不写入事实源、不触发执行。

## 非职责范围

SaaS 多租户、企业 SSO、实时协作、自动 merge、完整 PM 平台、完整移动执行端、远程 GraphQL API、webhook 平台、海外 AI 代理。
