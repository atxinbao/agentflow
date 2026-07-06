# AgentFlow v1.2.2 Release Proof Hardening and Commercial Boundary Preflight

更新日期：2026-07-06
执行者：Codex

## Release Baseline

`v1.2.2` 是 Release Proof Hardening and Commercial Boundary Preflight release baseline。

这一版建立在 `v1.2.1` First-run Execution Closure and Team Workflow Boundary 之上，先修复 release proof 可审计性，再冻结商业边界的第一批合同：

```text
V121 release proof hardening
-> root certification metadata alignment
-> primary proof artifact package
-> issue / milestone closeout gate
-> Desktop team workflow surface binding
-> commercial boundary contract
-> license / entitlement boundary
-> paid feature boundary
-> paid report flow vs managed project flow contract
-> v1.2.2 release certification
```

## Scope

`v1.2.2` 收口以下内容：

1. v1.2.1 dedicated release certification gate。
2. Root certification top-level metadata alignment。
3. V121 primary proof artifact generation。
4. V121 issue traceability and milestone closeout gate。
5. Desktop team workflow surface binding。
6. Commercial boundary contract。
7. License / entitlement boundary。
8. Paid feature boundary。
9. Paid report flow vs managed project flow contract。
10. v1.2.2 release certification。

## Certified Boundary

`v1.2.2` 认证的是证明链和商业边界，不是商业发布。

这一版确认：

- small certification artifact carries top-level release metadata；
- primary proofs are hash-indexed and externally readable；
- V121 closeout is traceable from issue to release proof；
- Desktop team workflow surfaces are bound to Runtime read models；
- commercial product layer cannot bypass Runtime admission；
- license / entitlement only decides access boundary, not payment；
- paid feature boundary blocks paid-only flow before Runtime execution；
- paid report flow and managed project flow both map to Spec / Evidence / Decision / Delivery。

## Commercial Boundary

`v1.2.2` defines two commercial workflow shapes:

```text
Paid Report Flow
= input -> product access check -> order intent -> controlled run -> evidence -> decision -> report delivery -> feedback

Managed Project Flow
= goal -> spec -> tasks -> execution -> evidence -> decision -> delivery -> feedback
```

Both reuse Core Runtime facts and Product surfaces. Neither flow adds a new industry Product.

## Non-goals

`v1.2.2` 不包含：

- payment provider integration；
- checkout / billing implementation；
- cloud multi-tenant collaboration；
- public commercial launch；
- customer account system；
- organization admin；
- new industry Product；
- managed service operations。

## Primary Proof Index

| Proof | Path / URL | Purpose |
| --- | --- | --- |
| Release metadata | `runtime/v121-release-certification-top-level-metadata.json` | top-level metadata and V122 traceability |
| Primary proof manifest | `runtime/v121-certification-artifact-manifest-primary-proof-index.json` | proof hash / byte / issue index |
| First-run command | `runtime/v121-first-run-runtime-command-invocation.json` | inherited V121 command proof |
| Guided sample closure | `runtime/v121-guided-sample-execution-closure.json` | inherited V121 evidence / decision / delivery proof |
| Team workflow boundary | `runtime/v121-team-workflow-boundary-contract.json` | inherited local team workflow boundary |
| Project sharing | `runtime/v121-project-sharing-read-model.json` | readonly team project projection |
| Role / handoff | `runtime/v121-role-permission-handoff-view.json` | readonly role and handoff view |
| Team history | `runtime/v121-team-delivery-decision-history-view.json` | delivery / decision / feedback history |
| Desktop team surface | `runtime/v121-desktop-team-workflow-surface-binding.json` | Desktop reads team workflow surfaces |
| Commercial boundary | `runtime/v121-commercial-boundary-contract.json` | commercial layer boundary |
| License / entitlement | `runtime/v121-license-entitlement-boundary.json` | license and entitlement boundary |
| Paid feature | `runtime/v121-paid-feature-boundary.json` | paid-only feature access boundary |
| Commercial workflow shapes | `runtime/v121-commercial-workflow-shapes.json` | paid report vs managed project flow |
| V121 issue / milestone closeout | `runtime/v121-issue-milestone-closeout.json` | inherited V121 issue and milestone closeout |
| V121 release certification | `runtime/v121-release-certification.json` | inherited V121 certification package |
| V122 issue / milestone closeout | `runtime/v122-issue-milestone-closeout.json` | V122 issue and milestone closeout |
| V122 release certification | `runtime/v122-release-certification.json` | v1.2.2 final certification |

## GitHub Traceability

Task traceability is recorded in:

- [AGENTFLOW_V1_2_2_RELEASE_PROOF_COMMERCIAL_BOUNDARY_TASKS_V1.md](AGENTFLOW_V1_2_2_RELEASE_PROOF_COMMERCIAL_BOUNDARY_TASKS_V1.md)

## GitHub Milestone Closeout

`v1.2.2` GitHub milestone closeout is part of the release certification boundary.

```text
milestone: v1.2.2
milestoneNumber: 17
state: closed
openIssues: 0
closedIssues: 10
closedAt: 2026-07-06T17:40:17Z
waiver: none
repairRecordedBy: v1.2.3 / V123-003
repairProof: runtime/v122-milestone-closeout-repair.json
```

The release gate must not claim complete V122 traceability unless all V122
issues `#883` through `#892` are closed and the milestone is closed, or unless a
future certification records an explicit waiver reason.

### Post-release Closeout Repair

`v1.2.2` remains the published release baseline. The published tag, GitHub
Release, and source archive are not rewritten.

During `v1.2.3` release proof hardening, the live GitHub milestone was found
open even though all V122 issues were closed. V123-003 repaired the live
milestone state by closing milestone `#17` and added
`runtime/v122-milestone-closeout-repair.json` to the v1.2.3 release gate so
future certification proves the repaired state from live GitHub API evidence.

## Authority Rules

- GitHub issues are release planning and traceability records, not task authority.
- Commercial boundary docs define Product access shape, not Runtime authority.
- License / entitlement cannot bypass Runtime command admission.
- Paid feature gating must happen before Runtime execution, not after Delivery.
- Payment provider state is not part of this release.
- Desktop consumes Runtime API / projection views and must not promote commercial docs into execution authority.

## Release Certification

The release gate for `v1.2.2` must certify:

- all V122 issues are closed before the release certification issue closes;
- workspace and Desktop version metadata match `1.2.2`;
- V121 release proof hardening remains intact;
- Desktop team workflow, commercial boundary, license / entitlement, paid feature, and commercial workflow shape proofs all pass;
- the root certification artifact points at `runtime/v122-release-certification.json`;
- the small certification artifact includes top-level release metadata and primary proof index.

## Next Version

The next release can build on this boundary to continue Product console continuity and commercial preflight work without adding payment processing before Runtime authority is stable.
