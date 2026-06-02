# 006 - Legacy CLI Retirement and Archive Pruning

创建日期：2026-06-02  
执行者：Codex

## 用户目标

PR #8 已经完成：

```text
旧需求代码隔离
新需求代码拆分
legacy / active / shared 边界建立
```

PR #9 已经完成：

```text
legacy-removal-audit.md 审计
legacy 宽导出收窄
crate root 不再暴露 legacy
archive_2026_05 改为 private
删除无用 evidence public re-export
Desktop build / audit 风险修复
```

但是当前仓库仍然保留大量旧 CLI 和旧 archive 实现。

本需求的目标是：

```text
进一步退役旧 CLI command surface，并裁剪不再需要的 archived 2026-05 旧实现。
```

大白话：

> PR #8 是把旧代码关进 legacy。  
> PR #9 是把 legacy 的大门关小。  
> 006 要开始处理“谁还应该活着”：旧 CLI 命令要么保留为明确兼容入口，要么隐藏 / 禁用 / 删除；旧 archive 代码要按命令和 read model 依赖裁剪。  
> 这一步不是做 Goal Tree 新功能，而是继续减少旧流程污染。

---

## 背景

当前旧流程仍主要存在于：

```text
crates/agentflow-core/src/legacy/archive_2026_05.rs
crates/agentflow-core/src/legacy/*.rs
crates/agentflow-cli/src/legacy.rs
crates/agentflow-cli/src/args.rs
apps/desktop/src-tauri/src/commands/legacy_core.rs
crates/agentflow-core/src/active/
```

当前旧 CLI surface 仍包括：

```text
goal
feature
team
project create
project closure
project code-audit
project docs-refresh
milestone
issue
context
plan
run
verify
review
eligibility
lease
index
view
update
metrics
projects
project-seed
issue-link
search
review-assistant
state
```

这些命令多数来自 2026-05 归档旧流程。

当前 `docs/requirements/README.md` 已经明确：

```text
旧 Workflow Control 流程不继承
旧 Product Feature 流程不继承
旧 Project Closure / Audit / Docs Refresh 不继承
旧 GoalLoop / Eligibility / Lease / Evidence 自动推进不继承
```

所以这些旧命令不能继续作为新 AgentFlow 的默认产品入口。

---

## 一句话定义

> **006 Legacy CLI Retirement and Archive Pruning 是一轮旧 CLI 退役和旧实现裁剪需求：先按命令决定保留 / 隐藏 / 禁用 / 删除，再按依赖裁剪 archive 代码，继续为 Goal Tree V1 清空旧流程污染。**

---

## 范围

本需求包含 7 个范围：

```text
1. Legacy CLI command classification
2. Legacy CLI command retirement policy
3. CLI legacy command hiding / disabling
4. Archive implementation pruning
5. Legacy data writer pruning
6. Desktop active read model dependency review
7. Documentation and verification update
```

---

## 非目标

本需求不做以下事情：

```text
不新增 Goal Tree
不定义新的 Goal / Milestone / Issue 模型
不定义新的 AgentRun
不启动 Agent
不调用模型
不执行用户项目命令
不修改用户项目源码
不改变 Project Workspace / Graph / Project File Reader 行为
不删除 Graph watcher fallback
不删除 Project File Reader fallback
不删除 Browser Preview mock
不删除当前 Desktop 必需 active read model，除非已有替代
不直接删除所有 legacy/archive_2026_05.rs
不把旧命令误认为新产品入口
```

---

# 1. Legacy CLI command classification

## 目标

对旧 CLI command surface 做第二轮分类。

输出文档：

```text
docs/architecture/legacy-cli-retirement-plan.md
```

每个 CLI 命令必须分类为：

```text
keep-temporary
hide-from-help
disable-with-message
delete
defer-until-goal-tree
```

---

## 分类含义

### keep-temporary

暂时保留。

适用于：

