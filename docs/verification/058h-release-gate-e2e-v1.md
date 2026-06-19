# 058H Release Gate E2E V1

创建日期：2026-06-19
执行者：Codex

## 目的

给 `058H - CI / Release Gate / Real E2E Evidence` 提供一条正式、可复跑、可上传 artifact 的 gate 链路。

这条链路不是任意业务需求的全量回放。
它验证的是一条正式、最小、可复跑的 runtime 真链路：

```text
requirement
-> project intake
-> goal confirm
-> plan confirm
-> materialize
-> task-loop
-> build-agent prepare-review
-> build-agent write-closeout-proof
-> build-agent complete
-> completion inspect
-> completion decide
-> release prepare
-> release confirm
-> release record-tag
-> release record-remote
-> release publish
-> audit request-human
-> release publish refresh
```

一句话：

```text
证明 AgentFlow 已经具备从 requirement 到 project-level release 的正式 runtime 入口。
```

## 命令入口

本地和 CI 都统一走：

```bash
bash scripts/verify_release_gate.sh --artifact-dir artifacts/release-gate-e2e
```

GitHub Actions workflow：

```text
.github/workflows/release-gate.yml
```

## Gate 组成

正式 gate 由三段组成：

1. `cargo fmt --all --check`
2. `cargo test --workspace`
3. `npm --prefix apps/desktop run build`
4. `scripts/verify_release_gate.sh`

前 3 段验证代码基础质量。
最后 1 段验证 requirement 到 release runtime 的正式入口。

## E2E Fixture

脚本会在临时目录里创建一个最小 requirement：

```text
docs/requirements/058h-release-gate-e2e.md
```

然后按 CLI 正式入口推进：

- `agentflow project intake`
- `agentflow project confirm-goal`
- `agentflow project confirm-plan`
- `agentflow project materialize`
- `agentflow task-loop tick`
- `agentflow build-agent prepare-review`
- `agentflow build-agent write-closeout-proof`
- `agentflow build-agent complete`
- `agentflow completion inspect`
- `agentflow completion decide`
- `agentflow release prepare`
- `agentflow release confirm`
- `agentflow release record-tag`
- `agentflow release record-remote`
- `agentflow release publish`
- `agentflow audit request-human`

中间不会修改当前仓库源码。
所有运行时事实都写在临时 fixture workspace。

## E2E 如何推进任务

这条 gate 不再直接改：

- `.agentflow/spec/issues/**`
- `.agentflow/projections/**`

脚本会先 materialize 一个最小 project 和 2 条 issue，再通过：

- `task-loop tick`
- `build-agent prepare-review`
- `build-agent write-closeout-proof`
- `build-agent complete`

把两条 issue 沿正式 runtime 推进到 `done`，然后再进入 completion / release / audit 证明链。

## 产物

脚本会产出：

```text
artifacts/release-gate-e2e/
  cli/
  public/
  runtime/
  certification.json
  certification.md
  summary.json
  summary.md
```

其中关键产物：

### public/

- `CHANGELOG.md`
- `release-notes.md`
- `external-review.md`

这是给外部 reviewer 看的公开结果。

### certification

- `certification.json`
- `certification.md`

这是本轮 gate 的正式认证面。

必须表达：

- gate 当前是 `passed` 还是 `failed`
- 如果失败，失败阶段是哪一环
- requirement 到 release 的证明链是否齐全
- 外部 reviewer 能读取哪些公开证据

### runtime/

- `release-facts.json`
- `external-review-surface.json`
- `release-index.json`
- `external-review-index.json`

这是 release runtime 自己写出来的 authoritative facts。

### cli/

- `artifacts-intake.json`
- `artifacts-goal.json`
- `artifacts-plan.json`
- `artifacts-materialize.json`
- `artifacts-completion-inspect.json`
- `artifacts-completion-decide.json`
- `artifacts-release-prepare.json`
- `artifacts-release-confirm.json`
- `artifacts-release-publish.json`
- `artifacts-release-summary.txt`

这是整条链路的原始命令输出。

## 验收

- `releaseState == published`
- `gateStatus == ready`
- `completionState == accepted`
- `CHANGELOG.md` 已生成
- `docs/release-notes/<project-id>.md` 已生成
- `docs/reviews/<project-id>.md` 已生成
- `.agentflow/release/projects/<project-id>.json` 已生成
- `.agentflow/release/reviews/<project-id>.json` 已生成
- `artifacts/release-gate-e2e/certification.md` 已生成
- gate 失败时 `summary.md` / `certification.md` 能指出失败阶段

## 非目标

- 不验证任意真实业务需求的代码实现
- 不替代 provider closeout proof 真实联网查询
- 不把 audit 结果当 release publish 的自动前置
- 不把临时 fixture 作为真实项目状态写回当前仓库
