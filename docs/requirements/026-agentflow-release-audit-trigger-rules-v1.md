# 026 - AgentFlow Release Audit Trigger Rules V1

> 建议保存路径：`docs/requirements/026-agentflow-release-audit-trigger-rules-v1.md`
> 类型：审计触发规则 / Release 后强制审计 / Agent 工作流规则 / 前端交互修正
> 状态：Ready for Codex implementation
> 目标版本：`v0.1.0-base` 前置修复

---

## 1. 背景

当前 AgentFlow 的审计流程需要重新定义。

之前容易误解成：

```text
用户在交付页点击“请求审计”
用户在审计页点击“新建审计”
用户在页面上重新审计
用户在页面上补证据
```

这个方向不对。

AgentFlow 当前版本是给 Agent 使用的本地工作流控制面，不是给普通用户直接点按钮驱动审计的系统。

因此：

```text
人类不能在 App 页面上有“新建审计 / 请求审计 / 重新审计 / 补证据”这类业务动作按钮。
```

审计触发应该来自两类 Agent 工作流规则：

```text
1. human-via-agent
   人类通过和 Codex / Agent 沟通，要求 Agent 做审计。
   Agent 根据规则写入 audit request 和 audit report。

2. release-auto
   每次 Release 版本生成后，AgentFlow 规则自动要求 Agent 做审计。
   系统生成审计请求 / 审计任务，Agent 必须写入审计报告。
```

---

## 2. 一句话定义

> **AgentFlow 的审计不是由用户在 UI 上点“新建/请求”触发，而是由 Agent 工作流触发。Release 生成后必须自动进入审计要求；人类想审计时，也应该通过和 Agent 沟通触发，由 Agent 写入审计报告。App 只负责展示审计状态和报告，不提供创建审计的业务按钮。**

---

## 3. 术语定义

### 3.1 human-via-agent

含义：

```text
人类通过 Codex / Agent 对话提出审计要求。
Agent 收到后，根据 AgentFlow 规则写入审计请求和审计报告。
```

不是：

```text
用户在 AgentFlow App 页面里点击“请求审计”。
```

### 3.2 release-auto

含义：

```text
Release delivery 被生成 / 捕获后，AgentFlow 规则自动要求 Agent 执行审计。
```

注意：

```text
release-auto 不是 App 自动调用 Codex API。
release-auto 是本地工作流规则：Release 后必须生成审计请求，并要求 Agent 写入审计报告。
```

### 3.3 App 的角色

App 只做：

```text
展示 Release
展示 Audit 状态
展示 Audit Report
展示 Findings
展示 Evidence Map
展示 Traceability
提示审计是否缺失
```

App 不做：

```text
新建审计
请求审计
重新审计
补证据
自动调用 Codex
自动生成报告内容
```

---

## 4. 新审计触发模型

### 4.1 触发类型

新增 / 统一 audit trigger：

```text
human-via-agent
release-auto
```

建议 Rust / JSON 字段：

```json
{
  "trigger": "release-auto"
}
```

或：

```json
{
  "trigger": "human-via-agent"
}
```

### 4.2 trigger 含义

| trigger | 来源 | 谁触发 | 是否来自 UI 按钮 | 是否自动调用 Codex |
|---|---|---|---|---|
| `human-via-agent` | 人类和 Agent 对话 | Agent | 否 | 否 |
| `release-auto` | Release 生成后规则要求 | AgentFlow 规则 + Agent | 否 | 否 |

---

## 5. Release 后自动审计规则

### 5.1 规则

每次生成 Release Delivery 后，必须进入审计要求。

流程：

```text
Release Delivery created
  ↓
AgentFlow 检测到 release delivery
  ↓
自动生成 audit request / audit task
  ↓
trigger = release-auto
  ↓
Agent 必须根据请求写入 audit report
  ↓
App 展示 audit status / report
```

### 5.2 注意

这里的“自动触发”不是指 App 调用模型。

它只是：

```text
1. 自动创建审计请求
2. 自动把审计作为 Agent 下一步必须做的规则
3. 让 Agent 在后续工作中写入审计报告
```

---

## 6. 人类通过 Agent 触发审计

### 6.1 场景

人类可以在 Codex / Agent 对话中说：

```text
请审计这次交付
请重新核对 DEL-001
这次证据不够，请重新审计
请检查这个 release 是否符合 SPEC
```

Agent 根据 AGENTS.md / AgentFlow 规则执行：

```text
1. 读取 release delivery
2. 读取 issue / spec / evidence
3. 写入 audit request
4. 写入 audit report
5. 写入 findings / checklist / evidence-map / traceability
```

### 6.2 App 不提供按钮

这些动作不能出现在 App 普通页面上：

