# V100 Executor Adapter Contract Freeze V1

创建日期：2026-06-25  
执行者：Codex

executorAdapterContractVersion: agentflow-executor-adapter-contract.v1  
executorAdapterContractStatus: active  
stableContractBaseline: agentflow-stable-contract-baseline.v1  
runtimeApiSdkVersion: agentflow-runtime-api-sdk-freeze.v1  
filesystemContractVersion: agentflow-filesystem-contract-freeze.v1  
packContractVersion: agentflow-pack-contract-freeze.v1  
projectionContractVersion: agentflow-projection-readmodel-contract.v1  
evidenceAcceptanceContractVersion: agentflow-evidence-acceptance-contract.v1  
authority: docs/architecture/047-v100-executor-adapter-contract-freeze-v1.md

## Purpose

本文件冻结 v1.0.0 的 Executor Adapter 合同。

核心判断：

```text
AgentFlow owns the task contract.
Executor performs the work.
Evidence and Acceptance decide whether work can become Done.
Executor session is not project truth.
```

## References

- [041-v100-stable-contract-baseline-v1.md](041-v100-stable-contract-baseline-v1.md)
- [042-v100-runtime-api-sdk-freeze-v1.md](042-v100-runtime-api-sdk-freeze-v1.md)
- [044-v100-pack-contract-freeze-v1.md](044-v100-pack-contract-freeze-v1.md)
- [045-v100-projection-readmodel-contract-freeze-v1.md](045-v100-projection-readmodel-contract-freeze-v1.md)
- [046-v100-evidence-acceptance-contract-freeze-v1.md](046-v100-evidence-acceptance-contract-freeze-v1.md)
- [mcp-provider-adapter.md](mcp-provider-adapter.md)

## Executor Authority Boundary

AgentFlow 管：

- 当前 issue；
- role；
- allowed surface；
- denied path；
- non-goals；
- acceptance criteria；
- expected outputs；
- evidence policy；
- completion writeback。

Executor 管：

- model call；
- tool call；
- shell / file edit；
- local session；
- context window；
- provider behavior。

Executor 不拥有：

- project authority；
- issue status authority；
- Pack authority；
- Projection authority；
- Evidence / Acceptance / Completion authority。

## Work Handoff Schema

所有 executor 必须接收同一类 AgentFlow task contract。

稳定字段：

```text
version
issueId
role
sourceRequirementId
sourceIssuePath
workflowRef
allowedPaths
deniedPaths
nonGoals
acceptanceCriteria
expectedOutputs
evidencePolicy
completionPolicy
providerHints
```

`providerHints` 只能影响 provider 调用方式，不能改变任务合同。

## Allowed Path / Denied Path Rule

Executor 只能修改 `allowedPaths` 覆盖的路径。

规则：

- `deniedPaths` 优先级高于 `allowedPaths`；
- 未声明路径默认不可写；
- provider 自己生成的临时文件不能进入 project authority；
- out-of-scope diff 必须被 post-run validation 拒绝；
- 被拒绝的 executor result 不得推进 Acceptance Gate。

## Expected Outputs Rule

`expectedOutputs` 是 executor 完成工作的最小输出合同。

稳定输出类型：

```text
changedFiles
validationCommands
validationResults
evidenceRefs
handoffNotes
```

缺少必需 expected output 时，executor result 只能进入 `rejected` 或 `deferred`，不能进入 `accepted`。

## Evidence Return Rule

Executor 返回的执行结果必须被 AgentFlow 归一化成 Evidence Pack 输入。

规则：

- executor log 不是 Evidence Pack；
- executor chat history 不是 Evidence Pack；
- executor session memory 不是 Evidence Pack；
- 只有 AgentFlow runtime 写入的 evidence artifact 可以进入 Acceptance Gate；
- Evidence Pack 仍由 [046-v100-evidence-acceptance-contract-freeze-v1.md](046-v100-evidence-acceptance-contract-freeze-v1.md) 管。

## Diff Boundary Check

post-run validation 必须检查 executor diff。

稳定检查：

```text
changedFiles subset of allowedPaths
changedFiles excludes deniedPaths
no runtime cache as authority
no provider session as project truth
no unexpected public record mutation
```

违反边界时：

```text
executorResult = rejected
stableReason = diff-boundary-violation
writesEvidence = false
writesDone = false
```

## Session Isolation Rule

Executor session 是可观测对象，不是权威对象。

允许进入 Projection 的内容：

- provider name；
- session id；
- launch status；
- terminal status；
- last activity；
- normalized result；
- artifact references。

禁止进入 authority 的内容：

- raw chat history；
- provider memory；
- hidden tool state；
- executor-local task plan；
- executor-only status。

## Executor Result Normalization

Executor result 必须归一化为稳定状态：

```text
accepted
rejected
deferred
failed
```

含义：

| Result | 含义 | 后续 |
| --- | --- | --- |
| accepted | executor 输出满足任务合同 | 进入 Evidence / Acceptance |
| rejected | executor 输出违反任务合同 | 不写 Evidence，不写 Done |
| deferred | executor 未能执行，但不是任务失败 | 保留 reason，等待重试或人工处理 |
| failed | executor 执行失败 | 写 failure reason，不写 Done |

`accepted` 也不能直接写 Done。Done 仍必须通过 Acceptance Gate 和 Completion Commit。

## Provider Adapter Mapping

Codex、Claude Code 和后续 executor 都必须映射到同一 AgentFlow task contract。

映射规则：

| Provider surface | AgentFlow object |
| --- | --- |
| Codex task prompt | Work handoff |
| Claude Code task prompt | Work handoff |
| Provider session | Work loop session read model |
| Provider log | Runtime observation |
| Provider diff | Changed files input |
| Provider completion | Executor result |

provider-specific 字段只能放入 sidecar 或 read model，不能改变 stable task contract。

## Provider Smoke Boundary

Provider smoke 只证明 executor provider 最小可用性。

它可以证明：

- provider command / API 可用；
- launch request 可生成；
- session snapshot 可读取；
- terminal projection 可输出。

它不能替代：

- runtime fixture；
- Evidence Pack；
- Acceptance Gate；
- Completion Commit；
- release certification。

## Release Gate Fixture

release gate 必须覆盖：

- accepted executor result；
- rejected executor result；
- deferred executor result；
- diff boundary violation；
- provider smoke boundary；
- session isolation；
- Evidence / Acceptance handoff。

最小 fixture：

```text
accepted executor result
-> Evidence Pack input
-> Acceptance Gate
-> Completion Commit

rejected executor result
-> no Evidence Pack
-> no Done
-> stable rejection reason

deferred executor result
-> no Done
-> retry / human visible reason
```

## V100 Binding

`V100-007` 完成后：

- Executor Adapter contract 必须有 stable schema；
- Codex / Claude Code handoff 必须能映射到同一 AgentFlow task contract；
- executor 越界修改必须被 post-run validation 拒绝；
- executor session / memory 不得成为 AgentFlow authority；
- release gate 必须生成 `runtime/executor-adapter-contract.json`；
- 下游 `V100-008`、`V100-009`、`V100-010` 必须引用本合同。

## Non-goals

- 不重写 Codex / Claude Code 内部 runtime；
- 不保证所有第三方 executor 行为完全一致；
- 不把 executor 的 chat history 当成项目事实；
- 不让 provider smoke 替代 runtime fixture；
- 不让 executor result 绕过 Evidence / Acceptance / Completion。