```text
当前仍被测试使用
当前仍有明确兼容价值
删除会造成仓库验证大面积失败
还没有新需求替代
```

### hide-from-help

命令仍可执行，但从 help 中隐藏或明显标注 legacy。

适用于：

```text
不希望用户继续使用
但短期还不能删除
```

### disable-with-message

命令入口保留，但执行时返回说明，不再执行旧流程。

适用于：

```text
旧流程风险较高
不应继续写入旧数据
但命令名还需要给用户明确迁移提示
```

示例输出：

```text
This command belongs to archived 2026-05 AgentFlow workflow.
It is disabled because the new Goal Tree workflow is not defined yet.
```

### delete

直接删除命令。

适用于：

```text
无测试依赖
无 Desktop 依赖
无兼容价值
不属于当前新需求
```

### defer-until-goal-tree

暂不处理，等 Goal Tree 需求定义后决定。

适用于：

```text
未来可能被新流程重用概念，但当前不能继承旧实现
```

---

## 初始建议分类

| CLI command | 建议分类 | 原因 |
|---|---|---|
| `init` | hide-from-help / defer-until-goal-tree | 旧 goal bootstrap 初始化，不等于新 Project Workspace |
| `goal bootstrap/check/next` | disable-with-message | 旧 GoalLoop，不继承 |
| `feature create/status/next` | disable-with-message | 旧 Product Feature，不继承 |
| `team create` | disable-with-message | 旧 Team 模型未被新流程授权 |
| `project create` | disable-with-message | 旧 Project 模型未被新流程授权 |
| `project closure` | disable-with-message | 旧 Closure 不继承 |
| `project code-audit` | disable-with-message | 旧 Code Audit 不继承 |
| `project docs-refresh` | disable-with-message | 旧 Docs Refresh 不继承 |
| `milestone create` | disable-with-message | 旧 Milestone 模型未被新流程授权 |
| `issue create` | disable-with-message | 旧 IssueContract 未被新流程授权 |
| `context` | delete or disable-with-message | Graph 已替代旧 context 方向 |
| `plan` | disable-with-message | 旧 Issue planning 未被授权 |
| `run` | disable-with-message | 新 AgentRun 未定义，禁止旧 run |
| `verify` | disable-with-message | 旧验证流程未被授权 |
| `review` | disable-with-message | 旧 review/evidence 未被授权 |
| `eligibility` | disable-with-message | 旧 eligibility 不继承 |
| `lease` | disable-with-message | 旧 lease 逻辑未按新需求重定义 |
| `index rebuild` | delete or disable-with-message | Graph 已作为新索引底座 |
| `view save/show` | delete or disable-with-message | 旧 saved view 不继承 |
| `update summary` | disable-with-message | 旧 update 不继承 |
| `metrics` | keep-temporary | 可能仍用于当前 read-only snapshot |
| `projects` | keep-temporary | 当前 Desktop / local model 仍可能参考 |
| `project-seed` | disable-with-message | 旧 seed writer 不继承 |
| `issue-link` | disable-with-message | 旧 issue/project link 不继承 |
| `search` | keep-temporary / defer-until-goal-tree | 当前 Desktop legacy search 仍可能参考 |
| `review-assistant` | disable-with-message | 旧 assistant 不继承 |
| `state check` | disable-with-message | 旧 workflow state check 不继承 |

---

## 验收标准

```text
- [ ] 新增 docs/architecture/legacy-cli-retirement-plan.md。
- [ ] 每个旧 CLI command 都有分类。
- [ ] 每个分类都有原因。
- [ ] 明确哪些命令本轮禁用。
- [ ] 明确哪些命令本轮删除。
- [ ] 明确哪些命令暂时保留。
- [ ] 明确哪些命令等 Goal Tree 再决定。
```

---

# 2. Legacy CLI command retirement policy

## 目标

建立旧 CLI 退役策略。

---

## 2.1 默认策略

