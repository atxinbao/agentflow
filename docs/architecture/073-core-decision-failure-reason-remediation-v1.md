# Core Decision Failure Reason / Remediation Contract V1

更新日期：2026-06-29
执行者：Codex

## Purpose

Core Decision 不能只输出一段人类说明。

当 Decision outcome 是 `rejected`、`deferred`、`blocked` 或 `needs-fix` 时，系统必须留下结构化 failure reason，后续 loop 才能根据稳定字段继续判断下一步。

## Authority Boundary

Failure Reason 属于 Core Decision 的解释合同，只解释非 accepted outcome。

它可以读取：

- Decision outcome；
- authority refs；
- missing evidence refs；
- remediation route；
- retry eligibility。

它不能写：

- runtime state authority；
- evidence authority；
- projection read model；
- provider session；
- audit sidecar。

## Contract Version

```text
agentflow-core-decision-failure-reason.v1
```

## Applies To Outcomes

Failure Reason 只适用于：

```text
rejected
deferred
blocked
needs-fix
```

它不能适用于：

```text
accepted
```

`accepted` 表示 Decision 允许 subject 进入下一 route，不需要 failure reason。

## Required Fields

每条 failure reason 必须包含：

```text
reasonCode
message
authorityRefs
missingEvidenceRefs
remediationRoute
retryEligible
blocking
```

字段含义：

| 字段 | 说明 |
| --- | --- |
| `reasonCode` | 机器可读原因码。 |
| `message` | 给人看的短说明。 |
| `authorityRefs` | 参与本次判断的权威事实引用。 |
| `missingEvidenceRefs` | 缺失或不可用的证据引用。 |
| `remediationRoute` | 后续 loop 可执行的修复路线。 |
| `retryEligible` | 修复后是否允许重新评估 Decision。 |
| `blocking` | 当前原因是否阻断继续推进。 |

## Remediation Routes

Core 定义的稳定 remediation route：

| route | 含义 | retry eligible |
| --- | --- | --- |
| `wait-for-authority` | 等待权威事实更新。 | yes |
| `collect-evidence` | 补齐缺失证据后重新判断。 | yes |
| `revise-subject` | 修改 subject 边界后重新判断。 | yes |
| `cancel-subject` | 终止 subject，不自动重试。 | no |
| `retry-decision` | 权威输入变化后重新运行 Decision。 | yes |

## Validation Rules

1. `accepted` outcome 不能挂 failure reason。
2. `blocked` outcome 的 failure reason 必须 `blocking = true`。
3. `authorityRefs` 不能为空。
4. `missingEvidenceRefs` 不能为空。
5. `remediationRoute` 必须来自合同定义。
6. 如果 route 不允许 retry，则 failure reason 不能声明 `retryEligible = true`。
7. Core 合同内不能出现 Software Dev 行业词汇。

## Release Gate Evidence

Release gate 必须生成：

```text
runtime/core-decision-failure-reason-remediation.json
```

该 artifact 至少证明：

- 合同版本存在；
- Rust contract / validator 存在；
- 四类非 accepted outcome 都被覆盖；
- required fields 被覆盖；
- remediation routes 被覆盖；
- positive fixture 通过；
- negative fixtures 覆盖 accepted outcome、缺 authority refs、缺 evidence refs、未知 route、错误 retry eligibility。

## Non-goals

- 不实现 Agent 自动重试；
- 不定义行业特定 remediation catalog；
- 不把 failure reason 写成自由文本解析协议；
- 不把 Audit 作为默认修复路线；
- 不认证 projection read model。
