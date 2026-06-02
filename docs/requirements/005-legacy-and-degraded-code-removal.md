# 005 - Legacy and Degraded Code Removal

创建日期：2026-06-01  
执行者：Codex

## 用户目标

PR #8 已经完成了第一轮代码清理：

```text
旧需求代码被隔离到 legacy
当前 Desktop 仍需的旧 read model 被放到 active transitional read model
新需求代码 Project Workspace / Graph / Project File Reader 已经拆成清晰模块
```

但是 PR #8 只是“隔离”和“拆分”，没有真正删除旧代码。

本需求的目标是：

```text
在 PR #8 已经完成 legacy 隔离之后，做一轮安全删除，把不再被使用、不再被授权、不再需要兼容的旧代码和降级代码清理掉。
```

大白话：

> PR #8 是把旧代码关进 legacy 房间。  
> 005 是进去盘点：哪些还被 Desktop / CLI / 测试用着，哪些没人用了。  
> 还在用的先保留；没人用的删掉；暴露太宽的旧导出收窄。  
> 不允许拍脑袋删，必须先做引用审计。

---

## 背景

当前新需求体系已经明确：

```text
docs/requirements/ 是后续开发唯一需求入口。
旧 Workflow Control、旧 Product Feature、旧 Project Closure / Audit / Docs Refresh、
旧 GoalLoop / Eligibility / Lease / Evidence 自动推进都不继承，
除非新需求重新明确。
```

PR #8 已经完成：

```text
crates/agentflow-core/src/legacy/
crates/agentflow-core/src/active/
crates/agentflow-core/src/shared/

crates/agentflow-cli/src/legacy.rs
crates/agentflow-cli/src/args.rs
crates/agentflow-cli/src/print.rs

apps/desktop/src-tauri/src/commands/
apps/desktop/src-tauri/src/project_files/
apps/desktop/src-tauri/src/project_workspace/

crates/graph/src/watcher/
apps/desktop/src/features/project-files/
apps/desktop/src/types/
```

但当前仍存在几个问题：

```text
1. legacy/archive_2026_05.rs 仍保留大量旧实现。
2. legacy/mod.rs 仍可能存在宽导出，例如 pub use archive_2026_05::*。
3. CLI 旧命令仍然可以直接访问旧流程。
4. Desktop 仍有 active transitional read model，需要确认哪些旧代码只是为了它保留。
5. 旧的 run / verify / review / closure / audit / docs refresh / evidence / lease 代码需要逐项判断能否删除。
6. 一些“降级代码”需要区分：有些是必要 fallback，有些是旧流程残留。
```

---

## 一句话定义

> **005 Legacy and Degraded Code Removal 是一次安全删除需求：先做引用审计，再删除无引用、无授权、无兼容价值的旧代码，并收窄 legacy 暴露面。**

---

## 范围

本需求包含 6 个范围：

```text
1. Legacy reachability audit
2. Unused legacy code removal
3. Legacy export narrowing
4. Legacy CLI command deprecation boundary
5. Obsolete legacy data writer removal
6. Degraded code classification and cleanup
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
不执行项目命令
不修改用户项目源码
不删除当前 Desktop 仍依赖的 active read model
不删除 Graph OS native watcher 的 fallback
不删除仍被 CLI legacy command 使用的代码
不删除仍被测试覆盖且没有替代方案的代码
不改变 Tauri command 对外名称
不改变 Desktop 只读边界
不做 UI 大改版
```

---

# 1. 删除原则

## 1.1 先审计，再删除

任何 legacy 代码删除前，必须先进入审计表。

每个 legacy 符号至少要分类为：

```text
A: active read model 仍在使用
B: CLI legacy command 仍在使用
C: 只被测试使用
D: 无任何引用
E: 不确定，需要人工确认
```

只有 D 类可以直接删除。

C 类需要判断：

```text
测试是否只是旧流程测试？
测试是否仍然有价值？
能否随旧代码一起删？
```

A / B 类不能删除，除非先有替代方案。

---

## 1.2 不删除当前 Desktop 需要的兼容读模型

这些当前不能删除：

```text
read_desktop_workbench_snapshot
read_local_metrics_snapshot
read_local_project_model_snapshot
read_project_milestone_issue_view_model_snapshot
read_local_search_snapshot
WorkbenchBoundary
```

