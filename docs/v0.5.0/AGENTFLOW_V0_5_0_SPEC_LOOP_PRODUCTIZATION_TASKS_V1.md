# AgentFlow v0.5.0 Spec Loop Productization Tasks V1

日期：2026-06-20
执行者：Codex
状态：Version Planning Draft / 开发前置文档 / 不授权 Build Agent 执行

## 1. Purpose

本文档沉淀 AgentFlow `v0.5.0` 的开发任务规划。

`v0.5.0` 的核心目标是把 Spec Loop 产品化：

```text
人类输入
-> 需求清洗
-> 需求分类
-> 上下文解析
-> 边界判断
-> 路由决策
-> 预览生成
-> 人类确认
-> SPEC / Project / Issue 物化
-> Runtime Action Proposal
```

一句话：`v0.5.0` 要让 AgentFlow 稳定理解需求，并把需求转成可确认、可追踪、可进入 Runtime Foundation 的项目合同。

## 2. Baseline

`v0.5.0` 建立在 `v0.4.0` 的 Runtime Foundation 之上。

当前已存在的下层能力：

- `crates/ontology`：定义项目对象世界；
- `crates/action-contract`：定义动作合同；
- `crates/role-policy`：定义角色边界；
- `crates/object-state`：定义对象状态机；
- `crates/action-arbitration`：定义写前裁决；
- `crates/event-store`：沉淀事实事件；
- `crates/projection`：提供只读读模型；
- `crates/runtime-api`：提供 command / query 边界。

`v0.5.0` 不能绕过这些边界。Spec Loop 的输出必须进入 Runtime API / Action Proposal，而不是直接拼底层写逻辑。

## 3. Scope

`v0.5.0` 包含：

- Requirement Intake Normalizer；
- Requirement Classifier；
- Context Resolver；
- Boundary Checker；
- Route Decider；
- Preview Generator；
- Confirmation Gate；
- Spec Materializer；
- Spec-to-Action Proposal Bridge；
- Spec Loop Projection and Acceptance。

## 4. Non-goals

`v0.5.0` 不包含：

- Build Loop 多 Agent 并发执行；
- 对象锁、依赖队列、Evidence Gate 的完整执行产品化；
- 行业客户端壳；
- Domain Pack / Surface Pack / Connector Pack 标准；
- Message Bus；
- 云端部署；
- Desktop OS Console 全量产品化；
- 自动审计；
- Provider supervision 最终产品界面。

## 5. Main Chain

`v0.5.0` 的主链必须保持单线清晰：

```text
Raw Human Request
-> Normalized Requirement
-> Classified Requirement
-> Resolved Context
-> Boundary Result
-> Route Decision
-> Draft Preview
-> Human Confirmation
-> Spec Project / Spec Issue Contract
-> Runtime Action Proposal
```

关键规则：

1. 未确认前，不写正式事实源。
2. Projection 只读，不作为 authority。
3. SPEC / Project / Issue 必须来自确认后的 requirement。
4. Spec Loop 不能直接执行 Build Loop。
5. Audit 仍然保持独立入口。
6. Runtime API 是对下层 Runtime Foundation 的正式边界。

## 6. Issue Preview

| Issue | Title | Dependency | First executable |
| --- | --- | --- | --- |
| `AF-SPEC-001` | Requirement Intake Normalizer | 无 | 是 |
| `AF-SPEC-002` | Requirement Classifier | `AF-SPEC-001` | 否 |
| `AF-SPEC-003` | Context Resolver | `AF-SPEC-001` | 否 |
| `AF-SPEC-004` | Boundary Checker | `AF-SPEC-002`, `AF-SPEC-003` | 否 |
| `AF-SPEC-005` | Route Decider | `AF-SPEC-004` | 否 |
| `AF-SPEC-006` | Preview Generator | `AF-SPEC-005` | 否 |
| `AF-SPEC-007` | Confirmation Gate | `AF-SPEC-006` | 否 |
| `AF-SPEC-008` | Spec Materializer | `AF-SPEC-007` | 否 |
| `AF-SPEC-009` | Spec-to-Action Proposal Bridge | `AF-SPEC-008` | 否 |
| `AF-SPEC-010` | Spec Loop Projection and Acceptance | `AF-SPEC-009` | 否 |

