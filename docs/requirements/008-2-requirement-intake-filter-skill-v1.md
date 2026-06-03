# 008.2 - Requirement Intake Filter Skill V1

创建日期：2026-06-03
执行者：Codex

## 用户目标

当前 AgentFlow 已经完成：

```text
008 - Agent Working Manual Bootstrap V1
008.1 - Agent Working Manual Health Polish
```

AgentFlow 已经可以在本地项目中接管：

```text
AGENT.MD
.agentflow/define/agent/Agentflow.md
.agentflow/define/agent/skills/**
.agentflow/define/agent/skills-lock.json
```

但现在还缺一个关键能力：

```text
在人类输入需求之后，Agent 不能直接写 OpenSpec，也不能直接生成 Goal Tree。
必须先对需求做过滤、澄清、补全和准入判断。
```

本需求目标是：

```text
新增 AgentFlow 原生 Requirement Intake Filter Skill。
```

大白话：

> 用户经常会用很口语、很模糊、很跳跃的方式描述需求。
> Agent 不能听到一句话就开始写 OpenSpec，更不能直接写 Goal Tree。
> 它必须先判断：这到底是不是需求？够不够清楚？缺什么信息？有没有越界？是否可以进入 OpenSpec？
> 这个前置过滤器，就是 AgentFlow 的需求入口守门员。

---

## 与 Lyra 的关系

本需求参考 Lyra 类方法论的思想：

```text
先拆解输入
再诊断模糊点
再补齐上下文
最后输出结构化结果
```

但本需求不引入 Lyra 原文，不复制 Lyra 提示词，不保留 Lyra 名称，也不把它作为外部依赖。

原因：

```text
Lyra 原本是 Prompt Optimizer。
AgentFlow 需要的是 Requirement Intake Filter。
```

两者目标不同。

AgentFlow 的需求过滤器不是为了“优化提示词”，而是为了：

```text
判断用户输入能不能进入 OpenSpec。
```

因此，本需求只借鉴思想，重新写成 AgentFlow 原生 skill。

---

## 一句话定义

> **Requirement Intake Filter Skill 是 AgentFlow 在 OpenSpec Authoring 之前的需求准入过滤器。它负责把人类会话输入拆解成结构化 Intake Result，并决定下一步是进入 OpenSpec、继续澄清、只回答、阻断，还是延后。**

---

## 当前 Agent 工作流调整

当前 AgentFlow Agent 工作流应从：

```text
Conversation
→ Request triage
→ OpenSpec Draft
→ Human confirmation
→ Approved OpenSpec
→ Goal Tree materialization
→ Future AgentRun
```

调整为：

```text
Conversation
→ Request triage
→ Requirement intake filter
→ OpenSpec Draft Preview
→ Human confirmation
→ Approved OpenSpec
→ Goal Tree materialization
→ Future AgentRun
```

其中：

```text
request-triage
= 粗分类：这是 bug / feature / refactor / docs / research / cleanup / question？

requirement-intake-filter
= 深过滤：这个需求够不够清楚？能不能进入 OpenSpec？

openspec-authoring
= 在通过过滤后，生成 OpenSpec Draft Preview
```

大白话：

```text
request-triage 是分诊
requirement-intake-filter 是问诊
openspec-authoring 是写规格草案
```

---

# 1. 范围

本需求包含 8 个能力：

```text
1. 新增 requirement-intake-filter skill
2. 更新 Agentflow.md 工作流程
3. 更新 AGENT.MD 入口规则
4. 更新 skills-lock.json 模板
5. 更新 Agent Manual validation / repair
6. 新增 Intake Result 输出规范
7. 新增测试覆盖 skill 写入、hash、repair
8. 更新 requirements / verification 文档
```

---

# 2. 非目标

本需求不做以下事情：

```text
不复制 Lyra 原文
不新增 Lyra skill 名称
不调用模型
不生成 OpenSpec 文件
不写 .agentflow/define/openspec/**
不生成 Goal Tree
不启动 AgentRun
不执行项目命令
不写用户源码
不创建 PR
不创建远程 Issue
不修改 Project File Reader
不修改 Graph
不修改 Goal Tree
不恢复旧 CLI 写命令
不接外部 skills marketplace
```

