# 025 - AgentFlow Project Tree Multi-Project Navigation V1

> 建议保存路径：`docs/requirements/025-agentflow-project-tree-multi-project-navigation-v1.md`  
> 类型：前端基础能力 / 多项目导航 / AppShell 修复 / Base Release 前置需求  
> 状态：Ready for Codex implementation  
> 目标版本：`v0.1.0-base`

---

## 1. 背景

当前 AgentFlow 左侧栏的问题不是简单样式问题，而是产品结构和数据结构都还没有补齐。

现在前端更像：

```text
当前项目
├── 工作台
├── 任务
├── 文件
├── 交付
├── 审计
└── 高级
```

但 AgentFlow 的产品心智应该是：

```text
所有项目
├── my-web-app
│   ├── 工作台
│   ├── 任务
│   ├── 文件
│   ├── 交付
│   ├── 审计
│   └── 高级
├── agentflow
└── mobile-app
```

也就是说，左侧栏应该是 **多项目 Tree**，不是单项目菜单。

---

## 2. 当前问题

### 2.1 只能展示一个项目

当前 App 状态主要依赖单个 `projectRoot`：

```text
agentflow.interaction.projectRoot.v1
```

这意味着 App 只能稳定记住一个项目。

缺少：

```text
projects[]
activeProjectRoot
recentProjects
expandedProjectRoots
activePageByProject
```

### 2.2 ProjectTree 只接收一个 projectName

当前 ProjectTree 的输入心智是：

```text
activePage
projectName
onPageChange
```

它没有：

```text
projects
activeProjectRoot
onAddProject
onSelectProject
onToggleProject
```

所以它只能展示一个项目。

### 2.3 左侧栏目缺少父子关系

当前左侧栏目是直接展示页面：

```text
工作台 / 任务 / 文件 / 交付 / 审计 / 高级
```

而不是展示：

```text
项目
└── 页面
```

这样会让用户觉得 AgentFlow 是单项目工具，不像多项目桌面客户端。

---

## 3. 目标

把左侧栏从 **单项目菜单** 升级成 **多项目 Tree 导航**。

完成后用户可以：

```text
1. 添加多个本地项目
2. 在左侧看到多个项目
3. 展开 / 收起每个项目
4. 点击项目切换当前项目
5. 点击项目下面的栏目切换页面
6. 每个项目记住上次打开的页面
7. 工作台、任务、文件、交付、审计、高级都跟随 activeProjectRoot 加载数据
8. 底部状态栏跟随当前项目变化
```

---

## 4. 非目标

本需求不做：

```text
1. 不做云同步项目列表
2. 不做多账号项目管理
3. 不做远程仓库导入
4. 不做 GitHub 项目选择器
5. 不做项目删除真实文件
6. 不做项目分组 / 收藏 / 标签
7. 不做团队权限
8. 不改 Rust 后端状态机
9. 不改变 .agentflow 数据结构
```

---

## 5. 产品设计要求

### 5.1 左侧栏结构

左侧栏最终结构：

```text
+ 添加项目

所有项目
▾ my-web-app       ●
  工作台
  任务
  文件
  交付
  审计
  高级

▸ agentflow       ●
▸ mobile-app      ○

底部：
本地模式
只读工作台
```

### 5.2 项目状态点

每个项目右侧或左侧显示一个小状态点。

状态：

```text
ready       = 已就绪
loading     = 正在读取
blocked     = 有阻断
error       = 读取失败
missing     = 路径不存在
```

普通用户不要看到内部状态名。悬停提示用中文：

```text
已就绪
正在读取
有阻断
读取失败
项目路径不存在
```

### 5.3 展开 / 收起

点击箭头：

```text
▸ / ▾
```

只负责展开 / 收起，不切换页面。

点击项目名：

```text
切换当前项目
```

点击栏目：

```text
切换当前项目 + 切换页面
```

### 5.4 添加项目

点击：

```text
+ 添加项目
```

行为：

```text
1. 调用现有 choose_existing_project_folder
2. 调用 prepare_local_project_workspace
3. 加入 Project Registry
4. 设为 activeProjectRoot
5. 展开该项目
6. 进入该项目工作台
7. 触发 Base Release 初始化检查
```

### 5.5 多项目滚动

当项目超过可见区域时：

