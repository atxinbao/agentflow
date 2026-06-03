# Next Requirements

创建日期：2026-06-01
执行者：Codex

## 状态

已确认的新功能需求：

- [001-add-local-project.md](001-add-local-project.md)
- [002-graph-v1.md](002-graph-v1.md)
- [002-1-graph-v1-completion.md](002-1-graph-v1-completion.md)
- [002-2-graph-v1-os-native-watcher-closeout.md](002-2-graph-v1-os-native-watcher-closeout.md)
- [003-project-file-reader-v1-completion.md](003-project-file-reader-v1-completion.md)
- [004-legacy-cleanup-and-new-module-split.md](004-legacy-cleanup-and-new-module-split.md)
- [005-legacy-and-degraded-code-removal.md](005-legacy-and-degraded-code-removal.md)
- [006-legacy-cli-retirement-and-archive-pruning.md](006-legacy-cli-retirement-and-archive-pruning.md)
- [007-goal-tree-v1.md](007-goal-tree-v1.md)
- [007-1-goal-tree-agent-only-boundary-fix.md](007-1-goal-tree-agent-only-boundary-fix.md)
- [008-agent-working-manual-bootstrap-v1.md](008-agent-working-manual-bootstrap-v1.md)
- [008-1-agent-working-manual-health-polish.md](008-1-agent-working-manual-health-polish.md)
- [008-2-requirement-intake-filter-skill-v1.md](008-2-requirement-intake-filter-skill-v1.md)

后续新需求继续写入本文件或新增 `00N-*.md`。

## 背景

项目已完成旧文档清理。后续开发与 2026-05 的旧需求、旧规划、旧规格无继承关系。

## 新需求模板

```md
# <需求名称>

## 用户目标
<用户最终要完成什么>

## 范围
- 

## 非目标
- 

## 页面 / 功能
- 

## 数据来源
- 

## 交互边界
- 

## 验收标准
- [ ] 

## 验证命令
- `npm --prefix apps/desktop run build`
- `cargo test`
- `git diff --check`
```

## 当前不授权

- 不授权继续旧 roadmap。
- 不授权继续旧 workflow / product feature / closure 方向。
- 不授权从归档 specs 自动生成任务。
