# Core File-backed Ontology Registry and Projection V1

更新日期：2026-06-28
执行者：Codex

## Purpose

本文件定义 Core ontology 的文件化 Registry 与只读 Projection 基线。

目标很简单：

```text
Core ontology contract files
-> registry source list
-> read-only projection entries
-> UI / Runtime / Industry Apps query surface
```

它只解决“Core 定义从哪些文件来、对外提供哪些只读查询面”。它不执行动作，不写运行时事实，不替代 source contract。

## Authority Boundary

Core authority 只来自已版本化的架构合同文件和对应 Rust contract。

Reference App 可以把自身领域映射到这些 Core 定义，但 mapping is not Core authority。

Core 允许的通用语言：

- Object
- Link
- Action
- State
- Skill
- Evidence
- Decision
- Artifact
- Route
- Spec Bundle
- Projection

Core 不允许把行业任务系统词汇写成必填字段、状态、对象或投影权威。

禁止进入 Core authority 的词：

- bug
- feature
- issue
- pr
- pull-request
- release
- repository
- repository-patch
- test-log
- github-issue

这些词只能出现在 Reference App mapping 或行业 fixture 中，且必须明确不是 Core authority。

## Registry Sources

Registry 必须至少包含以下 source：

| sourceId | relativePath | contractVersion | readModelKind |
| --- | --- | --- | --- |
| core-ontology-kernel | docs/architecture/054-core-ontology-kernel-contract-v1.md | agentflow-core-ontology-kernel.v1 | OntologyKernel |
| core-object-link-schema | docs/architecture/055-core-object-link-schema-v1.md | agentflow-core-object-link-schema.v1 | ObjectLinkSchema |
| core-action-state-semantics | docs/architecture/056-core-action-state-semantics-v1.md | agentflow-core-action-state-semantics.v1 | ActionStateSemantics |
| core-skill-registry | docs/architecture/057-core-skill-registry-action-authorization-v1.md | agentflow-core-skill-registry.v1 | SkillRegistry |
| core-evidence-decision-reference-model | docs/architecture/058-core-evidence-decision-reference-model-v1.md | agentflow-core-evidence-decision-reference-model.v1 | EvidenceDecisionReferenceModel |

规则：

- `relativePath` 必须是稳定相对路径。
- source 必须声明 contract version。
- source 可被 Reference App 映射，但 mapping is not Core authority。

## Projection Entries

Projection 是只读派生面，不是新的事实源。

Projection entries do not replace source contracts.

| projectionId | sourceId | projectionKind | querySurfaces | minimumRecordCount |
| --- | --- | --- | --- | --- |
| core-kernel-map | core-ontology-kernel | KernelElementCatalog | coreElementCatalog, coreBoundaryMap | 11 |
| core-object-link-catalog | core-object-link-schema | ObjectLinkCatalog | objectCatalog, linkCatalog, relationshipQuery | 23 |
| core-action-state-catalog | core-action-state-semantics | ActionStateCatalog | actionCatalog, stateCatalog, transitionQuery | 34 |
| core-skill-capability-catalog | core-skill-registry | SkillCapabilityCatalog | skillCatalog, authorizationQuery, capabilityMatrix | 6 |
| core-evidence-decision-catalog | core-evidence-decision-reference-model | EvidenceDecisionCatalog | evidenceCatalog, decisionCatalog, outcomeQuery | 18 |

规则：

- projection 必须指向已存在的 registry source。
- projection 必须声明 query surface。
- projection 必须声明 minimum record count。
- projection 可以被 UI、Runtime、Industry Apps 查询。
- projection 不得替代 source contract。

## Runtime Contract

Rust contract：

```text
crates/ontology/src/file_registry.rs
```

对外入口：

```text
core_file_backed_ontology_registry_projection_contract()
validate_core_file_backed_ontology_registry_projection_contract()
```

release-gate primary proof：

```text
runtime/core-file-backed-ontology-registry.json
runtime/core-file-backed-ontology-registry-rust-test.log
```

## Acceptance

本合同完成后必须满足：

- Registry source list 覆盖 5 个 Core ontology contracts。
- Projection entries 覆盖 kernel / object-link / action-state / skill / evidence-decision。
- Rust validator 能拒绝绝对路径、缺失 source、空 projection、行业词污染。
- release-gate 能生成小型 certification artifact。
- Core 仍保持 industry-neutral。
