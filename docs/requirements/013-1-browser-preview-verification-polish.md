# 013.1 - Browser Preview Verification Polish

创建日期：2026-06-05
执行者：Codex
状态：已开发
版本：final

---

## 用户目标

PR #27 已完成 `012.1 - Desktop Human Audit Entry Polish` 和 `013 - Workflow State / Gate Orchestration V1` 的代码层验证，但 Browser Preview 可视核对没有闭环。

本轮目标是补齐 Desktop Browser Preview 的人工审计展示数据，使 `http://127.0.0.1:1421/` 能被本地浏览器预览打开并展示：

- 交付输出摘要计数。
- 可选择的 release delivery。
- 最新人工审计报告。
- 浏览器预览只读边界提示。

---

## 范围

- 补齐 `apps/desktop/src/browserPreviewData.ts` 中 output / audit / state 的 browser preview mock 数据。
- 让 Browser Preview 的人工审计区域展示 1 条 evidence、1 条 release delivery、1 条 audit。
- 让 Browser Preview 默认展示一份只读 `audit-report.md` mock 报告。
- 记录 Browser Preview 可视核对结果。

---

## 非目标

- 不修改 `request_human_audit` 的真实 Tauri 命令。
- 不修改 `.agentflow/output/audit` 的真实写入逻辑。
- 不自动触发 audit。
- 不修改 Rust audit / output / state 数据模型。
- 不新增 Desktop 执行动作。
- 不把 Browser Preview mock 当作真实审计事实源。

---

## 页面 / 功能

- Desktop Project 页面中的 `交付输出 / 人工审计` 区域。
- Browser Preview runtime 下的 output status / output index / audit index / audit report mock。

---

## 数据来源

- Browser Preview：`apps/desktop/src/browserPreviewData.ts`。
- 真实 Desktop：继续通过 Tauri commands 读取 `.agentflow/output/**` 和 `.agentflow/state/**`，本轮不改变。

---

## 交互边界

- Browser Preview 可以展示 release delivery 和 audit report。
- Browser Preview 中 `请求人工审计` 仍保持禁用，不写 `.agentflow/output/audit`。
- 真实 Desktop 中人工审计请求仍必须由人类选择 delivery 并填写 reason 后触发。

---

## 验收标准

- [x] `http://127.0.0.1:1421/` 可以打开 Desktop Browser Preview。
- [x] 人工审计区域展示 `证据 1`、`交付 1`、`审计 1`、`未完成 0`。
- [x] 交付材料下拉展示 `run-browser-preview-001 · iss-001 · delivered`。
- [x] 最新审计报告展示 `audit-browser-preview-001` 和 `Human Audit Browser Preview`。
- [x] Browser Preview 中人工审计请求按钮保持禁用。
- [x] Browser console 无 error / warning。

---

## 验证命令

- `npm --prefix apps/desktop run build`
- `cargo fmt --check`
- `cargo test -p agentflow-desktop`
- `cargo test`
- `git diff --check`

---

## 验证记录

记录见 [docs/verification/history.md](../verification/history.md) 的 `2026-06-05 Browser Preview Verification Polish` 小节。
