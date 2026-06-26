# Design Spec

创建日期：2026-05-21
最近压缩：2026-05-22
执行者：Codex

## 目标

界面只回答三件事：

1. 这次任务允许做什么。
2. 这次任务不允许做什么。
3. 这次执行留下了什么证据。

## 气质

开发者工具，安静、密集、清晰。使用左侧项目/issue，中间详情，右侧 run/evidence 状态。避免营销页、插画、卡片堆叠。

## 主视图

| View | 职责 | 事实源 |
| --- | --- | --- |
| Project | repo、Flow 0、settings、validation、latest update | `goal.md`, `environment.md`, `settings.json`, `updates/*` |
| Map | 模块地图、roadmap、candidate issue | `architecture.md`, `roadmap.md`, `index.json` |
| Issues | issue contract 列表和详情 | `issues/*` |
| Run | 当前或历史执行 | `runs/*` |
| Evidence | evidence report 和命令摘要 | `evidence/*` |
| Review | checklist 和 PR / handoff | `reviews/*` |

## Desktop MVP v0 边界

第一版 Desktop 只读 `.agentflow/` 事实源：Project Summary、SavedView、issue contract、run、evidence、review 和 review assistant。它不创建 issue、不执行 run、不调用模型、不写入事实源。

详细边界见 `docs/specs/desktop-workbench-mvp-boundary.md`。

## Linear 参考约束

- Views 是 filters，不是主模块。
- Project update 是 evidence 派生摘要，不是聊天流。
- Agent 状态必须可见：runtime、阶段、命令、验证、审查结论。
- Roadmap、candidate issue、saved view 不授权执行。

## 核心规则

- 默认 dry-run，apply 需要确认。
- 没有 issue contract 不允许 run。
- 没有 validation 不允许标记 completed。
- 失败也必须保存 evidence。
- saved view 只保存 filter。
- project update 可重新生成，但必须追溯到 evidence。

## 空状态

| 状态 | 主操作 |
| --- | --- |
| 未初始化 | Initialize |
| 无 issue | New Issue Contract |
| 无 run | Dry Run |
| 无 evidence | Generate Evidence |
| 模型未配置 | Configure Model |

## 验收

- 10 秒内看到项目是否初始化。
- 10 秒内找到 scope / non-goals。
- 清楚 validation 是否通过。
- 可复制 evidence / review。
- run 能回到 issue contract。
- 不需要学习完整 PM 概念。
