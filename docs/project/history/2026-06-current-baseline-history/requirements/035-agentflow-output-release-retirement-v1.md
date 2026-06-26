# AgentFlow Output Release Retirement V1

> 文档类型：开发需求
> 日期：2026-06-15
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

AgentFlow 正在从旧的目录流程：

```text
input -> execute -> output -> state
```

迁移到新的任务状态流：

```text
docs/requirements -> spec issue -> workflow event -> task projection -> task page
```

新架构已经明确：

```text
任务运行事实留在 .agentflow/tasks/<issue-id>/**
验证证据留在 .agentflow/tasks/<issue-id>/evidence/**
公开交付记录写到 PR/MR body、CHANGELOG.md 或 Release notes
```

因此旧的本地交付目录不再成立：

```text
.agentflow/output/release/**
```

它会让系统同时存在两套交付事实源：

```text
旧：.agentflow/output/release/<run-id>/delivery.json
新：task projection + PR/MR body + public delivery record
```

这会导致 Desktop、audit gate、Browser Preview、acceptance test 和 crate API 继续围绕旧 output 设计打补丁。

本需求用于收口旧 `output/release` 交付模型，后续开发必须按本文档拆分，不再继续零散清理。

---

## 2. 用户目标

用户在任务页查看一个 issue 时，只需要看到：

```text
任务状态流
每个状态留下的事件和证据
最终公开交付记录
```

用户不应该再理解：

```text
output release delivery
release delivery index
.agentflow/output/release/<run-id>/delivery.json
```

一句话：

```text
交付是任务状态流的结果，不是 output 目录里的另一套业务对象。
```

---

## 3. 新事实边界

### 3.1 需求入口

```text
docs/requirements/<requirement-id>.md
```

用途：

- 保留公开需求背景。
- 作为 Spec Agent 拆分 project / issues 的来源。
- 可以被外部审计和代码评审阅读。

### 3.2 本地任务合同

```text
.agentflow/spec/issues/<issue-id>.json
.agentflow/spec/projects/<project-id>.json
```

用途：

- 记录 issue 合同。
- 记录 workflowRef、priority、dependencies、allowedPaths、validationCommands。
- 不记录运行日志。
- 不记录交付产物。

### 3.3 本地运行事实

```text
.agentflow/events/task-events.jsonl
.agentflow/tasks/<issue-id>/runs/<run-id>/**
.agentflow/tasks/<issue-id>/evidence/**
.agentflow/projections/tasks/<issue-id>.json
.agentflow/projections/projects/<project-id>.json
```

用途：

- 事件日志记录事实。
- task runs 记录执行过程产物。
- evidence 记录验证证据。
- projection 驱动 Desktop UI。

### 3.4 公开交付记录

```text
PR/MR body
CHANGELOG.md
Release notes
```

用途：

- PR/MR body 是任务级交付说明。
- CHANGELOG.md / Release notes 是版本级交付说明。
- 外部审计通过 Git provider 和公开文档读取这些记录。

### 3.5 明确删除的事实源

```text
.agentflow/output/release/**
releaseDeliveries
OutputReleaseDelivery
prepare_release_delivery
load_release_delivery
```

这些不再是新架构事实源，也不作为兼容 fallback 保留。

---

## 4. 当前残留点

基于当前代码扫描，仍存在以下残留：

### 4.1 Rust crate

```text
crates/output
  仍定义 OutputReleaseDelivery / release delivery validation / output index。

crates/execute
  仍导出 prepare_release_delivery / load_release_delivery。
  仍有 completion 和测试依赖旧 output/release。

crates/acceptance
  仍调用 prepare_release_delivery。

crates/input
  旧 issue 默认模板仍生成 releaseDeliveryDir / output/release 路径。

crates/state
  测试 fixture 仍构造 output/release。

crates/loop
  少量测试或兼容逻辑仍引用 output/release。
```

### 4.2 Desktop

