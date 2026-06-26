# AgentFlow v1.0 Filesystem Contract Freeze V1

日期：2026-06-25
执行者：Codex

```yaml
filesystemContractVersion: agentflow-filesystem-contract-freeze.v1
filesystemContractStatus: active
stableContractBaseline: agentflow-stable-contract-baseline.v1
authority: docs/core/architecture/043-v100-agentflow-filesystem-contract-freeze-v1.md
```

## Purpose

本文档冻结 AgentFlow `v1.0.0` 的 `.agentflow/` 文件系统合同。

它回答四个问题：

```text
哪些路径是 authority。
哪些路径是 projection。
哪些路径只是 local cache / transport / tmp。
哪些 retired path 永远不能被重新写入。
```

本合同引用 [041-v100-stable-contract-baseline-v1.md](041-v100-stable-contract-baseline-v1.md) 中的 Stable Public Contracts、Deprecation Rule 和 Version Field Rule。

## Contract Boundary

`.agentflow/` 是本地 Project Runtime 控制平面。

它不是 GitHub issue 的镜像，也不是外部 provider session 的缓存目录。

稳定原则：

- authority path 必须可读、可审计、可回放；
- projection path 永远只读；
- local cache / tmp 不能成为事实源；
- release source archive 不包含本地 runtime state；
- retired path 一旦退休，后续版本不能重新写入。

## Stable Path Contract

| Path | Owner | Read Rule | Write Rule | Authority Level | Version Rule |
| --- | --- | --- | --- | --- | --- |
| `.agentflow/project/**` | Project Runtime | Desktop / CLI / Projection 可读 | 只能由 Project Runtime 写入 | authority | 文件必须带 schema/version 或由上级 manifest 声明 |
| `.agentflow/spec/requirements/<requirement-id>/**` | Spec Agent / Spec Runtime | Runtime / Projection 可读 | 只能由 Spec materialization 写入 | authority | requirement manifest 必须声明版本 |
| `.agentflow/spec/projects/<project-id>.json` | Spec Agent / Project Runtime | Runtime / Projection 可读 | 只能由 Spec materialization 写入 | authority | contract version 必须稳定 |
| `.agentflow/spec/issues/<issue-id>.json` | Spec Agent / Project Runtime | Runtime / Work Agent / Projection 可读 | 只能由 Spec materialization 或 Runtime 状态写回写入 | authority | issue schema version 必须稳定 |
| `.agentflow/spec/completions/<project-id>.json` | Completion Runtime | Runtime / Release gate 可读 | 只能由 Completion Commit 写入 | authority | completion schema version 必须稳定 |
| `.agentflow/runtime/commands/<command-id>.json` | Runtime API | Runtime / Release gate 可读 | 只能由 Runtime API command admission 写入 | authority | command contract version 必须稳定 |
| `.agentflow/runtime/proposals/<proposal-id>.json` | Runtime API | Runtime / Arbitration / Projection 可读 | 只能由 accepted command path 写入 | authority | proposal schema version 必须稳定 |
| `.agentflow/runtime/decisions/<proposal-id>.json` | Arbitration Runtime | Runtime / Projection 可读 | 只能由 arbitration / governance decision 写入 | authority | decision schema version 必须稳定 |
| `.agentflow/runtime/actions/<accepted-action-id>.json` | Runtime API | Runtime / Work Agent 可读 | 只能由 accepted decision path 写入 | authority | action schema version 必须稳定 |
| `.agentflow/packs/<pack-id>/**` | Pack Authoring / Pack Registry | Runtime / Projection / Release gate 可读 | 只能由 Pack authoring 写入；Runtime 不能静默改写 | definition | pack manifest version 必须稳定 |
| `.agentflow/tasks/<issue-id>/work-loop-contract.json` | Work Loop Runtime | Work Agent / Projection 可读 | 只能由 Work Loop Runtime 写入 | derived | derived artifact 必须引用 source issue version |
| `.agentflow/tasks/<issue-id>/runs/<run-id>/run.json` | Work Loop Runtime | Work Agent / Projection / Release gate 可读 | 只能由 Work Loop Runtime 写入 | derived | run schema version 必须稳定 |
| `.agentflow/tasks/<issue-id>/runs/<run-id>/preflight/preflight.json` | Work Loop Runtime | Work Agent / Projection 可读 | 只能由 Work Loop Runtime 写入 | derived | preflight schema version 必须稳定 |
| `.agentflow/tasks/<issue-id>/runs/<run-id>/launch/**` | Work Loop Runtime / Executor Adapter | Work Agent / Provider adapter 可读 | 只能由 launch preparation 写入 | transport | 必须引用 issue、run 和 provider 版本 |
| `.agentflow/tasks/<issue-id>/runs/<run-id>/commands/**` | Work Loop Runtime | Runtime / Projection 可读 | 只能由 Runtime command recorder 写入 | derived | command record version 必须稳定 |
| `.agentflow/tasks/<issue-id>/runs/<run-id>/checkpoints/**` | Work Loop Runtime | Runtime / Resume 可读 | 只能由 Runtime checkpoint recorder 写入 | derived | checkpoint version 必须稳定 |
| `.agentflow/tasks/<issue-id>/runs/<run-id>/review/**` | Work Loop Runtime | Runtime / Projection / Release gate 可读 | 只能由 prepare-review / closeout 写入 | derived | review artifact version 必须稳定 |
| `.agentflow/tasks/<issue-id>/evidence/**` | Work Loop Runtime | Runtime / Projection / Release gate 可读 | 只能由 validation / evidence recorder 写入 | authority | evidence schema version 必须稳定 |
| `.agentflow/events/**` | Event Store | Projection / Replay / Release gate 可读 | 只能由 Runtime append-only event writer 写入 | authority | event envelope version 必须稳定 |
| `.agentflow/projections/**` | Projection Runtime | Desktop / CLI / Query API 可读 | 只能由 Projection rebuild 写入 | projection | projection version 必须稳定 |
| `.agentflow/indexes/**` | Projection Runtime | Desktop / CLI / Query API 可读 | 只能由 index rebuild 写入 | projection | index version 必须稳定 |
| `.agentflow/release/**` | Release Runtime | Release gate / External review 可读 | 只能由 release prepare / publish / proof writer 写入 | authority | release fact version 必须稳定 |
| `.agentflow/audit/**` | Audit Runtime | Audit UI / Release gate 可读 | 只能由 audit sidecar 写入 | sidecar authority | audit schema version 必须稳定 |
| `.agentflow/tmp/**` | Local Runtime | Runtime 内部可读 | 只能写本地临时文件，不能被 projection 或 release gate 当作事实源 | local cache | 不承诺兼容 |

