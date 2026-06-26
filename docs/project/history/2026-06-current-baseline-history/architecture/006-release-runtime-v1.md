# 006 Release Runtime V1

创建日期：2026-06-18
执行者：Codex

## 目的

把 release 从“人工补动作”收口成正式项目阶段。

release runtime 只负责项目级 release：

- 读取 completion 判断结果；
- 读取 project delivery 汇总；
- 推进 canonical delivery workflow；
- 生成 release facts；
- 写出 CHANGELOG 和 release notes。

它不负责单条 task 执行，也不代替 Build Agent。

## Authority

release runtime 的上游 authority 只有两类：

1. `completion decision`
2. `project delivery summary`

release 不能绕过这两层直接开始。

## Gate

release gate 规则：

1. project completion 必须已经 `accepted`
2. project delivery summary 必须存在
3. project delivery summary 不能有 `missingCount > 0`
4. project 下必须至少有 1 条 `done` task 可汇总

否则 release 不能推进，只能停在：

- `pending`
- `blocked`

## Runtime State

release runtime 仍然沿用 canonical delivery workflow 的主状态：

```text
pending
  -> ready
  -> in_progress
  -> published
```

但 `published` 不再表示“本地 public record 已写入”，而是必须建立在远端发布证明之上。

因此 release facts 还要额外暴露一层 `publicationStage`：

```text
pending
public-record-written
tag-created
remote-release-created
published
```

语义如下：

- `pending`：release 还没确认
- `public-record-written`：CHANGELOG / release notes 已写出
- `tag-created`：本地 tag 证明已记录
- `remote-release-created`：远端 GitHub / GitLab Release 证明已记录
- `published`：远端发布证明与本地公开记录已对齐

异常态：

```text
blocked
cancel
returned
```

## Event

release runtime 追加项目级事件到统一 task event store：

- `delivery.ready`
- `delivery.started`
- `delivery.published`

这些事件仍走统一 envelope，但 `flowType = delivery`，`projectId` 必填。

本轮不新增新的 canonical delivery 状态事件；`tag` / `remote release` 证明以 release proof 文件进入 facts。

## Facts

release runtime 输出本地 release facts：

```text
.agentflow/release/projects/<project-id>.json
.agentflow/indexes/releases.json
```

其中 `project release facts` 必须包含：

- `projectId`
- `projectTitle`
- `currentState`
- `publicationStage`
- `gateStatus`
- `gateReason`
- `completionState`
- `completionOutcome`
- `deliveryStatus`
- `publicRecordWrittenAt`
- `changelogPath`
- `releaseNotesPath`
- `entryCount`
- `summaryLine`
- `tagName`
- `tagCommitSha`
- `tagProofPath`
- `remoteProvider`
- `remoteReleaseId`
- `remoteReleaseUrl`
- `remoteReleaseCommitSha`
- `remoteReleaseProofPath`
- `artifactManifestPath`
- `artifactManifestSha256`
- `latestEventId`
- `publishedAt`
- `updatedAt`

proof 文件本地路径：

```text
.agentflow/release/proofs/<project-id>/tag.json
.agentflow/release/proofs/<project-id>/remote-release.json
```

## Public Output

release runtime 负责写两类公开产物：

```text
CHANGELOG.md
docs/release-notes/<project-id>.md
```

公开产物只写 public delivery 内容，不暴露：

- `.agentflow/tasks/**`
- `.agentflow/events/**`
- `.agentflow/projections/**`

## Projection

project projection 需要读取 release facts，并暴露：

- `release.currentState`
- `release.publicationStage`
- `release.gateStatus`
- `release.gateReason`
- `release.completionState`
- `release.completionOutcome`
- `release.deliveryStatus`
- `release.publicRecordWrittenAt`
- `release.changelogPath`
- `release.releaseNotesPath`
- `release.entryCount`
- `release.summaryLine`
- `release.tagName`
- `release.tagCommitSha`
- `release.tagProofPath`
- `release.remoteProvider`
- `release.remoteReleaseId`
- `release.remoteReleaseUrl`
- `release.remoteReleaseCommitSha`
- `release.remoteReleaseProofPath`
- `release.artifactManifestPath`
- `release.artifactManifestSha256`
- `release.publishedAt`
- `release.updatedAt`

project stage 在所有任务完成后，优先显示 release 状态，而不是继续停在 completion-ready。

## 模块边界

`crates/release`

负责：

- release gate
- release facts
- public note generation
- project release index

不负责：

- task 执行
- run / checkpoint / evidence 细节
- audit 事实

`crates/workflow-runtime`

负责：

- canonical delivery workflow 状态推进

`crates/projection`

负责：

- 把 project release facts 投影到项目读模型

## 验证

本轮最小验证：

- `cargo test --workspace`
- `git diff --check`
- `bash scripts/verify_release_gate.sh --artifact-dir artifacts/release-gate-e2e`

正式 GitHub gate：

- `.github/workflows/release-gate.yml`

其中 release gate E2E 会输出：

- `artifacts/release-gate-e2e/certification.json`
- `artifacts/release-gate-e2e/certification.md`
- `CHANGELOG.md`
- `docs/release-notes/<project-id>.md`
- `docs/reviews/<project-id>.md`
- `.agentflow/release/projects/<project-id>.json`
- `.agentflow/release/reviews/<project-id>.json`

其中至少要覆盖：

1. completion 未 accept 时，release 不能启动
2. public delivery 不完整时，release 进入 blocked
3. 仅写 public record 时，release 最多停在 `public-record-written`
4. 缺少 remote release proof 时，`publish` 必须失败
5. Tag proof + remote release proof 都存在时，release 才能进入 `published`
6. CHANGELOG 和 release notes 被写出，且能追溯到 tag / remote release / artifact manifest