```text
apps/desktop/src/browserPreviewData.ts
  Browser Preview mock 仍包含 releaseDeliveries 和 output/release path。

apps/desktop/src/interaction/workflowRegression.test.ts
  仍断言 output.releaseDeliveries。

apps/desktop/src/types/status.ts
  OutputStatusSnapshot 仍暴露 releaseDeliveries。

apps/desktop/src/App.tsx
  仍存在旧交付页 / output bundle 读取 releaseDeliveries 的页面逻辑。
```

### 4.3 文档

旧需求文档中仍大量提到 `.agentflow/output/release/**`。这些分两类处理：

- 历史需求：保留原文，但标记为历史设计，不作为新开发依据。
- 当前架构文档：必须改成 `tasks/evidence + public delivery record`。

---

## 5. 范围

### 5.1 必须做

1. 移除新运行时对 `.agentflow/output/release/**` 的写入。
2. 移除新运行时对 `.agentflow/output/release/**` 的读取。
3. 移除 `releaseDeliveries` 作为 Desktop 任务页、状态页和 Browser Preview 的数据源。
4. 移除 `prepare_release_delivery` / `load_release_delivery` 的 active use case。
5. 移除 issue 默认模板里的 `releaseDeliveryDir`。
6. 把交付展示统一到 task projection 的 public delivery 字段。
7. 把验证证据统一到 `.agentflow/tasks/<issue-id>/evidence/**`。
8. 更新 Browser Preview mock，使用 task workflow / public delivery 数据。
9. 更新 regression tests，不再断言 `output.releaseDeliveries`。
10. 更新当前架构文档和 README 中的 active path 描述。

### 5.2 可以删除

```text
crates/output
crates/execute/src/delivery.rs
OutputReleaseDelivery*
release delivery validation
release delivery index
Desktop delivery page 中只服务 output/release 的逻辑
```

删除前必须确认没有 active crate / command / Desktop hook 依赖。

### 5.3 需要保留

```text
.agentflow/tasks/<issue-id>/runs/<run-id>/**
.agentflow/tasks/<issue-id>/evidence/**
.agentflow/audit/**
PR/MR body
CHANGELOG.md
Release notes
```

审计是独立流程，不在本需求里删除。

---

## 6. 非目标

- 不新增自动审计。
- 不修改审计报告结构。
- 不创建 GitHub Release。
- 不改 Spec Agent 拆 issue 的产品流程。
- 不把 delivery 重新放回 `.agentflow/tasks/<issue-id>/delivery/**`。
- 不新增 `public-delivery` crate。
- 不保留 `.agentflow/output/release/**` 兼容 fallback。

---

## 7. 目标架构

### 7.1 Build Agent 完成任务后

```text
1. 写 task events。
2. 写 .agentflow/tasks/<issue-id>/runs/<run-id>/**。
3. 写 .agentflow/tasks/<issue-id>/evidence/evidence.json。
4. 创建 PR/MR。
5. PR/MR body 记录任务交付说明。
6. 合并后写 merge proof。
7. projection 更新 issue 为 done。
8. 如有版本发布，再由 release 流程汇总 CHANGELOG / Release notes。
```

### 7.2 Desktop 任务页读取

```text
TaskProjection
  status timeline
  event stream
  current stage
  evidence summary
  publicDelivery
    prUrl
    prBodyStatus
    changelogPath
    releaseNotesUrl
```

Desktop 不再读取 output release index。

### 7.3 Audit 读取

审计读取：

```text
TaskProjection
Evidence
PR/MR public delivery
Merge proof
```

审计不再要求：

```text
.agentflow/output/release/<run-id>/delivery.json
```

---

## 8. 开发切片

### PR 1：Desktop mock 和任务页数据源清理

范围：

- `apps/desktop/src/browserPreviewData.ts`
- `apps/desktop/src/interaction/workflowRegression.test.ts`
- `apps/desktop/src/types/status.ts`
- `apps/desktop/src/App.tsx`

