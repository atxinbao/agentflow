# 028 - AgentFlow Codex Role Usage Guide V1

> 建议保存路径：`docs/requirements/029-agentflow-codex-role-usage-guide-v1.md`  
> 类型：产品交互 / Codex 角色使用说明 / Agent 线程规范 / 前端页面需求  
> 状态：Ready for Codex implementation  
> 目标版本：`v0.1.0-base` 前置补充

---

## 1. 背景

AgentFlow 当前有 3 个核心 Agent 角色：

```text
Spec Agent
Build Agent
Audit Agent
```

但当前版本的 AgentFlow App 不直接调用 Codex / Claude / DeepSeek API，也不会自动在 Codex 里创建 Agent。

真实使用方式是：

```text
人类在 Codex 里创建不同线程。
每个线程按 AgentFlow 给出的启动指令扮演一个固定角色。
AgentFlow App 负责展示任务、任务包、交付、审计和状态。
```

因此，AgentFlow 必须在页面上告诉用户：

```text
如何在 Codex 里创建 / 使用这 3 个角色。
```

否则用户会误以为：

```text
1. AgentFlow 会自动创建 Codex Agent。
2. 一个 Codex 线程可以同时做需求、开发、审计。
3. Build Agent 可以顺手审计。
4. Audit Agent 可以顺手改代码。
```

这些都是错误用法。

---

## 2. 一句话定义

> **AgentFlow 不直接控制 Codex。用户需要在 Codex 里按角色建立 3 个独立线程：Spec Agent、Build Agent、Audit Agent。每个线程只做一种工作，不能混用。**

---

## 3. 目标

本需求要实现：

```text
1. 在工作台告诉用户如何在 Codex 里使用 3 个角色。
2. 在任务详情里明确这个任务应该交给哪个 Agent。
3. 在高级页展示完整 Agent 角色规则。
4. 为每个角色提供可复制启动指令。
5. 按 issueCategory / requiredAgentRole 生成不同任务包。
6. 阻止用户把 Audit Issue 交给 Build Agent，或把 Spec Issue 交给 Audit Agent。
```

---

## 4. 非目标

本需求不做：

```text
1. 不调用 Codex API。
2. 不自动创建 Codex 线程。
3. 不做 OAuth 登录。
4. 不做远程 Agent 管理。
5. 不做多 Agent 自动调度。
6. 不让 App 页面直接执行 Agent。
7. 不在 App 里提供“运行 Agent”按钮。
```

---

## 5. 用户如何在 Codex 里创建 3 个角色

### 5.1 用户操作方式

用户需要在 Codex 中创建 3 个独立线程：

```text
1. Spec Agent 线程
2. Build Agent 线程
3. Audit Agent 线程
```

每个线程第一次启动时，复制 AgentFlow 提供的对应启动指令。

### 5.2 线程命名建议

建议用户在 Codex 中将线程命名为：

```text
AgentFlow / Spec Agent
AgentFlow / Build Agent
AgentFlow / Audit Agent
```

或者中文：

```text
AgentFlow / 需求助手
AgentFlow / 执行助手
AgentFlow / 审计助手
```

### 5.3 关键规则

页面必须提醒用户：

```text
不要在一个 Codex 线程里混用多个角色。
```

解释文案：

```text
不要让同一个 Codex 线程一会儿写代码、一会儿审计。这样容易混淆边界。
```

---

## 6. 页面放置位置

### 6.1 主入口：工作台

工作台新增一个轻量说明区：

```text
Codex 角色使用说明
```

位置建议：

```text
工作台
├── 项目状态
├── 当前任务
├── 最近活动
└── Codex 角色使用说明
```

如果工作台空间紧张，可以放成可折叠区域。

默认展开规则：

```text
新项目 / 没有任务时：默认展开
已有任务后：默认折叠
```

### 6.2 任务详情：显示当前任务对应角色

任务详情必须显示：

```text
任务类型
执行角色
Codex 线程
```

例如 Spec Issue：

```text
任务类型：需求任务
执行角色：执行助手 / Build Agent
Codex 线程：AgentFlow / Build Agent
```

