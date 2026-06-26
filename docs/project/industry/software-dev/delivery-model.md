# Software Dev Delivery Model

更新日期：2026-06-26
执行者：Codex

## Purpose

Delivery Model 定义 Software Dev 工作流最终交付什么，以及交付如何回流到下一轮 Spec。

## Delivery Package

一个完整交付包应包含：

| Artifact | 说明 |
| --- | --- |
| Summary | 本次交付的目标、范围和结果 |
| Source Diff | 代码或文档变更摘要 |
| Verification | 本地验证命令、输出摘要和失败说明 |
| Evidence Pack | diff、日志、截图、PR、release、provider proof |
| Decision Record | accepted / rejected / deferred / needs-fix 及原因 |
| Delivery Link | PR、release、artifact 或 handoff |
| Feedback Entry | 人类反馈或后续 Spec Evolution 入口 |

## Completion Commit

完成写入必须遵循：

```text
Decision accepted
-> append completed event
-> write issue / run status
-> refresh projection
-> write delivery record
```

Projection 不是 authority。状态写回必须来自 accepted decision 和 event store。

## Feedback Loop

交付后的反馈不直接修改历史 decision。

反馈应该进入：

```text
Feedback
-> Intent
-> Spec Evolution
-> New Spec Bundle / Issue
```
