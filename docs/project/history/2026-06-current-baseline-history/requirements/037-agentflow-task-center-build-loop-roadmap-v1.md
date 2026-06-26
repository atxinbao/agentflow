# 037 - AgentFlow 任务中心与执行闭环路线图 V1

更新日期：2026-06-17  
执行者：Codex

## 用户目标

把 AgentFlow 当前分散的任务 / 执行 / 交付体验收成一条主链路：

1. 任务页成为唯一主工作台。
2. 用户能直接看到单个任务从开始到结束的状态流和事件流。
3. Build Agent、Project Loop、交付和审计都围绕任务状态推进，不再让用户在多个栏目之间来回切换。
4. 外部 Agent 拉起、执行、验证、PR/MR、Done 写回形成闭环。

## 背景

当前底层模块已经基本完成从旧 `input / execute / output / core / workflow-events` 架构向新任务状态架构的清理，但产品层和运行层仍有几处没有收口：

- 任务页右侧信息还不是真正的状态流主视图。
- 执行和交付信息虽然开始并入任务页，但结构还不够稳定。
- Build Agent 外部拉起已经有基础能力，但会话生命周期、状态写回和中断恢复还需要继续压实。
- Project Loop 已经有底层能力，但产品触发、项目状态派生和顺序推进还没有收成稳定体验。
- 交付和审计的最后收尾展示还没有完全并入任务主链路。

## 一句话目标

用一个项目需求，串起：

`任务中心页面重构 -> Build Agent 执行闭环 -> Project Loop 产品化 -> 交付 / 审计收尾`

## 范围

### 一、任务页重构为状态流工作台

- 左侧保留 project / issue 父子结构。
- issue 列表按执行顺序和依赖顺序展示。
- 右侧改为单个 issue 的状态时间线和事件流。
- 已完成状态展示历史日志。
- 当前状态展示实时事件流。
- 未来状态只展示等待，不展示伪日志。
- 执行和交付信息并入任务详情，不再作为一级业务主链路拆开理解。

### 二、Build Agent loop 执行闭环

- 统一 Build Agent 启动、运行、验证、PR/MR、合并、Done 写回状态。
- 补齐外部 Agent 启动后的 session 生命周期展示。
- 补齐 launch / running / interrupted / resumed / completed / failed 事件写回。
- 补齐中断恢复和重试逻辑。
- 收口 issue 本体、run、projection、index、branch、PR/MR 状态的一致性。

### 三、Project Loop 产品化

- Project Loop 由用户显式触发，不做静默自动触发。
- Project 状态从 issue 状态派生。
- Project Loop 能按依赖顺序找到下一条可执行 issue。
- 完成当前 issue 后，Project Loop 能推进下一条 issue。
- 项目页提供清楚的 loop 触发入口和结果反馈。

### 四、交付 / 审计收尾展示

- 单任务完成后，右侧任务页直接展示交付结果。
- 审计保持独立，不并入 Build Agent 主执行链。
- 任务页能展示 PR/MR 记录、验证证据、合并证明、Done 写回状态。
- 在 in_review 和 done 状态下，交付区域要成为强展示区。

## 非目标

- 不重新引入旧 `input / execute / output` 模块。
- 不恢复旧 fallback / degraded 自动放行逻辑。
- 不把审计重新并入任务执行主链。
- 不在本轮做新的 Release 系统大改。
- 不在本轮扩展多 Agent 并发调度。

## 页面 / 功能

- 任务页：状态流主视图
- 项目页：Project Loop 触发和项目态展示
- 执行链路：Build Agent 外部会话生命周期
- 交付展示：任务内交付记录
- 审计展示：任务完成后的独立审计入口和摘要

## 数据来源

- `docs/requirements/**`
- `.agentflow/spec/projects/**`
- `.agentflow/spec/issues/**`
- `.agentflow/events/**`
- `.agentflow/tasks/<issue-id>/**`
- `.agentflow/projections/**`
- `.agentflow/indexes/**`
- GitHub / GitLab PR/MR 元数据
- 外部 Agent session 状态