```text
请求审计
新建审计
重新审计
补证据
```

如果当前前端有这些按钮，需要删除或改成只读提示。

---

## 7. Audit Request 数据结构

### 7.1 建议结构

```json
{
  "version": "audit-request.v1",
  "auditId": "audit-001",
  "trigger": "release-auto",
  "reason": "Release 已生成，AgentFlow 规则要求进行审计。",
  "source": {
    "kind": "release-delivery",
    "deliveryId": "DEL-001",
    "runId": "run-001",
    "issueId": "AF-104",
    "specId": "SPEC-001"
  },
  "scope": {
    "description": "审计 release delivery 是否符合 SPEC、Issue、Evidence 和验证结果。",
    "refs": [
      { "kind": "spec", "id": "SPEC-001", "path": ".agentflow/input/specs/approved/SPEC-001/spec.json" },
      { "kind": "issue", "id": "AF-104", "path": ".agentflow/input/issues/AF-104.json" },
      { "kind": "execute-run", "id": "run-001", "path": ".agentflow/execute/runs/run-001" },
      { "kind": "evidence", "id": "run-001", "path": ".agentflow/output/evidence/run-001.json" },
      { "kind": "release-delivery", "id": "DEL-001", "path": ".agentflow/output/release/run-001/delivery.json" }
    ]
  },
  "system": {
    "createdBy": "agentflow-release-auto",
    "createdAt": 1780360000
  }
}
```

### 7.2 trigger 枚举

建议：

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditTrigger {
    HumanViaAgent,
    ReleaseAuto,
}
```

---

## 8. Audit Report 输出要求

Agent 写入审计报告后，必须包含：

```text
audit-request.json
audit.json
audit-report.md
findings.json
checklist.md
evidence-map.json
traceability.json
```

路径：

```text
.agentflow/output/audit/<audit-id>/
```

### 8.1 audit.json

应包含：

```json
{
  "auditId": "audit-001",
  "trigger": "release-auto",
  "status": "passed-with-warnings",
  "sourceDeliveryId": "DEL-001",
  "sourceRunId": "run-001",
  "sourceIssueId": "AF-104"
}
```

### 8.2 audit-report.md

必须是人类可读报告：

```md
# Audit Report

## 结论
通过，有警告。

## 发现项
- 证据完整。
- 验证命令已记录。
- UI 截图证据建议补充。

## 证据链
SPEC → Issue → Run → Evidence → Delivery → Audit
```

---

## 9. 幂等规则

### 9.1 release-auto 必须幂等

同一个 Release Delivery 只允许有一个自动审计。

幂等键：

```text
source.deliveryId + trigger = release-auto
```

或者：

```text
source.runId + trigger = release-auto
```

如果已存在：

```text
不要重复创建
返回已有 audit
```

### 9.2 human-via-agent 可多次

人类通过 Agent 要求重新审计时，可以产生新的 audit。

但每次必须有新 auditId。

示例：

```text
AUD-001 release-auto
AUD-002 human-via-agent
AUD-003 human-via-agent
```

---

## 10. State 规则

### 10.1 Release 生成后不应停在 not-requested

旧规则：

```text
delivery-ready
auditStatus = not-requested
```

Base 之后需要改为：

```text
delivery-ready
→ audit required / audit requested
```

如果 audit request 已自动创建：

```text
auditStatus = requested
```

如果报告同步生成：

```text
auditStatus = passed / passed-with-warnings / failed / cancelled
currentStage = audit-completed
```

如果 Release 已有但 audit 缺失：

```text
state blockers / warnings 应提示：
Release 已生成，但审计请求缺失。
```

### 10.2 不新增 UI 按钮

这个状态只用于 Agent 和页面展示。
不能变成用户点击按钮。

---

## 11. 前端交付页修改

### 11.1 去掉按钮

交付页普通用户界面删除：

```text
请求审计
新建审计
重新审计
补证据
```

### 11.2 显示审计状态

交付详情显示：

```text
审计状态：等待 Agent 审计
审计状态：审计中
审计状态：已完成
审计状态：审计缺失
```

如果自动审计已存在：

```text
显示：查看审计报告
```

这个按钮只做跳转，不创建审计。

### 11.3 空态

如果 release 存在但 audit 缺失：

```text
Release 已生成，但还没有审计报告。
AgentFlow 规则要求 Agent 完成审计。
```

不要显示：

```text
[请求审计]
```

---

## 12. 前端审计页修改

### 12.1 审计页只读

审计页删除：

```text
新建审计
请求审计
重新审计
补证据
```

审计页只展示：

```text
审计列表
审计报告
发现项
证据链
处理建议
触发来源
```

### 12.2 显示 trigger

审计列表显示：

```text
Release 自动审计
人类通过 Agent 触发
```

不要显示原始：

```text
release-auto
human-via-agent
```

除非在高级页。

### 12.3 审计报告详情

展示：

```text
触发来源
关联交付
关联任务
审计结论
发现项
证据链
处理建议
```

---

## 13. 高级页修改

高级页可以显示 raw trigger：

```json
{
  "trigger": "release-auto"
}
```

但必须标注：

```text
只读诊断
```

---

## 14. AGENTS.md / Agent 规则更新

需要更新 Agent 规则，明确：

```text
1. 每次 release delivery 生成后，必须审计。
2. 审计必须写入 .agentflow/output/audit/<audit-id>/。
3. 不允许只口头说审计完成。
4. 审计报告必须包含 findings、evidence-map、traceability。
5. 如果人类通过对话要求审计，trigger = human-via-agent。
6. 如果 release 后自动要求审计，trigger = release-auto。
```

---

## 15. 不做事项

本需求不做：

```text
不调用 Codex API
不自动运行模型
不创建远程 PR
不 merge
不 deploy
不在普通 UI 暴露新建审计按钮
不让用户在 App 页面手动触发审计
不删除人工通过 Agent 触发的能力
```

---

## 16. 验收标准

必须满足：

```text
1. Release delivery 生成后，系统能创建 release-auto audit request。
2. 同一个 release 不重复创建 release-auto audit。
3. Agent 写入 audit report 后，审计页能读取。
4. 审计页能区分“Release 自动审计”和“人类通过 Agent 触发”。
5. 交付页不再出现“请求审计”按钮。
6. 审计页不再出现“新建审计”按钮。
7. 交付页能显示审计状态。
8. release-auto 缺失审计时，页面显示规则提示，不显示按钮。
9. 高级页能查看 raw trigger。
10. cargo test 通过。
11. npm build 通过。
```

---

## 17. Codex 实现指令

```text
你现在只做这个任务：AgentFlow Release Audit Trigger Rules V1。

