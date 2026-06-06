# AgentFlow 本地 AGENTS.md 文件管理规范

## 1. 目标
- 保证 `AGENTS.md` 文件本地存在、可用，供 AgentFlow 使用。
- 避免该文件被 Git 跟踪或提交。
- 在 rebase、切换分支或拉取远程更新时，确保本地修改不丢失。
- 保证团队成员操作一致，降低冲突和误操作风险。

## 2. 文件位置
- 根目录下：`./AGENTS.md`

## 3. 文件生成
- 当项目初始化或打开时，如果根目录不存在 `AGENTS.md`：
  - 系统自动生成默认内容，包括：
    - Agent 规则
    - 技能默认配置
    - 输出风格
- 若文件已存在，保留当前内容，不覆盖。

## 4. Git 忽略
- `.gitignore` 文件中加入：
```gitignore
# 忽略本地 AgentFlow Agent 定义文件
AGENTS.md
```
- 确保 Git 不会跟踪或提交该文件。

## 5. 本地修改处理
- 在进行 `rebase`、`merge`、`pull` 等操作前，临时保存本地修改：
```bash
git stash push -m "stash AGENTS.md" AGENTS.md
```
- 完成操作后恢复：
```bash
git stash pop
```

## 6. 建议操作流程
1. 打开项目 -> 系统生成或读取 `AGENTS.md`
2. 编辑本地 `AGENTS.md` 内容
3. 修改完成后，确认 `.gitignore` 已忽略该文件
4. 执行 Git 更新操作（rebase、pull、切分支）前：
   - 临时 stash 文件
5. 完成 Git 操作后：
   - 恢复 stash
6. 持续使用 AgentFlow 时：
   - 文件保持本地状态
   - 系统可随时更新内容，不影响版本库

## 7. 注意事项
- 永远不要将 `AGENTS.md` 手动添加到版本控制中。
- 本地修改只在 AgentFlow 运行环境有效。
- 该文件用于 AgentFlow 内部工作，不作为团队协作提交的内容。
- 团队成员必须统一操作流程，避免冲突.
