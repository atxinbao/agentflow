# 016 - Desktop Design System V1

创建日期：2026-06-05  
执行者：Codex  
状态：待开发  
版本：final-draft

---

## 用户目标

AgentFlow 已经完成：

```text
015 - Human-Agent Guided Experience V1
```

现在前端体验方向已经明确：

```text
Project Home
Next Step Card
Spec / Build / Audit 三个 Agent 卡片
Codex Handoff
Human Audit
Status Channel
Advanced Details
```

但在正式实现这些页面前，需要先做一层稳定的 Design System。

大白话：

> 先把颜色、字体、卡片、按钮、状态、空态、阻断态、代码块这些基础件做好。  
> 后面做 Project Home、Spec Flow、Codex Handoff、Audit 页面时，就不会每个页面各写一套样式。

---

## 一句话定义

> **Desktop Design System V1 是 AgentFlow Desktop 的前端设计系统底座。它不新增后端能力，不实现完整业务页面，只提供统一的视觉变量、基础组件、状态组件和页面壳，让后续 Human-Agent Guided Experience 能按同一套高保真体验落地。**

---

# 1. 当前问题

现在 Desktop 前端已经有不少功能页面：

```text
Agent Manual
Project Files
Panel
Input
Execute
Output
State
Audit
Status Channel
```

但前端样式还偏功能拼装。

问题是：

```text
1. 视觉层级不够统一。
2. 每个模块容易自己定义颜色和间距。
3. 状态表达不够稳定。
4. 空态 / 阻断态 / loading 态缺少统一组件。
5. Codex Handoff 这类关键页面还缺少可复用的代码块组件。
6. 后续如果直接做 Project Home，会很容易变成“一堆卡片堆起来”。
```

所以 016 先做 Design System。

---

# 2. 设计原则

## 2.1 继续使用当前客户端方案

继续使用：

```text
Tauri Desktop
React / TypeScript
Rust backend commands
CSS design system
```

不重写成：

```text
SwiftUI
AppKit
WPF
Qt
GPUI
Electron
```

原因：

```text
AgentFlow 当前核心不是重写客户端，而是把 Human-Agent 动线做顺。
Tauri + React 已经足够做高保真体验。
```

---

## 2.2 不引入重型组件库

本阶段不要引入：

```text
MUI
Ant Design
Chakra
Mantine
```

原因：

```text
这些组件库有强风格，容易把 AgentFlow 做成普通 SaaS 后台。
```

V1 优先：

```text
React + CSS
```

如未来需要无样式基础能力，再考虑：

```text
Radix UI primitives
```

但 016 不引入。

---

## 2.3 高保真靠设计系统，不靠换框架

高保真的关键是：

```text
统一颜色
统一字体
统一间距
统一卡片
统一按钮
统一状态
统一空态
统一阻断态
统一动效
统一文案风格
```

不是换客户端技术栈。

---

## 2.4 文案必须遵守 plain-work-style

所有用户可见文案必须：

```text
先说结论
讲人话
少废话
不写官话
不写营销腔
给明确下一步
```

坏：

```text
通过智能化编排能力，赋能开发流程全链路闭环。
```

好：

```text
先把需求说清楚，再交给 Codex 做。
```

---

# 3. 目标范围

016 只做 Design System 底座。

做：

```text
1. 新增 design tokens。
2. 新增基础布局样式。
3. 新增基础组件。
4. 新增状态组件。
5. 新增代码块 / 复制组件。
6. 新增 Advanced Details 基础壳。
7. 新增 Browser Preview 展示 / smoke 验证。
8. 小范围替换现有页面中最安全的基础样式。
```

不做：

```text
不实现完整 Project Home。
不实现 Spec Agent Flow。
不实现 SPEC Review。
不实现 Codex Handoff 真实业务。
不改 Rust 后端。
不新增 Tauri command。
不调用模型。
不写 .agentflow。
不执行项目命令。
```

---

# 4. 目标文件结构

建议新增：

```text
apps/desktop/src/design/
├── tokens.css
├── typography.css
├── layout.css
├── components.css
├── states.css
├── motion.css
└── index.css
```

建议新增基础组件：

```text
apps/desktop/src/components/
├── Button.tsx
├── SurfaceCard.tsx
├── StatusChip.tsx
├── MetricCard.tsx
├── EmptyState.tsx
├── BlockedState.tsx
├── LoadingState.tsx
├── WarningState.tsx
├── CopyableCodeBlock.tsx
├── AdvancedDetailsDrawer.tsx
└── index.ts
```

建议新增设计系统预览：

