# 039-055 - AgentFlow Post-Foundation Roadmap V1

创建日期：2026-06-18
执行者：Codex
版本目标：v0.3.0
文档状态：Roadmap / Ready for requirement slicing

## 用户目标

在 `038` 底层 runtime foundation 完成后，为 AgentFlow 规划下一阶段的连续开发主线，并把 `039-055` 明确收口为 `v0.3.0` 的版本目标。

目标不是再补零散功能，而是让后续工作围绕统一路线推进：

1. 先把 Project Brain 正式接入系统；
2. 再把 Task / Project 页面做成真正的主工作台；
3. 再把 Work / Audit / Delivery / Completion 跑成完整闭环；
4. 最后扩展 provider 生态和公开交付能力。

## 背景

当前已经形成以下基线：

- [038-agentflow-project-operating-system-runtime-foundation-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/038-agentflow-project-operating-system-runtime-foundation-v1.md)
- [038-1-agentflow-project-operating-system-runtime-foundation-project-and-issues-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/038-1-agentflow-project-operating-system-runtime-foundation-project-and-issues-v1.md)

`038` 的作用是把底层 authority、workflow、event、projection、provider boundary 定住。
`039` 开始，不再处理“地基是什么”，而是开始规划“地基之上的第一轮正式建设”。

## 一句话目标

以 `038` 的 runtime foundation 为前提，规划 039-055 的完整路线，让 AgentFlow 从任务运行时，继续长成真正的项目运行系统。

## v0.3.0 版本边界

`v0.3.0` 不再是“再补几个页面”。

这个版本的目标是：

1. 让 Project Brain 正式进入 runtime；
2. 让 Goal -> Plan -> Spec -> Task -> Delivery -> Audit -> Completion 这条链真正成立；
3. 让 Task / Project 页面成为用户真正可用的主工作台；
4. 让 Work Flow、Delivery Flow、Audit Flow、Completion Flow 各自独立，但又能串成一个项目闭环；
5. 让 provider 执行能力进入统一 dispatcher / session governance，而不是各自跑各自的逻辑；
6. 让 release 与 external review 拥有正式出口。

换句话说，`v0.3.0` 的完成标准不是“039-055 文件写完”，而是 AgentFlow 已经从单点任务运行器，进入项目级 Agent Operating System 的第一版可用状态。

## 前置条件

039-055 默认建立在以下完成前提之上：

- `038` 方向成立；
- Project / Contract / Workflow / Event / Projection 基线已经固定；
- 不再恢复旧 `input / execute / output` 架构；
- provider session 已被明确视为执行事实而不是业务 authority。

## 范围

本路线图覆盖以下五大主线：

1. Project Brain Runtime 接入
2. Task / Project 工作台产品化
3. Work / Audit / Delivery / Completion 闭环强化
4. Provider / Session 生态扩展
5. Public Delivery / Release / External Review 正式化

## 非目标

- 不重新做一轮底层 authority 宪法讨论
- 不引入新的聊天驱动多 Agent 编排模式
- 不把路线图本身直接写成 `.agentflow/spec/**`
- 不在本文件里直接启动具体 Build Agent 实现

## 阶段总览

### Phase D - Project Brain Activation

包含：

- `039`
- `040`
- `041`

### Phase E - Product Workbench

包含：

- `042`
- `043`
- `044`

### Phase F - Runtime Closure

包含：

- `045`
- `046`
- `047`
- `048`

### Phase G - Provider Expansion

包含：

- `049`
- `050`
- `051`
- `052`

### Phase H - Public Delivery And Review

包含：

- `053`
- `054`
- `055`

## 039-055 Issues

### 039 - Project Brain Runtime Entry

#### 目标

- 把 `GOAL.md / PLAN.md / DECISIONS.md / PROJECT_HEALTH.md` 从文档层正式接入 runtime；
- 让 Goal Agent 成为系统正式上游入口；
- 让 Project Flow 能从 Project Brain 派生下一步动作。

#### 范围

- Project Brain read model
- Goal Agent runtime entry
- Goal / Plan / Decisions snapshot
- Project Brain -> Spec runtime bridge

#### 非目标

- 不直接执行 issue
- 不直接启动 Work Agent

#### 依赖

- `038`

#### 验收

- [ ] Project Brain 可以被 runtime 读取
- [ ] Goal Agent 可以生成下一步建议
- [ ] Project Flow 可以基于 Project Brain 派生后续动作

### 040 - Requirement To Goal / Plan Preview Runtime

#### 目标

- 把原始需求转成 Goal Draft / Plan Draft Preview；
- 保持 preview-first、confirm-first；
- 确认后才能 materialize 成 `SpecProject / SpecIssue`。

