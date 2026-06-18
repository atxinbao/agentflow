# 058H Release Gate E2E V1

创建日期：2026-06-19
执行者：Codex

## 目的

给 `058H - CI / Release Gate / Real E2E Evidence` 提供一条正式、可复跑、可上传 artifact 的 gate 链路。

这条链路不是 Build Agent 真执行代码的全量回放。
它验证的是：

```text
requirement
-> project intake
-> goal confirm
-> plan confirm
-> materialize
-> completion inspect
-> completion decide
-> release prepare
-> release confirm
-> release publish
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
- `agentflow completion inspect`
- `agentflow completion decide`
- `agentflow release prepare`
- `agentflow release confirm`
- `agentflow release publish`

中间不会修改当前仓库源码。
所有运行时事实都写在临时 fixture workspace。

## 为什么允许脚本种入 done task fixture

这条 gate 的目标是验证：

- project / completion / release 的正式入口是否连通；
- release facts 是否能正确生成；
- public docs 和 external review handoff 是否能落盘；
- CI 是否能把这些证据作为 artifact 上传。

它不是为了替代 Build Agent 真正执行 issue。

因此脚本会在临时 workspace 中：

- 把 materialize 生成的 task 合同切成 `done`
- 写入最小 task projection fixture

这样 release runtime 可以直接进入 completion / release 证明链。

## 产物

脚本会产出：

```text
artifacts/release-gate-e2e/
  cli/
  public/
  runtime/
  summary.json
  summary.md
```

其中关键产物：

### public/

- `CHANGELOG.md`
- `release-notes.md`
- `external-review.md`

这是给外部 reviewer 看的公开结果。

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

## 非目标

- 不验证 Build Agent 真正写代码
- 不替代 provider closeout proof
- 不验证 audit 结果
- 不把临时 fixture 作为真实项目状态写回当前仓库
