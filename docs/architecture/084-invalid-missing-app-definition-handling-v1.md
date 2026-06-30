# Invalid / Missing App Definition Handling V1

Date: 2026-06-30
Author: Codex

## Purpose

This document defines how Projection handles invalid, missing, stale, or
disabled app / Pack definitions.

Projection must fail closed. It may explain unavailable app surfaces, but it
must not silently fallback to Software Dev or route unavailable commands into
execution.

## Status Vocabulary

Pack projection uses four explicit definition states:

| Status | Meaning | Command path |
| --- | --- | --- |
| `ready` | Definition is readable and valid. | Allowed only when the whole Pack is ready. |
| `invalid` | Definition is unreadable, malformed, or fails validation. | Blocked. |
| `deferred` | Definition or provider capability is known but currently unavailable. | Blocked. |
| `stale` | App definition is present but not accepted as current. | Blocked. |

## Projection Output

`PackIndustryWorkbenchView.definitionStatusIndex` records:

- `packId`;
- `definitionKind`;
- `status`;
- `reason`;
- `commandExecutionAllowed`.

`PackIndustryWorkbenchView.connectorCapabilityIndex` records command capability
status and `commandExecutionAllowed` per action.

## Failure Cases

Projection must emit structured non-ready states for:

- invalid Pack manifest;
- missing Domain definition;
- missing Surface definition;
- missing Connector definition;
- missing Surface view model mapping;
- disabled skill / provider;
- stale app definition.

## No Fallback Rule

When a selected Pack is invalid or incomplete, Projection must not render
Software Dev as a replacement.

The selected Pack remains selected and receives invalid / deferred / stale
status records. The UI can explain the problem; Runtime command execution must
not start from those records.

## Release Gate Evidence

The release gate must prove this through:

- `pack-projection-readiness.json.views[].definitionStatus`;
- `pack-projection-readiness.json.views[].disabledCommandCapabilities`;
- projection tests covering missing definitions;
- projection tests covering disabled provider state;
- projection tests covering stale app definition state.

