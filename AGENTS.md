# AGENTS.md

更新日期：2026-06-04
执行者：Codex

## 当前文档原则

项目文档已重置。2026-05 形成的旧需求、旧规划、旧规格和旧验证摘要已经移动到：

```text
docs/archive/2026-05-agentflow-legacy/
```

归档文档只作历史参考，不再作为实现授权。

## 必读路径

后续开发默认只读：

1. `README.md`
2. `GOAL.md`
3. `ROADMAP.md`
4. `docs/README.md`
5. `docs/requirements/README.md`
6. `docs/requirements/next-requirements.md`
7. `docs/requirements/008-3-agentflow-workflow-directory-blueprint-v1-final.md`

按需参考：

1. `design.md`
2. `verification.md`
3. `docs/archive/2026-05-agentflow-legacy/README.md`

## 工作边界

- 不把归档文档当作下一步需求。
- 不从归档 Roadmap / Specs / Planning 自动派生 issue。
- 不因为旧文档存在就继续旧 Workflow Control、Product Feature、Closure 或 GoalLoop 方向。
- 新功能必须由 `docs/requirements/` 下的新需求文档授权。
- Desktop 当前仍保持只读边界，除非新需求明确改变。

## 文档职责

| 文档 | 职责 |
| --- | --- |
| `GOAL.md` | 当前目标状态和新需求入口 |
| `ROADMAP.md` | 新 Roadmap 生成规则 |
| `docs/requirements/*` | 后续唯一需求入口 |
| `docs/archive/*` | 历史归档，不授权实现 |
| `design.md` | 当前界面设计记录，除非新需求覆盖 |
| `verification.md` | 历史验证记录 |

## 当前实现焦点

- 当前需求为 `docs/requirements/008-3-agentflow-workflow-directory-blueprint-v1-final.md`。
- 本项目根 `AGENTS.md` 仍作为仓库操作手册维护，不直接替换成运行时 managed entry 模板。
- 运行时 managed entry 模板由 `crates/agent-manual` 生成到目标项目的 `AGENTS.md`。

## 实施规则

1. 新需求到来后，先更新 `docs/requirements/next-requirements.md` 或创建新的 requirements 文件。
2. 再从新需求拆开发切片。
3. 每个切片必须包含范围、非目标和验证命令。
4. 不能引用归档文档作为唯一依据。