```text
apps/desktop/src/features/design-system/
├── DesignSystemPreview.tsx
├── DesignSystemPreview.css
└── index.ts
```

如果项目当前已有 `components/` 或类似目录，优先复用，不重复建同名结构。

---

# 5. Design Tokens

## 5.1 颜色变量

新增 `tokens.css`：

```css
:root {
  --af-bg: #0B1020;
  --af-surface: #11182B;
  --af-surface-2: #151E33;
  --af-surface-3: #1A2540;
  --af-line: #26324D;
  --af-line-muted: #202A42;

  --af-text: #E8EEF8;
  --af-text-muted: #91A0B8;
  --af-text-subtle: #64748B;

  --af-primary: #7C9CFF;
  --af-mint: #5EEAD4;
  --af-success: #22C55E;
  --af-warning: #FBBF24;
  --af-danger: #FB7185;
  --af-purple: #A78BFA;
  --af-blue: #38BDF8;
  --af-orange: #FDBA74;
}
```

---

## 5.2 字体变量

```css
:root {
  --af-font-sans: Inter, -apple-system, BlinkMacSystemFont, "PingFang SC", "Microsoft YaHei", "Noto Sans CJK SC", sans-serif;

  --af-text-xs: 12px;
  --af-text-sm: 13px;
  --af-text-md: 15px;
  --af-text-lg: 18px;
  --af-text-xl: 22px;
  --af-text-2xl: 28px;
  --af-text-3xl: 34px;

  --af-leading-tight: 1.2;
  --af-leading-normal: 1.5;
  --af-leading-relaxed: 1.7;
}
```

---

## 5.3 间距和圆角

```css
:root {
  --af-space-1: 4px;
  --af-space-2: 8px;
  --af-space-3: 12px;
  --af-space-4: 16px;
  --af-space-5: 20px;
  --af-space-6: 24px;
  --af-space-8: 32px;
  --af-space-10: 40px;
  --af-space-12: 48px;

  --af-radius-sm: 8px;
  --af-radius-md: 12px;
  --af-radius-lg: 16px;
  --af-radius-xl: 22px;
  --af-radius-2xl: 28px;
}
```

---

## 5.4 阴影和动效

```css
:root {
  --af-shadow-card: 0 16px 40px rgba(0, 0, 0, 0.28);
  --af-shadow-soft: 0 10px 26px rgba(0, 0, 0, 0.18);

  --af-motion-fast: 120ms ease;
  --af-motion-normal: 180ms ease;
}
```

---

# 6. 基础组件

## 6.1 Button

路径：

```text
apps/desktop/src/components/Button.tsx
```

支持：

```text
variant = primary | secondary | danger | ghost
size = sm | md | lg
disabled
loading
leftIcon
rightIcon
```

要求：

```text
主按钮视觉明显。
每个阶段只鼓励一个 primary button。
disabled 状态要明显，但不要像错误。
```

---

## 6.2 SurfaceCard

路径：

```text
apps/desktop/src/components/SurfaceCard.tsx
```

用于：

```text
Next Step Card
Agent Card
Status Card
Advanced Detail Card
Audit Card
```

支持：

```text
title
description
tone
footer
compact
```

---

## 6.3 StatusChip

路径：

```text
apps/desktop/src/components/StatusChip.tsx
```

支持状态：

```text
ready
working
warning
blocked
failed
idle
```

用户展示文案：

```text
已就绪
准备中
有风险
已阻断
异常
未开始
```

注意：

```text
内部状态 enum 不直接暴露给用户。
```

---

## 6.4 MetricCard

用于：

```text
证据数量
交付数量
审计数量
未完成数量
文件数量
诊断数量
```

要求：

```text
数字清楚
说明短
不要大面积彩色
```

---

## 6.5 EmptyState

用于：

```text
没有项目
没有需求
没有 SPEC
没有交付
没有审计
```

结构：

```text
标题
说明
主动作
次动作
```

文案例子：

```text
还没有需求。
先告诉 Spec Agent 你想做什么。
```

---

## 6.6 BlockedState

用于阻断。

结构：

```text
标题
原因
下一步动作
可展开的技术详情
```

文案例子：

```text
还不能交给 Codex。
原因是：这个需求还没有确认成 SPEC。
```

不要显示：

```text
Workflow gate blocked: missing approved spec
```

除非在高级详情里。

---

## 6.7 LoadingState

用于：

```text
准备工作手册
读取项目现场
刷新状态
加载审计报告
```

要求显示具体步骤，不只是 spinner：

