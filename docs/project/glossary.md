# Glossary

更新日期：2026-07-01
执行者：Codex

| 术语 | 定义 |
| --- | --- |
| AgentFlow | Spec-Driven AI OS Project |
| AI OS Project | Core OS Runtime 加 Industry Product Surface 的项目操作系统形态 |
| Core OS Runtime | Spec、Ontology、Runtime、Evidence、Decision、Projection 六个 Kernel 的底层能力 |
| Industry Product Surface | 行业产品表面，可以是 Paid Report Flow、Managed Project Flow 或 Assistant / Ops Flow |
| Industry Product Source | `products/**` 下的行业壳源码定义，例如 `products/software-dev/**` |
| Paid Report Flow | 用户输入信息并付费，AgentFlow 内部完成一次受控 Run，交付一份可验收报告 |
| Managed Project Flow | 围绕目标、任务、证据、验收和交付运行的多步项目型行业产品形态 |
| Assistant / Ops Flow | 持续托管、监控、提醒、执行和反馈的行业产品形态 |
| Software Dev Reference App | 第一个官方行业参考产品，用于认证 Core OS Runtime 的真实闭环 |
| Spec Bundle | 经确认的需求、计划、任务、证据、判定和交付合同 |
| Domain Pack | Product Surface 内置的行业对象、动作、关系和状态定义 |
| Surface Pack | Product Surface 内置的行业页面、报告、读模型、命令入口和工作台定义 |
| Connector Pack | Product Surface 内置的外部工具、provider、skill 和执行器连接定义 |
| Evidence | 证明发生过什么的材料，例如 diff、test log、PR、release、截图 |
| Decision | Done / rejected / deferred / needs-fix 等完成判定 |
| Delivery | PR、release、handoff、decision record 或交付包 |
| Audit Sidecar | 独立审计流程，不属于默认业务主链 |
| Runtime Authority | `.agentflow/**` 中由 AgentFlow Runtime 维护的执行事实源 |
| Projection | 从 authority 派生给 UI 和第三方读取的只读视图 |
