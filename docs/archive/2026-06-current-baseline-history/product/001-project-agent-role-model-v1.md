# 001 - Project Agent Role Model V1

创建日期：2026-06-17
执行者：Codex

## Purpose

本文定义 AgentFlow 以 Project 为中心的 Agent 角色模型。

AgentFlow 打开的入口是“添加项目”。因此所有 Agent 角色都必须围绕 Project 目标和最终交付服务，而不是围绕单个工具、页面或任务服务。

## Role Overview

```text
Project
├── Goal Agent
├── Spec Agent
├── Work Agent
├── Audit Agent
├── Delivery Agent
└── Specialist Agent / Skill
```

## Goal Agent / 项目大脑

### Responsibility

Goal Agent 负责从项目大局维护方向。

它持续回答：

- 这个项目为什么做？
- 项目最终要交付什么结果？
- 当前计划是否仍然服务于目标？
- 当前执行是否偏离目标？
- 是否出现新的约束、风险或范围变化？
- 下一步应该继续、调整、暂停还是交付？

### Inputs

- 用户原始需求
- 当前 GOAL.md
- 当前 PLAN.md
- DECISIONS.md
- SpecProject / SpecIssue
- Issue 状态
- Evidence
- Audit Report
- Delivery Report

### Outputs

- Goal Draft
- Goal Update Draft
- Plan Update Proposal
- Scope Change Proposal
- Next Decision Proposal
- Project Health Summary

### Boundaries

Goal Agent 不允许：

- 不写代码。
- 不执行任务。
- 不直接修改 Issue 状态。
- 不直接标记项目完成。
- 不代替用户确认范围变化。
- 不绕过 Spec Agent 生成可执行任务。

## Spec Agent / 规划拆解

### Responsibility

Spec Agent 负责把 Goal / Plan 转成可确认的结构化计划和执行合同。

它处理：

- 需求分类：产品 / 设计 / 技术 / 修复 / 审计 / 混合。
- Goal Draft 到 Plan Draft 的转换。
- Milestone 草案。
- Issue Contract 草案。
- 验收标准和证据要求。
- 执行边界和非目标。

### Inputs

- 用户输入
- Goal Draft
- PLAN.md
- DECISIONS.md
- 项目上下文
- 技术 / 设计 / 产品约束

### Outputs

- Plan Draft
- Milestone Draft
- Issue Contract Draft
- SpecProject Draft
- SpecIssue Draft
- Clarification Questions

### Boundaries

Spec Agent 不允许：

- 不执行代码修改。
- 不直接进入 Work Loop。
- 不从原始需求直接写入可执行 Issue。
- 不跳过用户确认。
- 不把 Audit 任务混入 Build 任务。

## Work Agent / 执行代理

### Responsibility

Work Agent 负责执行确认后的 Issue Contract。

它可以根据项目类型加载不同能力，例如：

- 产品文档执行
- 设计实现
- 前端实现
- 后端实现
- 测试补充
- 文档更新
- 部署准备

### Inputs

- 已确认的 SpecIssue / Issue Contract
- Allowed Files / Forbidden Files
- Validation Commands
- Evidence Requirements
- 当前工作区状态

### Outputs

- 代码或文档改动
- 执行记录
- 验证结果
- Evidence
- Known Limitations
- Handoff Summary

### Boundaries

Work Agent 不允许：

- 不修改 Goal。
- 不修改 Plan。
- 不扩大 Scope。
- 不执行未确认 Issue。
- 不绕过验证。
- 不创建审计结论。
- 不隐藏失败。

## Audit Agent / 审计验收

### Responsibility

Audit Agent 负责判断执行结果是否可信。

它检查：

- 是否满足 Goal。
- 是否满足 Issue Contract。
- 是否越界。
- Evidence 是否完整。
- Validation 是否可信。
- Delivery 是否可接受。

### Inputs

- Goal / Plan
- Issue Contract
- Work Agent 输出
- Evidence
- Validation Output
- Diff Summary
- Delivery Draft

### Outputs

- Audit Report
- Pass / Blocked / Needs Repair 判断
- Risk Findings
- Evidence Gap
- Repair Recommendation

### Boundaries

Audit Agent 不允许：

- 不修代码。
- 不执行任务。
- 不代替 Work Agent。
- 不直接更新 Goal / Plan。
- 不在证据不足时给出通过结论。

## Delivery Agent / 交付整理

### Responsibility

Delivery Agent 负责把项目结果整理成人类可接收的交付物。

它输出：

- 交付摘要
- 完成内容
- 未完成内容
- 已知限制
- 验证结果
- Evidence 索引
- 后续建议
- Delivery Report

### Inputs

- Goal / Plan
- Completed Issues
- Evidence
- Audit Report
- User-facing Context

### Outputs

- DELIVERY.md
- Release / Handoff Summary
- Change Summary
- Next Stage Recommendation

### Boundaries

Delivery Agent 不允许：

- 不判断项目方向。
- 不执行代码修改。
- 不代替 Audit Agent 做通过判断。
- 不自动发布远程版本。
- 不把交付整理等同于项目关闭。

## Specialist Agent / Skill

Specialist Agent 是按需能力，不是常驻顶层项目角色。

示例：

- Product Specialist
- Design Specialist
- Frontend Specialist
- Backend Specialist
- Test Specialist
- DevOps Specialist
- Docs Specialist

Specialist 由 Goal Agent / Spec Agent / Work Agent 在特定阶段调用，用于补充专业能力。它不拥有 Project 方向，也不独立驱动项目。

## Role Flow

```text
Goal Agent
-> Spec Agent
-> Work Agent
-> Audit Agent
-> Delivery Agent
-> Goal Agent
```

Delivery 后必须回到 Goal Agent 做项目回看。项目是否继续、调整、暂停或进入下一阶段，由 Goal Agent 生成建议并等待用户确认。
