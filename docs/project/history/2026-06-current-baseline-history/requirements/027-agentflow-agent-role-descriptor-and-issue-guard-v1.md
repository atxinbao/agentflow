下面是审核版开发需求。建议文件名：

docs/requirements/027-agentflow-agent-role-descriptor-and-issue-guard-v1.md

⸻

027 - AgentFlow Agent Role Descriptor & Issue Guard V1

1. 背景

AgentFlow 现在有 3 个 Agent 角色：

Spec Agent
Build Agent
Audit Agent

但现在的问题是：

如何准确判断一个 Agent 是什么角色？
如何防止 Spec Agent 去做 Build Agent 的活？
如何防止 Audit Agent 去做 Build Agent 的活？
如何防止 Build Agent 去做 Audit Agent 的活？

当前版本 AgentFlow App 不直接调用 Codex / Claude / DeepSeek API。
Agent 实际运行在外部聊天 / Codex 环境里。

所以不能靠：

模型自己说“我是 Audit Agent”

来判断身份。

必须靠 AgentFlow 自己生成的：

角色描述
任务类型
任务包
写回文件
状态校验

来判断。

一句话：

Agent 的身份不能靠它自称，要靠 AgentFlow 发给它的任务包和它写回的结构化结果来验证。

⸻

2. 目标

本需求要实现：

1. 给 Agent 定义明确角色描述。
2. 给 Issue 定义明确分类。
3. 给 Issue 绑定唯一可执行 Agent。
4. 给任务包写入 requiredAgentRole。
5. 给写回结果做角色校验。
6. 不允许错误 Agent 执行错误类型的 Issue。
7. Release 后生成 Audit Issue，只能由 Audit Agent 执行。

⸻

3. 核心原则

3.1 Agent 身份必须显式声明

不要靠推断。

必须有结构化字段：

agentRole

取值：

spec-agent
build-agent
audit-agent

3.2 Issue 必须有分类

新增：

issueCategory

取值：

spec
audit

说明：

spec = 从已确认 SPEC 派生出来的开发 / 修复 / 文档 / 验证任务
audit = 审计 Release / Delivery / Evidence 的任务

3.3 Issue 必须绑定可执行角色

新增：

requiredAgentRole

规则：

issueCategory = spec  → requiredAgentRole = build-agent
issueCategory = audit → requiredAgentRole = audit-agent

Spec Agent 不执行 Issue。

Spec Agent 只做：

需求确认
整理规格
生成 Issue

⸻

4. Agent 角色定义

4.1 Spec Agent

职责：

确认需求
追问缺失信息
整理 SPEC
生成 Issue

允许写入：

