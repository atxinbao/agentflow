# AgentFlow v0.7.1 Release Certification Evidence V1

日期：2026-06-22
执行者：Codex
状态：Release Certification Evidence / V071-001

## 1. Certification Judgment

`v0.7.0` 已完成发布，并且以下三类 release-gate 都已经通过：

```text
main push gate
tag push gate
GitHub Release published gate
```

本文件补齐的不是功能能力，而是 durable release evidence：

```text
v0.7.0 有成功 gate run。
v0.7.1 把这些 run、artifact、release URL 和 source commit 固定成可复查版本证据。
```

## 2. Release Facts

| Field | Value |
| --- | --- |
| Release | [AgentFlow v0.7.0](https://github.com/atxinbao/agentflow/releases/tag/v0.7.0) |
| Tag | `v0.7.0` |
| Source commit | `2890c4d173a935eb8cbafc6f7fc067cc9ce77172` |
| Release published at | `2026-06-22T11:39:28Z` |
| Release title | `AgentFlow v0.7.0` |
| Release gate artifact name | `release-gate-v0.7.0` |

## 3. Gate Runs

| Gate | Run | Event | Commit | Artifact | Conclusion |
| --- | --- | --- | --- | --- | --- |
| PR gate | [27949420836](https://github.com/atxinbao/agentflow/actions/runs/27949420836) | `pull_request` | `fdea2fbd264235d38fbf84c6c35f921f7e5d6cb0` | `release-gate-v0.7.0` | success |
| mainGateRun | [27949663555](https://github.com/atxinbao/agentflow/actions/runs/27949663555) | `push` / `main` | `2890c4d173a935eb8cbafc6f7fc067cc9ce77172` | `release-gate-v0.7.0` | success |
| tagGateRun | [27949712566](https://github.com/atxinbao/agentflow/actions/runs/27949712566) | `push` / `v0.7.0` | `2890c4d173a935eb8cbafc6f7fc067cc9ce77172` | `release-gate-v0.7.0` | success |
| releaseGateRun | [27949939628](https://github.com/atxinbao/agentflow/actions/runs/27949939628) | `release` / published | `2890c4d173a935eb8cbafc6f7fc067cc9ce77172` | `release-gate-v0.7.0` | success |

Artifact API references at the time this evidence was written:

| Gate | Artifact id | Download API |
| --- | --- | --- |
| PR gate | `7791041591` | `https://api.github.com/repos/atxinbao/agentflow/actions/artifacts/7791041591/zip` |
| mainGateRun | `7791140984` | `https://api.github.com/repos/atxinbao/agentflow/actions/artifacts/7791140984/zip` |
| tagGateRun | `7791152566` | `https://api.github.com/repos/atxinbao/agentflow/actions/artifacts/7791152566/zip` |
| releaseGateRun | `7791247648` | `https://api.github.com/repos/atxinbao/agentflow/actions/artifacts/7791247648/zip` |

说明：GitHub Actions artifact 有保留期限制。长期证明以本文件中的 run URL、commit、release URL、artifact name 和 gate conclusion 为准。

## 4. Certification Table

| Requirement | Evidence | Status |
| --- | --- | --- |
| `mainGateRun` recorded | [27949663555](https://github.com/atxinbao/agentflow/actions/runs/27949663555) | done |
| `tagGateRun` recorded | [27949712566](https://github.com/atxinbao/agentflow/actions/runs/27949712566) | done |
| `releaseGateRun` recorded | [27949939628](https://github.com/atxinbao/agentflow/actions/runs/27949939628) | done |
| Release URL recorded | [AgentFlow v0.7.0](https://github.com/atxinbao/agentflow/releases/tag/v0.7.0) | done |
| Source commit recorded | `2890c4d173a935eb8cbafc6f7fc067cc9ce77172` | done |
| Gate artifact recorded | `release-gate-v0.7.0` | done |
| No false claim that v0.7.0 lacked workflow visibility | This document states the gap was durable evidence, not missing gate execution. | done |

## 5. Boundary

This certification proves:

- release metadata for `v0.7.0` was aligned before tag creation;
- `v0.7.0` tag and GitHub Release exist;
- main / tag / release event gates passed for the same source commit;
- release-gate uploaded certification artifacts.

This certification does not prove:

- full provider production lifecycle execution;
- automatic audit execution;
- cloud runtime readiness;
- industry Pack readiness;
- that GitHub issue content is an AgentFlow authority source.

## 6. Follow-up

The next V071 items should keep this evidence boundary:

- `V071-002` should make Console readiness commands explicit in release-gate.
- `V071-003` should add a real temporary workspace projection readiness check.
- `V071-004` should update released v0.7.0 docs from planning language to closeout language.
- `V071-005` should clarify onboarding writes versus Console read-only surfaces.
- `V071-006` should avoid overclaiming real provider lifecycle coverage.

