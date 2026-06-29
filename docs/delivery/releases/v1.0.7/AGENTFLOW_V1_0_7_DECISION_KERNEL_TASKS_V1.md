# AgentFlow v1.0.7 Decision Kernel Tasks V1

更新日期：2026-06-29
执行者：Codex

## Goal

`v1.0.7` 将 `v1.0.6` Evidence Kernel 的证据链转成可解释、可追踪、可阻断的 Decision Kernel。

本版本必须保持 Core 行业无关：

```text
Spec + Runtime State + Evidence
-> Decision Input
-> Decision Outcome
-> Completion / Delivery / Optional Audit route
```

Software Dev 的 PR、test log、release record 只能作为 Reference App mapping，不是 Core Decision authority。

## Authority Boundary

- GitHub issues 是 planning mirror；
- `docs/delivery/releases/v1.0.7/**` 是本版本 release public record；
- `docs/architecture/058` 是 Core Evidence / Decision reference model；
- `docs/architecture/069` 是 release provenance tag policy；
- `runtime/v106-release-certification.json` 是 Evidence Kernel handoff；
- provider session、GitHub issue、PR、test log 和 release note 都不是 Core Decision authority；
- Audit 是 optional sidecar，不是默认业务链。

## Task Order

### V107-001 Release Provenance Tag Policy and Evidence Handoff Fix

状态：done

目标：

- 定义 release provenance tag policy；
- 结构化记录 tag object、tag commit、signature status、release run 和 artifact digest；
- 明确 unsigned tag 是 warning-only-visible，不允许静默忽略；
- 绑定 `v1.0.6` Evidence Kernel handoff artifacts；
- 不启动 Decision outcome；
- 对应 GitHub issue：#693。

依赖：无。

### V107-002 Core Decision Model Contract

状态：done

目标：

- 定义行业无关 Decision 模型；
- 明确 Decision 能读取哪些 authority facts；
- 排除 Software Dev 行业词汇进入 Core authority；
- 定义 `decisionId`、`version`、`decidedAt`、`subject`、`inputs`、`outcome`、`reasons`、`writes` 稳定字段；
- 对应 release-gate artifact：`runtime/core-decision-model-contract.json`；
- 对应 GitHub issue：#694。

依赖：#693。

### V107-003 Decision Input Binding

状态：done

目标：

- 绑定 Spec、Runtime State 和 Evidence；
- 绑定 Ontology object；
- Delivery context 只作为可选上下文；
- 缺失输入必须输出稳定阻断原因；
- stale / projection-only / provider-session ref 必须被拒绝；
- 对应 release-gate artifact：`runtime/core-decision-input-binding.json`；
- 对应 GitHub issue：#695。

依赖：#694。

### V107-004 Decision Outcomes and State Transition Semantics

状态：done

目标：

- 定义 accepted / rejected / deferred / blocked / needs-fix outcome；
- 绑定 Core Action / State Semantics；
- 明确 Decision outcome 不能直接写 completed；
- 定义每个 outcome 的 allowed source states、allowed next states 和 reason shape；
- 对应 release-gate artifact：`runtime/core-decision-outcome-transitions.json`；
- 对应 GitHub issue：#696。

依赖：#694、#695。

### V107-005 Failure Reason and Remediation Contract

状态：done

目标：

- 定义 failure reason 和 remediation hint；
- failure reason 必须包含 `reasonCode`、`message`、`authorityRefs`、`missingEvidenceRefs`、`remediationRoute`、`retryEligible` 和 `blocking`；
- remediation route 必须是稳定机器可读值；
- `accepted` outcome 不能挂 failure reason；
- 不能只输出人类文案；
- 对应 release-gate artifact：`runtime/core-decision-failure-reason-remediation.json`；
- 对应 GitHub issue：#697。

依赖：#696。

### V107-006 Evidence-to-Decision Gate

状态：done

目标：

- 把 Evidence Completeness 输出接入 Decision Gate；
- missing / invalid evidence 不能被判断为 Done；
- `complete` evidence 才能得到 `accepted-ready`；
- missing evidence 生成 `deferred` + structured failure reason；
- fake / invalid / wrong-subject evidence 生成 `rejected` + structured failure reason；
- 对应 release-gate artifact：`runtime/core-evidence-to-decision-gate.json`；
- 对应 GitHub issue：#698。

依赖：#695、#696、#697。

### V107-007 Completion Commit Authority Boundary

状态：in-progress

目标：

- 定义 Completion Commit 写入边界；
- Decision 可以授权 completion，但不能让 projection 或 provider session 写 authority；
- 对应 release-gate artifact：`runtime/core-completion-commit-authority.json`；
- 对应 GitHub issue：#699。

依赖：#696、#698。

### V107-008 Delivery Readiness and Optional Audit Trigger Evaluation

状态：planned

目标：

- 定义 delivery readiness；
- audit trigger 是 optional sidecar evaluation；
- 对应 GitHub issue：#700。

依赖：#697、#699。

### V107-009 Decision Projection Read Model and Negative Fixtures

状态：planned

目标：

- 定义 Decision read model；
- projection 只能读取和解释，不写回 authority；
- 对应 GitHub issue：#701。

依赖：#696、#697、#698、#699、#700。

### V107-010 v1.0.7 Release Certification

状态：planned

目标：

- 增加 v1.0.7 release-gate certification artifact；
- 认证 release provenance handoff、Decision model、input binding、outcomes、failure reason、gate、completion boundary、delivery readiness 和 projection；
- 对应 GitHub issue：#702。

依赖：#693、#694、#695、#696、#697、#698、#699、#700、#701。

## Dependency Graph

```text
#693
  -> #694
    -> #695
      -> #696
        -> #697
          -> #698
            -> #699
              -> #700
                -> #701

#702 depends on #693-#701.
```

## Release Gate Artifacts

`v1.0.7` release gate 必须至少生成：

```text
runtime/v107-release-provenance-handoff.json
```

后续 issue 会继续增加 Decision Kernel artifacts。
