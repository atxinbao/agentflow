# Runtime Governance Policy V1

创建日期：2026-06-25
执行者：Codex

## 1. 目标

Runtime Governance Policy 是 Runtime admission 前的只读治理层。

它统一回答一件事：

```text
当前角色是否可以让当前 worker / connector / provider 执行这个动作？
```

它不负责执行动作，也不写 authority。

## 2. 输入

Governance Policy 读取这些事实：

- Role Policy：角色能不能执行这个 action；
- Action Policy：action 是否匹配 object type；
- Capability Registry：worker / connector / provider 是否提供所需能力；
- Provider Smoke：provider smoke 失败是否让能力不可用；
- Audit Sidecar Boundary：audit 是否仍然独立，不被绑定回主链。

## 3. 输出

Governance Policy 输出三种决策：

| decision | 含义 |
| --- | --- |
| `allowed` | 允许进入 Runtime admission |
| `rejected` | 明确拒绝，当前请求不应继续执行 |
| `deferred` | 暂缓，通常是 provider 未检查、未认证或需要外部状态补齐 |

每个报告必须包含：

- role policy decision；
- capability policy decision；
- audit sidecar policy decision；
- trace evidence；
- runtimeAdmission 布尔值。

## 4. 边界

Governance Policy 不做这些事：

- 不 append event；
- 不写 `.agentflow/spec/**`；
- 不写 task authority；
- 不 launch provider；
- 不改 audit sidecar 状态；
- 不绕过 Acceptance Gate。

## 5. Audit Sidecar 规则

Audit sidecar 只能保持独立。

允许：

```text
audit-sidecar-mode = independent
audit-sidecar-mode = not-requested
```

拒绝：

```text
audit-sidecar-mode = bound-to-main-chain
```

原因：

```text
Audit 是独立验收 / 审计流程，不允许被 Work / Runtime 主链重新绑定成阻断主链的执行步骤。
```

## 6. Release Gate 验收

Release gate 必须生成：

```text
runtime/governance-policy.json
```

该文件必须证明：

- 至少一个 `allowed`；
- 至少一个 `deferred`；
- 至少一个 `rejected`；
- 每个 decision 有 trace；
- disabled capability 或 provider smoke failure 会影响 runtime admission；
- audit sidecar 绑定回主链会被拒绝。

## 7. 对应代码

- `crates/governance-policy`
- `crates/role-policy`
- `crates/capability-registry`
- `scripts/verify_release_gate.sh`