.agentflow/input/intake/**
.agentflow/input/specs/drafts/**
.agentflow/input/specs/approved/**
.agentflow/input/issues/**

禁止：

不执行代码
不生成 patch
不写 execute run
不写 release delivery
不写 audit report
不修改用户源码

4.2 Build Agent

职责：

执行 spec issue
生成 Codex 任务包
检查写回
生成 evidence
生成 release delivery

允许处理：

issueCategory = spec
requiredAgentRole = build-agent

允许写入：

.agentflow/execute/**
.agentflow/output/evidence/**
.agentflow/output/release/**
.agentflow/state/events/**

禁止：

不处理 audit issue
不写 audit report
不写 findings
不写 evidence-map
不写 traceability
不审计 release

4.3 Audit Agent

职责：

执行 audit issue
读取 SPEC / Issue / Evidence / Release
生成 audit report
生成 findings
生成 evidence-map
生成 traceability

允许处理：

issueCategory = audit
requiredAgentRole = audit-agent

允许写入：

.agentflow/output/audit/**
.agentflow/state/events/**

禁止：

不处理 spec issue
不修改源码
不生成 patch
不执行开发任务
不生成 release
不创建远程 PR
不 merge
不 deploy

⸻

5. Issue 数据结构

5.1 新增字段

在 InputIssue 增加：

pub issue_category: IssueCategory;
pub required_agent_role: AgentRole;

建议枚举：

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IssueCategory {
    Spec,
    Audit,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentRole {
    SpecAgent,
    BuildAgent,
    AuditAgent,
}

5.2 兼容旧 Issue

旧 Issue 没有 issueCategory 时：

默认 issueCategory = spec
默认 requiredAgentRole = build-agent

Rust 字段需要：

#[serde(default)]

⸻

6. Spec Issue 示例

{
  "version": "input-issue.v1",
  "issueId": "AF-104",
  "issueCategory": "spec",
  "requiredAgentRole": "build-agent",
  "sourceSpecId": "spec-001",
  "title": "完善任务合约详情阅读",
  "summary": "根据已确认规格实现任务合约详情阅读体验。",
  "status": "ready-for-execute",
  "displayStatus": "ready",
  "riskLevel": "medium",
  "scope": [
    "实现任务合约详情展示",
    "展示目标、范围、验收标准、证据要求"
  ],
  "nonGoals": [
    "不做审计报告生成",
    "不修改 release 规则"
  ]
}

⸻

7. Audit Issue 示例

Release 后自动生成：

{
  "version": "input-issue.v1",
  "issueId": "audit-release-v0.1.0",
  "issueCategory": "audit",
  "requiredAgentRole": "audit-agent",
  "sourceSpecId": "release-v0.1.0",
  "projectId": null,
  "title": "审计 Release v0.1.0",
  "summary": "检查 release-v0.1.0 是否符合已确认需求、任务、证据和交付边界。",
  "status": "ready-for-execute",
  "displayStatus": "ready",
  "riskLevel": "high",
  "audit": {
    "trigger": "release-auto",
    "sourceReleaseId": "release-v0.1.0",
    "sourceRunId": "run-v0.1.0",
    "expectedOutputs": [
      "audit.json",
      "audit-report.md",
      "findings.json",
      "evidence-map.json",
      "traceability.json"
    ]
  },
  "scope": [
    "读取 release-v0.1.0 的 delivery.json",
    "读取关联 SPEC / Issue / Evidence",
    "检查验证命令和结果",
    "检查是否有越界改动",
    "生成审计报告"
  ],
  "nonGoals": [
    "不修改用户源码",
    "不创建远程 PR",
    "不发布 GitHub Release",
    "不自动修复问题"
  ],
  "acceptanceCriteria": [
    ".agentflow/output/audit/audit-release-v0.1.0/audit.json 存在",
    ".agentflow/output/audit/audit-release-v0.1.0/audit-report.md 存在",
    ".agentflow/output/audit/audit-release-v0.1.0/findings.json 存在",
    ".agentflow/output/audit/audit-release-v0.1.0/evidence-map.json 存在",
    ".agentflow/output/audit/audit-release-v0.1.0/traceability.json 存在"
  ],
  "system": {
    "createdBy": "agentflow-release-auto",
    "path": ".agentflow/input/issues/audit-release-v0.1.0.json"
  }
}

⸻

8. Agent Descriptor

新增 Agent 描述文件。

建议路径：

.agentflow/define/agent/roles.json

结构：

{
  "version": "agent-roles.v1",
  "roles": [
    {
      "agentRole": "spec-agent",
      "label": "需求助手",
      "allowedIssueCategories": [],
      "allowedWrites": [
        ".agentflow/input/intake/**",
        ".agentflow/input/specs/**",
        ".agentflow/input/issues/**"
      ],
      "forbiddenWrites": [
        ".agentflow/execute/**",
        ".agentflow/output/release/**",
        ".agentflow/output/audit/**"
      ]
    },
    {
      "agentRole": "build-agent",
      "label": "执行助手",
      "allowedIssueCategories": ["spec"],
      "allowedWrites": [
        ".agentflow/execute/**",
        ".agentflow/output/evidence/**",
        ".agentflow/output/release/**"
      ],
      "forbiddenWrites": [
        ".agentflow/output/audit/**"
      ]
    },
    {
      "agentRole": "audit-agent",
      "label": "审计助手",
      "allowedIssueCategories": ["audit"],
      "allowedWrites": [
        ".agentflow/output/audit/**"
      ],
      "forbiddenWrites": [
        ".agentflow/execute/**",
        "用户源码"
      ]
    }
  ]
}

⸻

9. 判断 Agent 角色的方式

9.1 不可信方式

不要用：

Agent 自己说“我是 Audit Agent”
用户在聊天里说“你现在是 Build Agent”
模型回答风格
模型名字
线程标题

这些都不可靠。

9.2 可信方式

可信判断来源：

1. AgentFlow 生成的任务包
2. Issue.requiredAgentRole
3. Issue.issueCategory
4. Handoff package 中的 requiredAgentRole
5. Agent 写回文件中的 role claim
6. 写回路径是否符合 role 允许范围

最终判断不是：

这个模型是谁

而是：

这次写回是否符合这个 Issue 允许的 Agent Role

⸻

10. Handoff Package 规则

任务包必须包含：

{
  "handoffVersion": "agent-handoff.v1",
  "issueId": "AF-104",
  "issueCategory": "spec",
  "requiredAgentRole": "build-agent",
  "agentInstruction": "你现在是 Build Agent，只能执行 spec issue。"
}

Audit Issue 的任务包：

{
  "handoffVersion": "agent-handoff.v1",
  "issueId": "audit-release-v0.1.0",
  "issueCategory": "audit",
  "requiredAgentRole": "audit-agent",
  "agentInstruction": "你现在是 Audit Agent，只能执行 audit issue。"
}

任务包必须写明：

如果你不是 requiredAgentRole，请停止执行。
如果 issueCategory 不属于你，请停止执行。
不要改无关文件。
不要执行其他 Agent 的任务。

⸻

11. 写回校验

Agent 写回时必须有 manifest。

建议路径：

.agentflow/execute/runs/<run-id>/agent-claim.json

或者：

.agentflow/output/<kind>/<id>/agent-claim.json

结构：

{
  "version": "agent-claim.v1",
  "issueId": "audit-release-v0.1.0",
  "issueCategory": "audit",
  "claimedAgentRole": "audit-agent",
  "handoffId": "handoff-audit-release-v0.1.0",
  "createdBy": "audit-agent"
}

11.1 校验规则

claimedAgentRole 必须等于 issue.requiredAgentRole
issueCategory 必须等于 issue.issueCategory
写入路径必须在该 role allowedWrites 内
写入路径不能命中 forbiddenWrites

11.2 校验失败

如果失败：

不接受写回
不更新 issue 为 done
不更新 release 为 audited
写入 blocker
写入 timeline event
前端显示“Agent 角色不匹配”

⸻

12. Capability Matrix

Agent	可处理 issueCategory	可写 input	可写 execute	可写 release	可写 audit	可改源码
Spec Agent	无执行 Issue	是	否	否	否	否
Build Agent	spec	否	是	是	否	通过任务包约束
Audit Agent	audit	否	否	否	是	否

⸻

13. Release 自动审计流程

新的主链路：

Spec Issue
→ Build Agent
→ Delivery
Release Delivery
→ Audit Issue
→ Audit Agent
→ Audit Report

Release 后不要只生成 audit-request.json。

必须生成：

.agentflow/input/issues/audit-<release-id>.json

audit-request.json 可以废弃，或只作为兼容 metadata。

⸻

14. 前端展示要求

任务页显示：

任务类型
执行角色

示例：

AF-104
完善任务合约详情阅读
需求任务 · 执行助手 · 进行中
audit-release-v0.1.0
审计 Release v0.1.0
审计任务 · 审计助手 · 就绪

右侧任务合约显示：

任务类型：审计任务
执行角色：审计助手
触发来源：Release 自动审计
关联交付：release-v0.1.0

⸻

15. 前端动作限制

15.1 Spec Issue

显示：

复制 Build Agent 任务包
我已交给 Codex
检查写回
查看交付

不显示：

生成审计报告
写 findings
写 evidence-map

15.2 Audit Issue

显示：

复制 Audit Agent 任务包
等待审计报告
查看审计报告

不显示：

我已交给 Codex 改代码
检查源码写回
创建 release
修改文件

⸻

16. 校验函数建议

16.1 Rust

pub fn validate_agent_issue_permission(
    issue: &InputIssue,
    claimed_role: &AgentRole,
) -> Result<()> {
    if &issue.required_agent_role != claimed_role {
        anyhow::bail!(
            "Agent role mismatch: issue requires {:?}, got {:?}",
            issue.required_agent_role,
            claimed_role
        );
    }
    match (&issue.issue_category, claimed_role) {
        (IssueCategory::Spec, AgentRole::BuildAgent) => Ok(()),
        (IssueCategory::Audit, AgentRole::AuditAgent) => Ok(()),
        _ => anyhow::bail!("Agent role cannot execute this issue category"),
    }
}

16.2 TypeScript

function canAgentHandleIssue(agentRole: AgentRole, issue: InputIssue): boolean {
  return issue.requiredAgentRole === agentRole;
}

⸻

17. 状态与 blocker

如果出现角色不匹配，写入：

.agentflow/state/gates/blockers.json

示例：

{
  "action": "execute-issue",
  "reason": "这个任务需要 Audit Agent，但当前写回来自 Build Agent。",
  "sourcePath": ".agentflow/input/issues/audit-release-v0.1.0.json"
}

事件：

{
  "event": "agent.role_mismatch",
  "issueId": "audit-release-v0.1.0",
  "requiredAgentRole": "audit-agent",
  "claimedAgentRole": "build-agent"
}

⸻

18. 验收标准

必须满足：

1. Issue 有 issueCategory 字段。
2. Issue 有 requiredAgentRole 字段。
3. 旧 Issue 缺字段时兼容为 spec + build-agent。
4. Release 后生成 Audit Issue。
5. Audit Issue = audit + audit-agent。
6. Spec Issue = spec + build-agent。
7. Build Agent Handoff 不接受 audit issue。
8. Audit Agent Handoff 不接受 spec issue。
9. Agent 写回时校验 claimedAgentRole。
10. 角色不匹配时不接受写回。
11. 角色不匹配时写 blocker 和 timeline event。
12. 前端任务页显示任务类型和执行角色。
13. cargo test 通过。
14. npm build 通过。

⸻

19. 不做事项

不靠 Agent 自称判断身份
不让一个 Agent 执行两类任务
不让 Audit Agent 改代码
不让 Build Agent 写 audit report
不把 audit-request 当主入口
不自动调用 Codex API
不做远程权限系统

⸻

20. Codex 实现指令

你现在只做这个任务：AgentFlow Agent Role Descriptor & Issue Guard V1。
背景：
AgentFlow 需要确保 Spec Agent / Build Agent / Audit Agent 不会互相越权执行任务。Agent 身份不能靠模型自称判断，必须靠 Issue 分类、任务包、写回声明和路径校验来验证。
目标：
实现 Issue 分类和 Agent 角色隔离。
范围：
- crates/input/**
- crates/state/**
- crates/output/**
- apps/desktop/src/**
- .agentflow/define/agent/**
- AGENTS.md 生成规则
- docs/requirements/**
- 相关测试
具体要求：
1. 给 InputIssue 增加：
   - issueCategory
   - requiredAgentRole
2. 新增枚举：
   - IssueCategory: spec / audit
   - AgentRole: spec-agent / build-agent / audit-agent
3. 兼容旧 Issue：
   - 缺 issueCategory → spec
   - 缺 requiredAgentRole → build-agent
4. 新增 Agent Descriptor：
   .agentflow/define/agent/roles.json
5. Release 后生成 Audit Issue：
   - issueCategory = audit
   - requiredAgentRole = audit-agent
6. Handoff Package 必须包含：
   - issueId
   - issueCategory
   - requiredAgentRole
7. 写回必须包含 agent claim：
   - claimedAgentRole
   - issueCategory
   - issueId
   - handoffId
8. 校验：
   - claimedAgentRole == issue.requiredAgentRole
   - 写入路径符合 role allowedWrites
   - 不符合则拒绝写回
9. 前端：
   - 任务页显示任务类型和执行角色
   - spec issue 显示“需求任务 · 执行助手”
   - audit issue 显示“审计任务 · 审计助手”
   - 不显示错误角色的动作按钮
10. Agent 规则：
   - Spec Agent 只做需求整理
   - Build Agent 只执行 spec issue
   - Audit Agent 只执行 audit issue
禁止：
- 不要让 Build Agent 执行 Audit Issue
- 不要让 Audit Agent 执行 Spec Issue
- 不要靠 Agent 自称判断身份
- 不要让 UI 创建审计按钮
- 不要自动调用 Codex API
- 不要重复生成 audit issue
验证：
1. cargo test 通过。
2. npm --prefix apps/desktop run build 通过。
3. 旧 issue 能兼容读取。
4. Release 后生成 audit issue。
5. Build Agent 写回 audit issue 被拒绝。
6. Audit Agent 写回 spec issue 被拒绝。
7. 前端显示任务类型和执行角色。
输出：
- 改了哪些文件
- 新字段如何设计
- Agent Descriptor 如何生成
- 写回校验如何实现
- 前端如何显示
- 测试结果

⸻

这个版本的关键点是：

AgentFlow 不判断“这个模型到底是谁”。
AgentFlow 只判断：
这个 Issue 要求什么角色，
这个任务包要求什么角色，
这次写回声明什么角色，
写回内容和路径是否符合这个角色。

这样才能准确、稳定，而且适合当前“不直接调用 Codex API”的版本。