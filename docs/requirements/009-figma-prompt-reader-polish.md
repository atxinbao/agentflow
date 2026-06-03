# Figma Prompt Reader Polish

创建日期：2026-06-03
执行者：Codex

## 用户目标

用户提供了一份“机构级量化交易系统”Figma AI 高保真原型提示词，希望 AgentFlow 能把这类产品 / 设计提示词作为项目文件稳定阅读，并先形成需求文档，再做小范围修复。

## 背景

本次输入来自用户粘贴文本，核心内容是一份面向 Figma AI 的机构级量化交易系统原型生成提示词，包含：

- 产品定位：机构级量化交易平台，而不是零售交易 App。
- 目标用户：量化研究员、交易员、投资经理、风控、执行团队、数据工程师、管理员、CIO / COO。
- 设计方向：深色、高信息密度、投行 / 对冲基金内部工具风格。
- 输出范围：Design System、左侧导航、顶部状态栏、14 个业务页面、弹窗、Drawer、Prototype Flow。
- 关键约束：不要营销页、不要低密度 Dashboard、不要 crypto 赌场感视觉、不要 lorem ipsum。

## 两个小问题

### 问题 1：设计提示词需求没有进入项目需求入口

用户输入仍停留在外部粘贴文本中，不属于 `docs/requirements/` 下的新需求入口。后续开发如果要围绕该内容继续推进，需要先把它沉淀为项目需求文档。

### 问题 2：文本提示词文件在阅读器中容易被误标为代码

Project File Reader 已能读取 Markdown、代码、JSON、配置和普通文本，但 `.txt` 或提示词类文本在文件头部可能显示成“代码”，这会让设计提示词、产品提示词、长文本需求的阅读心智不准确。

## 范围

- 新增本需求文档。
- 更新 requirements 索引。
- 修正 Project File Reader 的文本类文件展示标签。
- Browser Preview mock 数据中增加本需求文档和提示词文本样例，方便浏览器环境验证。

## 非目标

- 不调用 Figma。
- 不生成 Figma 文件。
- 不创建远程设计项目。
- 不实现机构级量化交易系统。
- 不新增模型调用。
- 不写 `.agentflow/` 运行态数据。
- 不改变 Desktop 只读边界。

## 页面 / 功能

- Project File Reader
- Browser Preview mock file tree
- Requirements docs

## 数据来源

- 用户粘贴文本：`/Users/mac/.codex/attachments/c32fcfab-012d-4961-b1bb-3fbbd73e2ad1/pasted-text.txt`
- 项目需求入口：`docs/requirements/`

## 交互边界

- Desktop 只读取并展示文件。
- Browser Preview 只展示 mock 数据。
- 点击文件只加载内容，不执行命令。
- 不写入工作区运行态。

## 验收标准

- [ ] `docs/requirements/009-figma-prompt-reader-polish.md` 存在，并说明本次两个小问题。
- [ ] `docs/requirements/README.md` 和 `docs/requirements/next-requirements.md` 包含本需求。
- [ ] `.txt` / 普通文本类内容不再显示为“代码”，而显示为“文本”或“提示词文档”。
- [ ] Browser Preview mock 文件树可看到本需求文档和提示词文本样例。
- [ ] Desktop build 通过。

## 验证命令

- `npm --prefix apps/desktop run build`
- `cargo test`
- `git diff --check`
