# Pack Migration Execution Model V1

创建日期：2026-06-24
执行者：Codex

## 1. 目标

本文件定义 Pack / Ontology migration 从 preview 进入受控 apply 的运行时边界。

核心规则：

```text
preview 只是影响预览。
apply 必须有明确 confirmation。
cancel 和 rollback 必须有独立 receipt。
migration 后必须能继续 replay / projection rebuild，或给出结构化失败。
```

## 2. 非目标

- 不做隐式自动迁移。
- 不把 preview receipt 当作 applied receipt。
- 不在 migration preview 阶段写 authority。
- 不绕过 confirmation 直接 apply。
- 不把 cancel 当 rollback。

## 3. Migration 阶段

```text
Pack Migration Preview
-> Explicit Confirmation
-> Controlled Apply
-> Applied Receipt
-> Replay / Projection Rebuild Check
```

旁路语义：

```text
Preview -> Cancel Receipt
Applied Receipt -> Rollback Receipt
```

## 4. Receipt 类型

### Preview Receipt

用途：说明 migration 将影响哪些对象和 projection。

要求：

- `writesAuthority = false`
- `requiredHumanConfirmation = true`
- 包含 affected objects / affected projections
- 不能作为 apply 证明

### Applied Receipt

用途：证明 migration 已经在明确 confirmation 后进入 apply。

要求：

- 必须绑定同一个 `previewId`
- 必须有 `confirmed = true`
- 必须有 actor 和 reason
- `writesAuthority = true`
- schema 与 preview receipt 不同

### Cancel Receipt

用途：证明 migration preview 被取消。

要求：

- 只能基于 preview 生成
- `writesAuthority = false`
- 不等同于 rollback

### Rollback Receipt

用途：证明已 apply 的 migration 被回滚。

要求：

- 必须基于 applied receipt
- 必须有 actor 和 reason
- `writesAuthority = true`
- 必须保留 applied receipt version

## 5. CLI Surface

V1 暴露以下命令：

```text
agentflow pack migration-preview
agentflow pack migration-apply
agentflow pack migration-cancel
agentflow pack migration-rollback
```

`migration-apply` 不接受默认确认。

缺少 `--confirmed` 时必须失败，并且不能写 applied receipt。

## 6. Release Gate

release gate 必须验证：

1. migration preview 不写 authority；
2. 未确认 apply 会失败；
3. 确认 apply 生成 applied receipt；
4. cancel receipt 与 applied receipt 明确区分；
5. rollback receipt 与 applied receipt 明确区分；
6. migration receipt 生成后，event replay / projection rebuild 仍然可运行，或产生结构化失败。

## 7. 与 Projection 的关系

Migration 本身不能替代 projection rebuild。

正确关系是：

```text
migration receipt
-> event replay
-> projection rebuild report
```

Projection 仍然是 read model，不写 authority。

## 8. 验收锚点

- `crates/pack` 提供 typed receipt API。
- `crates/cli` 提供 migration CLI。
- `scripts/verify_release_gate.sh` 生成 migration artifacts。
- `summary.json` / `certification.json` 暴露 migration release gate 结果。

