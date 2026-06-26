# 013.2 - Browser Preview Smoke Script

创建日期：2026-06-05
更新日期：2026-06-05
执行者：Codex
状态：已开发
版本：final

## 用户目标

PR #28 已经补齐 Browser Preview 的人工审计可视核对，但验证仍依赖一次人工 DOM snapshot。后续本地验证需要一个可重复执行的 smoke 命令，稳定确认 Browser Preview 展示数据和只读边界没有回退。

## 一句话定义

013.2 为 Desktop Browser Preview 增加可复跑 smoke 脚本，用自动断言替代人工 DOM snapshot 作为基础回归检查。

## 范围

- 新增或维护 `npm --prefix apps/desktop run preview:smoke`。
- 新增或维护 `apps/desktop/scripts/browser-preview-smoke.mjs`。
- 验证 Browser Preview output status 中 evidence / release delivery / audit mock 已存在。
- 验证 Browser Preview human audit report 包含 `Human Audit Browser Preview`。
- 验证 Browser Preview workflow state 暴露 `workspace-ready` 和 `passed-with-warnings`。
- 验证 Human Audit 面板源码中 preview 分支使用只读 mock 数据。
- 验证 `请求人工审计` 在 preview source 下保持 disabled。
- 验证 preview-only guard 在真实 `request_human_audit` invoke 之前返回。
- 验证 smoke 过程不写 `.agentflow/output/audit`。

## 非目标

- 不新增真实浏览器 DOM 自动化。
- 不新增 CI。
- 不修改 Tauri `request_human_audit` 命令。
- 不修改 Rust output / audit / state 核心模型。
- 不把 Browser Preview mock 当作真实审计事实源。
- 不在 Browser Preview 中写 `.agentflow/output/audit`。

## 页面 / 功能

- Desktop Browser Preview。
- Human Audit 面板。
- Browser Preview mock 数据。
- 本地 npm smoke 命令。

## 数据来源

- `apps/desktop/src/browserPreviewData.ts`
- `apps/desktop/src/features/output/OutputAuditPanel.tsx`
- 临时 smoke root：`os.tmpdir()/agentflow-browser-preview-smoke-*`

## 交互边界

- smoke 脚本只读取前端源码和 Browser Preview mock factory。
- smoke 脚本只在临时目录中检查禁写结果。
- smoke 脚本不得触发 Tauri Desktop 命令。
- Browser Preview request human audit 路径必须在 preview-only guard 内返回。

## 验收标准

- [x] `npm --prefix apps/desktop run preview:smoke` 存在且可执行。
- [x] smoke 断言 `evidence = 1`。
- [x] smoke 断言 `releaseDeliveries = 1`。
- [x] smoke 断言 `audits = 1`。
- [x] smoke 断言 `incompleteEvidence = 0`。
- [x] smoke 断言 `incompleteDeliveries = 0`。
- [x] smoke 断言 release delivery run id 为 `run-browser-preview-001`。
- [x] smoke 断言 audit id 为 `audit-browser-preview-001`。
- [x] smoke 断言报告包含 `Human Audit Browser Preview`。
- [x] smoke 断言 state stage 为 `workspace-ready`。
- [x] smoke 断言 audit status 为 `passed-with-warnings`。
- [x] smoke 断言源码中存在 Browser Preview mock report 注入。
- [x] smoke 断言 request disabled 条件包含 `previewOnly`。
- [x] smoke 断言 preview-only guard 位于真实 `request_human_audit` invoke 之前。
- [x] smoke 断言临时目录中不存在 `.agentflow/output/audit`。

## 验证命令

```bash
npm --prefix apps/desktop run preview:smoke
npm --prefix apps/desktop run build
git diff --check
```

## 留痕

验证记录见 [docs/verification/history.md](../verification/history.md) 的 `2026-06-05 Browser Preview Smoke Script` 小节。
