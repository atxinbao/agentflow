# Public Delivery Standard V1

创建日期：2026-06-18
执行者：Codex

## 目标

让外部 reviewer 不进入 `.agentflow/**`，也能看懂一次交付做了什么、怎么验证、影响是什么。

## 公开交付边界

本地事实继续留在：

- `.agentflow/events/**`
- `.agentflow/projections/**`
- `.agentflow/tasks/<issue-id>/runs/**`
- `.agentflow/tasks/<issue-id>/evidence/**`

公开交付只进入：

- PR/MR body
- `CHANGELOG.md`
- release notes

公开面不能直接暴露：

- `.agentflow/tasks/**` 内部路径
- checkpoint / run / event 原始文件路径
- 只对本地 runtime 有意义的中间状态文件名

## 任务级公开交付模板

任务级公开交付由 Build Agent 写入 PR/MR body。

固定结构：

1. 任务
   - issue id
   - 标题
   - 来源需求 id
   - 来源需求文档
   - workflow ref
2. 变更摘要
   - 本次任务做了什么
3. 验证
   - 验证命令数量
   - 证据状态
4. 公开交付
   - 当前公开交付状态
   - 目标位置
   - PR/MR
   - 合并提交
   - CHANGELOG
   - release notes

说明：

- PR/MR body 是任务级公开交付主记录。
- 任务级模板可以引用 `docs/requirements/**`，不能要求外部读者打开 `.agentflow/**`。
- 本地证据可以被说明为“已记录”，但不直接暴露内部 evidence 路径。

## 版本级公开交付模板

版本级公开交付由 `release` 模块从已完成任务汇总生成。

### CHANGELOG

每条任务至少包含：

- issue id
- 标题
- 来源需求 id
- 变更摘要
- 验证状态
- 公开交付目标
- PR/MR
- 合并提交

### Release Notes

每条任务至少包含：

- issue id
- 标题
- 变更摘要
- 状态
- 验证摘要
- 公开交付目标
- PR/MR
- 合并提交
- 来源需求文档

## Projection 读模型要求

任务页和项目页读取的 `delivery summary` 必须统一输出：

- `status`
- `evidenceStatus`
- `publicRecordPath`
- `publicRecordTargets`
- `publicRecordItems`
- `missingPublicRecords`
- `summaryLine`
- `publicRecordMarkdown`

其中：

- `summaryLine` 是给 UI 的单行摘要；
- `publicRecordMarkdown` 是任务级公开交付模板正文；
- `publicRecordItems` 只记录已经落地的公开记录；
- `publicRecordTargets` 记录应该出现在哪些公开表面。

## 不做事项

- 不新增独立公开门户
- 不把公开交付重新写回 `.agentflow/tasks/<issue-id>/delivery/**`
- 不在公开交付里直接展示内部 runtime 文件路径
