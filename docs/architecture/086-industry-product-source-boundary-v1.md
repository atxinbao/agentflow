# Industry Product Source Boundary V1

创建日期：2026-07-01
执行者：Codex

## Purpose

本文固定 AgentFlow `v1.0.9` 之前必须先确认的源码边界。

AgentFlow 的系统公式是：

```text
AgentFlow AI OS Project
= Core OS Runtime
+ Industry Product Surface
```

这个公式在源码层面必须对应为：

```text
crates/**     = Core OS Runtime
products/**   = Industry Product Surface / Reference App source
apps/**       = user-facing clients
docs/**       = human-readable project and architecture records
.agentflow/** = Runtime fact source
```

核心原则：

```text
Core 定规则。
Product 用规则。
```

## Directory Authority

| Path | Owns | Must not own |
| --- | --- | --- |
| `crates/**` | Core kernels, runtime contracts, Pack loader / validator / registry, projection contracts | Industry-specific objects such as Issue / PR / Release / Payment as Core authority |
| `products/**` | first-party and future industry product source definitions | Runtime authority facts or Core kernel contracts |
| `apps/**` | UI clients and product-facing interaction surfaces | Core authority facts or Pack source of truth |
| `docs/**` | human-readable project goal, roadmap, architecture, requirements, delivery records | Runtime state or executable task authority |
| `.agentflow/**` | materialized runtime facts, specs, tasks, events, evidence, projections | source product definitions |

`products/**` is source code / product definition surface. `.agentflow/**` is runtime fact surface.

They are not interchangeable.

## Core Boundary

Core OS Runtime can define these universal concepts:

```text
Spec
Object
Link
Action
State
Skill
Run
Artifact
Evidence
Decision
Projection
Route
```

Core OS Runtime must not hardcode Software Dev concepts as universal authority:

```text
Issue
Pull Request
Release
Patch
Test Log
Repository
GitHub
Codex
Claude
```

Core OS Runtime must not hardcode Paid Report commercial concepts as universal authority:

```text
Order
Payment
Refund
Customer
Report Template
Report Delivery
```

These concepts belong to industry product source definitions under `products/**` and enter Core only through Pack contracts, Runtime API, Evidence, Decision and Projection boundaries.

## Product Boundary

Industry product source should use this shape:

```text
products/
  <product-id>/
    product.toml
    domain/
    surface/
    connectors/
    flows/
    projections/
    fixtures/
    tests/
```

For the first reference product:

```text
products/
  software-dev/
    product.toml
    domain/
    surface/
    connectors/
    flows/
    projections/
    fixtures/
    tests/
```

Software Dev can be first-party and built into AgentFlow distribution.

It still remains a product source module, not Core.

## Pack Boundary

`crates/pack` owns the generic Pack contract:

- manifest loading;
- schema validation;
- registry lookup;
- migration receipt semantics;
- projection readiness checks;
- invalid / missing definition handling.

`crates/pack` must not become the permanent home of first-party product source.

`crates/pack/fixtures/**` is allowed only for test fixtures and release-gate negative / positive fixtures.

Current Software Dev fixture definitions may be used as migration input, but `v1.0.9` should certify the first-party product source from:

```text
products/software-dev/**
```

not from:

```text
crates/pack/fixtures/packs/software-dev/**
```

## Runtime Boundary

Product source does not write authority.

The legal path is:

```text
products/<product-id>/**
-> Pack Registry / Product Registry
-> Runtime API
-> .agentflow/spec/**
-> .agentflow/tasks/**
-> .agentflow/events/**
-> .agentflow/evidence/**
-> .agentflow/projections/**
```

Product source can define object names, surfaces, connectors and flows.

Only Runtime can materialize confirmed specs, accepted actions, evidence, decisions and projections.

## App Boundary

`apps/**` can render product surfaces and command entry points.

It must consume:

```text
Projection API
View Model
Runtime Command API
Product Surface definitions
```

It must not directly mutate:

```text
.agentflow/spec/**
.agentflow/events/**
.agentflow/tasks/**
.agentflow/evidence/**
.agentflow/projections/**
```

## v1.0.9 Requirement

`v1.0.9` should certify Software Dev as the first industry shell:

```text
Core OS Runtime
+ products/software-dev
= Software Dev Reference App
```

The certification must prove:

- Software Dev source lives outside Core;
- Software Dev definitions enter Core through Pack / Product registry contracts;
- Core does not hardcode Software Dev objects as universal authority;
- `crates/pack/fixtures/**` remains fixture-only;
- Projection reads Product / Pack mapping but does not write authority;
- release certification can trace Product source -> Runtime facts -> Evidence -> Decision -> Delivery.

## Non-goals

This boundary does not implement `products/software-dev/**`.

It does not define marketplace packaging.

It does not define paid report ordering, payment, refund or customer account systems.

It does not move existing fixtures by itself.

It exists to prevent `v1.0.9` from turning the first industry shell into Core pollution.
