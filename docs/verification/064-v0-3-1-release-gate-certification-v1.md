# V0.3.1 Release Gate Certification V1

创建日期：2026-06-19  
执行者：Codex

## 目的

给 `#304 V031-008 Release Gate Certification` 定义一份正式、外部可读、可复查的版本认证面。

这份认证不是新的 runtime。

它只负责回答三件事：

1. `release-gate` 到底有没有跑真实 E2E；
2. 如果失败，失败在什么阶段；
3. 外部 reviewer 能通过哪些产物复查 requirement-to-release 证明链。

## 认证输入

认证只读取 `scripts/verify_release_gate.sh` 产出的 artifacts：

```text
artifacts/release-gate-e2e/
  status.json
  stage-log.jsonl
  summary.json
  summary.md
  certification.json
  certification.md
  public/
  runtime/
  cli/
```

## 正式认证文件

### 1. certification.json

结构化认证事实。

至少包含：

- `releaseVersion`
- `gateWorkflow`
- `gateStatus`
- `failedStage`
- `failureMessage`
- `requirementId`
- `projectId`
- `issueCount`
- `proofChain[]`
- `checklist[]`
- `publicArtifacts[]`
- `runtimeArtifacts[]`

### 2. certification.md

人类直接阅读的认证清单。

必须能直接回答：

- 当前 gate 是否通过；
- 如果失败，失败阶段和原因是什么；
- requirement / project / issues 是否已经进入正式 runtime 链路；
- completion / release / audit 是否留下了正式证明；
- 外部 reviewer 该看哪些公开文件。

## 认证清单

`v0.3.1` 的 release gate 至少要检查：

1. release gate 跑真实 runtime E2E；
2. 失败时能指出具体 runtime stage；
3. public artifacts 已生成：
   - `CHANGELOG.md`
   - `release-notes.md`
   - `external-review.md`
4. runtime artifacts 已生成：
   - `release-facts.json`
   - `external-review-surface.json`
   - `completion-runtime.json`
   - `final-closeout-proof.json`
5. requirement-to-release proof chain 完整。

## GitHub Actions 呈现

`release-gate.yml` 必须：

1. 始终上传 `artifacts/release-gate-e2e/`；
2. 始终把 `certification.md` 追加到 `GITHUB_STEP_SUMMARY`；
3. 如果 `summary.md` 存在，也一并追加；
4. 这样无论成功还是失败，PR / push 页面都能直接看到认证结论。

## 外部审计使用方式

外部 reviewer 不需要进入 `.agentflow` 本地目录。

优先阅读顺序：

1. `certification.md`
2. `summary.md`
3. `public/CHANGELOG.md`
4. `public/release-notes.md`
5. `public/external-review.md`

如果要追溯 runtime facts，再看：

1. `runtime/release-facts.json`
2. `runtime/external-review-surface.json`
3. `runtime/final-closeout-proof.json`
4. `runtime/completion-runtime.json`

## 验证

- `bash scripts/verify_release_gate.sh --artifact-dir artifacts/release-gate-e2e`
- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `cargo fmt --all --check`
- `git diff --check`