```text
正在准备 Agent 工作手册
正在读取项目现场
正在检查 Git 状态
正在生成工作流状态
```

---

## 6.8 WarningState

用于：

```text
Panel degraded
Shadow Agent entry detected
Locale fallback
Git not found
```

要求：

```text
可以继续，但要说明风险。
```

---

## 6.9 CopyableCodeBlock

用于：

```text
Codex Handoff
验证命令
调试信息
scope refs
```

支持：

```text
title
language
content
copy button
copied state
line wrap
max height
```

要求：

```text
复制按钮明显
复制成功有反馈
```

---

## 6.10 AdvancedDetailsDrawer

用于隐藏内部信息。

包含：

```text
manifest
skills-lock
workflow gates
blockers
panel manifest
input index
output index
locale.json
style.json
```

V1 可以先做基础壳：

```text
打开 / 关闭
标题
说明
children
```

不需要复杂树形 JSON viewer。

---

# 7. Design System Preview

新增一个预览组件：

```text
DesignSystemPreview
```

它展示：

```text
颜色
按钮
StatusChip
SurfaceCard
MetricCard
EmptyState
BlockedState
LoadingState
CopyableCodeBlock
AdvancedDetailsDrawer
```

用途：

```text
Browser Preview 可看
后续前端实现可复用
设计回归可检查
```

---

# 8. Browser Preview 接入

Browser Preview 中要能看到 Design System Preview。

可以做成：

```text
如果是 Browser Preview runtime
在 Advanced Details 或开发预览区展示 Design System Preview
```

也可以新增一个轻量入口：

```text
?preview=design-system
```

如果路由结构复杂，V1 可以只在现有 Browser Preview 数据中加入该组件的 smoke marker。

---

# 9. preview:smoke 更新

更新：

```text
apps/desktop/scripts/browser-preview-smoke.mjs
```

增加检查：

```text
Design System marker 存在
Button marker 存在
StatusChip marker 存在
SurfaceCard marker 存在
EmptyState marker 存在
BlockedState marker 存在
CopyableCodeBlock marker 存在
```

建议给组件加 data marker：

```tsx
data-agentflow-component="button"
data-agentflow-component="surface-card"
data-agentflow-component="status-chip"
data-agentflow-component="empty-state"
data-agentflow-component="blocked-state"
data-agentflow-component="copyable-code-block"
data-agentflow-design-system="v1"
```

---

# 10. 可访问性要求

基础组件必须支持：

```text
button disabled
aria-label
aria-busy
aria-live for loading / copied
keyboard focus visible
semantic section / header
```

Focus 样式统一：

```css
:focus-visible {
  outline: 2px solid var(--af-primary);
  outline-offset: 2px;
}
```

---

# 11. 与现有页面的关系

016 不要大改现有页面。

只做安全接入：

```text
1. 引入 design/index.css。
2. 让 Browser Preview 能展示 DesignSystemPreview。
3. 小范围把现有 OutputAuditPanel 的按钮 / 卡片样式迁移到 token。
4. 不改变 OutputAuditPanel 业务逻辑。
5. 不改变 Status Channel 数据逻辑。
```

如果迁移风险高，现有页面只引入 tokens，不强行替换组件。

---

# 12. 非目标

016 不做：

```text
不实现 Project Home V1。
不实现 Next Step Card 业务逻辑。
不实现 Spec Agent Flow。
不实现 SPEC Draft。
不实现 Codex Handoff 真实数据。
不改 Tauri commands。
不改 Rust 后端。
不改 .agentflow 写入逻辑。
不调用模型。
不自动执行 Codex。
不创建远程 PR。
不引入大型组件库。
不做完整主题系统。
不做浅色主题。
```

---

# 13. 写入边界

允许修改：

```text
apps/desktop/src/design/**
apps/desktop/src/components/**
apps/desktop/src/features/design-system/**
apps/desktop/src/browserPreviewData.ts
apps/desktop/src/App.tsx
apps/desktop/scripts/browser-preview-smoke.mjs
apps/desktop/src/**/*.css
docs/requirements/016-desktop-design-system-v1.md
docs/requirements/README.md
docs/requirements/next-requirements.md
verification.md
```

不允许修改：

```text
crates/**
apps/desktop/src-tauri/**
.agentflow/**
用户源码 fixture
```

除非只是前端类型引用需要轻微调整，但原则上 016 不改 Rust。

---

# 14. 验收标准