---

# 3. 文件结构变更

当前 Agent Manual skills 目录为：

```text
.agentflow/define/agent/skills/
├── request-triage/
├── openspec-authoring/
├── goal-tree-materialization/
├── boundary-check/
└── validation/
```

本需求新增：

```text
.agentflow/define/agent/skills/requirement-intake-filter/
└── SKILL.md
```

更新后结构：

```text
.agentflow/define/agent/
├── Agentflow.md
├── skills-lock.json
├── skills/
│   ├── request-triage/
│   │   └── SKILL.md
│   ├── requirement-intake-filter/
│   │   └── SKILL.md
│   ├── openspec-authoring/
│   │   └── SKILL.md
│   ├── goal-tree-materialization/
│   │   └── SKILL.md
│   ├── boundary-check/
│   │   └── SKILL.md
│   └── validation/
│       └── SKILL.md
└── state/
    ├── bootstrap.json
    └── validation.json
```

---

# 4. Requirement Intake Filter Skill

路径：

```text
.agentflow/define/agent/skills/requirement-intake-filter/SKILL.md
```

## 4.1 Skill 目标

该 skill 负责：

```text
在人类输入和 OpenSpec Draft 之间做需求准入判断。
```

它必须回答：

```text
这是不是一个需求？
是什么类型的需求？
是否足够清楚？
是否缺上下文？
是否有可测试验收标准？
是否有明确非目标？
是否存在越界风险？
是否能进入 OpenSpec？
下一步应该做什么？
```

---

## 4.2 Skill 输入

输入来自：

```text
人类会话内容
当前 Project Workspace 状态
Graph 状态
Project File Reader 能力
已有 Goal Tree snapshot
已有 OpenSpec drafts / approvals
Agentflow.md
skills-lock.json
request-triage 结果
```

V1 不要求系统自动读取全部上下文，但 skill 文档必须要求 Agent 在判断前优先读取这些上下文。

---

## 4.3 Skill 输出

Skill 不输出 OpenSpec。

Skill 输出：

```text
Requirement Intake Result
```

建议 JSON 结构：

```json
{
  "version": "requirement-intake-filter.v1",
  "status": "needs-clarification",
  "requestType": "feature",
  "summary": "用户希望为 AgentFlow 增加需求过滤能力。",
  "knowns": [],
  "unknowns": [],
  "clarifyingQuestions": [],
  "scopeCandidates": [],
  "nonGoalCandidates": [],
  "acceptanceCriteriaCandidates": [],
  "boundaryRisks": [],
  "recommendedNextStep": "ask-clarifying-questions"
}
```

状态枚举：

```text
ready-for-openspec
needs-clarification
answer-only
blocked-by-boundary
defer
```

---

# 5. Intake 状态定义

## 5.1 ready-for-openspec

表示：

```text
需求已经足够清楚，可以进入 OpenSpec Draft Preview。
```

条件至少包括：

```text
目标明确
范围初步明确
非目标初步明确
验收标准可形成
没有明显越界
不需要立即执行命令
不需要立即写源码
```

推荐下一步：

```text
generate-openspec-draft-preview
```

---

## 5.2 needs-clarification

表示：

```text
需求方向明确，但缺少关键上下文。
```

常见原因：

```text
目标用户不清楚
范围不清楚
验收标准不清楚
优先级不清楚
涉及模块不清楚
是否允许写入不清楚
```

要求：

```text
Agent 最多提出 3 个关键澄清问题。
不要一次问太多。
不要写事实源。
```

推荐下一步：

```text
ask-clarifying-questions
```

---

## 5.3 answer-only

表示：

```text
这不是需求，只是问题 / 解释 / 咨询。
```

要求：

```text
直接回答
不写 OpenSpec
不写 Goal Tree
不写文件
```

推荐下一步：

```text
answer-in-conversation
```

---

## 5.4 blocked-by-boundary

