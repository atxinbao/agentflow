# Pack-specific Projection Mapping Boundary V1

Date: 2026-06-30
Author: Codex

## Purpose

This document defines how Domain Pack, Surface Pack, and Connector Pack data can
shape projection view models without moving industry-specific rules into Core.

Core Projection stays industry-neutral. Pack-specific surfaces provide mapping
metadata, and Projection turns that metadata into read-only view model indexes.

## Boundary

Pack-specific projection mapping is allowed to read:

- Pack manifest metadata;
- Domain object definitions;
- Surface page, workbench, and view model mapping definitions;
- Connector capability definitions.

Pack-specific projection mapping must not write:

- `.agentflow/spec/**`;
- `.agentflow/events/**`;
- `.agentflow/tasks/**`;
- `.agentflow/projections/**` authority inputs;
- `.agentflow/runtime/**` decisions;
- `.agentflow/release/**`.

## Mapping Source

The mapping source is the Surface Pack definition:

```text
PackSurfaceDefinition.viewModelMappings
```

Projection must not hardcode Software Dev page-to-view-model tables as Core
logic. Built-in Software Dev and UI Design packs are only reference pack
definitions.

## Read Model Output

`PackIndustryWorkbenchView` exposes:

- pack readiness;
- domain object index;
- surface page index;
- view model mapping index;
- connector capability index;
- workbench index;
- readonly freshness and warnings.

The `viewModelMappingIndex` is the contract that tells industry apps which
projection read model backs a pack page.

## Missing Mapping Behavior

When a Pack page has no view model mapping, Projection must not fallback to
another pack or to a Core default surface.

The missing mapping is projected as:

```text
status: deferred
reason: pack-surface-view-model-mapping-missing
```

If the Pack definition itself is invalid or unreadable, pack readiness becomes
`invalid`. The UI may display the pack and explain why it is unavailable, but it
must not silently render another pack's task workbench.

## Authority Rule

Pack-specific projection mapping cannot override Core authority status.

It can explain a page, view model, or workbench as ready, invalid, or deferred,
but it cannot change:

- issue status;
- project status;
- decision status;
- evidence status;
- delivery status.

Those facts remain owned by Core authority sources and their read models.

## Release Gate Evidence

The release gate must prove this boundary through:

- `pack-projection-readiness.json.views[].viewModelMappingCount`;
- invalid or deferred mapping entries when mappings are missing;
- projection tests proving custom pack mappings are used;
- negative tests proving custom packs do not fallback to Software Dev objects.

