# 009 - Runtime Foundation Closeout Baseline V1

创建日期：2026-06-20
执行者：Codex

## Purpose

本文把 v0.4.0 Runtime Foundation 当前已经收敛出来的底层能力，整理成一份正式 closeout baseline。

这份文档回答三个问题：

1. Runtime Foundation 的主链现在是否已经闭环可验证；
2. 哪些模块已经形成稳定地基；
3. v0.5.0 Spec Loop 和 v0.6.0 Work Loop 应该建立在什么边界之上。

## Closeout 结论

当前 Runtime Foundation 已经具备继续向上迭代的最小闭环：

- Ontology 负责对象世界定义；
- Action Contract 负责动作合同；
- Role Policy 负责角色边界；
- Object State 负责状态机；
- Arbitration 负责写前裁决；
- Event Store 负责事实沉淀；
- Projection 负责只读查询；
- Runtime API 负责统一 command / query 入口。

这一层已经可以支撑：

- v0.5.0 的 Spec Loop；
- v0.6.0 的 Work Loop；
- 后续 Desktop 任务页按状态流和事件流重构。

它还不代表行业产品已经完成，也不代表 Provider orchestration 已经彻底产品化。

## Runtime Main Chain

Runtime Foundation 的主链统一定义为：

```text
Requirement
-> Spec Project / Spec Issue Contract
-> Runtime Command
-> Action Proposal
-> Arbitration Decision
-> Runtime Event
-> Projection Read Model
-> Desktop / CLI / Provider Query Surface
```

收口规则：

1. Contract 是 authority，不是 Agent。
2. Arbitration 是唯一写前裁决口。
3. Event Store 是唯一事实源。
4. Projection 只读，不回写 authority。
5. Runtime API 是对外正式边界，不能绕过它直接拼低层写逻辑。

## Foundation 模块闭环

| 层 | 模块 | 当前职责 | Closeout 结论 |
| --- | --- | --- | --- |
| 世界定义 | `crates/ontology` | 对象、关系、定义版本 | 已稳定 |
| 动作合同 | `crates/action-contract` | action schema、evidence 要求、contract 校验 | 已稳定 |
| 角色边界 | `crates/role-policy` | role capability、handoff、alias | 已稳定 |
| 状态机 | `crates/object-state` | object lifecycle、transition 规则 | 已稳定 |
| 写前裁决 | `crates/action-arbitration` | accepted / rejected / human decision gate | 已稳定 |
| 事实存储 | `crates/event-store` | append-only runtime event authority | 已稳定 |
| 读模型 | `crates/projection` | task/project/audit/delivery 只读模型 | 已稳定 |
| 正式边界 | `crates/runtime-api` | command / query API boundary | 已稳定 |
| 流程执行 | `crates/workflow-core` / `crates/workflow-runtime` | workflow schema 与 runtime transition | 已稳定 |
| 调度 | `crates/task-loop` | issue 启动、依赖顺序、launch 请求 | 已稳定 |
| 外部适配 | `crates/agent-dispatcher` / `crates/mcp` | provider role binding、session launch | 可继续增强 |

## 术语与角色基线

本轮 closeout 后，运行时主术语固定为：

- `work-agent` 是运行时主角色；
- `build-agent` 只保留为 provider-facing / CLI compatibility alias；
- canonical workflow ref 使用 `work-agent.issue-loop@v1`；
- Projection、状态、页面默认展示都围绕 `work-agent` 主命名。

这条规则是 v0.5.0 / v0.6.0 的硬前提，不再回退到双轨命名。

## 边界确认

当前 closeout 明确确认以下边界：

### 已确认

- Projection 不是事实源；
- Event Store 仍然是唯一事实权威；
- Runtime API 不绕过 Arbitration；
- Work Done 不自动触发 Audit；
- Delivery 不是核心写对象；
- Provider session 只是外部执行载体，不拥有业务 authority。

### 不在本轮解决

- 云端多租户；
- 分布式消息总线；
- 行业客户端壳；
- WorkPackage 独立核心对象；
- 自动审计；
- 完整 provider supervision 产品化界面。

## 验证锚点

本轮 closeout 以代码与验收锚点共同确认：

### 关键验证命令

```text
cargo test --workspace --manifest-path Cargo.toml
npm --prefix apps/desktop run build
git diff --check
```

### 关键验收锚点

- `crates/acceptance/src/lib.rs`
  - `runtime_foundation_main_chain_closeout_is_verifiable`
- `crates/runtime-api/src/commands.rs`
  - Runtime command -> arbitration -> response
- `crates/projection/src/query.rs`
  - project / task / runtime health query surface

这意味着 closeout 不是纯文档判断，而是有代码可跑的主链证据。

## 已知延期项

以下内容明确延期到后续版本，不属于 v0.4.0 Foundation closeout：

- Spec Loop 产品化工作台；
- Work Loop 完整 provider 自动执行闭环；
- Delivery public surface 最终产品体验；
- Audit surface 最终产品体验；
- Goal / Plan / Project Brain 的更深层 runtime binding；
- 行业客户端定制壳。

## 对后续版本的要求

### v0.5.0 Spec Loop

必须建立在以下基线上：

- Requirement -> Spec preview -> Spec project / issue materialization
- Runtime API command boundary 不被绕过
- Projection 继续保持只读

### v0.6.0 Work Loop

必须建立在以下基线上：

- Task Loop 仍是 issue 启动 authority
- Work Agent 仍是 runtime 主角色
- Provider 只消费 launch request，不改业务 authority

## Acceptance

本 closeout baseline 成立时，应满足：

- Runtime main chain 有明确单链定义；
- Ontology / Contract / Role / State / Arbitration / Event / Projection / Runtime API 闭环可验证；
- Build / Audit / Delivery 边界没有重新混淆；
- `work-agent` 主命名已经稳定；
- 后续 v0.5.0 / v0.6.0 可以直接以这份 baseline 为起点继续推进。
