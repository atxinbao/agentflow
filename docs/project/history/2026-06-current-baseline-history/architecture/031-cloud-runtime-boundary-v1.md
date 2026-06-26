# Cloud Runtime Boundary V1

创建日期：2026-06-23
执行者：Codex

## Purpose

本文档定义 AgentFlow Cloud Runtime 的部署边界。

它回答：

```text
云端 Runtime 承载什么？
行业 UI 如何接入？
Pack / Connector 如何加载？
云端 Runtime 和本地文件事实源是什么关系？
哪些内容绝不能进入 Runtime Core？
```

`v0.9.0 / V090-002` 只定义 Cloud Runtime boundary，不绑定任何云厂商。

## Cloud Runtime Shape

Cloud Runtime 只承载 Runtime Core 和 API Plane。

```text
Cloud Runtime
  -> Runtime API
  -> Event API
  -> Projection Query API
  -> Pack Registry API
  -> Governance Policy
  -> Worker / Connector Admission
```

Cloud Runtime 不承载行业 UI，不拥有行业页面，也不直接写项目源码。

## Local / Cloud Split

| Responsibility | Local Runtime | Cloud Runtime |
| --- | --- | --- |
| Developer workspace | owns local files | no direct ownership |
| `.agentflow/spec/**` authority | file-backed source | synced / imported authority view |
| `.agentflow/events/**` | local durable facts | hosted event stream / replicated facts |
| `.agentflow/projections/**` | local rebuild output | queryable read model |
| Pack definitions | file / fixture / project source | registered pack version |
| Runtime API | local process | hosted API |
| Industry UI | local Desktop / web shell | external client |
| Connector output | local connector artifact | admitted external fact |

云端 Runtime 可以承载 authority 的托管副本，但所有写入仍必须经过 Runtime API /
Event API / Governance Policy。

## Runtime Core Boundary

Cloud Runtime Core 包含：

- command admission
- action proposal lifecycle
- arbitration / decision
- event append
- projection rebuild trigger
- pack registry lookup
- capability / provider status lookup
- governance policy decision

Cloud Runtime Core 不包含：

- industry-specific UI；
- Desktop 页面；
- project file editor；
- GitHub / GitLab 直接写入逻辑；
- provider session UI；
- audit report authoring UI。

## Industry Client Boundary

行业客户端只能通过 API 访问 Cloud Runtime：

```text
Industry Client
  -> Runtime API / SDK
  -> Projection Query API
  -> Command Surface
```

行业客户端可以：

- 读取 Projection；
- 提交 command；
- 读取 command status；
- 读取 rejected validation report；
- 展示 Pack-specific surface。

行业客户端不能：

- 直接写 Event Store；
- 直接写 Spec Issue；
- 直接修改 Projection；
- 直接绕过 Governance Policy；
- 直接把 connector 输出写成 authority。

## Pack / Connector Loading

Cloud Runtime 只能加载已登记的 Pack version。

```text
Pack Registry
  -> Pack Version
  -> Domain / Surface / Connector Definition
  -> Runtime API Plane
```

Connector 输出必须先进入 External Fact / Evidence / Provider Artifact，再由 Runtime API
做 admission。Connector 不能直接写：

```text
spec issue
event store
task evidence
projection
release fact
```

## Local Filesystem Source Relation

本地项目文件仍然是开发者体验的主要事实源。

Cloud Runtime 可以读取或同步这些事实：

- requirements；
- spec issues；
- pack definitions；
- event stream；
- task evidence；
- release certification artifact。

同步后必须保留：

- source workspace id；
- source commit / fingerprint；
- schema version；
- pack version；
- event cursor；
- projection rebuild id。

Cloud Runtime 不能把无法追溯来源的同步结果升级为 authority。

## Governance Boundary

Cloud Runtime 必须在 command admission 前运行 Governance Policy。

Governance 输入包括：

- role policy；
- capability registry；
- provider smoke；
- connector policy；
- pack compatibility；
- audit sidecar policy；
- migration state；
- replay / projection freshness。

Governance 输出只能是：

```text
allow
reject
defer
```

`allow` 只表示可以进入 Runtime API；不表示执行已经成功。

## Deployment Invariants

- Cloud Runtime 只暴露 Runtime API / SDK，不绑定行业 UI。
- Industry UI 通过 Projection / API 读取，不进入 Runtime Core。
- Connector 输出必须经过 Runtime admission。
- Pack 是定义，不是 authority writer。
- Projection 是 read model，不是 fact source。
- Governance 是 admission gate，不是事后报告。
- Local Runtime 和 Cloud Runtime 可以部署形态不同，但 authority 边界必须一致。

## Non-goals

本文件不做：

- 绑定 AWS / GCP / Azure / Vercel / Fly 等云厂商；
- 实现远程多租户；
- 实现生产部署；
- 引入默认 Message Bus；
- 把行业 UI 放进 Runtime Core；
- 让 Connector 直接写 authority；
- 让 Projection 成为同步事实源。