```text
顶部“添加项目”固定
中间项目 Tree 滚动
底部本地模式信息固定
```

不要让整个侧栏一起滚走。

---

## 6. 数据模型

### 6.1 新增 AgentFlowProjectRef

建议新增类型：

```ts
export type AgentFlowProjectStatus =
  | "ready"
  | "loading"
  | "blocked"
  | "error"
  | "missing";

export type AgentFlowProjectRef = {
  id: string;
  name: string;
  root: string;
  kind: "local";
  status: AgentFlowProjectStatus;
  lastOpenedAt: number;
  createdAt: number;
  expanded: boolean;
  lastActivePage: AppPage;
  error?: string | null;
};
```

### 6.2 本地存储 key

建议使用：

```text
agentflow.projects.v1
agentflow.activeProjectRoot.v1
agentflow.expandedProjectRoots.v1
agentflow.activePageByProject.v1
```

### 6.3 向后兼容

当前已有：

```text
agentflow.interaction.projectRoot.v1
agentflow.interaction.activePage.v1
```

迁移规则：

```text
1. 如果 agentflow.projects.v1 不存在
2. 但 agentflow.interaction.projectRoot.v1 存在
3. 自动创建一个 AgentFlowProjectRef
4. 写入 agentflow.projects.v1
5. 设置 activeProjectRoot
6. 保留旧 key，不强制删除
```

---

## 7. 前端状态结构

### 7.1 当前应替换的状态

从：

```ts
const [projectRoot, setProjectRoot] = useState<string | null>(...)
const [activePage, setActivePage] = useState<AppPage>(...)
```

升级为：

```ts
const [projects, setProjects] = useState<AgentFlowProjectRef[]>(...)
const [activeProjectRoot, setActiveProjectRoot] = useState<string | null>(...)
const [activePage, setActivePage] = useState<AppPage>(...)
const [expandedProjectRoots, setExpandedProjectRoots] = useState<Set<string>>(...)
const [activePageByProject, setActivePageByProject] = useState<Record<string, AppPage>>(...)
```

### 7.2 派生当前项目

```ts
const activeProject = projects.find((project) => project.root === activeProjectRoot) ?? null;
const projectRoot = activeProject?.root ?? null;
```

后续 hooks 仍然可以使用 `projectRoot`，但它必须来自 `activeProjectRoot`。

---

## 8. ProjectTree 组件接口

### 8.1 新接口

将 ProjectTree 从单项目 props 改为多项目 props：

```ts
type ProjectTreeProps = {
  projects: AgentFlowProjectRef[];
  activeProjectRoot: string | null;
  activePage: AppPage;
  expandedProjectRoots: Set<string>;
  onAddProject: () => void;
  onSelectProject: (projectRoot: string) => void;
  onToggleProject: (projectRoot: string) => void;
  onPageChange: (projectRoot: string, page: AppPage) => void;
};
```

### 8.2 组件行为

```text
onAddProject
= 添加新项目

onSelectProject
= 切换 activeProjectRoot，并恢复该项目 lastActivePage

onToggleProject
= 展开 / 收起项目

onPageChange
= 设置 activeProjectRoot + activePage + activePageByProject[root]
```

---

## 9. 页面数据加载规则

### 9.1 切换项目时必须刷新

切换 activeProjectRoot 后，需要刷新：

```text
project files
agent manual
project panel
input status
input snapshot
execute status
output status
state status
issue-status index
workspace data
output bundle
```

### 9.2 切换页面时按需刷新

```text
工作台：
prepareProjectPanel
load state / output summary

任务：
prepareProjectPanel
load input snapshot
load issue-status index

文件：
loadProjectFiles

交付：
load output index
load delivery

审计：
load audit index
load audit report

高级：
load state / panel / input / execute / output snapshots
```

### 9.3 项目路径不存在

如果项目 root 不存在：

```text
项目状态 = missing
页面显示：
项目路径不存在，请重新添加项目或从列表中移除。
```

本需求不要求实现“移除项目”，但可以预留按钮。

---

## 10. Project Registry 操作

### 10.1 添加项目

伪代码：

