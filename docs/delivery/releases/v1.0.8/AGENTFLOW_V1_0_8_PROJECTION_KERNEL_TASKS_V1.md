# AgentFlow v1.0.8 Projection Kernel Tasks V1

更新日期：2026-06-30
执行者：Codex

## Goal

`v1.0.8` 将 `v1.0.7` Decision Kernel 之后的 Core facts 投影为稳定、只读、可重放、可刷新、可反馈的 Projection Kernel。

本版本必须保持 Core 行业无关：

```text
Core authority facts
-> Projection Kernel
-> Read Models
-> View Models
-> Freshness / Feedback Receipts
```

Software Dev 的页面、PR、测试和 release 记录只能作为 Reference App mapping，不是 Core Projection authority。

## Authority Boundary

- GitHub issues 是 planning mirror；
- `docs/delivery/releases/v1.0.8/**` 是本版本 release public record；
- `docs/architecture/079` 到 `085` 是 Projection Kernel architecture record；
- `runtime/v107-release-certification.json` 是 Decision Kernel handoff；
- provider session、GitHub issue、PR、test log 和 release note 都不是 Core Projection authority；
- Projection 是 read-only surface，不能写 Spec / Runtime / Evidence / Decision / Completion / Delivery / Audit authority。

## Task Order

### V108-001 Core Projection Kernel Contract

状态：done

目标：

- 定义 Projection Kernel 的只读合同；
- 明确 accepted source refs；
- 明确 read model / view model / freshness 输出；
- 拒绝 Projection 写 authority；
- 对应 release-gate artifact：`runtime/core-projection-kernel-contract.json`；
- 对应 GitHub issue：#713。

依赖：无。

### V108-002 Event Replay and Projection Rebuild

状态：done

目标：

- 定义事件重放和投影重建行为；
- 生成 deterministic rebuild receipt；
- 失败重放必须保留稳定错误原因；
- 对应 release-gate artifacts：`runtime/event-replay-projection-report.json`、`runtime/event-replay-projection-failure-report.json`；
- 对应 GitHub issue：#714。

依赖：#713。

### V108-003 Core Read Model Schema

状态：done

目标：

- 定义 spec / evidence / decision / delivery read model schema；
- 统一 freshness、authority boundary、source refs 和 required fields；
- 负向夹具必须拒绝缺失 source authority 的 read model；
- 对应 release-gate artifact：`runtime/core-read-model-schema.json`；
- 对应 GitHub issue：#715。

依赖：#713、#714。

### V108-004 View Model Contract for Industry Apps

状态：done

目标：

- 定义行业应用只能读取 view model；
- view model 必须从 read model 映射；
- command surface 不能直接读取 authority；
- 对应 release-gate artifact：`runtime/core-view-model-contract.json`；
- 对应 GitHub issue：#716。

依赖：#715。

### V108-005 Pack-specific Projection Mapping Boundary

状态：done

目标：

- 定义 Pack-specific projection mapping boundary；
- Software Dev / UI Design 只能作为 app mapping；
- Pack mapping 不能写 Core authority；
- 对应 release-gate artifact：`pack-projection-readiness.json`；
- 对应 GitHub issue：#717。

依赖：#715、#716。

### V108-006 Invalid / Missing App Definition Handling

状态：done

目标：

- invalid / missing / stale app definition 必须 fail closed；
- disabled command capabilities 必须可见；
- projection 不能补写 app definition authority；
- 对应 release-gate artifact：`pack-projection-readiness.json`；
- 对应 GitHub issue：#718。

依赖：#717。

### V108-007 Projection Surface Read Model and View Stability

状态：done

目标：

- 稳定 projection surface catalog；
- 稳定 read model 到 view model 的结构映射；
- 保证 app console 只能消费投影 surface；
- 对应 release-gate artifacts：`runtime/core-read-model-schema.json`、`runtime/core-view-model-contract.json`；
- 对应 GitHub issue：#719。

依赖：#715、#716、#717。

### V108-008 Projection Replay / Rebuild Runtime Proof

状态：done

目标：

- 证明 replay / rebuild 由事件和 authority facts 驱动；
- 重建结果必须可复现；
- 错误输入必须产生 failure report；
- 对应 release-gate artifacts：`runtime/event-replay-projection-report.json`、`runtime/event-replay-projection-failure-report.json`；
- 对应 GitHub issue：#720。

依赖：#714、#719。

### V108-009 Feedback Surface Projection and Projection Freshness Receipts

状态：done

目标：

- 定义 projection freshness receipt；
- stale / incomplete projection 必须路由到 Spec evolution preview；
- feedback route 不能直接写 authority；
- 对应 release-gate artifact：`runtime/projection-feedback-freshness-receipts.json`；
- 对应 GitHub issue：#721。

依赖：#714、#715、#716、#719、#720。

### V108-010 v1.0.8 Release Certification

状态：done

目标：

- 增加 v1.0.8 release-gate certification artifact；
- 认证 quick-audit artifacts self-contained；
- 认证 manifests truthful；
- 认证 Projection Kernel read-only；
- 认证 replay / rebuild、read / view model、Pack mapping、invalid app definition 和 feedback / freshness surface；
- 对应 release-gate artifact：`runtime/v108-release-certification.json`；
- 对应 GitHub issue：#722。

依赖：#713、#714、#715、#716、#717、#718、#719、#720、#721。

## Dependency Graph

```text
#713
  -> #714
    -> #715
      -> #716
        -> #717
          -> #718
      -> #719
        -> #720
          -> #721

#722 depends on #713-#721.
```

## Release Gate Artifacts

`v1.0.8` release gate 必须至少生成：

```text
runtime/core-projection-kernel-contract.json
runtime/event-replay-projection-report.json
runtime/event-replay-projection-failure-report.json
runtime/core-read-model-schema.json
runtime/core-view-model-contract.json
runtime/projection-feedback-freshness-receipts.json
runtime/projection-feedback-freshness-rust-test.log
runtime/core-decision-projection-read-model.json
runtime/v108-release-certification.json
pack-projection-readiness.json
```

后续版本从 `v1.0.9` Software Dev Reference App certification 开始，不继续扩大 `v1.0.8` Core Projection Kernel scope。
