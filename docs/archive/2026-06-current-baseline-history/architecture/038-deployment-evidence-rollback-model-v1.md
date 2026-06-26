# 038 - Deployment Evidence and Rollback Model V1

创建日期：2026-06-25
执行者：Codex

## 1. 背景

v0.9.0 已经把 Runtime API、Pack、Event Store replay、Projection rebuild、Pack migration、治理策略和调度决策接入 release gate。

但 release artifact 还需要回答一个更直接的问题：

```text
这个版本到底部署成什么形态？
如果部署失败，能不能不靠口头说明回滚？
```

本文件定义部署证据和回滚模型的技术边界。

## 2. 核心原则

- Deployment Evidence 是 release artifact，不是新的 authority。
- Deployment Evidence 只读取已有 runtime / release gate artifact。
- Rollback Model 不绑定 GitHub、GitLab、Vercel、云厂商或单一 provider。
- 回滚必须能追到 release tag、commit、migration rollback receipt 和 failed deployment report。
- Projection 缓存不能替代 replay proof。

## 3. Report

schema version：

```text
agentflow-deployment-evidence-report.v1
```

默认输出：

```text
runtime/deployment-evidence.json
```

核心字段：

```text
releaseVersion
releaseTag
sourceCommitSha
runtimeVersion
localDeployment
cloudDeployment
releaseFacts
configFingerprint
packVersionFingerprint
eventStoreFingerprint
projectionRebuildProof
migrationReceipt
rollbackModel
writesAuthority
missingEvidence
generatedAt
```

`writesAuthority` 必须为 `false`。

## 4. Local / Cloud Shape

Deployment Evidence 不直接执行部署。

它只证明两个 deployment shape 的证据是否齐全：

```text
local:
  release facts
  pack fingerprint
  event replay report
  projection rebuild proof
  migration receipt

cloud:
  remote release proof
  release facts
```

如果证据缺失，report 状态为 `failed`，并写入 `missingEvidence`。

## 5. Rollback Model

Rollback Model 必须 provider-agnostic：

```text
rollbackModel:
  providerAgnostic: true
  targetTag
  targetCommitSha
  rollbackReceipt
  failedDeploymentReport
  requiresHumanConfirmation
```

它不执行回滚，只证明：

- 回滚目标是谁；
- 回滚依据来自哪个 artifact；
- 失败部署报告在哪里；
- 回滚是否需要人工确认。

## 6. Release Gate

release gate 必须生成：

```text
runtime/deployment-evidence.json
```

并纳入：

- `summary.json`
- `summary.md`
- `certification.json`
- `certification.md`
- certification checklist

Checklist ID：

```text
v090-deployment-evidence-rollback
```

## 7. 非目标

不做：

- 不接云厂商 rollback API；
- 不新增远程部署系统；
- 不用 Projection 缓存替代 replay proof；
- 不把 deployment evidence 写成 authority；
- 不把 audit sidecar 变成 release blocking gate。
