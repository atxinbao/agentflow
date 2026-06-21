# AgentFlow v0.6.1 Remediation Tasks V1

日期：2026-06-21
执行者：Codex
状态：Remediation Planning Draft / 不授权 Build Agent 执行

## 1. Purpose

本文档把 `v0.6.0` 审计发现转成 `v0.6.1` 修复任务。

`v0.6.1` 的目标是：

```text
Release Closeout
+ Acceptance Gate Refinement
+ Completion Commit Authority
+ Audit Separation Cleanup
```

## 2. Issue Preview

| Issue | Title | Dependency | Priority | First executable |
| --- | --- | --- | --- | --- |
| `V061-001` | Release Metadata and Version Alignment | 无 | P0 | 是 |
| `V061-002` | v0.6.0 Changelog and Documentation Closeout | `V061-001` | P0 | 否 |
| `V061-003` | Release Gate Version Certification | `V061-001` | P0 | 否 |
| `V061-004` | Acceptance Gate Contract | 无 | P0 | 否 |
| `V061-005` | Acceptance Decision Persistence and Failure Reasons | `V061-004` | P0 | 否 |
| `V061-006` | Completion Commit Authority Boundary | `V061-005` | P0 | 否 |
| `V061-007` | Optional Audit Trigger Evaluation | `V061-006` | P1 | 否 |
| `V061-008` | v0.6.1 Release Audit Certification | `V061-002`, `V061-003`, `V061-006`, `V061-007` | P1 | 否 |

## 3. Development Tasks

### V061-001 - Release Metadata and Version Alignment

目标：

修复 release 版本号漂移。

范围：

- 更新 workspace version；
- 更新 Desktop package version；
- 更新 Tauri version；
- 更新 package lock 中的根版本；
- 检查 release-gate 读取的 version metadata；
- 增加版本一致性检查。

验收标准：

- `Cargo.toml`、`apps/desktop/package.json`、`apps/desktop/package-lock.json`、`apps/desktop/src-tauri/tauri.conf.json` 都指向 `0.6.1`；
- release-gate 能检测 package version 与目标 release version 不一致；
- 版本检查失败时阻止 release gate 通过。

非目标：

- 不改变 Work Loop 功能范围；
- 不发布 tag。

### V061-002 - v0.6.0 Changelog and Documentation Closeout

目标：

把 `v0.6.0` 从 planning draft 收成 released baseline。

范围：

- 在 `CHANGELOG.md` 增加 `0.6.0 - 2026-06-21`；
- 将 `docs/v0.6.0/**` 状态改为 released baseline / release closeout；
- 将 `docs/v0.5.1/**` 状态改为 completed / folded into release path，不能继续阻塞已发布的 `v0.6.0`；
- 更新 `docs/README.md` 入口说明；
- 保留 `v0.6.0` 审计结论。

验收标准：

- 文档不再声称 `v0.6.0` 仍未授权执行；
- `v0.5.1` 不再与 `v0.6.0` release fact 冲突；
- `CHANGELOG.md` 能解释 `v0.6.0` 做了什么、验证了什么、留下什么进入 `v0.6.1`。

非目标：

- 不改源码逻辑；
- 不隐藏 v0.6.0 遗留问题。

### V061-003 - Release Gate Version Certification

目标：

让 release-gate 对版本事实敏感，不再默认停在旧版本。

范围：

- 移除或参数化 `v0.5.1` 默认值；
- 移除 `v0.5.1-e2e` 硬编码 tag / URL fixture；
- 增加 release version、tag name、package versions、CHANGELOG entry、GitHub Release fact 的一致性检查；
- 生成 certification artifact；
- 在 release notes 中引用 certification artifact。

验收标准：

- 未传入 release version 时，脚本不会静默使用旧版本；
- release version 与 metadata 不一致时 gate 失败；
- certification artifact 记录 main gate run、tag gate run、source commit、release URL；
- `v0.6.1` release notes 能指向该 artifact。

非目标：

- 不引入外部 CI 之外的新发布平台；
- 不做云端部署。

### V061-004 - Acceptance Gate Contract

目标：

把 `Evidence Gate` 升级为完整的 `Acceptance Gate`。

范围：

- 定义 `Acceptance Gate`；
- 定义 `Verification Gate`；
- 定义 `Evidence Gate`；
- 定义 `Contract Gate`；
- 定义 `State Gate`；
- 明确四个子 gate 的输入、输出、失败原因和修复建议；
- 更新 Work Loop 文档和代码命名。

验收标准：

- Done 决策不再只写成 evidence gate；
- 验证失败、证据不足、合同未满足、状态非法都能阻止 Done；
- Audit 不在 Acceptance Gate 内执行；
- Acceptance Gate 输出能被 Completion Commit 消费。