旧命令默认不再执行旧写流程。

尤其是以下命令：

```text
goal bootstrap
goal next
feature create
team create
project create
milestone create
issue create
plan
run
verify
review
eligibility
lease
project closure
project code-audit
project docs-refresh
project-seed --write
issue-link --write
review-assistant
```

这些命令都属于旧流程写入或自动推进逻辑，应优先禁用。

---

## 2.2 允许暂时只读保留的命令

这些命令可以暂时保留为 read-only compatibility：

```text
metrics
projects
search
```

原因：

```text
当前 Desktop active read model 仍有相似 read-only 能力
这些命令可能用于人工检查旧数据
它们不应该写入项目
```

---

## 2.3 禁用提示

禁用旧命令时，输出必须明确：

```text
1. 该命令属于 archived 2026-05 workflow
2. 新 AgentFlow 流程尚未授权该行为
3. 当前不会执行写入 / 运行 / 验证 / review
4. 后续请等待 Goal Tree V1
```

建议文案：

```text
This command belongs to the archived 2026-05 AgentFlow workflow.
It is disabled in the new requirements track.
The new Goal Tree / AgentRun workflow has not been defined yet.
No files were written and no command was executed.
```

---

## 验收标准

```text
- [ ] 旧写命令不会继续执行旧写流程。
- [ ] 禁用命令输出明确 legacy message。
- [ ] 禁用命令不写文件。
- [ ] 禁用命令不执行项目命令。
- [ ] 禁用命令不调用模型。
- [ ] 保留命令只读。
```

---

# 3. CLI implementation changes

## 当前结构

PR #8 / #9 后 CLI 结构为：

```text
crates/agentflow-cli/src/
├── main.rs
├── args.rs
├── legacy.rs
└── print.rs
```

## 目标结构

新增：

```text
crates/agentflow-cli/src/
├── main.rs
├── args.rs
├── legacy.rs
├── print.rs
├── retirement.rs
└── active.rs
```

---

## 3.1 `retirement.rs`

负责：

```text
legacy_disabled_message
legacy_command_status
should_disable_legacy_command
print_legacy_retirement_message
```

示例：

```rust
pub enum LegacyCommandDisposition {
    KeepTemporary,
    HideFromHelp,
    DisableWithMessage,
    Delete,
    DeferUntilGoalTree,
}
```

---

## 3.2 `legacy.rs`

改成：

```text
只处理仍保留的 compatibility 命令
禁用命令走 retirement message
不再直接执行旧写流程
```

建议：

```rust
match cli.command {
    Command::Metrics => run_metrics_readonly(...),
    Command::Projects => run_projects_readonly(...),
    Command::Search { query } => run_search_readonly(...),
    command if should_disable_legacy_command(&command) => print_disabled_message(...),
    ...
}
```

---

## 3.3 `active.rs`

当前可以很小。

放：

```text
version / doctor / read-only status
```

如果暂时没有新 CLI，则只留空边界注释：

```rust
//! Active CLI surface.
//!
//! New CLI commands must be added here only after a new requirement authorizes them.
```

---

## 验收标准

```text
- [ ] 新增 retirement.rs。
- [ ] 禁用命令不再调用旧 legacy writer。
- [ ] keep-temporary 命令仍可读。
- [ ] 不新增新 CLI 功能。
- [ ] cargo test -p agentflow-cli 通过。
```

---

# 4. Archive implementation pruning

## 目标

CLI 退役之后，继续裁剪：

```text
crates/agentflow-core/src/legacy/archive_2026_05.rs
```

但不能盲删。

---

## 删除顺序

必须按这个顺序：

```text
1. 禁用 CLI 写命令
2. 删除对应 CLI legacy imports
3. 删除对应 named legacy module re-export
4. 删除 archive 中仅被该 module 使用的 functions / DTOs
5. 删除对应 tests 或迁移 tests 到 new requirements
```

---

## 优先裁剪对象