#### 范围

- Requirement intake
- Goal Draft Preview
- Plan Draft Preview
- Confirmation gate

#### 非目标

- 不绕过确认直接写入执行合同

#### 依赖

- `039`

#### 验收

- [ ] 原始需求不会直接进入 Work Flow
- [ ] Goal / Plan Preview 有清晰确认门
- [ ] 确认后才进入 Spec materialization

### 041 - Goal Recheck And Completion Decision Runtime

#### 目标

- 项目一轮执行后回到 Goal Agent；
- Goal Agent 判断继续、调整、暂停、接受交付还是进入下一阶段；
- Completion Decision 成为 Project 完成判断的一部分。

#### 范围

- Goal Recheck
- Completion Decision
- Project acceptance / continue proposal

#### 非目标

- 不直接标记项目完成而绕过 Completion

#### 依赖

- `039`
- `040`

#### 验收

- [ ] Delivery 后可以回到 Goal Recheck
- [ ] Completion Decision 有明确输出
- [ ] Project 完成不再只是 issue done 的简单累计

### 042 - Task Page Productization

#### 目标

- 把任务页从“状态调试面板”提升为真正主工作台；
- 右侧状态流、实时事件、历史日志、交付信息更稳定、更可读。

#### 范围

- Task page layout
- State timeline UX
- Current / history / future presentation
- Task detail reading model polish

#### 非目标

- 不重做全局视觉系统

#### 依赖

- `038.5`

#### 验收

- [ ] 任务页足以承载主链路理解
- [ ] 用户能从任务页看懂执行、审计、交付过程

### 043 - Project Page Productization

#### 目标

- 把项目页做成真正的 Project Flow 页面；
- 展示阶段、当前 issue、next actions、blockers、completion hints。

#### 范围

- Project stage summary
- Active issue summary
- Goal Recheck / completion hint
- Loop trigger / project action area

#### 非目标

- 不让项目页直接改底层 authority

#### 依赖

- `039`
- `041`
- `042`

#### 验收

- [ ] 项目页能解释项目当前在哪个阶段
- [ ] 项目页能解释为什么下一条 issue 还没开始

### 044 - Task Tree And Sequence UX

#### 目标

- 优化 project / issue 树形结构；
- 当前、已完成、等待中的展示组织稳定；
- 依赖顺序和执行顺序对用户可见。

#### 范围

- Task tree grouping
- Project / standalone issue grouping
- Dependency order UX
- Sequence visualization

#### 非目标

- 不做复杂图形化 DAG 编辑器

#### 依赖

- `042`
- `043`

#### 验收

- [ ] 左侧任务树排序和组织稳定
- [ ] 用户能看出当前 / 过去 / 未来任务关系

### 045 - Work Loop Hardening

#### 目标

- 强化 Work Loop 的 run lifecycle；
- pause / resume / retry / interruption recovery 更稳定；
- preflight / verify / review 流程更严谨。

#### 范围

- Work workflow runtime
- run lifecycle
- recovery / retry
- interruption handling

#### 非目标

- 不新增更多流程类型

#### 依赖

- `038.3`
- `038.4`
- `038.6`

#### 验收

- [ ] 中断恢复不再打乱业务状态
- [ ] retry / resume 有稳定事件链

### 046 - Audit Flow Productization

#### 目标

- 把 Audit Agent 正式做成独立流程；
- findings / evidence gap / repair recommendation 在产品上清晰可见。

#### 范围

- Audit flow runtime
- Audit result read model
- Audit page / task page summary

#### 非目标

- 不让 Audit Agent 执行修复

#### 依赖

- `045`

#### 验收

- [ ] Audit 作为独立阶段可运行、可展示
- [ ] findings 和 repair recommendation 对用户可读

### 047 - Delivery Flow Productization

#### 目标

- 把 Delivery Agent 正式做成交付整理流程；
- Delivery Summary / Release Notes / Change Summary 有统一出口。

#### 范围

- Delivery flow runtime
- Delivery summary model
- Release / changelog integration

#### 非目标

- 不把 Delivery 重新塞回 Work Loop

#### 依赖

- `046`

#### 验收

- [ ] Delivery 有清晰产物模型
- [ ] 本地事实与公开交付边界稳定

### 048 - Project Completion Runtime

#### 目标

- 把项目完成判断从“issue done 累加”提升为真正 completion runtime；
- 结合 audit、delivery、goal recheck 来决定项目是否 accepted / complete。

#### 范围

- Completion evaluation
- Completion decision
- Project accepted / complete status

#### 非目标

- 不直接把 delivery produced 视为 project done