非目标：

- 不写 audit report；
- 不自动创建 audit issue。

### V061-005 - Acceptance Decision Persistence and Failure Reasons

目标：

让验收判定成为可追溯事实。

范围：

- 定义 acceptance decision record；
- 记录通过 / 拒绝 / 需要人工判断；
- 记录每个子 gate 的结果；
- 记录失败原因；
- 记录下一步修复建议；
- 接入 event / task evidence。

验收标准：

- 每次 Done 前都有 acceptance decision；
- 被拒绝的 Done 有明确原因；
- acceptance decision 可从 issue / run / session 追溯；
- projection 可以只读展示 acceptance summary。

非目标：

- 不让 projection 成为 authority；
- 不把人工判断伪装成测试通过。

### V061-006 - Completion Commit Authority Boundary

目标：

定义验收通过后的唯一完成写入边界。

范围：

- 定义 `Completion Commit`；
- 要求 `Acceptance Decision passed` 才能进入 commit；
- 将 accepted action 写入 Event Store；
- 写回 issue / run status；
- 刷新 projection；
- 写入 delivery record；
- 明确 projection refresh 是派生视图，不是 authority。

验收标准：

- Completion Commit 不能在 acceptance failed 时发生；
- Event Store 是完成事实入口；
- issue / run status 来自 accepted completion action；
- delivery record 有 evidence 链；
- projection 不可直接驱动 Done。

非目标：

- 不改写历史事件；
- 不做 release 系统大改。

### V061-007 - Optional Audit Trigger Evaluation

目标：

把 Audit 从 Work Loop 主闭环拿出来，只保留 Done 后的独立触发判断。

范围：

- 定义 optional audit trigger evaluation；
- 输入：Done result、delivery record、risk flags、human request、release policy；
- 输出：`audit queue` 或 `no audit`；
- 明确该判断不改变 Done 事实；
- 明确 Audit Agent 仍从独立 audit issue 或明确人类请求进入。

验收标准：

- Done 不自动创建 audit issue；
- no audit 是合法结果；
- audit queue 只是建议或入口，不代表 audit pass；
- Audit Agent 仍然不能修改 Work Loop facts。

非目标：

- 不自动生成审计报告；
- 不把 Audit 重新塞回 Build / Work Loop。

### V061-008 - v0.6.1 Release Audit Certification

目标：

证明 `v0.6.1` 修复了 `v0.6.0` 的发布收口和验收闭环问题。

范围：

- 生成 release audit certification；
- 记录版本元数据；
- 记录 CHANGELOG / docs closeout；
- 记录 release-gate certification；
- 记录 Acceptance Gate 测试；
- 记录 Completion Commit 测试；
- 记录 Audit separation 测试。

验收标准：

- `v0.6.1` 可以被判断为 clean remediation release；
- 审计结论能追溯到具体命令、PR、commit、release gate run；
- 未解决问题必须明确进入后续版本，不隐藏。

非目标：

- 不承诺 v1.0 稳定 API；
- 不引入行业 Pack。

## 4. Suggested Milestones

### Milestone 1 - Release Closeout

包含：

- `V061-001`
- `V061-002`
- `V061-003`

完成后，版本事实、文档状态和 release gate 不再漂移。

### Milestone 2 - Acceptance Gate

包含：

- `V061-004`
- `V061-005`

完成后，验收判定成为 Work Loop 主闭环的一等事实。

### Milestone 3 - Completion and Audit Separation

包含：

- `V061-006`
- `V061-007`

完成后，完成写入有唯一边界，Audit 保持 Done 后独立流程。

### Milestone 4 - Certification

包含：

- `V061-008`

完成后，`v0.6.1` 可以作为 `v0.6.0` 后的 clean remediation release。

## 5. Completion Criteria

`v0.6.1` 完成时，必须满足：

- `v0.6.0` release closeout 文档与 tag / release fact 一致；
- `v0.6.1` 版本元数据一致；
- release-gate 能阻止旧版本默认值漂移；
- Acceptance Gate 成为 Done 决策入口；
- Acceptance Decision 可持久化、可投影；
- Completion Commit 明确 Event Store / status / projection / delivery 顺序；
- Done 不自动触发 Audit；
- release audit certification 可追溯。

## 6. Next Step

下一步不是直接执行这些任务。

正确顺序是：

```text
SPEC Draft Preview
-> Project Preview
-> Issues Preview
-> Human Confirmation
-> docs/requirements/**
-> .agentflow/spec/projects/**
-> .agentflow/spec/issues/**
```

确认前，本文件只是 `v0.6.1` 的修复规划。