### P0：旧写流程入口

优先处理：

```text
create_product_feature
create_team
create_project
create_milestone
create_issue
plan_issue
run_issue
verify_issue
review_issue
write_workflow_eligibility
write_workflow_lease_snapshot
write_project_closure_state
write_project_code_audit_snapshot
write_project_docs_refresh_snapshot
write_local_project_seed
write_issue_project_link
write_review_assistant
save_view
```

这些都属于旧写 / 自动推进 / 旧管理流程。

### P1：旧数据 writer helper

```text
write_json_file
write_markdown_file
write_issue_artifacts
write_run_artifacts
write_evidence_artifacts
write_review_artifacts
write_update_artifacts
write_saved_view_artifacts
write_index_db
```

注意：如果这些是 shared 工具且仍被 active read model 用，要保留或迁到 shared。

### P2：旧 DTO

只有当使用它的函数全部删除后，才能删除 DTO。

例如：

```text
ProductFeature*
AgentRun*
Review*
Evidence*
ProjectClosure*
ProjectAudit*
DocsRefresh*
SavedView*
Eligibility*
Lease*
```

---

## 暂时不能裁剪对象

```text
Desktop active read model 仍需要的 DTO
read_local_metrics_snapshot 依赖的 DTO
read_local_project_model_snapshot 依赖的 DTO
read_desktop_workbench_snapshot 依赖的 DTO
read_local_search_snapshot 依赖的 DTO
```

---

## 验收标准

```text
- [ ] 每个删除的 archive symbol 都在 legacy-cli-retirement-plan 或 legacy-removal-audit 中有记录。
- [ ] 删除顺序遵守 CLI -> re-export -> archive -> test。
- [ ] 不删除 active read model 依赖。
- [ ] 不删除 still-used DTO。
- [ ] cargo test -p agentflow-core 通过。
```

---

# 5. Legacy data writer pruning

## 目标

裁剪旧数据写入路径。

重点路径：

```text
.agentflow/issues/
.agentflow/runs/
.agentflow/evidence/
.agentflow/reviews/
.agentflow/updates/
.agentflow/state/
.agentflow/views/
.agentflow/index.db
.agentflow/index.json
graphify-out/
.codex/
```

---

## 行为要求

如果命令已禁用，就不应继续保留对应 writer 的 public 路径。

例如：

```text
run / verify / review 禁用
-> 删除 run / evidence / review writer public exposure
-> archive private helper 若无引用则删除
```

---

## 注意

不能删除：

```text
当前 Desktop 仍读取的旧数据 parser
当前 active read model 仍依赖的数据 reader
```

但可以删除：

```text
旧写入函数
旧生成函数
旧自动推进函数
```

---

## 验收标准

```text
- [ ] 禁用旧写命令后，对应 writer 不再被 CLI 调用。
- [ ] 无引用 writer 被删除。
- [ ] 有 read-only reader 需求的 parser 保留。
- [ ] legacy-removal-audit.md 更新。
- [ ] cargo test 通过。
```

---

# 6. Desktop active read model dependency review

## 目标

确认当前 Desktop active read model 是否仍然需要旧 archive。

当前暂时不能删除：

```text
read_desktop_workbench_snapshot
read_local_metrics_snapshot
read_local_project_model_snapshot
read_project_milestone_issue_view_model_snapshot
read_local_search_snapshot
```

但需要进一步审查它们依赖哪些旧 DTO / reader。

---

## 输出文档

更新：

```text
docs/architecture/legacy-removal-audit.md
```

新增小节：

```text
Desktop Active Read Model Dependency Map
```

内容：

```text
active function
used legacy DTO
used legacy reader
used legacy path
can replace later?
replacement candidate
```

示例：

```md
| Active read model | Uses legacy data | Current reason | Replacement candidate |
|---|---|---|---|
| read_local_project_model_snapshot | old projects/issues/milestones | Desktop current UI still renders old model | Goal Tree V1 |
| read_local_search_snapshot | old issue/project docs | Desktop search compatibility | Graph search + Project File search |
```