```text
- [ ] 新增 docs/requirements/016-desktop-design-system-v1.md。
- [ ] 新增 apps/desktop/src/design/tokens.css。
- [ ] 新增 apps/desktop/src/design/typography.css。
- [ ] 新增 apps/desktop/src/design/layout.css。
- [ ] 新增 apps/desktop/src/design/components.css。
- [ ] 新增 apps/desktop/src/design/states.css。
- [ ] 新增 apps/desktop/src/design/motion.css。
- [ ] 新增 apps/desktop/src/design/index.css。
- [ ] App 入口引入 design/index.css。
- [ ] 新增 Button 组件。
- [ ] 新增 SurfaceCard 组件。
- [ ] 新增 StatusChip 组件。
- [ ] 新增 MetricCard 组件。
- [ ] 新增 EmptyState 组件。
- [ ] 新增 BlockedState 组件。
- [ ] 新增 LoadingState 组件。
- [ ] 新增 WarningState 组件。
- [ ] 新增 CopyableCodeBlock 组件。
- [ ] 新增 AdvancedDetailsDrawer 组件。
- [ ] 组件包含 data-agentflow-component marker。
- [ ] 新增 DesignSystemPreview。
- [ ] Browser Preview 可展示 DesignSystemPreview 或可被 smoke 检查。
- [ ] preview:smoke 检查 design system marker。
- [ ] 所有用户文案遵守 plain-work-style。
- [ ] 不引入重型组件库。
- [ ] 不改 Rust 后端。
- [ ] 不新增 Tauri command。
- [ ] 不调用模型。
- [ ] 不写 .agentflow。
- [ ] 不自动执行 Codex。
- [ ] 不创建远程 PR。
```

---

# 15. 验证命令

必须执行：

```bash
npm --prefix apps/desktop run build
npm --prefix apps/desktop run preview:smoke
cargo test -p agentflow-desktop
cargo test
cargo fmt --check
git diff --check
```

如果确实只改前端且 `cargo test` 过慢，也至少执行：

```bash
cargo test -p agentflow-desktop
cargo fmt --check
git diff --check
```

但 PR 说明必须解释为什么没有跑全量 `cargo test`。

---

# 16. PR 说明要求

PR 描述必须说明：

```text
1. 016 是 Design System 底座，不是 Project Home 实现。
2. 新增了哪些 tokens。
3. 新增了哪些基础组件。
4. Browser Preview 如何展示 / 验证 Design System。
5. preview:smoke 增加了哪些检查。
6. 是否引入组件库：必须说明没有。
7. 是否改 Rust 后端：必须说明没有。
8. 是否改业务逻辑：必须说明没有。
9. 验证命令和结果。
```

---

# 17. Codex 执行指令

```md
请执行 016 - Desktop Design System V1。

目标：
为 AgentFlow Desktop 新增前端设计系统底座。只做 design tokens、基础组件、状态组件、代码块、Advanced Details 基础壳和 Browser Preview 验证。不要实现完整 Project Home，也不要改后端。

必须遵守：
1. 继续使用 Tauri + React 当前方案。
2. 不引入 MUI / Ant Design / Chakra / Mantine 等重型组件库。
3. 不新增 Tauri command。
4. 不改 Rust 后端模型。
5. 不调用模型。
6. 不写 .agentflow。
7. 不执行 Codex。
8. 不创建远程 PR。
9. 不实现 Spec Agent Flow。
10. 不实现 Codex Handoff 真实业务。
11. 不实现完整 Project Home。
12. 所有用户文案遵守 plain-work-style。

实现范围：
- 新增 docs/requirements/016-desktop-design-system-v1.md。
- 新增 apps/desktop/src/design/**。
- 新增 apps/desktop/src/components/** 基础组件。
- 新增 DesignSystemPreview。
- App 入口引入 design/index.css。
- Browser Preview 能展示或 smoke 检查 Design System。
- 更新 preview:smoke，检查 data-agentflow-design-system 和基础组件 markers。
- 小范围复用 tokens，不改变业务逻辑。
- 更新 requirements index 和 verification。

验证命令：
- npm --prefix apps/desktop run build
- npm --prefix apps/desktop run preview:smoke
- cargo test -p agentflow-desktop
- cargo test
- cargo fmt --check
- git diff --check
```

---

# 18. 完成定义

完成后，AgentFlow Desktop 应该拥有：

```text
统一颜色
统一字体
统一间距
统一按钮
统一卡片
统一状态标签
统一空态
统一阻断态
统一 loading 态
统一代码块
统一高级详情壳
```

后续 017 / Project Home 实现时，不再从零写样式。

最终一句话：

> **016 先把高保真体验的地基打好：统一设计变量和基础组件，再做 Project Home、Next Step Card 和三 Agent 动线。**