## 7. Development Tasks

### AF-SPEC-001 - Requirement Intake Normalizer

目标：

把人类原始输入清洗成稳定的 `Normalized Requirement`。

范围：

- 保留原始输入；
- 识别 `agentLocale`；
- 提取引用文件、URL、版本号、release、branch、issue、PR；
- 提取显性动作词，例如审计、修复、设计、规划、执行、确认、取消；
- 标记输入来源，例如 human conversation、release link、selected text、file reference；
- 生成后续 Classifier 可消费的结构。

验收标准：

- 同一条人类输入可以稳定生成同一种 normalized record；
- 原始文本不丢失；
- 引用对象可以被后续 Context Resolver 使用；
- 不做需求决策，只做清洗和结构化。

非目标：

- 不判断能不能执行；
- 不写 `docs/requirements/**`；
- 不写 `.agentflow/spec/**`；
- 不启动 Build Agent。

### AF-SPEC-002 - Requirement Classifier

目标：

把 normalized requirement 分类成可路由的需求类型。

范围：

- 支持基础分类：`question`、`research`、`feature`、`bug`、`audit`、`design-only`、`executable issue`、`release`、`maintenance`、`cleanup`；
- 支持多维分类：意图类型、执行权限、事实源影响、风险等级、目标对象、是否需要确认；
- 标记模糊需求和冲突需求；
- 输出分类理由，不能只输出标签。

验收标准：

- 同一需求可以同时带多个维度标签；
- 分类结果能解释为什么；
- `question` 不进入 SPEC；
- `audit` 不混入 Build Loop；
- `design-only` 不误写为可执行代码任务。

非目标：

- 不生成 SPEC；
- 不写正式事实源；
- 不做项目上下文解析。

### AF-SPEC-003 - Context Resolver

目标：

为需求找到当前项目上下文和潜在冲突。

范围：

- 查询相关 `docs/requirements/**`；
- 查询相关 `.agentflow/spec/projects/**` 和 `.agentflow/spec/issues/**`；
- 查询当前 release / tag / branch / PR 事实；
- 查询 Runtime Foundation 文档和 closeout baseline；
- 识别重复需求、冲突边界、过期文档和缺失证据；
- 输出 context summary，供 Boundary Checker 和 Route Decider 使用。

验收标准：

- 能区分当前事实源、历史文档、草案文档；
- 能识别 release 已发布但文档仍标记 draft 的情况；
- 能指出缺失上下文，而不是猜测；
- 不直接修改任何事实源。

非目标：

- 不做分类；
- 不做路由；
- 不自动修复上下文问题。

### AF-SPEC-004 - Boundary Checker

目标：

判断需求能不能写、能不能执行、必须走哪种确认边界。

范围：

- 检查是否允许写 `docs/requirements/**`；
- 检查是否允许写 `.agentflow/spec/**`；
- 检查是否要求先出 SPEC Draft Preview；
- 检查是否误把 audit 当 build；
- 检查是否试图绕过 Runtime API；
- 检查是否试图把 UI/design-only 需求写成执行任务；
- 检查是否需要人类确认。

验收标准：

- 未确认前阻止正式 SPEC 写入；
- Audit 仍然保持独立流程；
- Build Agent 不从聊天直接执行；
- 对每个阻断给出明确原因和允许替代路径。

非目标：

- 不生成 preview；
- 不执行 issue；
- 不创建 PR。

### AF-SPEC-005 - Route Decider

目标：

把需求路由到正确处理路径。

范围：

- 输出路线：`answer-only`、`research-only`、`design-preview`、`requirement-draft`、`spec-preview`、`audit-preview`、`build-issue-preview`、`release-closeout`；
- 根据 Classifier、Context Resolver、Boundary Checker 的结果决策；
- 支持 route reason；
- 支持 route confidence；
- 支持需要澄清时输出最多 3 个问题。