---

## 验收标准

```text
- [ ] active read model dependency map 完成。
- [ ] 标明哪些 active read model 可由 Graph / Project File Reader 替代。
- [ ] 标明哪些必须等 Goal Tree 替代。
- [ ] 不删除仍被 Desktop 使用的 read model。
```

---

# 7. Documentation updates

需要更新：

```text
docs/architecture/legacy-removal-audit.md
docs/architecture/legacy-cli-retirement-plan.md
docs/architecture/legacy-code-map.md
docs/architecture/current-module-boundaries.md
verification.md
```

---

## 7.1 legacy-code-map.md

更新：

```text
哪些 legacy command 已禁用
哪些已删除
哪些暂时保留
哪些等待 Goal Tree
```

---

## 7.2 current-module-boundaries.md

更新：

```text
Legacy CLI 状态
Archive pruning 状态
Active read model 仍保留原因
```

---

## 7.3 verification.md

记录：

```text
执行者
目标
禁用了哪些命令
删除了哪些旧代码
保留了哪些旧代码
是否有行为变化
验证命令
结果
```

---

# 8. 建议开发切片

## Slice 1：Legacy CLI retirement plan

目标：

```text
新增 docs/architecture/legacy-cli-retirement-plan.md
对所有旧 CLI 命令分类
```

验收：

```text
分类完整
不改代码行为
cargo test 通过
```

---

## Slice 2：Disable old write commands

目标：

```text
旧写命令执行时输出 disabled legacy message
不再调用旧 writer
```

优先禁用：

```text
feature create
team create
project create
milestone create
issue create
plan
run
verify
review
eligibility
lease
project closure
project code-audit
project docs-refresh
project-seed --write
issue-link --write
review-assistant
```

验收：

```text
禁用命令不写文件
禁用命令不执行项目命令
cargo test -p agentflow-cli 通过
```

---

## Slice 3：Remove CLI imports for disabled writers

目标：

```text
从 legacy.rs 删除已禁用 writer imports
从 named legacy modules 删除对应 re-export
```

验收：

```text
rg 确认 CLI 不再引用旧 writer
cargo test 通过
```

---

## Slice 4：Prune archive writer implementations

目标：

```text
删除已无引用的 archive writer functions 和 DTOs
```

验收：

```text
legacy-removal-audit.md 更新
cargo test -p agentflow-core 通过
```

---

## Slice 5：Desktop active read model dependency review

目标：

```text
补 active read model dependency map
```

验收：

```text
不删除 active read model
清楚标注替代候选
```

---

## Slice 6：Docs and verification closeout

目标：

```text
更新 legacy-code-map
更新 current-module-boundaries
更新 verification
```

验收：

```text
验证命令全部通过
```

---

# 9. 总验收标准

```text
- [ ] 新增 docs/architecture/legacy-cli-retirement-plan.md。
- [ ] 所有旧 CLI command 完成 keep / hide / disable / delete / defer 分类。
- [ ] 旧写命令不再执行旧写流程。
- [ ] 禁用命令输出明确 legacy disabled message。
- [ ] 禁用命令不写文件。
- [ ] 禁用命令不执行项目命令。
- [ ] CLI legacy.rs 删除已禁用 writer imports。
- [ ] named legacy modules 删除已禁用 writer re-export。
- [ ] archive_2026_05.rs 删除无引用旧 writer implementations。
- [ ] 删除无引用旧 DTO。
- [ ] legacy-removal-audit.md 更新。
- [ ] Desktop active read model dependency map 完成。
- [ ] 保留 Desktop 必需 active read model。
- [ ] 保留 Graph watcher fallback。
- [ ] 保留 Project File Reader fallback。
- [ ] 不新增 Goal Tree。
- [ ] 不改变 Tauri command 名称。
- [ ] 不改变 Desktop 只读边界。
- [ ] cargo fmt --check 通过。
- [ ] cargo test -p agentflow-core 通过。
- [ ] cargo test -p agentflow-cli 通过。
- [ ] cargo test -p agentflow-graph 通过。
- [ ] cargo test 通过。
- [ ] npm --prefix apps/desktop run build 通过。
- [ ] npm --prefix apps/desktop audit 通过。
- [ ] git diff --check 通过。
```

