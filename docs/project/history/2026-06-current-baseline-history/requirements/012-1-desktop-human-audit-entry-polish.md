请执行一个小 polish：
# 012.1 - Desktop Human Audit Entry Polish
目标：
在 Desktop 里补一个人类可见的“触发审计 / 查看审计报告”入口。PR #26 已经实现了后端和 Tauri command：`request_human_audit`、`load_audit_report`、`load_audit_index`、`load_audit_status`。本次只做 Desktop 可见入口和只读报告查看，不改 Audit 核心模型。
---
## 背景
PR #26 已完成：
- Audit 不自动执行。
- 只有人类触发才写 `.agentflow/output/audit/<audit-id>/`。
- `request_human_audit` 会生成完整 audit package：
  - `audit-request.json`
  - `audit.json`
  - `audit-report.md`
  - `findings.json`
  - `checklist.md`
  - `evidence-map.json`
  - `traceability.json`
- 已有 Tauri commands：
  - `request_human_audit`
  - `load_audit_report`
  - `load_audit_index`
  - `load_audit_status`
当前缺口：
Desktop 状态通道能看到 output 状态，但人类还没有一个明显入口来：
1. 选择某次 release delivery / run。
2. 输入 audit reason。
3. 点击 Request Human Audit。
4. 查看生成后的 audit report。
---
## 必须实现
### 1. 新增 Output / Audit Desktop 入口
在 Desktop 中新增一个可见入口，建议放在现有“交付输出 / Output status”相关区域。
入口名称可以是：
```text
Human Audit

中文 UI 可显示：

人工审计

需要展示：

Evidence count
Release delivery count
Audit count
Incomplete count
Latest release delivery
Latest audit report

⸻

2. 支持选择 release delivery

从现有 output index 中读取：

load_output_index

使用：

index.releaseDeliveries

作为可审计对象列表。

每个可选对象至少展示：

runId
issueId
sourceSpecId
status
path

如果没有 release delivery：

显示：暂无可审计交付材料
禁用 Request Human Audit 按钮

⸻

3. 生成 audit scope refs

点击 Request Human Audit 时，基于选中的 release delivery 自动生成 scope refs：

{
  "reason": "<human input>",
  "scope": {
    "description": "Human requested audit for Build Agent delivery.",
    "refs": [
      {
        "kind": "spec",
        "id": "<sourceSpecId>",
        "path": ".agentflow/input/specs/approved/<sourceSpecId>/"
      },
      {
        "kind": "issue",
        "id": "<issueId>",
        "path": ".agentflow/input/issues/<issueId>.json"
      },
      {
        "kind": "execute-run",
        "id": "<runId>",
        "path": ".agentflow/execute/runs/<runId>/"
      },
      {
        "kind": "evidence",
        "id": "<runId>",
        "path": ".agentflow/output/evidence/<runId>.json"
      },
      {
        "kind": "release-delivery",
        "id": "<runId>",
        "path": ".agentflow/output/release/<runId>/delivery.json"
      }
    ]
  }
}

注意：

不要让人类手动编辑 scope refs。
Desktop 自动带出。

⸻

4. Audit reason 必填

UI 需要一个 reason 输入框。

如果为空，按钮禁用或提交时报错：

请输入审计原因

后端已有 reason 非空校验，但前端也要做基础校验。

⸻

5. 调用 request_human_audit

使用 Tauri：

invoke("request_human_audit", {
  projectRoot,
  draft
})

成功后：

刷新 output status
刷新 audit index
自动加载新 audit report
显示 audit-report.md

⸻

6. 查看审计报告

新增只读报告查看区。

使用：

load_audit_index
load_audit_report

展示：

auditId
status
requestedBy
requestedAt
audit-report.md
findings count
checklist
evidence-map
traceability

V1 只需要：

默认展示 audit-report.md
可以展开查看 findings / checklist / evidence-map / traceability

如果 UI 复杂度太大，可以先做：

audit-report.md 主展示
findings / checklist / evidence-map / traceability 以 details JSON / markdown block 展示

⸻

7. Audit 状态展示

在 Output status channel / Output 页面里显示：

Audit count
Latest audit status
Latest audit ID

Audit status 使用：

passed
passed-with-warnings
failed
cancelled

没有 audit 时显示：

not requested

中文：

未请求审计

⸻

不允许做

本次只做 Desktop polish，不扩大后端能力。

不要做：

不改 audit 核心模型
不改 request_human_audit 的核心逻辑
不自动触发 audit
不在 execute / output 完成后自动审计
不写 input facts
不写 execute facts
不写 evidence
不写 release delivery
不写用户源码
不执行命令
不创建 PR
不 merge
不 deploy
不调用模型
不新增 run/project/batch 三套 audit 分类

⸻

文件建议

重点检查 / 修改：

apps/desktop/src/features/output/
apps/desktop/src/features/output/hooks/useOutputStatus.ts
apps/desktop/src/features/status-channel/statusAdapters.ts
apps/desktop/src/types/status.ts
apps/desktop/src/browserPreviewData.ts
apps/desktop/src/App.tsx

如当前没有 Output 页面，可以新增：

apps/desktop/src/features/output/OutputAuditPanel.tsx
apps/desktop/src/features/output/hooks/useAuditReports.ts

或等价结构。

⸻

TypeScript 类型建议

如果还没有对应类型，新增：

export type AuditScopeRef = {
  kind: string;
  id: string;
  path: string;
};
export type AuditScope = {
  description: string;
  refs: AuditScopeRef[];
};
export type HumanAuditRequestDraft = {
  reason: string;
  scope: AuditScope;
};
export type AuditStatus =
  | "passed"
  | "passed-with-warnings"
  | "failed"
  | "cancelled";
export type HumanAuditReport = {
  request: unknown;
  audit: {
    auditId: string;
    status: AuditStatus;
    requestedBy: string;
    requestedAt: number;
  };
  reportMarkdown: string;
  findings: unknown;
  checklistMarkdown: string;
  evidenceMap: unknown;
  traceability: unknown;
};

按 Rust serde camelCase 实际返回字段调整。

⸻

Browser Preview

Browser Preview 不允许真实写 audit。

预览中：

Request Human Audit 按钮应 disabled
或显示 preview-only 提示

提示：

浏览器预览不写 .agentflow/output/audit；请在 Tauri Desktop 中触发人工审计。

⸻

测试要求

至少新增 / 更新：

1. Output Audit UI 在没有 release delivery 时禁用 Request Human Audit。
2. reason 为空时不能提交。
3. 选中 release delivery 后能构造正确 scope refs。
4. 成功 request_human_audit 后刷新 audit report。
5. Browser Preview 不会调用 request_human_audit。
6. Existing desktop build still passes。

如果当前前端测试体系不足，至少用 TypeScript build 和 browser preview mock 保证不破坏构建。

⸻

验收标准

- [ ] Desktop 有可见的“人工审计 / Human Audit”入口。
- [ ] 可以看到 release delivery 列表。
- [ ] 可以选择某个 release delivery。
- [ ] reason 必填。
- [ ] 点击后调用 request_human_audit。
- [ ] scope refs 自动根据 runId / issueId / sourceSpecId 生成。
- [ ] 成功后可以查看 audit-report.md。
- [ ] 可以查看 findings / checklist / evidence-map / traceability。
- [ ] Output 状态显示 audit count / latest audit status。
- [ ] 没有 delivery 时按钮禁用。
- [ ] Browser Preview 不执行真实 audit 写入。
- [ ] 不自动触发 audit。
- [ ] 不改 input / execute / evidence / release delivery。
- [ ] 不创建 PR / merge / deploy。
- [ ] 不调用模型。
- [ ] cargo fmt --check 通过。
- [ ] cargo test -p agentflow-desktop 通过。
- [ ] cargo test 通过。
- [ ] npm --prefix apps/desktop run build 通过。
- [ ] git diff --check 通过。

⸻

验证命令

cargo fmt --check
cargo test -p agentflow-desktop
cargo test
npm --prefix apps/desktop run build
git diff --check

⸻

PR 标题建议

[codex] Add Desktop human audit entry

PR 描述必须说明：

1. 本次只补 Desktop 可见入口。
2. Audit 仍然不会自动执行。
3. 人类必须选择 delivery 并填写 reason。
4. request_human_audit 只写 output/audit/<audit-id>。
5. Browser Preview 不执行真实写入。
6. 本次没有修改 input / execute / evidence / release delivery。
7. 本次没有创建 PR / merge / deploy。
8. 本次没有调用模型。
9. 验证命令和结果。