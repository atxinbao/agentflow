# AEP Reference Options

创建日期：2026-05-21
最近压缩：2026-05-22
执行者：Codex

## 定位

AEP 是工程协议和项目治理模板；AgentFlow 是本地产品。只继承能支撑本地 AI 执行闭环的部分。

## 继承

| AEP | AgentFlow |
| --- | --- |
| `GOAL.md` | `.agentflow/goal.md` |
| `ARCHITECTURE.md` | `.agentflow/architecture.md` |
| `ROADMAP.md` | `.agentflow/roadmap.md` |
| Linear Issue Draft | local issue contract |
| Evidence Chain | evidence report |
| Review Validation Gate | review checklist / handoff |
| latest summary | `docs/validation/latest-verification-summary.md` |
| append-only verification | `verification.md` |

## Flow 0

| Flow | 输出 | 不授权 |
| --- | --- | --- |
| 0.1 Intent | goal / target user / boundaries | 不生成执行任务 |
| 0.2 Rules | environment / settings / validation | 不执行代码 |
| 0.3 Map | architecture / roadmap / candidate issue | 候选 issue 不可直接执行 |

## 不继承

| AEP 选项 | 替代 |
| --- | --- |
| Linear Team / Project / Milestone | 本地 `.agentflow/index.json` |
| Symphony 常驻调度 | 手动 CLI / Desktop 操作 |
| Authorized Merge Agent | Review Assistant 只生成文案 |
| Graphify read-only context | 本地 Context Collector |
| GitHub + Linear auto Done | 本地 issue status |
| Queue invariant | 用户每次选择单一 issue |

## 核心迁移

AI Agent 不能只靠 prompt 工作。它需要：

- 目标。
- 地图。
- issue contract。
- validation。
- evidence chain。
- review gate。

任何 Agent run 都必须从合法 `IssueContract` 开始。
