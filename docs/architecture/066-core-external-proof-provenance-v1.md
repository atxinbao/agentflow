# 066 - Core External Proof Provenance V1

日期：2026-06-29
执行者：Codex

## 1. 目标

External Proof Provenance 定义外部 proof 的 receipt 和验证规则。

外部链接、远端运行记录、版本地址、远程 artifact 都只能作为带 provenance 的 reference，不能被直接当成事实真相。

它只回答：

```text
这个外部 proof 是谁观测的？
观测到的 commit / tag / version 是什么？
观测时间是什么？
是否带有 digest？
它是否与当前期望一致？
```

## 2. 权威来源

```text
crates/ontology/src/evidence.rs
docs/architecture/066-core-external-proof-provenance-v1.md
release-gate runtime/core-external-proof-provenance.json
```

## 3. Receipt Contract

External proof receipt 固定版本：

```text
agentflow-core-external-proof-receipt.v1
```

Receipt 必须包含：

| 字段 | 说明 |
| --- | --- |
| `provider` | 外部 provider，不限定 GitHub |
| `url` | 外部 proof URL |
| `externalId` | 外部系统 id |
| `proofKind` | proof 类型 |
| `observedCommit` | 观测到的 commit |
| `observedTag` | 观测到的 tag |
| `observedVersion` | 观测到的版本 |
| `observedAt` | 观测时间 |
| `digest` | 可用时记录 artifact digest |

## 4. Expectation Contract

验证时必须显式传入 expectation：

```text
provider
urlPrefix
expectedCommit
expectedTag
expectedVersion
expectedDigest
```

Receipt 只记录观测事实，expectation 记录当前任务或发布期望。

二者必须匹配后，外部 proof 才能进入后续 evidence completeness / decision 判断。

## 5. Stable Reasons

必须输出稳定机器可读原因：

| 场景 | Stable reason |
| --- | --- |
| wrong tag | `external-proof-tag-mismatch` |
| wrong commit | `external-proof-commit-mismatch` |
| stale URL | `external-proof-url-stale` |
| mismatched artifact digest | `external-proof-digest-mismatch` |
| missing digest | `external-proof-digest-missing` |
| untrusted URL | `external-proof-url-untrusted` |

## 6. Negative Fixtures

`v1.0.6` 必须覆盖四类负向样例：

```text
wrong tag
wrong commit
stale URL
mismatched artifact digest
```

这些样例必须证明：

- 外部 tag 与期望不一致会失败；
- 外部 commit 与期望不一致会失败；
- stale URL 会失败；
- artifact digest 不一致会失败；
- 所有验证都不需要 live network call。

## 7. Provider Boundary

External proof provider 是开放字段。

允许：

```text
generic-git-provider
github
gitlab
local-mirror
artifact-registry
```

不允许把 GitHub 当成唯一 provider。

## 8. 非目标

- 不对每次本地测试强制 live network call。
- 不把 GitHub 作为唯一外部 proof provider。
- 不让外部 URL 直接成为 AgentFlow authority。
- 不替代 Evidence Pack / Completeness Policy / Decision Kernel。

## 9. Release Gate

`v1.0.6` release gate 必须生成：

```text
runtime/core-external-proof-provenance.json
runtime/core-external-proof-provenance-rust-test.log
```

证明内容：

- external proof receipt contract 已定义；
- provider / URL / external id / commit / tag / version / observedAt / digest 字段存在；
- wrong tag / wrong commit / stale URL / mismatched artifact digest 都会失败；
- release gate 小产物包含外部 provenance evidence；
- 不需要 live network call。