表示：

```text
用户请求触发当前阶段禁止行为。
```

例如：

```text
要求直接改源码
要求直接运行测试
要求直接启动 AgentRun
要求直接写 Goal Tree
要求绕过 OpenSpec
要求创建 PR / 远程 Issue
```

要求：

```text
停止
说明被哪个规则阻断
提示当前允许的替代流程
```

推荐下一步：

```text
explain-boundary-and-stop
```

---

## 5.5 defer

表示：

```text
需求目前无法处理，需要等后续能力。
```

例如：

```text
依赖 AgentRun
依赖 OpenSpec Authoring 尚未实现
依赖外部集成
依赖多人协作
```

推荐下一步：

```text
record-as-future-capability
```

V1 不写 future backlog，先只在会话中说明。

---

# 6. Skill 文档内容要求

`requirement-intake-filter/SKILL.md` 至少包含：

```text
Purpose
Required Reading
Input Sources
Output Contract
Status Definitions
Filtering Steps
Clarification Rules
Boundary Checks
Examples
Non-goals
```

---

## 6.1 Filtering Steps

建议步骤：

```text
1. Restate the user request in one sentence.
2. Classify request type.
3. Extract known facts.
4. Identify missing facts.
5. Identify scope candidates.
6. Identify non-goal candidates.
7. Draft acceptance criteria candidates.
8. Check AgentFlow boundaries.
9. Decide intake status.
10. Return Requirement Intake Result.
```

---

## 6.2 Clarification Rules

规则：

```text
如果缺信息，最多问 3 个问题。
问题必须具体。
问题必须服务 OpenSpec。
不要问已经能从项目上下文读取的信息。
不要为了完美信息过度追问。
```

---

## 6.3 Boundary Checks

必须检查：

```text
是否要写用户源码？
是否要执行命令？
是否要写 OpenSpec 事实源？
是否要写 Goal Tree？
是否已有 approved OpenSpec？
是否要启动 AgentRun？
是否要创建远程对象？
是否涉及 legacy 路径？
是否绕过 AGENT.MD / Agentflow.md / skills-lock？
```

---

# 7. AGENT.MD 更新

当前 AGENT.MD 中已有：

```text
Every Agent MUST read and follow:
1. Agentflow.md
2. skills-lock.json
3. All skills referenced by skills-lock.json
```

本需求需要补一条硬规则：

```md
- Before producing an OpenSpec Draft, every Agent MUST run the requirement-intake-filter skill.
```

完整规则中应体现：

```text
Conversation
→ Request triage
→ Requirement intake filter
→ OpenSpec Draft
→ Human confirmation
→ Approved OpenSpec
→ Goal Tree materialization
→ Future AgentRun
```

---

# 8. Agentflow.md 更新

`Agentflow.md` 中 Required Workflow 从：

```text
Conversation
→ Request triage
→ OpenSpec Draft
→ Human confirmation
→ Approved OpenSpec
→ Goal Tree materialization
→ Future AgentRun
```

更新为：

```text
Conversation
→ Request triage
→ Requirement intake filter
→ OpenSpec Draft Preview
→ Human confirmation
→ Approved OpenSpec
→ Goal Tree materialization
→ Future AgentRun
```

同时在 OpenSpec First Rule 中加入：

```text
Before OpenSpec Authoring, the Agent must produce a Requirement Intake Result.
Only `ready-for-openspec` may proceed to OpenSpec Draft Preview.
```

---

# 9. skills-lock.json 更新

`skills-lock.json` 中新增：

```json
"requirement-intake-filter": {
  "version": "v1",
  "path": ".agentflow/define/agent/skills/requirement-intake-filter/SKILL.md",
  "hash": "<sha256>"
}
```

skillCount 从：

```text
5
```

变成：

```text
6
```

所有 hash 必须随模板更新重新计算。

---

# 10. Agent Manual validation / repair 更新

需要更新：

```text
crates/agent-manual/src/templates.rs
crates/agent-manual/src/lock.rs
crates/agent-manual/src/validate.rs
crates/agent-manual/src/repair.rs
crates/agent-manual/src/lib.rs tests
```

