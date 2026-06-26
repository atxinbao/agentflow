# Software Dev Connector Pack

更新日期：2026-06-26
执行者：Codex

## Purpose

Connector Pack 定义 Software Dev 产品可以连接哪些外部工具，以及这些工具在 AgentFlow 中的边界。

连接器提供能力，不提供项目事实源。

## Standard Connectors

| Connector | 作用 | 边界 |
| --- | --- | --- |
| Git | diff、branch、commit、merge proof | 不能替代 AgentFlow issue / run state |
| GitHub | PR、issue sync、release、checks URL | GitHub issue 不是 AgentFlow authority |
| Codex | 执行本地任务、修改文件、运行验证 | Codex session 不是项目事实源 |
| Claude Code | 可选执行器 runtime | Claude session 不是项目事实源 |
| Browser Preview | 前端渲染、截图、交互验证 | 只产生 evidence |
| Local Test Command | cargo、npm、脚本验证 | 只产生 verification evidence |
| Release Gate | 发布认证与 negative fixture | 只产生 certification evidence |

## Rules

- Connector 必须声明 capability。
- Connector 失败不能被伪装成 ready。
- Connector 输出必须归档为 evidence 或 provider proof。
- Connector 不能绕过 Runtime API 写 Done。
- Provider smoke optional 必须有 skipped / not configured / failed / passed 的结构化 proof。
