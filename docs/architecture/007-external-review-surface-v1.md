# 007 External Review Surface V1

创建日期：2026-06-18  
执行者：Codex

## 目的

让外部 reviewer / 审计者不需要进入 `.agentflow` 本地运行目录，也能直接读懂一次项目交付。

这层 surface 不是新门户，只是统一的对外交付阅读面。

## Surface 组成

项目级 external review surface 由四部分组成：

1. external review summary
2. audit summary surface
3. evidence index surface
4. review handoff package

## 输入事实

external review surface 只读取现有 authority：

- `SpecProject`
- `ProjectReleaseFacts`
- `PublicReleaseSummary`
- `ProjectAuditReviewSummary`

它不直接改变 task / run / audit 状态。

## 本地结构化事实

```text
.agentflow/release/reviews/<project-id>.json
.agentflow/indexes/external-reviews.json
```

结构化 summary 至少要包含：

- `projectId`
- `projectTitle`
- `sourceRequirementId`
- `sourceRequirementPath`
- `objective`
- `reviewStatus`
- `releaseState`
- `releaseSummaryLine`
- `handoffPath`
- `totalEntries`
- `evidenceEntries`
- `auditSummary`
- `riskItems`
- `summaryLine`
- `generatedAt`

## 对外阅读面

对外 handoff package 写到：

```text
docs/reviews/<project-id>.md
```

这份文档必须能直接回答：

- 这次交付解决什么问题
- 交付包含哪些任务
- 每条任务的验证和 review proof 是什么
- 当前 release 是否已经正式发布
- 有没有审计结论
- 是否还有风险需要 reviewer 留意

## Evidence Index Surface

evidence index 不是把 `.agentflow/tasks/**` 整个暴露出去。

它只整理 reviewer 真正需要的字段：

- `issueId`
- `title`
- `summary`
- `evidenceStatus`
- `validationCommandCount`
- `publicRecordTargets`
- `prUrl`
- `mergeCommit`

内部 evidence path 可以保留在结构化 JSON 中，供本地读取；  
但外部 handoff Markdown 不要求 reviewer 去打开这些路径。

## Audit Summary Surface

audit summary surface 是项目级聚合：

- latest audit id / status
- latest report path
- total audit count
- findings count
- top findings
- evidence gaps
- repair recommendations

如果当前没有 audit，也必须明确写成：

```text
当前没有关联审计记录
```

## Projection

project projection 需要挂载 external review surface 摘要：

- `reviewStatus`
- `handoffPath`
- `totalEntries`
- `summaryLine`
- `latestAuditStatus`
- `findingsCount`
- `riskItems`
- `generatedAt`

Desktop 只读 projection，不直接拼 release / audit 内部文件。

## 边界

Release 负责：

- 生成 external review surface
- 写 public handoff Markdown

Audit 负责：

- 提供项目级 audit summary surface

Projection 负责：

- 把 external review summary 投影到项目读模型

## 验证

- `cargo test --workspace`
- `git diff --check`
