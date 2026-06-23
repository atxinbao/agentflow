# Pack-aware Projection Read Models V1

创建日期：2026-06-23
执行者：Codex

## 目标

让 Projection 能按 Pack 暴露行业对象和行业视图，供 Desktop / CLI 查询 Pack readiness 和行业工作台结构。

本层只做只读投影，不写 `.agentflow/spec/**`、`.agentflow/events/**`、`.agentflow/tasks/**`，也不把 Pack 文件升级为 Runtime authority。

## Read Model

新增 Pack industry workbench read model：

```text
get_pack_industry_workbench_view(project_root, pack_id?)
```

返回：

- pack list；
- active pack；
- pack validation / readiness status；
- domain object index；
- surface page index；
- connector capability index；
- industry workbench index；
- projection freshness；
- source refs。

## Authority Boundary

Pack-aware Projection 只读以下来源：

```text
.agentflow/packs/**              Pack definition layer
agentflow-pack built-in baseline Software Dev / UI Design
.agentflow/events/**             freshness / state trace
.agentflow/projections/**        existing task / project read models
```

它不负责：

- 写 Pack 文件；
- 写 spec authority；
- 写 event store；
- 写 task artifacts；
- 调用 provider / MCP；
- 创建 command / action proposal。

## Runtime API Boundary

Desktop / CLI 通过 `agentflow-runtime-api` 查询：

```text
projection.pack-industry-workbench
  -> get_pack_industry_workbench_view
```

API Plane 把它登记为 `query`，不是 `pack_actions`。

含义：

- `pack_actions` 读取 / 校验 Pack 文件；
- `projection.pack-industry-workbench` 提供 UI / CLI 可消费的 Pack 读模型。

## Built-in Pack Coverage

V1 必须能解释两类 Pack：

```text
software-dev
  object: Issue / Run / PullRequest / Evidence / Release / Finding
  surface: project-home / spec-workbench / task-workbench / acceptance / delivery / audit
  connector: git / github / codex / claude / browser-preview

ui-design
  object: ProductBrief / Prd / Direction / Wireframe / HiFi / DesignSystem / Handoff
  surface: design-home / brief-intake / direction-board / wireframe-board / hifi-review / handoff
  connector: figma / image-assets / frontend-repo / design-export / browser-preview
```

如果项目没有 `.agentflow/packs/**` 文件，Projection 仍可展示 built-in baseline。

如果项目注册了 Pack manifest，Projection 会把 manifest path 和 validation status 加入 read model，但仍不把 manifest 当作任务事实 authority。

## Desktop / CLI Usage

Desktop 可以用该 read model 展示：

- 当前项目可用的 Pack；
- Pack readiness；
- 行业工作台入口；
- 当前 Pack 的对象 / 页面 / connector 能力。

CLI 可以用该 read model 做：

- pack readiness inspection；
- command surface 前的只读解释；
- release readiness summary。

## 验收

- Projection catalog 包含 `pack-industry-workbench`；
- `get_pack_industry_workbench_view` 能返回 Software Dev 和 UI Design 的不同对象、页面和 connector capability；
- 所有返回值都标记 `authority=false`；
- `agentflow-runtime-api` 暴露同名只读查询；
- API Plane 把该入口归类到 Projection Query。