这些属于：

```text
active transitional read model
```

作用是让当前 Desktop 继续读取旧数据并保持页面可用。

---

## 1.3 不删除 Graph watcher fallback

Graph watcher fallback 不是旧需求代码。

当前 Graph watcher 已经是：

```text
OS native watcher by default
fingerprint fallback as degraded backup
```

这个 fallback 不能删除，因为下面这些环境可能需要：

```text
Docker
VM
WSL
网络盘
Linux inotify limit 不足
macOS FSEvents 权限限制
原生 watcher 创建失败
```

本需求中“降级代码删除”不包括 Graph watcher fallback。

---

## 1.4 删除旧流程降级代码

可以清理的“降级代码”主要是旧流程里的 fallback / compatibility 逻辑，例如：

```text
旧 GoalLoop fallback
旧 eligibility fallback
旧 lease snapshot fallback
旧 run / verify / review fallback
旧 product feature fallback
旧 docs refresh / code audit fallback
旧 evidence / review / update 兼容路径
旧 graphify-out / .codex 相关残留
```

但前提仍然是：

```text
无 active read model 引用
无 CLI legacy 引用
无 Desktop 引用
无有效测试依赖
```

---

# 2. Legacy Reachability Audit

## 目标

生成一份旧代码引用审计报告。

路径：

```text
docs/architecture/legacy-removal-audit.md
```

这份报告要列出：

```text
legacy/archive_2026_05.rs 中仍然暴露的 pub struct
legacy/archive_2026_05.rs 中仍然暴露的 pub enum
legacy/archive_2026_05.rs 中仍然暴露的 pub fn
legacy 子模块中的 re-export
CLI legacy command 使用了哪些旧符号
Desktop active read model 使用了哪些旧符号
测试使用了哪些旧符号
完全无引用的旧符号
```

---

## 审计分类

每个符号都要打标签：

```text
active-read-model
cli-legacy
test-only
unused
uncertain
```

示例：

```md
| Symbol | Kind | Module | Used by | Category | Action |
|---|---|---|---|---|---|
| IssueContract | struct | legacy/archive_2026_05.rs | Desktop read model, CLI issue | active-read-model | keep |
| write_project_docs_refresh_snapshot | fn | legacy/archive_2026_05.rs | CLI project docs-refresh | cli-legacy | keep temporarily |
| old_graphify_out_helper | fn | legacy/archive_2026_05.rs | none | unused | delete |
```

---

## 推荐审计命令

可以使用：

```bash
rg "pub struct|pub enum|pub fn|pub const|pub type" crates/agentflow-core/src/legacy
rg "<symbol_name>" crates apps docs
cargo test
```

如果需要更可靠，可以用：

```bash
cargo check --all-targets
cargo test --all-targets
```

---

## 验收标准

```text
- [ ] 新增 docs/architecture/legacy-removal-audit.md。
- [ ] audit 覆盖 legacy/archive_2026_05.rs 中的主要 pub 符号。
- [ ] audit 覆盖 legacy 子模块 re-export。
- [ ] audit 标出 active-read-model / cli-legacy / test-only / unused / uncertain。
- [ ] audit 明确哪些符号可以删除。
- [ ] audit 明确哪些符号暂时不能删除。
```

---

# 3. Unused Legacy Code Removal

## 目标

删除审计后确认无引用的旧代码。

只删除：

```text
D: unused
```

不删除：

```text
A: active-read-model
B: cli-legacy
C: test-only，除非确认测试也属于旧流程残留且可以删除
E: uncertain
```

---

## 可删除候选

优先扫描：

```text
无引用 legacy helper
无引用旧 DTO
无引用旧 markdown writer
无引用旧 JSON writer
无引用旧 sqlite index helper
无引用旧 graphify-out helper
无引用旧 .codex helper
无引用旧 docs refresh helper
无引用旧 code audit helper
无引用旧 review assistant helper
无引用旧 fallback 函数
```

---

## 删除规则

每删除一批代码，必须执行：

```bash
cargo test -p agentflow-core
cargo test -p agentflow-cli
cargo test
```

如果删的是前端相关类型，还要执行：

```bash
npm --prefix apps/desktop run build
```

---

## 验收标准

