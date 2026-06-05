# Testing

日期：2026-06-05
执行者：Codex
任务：013.2 - Browser Preview Smoke Script

## 执行结果

- `npm --prefix apps/desktop run preview:smoke`：pass。
  - 输出：`Browser Preview smoke passed: workflow state and human audit preview are read-only.`
- `npm --prefix apps/desktop run build`：pass。
- `cargo fmt --check`：pass。
- `cargo test -p agentflow-workflow-acceptance`：pass，6 tests。
- `git diff --check`：pass。

## 覆盖点

- Browser Preview output status mock。
- Browser Preview output / audit index mock。
- Browser Preview human audit report mock。
- Browser Preview workflow state mock。
- Human Audit 面板 preview-only 禁用边界。
- Browser Preview 不写 `.agentflow/output/audit`。

## 未执行项

- 未执行真实浏览器 DOM 自动化；013.2 的目标是补可复跑 smoke，不替代 PR #28 已完成的人工 DOM snapshot 记录。
- 未执行完整 `cargo test`；本轮只修改前端 smoke 和文档，014 全量验证已在当前分支前序记录中完成。
