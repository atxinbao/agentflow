# Software Dev Domain Pack

更新日期：2026-06-26
执行者：Codex

## Purpose

Domain Pack 定义软件开发行业里 Runtime 需要理解的对象、关系、动作和状态。

## Core Objects

| 对象 | 作用 |
| --- | --- |
| Intent | 人类原始意图、问题、需求或反馈 |
| Spec Bundle | 经确认的需求、计划、任务、验收和交付合同 |
| Requirement | 人类可读的需求记录 |
| Plan | 从 Spec 派生的技术计划或执行计划 |
| Issue | 可执行任务合同 |
| Run | Agent 执行一次任务的过程记录 |
| Evidence | 验证、diff、日志、截图、PR、release 等证明材料 |
| Decision | Done / rejected / deferred / needs-fix 判定 |
| Delivery | PR、release、handoff 或交付包 |
| Feedback | 人类或系统对交付结果的反馈 |
| Finding | 独立 Audit sidecar 输出的问题或风险 |

## Standard Flow

```text
Intent
-> Spec Bundle
-> Plan
-> Issue
-> Run
-> Evidence
-> Decision
-> Delivery
-> Feedback
-> Spec Evolution
```

## State Rules

- `Spec Bundle` 确认前不能派生执行合同。
- `Issue` 是 Build Agent 的执行入口。
- `Run` 不能直接写 Done，必须提交 evidence 和 result。
- `Decision` 是完成判定，不等于验证命令通过。
- `Finding` 属于独立 Audit sidecar，不阻塞默认业务链。