---

# 10. 验证命令

必须执行：

```bash
cargo fmt --check
cargo test -p agentflow-core
cargo test -p agentflow-cli
cargo test -p agentflow-graph
cargo test
npm --prefix apps/desktop run build
npm --prefix apps/desktop audit
git diff --check
```

---

# 11. 交付说明要求

PR 描述必须包含：

```text
1. 哪些 CLI 命令被禁用。
2. 哪些 CLI 命令暂时保留。
3. 哪些 CLI 命令删除或隐藏。
4. 删除了哪些 legacy writer。
5. 删除了哪些 archive DTO / helper。
6. 保留了哪些 active read model，为什么。
7. 保留了哪些 fallback，为什么。
8. 是否有行为变化。
9. 验证命令和结果。
```

---

# 12. Codex 执行指令

```md
请执行 006 - Legacy CLI Retirement and Archive Pruning。

目标：
在 PR #9 已经完成 legacy audit、宽导出收窄和 Desktop build fixes 后，继续退役旧 CLI command surface，并裁剪不再需要的 archive 旧实现。

必须遵守：
1. 不新增 Goal Tree。
2. 不定义新的 Goal / Milestone / Issue。
3. 不定义新的 AgentRun。
4. 不启动 Agent。
5. 不调用模型。
6. 不执行用户项目命令。
7. 不修改用户项目源码。
8. 不改变 Tauri command 名称。
9. 不改变 Desktop 只读边界。
10. 不删除当前 Desktop 必需 active read model。
11. 不删除 Graph watcher fallback。
12. 不删除 Project File Reader fallback。
13. 删除前必须在文档中分类和说明。

执行步骤：
1. 新增 docs/architecture/legacy-cli-retirement-plan.md。
2. 对所有旧 CLI command 分类为 keep-temporary / hide-from-help / disable-with-message / delete / defer-until-goal-tree。
3. 禁用旧写命令，让它们输出 legacy disabled message，不再执行旧写流程。
4. 保留必要只读命令，例如 metrics / projects / search，除非文档确认删除。
5. 从 CLI legacy.rs 删除已禁用 writer imports。
6. 从 named legacy modules 删除已禁用 writer re-export。
7. 删除 archive_2026_05.rs 中已经无引用的 writer implementation 和 DTO。
8. 更新 legacy-removal-audit.md。
9. 更新 legacy-code-map.md。
10. 更新 current-module-boundaries.md。
11. 更新 verification.md。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-core
- cargo test -p agentflow-cli
- cargo test -p agentflow-graph
- cargo test
- npm --prefix apps/desktop run build
- npm --prefix apps/desktop audit
- git diff --check

交付时说明：
- 哪些旧 CLI 命令被禁用。
- 哪些旧 CLI 命令暂时保留。
- 哪些 archive 代码被删除。
- 哪些 active read model 被保留。
- 是否有行为变化。
```

---

# 13. 完成定义

本需求完成后，AgentFlow 应达到：

```text
旧 CLI 写流程不再可执行
旧写入函数开始从 archive 中裁剪
legacy archive 继续缩小
Desktop read-only compatibility 仍保留
Graph / Project File Reader 新底座不受影响
Goal Tree V1 不再被旧 CLI 写流程污染
```

最终一句话：

> **006 的重点不是再做一轮文档审计，而是开始退役旧 CLI 写入口，并按依赖裁剪 archive 实现，让旧流程真正退出新主干。**
