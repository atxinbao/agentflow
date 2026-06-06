# 技术约束

## 允许修改

- `AGENTS.md`
- `.agentflow/define/agent/Agentflow.md`
- `.agentflow/define/agent/skills-lock.json`
- `.agentflow/define/agent/skills/boundary-check/SKILL.md`
- `.agentflow/define/agent/skills/requirement-intake-filter/SKILL.md`
- `.agentflow/define/agent/skills/spec-gate-authoring/SKILL.md`
- `.agentflow/define/agent/skills/input-issue-generation/SKILL.md`
- `.agentflow/define/agent/skills/validation/SKILL.md`

## 实现要求

- 只删除 legacy write ban 相关硬阻断语句。
- 保留 `.agentflow/input/**` 是当前事实源的说明。
- 保留 SPEC Gate 是 `product.md`、`tech.md`、`approval.json` 的规则。
- 保留未确认前不得写 Approved SPEC、不得生成 input issue、不得启动 AgentRun 的规则。
- 修改技能或手册后，重新计算并更新 `skills-lock.json` 中对应 hash。

## 验证方式

- 使用本地文本检索确认 legacy write ban 文案已移除。
- 使用本地 hash 计算确认 `skills-lock.json` 与实际文件内容一致。
- 检查没有新增 `.agentflow/spec/**` 或 `.agentflow/goal-tree/**` 文件。