#### 依赖

- `041`
- `046`
- `047`

#### 验收

- [ ] Project 完成判断具备独立 runtime 规则
- [ ] 项目不会因为单纯 issue done 就直接 complete

### 049 - Codex Provider Hardening

#### 目标

- 把当前 Codex provider 从“能跑”提升到“稳定可长期运行”；
- launch / poll / logs / merge / writeback / recovery 全面收口。

#### 范围

- Codex launch plan
- session lifecycle
- merge proof handling
- resume / recovery

#### 非目标

- 不引入第二套 authority

#### 依赖

- `045`

#### 验收

- [ ] Codex provider 生命周期稳定
- [ ] 失败和恢复路径可解释、可重试

### 050 - Claude Code Provider

#### 目标

- 在不新增第二套 authority 的前提下，接入 Claude Code provider；
- 保持相同 role / workflow / event / projection 模型。

#### 范围

- Claude provider adapter
- launch / poll / cancel / logs
- provider capability integration

#### 非目标

- 不为 Claude 单独发明一套业务流程

#### 依赖

- `049`
- `051`

#### 验收

- [ ] Claude Code 可以作为等价 provider 接入
- [ ] 同一条 issue flow 不因 provider 不同而改变 authority

### 051 - Provider Capability Matrix

#### 目标

- 定义 provider 与 role / skill / flow 的能力矩阵；
- 明确哪些 provider 支持 health / launch / full session / logs / merge。

#### 范围

- provider capability schema
- role / skill / provider compatibility
- dispatcher selection rules

#### 非目标

- 不做 provider 自动抢占调度

#### 依赖

- `038.2`
- `049`

#### 验收

- [ ] provider 能力边界清楚
- [ ] dispatcher 选择规则可解释

### 052 - Agent Session Governance

#### 目标

- 建立 session 管理规则；
- 支持 claim / timeout / cancel / takeover / retry。

#### 范围

- session policy
- timeout policy
- takeover policy
- session-level governance facts

#### 非目标

- 不让 session 本身升级为业务 authority

#### 依赖

- `049`
- `050`
- `051`

#### 验收

- [ ] 多 session 生命周期受统一治理
- [ ] session 失败不会破坏业务状态

### 053 - Public Delivery Standardization

#### 目标

- 统一 PR/MR body、`CHANGELOG.md`、release notes、delivery summary 的公开交付格式；
- 让外部读者不进入 `.agentflow` 也能看懂交付。

#### 范围

- public delivery format
- changelog conventions
- release note template
- delivery summary template

#### 非目标

- 不在本阶段做完整发布平台整合

#### 依赖

- `047`
- `048`

#### 验收

- [ ] 公开交付格式统一
- [ ] 外部读者能理解一次交付结果

### 054 - Release Runtime

#### 目标

- 把 release 变成 Project Completion 之后的正式能力；
- 不再是零散人工动作拼接。

#### 范围

- release runtime
- release gate
- release fact / public note generation

#### 非目标

- 不在本阶段做所有渠道分发自动化

#### 依赖

- `053`

#### 验收

- [ ] release 成为 project completion 后的正式阶段
- [ ] release facts 与 public delivery 一致

### 055 - External Audit And Review Surface

#### 目标

- 为外部 reviewer / 审计者提供统一可读 surface；
- 不需要进入本地 runtime 细节，也能理解目标、范围、交付、证据和风险。

#### 范围

- external review summary
- audit summary surface
- evidence index surface
- review handoff package

#### 非目标

- 不做新的外部门户产品

#### 依赖

- `046`
- `053`
- `054`

#### 验收

- [ ] 外部 reviewer 能读懂一次交付
- [ ] 不需要直接进入本地运行事实目录

## 依赖关系总表

### 强顺序

- `039 -> 040 -> 041`
- `042 -> 043 -> 044`
- `045 -> 046 -> 047 -> 048`
- `049 -> 051 -> 050 -> 052`
- `053 -> 054 -> 055`

### 跨阶段依赖

- `043` 依赖 `041`
- `045` 依赖 `038` runtime foundation 完成
- `048` 依赖 `041`、`046`、`047`
- `053` 依赖 `047`、`048`

## 建议执行顺序

建议后续按以下大顺序推进：

1. `039`
2. `040`
3. `041`
4. `042`
5. `043`
6. `044`
7. `045`
8. `046`
9. `047`
10. `048`
11. `049`
12. `051`
13. `050`
14. `052`
15. `053`
16. `054`
17. `055`

## v0.3.0 版本验收标准

### 1. Project Brain 与上游入口

