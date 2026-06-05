
# AgentFlow 代码清理开发需求文档（Rust workspace 全局）

## 1. 项目范围

- `crates/agentflow-core`：核心状态模型、IssueContract、AgentRun、Workbench、Metrics、Project/Milestone/Closure 等。
- `crates/agentflow-cli`：CLI 命令，包括 goal/feature/team/milestone/issue/run/verify/review/index/project/search。
- `crates/graph`：现场资源索引 / 图谱处理。
- `apps/desktop`：Tauri Desktop 前端应用（本轮清理只关注 Rust 后端逻辑，不清理 UI 组件）。

## 2. 清理目标

1. **去掉遗留旧需求相关代码**
   - 旧 CLI 命令（如 goal/feature/milestone/issue/run/verify/review 中已废弃的命令）。
   - 已降级或废弃的 workflow 模块。
   - 未使用的 helper / util 函数。

2. **移除冗余结构**
   - 不再被任何模块引用的结构体、枚举和 trait。
   - 重复或多余的模型定义（尤其是 Project/Milestone/Issue 的旧版本）。

3. **优化模块边界**
   - 将 core 功能与 CLI 功能明确分离。
   - 将 graph 功能单独保持，避免混入 core 逻辑。

4. **确保依赖安全**
   - 清理后依赖仍然完整，所有 Cargo.toml 依赖保持最小化且可编译。

5. **保留核心功能闭环**
   - Workflow 状态机（Project → Milestone → Issue → Lease → Execution → Evidence → Audit）必须完整。
   - Agent 执行逻辑（AgentRun / Lease / Run / Checkpoint / Command / Validation / Result）。
   - Panel / Graph / State 模块功能完整。

## 3. 清理优先级和顺序

1. **分析遗留 CLI 和 workflow**
   - 标记所有未使用命令。
   - 检查与现有 AgentFlow V16 流程不匹配的命令。

2. **删除冗余模型**
   - 对比现有 core/lib.rs 1.4 万行代码。
   - 移除重复 Project/Milestone/Issue 结构、旧 Metrics、Workbench 中未使用的字段。

3. **优化 graph 模块**
   - 清理不必要的 indexing helper。
   - 保留生成 panel/output/graph 的核心逻辑。
   - 修正模块内依赖边界。

4. **整理 CLI 命令**
   - 保留与当前流程匹配的命令：goal/feature/team/milestone/issue/run/verify/review/index/project/search。
   - 删除废弃命令和重复逻辑。
   - 检查 CLI 模块调用 core 模块的边界，保证 clean dependency。

5. **删除无用文件和测试**
   - 移除不再使用的测试文件、样例数据、脚本。
   - 保留必要的 integration test、unit test。

6. **编译检查与 CI**
   - 清理后确保 workspace 可正常编译。
   - CI pipeline 仍能通过所有单元测试。

## 4. 风险控制

- **核心功能必须完整**：Project → Milestone → Issue → Agent → Execution → Evidence → Audit 的流程不能断。
- **Agent 执行逻辑必须保留**：Lease、Run、Command、Validation、Patch、Result。
- **Graph / Panel 数据流必须保留**：生成 panel/output/graph 的能力。
- **不要触碰前端逻辑**：apps/desktop/src/** 文件除非与 Rust workspace 强绑定，否则不修改。

## 5. 清理产物

- 更新后的 `crates/agentflow-core`、`crates/agentflow-cli`、`crates/graph`。
- 清理后的 Cargo.toml 与 workspace 编译可用。
- 删除冗余模型、命令、旧 workflow。
- 保持现有单元测试和集成测试通过。

## 6. 后续工作

- 清理完成后，再进入 **前端项目页面/UX 设计对应实现**。
- 确保前端调用 core / graph API 时，接口不被破坏。
