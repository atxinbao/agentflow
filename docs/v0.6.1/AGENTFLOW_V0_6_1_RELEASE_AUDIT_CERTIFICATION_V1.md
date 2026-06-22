# AgentFlow v0.6.1 Release Audit Certification V1

日期：2026-06-22
执行者：Codex
状态：Release Audit Certification / v0.6.1 release closeout hotfix / 可作为 release notes 证据入口

## 1. Certification Judgment

`v0.6.1` 修复链已经完成 `v0.6.0` 发布后审计提出的主要问题。

本认证结论：

```text
v0.6.1 可以作为 v0.6.0 之后的 clean remediation release 候选。
```

这里的 clean remediation release 指：

- 版本元数据已经对齐到 `0.6.1`；
- `v0.6.0` 的文档状态已经从 planning draft 收口为 released functional baseline；
- release-gate 不再默认携带旧 `v0.5.1` 版本语义；
- Work Loop closeout 已从 Evidence Gate 升级为 Acceptance Gate；
- Completion Commit 成为 Done 写回前的明确权威边界；
- Done 后只生成 optional audit trigger evaluation，不自动创建 Audit issue。

本认证不代表：

- `v1.0` API 稳定；
- `v0.7.0` Projection Surface 已经完成；
- 自动审计已经进入默认闭环；
- tag / GitHub Release 首次创建后，release event gate 暴露 changelog release entry 缺口。

## 2. Version Facts

当前 main 修复基线：

- Commit：`e1c66d91727910094bb89e66016875f55bb57813`
- Date：`2026-06-22`
- Release candidate：`v0.6.1`

版本元数据：

| File | Expected | Status |
| --- | --- | --- |
| `Cargo.toml` workspace package version | `0.6.1` | aligned |
| `apps/desktop/package.json` | `0.6.1` | aligned |
| `apps/desktop/package-lock.json` root package | `0.6.1` | aligned |
| `apps/desktop/src-tauri/tauri.conf.json` | `0.6.1` | aligned |

## 3. Remediation PR Chain