## Authority Classes

### Authority

Authority path 是项目事实源。

它可以驱动 Runtime、Projection、Release gate 和外部审计。

Authority path 包括：

- `.agentflow/project/**`
- `.agentflow/spec/requirements/<requirement-id>/**`
- `.agentflow/spec/projects/<project-id>.json`
- `.agentflow/spec/issues/<issue-id>.json`
- `.agentflow/spec/completions/<project-id>.json`
- `.agentflow/runtime/commands/<command-id>.json`
- `.agentflow/runtime/proposals/<proposal-id>.json`
- `.agentflow/runtime/decisions/<proposal-id>.json`
- `.agentflow/runtime/actions/<accepted-action-id>.json`
- `.agentflow/tasks/<issue-id>/evidence/**`
- `.agentflow/events/**`
- `.agentflow/release/**`

### Definition

Definition path 描述能力、领域和入口，但不能直接推进 Runtime。

Definition path 包括：

- `.agentflow/packs/<pack-id>/**`

Pack 只能通过 Runtime API / Command Surface 进入执行面。

### Derived / Transport

Derived / transport path 是运行过程记录，不替代 authority。

包括：

- `.agentflow/tasks/<issue-id>/work-loop-contract.json`
- `.agentflow/tasks/<issue-id>/runs/<run-id>/**`

如果 derived artifact 和 spec issue 冲突，必须以 spec issue 为准。

### Projection

Projection path 是只读读模型。

包括：

- `.agentflow/projections/**`
- `.agentflow/indexes/**`

Projection 不能写 authority，不能被 Desktop / CLI 当成事实源修改。

### Sidecar Authority

Audit 是独立 sidecar。

`.agentflow/audit/**` 可以拥有审计事实，但不自动改变主业务 Done 结论。

只有 release policy 明确绑定 audit 时，audit 才能影响 release gate。

### Local Cache

`.agentflow/tmp/**` 只允许存放本地临时文件。

规则：

- 不能进入 release source archive；
- 不能进入 public delivery；
- 不能被 projection 当成事实；
- 不承诺兼容。

## Public Record Boundary

公开交付记录不在 `.agentflow/` 内。

公开记录包括：

- `docs/requirements/**`
- PR / MR body
- `CHANGELOG.md`
- release notes
- external review document

`.agentflow/tasks/<issue-id>/evidence/**` 是本地验证事实出口。

公开交付由 PR / MR body、`CHANGELOG.md`、release notes 和 external review 承担。

