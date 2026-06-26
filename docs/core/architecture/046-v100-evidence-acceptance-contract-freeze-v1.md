# V100 Evidence / Acceptance Contract Freeze V1

创建日期：2026-06-25  
执行者：Codex

evidenceAcceptanceContractVersion: agentflow-evidence-acceptance-contract.v1  
evidenceAcceptanceContractStatus: active  
stableContractBaseline: agentflow-stable-contract-baseline.v1  
runtimeApiSdkVersion: agentflow-runtime-api-sdk-freeze.v1  
filesystemContractVersion: agentflow-filesystem-contract-freeze.v1  
packContractVersion: agentflow-pack-contract-freeze.v1  
projectionContractVersion: agentflow-projection-readmodel-contract.v1  
authority: docs/core/architecture/046-v100-evidence-acceptance-contract-freeze-v1.md

## Purpose

本文件冻结 v1.0.0 的 Evidence / Acceptance / Completion Commit 合同。

核心判断：

```text
Evidence Pack proves work.
Acceptance Gate decides Done.
Completion Commit writes Done.
Audit sidecar reviews after the main chain and does not own Done.
```

## References

- [041-v100-stable-contract-baseline-v1.md](041-v100-stable-contract-baseline-v1.md)
- [042-v100-runtime-api-sdk-freeze-v1.md](042-v100-runtime-api-sdk-freeze-v1.md)
- [043-v100-agentflow-filesystem-contract-freeze-v1.md](043-v100-agentflow-filesystem-contract-freeze-v1.md)
- [044-v100-pack-contract-freeze-v1.md](044-v100-pack-contract-freeze-v1.md)
- [045-v100-projection-readmodel-contract-freeze-v1.md](045-v100-projection-readmodel-contract-freeze-v1.md)
- [010-work-loop-filesystem-contract-v1.md](../../archive/2026-06-current-baseline-history/architecture/010-work-loop-filesystem-contract-v1.md)

## Acceptance Authority Boundary

v1.0.0 冻结以下主链：

```text
Confirmed Work
-> Admission
-> Execution
-> Verification
-> Evidence Pack
-> Acceptance Gate
-> Completion Commit
-> Done
```

权威边界：

| Object | Authority |
| --- | --- |
| Verification result | `.agentflow/tasks/<issue-id>/runs/<run-id>/validation.json` |
| Evidence Pack | `.agentflow/tasks/<issue-id>/evidence/evidence.json` |
| Acceptance Gate | `.agentflow/tasks/<issue-id>/acceptance-gate.json` |
| Closeout proof | `.agentflow/tasks/<issue-id>/runs/<run-id>/review/closeout-proof.json` |
| Completion Commit | `issue.completion.committed` event and completion commit artifact |
| Done status | Derived from passed Acceptance Decision and Completion Commit |
| Delivery read model | Projection / public record only, not Done authority |
| Audit result | Independent Audit sidecar, not Done authority |

`Done` 不得由 UI 按钮、Projection rebuild、Audit result、Delivery record 或 executor session 直接写入。

## Evidence Pack Contract

Evidence Pack 是 Work Agent 对当前任务完成情况的验证证据包。

稳定字段：

```text
version
issueId
runId
status
summary
runPath
commandPaths
validationPath
changedFilesPath
validationCommandHash
validationOutputHash
patchSha256
fileContentSha256
treeSha
baseCommit
headCommit
entries
createdAt
```

规则：

- `status` 只能用 `passed`、`failed`、`missing`；
- required evidence entry 不能缺失；
- `verificationLog` 和 `implementationSummary` 是必需证据；
- 可选 screenshot 缺失不能单独阻断 Done，除非 Pack evidence policy 明确要求；
- Evidence Pack 必须能追溯到 run、validation command、changed files 和 commit hash；
- Evidence Pack 不写 Done，只提供 Acceptance Gate 的输入。

## Acceptance Gate Contract

Acceptance Gate 汇总四个子门：

```text
Verification Gate
Evidence Gate
Contract Gate
State Gate
```

### Verification Gate

判断验证命令是否已经运行并通过。

必须读取：

```text
.agentflow/tasks/<issue-id>/runs/<run-id>/validation.json
.agentflow/tasks/<issue-id>/runs/<run-id>/commands/*.json
```

