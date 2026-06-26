# AgentFlow

更新日期：2026-06-26
执行者：Codex

AgentFlow 是本地优先的 Agent 项目运行时和桌面工作台。

## 文档入口

- [docs/README.md](docs/README.md)
- [docs/product/006-spec-driven-software-dev-product-goal-v1.md](docs/product/006-spec-driven-software-dev-product-goal-v1.md)
- [docs/foundation/021-ai-os-project-core-capabilities-v1.md](docs/foundation/021-ai-os-project-core-capabilities-v1.md)
- [docs/v1.0.1/README.md](docs/v1.0.1/README.md)
- [docs/requirements/README.md](docs/requirements/README.md)
- [design.md](design.md)
- [CHANGELOG.md](CHANGELOG.md)

## 当前边界

- `docs/requirements/**` 是 confirmed Spec Bundle 的公共记录入口。
- `docs/product/**` 是产品和设计基线。
- `docs/foundation/**` 是 AI OS Project Core 底层能力设计。
- `.agentflow/spec/**` 是 project / issue 合同事实源。
- `.agentflow/events/**` 是任务事件流。
- `.agentflow/tasks/<issue-id>/**` 保存 run 与 evidence。
- Desktop 只读 Projection，不直接写运行事实。
- Build Agent 和 Audit Agent 是独立流程。
- 旧 `GOAL.md`、`ROADMAP.md` 和根 `verification.md` 已退出根入口。
- 根 `design.md` 只保留兼容入口，完整正文迁入 `docs/product/design-system.md`。
- 旧版本计划、旧 flat requirements 和旧 pre-v1 文档已归档到 `docs/archive/2026-06-current-baseline-history/`。

## 开发命令

```bash
npm --prefix apps/desktop run build
npm --prefix apps/desktop run dev
cargo test --workspace
git diff --check
```

历史验证记录见 [docs/verification/history.md](docs/verification/history.md)。