例如 Audit Issue：

```text
任务类型：审计任务
执行角色：审计助手 / Audit Agent
Codex 线程：AgentFlow / Audit Agent
```

### 6.3 高级页：完整规则

高级页新增：

```text
Agent 角色规则
```

展示完整能力边界：

```text
Spec Agent 能做什么 / 不能做什么
Build Agent 能做什么 / 不能做什么
Audit Agent 能做什么 / 不能做什么
roles.json 原始规则
```

---

## 7. 不建议放置的位置

不要放在：

```text
登录页
首次引导
底部状态栏
左侧项目 Tree
交付页主按钮区
审计页主按钮区
```

原因：

```text
Base 版本会去掉登录 / 首次引导阻断。
底部状态栏空间太小。
左侧 Tree 只做导航。
交付 / 审计页不应该承担角色教学。
```

---

## 8. 工作台卡片需求

### 8.1 标题

```text
Codex 角色使用说明
```

### 8.2 主说明

```text
AgentFlow 不直接控制 Codex。你需要在 Codex 里按角色开线程，每个线程只做一种工作。
```

### 8.3 补充说明

```text
不要让同一个 Codex 线程一会儿写代码、一会儿审计。这样容易混淆边界。
```

### 8.4 三个角色卡片

#### 需求助手 / Spec Agent

展示：

```text
确认需求 · 整理规格 · 生成任务
```

按钮：

```text
复制启动指令
```

#### 执行助手 / Build Agent

展示：

```text
任务打包 · 执行改动 · 写回结果
```

按钮：

```text
复制启动指令
```

#### 审计助手 / Audit Agent

展示：

```text
审计交付 · 核对证据 · 生成报告
```

按钮：

```text
复制启动指令
```

---

## 9. 角色启动指令

### 9.1 Spec Agent 启动指令

```text
你现在是 AgentFlow 的 Spec Agent。

你只做三件事：
1. 确认用户需求。
2. 整理 SPEC。
3. 生成 Issue。

你不能做：
- 不改代码
- 不执行命令
- 不生成 release
- 不写 audit report
- 不执行 Build Agent 或 Audit Agent 的任务

你必须遵守：
- 只写 .agentflow/input/**
- 不修改用户源码
- 不写 .agentflow/execute/**
- 不写 .agentflow/output/release/**
- 不写 .agentflow/output/audit/**

如果用户要求你改代码、执行任务或审计，请停止并提示需要切换到正确 Agent。
```

### 9.2 Build Agent 启动指令

```text
你现在是 AgentFlow 的 Build Agent。

你只能执行：
issueCategory = spec
requiredAgentRole = build-agent

你要做：
1. 读取指定 Issue。
2. 按任务包执行改动。
3. 写入 execute 过程记录。
4. 写入 evidence。
5. 写入 release delivery。

你不能做：
- 不执行 audit issue
- 不写 audit report
- 不写 findings.json
- 不写 evidence-map.json
- 不写 traceability.json
- 不越过任务边界
- 不创建远程 PR
- 不 merge
- 不 deploy

如果任务不是 spec issue，必须停止。
如果 requiredAgentRole 不是 build-agent，必须停止。
```

### 9.3 Audit Agent 启动指令

```text
你现在是 AgentFlow 的 Audit Agent。

你只能执行：
issueCategory = audit
requiredAgentRole = audit-agent

你要做：
1. 读取 Audit Issue。
2. 读取关联 SPEC / Issue / Evidence / Release。
3. 检查是否符合需求、范围和边界。
4. 写入 audit report。
5. 写入 findings.json。
6. 写入 evidence-map.json。
7. 写入 traceability.json。

你不能做：
- 不改代码
- 不执行 spec issue
- 不生成 release
- 不创建 PR
- 不 merge
- 不 deploy
- 不修改用户源码

如果任务不是 audit issue，必须停止。
如果 requiredAgentRole 不是 audit-agent，必须停止。
```

---

## 10. 任务页角色提示

### 10.1 Spec Issue

任务详情展示：

