# AgentFlow v1.0.4 Core Ontology Kernel

更新日期：2026-06-28
执行者：Codex

## Status

`v1.0.4` 是 `v1.0.3` Core 4-D Spec Intake 之后的 Core Ontology Kernel baseline。

本版本不启动 `v1.1` 产品功能，不把 Software Dev Reference App 的行业词汇写入 Core authority。它把 Object、Link、Action、State、Skill、Evidence、Decision、Registry 和 Projection 合同落成，并由 release gate 认证。

## Scope

`v1.0.4` 收口以下内容：

1. Core intake pollution guard hardening。
2. Core 4-D positive / negative certification。
3. Core Ontology Kernel Contract。
4. Core Object / Link Schema。
5. Core Action / State Semantics。
6. Core Skill Registry / Action Authorization。
7. Core Evidence / Decision Reference Model。
8. Core file-backed ontology registry and read-only projection baseline。
9. v1.0.4 release certification artifact。

## Closeout Artifacts

Release gate 必须生成：

- `runtime/core-ontology-kernel.json`
- `runtime/core-object-link-schema.json`
- `runtime/core-action-state-semantics.json`
- `runtime/core-skill-registry.json`
- `runtime/core-evidence-decision-reference-model.json`
- `runtime/core-file-backed-ontology-registry.json`
- `runtime/v104-release-certification.json`

## Public Records

- [../../../architecture/054-core-ontology-kernel-contract-v1.md](../../../architecture/054-core-ontology-kernel-contract-v1.md)
- [../../../architecture/055-core-object-link-schema-v1.md](../../../architecture/055-core-object-link-schema-v1.md)
- [../../../architecture/056-core-action-state-semantics-v1.md](../../../architecture/056-core-action-state-semantics-v1.md)
- [../../../architecture/057-core-skill-registry-action-authorization-v1.md](../../../architecture/057-core-skill-registry-action-authorization-v1.md)
- [../../../architecture/058-core-evidence-decision-reference-model-v1.md](../../../architecture/058-core-evidence-decision-reference-model-v1.md)
- [../../../architecture/059-core-file-backed-ontology-registry-projection-v1.md](../../../architecture/059-core-file-backed-ontology-registry-projection-v1.md)

## Non-goals

- 不启动 `v1.1` 功能；
- 不实现行业壳；
- 不引入新的 Message Bus；
- 不把 GitHub issue 当成 AgentFlow authority；
- 不把 Software Dev 的 issue、PR、repository、test log 写成 Core required field。
