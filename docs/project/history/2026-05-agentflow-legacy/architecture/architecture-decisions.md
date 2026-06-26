# AgentFlow Architecture Decisions

创建日期：2026-05-21
最近压缩：2026-05-22
执行者：Codex

## 决策表

| ADR | 决策 | 结果 |
| --- | --- | --- |
| 001 | IssueContract 是执行唯一入口 | run 前必须解析 scope / non-goals / validation / evidence requirements |
| 002 | 成功失败都生成 evidence | failed run 也可审查和复盘 |
| 003 | Markdown + JSON 为事实源 | 用户可读，机器可解析 |
| 004 | SQLite 只做索引 | 损坏可从 `.agentflow/` 重建 |
| 005 | Core 用 Rust | 文件、命令、模型、evidence 规则集中 |
| 006 | CLI 先于 Desktop | CLI 是第一验收入口 |
| 007 | Desktop 用 Tauri 2 | 复用 Rust core，避免 Electron 体量 |
| 008 | UI 用 React + TypeScript | 表单、状态和 evidence 展示效率高 |
| 009 | DuckDB 后置 | MVP 不做大量分析 |
| 010 | Mobile 后置为 companion | 手机只看状态和确认，不直接操作 repo |
| 011 | 不做海外代理/支付中转 | 只提供模型配置和适配 |
| 012 | SavedView 是 filter | 不复制事实，不授权执行 |
| 013 | ProjectUpdate 派生自 evidence | 不能成为独立真相 |
| 014 | GoalCompiler 是项目入口 | `/goal` 必须先编译成 `goal.json` |
| 015 | ProjectDefinition 是 `/goal` 的第一阶段合同 | 新项目初始化必须生成 `project-definition.json` 和 `bootstrap/*` |
| 016 | ScopeState 记录 WIP=1 | 同一时间只允许一个 active issue run |
| 017 | AEP issue 字段是默认契约 | stop condition、feedback loop、docs claim trace 和 boundary 必须进入 issue contract |
| 018 | GoalLoopState 是下一步决策结果 | `goal next` 只输出建议，不执行命令 |
| 019 | LocalWorkspace / LocalTeam / LocalProject 是本地组织层 | project backlog 可以驱动下一条 issue intent，但不能替代 IssueContract |
| 020 | LocalProjectModelSnapshot 先只读派生 | workspace/team/project 文件落盘必须后置到单独 seed issue |
| 021 | Desktop Project View 复用只读 project snapshot | 桌面层级可见，但 recommended command 和 seed 写入仍由 CLI / IssueContract 边界控制 |
| 022 | Local Project Seed 必须先有确认门 | workspace/team/project seed 文件只允许在显式 seed writer issue 中写入 |
| 023 | LocalProjectSeed writer 默认只预览 | `agentflow project-seed` 不写文件，只有 `--write --yes` 创建默认 seed，且拒绝覆盖 |
| 024 | IssueProjectLink 先边界后写入 | `projectLink` 只提供归属上下文，不能替代 IssueContract 或改写 GoalLoop |
| 025 | IssueProjectLink writer 必须显式确认 | `agentflow issue-link ISSUE-XXXX` 默认 preview，只有 `--write --yes` 写指定 issue JSON / Markdown，且拒绝覆盖 |
| 026 | Project-aware GoalLoop 只做本地推荐 | project candidate 只在无 active / incomplete issue 时参与推荐，不执行命令、不迁移 issue |
| 027 | Project closure gate 先做只读 snapshot | Project Code Audit Snapshot 和 Root Docs Refresh Snapshot 只汇总输入、缺口和 blockers，不自动修复、不刷新文档、不标记 Project done |

## 核心对象

| 对象 | 来源 | 用途 |
| --- | --- | --- |
| `ProjectGoal` | `goal.{md,json}` | 目标和边界 |
| `ProjectDefinition` | `project-definition.json` | AEP 第一阶段初始化合同 |
| `AgentScopeState` | `scope-state.json` | WIP、active issue、执行授权边界 |
| `ProjectEnvironment` | `environment.md` | 运行规则 |
| `AgentflowSettings` | `settings.json` | 模型、验证、数据策略 |
| `CapabilityRoadmap` | `roadmap.md` | 能力顺序 |
| `LocalWorkspace` | 后续 `workspace.json` | 本地顶层组织容器 |
| `LocalTeam` | 后续 `teams/*.json` | workflow、validation preset、ownership 分组 |
| `LocalProject` | 后续 `projects/*.json` | milestones、issue backlog、project updates |
| `LocalProjectSeed` | `agentflow project-seed` / 后续 seed 文件 | 从 read model 固化默认本地组织事实源；默认 preview，显式确认后写入 |
| `IssueProjectLink` | `IssueContract.projectLink` / `agentflow issue-link` | 记录 issue 归属 team/project/milestone 和 link source |
| `Milestone` | 后续 project 内嵌字段 | project 阶段切分 |
| `LocalProjectModelSnapshot` | 现有 `.agentflow/` 事实源派生 | CLI / Desktop 的只读 project read model |
| `IssueContract` | `issues/*.json` | 执行输入 |
| `ExecutionRun` | `runs/*/run.json` | 执行记录 |
| `CommandRecord` | `commands.jsonl` | 命令证据 |
| `EvidenceReport` | `evidence/*.md` | 人类可读证据 |
| `ReviewChecklist` | `reviews/*.md` | 审查和 PR |
| `SavedView` | `views/*.json` | 本地 filter |
| `ProjectUpdate` | `updates/*.md` | 进展摘要 |
| `GoalLoopState` | `goal-loop.json` | 本地下一步决策 |
| `AgentProfile` | settings + run metadata | 执行器和模型边界 |

## 禁止状态

- 没有 `.agentflow/` 就执行 run。
- Flow 0 产物缺失就执行 run。
- `project-definition.json` 或 `scope-state.json` 缺失就进入 executable issue。
- 没有 issue contract 就执行 run。
- contract 缺少 scope、non-goals、validation 或 AEP issue 字段。
- run 完成但没有 evidence。
- goal loop 直接执行 run、创建远程 issue 或调用模型。
- project backlog 直接执行 run 或绕过 `agentflow plan`。
- local team 被当成账号、权限或远程协作边界。
- local project 替代 issue evidence / validation / review。
- saved view 被当成执行授权。
- project update 缺少 evidence source。
- settings 保存明文 API key。
- 默认上传代码或项目文件。