```text
任务类型：需求任务
执行角色：执行助手 / Build Agent
Codex 线程：AgentFlow / Build Agent
```

按钮：

```text
复制 Build Agent 任务包
```

### 10.2 Audit Issue

任务详情展示：

```text
任务类型：审计任务
执行角色：审计助手 / Audit Agent
Codex 线程：AgentFlow / Audit Agent
```

按钮：

```text
复制 Audit Agent 任务包
```

### 10.3 禁止用户选择错误角色

不要让用户在 UI 里自由选择 Agent。

错误设计：

```text
请选择 Agent：Spec / Build / Audit
```

正确设计：

```text
这个任务需要：Build Agent
```

或者：

```text
这个任务需要：Audit Agent
```

角色由 Issue 决定，不由用户临时选择。

---

## 11. Handoff 规则

### 11.1 Handoff 必须包含角色字段

每个任务包必须包含：

```json
{
  "handoffVersion": "agent-handoff.v1",
  "issueId": "AF-104",
  "issueCategory": "spec",
  "requiredAgentRole": "build-agent",
  "codexThreadName": "AgentFlow / Build Agent"
}
```

Audit Issue：

```json
{
  "handoffVersion": "agent-handoff.v1",
  "issueId": "audit-release-v0.1.0",
  "issueCategory": "audit",
  "requiredAgentRole": "audit-agent",
  "codexThreadName": "AgentFlow / Audit Agent"
}
```

### 11.2 Handoff 指令必须提醒

任务包中必须包含：

```text
如果你不是 requiredAgentRole，请停止执行。
如果 issueCategory 不属于你，请停止执行。
不要执行其他 Agent 的任务。
不要越过任务边界。
```

---

## 12. 复制按钮规则

### 12.1 工作台角色说明卡

每个角色一个按钮：

```text
复制 Spec Agent 启动指令
复制 Build Agent 启动指令
复制 Audit Agent 启动指令
```

复制后提示：

```text
已复制。请粘贴到 Codex 对应线程。
```

### 12.2 任务详情

Spec Issue：

```text
复制 Build Agent 任务包
```

Audit Issue：

```text
复制 Audit Agent 任务包
```

复制后提示：

```text
已复制。请粘贴到 AgentFlow / Build Agent 线程。
```

或：

```text
已复制。请粘贴到 AgentFlow / Audit Agent 线程。
```

---

## 13. 高级页角色规则

高级页新增一个分类：

```text
Agent 角色
```

展示：

```text
roles.json
Spec Agent 规则
Build Agent 规则
Audit Agent 规则
```

### 13.1 人话说明

```text
Spec Agent 只整理需求，不执行代码。
Build Agent 只执行需求任务，不做审计。
Audit Agent 只审计交付，不改代码。
```

### 13.2 Raw JSON

高级页可以展示：

```text
.agentflow/define/agent/roles.json
```

但必须标注：

```text
只读诊断
```

---

## 14. 数据来源

### 14.1 角色规则来源

```text
.agentflow/define/agent/roles.json
AGENTS.md
.agentflow/define/agent/Agentflow.md
```

### 14.2 任务角色来源

```text
.agentflow/input/issues/*.json
```

字段：

```text
issueCategory
requiredAgentRole
```

### 14.3 状态来源

```text
.agentflow/state/indexes/issue-status.json
```

建议同步包含：

```text
issueCategory
requiredAgentRole
```

---

## 15. 前端组件建议

新增或调整：

```text
CodexRoleGuideCard
CodexRoleCard
AgentStartupInstructionCopyButton
IssueRequiredAgentBadge
IssueCategoryBadge
HandoffTargetThreadHint
AgentRoleRulesPanel
```

### 15.1 工作台

```text
WorkspaceHomePage
└── CodexRoleGuideCard
```

### 15.2 任务页

```text
TaskContractReader
├── IssueCategoryBadge
├── IssueRequiredAgentBadge
└── HandoffTargetThreadHint
```

### 15.3 高级页

```text
AdvancedPage
└── AgentRoleRulesPanel
```

---

## 16. 文案规范

