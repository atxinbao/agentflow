# Docs Index

更新日期：2026-07-03
执行者：Codex

## Current Standard

AgentFlow 当前按新建项目标准组织 `docs/`：

```text
docs/
  project/       # 定义项目
  requirements/  # 定义确认后的 Spec
  architecture/  # 沉淀长期架构
  delivery/      # 记录交付结果
```

## Current Entries

| 路径 | 作用 |
| --- | --- |
| [project/README.md](project/README.md) | 项目目标、路线图、上下文、术语和历史上下文 |
| [requirements/README.md](requirements/README.md) | confirmed Spec Bundle 公共记录入口 |
| [architecture/README.md](architecture/README.md) | AI OS Project Core、filesystem-first 边界和 v1.x 稳定架构合同 |
| [delivery/README.md](delivery/README.md) | 当前 release baseline、交付记录和发布认证入口 |

## Default Reading Order

1. [../README.md](../README.md)
2. [project/goal.md](project/goal.md)
3. [project/roadmap.md](project/roadmap.md)
4. [project/context.md](project/context.md)
5. [architecture/021-ai-os-project-core-capabilities-v1.md](architecture/021-ai-os-project-core-capabilities-v1.md)
6. [architecture/builtin-pack-registry.md](architecture/builtin-pack-registry.md)
7. [architecture/086-industry-product-source-boundary-v1.md](architecture/086-industry-product-source-boundary-v1.md)
8. [architecture/041-v100-stable-contract-baseline-v1.md](architecture/041-v100-stable-contract-baseline-v1.md)
9. [architecture/053-core-4d-spec-intake-kernel-v1.md](architecture/053-core-4d-spec-intake-kernel-v1.md)
10. [delivery/releases/v1.1.5/README.md](delivery/releases/v1.1.5/README.md)
11. [requirements/README.md](requirements/README.md)
12. [requirements/v0.18.0-core-4d-spec-intake/spec-bundle.md](requirements/v0.18.0-core-4d-spec-intake/spec-bundle.md)
13. [../CHANGELOG.md](../CHANGELOG.md)

## Rules

- `docs/project/**` 定义项目，不直接授权实现。
- `docs/project/roadmap.md` 定义目标到版本的路线，不直接拆 task。
- `docs/requirements/**` 只保存 confirmed Spec Bundle。
- `docs/architecture/**` 保存长期架构、Core 能力和稳定合同，不直接授权实现。
- `docs/delivery/**` 记录 release、handoff、decision record 和交付结果。
- `products/**` 是 Industry Product Surface / Reference App source。
- 内置 Pack 是 App / Product 能力，不进入 `docs/project/**`；项目只记录当前启用的 Pack 引用。
- `crates/pack/fixtures/**` 只保留为测试夹具，不能作为正式行业壳源码位置。
- 后续新开发必须先进入 confirmed Spec Bundle，再派生 `.agentflow/spec/**` 执行合同。

历史文档只保留在 [project/history/](project/history/README.md)，不作为新建项目默认目录。
