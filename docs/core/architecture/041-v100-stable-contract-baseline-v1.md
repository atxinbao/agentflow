# AgentFlow v1.0 Stable Contract Baseline V1

日期：2026-06-25
执行者：Codex

```yaml
stableContractVersion: agentflow-stable-contract-baseline.v1
stableContractStatus: active
releaseLine: v1.0.0
authority: docs/core/architecture/041-v100-stable-contract-baseline-v1.md
```

## Purpose

本文档定义 AgentFlow `v1.0.0` 之后承诺稳定的 Project OS 合约边界。

目标不是冻结所有内部实现，而是明确：

```text
哪些是外部客户端、Pack、Executor Adapter 和 release gate 可以依赖的 stable contract。
哪些只是 AgentFlow 内部运行时实现细节。
哪些仍然是 experimental surface。
```

后续 `V100-002` 到 `V100-010` 必须引用本 baseline。缺少本 baseline 时，后续 v1.0 issue 不允许进入 Done。

## Stable Public Contracts

以下对象进入 `v1.0.0` 稳定承诺：

| Contract | Stable boundary |
| --- | --- |
| Runtime API / SDK command contract | command input、admission result、accepted / rejected / deferred / failed 语义 |
| Runtime API / SDK query contract | read-only query shape、error code、evidence path |
| Runtime event envelope | event id、event type、object ref、causation、correlation、timestamp、schema version |
| `.agentflow/` filesystem contract | stable authority path、projection path、local cache path、retired path 禁止写入 |
| Spec project / issue contract | `.agentflow/spec/projects/**`、`.agentflow/spec/issues/**` 的 owner、status、dependency、workflowRef、expected outputs |
| Task runtime artifact contract | task event、run checkpoint、evidence、acceptance decision、completion commit |
| Pack contract | Domain Pack、Surface Pack、Connector Pack manifest、capability、migration、readiness 状态 |
| Projection / Read Model contract | projection schema、stale / invalid / deferred 语义、rebuild report |
| Evidence / Acceptance contract | evidence index、acceptance gate、failure reason、Done writeback 条件 |
| Executor Adapter contract | provider capability、launch request、session snapshot、closeout attestation |
| Release certification contract | release source Agent entry、version metadata、deployment evidence、negative semantic fixtures |

这些 contract 的字段可以新增可选字段，但不能破坏已有必填字段、状态语义和 authority 边界。

## Internal Implementation Details

以下内容不进入 v1 稳定承诺：

- Rust crate 内部函数签名；
- 临时 test fixture 目录结构；
- debug-only CLI 输出文本；
- provider 进程启动细节；
- Desktop 组件内部状态变量；
- release gate 临时 workspace 路径；
- 本地运行时缓存；
- 未公开的 helper schema；
- 单次测试生成的 stdout / stderr 文本格式。

这些实现可以在不破坏 stable public contracts 的前提下重构。

## Experimental Contracts

以下能力仍然是 experimental，不能被后续版本当作 stable API 依赖：

- live provider smoke session；
- cross-process Message Bus；
- cloud runtime deployment shape；
- 非 Software Dev Pack 的行业壳；
- Pack marketplace；
- 多租户运行时；
- 非默认 executor provider；
- UI Design Pack 的完整主链执行。

Experimental 能力必须通过 explicit readiness / capability 状态暴露，不允许静默伪装成 stable ready。

## Compatibility Promise

`v1.0.0` 之后，AgentFlow 对 stable public contracts 承诺：

- 必填字段不删除；
- 状态值不重命名；
- authority path 不迁移到未声明位置；
- projection 仍然只读；
- Audit sidecar 不进入主业务链；
- Executor Adapter 不拥有项目 truth；
- release gate 必须继续能复跑 stable contract certification。

如果确实需要破坏性变更，必须进入 breaking-change flow。

## Breaking Change Rule

任何破坏 stable public contract 的变更都必须满足：

1. 新增明确的 replacement contract；
2. 提供 migration preview；
3. 提供 rollback / compatibility 说明；
4. release gate 增加 negative fixture；
5. 文档标记 breaking change；
6. 不允许在 patch release 中执行。

未满足以上条件的破坏性变更必须被 release gate 阻断。

## Deprecation Rule

弃用 stable public contract 时必须：

- 在文档中标记 deprecated；
- 提供 replacement；
- 保留至少一个 minor version 的 read compatibility；
- release certification 输出 deprecation evidence；
- 不允许继续新增对 deprecated contract 的写入。

Retired path 一旦进入 retired 状态，后续版本不能重新写入。

## Version Field Rule

所有 stable public contract 必须包含可校验版本字段。

版本字段规则：

- 顶层 version 必须稳定且可机器读取；
- schema version 与 release version 不混用；
- release gate 必须校验 stable contract baseline metadata；
- `stableContractVersion` 当前固定为 `agentflow-stable-contract-baseline.v1`；
- `stableContractStatus` 当前固定为 `active`。

## Release Certification Rule

release gate 必须在进入 v1.0 release certification 前检查：

- 本文档存在；
- `stableContractVersion = agentflow-stable-contract-baseline.v1`；
- `stableContractStatus = active`；
- stable / internal / experimental 三类边界均已定义；
- compatibility、breaking change、deprecation、version field、release certification 规则均已定义。

如果任一项缺失，`V100-002` 到 `V100-010` 不允许进入 Done。

## V100 Issue Binding

后续任务绑定关系：

| Issue | Required baseline usage |
| --- | --- |
| V100-002 | Runtime API / SDK Freeze 必须引用 Stable Public Contracts 和 Version Field Rule |
| V100-003 | Filesystem Contract Freeze 必须引用 Stable Public Contracts 与 Retired Path 规则 |
| V100-004 | Pack Contract Freeze 必须引用 Pack contract、Compatibility Promise 和 Migration 规则 |
| V100-005 | Projection Contract 必须引用 Projection 只读边界 |
| V100-006 | Evidence / Acceptance Contract 必须引用 Evidence / Acceptance stable boundary |
| V100-007 | Executor Adapter Contract 必须引用 Executor Adapter 不拥有 truth 的规则 |
| V100-008 | Replay / Migration / Upgrade Certification 必须引用 Breaking Change Rule 和 Deprecation Rule |
| V100-009 | Software Dev Pack Stable Baseline 必须引用 Pack contract 与 stable / experimental 边界 |
| V100-010 | v1.0.0 Release Certification 必须引用完整 baseline 与 release gate proof |

## Non-goals

- 不承诺历史草案兼容；
- 不把内部 Rust API 全部变成 stable API；
- 不把 GitHub issue、GitHub PR 或外部 provider session 当成 AgentFlow authority；
- 不把 experimental capability 伪装成 stable contract。