## 10.1 templates.rs

新增第 6 个 skill template：

```text
requirement-intake-filter
```

并更新 AGENT.MD / Agentflow.md 文案。

## 10.2 lock.rs

expected skills lock 自动包含第 6 个 skill。

## 10.3 validate.rs

validate 必须检查第 6 个 skill：

```text
exists
hashMatches
version
```

如果缺失：

```text
ready=false
errors 包含 Skill requirement-intake-filter is missing
```

如果 hash mismatch：

```text
ready=false
errors 包含 Skill requirement-intake-filter hash mismatch
```

## 10.4 repair.rs

repair 必须能写入 / 恢复第 6 个 skill。

---

# 11. 测试要求

新增或更新测试：

```text
prepare_creates_agent_manual_tree
validate_detects_skill_hash_mismatch_and_repair_restores_it
skills-lock skill count
```

必须新增断言：

```rust
assert!(dir
    .path()
    .join(".agentflow/define/agent/skills/requirement-intake-filter/SKILL.md")
    .is_file());
```

并断言：

```text
skillsLock.skillCount == 6
```

新增测试：

```text
validate_detects_missing_requirement_intake_filter_skill
```

场景：

```text
prepare
删除 requirement-intake-filter/SKILL.md
validate -> ready=false
repair -> ready=true
文件恢复
```

---

# 12. Documentation 更新

需要新增：

```text
docs/requirements/008-2-requirement-intake-filter-skill-v1.md
```

并更新：

```text
docs/requirements/README.md
docs/requirements/next-requirements.md
verification.md
```

---

# 13. Desktop / Browser Preview

## 13.1 Browser Preview mock

Browser Preview Agent Environment Status 里 skillCount 要从：

```text
5
```

改成：

```text
6
```

skills list 增加：

```text
requirement-intake-filter
```

## 13.2 Desktop 状态通道

工作手册状态无需新增 UI。

但状态 metrics 应自动显示：

```text
Skills 6/6
```

---

# 14. 允许写入路径

本需求仍只允许写：

```text
<project-root>/AGENT.MD
.agentflow/define/agent/**
.agentflow/output/backup/agent-md/**
.agentflow/output/logs/**
```

不允许写：

```text
用户源码
OpenSpec changes
Goal Tree
AgentRun
旧 .agentflow/issues
旧 .agentflow/runs
旧 .agentflow/evidence
旧 .agentflow/reviews
旧 .agentflow/updates
旧 .agentflow/views
.gitignore
远程服务
```

---

# 15. 开发切片

## Slice 1：新增 skill 模板

目标：

```text
新增 requirement-intake-filter/SKILL.md 模板
更新 skill_templates()
```

验收：

```text
prepare 后文件存在
```

---

## Slice 2：更新 Agentflow.md / AGENT.MD

目标：

```text
加入 requirement-intake-filter 流程和硬规则
```

验收：

```text
AGENT.MD 内容包含 requirement-intake-filter
Agentflow.md Required Workflow 包含 requirement-intake-filter
```

---

## Slice 3：更新 lock / validate / repair

目标：

```text
skills-lock 包含第 6 个 skill
validate 能检测缺失和 mismatch
repair 能恢复
```

验收：

```text
cargo test -p agentflow-agent-manual
```

---

## Slice 4：Browser Preview + docs

目标：

```text
Browser Preview skillCount 6
requirements / verification 更新
```

验收：

```text
npm build 通过
```

---

# 16. 总验收标准

