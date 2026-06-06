# 技术约束

- 只写入 `.agentflow/**` 运行事实、需求文档和索引。
- 不删除用户源码、不删除 Git 仓库、不调用模型。
- `AGENTS.md` 由本地生成并加入忽略；如果已经被 Git 跟踪，只提示用户手动 `git rm --cached AGENTS.md`。
- Browser Preview mock 只能保留在 `apps/desktop/src/browserPreviewData.ts`，真实客户端不能静默回退到 mock。