验收标准：

- `question` 不误入 SPEC；
- `feature` 和复杂变更进入 SPEC Preview；
- `audit` 进入 audit preview，不和 build 混合；
- `release` 类需求能进入 release audit / closeout 路线；
- 每次路由都有可解释依据。

非目标：

- 不物化 SPEC；
- 不直接写 issue；
- 不直接执行 Build Loop。

### AF-SPEC-006 - Preview Generator

目标：

生成可供人类确认的预览。

范围：

- 生成 `SPEC Draft Preview`；
- 生成 `Project Preview`；
- 生成 `Issues Preview`；
- 明确目标、范围、非目标、验收标准、风险、依赖顺序；
- 标记 first executable issue candidate；
- 标记 forbidden paths 和 validation direction；
- 支持人类修改、确认、取消。

验收标准：

- Preview 是人类可读文本，不是默认 JSON dump；
- Issues Preview 有清晰依赖；
- 每个 issue 都有目标、范围、验收和非目标；
- 未确认前不写 `docs/requirements/**` 或 `.agentflow/spec/**`。

非目标：

- 不自动写文件；
- 不创建外部 issue；
- 不启动执行。

### AF-SPEC-007 - Confirmation Gate

目标：

把人类确认变成 Spec Loop 的正式写入门禁。

范围：

- 识别 `确认`、`取消`、`修改后再看` 等确认语义；
- 确认前保持 preview-only；
- 确认后允许进入 Spec Materializer；
- 取消后停止写入；
- 修改后生成新 preview 版本；
- 记录确认对象和确认范围。

验收标准：

- 没有确认就不能写正式事实源；
- 确认必须绑定到具体 preview；
- 取消不会留下半成品 spec；
- 修改后的 preview 不覆盖旧确认语义。

非目标：

- 不判断需求分类；
- 不执行任务；
- 不审计 delivery。

### AF-SPEC-008 - Spec Materializer

目标：

把确认后的 preview 物化为正式 requirement、spec project 和 spec issues。

范围：

- 写入正式 `docs/requirements/**`；
- 写入 `.agentflow/spec/projects/**`；
- 写入 `.agentflow/spec/issues/**`；
- 生成稳定 issue IDs；
- 写入 `blockedBy` 依赖；
- 写入 `sourceRequirementId`、`sourceRequirementPath`、`sourceSpecId`、`workflowRef`；
- 写入 `allowedPaths`、`forbiddenPaths`、`validationCommands`、`expectedOutputs`；
- 结构化校验生成结果。

验收标准：

- 只从确认后的 requirement / preview 物化；
- 不从聊天直接生成正式 issue；
- issue 依赖使用 `blockedBy`；
- 不写 legacy `.agentflow/input/**`、`.agentflow/execute/**`、`.agentflow/output/**`、`.agentflow/goal-tree/**`；
- 生成结果可被后续 Build Agent preflight 消费。

非目标：

- 不执行 issue；
- 不创建 PR；
- 不跑 Build Loop。

### AF-SPEC-009 - Spec-to-Action Proposal Bridge

目标：

把确认后的 Spec Loop 输出转换成 Runtime Foundation 可理解的 Action Proposal。

范围：

- 将 Spec Materializer 的结果映射到 Runtime Command；
- 生成 Action Proposal；
- 绑定 Ontology object type；
- 绑定 Action Contract；
- 绑定 Role Policy；
- 绑定 Object State precondition；
- 交给 Arbitration 做写前裁决；
- 不绕过 Runtime API。

验收标准：

- Spec 输出可以进入 `Runtime Command -> Action Proposal -> Arbitration` 链路；
- Action Proposal 有明确 object、action、role、precondition、expected evidence；
- 被拒绝的 proposal 不写 Event Store；
- 被接受的 proposal 才能进入事件沉淀。

非目标：