```text
- [ ] 新增 docs/requirements/008-2-requirement-intake-filter-skill-v1.md。
- [ ] 新增 requirement-intake-filter/SKILL.md 模板。
- [ ] AGENT.MD 模板包含 requirement-intake-filter 硬规则。
- [ ] Agentflow.md 工作流包含 Requirement intake filter。
- [ ] skills-lock.json 包含 requirement-intake-filter。
- [ ] skillCount 从 5 变成 6。
- [ ] prepare 能写入第 6 个 skill。
- [ ] validate 能检测第 6 个 skill 缺失。
- [ ] validate 能检测第 6 个 skill hash mismatch。
- [ ] repair 能恢复第 6 个 skill。
- [ ] Browser Preview Agent Manual 显示 Skills 6/6。
- [ ] 不复制 Lyra 原文。
- [ ] 不使用 Lyra 名称。
- [ ] 不写 OpenSpec。
- [ ] 不写 Goal Tree。
- [ ] 不启动 Agent。
- [ ] 不执行项目命令。
- [ ] 不调用模型。
- [ ] 不写用户源码。
- [ ] cargo fmt --check 通过。
- [ ] cargo test -p agentflow-agent-manual 通过。
- [ ] cargo test 通过。
- [ ] npm --prefix apps/desktop run build 通过。
- [ ] git diff --check 通过。
```

---

# 17. 验证命令

必须执行：

```bash
cargo fmt --check
cargo test -p agentflow-agent-manual
cargo test
npm --prefix apps/desktop run build
git diff --check
```

如果改 Desktop Tauri：

```bash
cargo test -p agentflow-desktop
```

---

# 18. PR 说明要求

PR 描述必须包含：

```text
1. 为什么新增 requirement-intake-filter。
2. 它与 request-triage / openspec-authoring 的关系。
3. 明确说明没有复制 Lyra 原文。
4. skills-lock skill count 从 5 变成 6。
5. 新增 skill 的 hash / validation / repair 行为。
6. Browser Preview 是否更新为 6/6。
7. 本次没有写 OpenSpec。
8. 本次没有写 Goal Tree。
9. 本次没有启动 Agent / 执行命令 / 调用模型 / 写源码。
10. 验证命令和结果。
```

---

# 19. Codex 执行指令

```md
请执行 008.2 - Requirement Intake Filter Skill V1。

目标：
在 Agent Working Manual 中新增 AgentFlow 原生 requirement-intake-filter skill。该 skill 位于 request-triage 和 openspec-authoring 之间，用于在写 OpenSpec Draft 前过滤、澄清、补全和判断需求是否准入。

必须遵守：
1. 不复制 Lyra 原文。
2. 不使用 Lyra 名称。
3. 只借鉴“拆解 / 诊断 / 补齐 / 交付”的思想，改写成 AgentFlow 原生需求过滤器。
4. 不生成 OpenSpec。
5. 不写 Goal Tree。
6. 不启动 Agent。
7. 不执行项目命令。
8. 不调用模型。
9. 不写用户源码。
10. 不写旧 .agentflow paths。
11. 保持 Agent Manual 写入边界不变。

实现范围：
- 新增 docs/requirements/008-2-requirement-intake-filter-skill-v1.md。
- 更新 crates/agent-manual/src/templates.rs。
- 新增 requirement-intake-filter/SKILL.md 模板。
- 更新 AGENT.MD 模板，加入 requirement-intake-filter 硬规则。
- 更新 Agentflow.md 模板，加入 Requirement intake filter 流程。
- 更新 skills-lock expected template，使 skillCount 从 5 变成 6。
- 更新 validate / repair 测试。
- 更新 Browser Preview Agent Manual mock 为 Skills 6/6。
- 更新 requirements index 和 verification。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-agent-manual
- cargo test
- npm --prefix apps/desktop run build
- git diff --check
```

---

# 20. 完成定义

本需求完成后，AgentFlow 的 Agent 工作流应变成：

```text
Conversation
→ Request triage
→ Requirement intake filter
→ OpenSpec Draft Preview
→ Human confirmation
→ Approved OpenSpec
→ Goal Tree materialization
→ Future AgentRun
```

Agent 在进入 OpenSpec 之前，必须先输出 Requirement Intake Result：

```text
ready-for-openspec
needs-clarification
answer-only
blocked-by-boundary
defer
```

最终一句话：

> **Requirement Intake Filter 是 OpenSpec 的前置守门员：用户输入先经过过滤和澄清，确认足够清楚后，才允许进入 OpenSpec Draft。**