### 16.1 角色中文名

```text
Spec Agent = 需求助手
Build Agent = 执行助手
Audit Agent = 审计助手
```

### 16.2 任务类型中文名

```text
spec issue = 需求任务
audit issue = 审计任务
```

### 16.3 执行角色中文名

```text
build-agent = 执行助手
audit-agent = 审计助手
spec-agent = 需求助手
```

### 16.4 不要写

```text
智能体调度
多 Agent 编排
角色授权矩阵
agent capability runtime
```

普通页面用大白话。

高级页可以保留英文和 raw JSON。

---

## 17. 验收标准

必须满足：

```text
1. 工作台有 Codex 角色使用说明。
2. 用户能复制 3 个角色的启动指令。
3. 任务详情能显示任务类型。
4. 任务详情能显示执行角色。
5. Spec Issue 显示复制 Build Agent 任务包。
6. Audit Issue 显示复制 Audit Agent 任务包。
7. UI 不提供“选择 Agent”下拉框。
8. 高级页能查看 Agent 角色规则。
9. Handoff package 包含 issueCategory / requiredAgentRole / codexThreadName。
10. 复制后提示用户粘贴到正确 Codex 线程。
11. npm --prefix apps/desktop run build 通过。
```

---

## 18. 不做事项

```text
不自动创建 Codex 线程
不调用 Codex API
不做登录
不让用户在 UI 里自由选择 Agent
不让一个 Codex 线程混用多个角色
不把角色判断建立在 Agent 自称上
不把 Agent 规则只藏在高级页
```

---

## 19. Codex 实现指令

```text
你现在只做这个任务：AgentFlow Codex Role Usage Guide V1。

背景：
AgentFlow 当前不直接调用 Codex API。用户需要在 Codex 中手动创建 3 个独立线程：Spec Agent、Build Agent、Audit Agent。App 必须告诉用户如何创建和使用这些角色，并在任务详情中明确每个 Issue 应该交给哪个 Agent。

目标：
1. 在工作台新增 Codex 角色使用说明。
2. 提供 3 个角色的启动指令复制按钮。
3. 在任务详情显示任务类型和执行角色。
4. 按 Issue 类型生成正确 Handoff。
5. 在高级页展示完整角色规则。

范围：
- apps/desktop/src/**
- .agentflow/define/agent/**
- AGENTS.md 生成规则
- docs/requirements/**
- 相关测试

具体要求：
1. 工作台新增 CodexRoleGuideCard。
2. CodexRoleGuideCard 显示：
   - 需求助手 / Spec Agent
   - 执行助手 / Build Agent
   - 审计助手 / Audit Agent
3. 每个角色提供“复制启动指令”按钮。
4. 任务详情显示：
   - 任务类型
   - 执行角色
   - 推荐 Codex 线程名
5. Handoff package 必须包含：
   - issueId
   - issueCategory
   - requiredAgentRole
   - codexThreadName
6. Spec Issue 只能复制 Build Agent 任务包。
7. Audit Issue 只能复制 Audit Agent 任务包。
8. 高级页新增 Agent 角色规则分类。
9. 不允许 UI 出现“选择 Agent”下拉框。
10. 复制成功后提示用户粘贴到对应 Codex 线程。

禁止：
- 不要调用 Codex API。
- 不要自动创建 Codex 线程。
- 不要做登录。
- 不要让用户自由选择 Agent。
- 不要把 Audit Issue 交给 Build Agent。
- 不要把 Spec Issue 交给 Audit Agent。
- 不要只在高级页说明角色规则。

验证：
1. 工作台能看到角色使用说明。
2. 三个启动指令都能复制。
3. Spec Issue 显示执行助手 / Build Agent。
4. Audit Issue 显示审计助手 / Audit Agent。
5. 任务包包含 requiredAgentRole。
6. 高级页能查看角色规则。
7. npm --prefix apps/desktop run build 通过。

输出：
- 改了哪些文件
- 工作台角色说明如何展示
- 三个启动指令内容
- 任务详情如何显示执行角色
- Handoff 结构如何调整
- build 结果
```
