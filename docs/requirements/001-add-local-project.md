# 001 - Project Workspace Manager V0.2

创建日期：2026-06-01
执行者：Codex

## 用户目标

用户可以在 AgentFlow Desktop 中把一个本地文件夹接入为 Project。系统负责打开项目、准备本地 `.agentflow/` 运行目录、避免重复项目、支持从列表移除项目，并确保 `.agentflow/` 不进入 Git / PR。

第一版只做 Project Workspace Manager，不理解代码项目本身。

## 范围

- Add Project：用户选择一个现有本地文件夹。
- Open Project：被添加的项目出现在左侧 Project 列表，点击后打开项目文件阅读器。
- Prepare Workspace：如果项目没有 `.agentflow/`，创建本地运行目录；如果已有，则复用并补齐缺失目录。
- Deduplicate Project：同一路径重复添加时不新增重复项，只切换到已有项目。
- Remove Project：从 AgentFlow 列表移除项目，但不删除源码、不删除 `.agentflow/`。
- Protect `.agentflow` from Git / PR：如果项目是 Git 仓库，把 `.agentflow/` 写入 `.git/info/exclude`。
- 支持多个 Project。
- 即使项目目录没有旧 `.agentflow/`，也必须可以接入。

## 非目标

- 不检测技术栈。
- 不生成验证命令。
- 不创建技术栈文档。
- 不创建 Goal / Milestone / Issue。
- 不分析代码库。
- 不执行 run / verify / review。
- 不启动 Agent。
- 不调用模型。
- 不接 OpenSpec / Superpowers / gstack。
- 不创建远程 PR、GitHub issue 或 Linear issue。
- 不做云同步、账号、支付或 SaaS 能力。

## 页面 / 功能

- Sidebar:
  - “所有项目”标题右侧保留添加按钮。
  - 添加按钮在 Tauri 桌面环境打开本地文件夹选择器。
  - 添加按钮在浏览器预览环境展开路径输入面板。
  - 添加成功后项目出现在 Project 列表，并自动高亮和展开。
  - 本地添加的 Project 可以从列表移除；移除只影响客户端列表，不删除任何文件。
- Project 主体:
  - 对 `.agentflow/` 派生 Project，继续按现有项目读取逻辑展示。
  - 对用户添加的本地 Project，直接展示本地文件阅读器。
  - 当前选中 Project 的路径成为文件阅读器的根路径。
- 持久化:
  - 使用浏览器 / WebView 的 `localStorage` 保存已添加的本地 Project 列表。
  - 只保存项目路径、显示名称和轻量 UI 状态，不保存运行态数据。

## `.agentflow/` 目录结构

添加项目时创建或复用以下结构：

```text
.agentflow/
├── workspace.yaml
├── config.yaml
├── define/
│   ├── goals/
│   ├── milestones/
│   └── issues/
├── execute/
│   ├── leases/
│   ├── runs/
│   └── events/
└── output/
    ├── evidence/
    ├── logs/
    ├── cache/
    └── tmp/
```

规则：

- 有就复用。
- 缺就补齐。
- 不覆盖用户已有 `workspace.yaml` / `config.yaml`。
- 不写 Goal / Milestone / Issue 内容。
- `.agentflow/` 是本地运行态目录，不进入 Git / PR。

## 数据来源

- Tauri command: `choose_existing_project_folder`
- Tauri command: `prepare_local_project_workspace`
- Tauri command: `load_project_files_snapshot`
- Tauri command: `load_project_file_content`
- Client local state: `agentflow.localProjectFolders.v1`

## 交互边界

- Desktop 当前仍是只读客户端。
- 添加项目允许写入被选择项目下的 `.agentflow/` 本地运行目录。
- 添加项目允许写入 `.git/info/exclude`，只用于排除 `.agentflow/`。
- 除 `.agentflow/` 和 `.git/info/exclude` 外，不写项目其他文件。
- 选择文件夹后只读取文件树和文件内容。
- 不执行命令。
- 不写 `.codex/`。
- 不写 `graphify-out/`。

## 验收标准

- [ ] 删除 `.agentflow/` 后，Desktop 仍可通过“添加项目”添加一个本地目录。
- [ ] 添加后的 Project 出现在左侧 Project 列表。
- [ ] 添加后的 Project 自动成为当前选中 Project。
- [ ] 右侧 Project 页面展示该目录的文件阅读器，而不是空项目提示。
- [ ] 重复添加同一路径不会重复出现 Project，只会切换到已有 Project。
- [ ] 添加项目会创建或复用 `.agentflow/`。
- [ ] 添加项目会创建或复用 `workspace.yaml` 和 `config.yaml`。
- [ ] 添加项目会创建或复用 `define/`、`execute/`、`output/` 三阶段目录。
- [ ] Git 项目会把 `.agentflow/` 加入 `.git/info/exclude`。
- [ ] 移除项目不会删除源码，也不会删除 `.agentflow/`。
- [ ] 浏览器预览模式下可以通过显式 mock 项目和 mock 文件树验证 UI，但不写入真实工作区。
- [ ] 添加行为不会执行命令、调用模型或创建远程对象。

## 验证命令

- `npm --prefix apps/desktop run build`
- `cargo test`
- `git diff --check`
