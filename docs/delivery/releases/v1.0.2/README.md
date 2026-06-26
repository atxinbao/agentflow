# AgentFlow v1.0.2 Release Audit Fixes

更新日期：2026-06-26
执行者：Codex

## Status

`v1.0.2` 是 `v1.0.1` 发布后的 release audit fix baseline。

本版本不启动 v1.1 产品功能，不引入 Message Bus，也不把 GitHub issue
升级为 AgentFlow authority。它只修复 `v1.0.1` 发布后审计发现的
runtime governance、release provenance、negative fixture 和产品目标基线问题。

## Scope

`v1.0.2` 收口以下内容：

1. Runtime command admission 只能信任项目 runtime artifact / registry，
   不能信任 request input 里伪造的 provider-ready telemetry。
2. Release provenance 必须正确区分 lightweight tag 和 annotated tag。
3. Release certification 必须包含 V101 hardening claims 的 negative fixtures。
4. 当前产品目标必须明确为 Spec-Driven Software Dev Workflow。
5. Release gate 必须输出 `runtime/v102-release-certification.json`。

## Closeout Artifacts

Release gate 必须生成：

- `runtime/trusted-governance-telemetry.json`
- `runtime/release-provenance.json`
- `runtime/v102-negative-fixtures.json`
- `runtime/v102-release-certification.json`

## Product Baseline

当前产品目标：

```text
AgentFlow = Spec-Driven Software Dev Workflow
```

规则：

- Spec 是 workflow authority；
- Agent 是 executor；
- GitHub issue 只是 planning mirror；
- Software Dev 是当前唯一 active commercial product target。

## Non-goals

- 不启动 v1.1 功能；
- 不增加行业壳；
- 不引入 Message Bus；
- 不改变 v1.0.1 历史 release fact；
- 不把 GitHub issue 当成 AgentFlow authority。
