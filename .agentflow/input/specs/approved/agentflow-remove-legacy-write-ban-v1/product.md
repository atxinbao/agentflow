# 移除 legacy 写入禁令 SPEC

## 背景

当前 AgentFlow 规则中存在多处硬性禁止项：不得写入 legacy `.agentflow/spec/**` 或 `.agentflow/goal-tree/**`。

这条规则需要移除。移除后，AgentFlow 仍然以 `.agentflow/input/**` 作为当前需求事实源，但不再把 legacy path 写入本身作为独立阻断条件。

## 目标

- 删除 AGENTS、Agentflow manual 和锁定技能中的 legacy write ban 文案。
- 保持 AgentFlow 当前事实源路径仍然是 `.agentflow/input/**`。
- 保持 SPEC Gate 当前路径仍然是 `.agentflow/input/specs/**`。
- 更新锁文件 hash，避免规则文件内容和 `skills-lock.json` 不一致。

## 不做事项

- 不恢复 `.agentflow/spec/**` 或 `.agentflow/goal-tree/**` 为推荐写入路径。
- 不向 legacy path 写入新 SPEC 或 Goal Tree。
- 不改变 Spec Agent、Build Agent、Audit Agent 的职责边界。
- 不启动 AgentRun。
- 不创建 PR、远程 issue 或外部对象。

## 验收标准

- `AGENTS.md` 不再包含 legacy `.agentflow/spec/**` / `.agentflow/goal-tree/**` 写入禁令。
- `.agentflow/define/agent/Agentflow.md` 不再包含 legacy 写入禁令或“not new write paths”硬阻断表述。
- `boundary-check` 不再把写 legacy path 作为独立阻断检查。
- `requirement-intake-filter`、`spec-gate-authoring`、`input-issue-generation` 不再把写 legacy path 作为硬性禁止项。
- `validation` 不再检查 erroneous legacy SPEC / Goal Tree writes。
- `.agentflow/define/agent/skills-lock.json` 中受影响文件的 hash 已同步。