```text
- [ ] 删除 audit 中标记为 unused 的旧代码。
- [ ] 删除后没有 dangling re-export。
- [ ] 删除后没有 dead import。
- [ ] 删除后 legacy-removal-audit.md 更新。
- [ ] cargo test -p agentflow-core 通过。
- [ ] cargo test -p agentflow-cli 通过。
- [ ] cargo test 通过。
```

---

# 4. Legacy Export Narrowing

## 当前问题

PR #8 已经把旧实现隔离到 legacy，但为了兼容，可能仍存在宽导出：

```rust
pub use archive_2026_05::*;
```

这个导出太宽，会让新代码继续轻易引用旧流程。

## 目标

把 legacy 的暴露面收窄。

从：

```rust
pub use archive_2026_05::*;
```

收敛成：

```rust
pub mod goal_protocol;
pub mod product_feature;
pub mod workflow_control;
...
```

以及必要的显式 re-export。

---

## 行为要求

`legacy/mod.rs` 不应该继续 blanket export 整个 archive。

改成：

```rust
pub mod archive_2026_05;

pub mod goal_protocol;
pub mod product_feature;
pub mod team_project_milestone_issue;
pub mod workflow_control;
pub mod run_verify_review;
pub mod eligibility_lease;
pub mod project_closure;
pub mod project_audit_docs_refresh;
pub mod evidence;
pub mod saved_view;
pub mod sqlite_index;
```

如果 CLI 仍需要旧符号，应该从对应 legacy 子模块导入。

例如：

```rust
use agentflow_core::legacy::run_verify_review::{run_issue, verify_issue, review_issue};
```

不要：

```rust
use agentflow_core::*;
```

或者：

```rust
use agentflow_core::legacy::*;
```

---

## 验收标准

```text
- [ ] legacy/mod.rs 不再 blanket pub use archive_2026_05::*。
- [ ] CLI legacy.rs 改为显式 import 所需 legacy 子模块符号。
- [ ] active read model 只 import 自己需要的兼容符号。
- [ ] 新模块不使用 agentflow_core::legacy::*。
- [ ] rg "pub use archive_2026_05::\*" 没有结果。
- [ ] cargo test 通过。
```

---

# 5. Legacy CLI Deprecation Boundary

## 目标

旧 CLI 命令暂时保持可编译，但需要进一步降权。

当前旧命令仍包括：

```text
goal
feature
team
project create / closure / code-audit / docs-refresh
milestone
issue
run
verify
review
eligibility
lease
index
view
update
metrics
project-seed
issue-link
review-assistant
state
```

本需求不删除这些命令，但需要让它们明显属于 legacy。

---

## 行为

给旧 CLI 命令增加统一提示。

建议在执行旧命令时打印：

```text
warning: this command belongs to archived 2026-05 AgentFlow workflow and is kept for compatibility only.
```

也可以先只在 help / docs 中标注，不改变输出。

推荐第一版：

```text
不改变命令输出行为，只在 legacy.rs 顶部和文档里明确标注。
```

如果要加运行时 warning，需要确认不会破坏测试快照。

---

## 不做

```text
不把旧命令改名为 agentflow legacy xxx
不删除旧命令
不新增新 CLI 命令
```

---

## 验收标准

```text
- [ ] legacy CLI 命令实现仍在 crates/agentflow-cli/src/legacy.rs。
- [ ] legacy.rs 顶部注释明确旧命令只为兼容。
- [ ] legacy-code-map.md 记录旧命令 surface。
- [ ] 不新增新的旧流程入口。
- [ ] cargo test -p agentflow-cli 通过。
```

---

# 6. Obsolete Legacy Data Writer Removal

## 目标

删除旧流程里不再授权的数据写入函数。

重点扫描旧路径：

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

## 当前不能直接删除的情况

如果以下仍然成立，就不能删除：

```text
Desktop active read model 还读取这些路径
CLI legacy command 还写这些路径
测试还依赖这些路径
```

但可以先做：

```text
标记为 legacy writer
收窄导出
移动到 legacy writer section
```

---

## 可删除目标

可删除：

```text
无引用旧 writer
无引用旧 path constant
无引用旧 markdown generator
无引用旧 index writer
无引用旧 evidence writer
无引用旧 review writer
无引用旧 update writer
无引用旧 graphify-out writer
无引用旧 .codex writer
```

---

## 验收标准