## 交互边界

1. 任务页是主入口，但不直接改任务事实源。
2. 任务状态只能由 loop / runtime / event projection 推进。
3. Desktop 只读 projection 和任务产物，不手写状态。
4. Project Loop 负责项目级推进，Build Agent 只负责当前 issue。
5. 审计仍然独立，不因为任务 done 自动进入审计执行链。

## 交付拆解

### Project

- `AF-PROJECT-TASK-CENTER-001` 任务中心工作台重构与执行闭环 V1

### P1：任务页与执行闭环核心

1. `AF-TASK-CENTER-001` 任务页右侧状态时间线基础重构
2. `AF-TASK-CENTER-002` 任务状态节点展开与历史日志展示
3. `AF-TASK-CENTER-003` 任务当前状态实时信息流展示
4. `AF-TASK-CENTER-004` 执行与交付信息并入任务详情页
5. `AF-BUILD-LOOP-001` Build Agent launch 生命周期状态收口
6. `AF-BUILD-LOOP-002` Build Agent 运行时状态写回一致性修复

### P2：外部会话与项目推进

7. `AF-BUILD-LOOP-003` Build Agent 中断恢复与重试机制
8. `AF-BUILD-LOOP-004` Build Agent 任务页实时会话日志接入
9. `AF-PROJECT-LOOP-001` Project Loop 手动触发与结果反馈
10. `AF-PROJECT-LOOP-002` Project 状态从 issue 状态派生统一
11. `AF-PROJECT-LOOP-003` Project Loop 自动推进下一条 issue

### P3：交付 / 审计收尾

12. `AF-DELIVERY-AUDIT-001` 任务完成后的公开交付与审计收尾展示

## 依赖关系

- `AF-TASK-CENTER-002` 依赖 `AF-TASK-CENTER-001`
- `AF-TASK-CENTER-003` 依赖 `AF-TASK-CENTER-001`
- `AF-TASK-CENTER-004` 依赖 `AF-TASK-CENTER-002`、`AF-TASK-CENTER-003`
- `AF-BUILD-LOOP-002` 依赖 `AF-BUILD-LOOP-001`
- `AF-BUILD-LOOP-003` 依赖 `AF-BUILD-LOOP-002`
- `AF-BUILD-LOOP-004` 依赖 `AF-TASK-CENTER-003`、`AF-BUILD-LOOP-002`
- `AF-PROJECT-LOOP-002` 依赖 `AF-PROJECT-LOOP-001`
- `AF-PROJECT-LOOP-003` 依赖 `AF-BUILD-LOOP-003`、`AF-PROJECT-LOOP-002`
- `AF-DELIVERY-AUDIT-001` 依赖 `AF-TASK-CENTER-004`、`AF-BUILD-LOOP-004`、`AF-PROJECT-LOOP-003`

## 验收标准

- [ ] 任务页右侧以状态时间线和事件流为主，不再依赖旧块状说明卡片。
- [ ] 已完成状态能看到历史日志；进行中状态能看到实时事件；未来状态只展示等待。
- [ ] 执行和交付信息被整合进任务页，不再要求用户跳转理解主链路。
- [ ] Build Agent 外部会话生命周期在任务页和执行事实里一致。
- [ ] run / issue / projection / index / branch / PR/MR 状态一致，不再出现显示不同步。
- [ ] Project Loop 由项目页显式触发，能推进当前项目下的 issue 顺序执行。
- [ ] 任务完成后能在任务页看到公开交付记录和 Done 写回结果。
- [ ] 审计保持独立，但任务页能展示其收尾状态或入口。

## 验证命令

```bash
npm --prefix apps/desktop run build
cargo test --workspace
git diff --check
```

## 不做事项

- 不恢复旧架构目录或旧状态机。
- 不把审计重新并入 Build Agent 主 loop。
- 不做新的 GitHub Project / Linear 集成。
- 不做多 Agent 并行抢占执行。
