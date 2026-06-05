
# AgentFlow Issue 状态增强开发需求文档

## 1. 目标

对 `.agentflow/input/issues` 内的 Issue 状态进行增强，从原有的基础状态升级为 Kanban 风格的 6 个状态，用于前端任务看板展示和任务流管理。

新状态如下：

- **Backlog**：需求已创建，尚未整理成 SPEC。
- **Ready**：需求整理完成，Issue 可交给 Agent 执行。
- **In Progress**：Agent 已接手任务，正在执行。
- **Review**：Codex 执行完成，等待人工审计或验证。
- **Done**：任务执行完成，审计通过。
- **Cancel**：任务被取消或废弃。

## 2. 范围

- `crates/input/src/issue.rs`
- `crates/input/src/lib.rs`
- `crates/input/src/manager.rs`
- `.agentflow/input/issues/*.json`

## 3. 功能要求

1. **新增 DisplayStatus 枚举**

```rust
pub enum DisplayStatus {
    Backlog,
    Ready,
    InProgress,
    Review,
    Done,
    Cancel,
}
```

2. **计算逻辑**

- 每次加载 Issue 或刷新索引时，根据现有的 workflow / execute / output / audit 状态计算 `DisplayStatus`。
- 前端任务看板直接使用 `DisplayStatus` 渲染列和列表。
- 确保状态更新后能触发对应看板刷新和 UI 更新。

3. **JSON 序列化规范**

- 保存到 `.agentflow/input/issues/*.json` 时，使用 kebab-case 格式：
  ```text
  backlog
  ready
  in-progress
  review
  done
  cancel
  ```
- 保持与原有序列化方式一致，兼容现有索引生成逻辑。

4. **前端使用**

- 任务看板模式：每列对应一个 `DisplayStatus`。
- 列表模式：按 `DisplayStatus` 排序或分组。
- 卡片顶部显示状态标签或颜色，便于快速识别。

5. **回退兼容**

- 原有 `InputIssueStatus` 字段仍保留，用于 workflow 内部计算。
- 新状态仅用于 UI / TaskBoard 展示，确保不会破坏现有后端逻辑。

## 4. 验收标准

- 新增 6 个状态在 JSON 文件里正确保存。
- 看板页面显示每个 Issue 正确状态列。
- 状态切换逻辑与 workflow 保持一致。
- JSON 序列化与索引生成逻辑无误。

## 5. 风险控制

- 不影响 Agent 执行逻辑。
- 不影响 workflow 状态机的运行。
- 保持原有 IssueStatus 字段，避免兼容问题。
- 更新后 CI 测试全通过。