```text
- [ ] 审计 legacy data writer 引用。
- [ ] 删除无引用 legacy data writer。
- [ ] 不删除 current Desktop 仍读取的路径处理。
- [ ] 不删除 CLI legacy command 仍调用的 writer，除非命令一起删除。
- [ ] legacy-removal-audit.md 更新。
- [ ] cargo test 通过。
```

---

# 7. Degraded Code Classification and Cleanup

## 目标

区分“必要 fallback”和“旧流程降级残留”。

---

## 必须保留的 fallback

```text
Graph watcher fingerprint fallback
browser preview mock fallback
Project File Reader unsupported/binary fallback
Project File Reader browser preview fallback
path guard fallback / explicit error fallback
```

这些是当前新需求底座的一部分，不能删除。

---

## 可以删除的降级残留候选

```text
旧 GoalLoop fallback
旧 workflow readiness fallback
旧 eligibility fallback
旧 lease snapshot fallback
旧 run validation fallback
旧 review/evidence fallback
旧 product feature fallback
旧 docs refresh fallback
旧 code audit fallback
旧 graphify-out fallback
旧 .codex fallback
```

前提：

```text
无 active read model 引用
无 CLI legacy 引用
无测试依赖
```

---

## 验收标准

```text
- [ ] docs/architecture/legacy-removal-audit.md 中增加 degraded code 分类。
- [ ] 明确保留 Graph watcher fallback。
- [ ] 明确保留 Project File Reader fallback。
- [ ] 删除无引用旧流程降级残留。
- [ ] 不删除新需求底座 fallback。
```

---

# 8. 当前不能删除清单

以下暂时不能删除：

```text
crates/agentflow-core/src/active/
apps/desktop/src-tauri/src/commands/legacy_core.rs
Desktop 当前调用的 read-only snapshot
CLI legacy command 仍然引用的旧符号
Graph watcher fallback
Project File Reader fallback
Browser preview mocks
```

原因：

```text
这些仍然服务当前 UI / CLI 兼容 / 新底座稳定性。
```

---

# 9. 建议开发切片

## Slice 1：Legacy Removal Audit

目标：

```text
生成 docs/architecture/legacy-removal-audit.md。
```

内容：

```text
- legacy pub symbols 清单
- active read model 引用
- CLI legacy 引用
- test-only 引用
- unused 引用
- degraded code 分类
```

验收：

```text
- audit 文档完整。
- 不删除代码。
- cargo test 通过。
```

---

## Slice 2：Remove Unused Legacy Symbols

目标：

```text
删除 audit 中明确标记为 unused 的旧符号。
```

验收：

```text
- 删除 D 类 unused。
- audit 更新。
- cargo test -p agentflow-core 通过。
- cargo test 通过。
```

---

## Slice 3：Narrow Legacy Exports

目标：

```text
删除 pub use archive_2026_05::*。
```

行为：

```text
- legacy/mod.rs 改为显式模块导出。
- CLI legacy.rs 显式 import 所需符号。
- active read model 显式 import 所需符号。
```

验收：

```text
- rg "pub use archive_2026_05::\*" 无结果。
- cargo test 通过。
```

---

## Slice 4：Clean Obsolete Legacy Writers

目标：

```text
删除无引用旧数据写入函数。
```

重点：

```text
旧 issues/runs/evidence/reviews/updates/views/index writers
graphify-out / .codex 残留
```

验收：

```text
- 无引用 writer 删除。
- 有引用 writer 保留并标记 legacy。
- cargo test 通过。
```

---

## Slice 5：Legacy CLI Boundary Tightening

目标：

```text
旧 CLI 命令继续保留，但进一步标记为 legacy。
```

行为：

```text
- 不改命令名。
- 不删除命令。
- 在文档和模块注释中明确旧命令兼容性质。
- 可选：后续再加 runtime warning。
```

验收：

```text
- cargo test -p agentflow-cli 通过。
- CLI 仍可编译。
```

---

# 10. 总验收标准