```ts
async function addProject() {
  const selectedRoot = await invoke<string | null>("choose_existing_project_folder");
  if (!selectedRoot) return;

  const root = normalizeProjectRootKey(selectedRoot);
  await invoke("prepare_local_project_workspace", { projectRoot: root, appLocale: detectAppLocale() });

  upsertProject({
    id: stableProjectId(root),
    name: projectNameFromPath(root),
    root,
    kind: "local",
    status: "ready",
    expanded: true,
    lastActivePage: "home",
    createdAt: now(),
    lastOpenedAt: now(),
  });

  setActiveProjectRoot(root);
  setActivePage("home");
}
```

### 10.2 去重

如果用户添加同一个项目：

```text
不重复创建
更新 lastOpenedAt
切换为 activeProject
展开项目
进入工作台
```

### 10.3 最近项目数量

Base Release 建议最多保留：

```text
10 个项目
```

超出后按 `lastOpenedAt` 淘汰最旧项。

如果担心误删项目记录，可以先不自动淘汰，只保留 UI 滚动。

---

## 11. App 启动规则

### 11.1 启动时读取 Project Registry

启动流程：

```text
1. 读取 agentflow.projects.v1
2. 读取 activeProjectRoot
3. 如果没有 projects，但旧 projectRoot 存在，迁移旧项目
4. 如果有 activeProjectRoot，进入该项目 lastActivePage
5. 如果没有 activeProjectRoot，但有 projects，选 lastOpenedAt 最新项目
6. 如果没有任何项目，显示空项目工作台 / 添加项目入口
```

### 11.2 不再被登录 / 首次引导阻断

配合 Base Release：

```text
不要求 providerConnected
不要求 onboardingComplete
```

---

## 12. 空态设计

### 12.1 没有项目

左侧：

```text
+ 添加项目
所有项目
```

项目区留空，不展示“暂无项目”文字。

主内容：

```text
还没有项目
添加一个本地项目后，AgentFlow 会准备任务、文件、交付和审计工作区。
[添加项目]
```

### 12.2 项目加载中

```text
正在读取项目
正在准备 AgentFlow 工作区。
```

### 12.3 项目读取失败

```text
项目读取失败
请检查项目路径是否还存在，或重新添加项目。
```

### 12.4 项目路径不存在

```text
项目路径不存在
这个项目可能被移动或删除了。
```

### 12.5 所有项目被移除后的 Empty Registry

当用户把所有项目都从侧边栏移除后，AgentFlow 进入正常的空项目工作台状态：

```text
Empty Project Registry
```

这不等于：

```text
App 出错
重新登录
重新首次引导
删除本地项目文件
```

处理规则：

```text
projects = []
activeProjectRoot = null
activePage = home
expandedProjectRoots = []
activePageByProject = {}
selectedTaskId = null
selectedDeliveryRunId = null
selectedAuditId = null
taskSearch = ""
```

旧 key 不能让已移除项目在刷新后复活：

```text
agentflow.interaction.projectRoot.v1
```

如果 `agentflow.projects.v1` 已存在，即使它是空数组，也必须优先使用新 registry，不再从旧 projectRoot 迁移。

左侧栏显示：

```text
+ 添加项目
所有项目
```

项目区留空，不展示“暂无项目”文字。

主内容区显示：

```text
还没有项目
添加一个本地项目后，AgentFlow 会准备任务、文件、交付和审计工作区。
移除项目不会删除你的本地文件。
[添加本地项目]
```

顶部栏显示：

```text
AgentFlow
未选择项目 · 本地模式
⌘K
```

底部状态栏显示：

```text
未选择项目 · 本地模式 · ⌘K
```

验收：

```text
1. 移除最后一个项目后不报错。
2. 左侧只显示“添加项目 / 所有项目”，项目区留空。
3. 主区域显示“添加本地项目”空态。
4. 顶部不残留旧项目名。
5. 底部不残留旧工作流状态。
6. selectedTask / delivery / audit 状态清空。
7. 刷新 App 后仍保持空项目状态。
8. 再添加项目后能正常进入工作台。
```

---

## 13. 工作台联动

工作台顶部项目名、底部状态栏、主内容必须跟随 activeProjectRoot。

不要出现：

```text
左侧选中 agentflow
右侧还显示 my-web-app 数据
```

必须保证：

```text
切换项目
→ 页面显示 activeProject 对应数据
→ 状态栏显示 activeProject
→ 文件页读取 activeProject
→ 任务页读取 activeProject
```