- 不实现完整多 Agent 调度；
- 不实现 Message Bus；
- 不直接操作 Event Store。

### AF-SPEC-010 - Spec Loop Projection and Acceptance

目标：

为 Spec Loop 提供只读 Projection，并用验收测试证明主链闭环。

范围：

- 增加 Spec Loop read model；
- 展示 intake、classification、context、boundary、route、preview、confirmation、materialization 状态；
- 提供 query surface；
- 增加 acceptance 测试；
- 验证从 raw request 到 Runtime Action Proposal 的完整链路。

验收标准：

- Projection 只读；
- Runtime authority 仍然来自 Event Store 和 spec facts；
- acceptance 覆盖主链：

```text
Raw Human Request
-> Normalized Requirement
-> Classified Requirement
-> Resolved Context
-> Boundary Result
-> Route Decision
-> Preview
-> Confirmation
-> Spec Materialization
-> Runtime Action Proposal
```

- 验收测试不依赖人工点击；
- 不把 UI 状态当事实源。

非目标：

- 不做完整 Desktop OS Console；
- 不做行业客户端；
- 不做云端部署。

## 8. Suggested Milestones

### Milestone 1 - Intake and Classification

包含：

- `AF-SPEC-001`
- `AF-SPEC-002`
- `AF-SPEC-003`

完成后，AgentFlow 能把人类输入变成可解释的需求对象。

### Milestone 2 - Boundary and Routing

包含：

- `AF-SPEC-004`
- `AF-SPEC-005`

完成后，AgentFlow 能判断需求应该回答、研究、设计、进 SPEC、进 audit，还是等待澄清。

### Milestone 3 - Preview and Confirmation

包含：

- `AF-SPEC-006`
- `AF-SPEC-007`

完成后，AgentFlow 能稳定生成预览，并在确认前不写事实源。

### Milestone 4 - Materialization and Runtime Bridge

包含：

- `AF-SPEC-008`
- `AF-SPEC-009`
- `AF-SPEC-010`

完成后，AgentFlow 能把确认后的需求转成正式 SPEC，并接入 Runtime Foundation。

## 9. Completion Criteria

`v0.5.0` 完成时，必须满足：

- 用户输入可以被稳定清洗和分类；
- 系统能识别歧义、冲突、边界和执行风险；
- 系统能生成 `SPEC Draft Preview / Project Preview / Issues Preview`；
- 未确认前不写正式事实源；
- 确认后能生成正式 requirement、spec project、spec issues；
- 确认后的输出能转换成 Runtime Action Proposal；
- Runtime API 边界不被绕过；
- Projection 只读；
- Audit 和 Build 仍然独立；
- acceptance 测试能证明 Spec Loop 主链闭环。

## 10. Release Audit Carry-over

从 `v0.4.0` 审计带入 `v0.5.0` 的修复项：

1. 修正文档状态不一致问题。
   - `docs/v0.4.0/README.md` 仍标记为 `Version Planning Draft / 非执行需求 / 不授权 Build Agent 执行`。
   - `v0.4.0` 已发布，该文档状态应在后续 closeout 中改为 released baseline 或明确归档为 planning source。

2. 明确 Spec Loop 和 Runtime Foundation 的边界。
   - `v0.4.0` 已完成 Runtime Foundation；
   - `v0.5.0` 不重新定义 Runtime Core；
   - `v0.5.0` 只在 Runtime Foundation 上建立需求理解和 SPEC 物化链路。

3. 保持 `work-agent` 主术语。
   - `build-agent` 只作为兼容别名；
   - 新文档、投影和 runtime 输出默认使用 `work-agent` 主命名。

## 11. Next Step

下一步不是直接执行这些任务。

正确顺序是：

```text
SPEC Draft Preview
-> Project Preview
-> Issues Preview
-> Human Confirmation
-> docs/requirements/**
-> .agentflow/spec/projects/**
-> .agentflow/spec/issues/**
```

确认前，本文件只是 `v0.5.0` 的开发前置规划。