- [ ] `GOAL.md / PLAN.md / DECISIONS.md / PROJECT_HEALTH.md` 已成为 runtime 可读取、可投影、可解释的正式上游事实。
- [ ] 新需求不会直接落成任务执行合同，必须先经过 Goal / Plan Preview 与确认门。
- [ ] Goal Agent 可以基于 Project Brain 给出继续、调整、暂停、接受交付、进入下一阶段等明确决定。

### 2. Task / Project 成为主工作台

- [ ] Task 页面已经成为主工作台，用户能直接看懂单个任务当前处于什么状态、已经发生过什么、下一步会发生什么。
- [ ] Task 页面右侧不再只是静态字段堆叠，而是以状态流 / 事件流 / 交付流为核心阅读模型。
- [ ] Project 页面已经成为项目级工作台，能解释当前阶段、当前任务、阻断原因、下一步动作和 completion hint。
- [ ] 用户不需要分别跳转旧执行页、旧交付页、旧审计页，主要链路都能在 Task / Project 工作台中理解。

### 3. 四条流程闭环成立

- [ ] Work Flow 负责任务执行本身，状态推进稳定，支持中断恢复、retry、resume。
- [ ] Delivery Flow 负责交付整理，不再混进 Work Flow 内部状态。
- [ ] Audit Flow 作为独立流程存在，能清楚表达 findings、evidence gap、repair recommendation。
- [ ] Completion Flow 不再依赖“issue done 数量累计”，而是结合 delivery、audit、goal recheck 做项目接受判断。

### 4. 统一 runtime / event / projection 语义

- [ ] Project、Task、Delivery、Audit、Completion 的状态变化都能落到统一事件模型与 projection 模型中。
- [ ] Desktop 读取的是 projection / read model，而不是散乱直接读底层事实文件。
- [ ] 任务为什么处于当前状态、项目为什么停在当前阶段，都能通过事件与 projection 解释出来。
- [ ] 不再恢复旧 `input / execute / output` 业务分层，也不再回退到 provider 自带状态作为 authority。

### 5. Provider / Session 体系收口

- [ ] Codex provider 已具备稳定 launch / poll / logs / merge / writeback / recovery 生命周期。
- [ ] Claude Code provider 能在同一 authority、同一 workflow、同一 projection 语义下接入。
- [ ] provider capability matrix、session governance、timeout / takeover / retry 规则已经成立。
- [ ] provider session 仍然只是执行事实，不升级为业务 authority。

### 6. Public Delivery / Release / External Review 出口成立

- [ ] public delivery format 已统一，外部读者不进入本地 runtime 目录，也能读懂一次交付。
- [ ] release 已成为 completion 之后的正式阶段，而不是一串零散人工动作。
- [ ] external reviewer / 审计者有统一 surface 能查看目标、范围、交付、证据和风险。

### 7. 版本级回归标准

- [ ] 至少存在一条真实项目链路，可以从 Requirement -> Goal / Plan Preview -> Spec materialization -> Task execution -> Delivery -> Audit -> Completion -> Release 跑通。
- [ ] Task / Project 主工作台在 Browser Preview 与 Desktop 中都能正确展示关键状态与信息流。
- [ ] `cargo test --workspace`、`npm --prefix apps/desktop run build`、`git diff --check` 通过。
- [ ] 不存在为了兼容旧架构而恢复 retired module、legacy fallback、degraded-as-ready 的行为。

## v0.3.0 不通过条件

出现以下任一情况，`v0.3.0` 不能算验收通过：

- Project Brain 仍停留在文档层，没有正式进入 runtime；
- Task / Project 页面仍然无法承载主链路理解，用户还必须依赖旧执行页或旧交付页理解流程；
- Delivery / Audit / Completion 仍然混在 Work Flow 里，没有形成独立又串联的业务闭环；
- provider 接入后引入第二套 authority 或单独状态体系；
- release / external review 仍然只能依赖本地隐藏事实目录理解；
- 系统仍然无法解释“为什么当前任务是这个状态”或“为什么当前项目停在这个阶段”。

## 验证命令

当前仍为规划文档阶段，最低要求：

- `git diff --check`

进入具体切片实现时，每个切片再单独定义；进入 `v0.3.0` 版本收口时，至少应统一复核：

- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `git diff --check`
- Browser Preview / Desktop smoke：Task / Project / Delivery / Audit / Completion 主链路

## 后续动作

当前文档只定义 039-055 路线图和 issue 草案。

下一步可以继续做两种事：

1. 把每条 039-055 issue 再压成独立需求文档；
2. 先从 `039` 开始生成 `SpecProject Preview + SpecIssue Preview`，正式进入 `.agentflow/spec/**` 写入前预览。