---

## 14. Browser Preview 要求

Browser Preview 至少 mock 3 个项目：

```text
my-web-app
agentflow
mobile-app
```

其中：

```text
my-web-app = active + expanded
agentflow = collapsed
mobile-app = collapsed
```

这样能验证：

```text
1. 多项目展示
2. 展开 / 收起
3. 项目切换
4. 页面菜单层级
```

---

## 15. CSS / 视觉要求

### 15.1 ProjectTree 行高

```text
项目行：28px
栏目行：24px
```

### 15.2 缩进

```text
项目层级：16px
栏目层级：32px / 44px
```

### 15.3 状态点

```text
ready = green
loading = blue
blocked = orange
error = red
missing = gray
```

### 15.4 不要做

```text
不要项目卡片化
不要大圆角
不要重阴影
不要把状态详情塞进左侧
```

---

## 16. 测试要求

### 16.1 单元测试

建议测试：

```text
1. readProjectRegistry
2. writeProjectRegistry
3. migrateLegacyProjectRoot
4. upsertProject dedupe
5. activePageByProject restore
6. expandedProjectRoots persist
```

### 16.2 交互测试 / 手动验收

```text
1. 空项目时显示添加项目入口。
2. 添加第一个项目后，左侧出现项目和栏目。
3. 再添加第二个项目，两个项目都显示。
4. 点击第二个项目，主内容切换到第二个项目。
5. 收起第一个项目后，栏目隐藏。
6. 展开第一个项目后，栏目恢复。
7. 每个项目记住上次打开的页面。
8. 刷新 App 后，项目列表和 activeProject 恢复。
9. Browser Preview 能看到 3 个 mock 项目。
```

---

## 17. 验收标准

必须满足：

```text
1. 左侧可以展示多个项目。
2. 每个项目下面都有 工作台 / 任务 / 文件 / 交付 / 审计 / 高级。
3. 点击项目可以切换 activeProject。
4. 点击栏目可以切换 activeProject + activePage。
5. 项目展开 / 收起可用。
6. 添加项目不会重复添加同一路径。
7. 每个项目能记住上次打开页面。
8. 底部状态栏跟随 activeProject。
9. 主内容区不会显示上一个项目的数据。
10. Browser Preview 有多项目 mock。
11. TypeScript build 通过。
12. 不改 Rust 后端状态机。
```

---

## 18. Codex 实现指令

```text
你现在只做这个任务：AgentFlow Project Tree Multi-Project Navigation V1。

背景：
当前 AgentFlow 左侧栏只能展示一个项目，本质原因是前端只有单一 projectRoot 状态。AgentFlow 需要在 Base Release 前支持多项目 Tree 导航。

目标：
把左侧栏从“单项目菜单”升级为“多项目 Tree 导航”。

范围：
- 只改 apps/desktop/src/**
- 可以新增 project registry 工具
- 可以更新 browserPreviewData
- 不改 Rust 后端状态机
- 不改 .agentflow 数据结构

步骤：
1. 新增 AgentFlowProjectRef 类型。
2. 新增 Project Registry localStorage 工具。
3. 迁移旧 projectRoot 到 projects[]。
4. 将 App 状态从 projectRoot 升级为 projects + activeProjectRoot。
5. 改造 ProjectTree props 和渲染结构。
6. 实现添加项目、选择项目、展开收起、页面切换。
7. 让所有页面 hooks 使用 activeProjectRoot 派生出的 projectRoot。
8. 更新 StatusBar 和 TitleBar 显示 activeProject。
9. Browser Preview mock 3 个项目。
10. 跑 TypeScript build。

禁止：
- 不要接云端项目。
- 不要做项目删除真实文件。
- 不要改 Rust 后端状态机。
- 不要恢复登录 / 首次引导阻断。
- 不要让左侧栏塞大量状态详情。
- 不要每个项目重复渲染完整页面内容，只渲染导航树。

验证：
- npm --prefix apps/desktop run build
- Browser Preview 可看到多项目 Tree
- 手动切换项目和页面可用

输出：
- 改了哪些文件
- Project Registry 存储格式
- 旧 projectRoot 如何迁移
- 多项目导航如何验证
- 仍未完成的风险
```
