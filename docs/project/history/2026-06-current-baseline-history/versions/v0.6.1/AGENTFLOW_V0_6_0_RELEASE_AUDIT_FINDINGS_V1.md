# AgentFlow v0.6.0 Release Audit Findings V1

日期：2026-06-21
执行者：Codex
状态：Release Audit Finding Draft / v0.6.1 输入 / 不授权 Build Agent 执行

## 1. Conclusion

`v0.6.0` 可以保留为 Work Loop Handoff & Controlled Execution 的功能发布。

但它不是 clean stable closeout。

本次审计结论：

```text
功能主线成立。
发布收口不完整。
验收闭环需要在 v0.6.1 正式补强。
```

## 2. Evidence Reviewed

已确认的 release 事实：

- tag：`v0.6.0`
- tag commit：`4af86f6021633f0bc4ae2cd62639a08baa8fa2f8`
- GitHub Release：[AgentFlow v0.6.0](https://github.com/atxinbao/agentflow/releases/tag/v0.6.0)
- Release published at：`2026-06-21T14:47:06Z`
- main release-gate run：[27907474419](https://github.com/atxinbao/agentflow/actions/runs/27907474419)
- tag push release-gate run：[27907724510](https://github.com/atxinbao/agentflow/actions/runs/27907724510)
- release event release-gate run：[27907883094](https://github.com/atxinbao/agentflow/actions/runs/27907883094)
- release-gate conclusion：`success`
- release-gate commit：`4af86f6021633f0bc4ae2cd62639a08baa8fa2f8`

已确认的 V060 merge chain：

| PR | Title | Merge commit |
| --- | --- | --- |
| [#398](https://github.com/atxinbao/agentflow/pull/398) | feat: define work loop filesystem contract | `9f73ca7` |
| [#399](https://github.com/atxinbao/agentflow/pull/399) | feat: hand off spec issues as work commands | `f180ca3` |
| [#400](https://github.com/atxinbao/agentflow/pull/400) | feat: define work action proposal contracts | `5a4bb6b` |
| [#401](https://github.com/atxinbao/agentflow/pull/401) | Add issue preflight runtime gate | `56bafd3` |
| [#402](https://github.com/atxinbao/agentflow/pull/402) | feat: close issue lease and runtime lock loop | `88d2aa5` |
| [#403](https://github.com/atxinbao/agentflow/pull/403) | Add dependency queue report for task loop | `5ee4565` |
| [#404](https://github.com/atxinbao/agentflow/pull/404) | Enforce evidence gate before task completion | `fac3211` |
| [#405](https://github.com/atxinbao/agentflow/pull/405) | feat: enforce work state transitions | `4c3e87f` |
| [#406](https://github.com/atxinbao/agentflow/pull/406) | feat: durable work session recovery | `af24ea8` |
| [#407](https://github.com/atxinbao/agentflow/pull/407) | feat: add work loop event model projection | `4143aed` |
| [#408](https://github.com/atxinbao/agentflow/pull/408) | feat: controlled multi-agent proposal arbitration | `7185556` |
| [#409](https://github.com/atxinbao/agentflow/pull/409) | Add done writeback delivery audit separation acceptance | `70784fb` |
| [#410](https://github.com/atxinbao/agentflow/pull/410) | test: record release gate build agent sessions | `4af86f6` |

## 3. Findings

### P0 - Release version metadata does not match v0.6.0

At tag `v0.6.0`:

- `Cargo.toml` workspace version is `0.5.1`;
- `apps/desktop/package.json` version is `0.5.1`;
- `apps/desktop/src-tauri/tauri.conf.json` version is `0.5.1`;
- `CHANGELOG.md` has no `0.6.0` release entry.

Impact:

- published release identity and build metadata disagree;
- downstream package/version checks cannot trust the release version;
- release notes say `v0.6.0`, but repository metadata still says `0.5.1`.

v0.6.1 action:

- fix all release metadata to `0.6.1`;
- add a formal `0.6.0` closeout entry;
- ensure future tag checks fail when release metadata lags the tag.

### P0 - v0.6.0 was released while v0.5.1 docs still say v0.6.0 must not start

At tag `v0.6.0`:

- `docs/v0.5.1/README.md` says `v0.5.1` is still an unreleased remediation chain;
- the same file says `v0.6.0` cannot directly build on `v0.5.0`;
- `docs/README.md` says do not enter `v0.6.0` before `v0.5.1` completes.

Impact:

- version documentation contradicts the release fact;
- readers cannot tell whether `v0.6.0` is valid or out of policy;
- release governance appears bypassed even if code-level fixes were merged.

v0.6.1 action:

- close out `v0.5.1` as completed or explicitly folded into `v0.6.0`;
- update docs reading order and rules so current release facts and version docs agree.

### P0 - v0.6.0 docs still mark the version as planning draft

At tag `v0.6.0`:

- `docs/v0.6.0/README.md` status is `Version Planning Draft / 开发前置文档 / 不授权 Build Agent 执行`;
- `docs/v0.6.0/AGENTFLOW_V0_6_0_WORK_LOOP_HANDOFF_TASKS_V1.md` also says this is planning and not execution.

Impact:

- release notes claim Work Loop baseline is published;
- version docs still say the work is not authorized for execution;
- the source of truth for `v0.6.0` completion is ambiguous.

v0.6.1 action:

- convert `docs/v0.6.0/**` to released baseline / release closeout;
- preserve the original planning content as historical planning, but do not leave it as the current status.

### P1 - release-gate script still defaults to v0.5.1

At tag `v0.6.0`, `scripts/verify_release_gate.sh` still contains:

```text
ARTIFACT_DIR=artifacts/release-gate-v0.5.1-e2e
RELEASE_VERSION=v0.5.1
v0.5.1-e2e tag and release URL fixtures
```

Impact:

- release-gate can be parameterized, but default evidence still points to old version semantics;
- release-gate output can look valid while still carrying old version identity;
- future releases can repeat the same drift.

v0.6.1 action:

- make release-gate version explicit or derive it from tag context;
- remove hardcoded `v0.5.1-e2e` fixture values;
- fail if requested release version, tag name, package versions, changelog entry and release facts disagree.

### P1 - Acceptance Gate is not yet the formal Work Loop closeout model

At tag `v0.6.0`, code and docs mostly use:

```text
Evidence Gate
```

The product target has since been clarified:

```text
Acceptance Gate = Verification Gate + Evidence Gate + Contract Gate + State Gate
```

Impact:

- v0.6.0 can block no-evidence Done;
- but Done decision is not yet expressed as a full acceptance decision;
- verification, evidence, contract satisfaction and state legality are not first-class closeout components.

v0.6.1 action:

- define and implement Acceptance Gate as the Work Loop Done decision;
- persist acceptance decision and failure reasons;
- keep Audit separate.

### P1 - Completion Commit boundary needs to be made explicit

At tag `v0.6.0`, the intended sequence is present in pieces:

```text
accepted action
-> event store
-> issue / run status
-> projection
-> delivery
```

But version docs still phrase the end as `Done writeback`.

Impact:

- it can be unclear whether projection, status or event is authority;
- completion write order needs one named boundary;
- release closeout should make clear that Projection Refresh is derived, not authority.

v0.6.1 action:

- define `Completion Commit`;
- enforce `Acceptance Decision passed -> accepted action -> Event Store -> status writeback -> Projection Refresh -> Delivery Record`;
- add tests proving projection cannot act as authority.

### P2 - tag-level gates passed, but certification is not easy to audit from committed files

Confirmed gate runs:

- main push gate passed: [27907474419](https://github.com/atxinbao/agentflow/actions/runs/27907474419);
- tag push gate passed: [27907724510](https://github.com/atxinbao/agentflow/actions/runs/27907724510);
- release event gate passed: [27907883094](https://github.com/atxinbao/agentflow/actions/runs/27907883094).

Impact:

- not a gate failure;
- but release certification still requires live GitHub lookup;
- the release tag does not contain a committed certification artifact that records these run IDs and their scope.

v0.6.1 action:

- add a release certification artifact that records main gate run and tag gate run IDs;
- include the certification path in release notes.

## 4. Release Judgment

`v0.6.0` should be treated as:

```text
Functional Release / Not Clean Stable Closeout
```

It is safe to keep the tag and release as the Work Loop functional baseline.

It should not be used as the final clean baseline for future release governance until `v0.6.1` closes the issues above.