### Evidence Gate

判断 Evidence Pack 是否存在、可读、完整。

必须读取：

```text
.agentflow/tasks/<issue-id>/evidence/evidence.json
```

### Contract Gate

判断任务合同、Pack evidence policy、allowed paths、expected outputs 是否满足。

必须读取：

```text
.agentflow/spec/issues/<issue-id>.json
.agentflow/packs/**
```

### State Gate

判断 issue / run 是否处在允许完成的状态。

允许进入 Acceptance Gate 的前置状态：

```text
in_review
```

不允许从以下状态直接进入 Done：

```text
backlog
todo
in_progress
blocked
cancel
```

## Completion Commit Contract

Completion Commit 是唯一完成写入边界。
英文合同名固定为 `completion write boundary`。

Completion Commit 必须发生在：

```text
Acceptance Decision = passed
Closeout proof = merged / issue closed
Evidence Pack = passed
```

Completion Commit 必须写入：

```text
issue.completion.committed
issue.completed
```

Completion Commit 必须包含：

```text
issueId
runId
acceptanceDecisionPath
evidencePath
validationPath
closeoutProofPath
completionEventId
completedAt
```

如果 Acceptance Decision 失败，Completion Commit 不得发生。

## Failure Reason Contract

failed acceptance 必须输出稳定 reason。

稳定 reason code：

```text
verification-failed
evidence-missing
evidence-incomplete
contract-not-satisfied
state-blocked
closeout-proof-missing
completion-commit-rejected
```

每个 failure reason 必须包含：

```text
code
message
evidencePath
repairHint
blocking
```

release gate 必须覆盖：

```text
pass
fail
missing evidence
state blocked
```

## Status Writeback Contract

状态写回规则：

| Source event | Issue status | Projection status |
| --- | --- | --- |
| `issue.scheduled` | `todo` | `todo` |
| `agent.launch.requested` | `in_progress` | `in_progress` |
| `issue.validation.passed` | `in_review` | `in_review` |
| `issue.acceptance.accepted` | no direct Done write | acceptance ready |
| `issue.completion.committed` | `done` | `done` |
| `issue.completed` | `done` | `done` |
| `issue.audit.evaluated` | no Done authority | audit sidecar projection |

Projection 可以展示状态，但不得写状态。

## Delivery Record Contract

Delivery Record 是公开交付记录，不是 Done authority。

Delivery Record 可以来自：

```text
PR/MR body
CHANGELOG.md
docs/release-notes/**
.agentflow/release/**
```

规则：

- task Done 可以早于 project release publish；
- project release 需要 public delivery record；
- delivery read model 可展示 missing public records；
- delivery missing 不得绕过 Acceptance Gate；
- delivery ready 不得直接写 Done。

## Audit Sidecar Rule

Audit sidecar 只做独立复查。

规则：

- Audit 不属于 Acceptance Gate；
- Audit failed 不回滚已经通过 Acceptance Gate 和 Completion Commit 的任务 Done；
- Audit 可以让 project release readiness 进入 blocked-audit；
- Audit 可以产生后续 repair issue；
- Audit 不得成为默认 Done 阻断项。

## Release Gate Fixture

release gate 必须生成：

```text
runtime/evidence-acceptance-contract.json
```

该 fixture 必须检查：

- 本文件 metadata；
- required sections；
- Evidence Pack status；
- Acceptance Gate event；
- Completion Commit event；
- task projection Done 状态；
- closeout proof merged / issue closed；
- project delivery read model ready；
- Audit sidecar 非 Done authority；
- pass / fail / missing evidence / state blocked fixture。

## V100 Binding

下游任务必须引用本文件：

- `V100-007 Executor Adapter Stable Contract`
- `V100-008 Replay / Migration / Upgrade Certification`
- `V100-009 Software Dev Pack Stable Baseline`
- `V100-010 v1.0.0 Release Certification`

## Non-goals

- 不把 Audit Agent 放回默认完成主链；
- 不把 CI 当成唯一验证 authority；
- 不要求每个任务都有截图证据；
- 不让 Projection、Delivery、executor session 或外部 issue tracker 写 Done；
- 不把人工 review 当成唯一验收依据。