## Release Source Archive Boundary

Release source archive 必须包含可追溯的 Agent entry 和公开文档。

必须包含：

- `AGENTS.md` 或等价 tracked Agent entry；
- `docs/core/architecture/**`；
- `docs/core/**`；
- `docs/product/**`；
- `docs/requirements/**`；
- `docs/v*/**`；
- `CHANGELOG.md`；
- source code。

不能包含：

- `.agentflow/tasks/**` runtime state；
- `.agentflow/events/**` local event stream；
- `.agentflow/projections/**` read model cache；
- `.agentflow/tmp/**`；
- provider session cache；
- machine-local generated artifacts。

Release gate 必须证明 source archive 的 Agent entry 自洽，同时不能把本地 runtime state 当作 source truth。

## Retired Paths

以下路径已退休，后续版本不能重新写入：

| Retired Path | Replacement |
| --- | --- |
| `.agentflow/input/**` | `.agentflow/spec/requirements/**`、`.agentflow/spec/projects/**`、`.agentflow/spec/issues/**` |
| `.agentflow/execute/**` | `.agentflow/tasks/<issue-id>/runs/**`、`.agentflow/runtime/**` |
| `.agentflow/output/**` | `.agentflow/tasks/<issue-id>/evidence/**`、`.agentflow/release/**`、公开交付记录 |
| `.agentflow/goal-tree/**` | `.agentflow/project/**`、`.agentflow/spec/projects/**` |
| `.agentflow/define/goals/**` | `.agentflow/project/**` |
| `.agentflow/define/milestones/**` | `.agentflow/spec/projects/**` |
| `.agentflow/define/issues/**` | `.agentflow/spec/issues/**` |

如果 release gate 检测到这些路径被重新创建，必须失败。

## Runtime Write Rules

Runtime 写入规则：

1. Spec materialization 只能写 `.agentflow/spec/**`。
2. Runtime API command admission 只能写 `.agentflow/runtime/commands/**`。
3. Proposal / decision / action 只能由 Runtime / Governance / Arbitration 写入。
4. Work Loop 只能在当前 issue 的 `.agentflow/tasks/<issue-id>/**` 下写 run、checkpoint、evidence。
5. Event Store 只能 append `.agentflow/events/**`，不能改写历史事件。
6. Projection 只能写 `.agentflow/projections/**` 和 `.agentflow/indexes/**`。
7. Release Runtime 只能写 `.agentflow/release/**` 和公开交付记录。
8. Audit Runtime 只能写 `.agentflow/audit/**`。
9. Local tmp 只能写 `.agentflow/tmp/**`，不能被当成 authority。

## Version Rule

稳定路径必须满足：

- authority 文件有可机器读取的 version / schemaVersion / contractVersion；
- derived artifact 必须引用 source issue / project / command；
- projection 必须包含 projection version；
- event 必须包含 event envelope version；
- release gate artifact 必须包含 gate artifact version；
- local tmp 不承诺兼容。

## Release Gate Fixture

release gate 必须生成：

```text
runtime/filesystem-contract.json
```

该 fixture 至少证明：

- 本文档存在；
- `filesystemContractVersion = agentflow-filesystem-contract-freeze.v1`；
- `filesystemContractStatus = active`；
- `stableContractBaseline = agentflow-stable-contract-baseline.v1`；
- required stable paths 已列出；
- required authority classes 已列出；
- retired paths 未被重新写入；
- release source archive 与 local runtime state 边界清楚；
- source archive 不把 `.agentflow/**` runtime state 当作 source truth。

## V100 Binding

本合同绑定后续 v1 任务：

| Issue | Required usage |
| --- | --- |
| V100-004 | Pack contract 必须遵守 `.agentflow/packs/**` definition 边界 |
| V100-005 | Projection / Read Model 必须遵守 `.agentflow/projections/**` 和 `.agentflow/indexes/**` 只读边界 |
| V100-006 | Evidence / Acceptance 必须遵守 `.agentflow/tasks/<issue-id>/evidence/**` 和 public record boundary |
| V100-008 | Replay / Migration / Upgrade 必须证明 retired path 没有被重新写入 |
| V100-010 | Release certification 必须包含 filesystem contract fixture |

## Non-goals

- 不把所有本地运行事实提交到 git；
- 不恢复 retired `.agentflow/input/**`；
- 不恢复 retired `.agentflow/execute/**`；
- 不恢复 retired `.agentflow/output/**`；
- 不恢复 retired `.agentflow/goal-tree/**`；
- 不把 provider session 当成项目 authority；
- 不把 Desktop projection 变成写入口。
