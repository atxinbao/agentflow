# AgentFlow v0.7.1 Console Release Certification Closeout

日期：2026-06-22
执行者：Codex
状态：Release Certification Evidence / v0.7.x closeout remediation

## 1. Purpose

本目录收口 `v0.7.0` 发布后的 Console release certification 证据问题。

`v0.7.1` 不是新的 Console 功能版本。

它的目标是：

```text
把 v0.7.x release 事实、gate run、artifact 和 Console readiness 口径记录成可复查证据
```

大白话：

```text
v0.7.0 已经发布，并且 main / tag / release 三类 release-gate 都通过。
v0.7.1 要把这些证明写成稳定文档，避免以后只靠 GitHub 页面临时查询。
```

## 2. Reading Order

1. [AGENTFLOW_V0_7_1_RELEASE_CERTIFICATION_EVIDENCE_V1.md](AGENTFLOW_V0_7_1_RELEASE_CERTIFICATION_EVIDENCE_V1.md)
2. [../v0.7.0/README.md](../v0.7.0/README.md)
3. [../v0.7.0/AGENTFLOW_V0_7_0_PROJECT_OS_CONSOLE_READINESS_EVIDENCE_V1.md](../v0.7.0/AGENTFLOW_V0_7_0_PROJECT_OS_CONSOLE_READINESS_EVIDENCE_V1.md)

## 3. Scope

`v0.7.1` 包含：

- 记录 `v0.7.0` main push release-gate；
- 记录 `v0.7.0` tag push release-gate；
- 记录 `v0.7.0` GitHub Release published release-gate；
- 记录 release URL、tag、source commit 和 gate artifact；
- 明确当前 gate 是 Console release certification，不是 provider production E2E。
- 明确 Browser Preview readiness、Console readiness 和真实 Tauri workspace projection readiness 是三个不同层级。

`v0.7.1` 不包含：

- 新 Console 页面；
- 新 runtime 行为；
- Provider fleet；
- 自动审计；
- 行业 Pack；
- 从 GitHub issue 内容直接生成 `.agentflow/spec/**` authority。

## 4. Completion Standard

`v0.7.1` 完成时，必须满足：

- release evidence 文档包含 `mainGateRun`、`tagGateRun`、`releaseGateRun`；
- certification 表格链接 release URL、source commit、gate run 和 artifact；
- 文档不再把 `v0.7.0` 描述成缺失 workflow visibility；
- 后续 release gate 能继续产出同名 artifact，供 release evidence 引用。
- Browser Preview smoke 和 Console readiness 已进入 release-gate；
- Tauri workspace projection readiness 有真实临时 workspace fixture 覆盖；
- `docs/v0.7.0/**` 不再把已发布版本描述成 planning draft。

## 5. Closeout Status

| Item | Purpose | Status |
| --- | --- | --- |
| `V071-001` | 固定 `v0.7.0` release certification evidence | done |
| `V071-002` | 把 Browser Preview smoke 和 Console readiness 纳入 release-gate | done |
| `V071-003` | 增加真实临时 workspace 的 Tauri projection readiness 覆盖 | done |
| `V071-004` | 把 `docs/v0.7.0/**` 从 planning draft 收口为发布实现记录 | in progress |
| `V071-005` | 澄清 Console write boundary | pending |
| `V071-006` | 澄清 provider smoke gate 边界 | pending |

## 6. Boundary

`v0.7.1` 是 release closeout remediation。

它不改变 `v0.7.0` 的 Console 产品范围，也不把 Browser Preview smoke 夸大成完整桌面打包验收。