背景：
AgentFlow 是给 Agent 使用的本地工作流控制面。审计不应该由用户在 App 页面上点击“新建/请求”触发。当前需要把审计触发改成 Agent 工作流规则：
1. 人类通过和 Agent 对话触发审计：trigger = human-via-agent。
2. Release 版本生成后自动要求审计：trigger = release-auto。

目标：
Release 生成后必须自动创建审计请求，并要求 Agent 写入审计报告。App 只展示审计状态和报告，不提供创建审计的 UI 按钮。

范围：
- crates/output/**
- crates/state/**
- apps/desktop/src/**
- AGENTS.md / define agent rules
- docs/requirements/**
- 相关测试

具体要求：
1. 增加 audit trigger：
   - human-via-agent
   - release-auto

2. Release delivery 生成后：
   - 自动创建 audit request
   - trigger = release-auto
   - reason = Release 已生成，AgentFlow 规则要求进行审计。
   - scope refs 包含 spec / issue / execute-run / evidence / release-delivery

3. release-auto 幂等：
   - 同一个 deliveryId + release-auto 只能有一个 audit
   - 已存在则返回已有 audit
   - 不重复创建

4. human-via-agent：
   - 不来自 UI 按钮
   - 来自 Agent 根据人类对话要求写入
   - 可以多次生成新的 audit

5. 前端交付页：
   - 删除“请求审计”创建类按钮
   - 显示审计状态
   - 已有报告时可“查看审计报告”
   - 缺失报告时显示规则提示

6. 前端审计页：
   - 删除“新建审计 / 请求审计 / 重新审计 / 补证据”类按钮
   - 只读展示审计列表和报告
   - 显示触发来源：
     Release 自动审计
     人类通过 Agent 触发

7. 高级页：
   - 可显示 raw trigger
   - 标注只读诊断

禁止：
- 不要调用 Codex API。
- 不要自动运行模型。
- 不要创建远程 PR。
- 不要把 UI 按钮作为审计触发入口。
- 不要重复生成 release-auto audit。
- 不要删除 audit report 读取能力。
- 不要把 release-auto 和 human-via-agent 混成一种。

验证：
1. 生成 release delivery 后，出现 release-auto audit request。
2. 重复 refresh 不重复生成 audit。
3. Agent 写入 audit report 后，审计页可显示。
4. 交付页没有“请求审计”按钮。
5. 审计页没有“新建审计”按钮。
6. 审计列表能显示触发来源。
7. cargo test 通过。
8. npm --prefix apps/desktop run build 通过。

输出：
- 改了哪些文件
- trigger 字段如何设计
- release-auto 幂等如何保证
- 哪些 UI 按钮被删除
- 审计页如何显示触发来源
- 测试结果
```
