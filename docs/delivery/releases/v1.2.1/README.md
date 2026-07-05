# AgentFlow v1.2.1 First-run Execution Closure and Team Workflow Boundary

更新日期：2026-07-05
执行者：Codex

## Release Baseline

`v1.2.1` 是 First-run Execution Closure and Team Workflow Boundary release baseline。

这一版在 `v1.2.0` Product Onboarding 基础上，先补齐首次运行闭环，再建立本地、轻量、只读的团队协作边界：

```text
First-run onboarding command
-> Readiness binding
-> Guided sample actual run
-> Evidence / decision / delivery proof
-> Failure / retry state
-> Team workflow boundary
-> Project sharing read model
-> Role / permission / handoff view
-> Delivery / decision history view
```

## Scope

`v1.2.1` 收口以下内容：

1. Desktop first-run Runtime command invocation。
2. Desktop onboarding readiness read model binding。
3. Guided sample actual run receipt。
4. Guided sample evidence / decision / delivery proof。
5. First-run failure / retry UI state。
6. Team workflow boundary contract。
7. Project sharing read model。
8. Role / permission / handoff view。
9. Team-readable delivery and decision history。
10. v1.2.1 release certification。

## Team Workflow Boundary

`v1.2.1` 只认证 local / lightweight team workflow：

- Project sharing 只读 read model；
- Role / permission / handoff 只读 view；
- Delivery / decision history 只读 view；
- Feedback 只能回到下一轮 Spec evolution；
- Audit 保持 optional sidecar，不是默认阻断链。

## Non-goals

`v1.2.1` 不包含：

- cloud multi-tenant；
- public commercial launch；
- payment / billing；
- organization admin；
- remote user account system；
- new industry product。

## Primary Proof Index

| Proof | Path / URL | Purpose |
| --- | --- | --- |
| First-run command bridge | `apps/desktop/src-tauri/src/commands/runtime_api.rs` | Desktop 调 Runtime API 而不是本地假状态 |
| Product onboarding Runtime API | `crates/runtime-api/src/product_onboarding.rs` | first-run contract、readiness、guided sample |
| Team workflow boundary | `docs/architecture/087-team-workflow-boundary-contract-v1.md` | local/lightweight team workflow 边界 |
| Project sharing read model | `docs/architecture/088-project-sharing-read-model-v1.md` | Project sharing readonly projection |
| Role / permission / handoff view | `docs/architecture/089-role-permission-handoff-view-v1.md` | role、permission、handoff 只读 view |
| Delivery / decision history view | `docs/architecture/090-team-delivery-decision-history-v1.md` | decision、delivery、feedback、audit sidecar history |
| Release task traceability | `AGENTFLOW_V1_2_1_FIRST_RUN_TEAM_WORKFLOW_TASKS_V1.md` | GitHub issue 到 proof 的索引 |

## GitHub Traceability

Task traceability is recorded in:

- [AGENTFLOW_V1_2_1_FIRST_RUN_TEAM_WORKFLOW_TASKS_V1.md](AGENTFLOW_V1_2_1_FIRST_RUN_TEAM_WORKFLOW_TASKS_V1.md)

## Authority Rules

- GitHub issues are release planning and traceability records, not task authority.
- Runtime API owns command / query entrypoints.
- Projection views are read-only and never write authority.
- Desktop consumes Runtime API / projection views and must not directly promote `.agentflow/**` facts to UI authority.
- Team feedback must enter the next Spec Loop through confirmation.
- Audit sidecar remains separate from the default delivery chain.

## Release Certification

The release gate for `v1.2.1` must certify:

- all V121 issues are closed before the release certification issue closes;
- workspace and Desktop version metadata match `1.2.1`;
- Runtime API, Desktop build, Browser Preview smoke and release-gate E2E pass;
- the certification artifact includes top-level release metadata and primary proof index.

## Next Version

The next release can build on this team workflow boundary to improve Product console continuity after onboarding.