| Issue | Scope | PR | Merge commit | Status |
| --- | --- | --- | --- | --- |
| [#412](https://github.com/atxinbao/agentflow/issues/412) | V061-001 Release Metadata and Version Alignment | [#433](https://github.com/atxinbao/agentflow/pull/433) | `202eee1ec41e8a31e20d57b02854b12cd0d8ce23` | merged |
| [#413](https://github.com/atxinbao/agentflow/issues/413) | V061-002 v0.6.0 Changelog and Documentation Closeout | [#435](https://github.com/atxinbao/agentflow/pull/435) | `5f0a2562c13e4a5145be68bcf4b389e6200ec672` | merged |
| [#414](https://github.com/atxinbao/agentflow/issues/414) | V061-003 Release Gate Version Certification | [#436](https://github.com/atxinbao/agentflow/pull/436) | `8d58e6b81975627ff3caddaa52c7a22c965a5045` | merged |
| [#415](https://github.com/atxinbao/agentflow/issues/415) | V061-004 Acceptance Gate Contract | [#437](https://github.com/atxinbao/agentflow/pull/437) | `085f885e14c8832a6df3d4a34642c249e70d9794` | merged |
| [#416](https://github.com/atxinbao/agentflow/issues/416) | V061-005 Acceptance Decision Persistence and Failure Reasons | [#438](https://github.com/atxinbao/agentflow/pull/438) | `0e760343002ac867ebe4bd738d8e650e1dc25ac5` | merged |
| [#417](https://github.com/atxinbao/agentflow/issues/417) | V061-006 Completion Commit Authority Boundary | [#439](https://github.com/atxinbao/agentflow/pull/439) | `205e440aff82de1d81d164ce64e3e907c0899050` | merged |
| [#418](https://github.com/atxinbao/agentflow/issues/418) | V061-007 Optional Audit Trigger Evaluation | [#440](https://github.com/atxinbao/agentflow/pull/440) | `e1c66d91727910094bb89e66016875f55bb57813` | merged |

辅助修复：

| PR | Scope | Merge commit | Reason |
| --- | --- | --- | --- |
| [#434](https://github.com/atxinbao/agentflow/pull/434) | Fix release gate workflow metadata parser | `cba5678435773fae531039d2610eb08bcd9f4900` | 修复 #433 合并后暴露的 workflow YAML metadata parser 问题。 |

## 4. Release Gate Evidence

已确认的 gate runs：

| Scope | Run | Commit | Conclusion |
| --- | --- | --- | --- |
| V061-003 PR gate | [27924372027](https://github.com/atxinbao/agentflow/actions/runs/27924372027) | `ed088e0454bdf4bf721b52eba352578fcd21bb0c` | success |
| V061-006 PR gate | [27926338725](https://github.com/atxinbao/agentflow/actions/runs/27926338725) | `92b7c826a9b17aaaa2db55734b2d09030e5b275a` | success |
| V061-007 PR gate | [27926792014](https://github.com/atxinbao/agentflow/actions/runs/27926792014) | `d24e02dca4be97e472ec7e4ce3c0f1fe2a08d3eb` | success |
| Main after V061-007 | [27926959952](https://github.com/atxinbao/agentflow/actions/runs/27926959952) | `e1c66d91727910094bb89e66016875f55bb57813` | success |

Main after V061-007 已通过：

- Rust format check；
- Rust tests；
- Desktop build；
- Release gate E2E；
- Gate artifact upload。

## 5. Local Validation Evidence

V061-007 本地验证：

```text
cargo fmt --all --check
cargo test -p agentflow-event-store
cargo test -p agentflow-cli
cargo test -p agentflow-projection
cargo test --workspace
npm --prefix apps/desktop run build
git diff --check
```

结果：全部通过。

V061-008 本地验证将在本认证文件提交前执行：

```text
cargo fmt --all --check
cargo test --workspace
npm --prefix apps/desktop run build
git diff --check
```

## 6. Closure Mapping

| v0.6.0 audit finding | v0.6.1 closure |
| --- | --- |
| Release metadata still `0.5.1` | V061-001 对齐版本元数据，release-gate 增加版本一致性检查。 |
| v0.6.0 docs still planning draft | V061-002 将 `docs/v0.6.0/**` 收口为 released functional baseline。 |
| release-gate default points to old version | V061-003 从 tag / package metadata 派生 release version，并校验 changelog / release facts。 |
| Evidence Gate is not full Done decision | V061-004 / V061-005 将 Acceptance Gate 和 Acceptance Decision 持久化为 Work Loop closeout 主链。 |
| Completion boundary not explicit | V061-006 引入 `issue.completion.committed`，并让 Done 写回从 Completion Commit 派生。 |
| Audit might be read as default Done side effect | V061-007 引入 `issue.audit.evaluated`，只做建议判断，不创建 audit issue，不改 Done facts。 |
| Certification requires live lookup | V061-008 用本文件记录 PR、commit、gate run 和结论。 |

## 7. Remaining Items

`v0.6.1` 仍然保留以下边界：

- 首次 GitHub tag `v0.6.1` 和 GitHub Release 已创建，但 release event gate 因缺少 `## 0.6.1` changelog entry 失败；
- 本 hotfix 补齐 changelog release entry 后，需要重新创建 `v0.6.1` tag / GitHub Release；
- tag / release event 的 release-gate run 需要在重新发布时再次确认；
- `v0.7.0` 只能在 `v0.6.1` tag / release 成功后进入实现；
- 本认证不新增行业 Pack、OS Console 或自动审计能力。

## 8. Release Recommendation

建议按以下顺序发布：

```text
merge V061-008 certification PR
-> confirm main release-gate success
-> fix changelog release entry if release event gate reports missing 0.6.1 heading
-> recreate tag v0.6.1
-> publish GitHub Release v0.6.1
-> confirm tag / release event release-gate success
```

如果 tag / release event gate 失败，不应把 `v0.6.1` 视为 clean release。
