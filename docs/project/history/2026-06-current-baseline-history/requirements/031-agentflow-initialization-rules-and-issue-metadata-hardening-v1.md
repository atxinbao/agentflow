# 031 - AgentFlow Initialization Rules and Issue Metadata Hardening V1

创建日期：2026-06-07
执行者：Codex

## 背景

当前本地 `.agentflow/**` 里出现过 legacy 目录、Issue 目标信息缺失、Audit Agent 需要追问 `auditId` 等问题。直接手工修当前 `.agentflow/**` 生成物不会进入有效 PR，也不会影响下一次项目初始化。

本需求把规则固化到初始化代码、Agent 模板、Issue schema、validator、state blocker 和前端 handoff 展示里。

## 目标

- 新项目初始化后不再生成 legacy 事实源目录。
- Spec Issue 和 Audit Issue 都带完整执行目标 metadata。
- Release delivery 后自动生成 Audit Issue，Audit Issue 是审计执行入口。
- 复制 Agent handoff 前校验目标完整性，缺目标时不允许复制。
- 当前本地 `.agentflow/**` 生成物不是本轮修复入口，不进入 PR。

## 范围

- `crates/input/**`
- `crates/output/**`
- `crates/state/**`
- `crates/agent-manual/**`
- `apps/desktop/src-tauri/**`
- `apps/desktop/src/**`
- `docs/requirements/**`
- 相关测试和 fixtures

## 不做事项

- 不提交当前本地 `.agentflow/**` 生成物。
- 不手工修 `audit-001` 或 `AF-RULES-LEGACY-001`。
- 不创建 GitHub Release。
- 不创建远程 Issue。
- 不恢复 `.agentflow/spec/**` 或 `.agentflow/goal-tree/**`。
- 不新增模型调用。

## 初始化规则

新初始化不得生成这些目录：

- `.agentflow/define/goals/`
- `.agentflow/define/milestones/`
- `.agentflow/define/issues/`
- `.agentflow/spec/**`
- `.agentflow/goal-tree/**`

当前任务事实源只能是：

- `.agentflow/input/issues/**`

当前 SPEC 事实源只能是：

- `.agentflow/input/specs/drafts/**`
- `.agentflow/input/specs/approved/**`

人类直接给的需求不要求手写 raw 目录。正确流程是：人类把需求交给 Spec Agent，Spec Agent 生成 Draft SPEC，人类确认后写入 approved SPEC 和 input issues。

## Agent 角色规则

### Spec Agent

- 只澄清需求、生成 Requirement Intake Result、生成 SPEC Draft Preview。
- 人类确认后，只写 approved SPEC 和 input issues。
- 不写源码。
- 不执行命令。
- 不写 execute / release / audit。
- 不写 legacy `.agentflow/spec/**` 或 `.agentflow/goal-tree/**`。

### Build Agent

- 只处理 `issueCategory=spec`。
- `requiredAgentRole` 必须是 `build-agent`。
- 只根据 Spec Issue 生成任务包、execute、evidence、release delivery。
- 不处理 Audit Issue。
- 不写 audit report。

### Audit Agent

- 只处理 `issueCategory=audit`。
- `requiredAgentRole` 必须是 `audit-agent`。
- 只读取 release / evidence / issue / spec。
- 只写 `.agentflow/output/audit/**`。
- 不改源码。
- 不生成 release。
- 不处理 Spec Issue。

## Issue Metadata

所有 Issue 支持：

- `issueCategory`
- `requiredAgentRole`
- `displayStatus`

旧 Issue 兼容规则：

- 缺 `issueCategory` 时默认 `spec`。
- 缺 `requiredAgentRole` 时默认 `build-agent`。
- 缺 `displayStatus` 时按 `status` 推导展示状态。

Spec Issue 必须包含：

- `sourceSpecId`
- `sourceSpecPath`
- `issuePath`
- `handoffId`
- `contextPackPath`
- `allowedPaths`
- `forbiddenPaths`
- `forbiddenActions`
- `validationCommands`
- `expectedOutputs.executeRunDir`
- `expectedOutputs.evidencePath`
- `expectedOutputs.releaseDeliveryDir`

Audit Issue 必须包含：

- `audit.auditId`
- `audit.trigger`
- `audit.sourceReleaseId`
- `audit.sourceDeliveryPath`
- `audit.auditOutputDir`
- `audit.expectedOutputs.audit.json`
- `audit.expectedOutputs.audit-report.md`
- `audit.expectedOutputs.findings.json`
- `audit.expectedOutputs.evidence-map.json`
- `audit.expectedOutputs.traceability.json`

## Release Audit 入口

Release delivery 生成后必须生成 Audit Issue：

```text
.agentflow/input/issues/audit-<release-id>.json
```

Audit Issue 是 Audit Agent 执行入口。

`audit-request.json` 可以保留为兼容 metadata，但不能作为 Agent 执行入口。

## Handoff 规则

Build Agent handoff 必须包含：

- `issueId`
- `issueCategory=spec`
- `requiredAgentRole=build-agent`
- `sourceSpecId`
- `sourceSpecPath`
- `issuePath`
- `expectedOutputs.executeRunDir`
- `expectedOutputs.evidencePath`
- `expectedOutputs.releaseDeliveryDir`

Audit Agent handoff 必须包含：

- `issueId`
- `issueCategory=audit`
- `requiredAgentRole=audit-agent`
- `auditId`
- `sourceReleaseId`
- `auditOutputDir`
- `expectedOutputs`

复制 Handoff 前必须校验目标完整性。缺目标时提示：

```text
这个任务包不完整，缺少执行目标。请先修复 Issue 元数据。
```

## State Blocker

Issue 缺少目标 metadata 时写入 blocker：

- `action = copy-handoff`
- `reason = 任务缺少执行目标，不能生成任务包`

角色不匹配时，不接受写回、不更新 done，并写 blocker / timeline。

## 前端展示

任务详情必须显示：

- 任务类型
- 执行角色
- 来源 SPEC 或审计目标
- 输出位置摘要

Spec Issue 显示来源规格和 Evidence / Release Delivery 输出位置。

Audit Issue 显示审计目标、关联 Release 和 Audit Report 输出位置。

用户不能在 UI 里自由选择 Agent。Agent 角色来自 `issueCategory` / `requiredAgentRole`。

## 验收标准

- 初始化不生成 legacy define goals / milestones / issues。
- 初始化不生成 `.agentflow/spec/**` 或 `.agentflow/goal-tree/**`。
- 旧 Issue 可兼容读取。
- 新 Spec Issue 会补齐 build target metadata。
- Release 后生成带 audit target metadata 的 Audit Issue。
- Handoff 缺目标时不能复制。
- Build Agent 不接受 Audit Issue。
- Audit Agent 不接受 Spec Issue。
- Browser Preview mock 仍可使用。
- 真实 Desktop 客户端不 silent fallback mock。

## 验证命令

```bash
cargo check --workspace
cargo test --workspace
npm --prefix apps/desktop run build
git diff --check
```
