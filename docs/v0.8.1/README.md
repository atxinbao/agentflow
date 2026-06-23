# AgentFlow v0.8.1 Pack System Remediation

日期：2026-06-23
执行者：Codex
状态：Release certification baseline

## Purpose

`v0.8.1` 是 `v0.8.0` 发布后的 Pack System 修复版本。

一句话：

```text
把 v0.8.0 的 Pack baseline 从 built-in demo 拉回文件驱动事实源。
```

`v0.8.1` 不改变 `v0.8.0` 的大方向，也不提前进入 Cloud Runtime。它只修复 Pack System 成为真实文件驱动 baseline 所需的缺口。

当前收口结论：

```text
v0.8.1 = clean Pack System remediation release candidate
```

V081-001 到 V081-007 与 V081-009 已完成；V081-008 负责把这些修复汇总成 release certification。

## Reading Order

1. [AGENTFLOW_V0_8_1_PACK_SYSTEM_REMEDIATION_TASKS_V1.md](AGENTFLOW_V0_8_1_PACK_SYSTEM_REMEDIATION_TASKS_V1.md)
2. [../v0.8.0/README.md](../v0.8.0/README.md)
3. [../v0.8.0/AGENTFLOW_V0_8_0_PACK_SYSTEM_TASKS_V1.md](../v0.8.0/AGENTFLOW_V0_8_0_PACK_SYSTEM_TASKS_V1.md)
4. [../architecture/019-pack-filesystem-contract-v1.md](../architecture/019-pack-filesystem-contract-v1.md)
5. [../architecture/029-pack-release-gate-readiness-v1.md](../architecture/029-pack-release-gate-readiness-v1.md)

## Scope

`v0.8.1` 只处理 Pack System 修复：

- Pack Registry 使用可读取的文件或 fixture 作为事实源；
- Runtime Pack Command Resolver 从 registry 和 Pack 定义解析命令；
- Projection 加载 Pack-specific definitions，不能默认回退成 Software Dev；
- Capability status 受 capability registry 和 provider smoke 结果影响；
- invalid Pack command 只能产生 rejected validation report，不能进入提交路径；
- release summary 中 audit sidecar wording 与 release gate 结论分离；
- release gate 增加 Pack negative fixtures；
- release certification 证明 Pack System 已经从 built-in baseline 变成 file-driven baseline；
- 补入 Project Structural Information Principle，作为后续 Spec Loop / Pack / Projection 的结构化信息原则。

## Non-goals

`v0.8.1` 不包含：

- Cloud Runtime；
- remote Agent fleet；
- Pack marketplace；
- 大规模行业生态；
- 完整生产级 Connector；
- Runtime API / SDK 稳定化；
- Event replay；
- Ontology / Pack migration apply；
- Simulation / Evaluation 正式执行层；
- Message Bus 中心化；
- 把 Audit 放回软件开发主链；
- 形式化实现 arXiv:2601.03220 论文中的全部度量。

## Release Boundary

`v0.8.1` 的 release gate 必须证明：

```text
Pack definition comes from files.
Command resolution comes from registry.
Projection comes from pack-specific definitions.
Capability availability comes from real capability status.
Invalid pack commands stop before submit.
Audit sidecar wording cannot be confused with release failure.
```

这些条件已经进入 release gate 证明链。`v0.8.1` 可以作为 Pack System 的 clean remediation release 发布。
