# 067 - Software Dev Reference Evidence Mapping V1

日期：2026-06-29
执行者：Codex

## 1. 目标

Software Dev Reference Evidence Mapping 证明软件开发场景可以使用 Core Evidence Kernel。

它不是 Core authority。

它只回答：

```text
Software Dev 的常见证据字段如何映射到 Core evidence source type？
这些映射能否通过 Core source type registry？
缺失 Software Dev 证据是否通过 Core policy 失败？
```

## 2. 权威来源

```text
crates/ontology/src/evidence.rs
docs/architecture/067-software-dev-reference-evidence-mapping-v1.md
release-gate runtime/software-dev-reference-evidence-mapping.json
```

## 3. Contract

Software Dev evidence reference mapping 固定版本：

```text
agentflow-software-dev-evidence-reference-mapping.v1
```

Contract 必须包含：

| 字段 | 说明 |
| --- | --- |
| `referenceApp` | 固定为 `software-dev` |
| `authorityBoundary` | 必须声明不是 Core authority |
| `mappings` | Software Dev 字段到 Core source type 的映射 |
| `fixturePacks` | 可通过 Core registry 的 reference fixture |

## 4. Mapping

Software Dev 字段必须映射到 Core source type：

| Software Dev reference field | Core source type |
| --- | --- |
| `diff` | `diff` |
| `test-log` | `log` |
| `build-log` | `command-output` |
| `pr-link` | `external-proof` |
| `release-note` | `artifact` |
| `deployment-proof` | `provenance` |

每条 mapping 必须是：

```text
referenceOnly = true
```

## 5. Fixture Coverage

Reference fixture 必须覆盖一组 Software Dev task/run evidence pack：

```text
diff
test log
build log
PR link
release note
deployment proof
```

这些 fixture 必须全部通过：

```text
validate_core_evidence_pack_source_type
```

## 6. Missing Evidence Boundary

Software Dev 缺失证据不能走 ad hoc 检查。

它必须通过 Core Evidence Completeness Policy 失败：

```text
evidence-required-missing:software-dev-reference-required-evidence
```

## 7. Authority Boundary

允许 Software Dev 文案出现在 reference mapping。

不允许：

- 把 GitHub issues 当作 AgentFlow task authority；
- 把 PR / release / repository patch / test log 变成 Core-only schema；
- 把 Software Dev reference mapping 写进 Core authority。

## 8. 非目标

- 不认证完整 Software Dev closeout。
- 不实现 v1.0.9 的完整 Software Dev Reference App。
- 不新增 GitHub issue authority。

## 9. Release Gate

`v1.0.6` release gate 必须生成：

```text
runtime/software-dev-reference-evidence-mapping.json
runtime/software-dev-reference-evidence-mapping-rust-test.log
```

证明内容：

- Software Dev evidence mapping contract 已定义；
- diff / test log / build log / PR link / release note / deployment proof 都映射到 Core source type；
- fixture packs 通过 Core source type registry；
- 缺失 Software Dev evidence 通过 Core policy 失败；
- 所有 Software Dev 字段都是 reference app mapping，不是 Core authority。