```text
- [ ] 新增 docs/architecture/legacy-removal-audit.md。
- [ ] legacy symbols 完成 active / cli / test-only / unused / uncertain 分类。
- [ ] degraded code 完成 keep / delete 分类。
- [ ] 删除 unused legacy code。
- [ ] 删除无引用旧流程降级残留。
- [ ] legacy/mod.rs 不再 blanket pub use archive_2026_05::*。
- [ ] CLI legacy.rs 使用显式 legacy 子模块 import。
- [ ] active read model 使用显式 compatibility import。
- [ ] 保留 Desktop 必需 active read model。
- [ ] 保留必要 CLI legacy compatibility。
- [ ] 保留 Graph watcher fallback。
- [ ] 保留 Project File Reader fallback。
- [ ] 不改变 Tauri command 名称。
- [ ] 不改变 Desktop 只读边界。
- [ ] 不新增产品功能。
- [ ] cargo fmt --check 通过。
- [ ] cargo test -p agentflow-core 通过。
- [ ] cargo test -p agentflow-cli 通过。
- [ ] cargo test -p agentflow-graph 通过。
- [ ] cargo test 通过。
- [ ] npm --prefix apps/desktop run build 通过。
- [ ] git diff --check 通过。
```

---

# 11. 验证命令

必须执行：

```bash
cargo fmt --check
cargo test -p agentflow-core
cargo test -p agentflow-cli
cargo test -p agentflow-graph
cargo test
npm --prefix apps/desktop run build
git diff --check
```

---

# 12. 交付说明要求

PR 描述必须包含：

```text
1. legacy-removal-audit.md 的分类结果。
2. 删除了哪些 unused legacy code。
3. 保留了哪些 active read model。
4. 保留了哪些 CLI legacy compatibility。
5. 是否删除了任何数据 writer。
6. 是否收窄了 legacy export。
7. 明确说明 Graph watcher fallback 未删除。
8. 明确说明 Project File Reader fallback 未删除。
9. 是否有任何行为变化。
10. 验证命令和结果。
```

---

# 13. Codex 执行指令

```md
请执行 005 - Legacy and Degraded Code Removal。

目标：
在 PR #8 已经完成 legacy 隔离和模块拆分后，做一轮安全删除。
先做引用审计，再删除无引用、无授权、无兼容价值的旧代码，并收窄 legacy 暴露面。

必须遵守：
1. 不新增 Goal Tree。
2. 不定义新的 Goal / Milestone / Issue。
3. 不定义新的 AgentRun。
4. 不启动 Agent。
5. 不调用模型。
6. 不执行项目命令。
7. 不修改用户项目源码。
8. 不改变 Tauri command 名称。
9. 不改变 Desktop 只读边界。
10. 不删除当前 Desktop 仍依赖的 active read model。
11. 不删除仍被 CLI legacy command 使用的代码。
12. 不删除 Graph watcher fallback。
13. 不删除 Project File Reader fallback。
14. 删除前必须先在 legacy-removal-audit.md 中分类。

执行步骤：
1. 新增 docs/architecture/legacy-removal-audit.md。
2. 扫描 crates/agentflow-core/src/legacy 下所有 pub symbols。
3. 将每个 symbol 分类为 active-read-model / cli-legacy / test-only / unused / uncertain。
4. 将 degraded code 分类为 keep / delete。
5. 删除明确 unused 的旧代码。
6. 删除无引用旧流程降级残留。
7. 收窄 legacy/mod.rs，移除 pub use archive_2026_05::*。
8. CLI legacy.rs 改为显式 import 所需 legacy 子模块。
9. active read model 改为显式 import 所需 compatibility 符号。
10. 更新 docs/architecture/legacy-code-map.md。
11. 更新 docs/architecture/current-module-boundaries.md。
12. 更新 verification.md。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-core
- cargo test -p agentflow-cli
- cargo test -p agentflow-graph
- cargo test
- npm --prefix apps/desktop run build
- git diff --check

交付说明必须说明：
- 删除了哪些旧代码。
- 哪些旧代码保留，为什么。
- 哪些 fallback 保留，为什么。
- 是否有行为变化。
- 验证结果。
```

---

# 14. 完成定义

本需求完成后，AgentFlow 应达到：

```text
旧代码不再宽导出
无引用旧代码被删除
旧流程降级残留被清理
当前 Desktop read model 仍可用
CLI legacy compatibility 仍可编译
Graph / Project File Reader fallback 被正确保留
下一阶段 Goal Tree 不再暴露在旧流程符号污染下
```

最终一句话：

> **005 不是继续隔离旧代码，而是开始安全删除旧代码。删除前必须有审计，删除后必须收窄 legacy 导出，确保新产品主干不会再误用旧流程。**