要求：

- Browser Preview 不再生成 `releaseDeliveries`。
- 任务页只读取 task projection public delivery。
- 输出状态里的 release delivery 计数删除或改为 public delivery summary。
- 旧交付页如果仍保留，只能展示 task projection 派生信息，不能读取 output/release。

### PR 2：execute active release delivery API 清理

范围：

- `crates/execute`
- `crates/acceptance`
- 相关 tests

要求：

- 移除 `prepare_release_delivery` active path。
- `build-agent complete` 不再准备 output release delivery。
- acceptance 改用 task evidence + public delivery projection。

### PR 3：output crate 退场

范围：

- `crates/output`
- `Cargo.toml`
- 依赖它的 crate

要求：

- 删除或拆空 output crate。
- 如果 audit 仍依赖其中模型，先迁移到 audit 自有模型或 task evidence 模型。
- workspace 不再把 output 作为 active business crate。

### PR 4：spec 模板和初始化清理

范围：

- `crates/spec`
- 旧 `crates/input` 迁移残留
- 初始化模板和 issue 默认字段

要求：

- issue 默认合同不再生成 `releaseDeliveryDir`。
- expectedOutputs 改为 task evidence / public delivery expectation。

### PR 5：文档与扫描收口

范围：

- `README.md`
- `docs/requirements/034-agentflow-task-workflow-yaml-runtime-v1.md`
- 当前 active architecture docs

要求：

- active 文档不再描述 output/release。
- 历史需求保留，但需要明确“历史设计，不作为新架构依据”。

---

## 9. 验收标准

- [ ] 新运行时不写 `.agentflow/output/release/**`。
- [ ] 新运行时不读 `.agentflow/output/release/**`。
- [ ] Desktop Browser Preview 不包含 `releaseDeliveries`。
- [ ] Desktop 任务页可展示 PR/MR、证据、merge proof、Done 写回。
- [ ] issue 默认合同不包含 `releaseDeliveryDir`。
- [ ] `prepare_release_delivery` 不再是 active API。
- [ ] `OutputReleaseDelivery` 不再被 active crate 依赖。
- [ ] audit gate 不要求 output release delivery。
- [ ] 历史文档中的旧路径不会被当作新需求入口。

最终扫描目标：

```bash
rg "\.agentflow/output/release|releaseDeliveries|prepare_release_delivery|OutputReleaseDelivery"
```

允许命中：

- `docs/requirements/0xx-*` 历史需求里的原文。
- 本需求文档中作为待删除目标的描述。

不允许命中：

- active Rust runtime。
- Desktop task page。
- Browser Preview mock。
- new architecture docs。

---

## 10. 验证命令

每个实现 PR 至少运行：

```bash
cargo check --workspace
cargo test --workspace
npm --prefix apps/desktop run build
git diff --check
```

涉及 Desktop 任务页或 Browser Preview 的 PR 还需要运行：

```bash
npm --prefix apps/desktop run build
```

并通过 Browser Preview smoke：

```text
打开任务页
选择一个 done issue
确认右侧显示状态流、证据、PR/MR、Done 写回
确认页面不显示 output release delivery
```

---

## 11. 不做事项

- 不恢复旧 output 页面为主要业务入口。
- 不新增 delivery 目录。
- 不把交付说明藏在 `.agentflow` 里作为唯一记录。
- 不让审计流程自动触发。
- 不让 GitHub/GitLab issue 成为任务源。
- 不在本地生成公开 release notes 后不提交。

---

## 12. 开发顺序

先顺序做：

```text
Desktop mock / UI 数据源
  -> execute release delivery API
  -> output crate 退场
  -> spec 模板清理
  -> 文档扫描收口
```

不要一口气删除所有 `output` 相关代码。每个 PR 都必须保持：

```text
可编译
可测试
任务页可打开
Browser Preview 可核对
```
