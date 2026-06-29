# 069 - Release Provenance Tag Policy V1

日期：2026-06-29
执行者：Codex

## 1. 目标

Release Provenance Tag Policy 定义 `v1.0.x` 发布标签的验证口径，以及 Evidence Kernel 到 Decision Kernel 的交接合同。

它只回答：

```text
当前 release tag 是什么对象？
tag 指向哪个 commit？
tag 签名状态是什么？
未签名 tag 是 warning 还是 blocking？
v1.0.6 Evidence Kernel 产物如何交给 v1.0.7 Decision Kernel？
```

本合同不定义 Decision outcome，不判断 Done，不启动 Audit。

## 2. 权威来源

```text
docs/architecture/069-release-provenance-tag-policy-v1.md
scripts/verify_release_gate.sh
release-gate runtime/v107-release-provenance-handoff.json
```

## 3. Tag Policy

`v1.0.x` 阶段允许两类 tag：

```text
annotated
lightweight
```

签名策略：

| tag 状态 | 处理方式 |
| --- | --- |
| signed annotated tag | 通过 |
| unsigned annotated tag | warning-only-visible |
| lightweight tag | warning-only-visible |
| tag commit mismatch | blocking |
| missing tag commit in release/tag context | blocking |
| literal revspec leaked into provenance | blocking |
| release URL not bound to tag | blocking |

重要约束：

```text
unsigned tag 允许 warning-only，但不能静默忽略。
```

所有 warning 必须结构化写入 release provenance artifact，并进入 release certification summary。

## 4. Required Structured Fields

Release provenance artifact 必须暴露：

| 字段 | 说明 |
| --- | --- |
| `tagName` | release tag |
| `tagObjectKind` | `tag` / `commit` / `pending` |
| `annotatedTagObjectId` | annotated tag object id，可为空 |
| `tagCommitSha` | tag 指向的 commit |
| `sourceCommitSha` | release gate 当前源码 commit |
| `tagCommitMatchesSource` | tag commit 是否匹配源码 commit |
| `tagSignatureStatus` | `verified` / `unsigned` |
| `unsignedReason` | 未签名原因，不能为空白吞掉 |
| `artifactManifestSha256` | artifact manifest digest |
| `artifactHashes` | certification / summary / stage log digest |
| `gateRunIds` | GitHub Actions run id / attempt |

## 5. v1.0.6 Evidence Kernel Handoff

`v1.0.7` Decision Kernel 只能在 `v1.0.6` Evidence Kernel 已完成后继续。

必须交接的 release-gate runtime artifacts：

```text
runtime/core-evidence-pack-schema.json
runtime/core-evidence-source-type-registry.json
runtime/core-evidence-capture-receipts.json
runtime/core-evidence-authority-trace-links.json
runtime/core-evidence-completeness-policy.json
runtime/core-missing-evidence-handling.json
runtime/core-external-proof-provenance.json
runtime/software-dev-reference-evidence-mapping.json
runtime/evidence-projection-read-model.json
runtime/v106-release-certification.json
```

交接规则：

- `v106-release-certification` 必须是 `passed`；
- 每个 required artifact 必须在 `certifiedArtifacts` 中为 `passed`；
- 每个 certified artifact 必须有 `sha256` 和 `bytes`；
- `eventEvidence.releaseTagName` 必须存在；
- 交接产物只能作为 Decision Kernel 输入证据，不能直接写 Decision outcome。

## 6. Release Gate Artifact

`v1.0.7` 前置 gate 必须生成：

```text
runtime/v107-release-provenance-handoff.json
```

该产物必须包含：

- tag policy；
- observed release provenance；
- v1.0.6 Evidence Kernel handoff artifact list；
- certified artifact digest list；
- warning-only unsigned tag behavior；
- blocking failure list；
- coverage map；
- `failedCoverage`。

## 7. 非目标

- 不要求本项目立刻启用完整 supply-chain signing；
- 不把 unsigned tag 直接当作 blocking；
- 不启动 Decision outcome / completion decision；
- 不把 GitHub issue 当成 AgentFlow authority；
- 不把 Audit 移入主业务链。

