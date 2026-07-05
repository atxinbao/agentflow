# Role / Permission / Handoff View v1

更新日期：2026-07-05  
执行者：Codex

## Purpose

`v1.2.1` 的团队工作流需要一个用户可读的角色、权限和交接视图。

这个视图回答：

- 当前谁负责下一步；
- 每个角色能读什么；
- 每个角色能做什么；
- Spec / Build / Audit / Review / Owner / Viewer 之间如何交接；
- 缺失 owner 或未知角色如何被显式标记为 invalid。

## Runtime API

Runtime API 暴露只读查询：

```text
role_permission_handoff_view(projectRoot, projectId)
```

版本：

```text
agentflow-role-permission-handoff-view.v1
```

## Source Boundary

该 view 只能读取 projection/read model：

```text
.agentflow/projections/projects/<project-id>.json
```

它不能把 Desktop-only state 当成权限事实源，也不能直接写：

```text
.agentflow/spec/**
.agentflow/events/**
.agentflow/tasks/**
.agentflow/runtime/**
```

## Roles

| Role | Can Read | Can Act |
| --- | --- | --- |
| Spec Agent | project sharing、confirmed requirements、feedback input | preview requirement、materialize spec |
| Build Agent | project sharing、spec issue、handoff、task evidence | start work loop、write evidence、submit delivery |
| Audit Agent | project sharing、delivery history、decision history | inspect delivery、write audit report |
| Review Agent | project sharing、handoff state、delivery history | record review feedback |
| Human Owner | project sharing、all readonly team views | accept delivery、request changes、confirm next loop |
| Viewer | project sharing、delivery history | none |

## Handoff States

| Handoff | Meaning |
| --- | --- |
| `spec-to-build` | confirmed task enters work loop |
| `build-to-audit` | delivery evidence can be inspected |
| `audit-to-owner` | audit report returns to Human Owner |
| `owner-to-spec-feedback` | feedback enters Spec Loop only after confirmation |

## Invalid Cases

The view must declare negative fixtures:

- `invalid-role`：未知角色不能获得权限；
- `missing-owner`：项目还有未完成任务但没有 current owner 时，view 为 `invalid`。

## Authority Rule

Role / Permission / Handoff View 必须始终：

```text
readonly = true
authority = false
```

缺失 project projection 时：

```text
projectionBacked = false
status = invalid
```

存在 project projection 时：

```text
projectionBacked = true
```

## Acceptance

- Runtime API 返回结构化 role / permission / handoff view；
- API plane 暴露 `projection.role-permission-handoff`，且是 readonly projection query；
- Desktop Tauri 暴露只读命令；
- view 包含 Spec / Build / Audit / Review / Owner / Viewer 六类角色；
- view 显示 current owner；
- missing owner 和 invalid role 均有 negative fixture；
- 权限状态不写在 Desktop-only 逻辑里。
